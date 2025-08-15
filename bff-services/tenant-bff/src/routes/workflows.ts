import { Router } from 'express';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { TenantRequest } from '../middleware/tenant.js';
import { createAuthMiddleware } from '../middleware/auth.js';
import { requireTenant, requireTenantAccess, requireTenantPermission } from '../middleware/tenant.js';
import { createRateLimitMiddleware, rateLimitConfigs } from '../middleware/rateLimit.js';
import { asyncHandler } from '../middleware/errorHandler.js';
import { TenantWorkflowRequestSchema } from '../types/tenant.js';

export function createWorkflowRoutes(
  redisClient: RedisClient,
  apiClient: ApiClient,
  jwtSecret: string
): Router {
  const router = Router();
  const authMiddleware = createAuthMiddleware(redisClient, jwtSecret);

  // Initiate tenant-related workflow
  router.post(
    '/initiate',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    createRateLimitMiddleware(redisClient, rateLimitConfigs.workflows),
    asyncHandler(async (req: TenantRequest, res) => {
      const { user, tenant } = req;
      
      if (!user || !tenant) {
        return res.status(401).json({ error: 'Authentication and tenant context required' });
      }

      // Validate request
      const validationResult = TenantWorkflowRequestSchema.safeParse({
        ...req.body,
        tenantId: tenant.id,
        userId: user.id,
      });

      if (!validationResult.success) {
        return res.status(400).json({
          error: {
            code: 'VALIDATION_ERROR',
            message: 'Invalid workflow request',
            details: validationResult.error.errors,
          },
        });
      }

      const workflowRequest = validationResult.data;

      // Check tenant-specific permissions for workflow type
      const workflowPermissions: Record<string, string> = {
        'provision-tenant': 'tenant:provision',
        'migrate-tenant': 'tenant:migrate',
        'bulk-invite-users': 'tenant:members:invite',
        'bulk-update-memberships': 'tenant:members:manage',
        'tenant-backup': 'tenant:backup',
        'tenant-restore': 'tenant:restore',
        'tenant-analytics-export': 'tenant:analytics:export',
        'tenant-configuration-backup': 'tenant:config:backup',
      };

      const requiredPermission = workflowPermissions[workflowRequest.workflowType];
      if (requiredPermission && !hasTenantPermission(req, requiredPermission)) {
        return res.status(403).json({
          error: {
            code: 'INSUFFICIENT_PERMISSIONS',
            message: `Permission required: ${requiredPermission}`,
          },
        });
      }

      // Initiate workflow
      const workflowResponse = await apiClient.initiateWorkflow(
        workflowRequest,
        req.headers.authorization?.substring(7) || ''
      );

      res.status(202).json({
        ...workflowResponse,
        workflowType: workflowRequest.workflowType,
        tenantId: tenant.id,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get workflow status
  router.get(
    '/:operationId/status',
    authMiddleware,
    requireTenantAccess,
    asyncHandler(async (req: TenantRequest, res) => {
      const { operationId } = req.params;

      const status = await apiClient.getWorkflowStatus(
        operationId,
        req.headers.authorization?.substring(7) || ''
      );

      res.json({
        ...status,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Cancel workflow
  router.post(
    '/:operationId/cancel',
    authMiddleware,
    requireTenantAccess,
    requireTenantPermission('workflow:cancel'),
    asyncHandler(async (req: TenantRequest, res) => {
      const { operationId } = req.params;

      await apiClient.cancelWorkflow(
        operationId,
        req.headers.authorization?.substring(7) || ''
      );

      res.json({
        message: 'Workflow cancellation requested',
        operationId,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Tenant provisioning workflow
  router.post(
    '/provision-tenant',
    authMiddleware,
    requireTenantPermission('tenant:provision'),
    createRateLimitMiddleware(redisClient, {
      windowMs: 60 * 1000,
      maxRequests: 5, // Very limited for provisioning
      message: 'Too many tenant provisioning requests',
    }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { user } = req;
      
      if (!user) {
        return res.status(401).json({ error: 'Authentication required' });
      }

      const tenantData = req.body;

      const workflowResponse = await apiClient.provisionTenant(
        tenantData,
        user.id,
        req.headers.authorization?.substring(7) || ''
      );

      res.status(202).json({
        ...workflowResponse,
        message: 'Tenant provisioning initiated',
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Tenant migration workflow
  router.post(
    '/migrate-tenant',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:migrate'),
    createRateLimitMiddleware(redisClient, {
      windowMs: 60 * 1000,
      maxRequests: 2, // Very limited for migration
      message: 'Too many tenant migration requests',
    }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { user, tenant } = req;
      
      if (!user || !tenant) {
        return res.status(401).json({ error: 'Authentication and tenant context required' });
      }

      const migrationData = req.body;

      const workflowResponse = await apiClient.migrateTenant(
        tenant.id,
        migrationData,
        user.id,
        req.headers.authorization?.substring(7) || ''
      );

      res.status(202).json({
        ...workflowResponse,
        message: 'Tenant migration initiated',
        tenantId: tenant.id,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Bulk invite users workflow
  router.post(
    '/bulk-invite-users',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:members:invite'),
    createRateLimitMiddleware(redisClient, {
      windowMs: 60 * 1000,
      maxRequests: 10,
      message: 'Too many bulk invite requests',
    }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { user, tenant } = req;
      
      if (!user || !tenant) {
        return res.status(401).json({ error: 'Authentication and tenant context required' });
      }

      const { invitations } = req.body;

      if (!Array.isArray(invitations) || invitations.length === 0) {
        return res.status(400).json({
          error: {
            code: 'INVALID_INVITATIONS',
            message: 'Invitations array is required and must not be empty',
          },
        });
      }

      if (invitations.length > 100) {
        return res.status(400).json({
          error: {
            code: 'TOO_MANY_INVITATIONS',
            message: 'Maximum 100 invitations per request',
          },
        });
      }

      const workflowResponse = await apiClient.bulkInviteUsers(
        tenant.id,
        invitations,
        user.id,
        req.headers.authorization?.substring(7) || ''
      );

      // Invalidate memberships cache
      await redisClient.invalidateTenantMemberships(tenant.id);

      res.status(202).json({
        ...workflowResponse,
        message: 'Bulk user invitation initiated',
        tenantId: tenant.id,
        invitationCount: invitations.length,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Bulk update memberships workflow
  router.post(
    '/bulk-update-memberships',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:members:manage'),
    createRateLimitMiddleware(redisClient, {
      windowMs: 60 * 1000,
      maxRequests: 15,
      message: 'Too many bulk membership update requests',
    }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { user, tenant } = req;
      
      if (!user || !tenant) {
        return res.status(401).json({ error: 'Authentication and tenant context required' });
      }

      const { updates } = req.body;

      if (!Array.isArray(updates) || updates.length === 0) {
        return res.status(400).json({
          error: {
            code: 'INVALID_UPDATES',
            message: 'Updates array is required and must not be empty',
          },
        });
      }

      if (updates.length > 50) {
        return res.status(400).json({
          error: {
            code: 'TOO_MANY_UPDATES',
            message: 'Maximum 50 updates per request',
          },
        });
      }

      const workflowResponse = await apiClient.bulkUpdateMemberships(
        tenant.id,
        updates,
        user.id,
        req.headers.authorization?.substring(7) || ''
      );

      // Invalidate memberships cache
      await redisClient.invalidateTenantMemberships(tenant.id);

      res.status(202).json({
        ...workflowResponse,
        message: 'Bulk membership update initiated',
        tenantId: tenant.id,
        updateCount: updates.length,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant-specific workflow history
  router.get(
    '/history',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('workflow:read'),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenant } = req;
      const page = parseInt(req.query.page as string) || 1;
      const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);
      const workflowType = req.query.workflowType as string;
      const status = req.query.status as string;

      if (!tenant) {
        return res.status(400).json({ error: 'Tenant context required' });
      }

      // This would typically call a workflow history API
      // For now, return a placeholder response
      const workflowHistory = {
        workflows: [],
        pagination: {
          page,
          limit,
          total: 0,
          totalPages: 0,
        },
        filters: {
          tenantId: tenant.id,
          workflowType,
          status,
        },
      };

      res.json({
        ...workflowHistory,
        timestamp: new Date().toISOString(),
      });
    })
  );

  return router;
}

// Helper function to check tenant permissions
function hasTenantPermission(req: TenantRequest, permission: string): boolean {
  if (!req.tenantContext) return false;

  const { permissions } = req.tenantContext;

  // Check direct permissions
  if (permissions.includes(permission)) {
    return true;
  }

  // Check wildcard permissions
  for (const perm of permissions) {
    if (perm.endsWith('*')) {
      const wildcardBase = perm.slice(0, -1);
      if (permission.startsWith(wildcardBase)) {
        return true;
      }
    }
  }

  return false;
}