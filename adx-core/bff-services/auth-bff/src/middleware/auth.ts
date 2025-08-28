import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { RedisClient } from '../services/redis.js';
import { BFFError } from './errorHandler.js';

export interface AuthenticatedRequest extends Request {
  user?: {
    id: string;
    email: string;
    roles: string[];
    permissions: string[];
    tenantId: string;
    sessionId: string;
  };
  token?: string;
}

export interface JWTPayload {
  sub: string; // User ID
  email: string;
  tenant_id: string;
  session_id: string;
  roles: string[];
  permissions: string[];
  exp: number;
  iat: number;
  iss: string;
  aud: string;
}

export function createAuthMiddleware(redisClient: RedisClient, jwtSecret: string) {
  return async (req: AuthenticatedRequest, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Skip auth for health check and public endpoints
      if (isPublicEndpoint(req.path)) {
        return next();
      }

      const token = extractToken(req);
      if (!token) {
        throw new BFFError(401, 'Authentication token required', 'MISSING_TOKEN');
      }

      // Verify JWT token
      let payload: JWTPayload;
      try {
        payload = jwt.verify(token, jwtSecret) as JWTPayload;
      } catch (error) {
        if (error.name === 'TokenExpiredError') {
          throw new BFFError(401, 'Token has expired', 'TOKEN_EXPIRED');
        } else if (error.name === 'JsonWebTokenError') {
          throw new BFFError(401, 'Invalid token', 'INVALID_TOKEN');
        }
        throw error;
      }

      // Check if session is still valid in Redis
      const sessionData = await redisClient.getUserSession(payload.session_id);
      if (!sessionData) {
        throw new BFFError(401, 'Session has expired', 'SESSION_EXPIRED');
      }

      // Check if session is active
      if (!sessionData.isActive) {
        throw new BFFError(401, 'Session is inactive', 'SESSION_INACTIVE');
      }

      // Update last activity
      await redisClient.cacheUserSession(payload.session_id, {
        ...sessionData,
        lastActivityAt: new Date().toISOString(),
      }, 3600); // 1 hour TTL

      // Attach user info to request
      req.user = {
        id: payload.sub,
        email: payload.email,
        roles: payload.roles || [],
        permissions: payload.permissions || [],
        tenantId: payload.tenant_id,
        sessionId: payload.session_id,
      };
      req.token = token;

      next();
    } catch (error) {
      next(error);
    }
  };
}

export function requireAuth(req: AuthenticatedRequest, res: Response, next: NextFunction): void {
  if (!req.user) {
    throw new BFFError(401, 'Authentication required', 'AUTHENTICATION_REQUIRED');
  }
  next();
}

export function requirePermission(permission: string) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.user) {
      throw new BFFError(401, 'Authentication required', 'AUTHENTICATION_REQUIRED');
    }

    if (!hasPermission(req.user, permission)) {
      throw new BFFError(403, `Permission '${permission}' required`, 'INSUFFICIENT_PERMISSIONS', {
        required: permission,
        userPermissions: req.user.permissions,
      });
    }

    next();
  };
}

export function requireRole(role: string) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.user) {
      throw new BFFError(401, 'Authentication required', 'AUTHENTICATION_REQUIRED');
    }

    if (!req.user.roles.includes(role)) {
      throw new BFFError(403, `Role '${role}' required`, 'INSUFFICIENT_ROLE', {
        required: role,
        userRoles: req.user.roles,
      });
    }

    next();
  };
}

export function requireAnyRole(roles: string[]) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.user) {
      throw new BFFError(401, 'Authentication required', 'AUTHENTICATION_REQUIRED');
    }

    const hasAnyRole = roles.some(role => req.user!.roles.includes(role));
    if (!hasAnyRole) {
      throw new BFFError(403, `One of roles [${roles.join(', ')}] required`, 'INSUFFICIENT_ROLE', {
        required: roles,
        userRoles: req.user.roles,
      });
    }

    next();
  };
}

export function requireAnyPermission(permissions: string[]) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction): void => {
    if (!req.user) {
      throw new BFFError(401, 'Authentication required', 'AUTHENTICATION_REQUIRED');
    }

    const hasAnyPermission = permissions.some(permission => hasPermission(req.user!, permission));
    if (!hasAnyPermission) {
      throw new BFFError(403, `One of permissions [${permissions.join(', ')}] required`, 'INSUFFICIENT_PERMISSIONS', {
        required: permissions,
        userPermissions: req.user.permissions,
      });
    }

    next();
  };
}

function extractToken(req: Request): string | null {
  // Check Authorization header
  const authHeader = req.headers.authorization;
  if (authHeader && authHeader.startsWith('Bearer ')) {
    return authHeader.substring(7);
  }

  // Check query parameter (for WebSocket upgrades)
  if (req.query.token && typeof req.query.token === 'string') {
    return req.query.token;
  }

  // Check cookie
  if (req.cookies && req.cookies.auth_token) {
    return req.cookies.auth_token;
  }

  return null;
}

function isPublicEndpoint(path: string): boolean {
  const publicPaths = [
    '/health',
    '/api/auth/login',
    '/api/auth/register',
    '/api/auth/refresh',
    '/api/auth/password-reset',
    '/api/auth/password-reset/confirm',
    '/api/auth/verify-email',
    '/ws', // WebSocket endpoint handles auth separately
  ];

  return publicPaths.some(publicPath => path.startsWith(publicPath));
}

function hasPermission(user: { permissions: string[] }, requiredPermission: string): boolean {
  // Check direct permissions
  if (user.permissions.includes(requiredPermission)) {
    return true;
  }

  // Check wildcard permissions
  const permissionParts = requiredPermission.split(':');
  for (const permission of user.permissions) {
    if (permission.endsWith('*')) {
      const wildcardBase = permission.slice(0, -1);
      if (requiredPermission.startsWith(wildcardBase)) {
        return true;
      }
    }
  }

  return false;
}

// Optional authentication middleware (doesn't throw if no auth)
export function optionalAuth(redisClient: RedisClient, jwtSecret: string) {
  return async (req: AuthenticatedRequest, res: Response, next: NextFunction): Promise<void> => {
    try {
      const token = extractToken(req);
      if (!token) {
        return next();
      }

      const payload = jwt.verify(token, jwtSecret) as JWTPayload;
      const sessionData = await redisClient.getUserSession(payload.session_id);
      
      if (sessionData && sessionData.isActive) {
        req.user = {
          id: payload.sub,
          email: payload.email,
          roles: payload.roles || [],
          permissions: payload.permissions || [],
          tenantId: payload.tenant_id,
          sessionId: payload.session_id,
        };
        req.token = token;

        // Update last activity
        await redisClient.cacheUserSession(payload.session_id, {
          ...sessionData,
          lastActivityAt: new Date().toISOString(),
        }, 3600);
      }
    } catch (error) {
      // Ignore auth errors for optional auth
      console.log('Optional auth failed:', error.message);
    }
    
    next();
  };
}