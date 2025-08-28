import { Router } from 'express';
import { z } from 'zod';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { WebSocketService } from '../services/websocket.js';
import { AuthenticatedRequest, requireAuth } from '../middleware/auth.js';
import { TenantRequest, requireTenant } from '../middleware/tenant.js';
import { createRateLimitMiddleware, rateLimitConfigs } from '../middleware/rateLimit.js';
import { asyncHandler, BFFError } from '../middleware/errorHandler.js';
import {
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  RegisterResponse,
  RefreshTokenRequest,
  RefreshTokenResponse,
  PasswordResetRequest,
  PasswordResetResponse,
  PasswordUpdateRequest,
  TenantSwitchRequest,
  TenantSwitchResponse,
  UserProfileUpdateRequest,
  MfaSetupRequest,
  MfaSetupResponse,
  MfaVerifyRequest,
  WorkflowResponse,
} from '../types/auth.js';

// Validation schemas
const loginSchema = z.object({
  email: z.string().email('Invalid email format'),
  password: z.string().min(1, 'Password is required'),
  tenantId: z.string().optional(),
  deviceInfo: z.object({
    type: z.enum(['web', 'mobile', 'desktop']),
    os: z.string().optional(),
    browser: z.string().optional(),
    version: z.string().optional(),
  }).optional(),
  rememberMe: z.boolean().optional(),
});

const registerSchema = z.object({
  email: z.string().email('Invalid email format'),
  password: z.string().min(8, 'Password must be at least 8 characters'),
  firstName: z.string().optional(),
  lastName: z.string().optional(),
  tenantName: z.string().optional(),
  inviteToken: z.string().optional(),
});

const refreshTokenSchema = z.object({
  refreshToken: z.string().min(1, 'Refresh token is required'),
});

const passwordResetSchema = z.object({
  email: z.string().email('Invalid email format'),
  tenantId: z.string().optional(),
});

const passwordUpdateSchema = z.object({
  resetToken: z.string().min(1, 'Reset token is required'),
  newPassword: z.string().min(8, 'Password must be at least 8 characters'),
});

const tenantSwitchSchema = z.object({
  targetTenantId: z.string().min(1, 'Target tenant ID is required'),
});

const profileUpdateSchema = z.object({
  firstName: z.string().optional(),
  lastName: z.string().optional(),
  avatar: z.string().url().optional(),
  preferences: z.record(z.any()).optional(),
});

const mfaSetupSchema = z.object({
  type: z.enum(['totp', 'sms']),
  phoneNumber: z.string().optional(),
});

const mfaVerifySchema = z.object({
  code: z.string().min(6, 'Code must be at least 6 characters'),
  type: z.enum(['totp', 'sms']),
});

export function createAuthRoutes(
  redisClient: RedisClient,
  apiClient: ApiClient,
  wsService: WebSocketService
): Router {
  const router = Router();

  // Apply rate limiting to auth endpoints
  const authRateLimit = createRateLimitMiddleware(redisClient, rateLimitConfigs.auth);
  const passwordResetRateLimit = createRateLimitMiddleware(redisClient, rateLimitConfigs.passwordReset);
  const registrationRateLimit = createRateLimitMiddleware(redisClient, rateLimitConfigs.registration);

  // Login endpoint
  router.post('/login', authRateLimit, asyncHandler(async (req, res) => {
    const loginData = loginSchema.parse(req.body);

    try {
      // Call auth service
      const authResult = await apiClient.login(
        loginData.email,
        loginData.password,
        loginData.tenantId
      );

      // Cache session data
      await redisClient.cacheUserSession(
        authResult.session.id,
        authResult.session,
        24 * 60 * 60 // 24 hours
      );

      // Cache user data
      await redisClient.cacheUserData(
        authResult.user.id,
        authResult.user,
        5 * 60 // 5 minutes
      );

      // Cache tenant data if available
      if (authResult.tenant) {
        await redisClient.cacheTenantData(
          authResult.tenant.id,
          authResult.tenant,
          5 * 60 // 5 minutes
        );
      }

      // Notify WebSocket clients
      wsService.notifyAuthStatusUpdate({
        type: 'login',
        userId: authResult.user.id,
        tenantId: authResult.tenant?.id,
        sessionId: authResult.session.id,
        data: {
          email: authResult.user.email,
          tenantName: authResult.tenant?.name,
        },
      });

      const response: LoginResponse = {
        user: authResult.user,
        tenant: authResult.tenant,
        session: authResult.session,
        availableTenants: authResult.availableTenants || [],
        permissions: authResult.permissions || [],
        features: authResult.features || [],
      };

      res.json(response);
    } catch (error) {
      if (error.status === 401) {
        throw new BFFError(401, 'Invalid credentials', 'INVALID_CREDENTIALS');
      } else if (error.status === 423) {
        throw new BFFError(423, 'Account locked', 'ACCOUNT_LOCKED', error.data);
      }
      throw error;
    }
  }));

  // Register endpoint
  router.post('/register', registrationRateLimit, asyncHandler(async (req, res) => {
    const registerData = registerSchema.parse(req.body);

    try {
      const result = await apiClient.register(registerData);

      const response: RegisterResponse = {
        user: result.user,
        tenant: result.tenant,
        requiresEmailVerification: result.requiresEmailVerification || false,
        message: result.message || 'Registration successful',
      };

      res.status(201).json(response);
    } catch (error) {
      if (error.status === 409) {
        throw new BFFError(409, 'Email already registered', 'EMAIL_ALREADY_EXISTS');
      } else if (error.status === 422) {
        throw new BFFError(422, 'Registration validation failed', 'REGISTRATION_VALIDATION_FAILED', error.data);
      }
      throw error;
    }
  }));

  // Refresh token endpoint
  router.post('/refresh', asyncHandler(async (req, res) => {
    const { refreshToken } = refreshTokenSchema.parse(req.body);

    try {
      const result = await apiClient.refreshToken(refreshToken);

      // Update cached session
      const sessionData = await redisClient.getUserSession(result.sessionId);
      if (sessionData) {
        await redisClient.cacheUserSession(
          result.sessionId,
          {
            ...sessionData,
            token: result.token,
            refreshToken: result.refreshToken,
            expiresAt: result.expiresAt,
          },
          24 * 60 * 60 // 24 hours
        );
      }

      const response: RefreshTokenResponse = {
        token: result.token,
        refreshToken: result.refreshToken,
        expiresAt: result.expiresAt,
      };

      res.json(response);
    } catch (error) {
      if (error.status === 401) {
        throw new BFFError(401, 'Invalid refresh token', 'INVALID_REFRESH_TOKEN');
      }
      throw error;
    }
  }));

  // Logout endpoint
  router.post('/logout', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    try {
      await apiClient.logout(req.user!.sessionId, req.token!);

      // Remove from cache
      await redisClient.invalidateUserSession(req.user!.sessionId);

      // Notify WebSocket clients
      wsService.notifyAuthStatusUpdate({
        type: 'logout',
        userId: req.user!.id,
        tenantId: req.user!.tenantId,
        sessionId: req.user!.sessionId,
      });

      res.status(204).send();
    } catch (error) {
      // Even if backend logout fails, clear local cache
      await redisClient.invalidateUserSession(req.user!.sessionId);
      res.status(204).send();
    }
  }));

  // Password reset request
  router.post('/password-reset', passwordResetRateLimit, asyncHandler(async (req, res) => {
    const { email, tenantId } = passwordResetSchema.parse(req.body);

    try {
      const result = await apiClient.requestPasswordReset(email, tenantId);

      const response: PasswordResetResponse = {
        message: result.message || 'Password reset email sent',
        ...(process.env.NODE_ENV === 'development' && { resetToken: result.resetToken }),
      };

      res.json(response);
    } catch (error) {
      // Always return success to prevent email enumeration
      res.json({ message: 'If the email exists, a password reset link has been sent' });
    }
  }));

  // Password reset confirmation
  router.post('/password-reset/confirm', asyncHandler(async (req, res) => {
    const { resetToken, newPassword } = passwordUpdateSchema.parse(req.body);

    try {
      await apiClient.resetPassword(resetToken, newPassword);
      res.json({ message: 'Password updated successfully' });
    } catch (error) {
      if (error.status === 400) {
        throw new BFFError(400, 'Invalid or expired reset token', 'INVALID_RESET_TOKEN');
      }
      throw error;
    }
  }));

  // Email verification
  router.post('/verify-email', asyncHandler(async (req, res) => {
    const { verificationToken } = z.object({
      verificationToken: z.string().min(1, 'Verification token is required'),
    }).parse(req.body);

    try {
      const result = await apiClient.verifyEmail(verificationToken);
      res.json({ message: result.message || 'Email verified successfully' });
    } catch (error) {
      if (error.status === 400) {
        throw new BFFError(400, 'Invalid or expired verification token', 'INVALID_VERIFICATION_TOKEN');
      }
      throw error;
    }
  }));

  // Tenant switching (workflow-based)
  router.post('/tenant-switch', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    const { targetTenantId } = tenantSwitchSchema.parse(req.body);

    try {
      // Initiate tenant switch workflow
      const workflowResult = await apiClient.initiateWorkflow<TenantSwitchResponse>(
        'tenant-switch',
        {
          userId: req.user!.id,
          currentTenantId: req.user!.tenantId,
          targetTenantId,
        },
        req.token!,
        req.user!.tenantId,
        true // synchronous for better UX
      );

      if (workflowResult.type === 'sync' && workflowResult.data) {
        // Update cached data
        await redisClient.cacheUserSession(
          workflowResult.data.session.id,
          workflowResult.data.session,
          24 * 60 * 60
        );

        await redisClient.cacheTenantData(
          workflowResult.data.tenant.id,
          workflowResult.data.tenant,
          5 * 60
        );

        // Notify WebSocket clients
        wsService.notifyTenantSwitched(
          req.user!.id,
          req.user!.tenantId,
          targetTenantId
        );

        res.json(workflowResult.data);
      } else {
        // Async workflow
        const response: WorkflowResponse<TenantSwitchResponse> = {
          type: 'async',
          operationId: workflowResult.operationId,
          statusUrl: workflowResult.statusUrl,
          estimatedDuration: 30, // seconds
        };
        res.status(202).json(response);
      }
    } catch (error) {
      if (error.status === 403) {
        throw new BFFError(403, 'Access denied to target tenant', 'TENANT_ACCESS_DENIED');
      }
      throw error;
    }
  }));

  // Profile update
  router.put('/profile', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    const updates = profileUpdateSchema.parse(req.body);

    try {
      const updatedUser = await apiClient.updateUserProfile(
        req.user!.id,
        updates,
        req.token!,
        req.tenantId!
      );

      // Update cache
      await redisClient.cacheUserData(req.user!.id, updatedUser, 5 * 60);

      // Notify WebSocket clients
      wsService.notifyProfileUpdated(req.user!.id, req.tenantId!, updates);

      res.json(updatedUser);
    } catch (error) {
      if (error.status === 422) {
        throw new BFFError(422, 'Profile validation failed', 'PROFILE_VALIDATION_FAILED', error.data);
      }
      throw error;
    }
  }));

  // MFA setup
  router.post('/mfa/setup', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    const { type, phoneNumber } = mfaSetupSchema.parse(req.body);

    try {
      // Initiate MFA setup workflow
      const workflowResult = await apiClient.initiateWorkflow<MfaSetupResponse>(
        'mfa-setup',
        { type, phoneNumber, userId: req.user!.id },
        req.token!,
        req.user!.tenantId
      );

      if (workflowResult.type === 'sync' && workflowResult.data) {
        res.json(workflowResult.data);
      } else {
        res.status(202).json({
          type: 'async',
          operationId: workflowResult.operationId,
          statusUrl: workflowResult.statusUrl,
        });
      }
    } catch (error) {
      throw error;
    }
  }));

  // MFA verification
  router.post('/mfa/verify', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    const { code, type } = mfaVerifySchema.parse(req.body);

    try {
      const result = await apiClient.initiateWorkflow(
        'mfa-verify',
        { code, type, userId: req.user!.id },
        req.token!,
        req.user!.tenantId,
        true
      );

      res.json(result.data);
    } catch (error) {
      if (error.status === 400) {
        throw new BFFError(400, 'Invalid MFA code', 'INVALID_MFA_CODE');
      }
      throw error;
    }
  }));

  // Get current user info (aggregated)
  router.get('/me', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    try {
      // Try cache first
      const cacheKey = `user_profile:${req.user!.id}:${req.user!.tenantId}`;
      let userData = await redisClient.getAggregatedData(cacheKey);

      if (!userData) {
        // Fetch from multiple sources
        const [userProfile, userTenants] = await Promise.all([
          apiClient.getUserProfile(req.user!.id, req.token!, req.user!.tenantId),
          apiClient.getUserTenants(req.user!.id, req.token!),
        ]);

        userData = {
          ...userProfile,
          availableTenants: userTenants,
        };

        // Cache for 2 minutes
        await redisClient.cacheAggregatedData(cacheKey, userData, 120);
      }

      res.json(userData);
    } catch (error) {
      throw error;
    }
  }));

  return router;
}