import { Router } from 'express';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { TenantRequest } from '../middleware/tenant.js';
import { createAuthMiddleware, requirePermission } from '../middleware/auth.js';
import { 
  requireTenant, 
  requireTenantAccess, 
  requireTenantPermission,
  requireTenantRole 
} from '../middleware/tenant.js';
import { 
  createRateLimitMiddleware, 
  rateLimitConfigs,
  createEndpointRateLimit 
} from '../middleware/rateLimit.js';
import { asyncHandler } from '../middleware/errorHandler.js';
import { 
  TenantSwitchRequestSchema, 
  AnalyticsPeriodSchema,
  AnalyticsPeriod 
} from '../types/tenant.js';

export function createTenantRoutes(
  redisClient: RedisClient,
  apiClient: ApiClient,
  jwtSecret: string
): Router {
  const router = Router();
  const authMiddleware = createAuthMiddleware(redisClient, jwtSecret);

  // Get current tenant information
  router.get(
    '/current',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenant, tenantContext } = req;

      res.json({
        tenant,
        context: tenantContext,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get user's available tenants
  router.get(
    '/available',
    authMiddleware,
    createEndpointRateLimit(redisClient, 'available-tenants', { maxRequests: 30 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { user } = req;
      if (!user) {
        return res.status(401).json({ error: 'Authentication required' });
      }

      // Try cache first
      let tenants = await redisClient.getCachedUserTenants(user.id);

      if (!tenants) {
        // Fetch from API
        tenants = await apiClient.getUserTenants(
          user.id,
          req.headers.authorization?.substring(7) || ''
        );

        // Cache for 5 minutes
        await redisClient.cacheUserTenants(user.id, tenants, 300);
      }

      res.json({
        tenants,
        count: tenants.length,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Switch tenant
  router.post(
    '/switch',
    authMiddleware,
    createRateLimitMiddleware(redisClient, rateLimitConfigs.tenantSwitch),
    asyncHandler(async (req: TenantRequest, res) => {
      const { user } = req;
      if (!user) {
        return res.status(401).json({ error: 'Authentication required' });
      }

      // Validate request
      const validationResult = TenantSwitchRequestSchema.safeParse(req.body);
      if (!validationResult.success) {
        return res.status(400).json({
          error: {
            code: 'VALIDATION_ERROR',
            message: 'Invalid tenant switch request',
            details: validationResult.error.errors,
          },
        });
      }

      const switchRequest = validationResult.data;

      // Initiate tenant switch workflow
      const workflowResponse = await apiClient.switchTenant(
        switchRequest,
        user.id,
        req.headers.authorization?.substring(7) || ''
      );

      // Invalidate relevant caches
      await Promise.all([
        redisClient.invalidateUserTenants(user.id),
        redisClient.invalidateTenantContext(switchRequest.targetTenantId, user.id),
        switchRequest.currentTenantId ? 
          redisClient.invalidateTenantContext(switchRequest.currentTenantId, user.id) : 
          Promise.resolve(),
      ]);

      res.json({
        ...workflowResponse,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant overview/dashboard data
  router.get(
    '/:tenantId/overview',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:read'),
    createEndpointRateLimit(redisClient, 'tenant-overview', { maxRequests: 20 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenantId } = req.params;
      const { user } = req;

      if (!user) {
        return res.status(401).json({ error: 'Authentication required' });
      }

      // Get aggregated dashboard data
      const dashboardData = await apiClient.getTenantDashboardData(
        tenantId,
        user.id,
        req.headers.authorization?.substring(7) || ''
      );

      res.json({
        ...dashboardData,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant memberships
  router.get(
    '/:tenantId/memberships',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:members:read'),
    createEndpointRateLimit(redisClient, 'tenant-memberships', { maxRequests: 20 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenantId } = req.params;
      const page = parseInt(req.query.page as string) || 1;
      const limit = Math.min(parseInt(req.query.limit as string) || 50, 100);

      // Try cache first
      let memberships = await redisClient.getCachedTenantMemberships(tenantId);

      if (!memberships) {
        // Fetch from API
        const membershipData = await apiClient.getTenantMemberships(
          tenantId,
          req.headers.authorization?.substring(7) || '',
          page,
          limit
        );

        memberships = membershipData.memberships;

        // Cache for 5 minutes
        await redisClient.cacheTenantMemberships(tenantId, memberships, 300);
      }

      // Apply pagination to cached data
      const startIndex = (page - 1) * limit;
      const endIndex = startIndex + limit;
      const paginatedMemberships = memberships.slice(startIndex, endIndex);

      res.json({
        memberships: paginatedMemberships,
        pagination: {
          page,
          limit,
          total: memberships.length,
          totalPages: Math.ceil(memberships.length / limit),
        },
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant analytics
  router.get(
    '/:tenantId/analytics',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:analytics:read'),
    createRateLimitMiddleware(redisClient, rateLimitConfigs.analytics),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenantId } = req.params;
      const periodParam = req.query.period as string || 'day';

      // Validate period
      const periodValidation = AnalyticsPeriodSchema.safeParse(periodParam);
      if (!periodValidation.success) {
        return res.status(400).json({
          error: {
            code: 'INVALID_PERIOD',
            message: 'Invalid analytics period',
            validPeriods: Object.values(AnalyticsPeriod),
          },
        });
      }

      const period = periodValidation.data;

      // Try cache first
      let analytics = await redisClient.getCachedTenantAnalytics(tenantId, period);

      if (!analytics) {
        // Fetch from API
        analytics = await apiClient.getTenantAnalytics(
          tenantId,
          period,
          req.headers.authorization?.substring(7) || ''
        );

        // Cache for 10 minutes (analytics can be slightly stale)
        await redisClient.cacheTenantAnalytics(tenantId, period, analytics, 600);
      }

      res.json({
        ...analytics,
        cached: !!analytics,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant usage information
  router.get(
    '/:tenantId/usage',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:usage:read'),
    createEndpointRateLimit(redisClient, 'tenant-usage', { maxRequests: 30 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenantId } = req.params;

      const usage = await apiClient.getTenantUsage(
        tenantId,
        req.headers.authorization?.substring(7) || ''
      );

      res.json({
        ...usage,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant configuration
  router.get(
    '/:tenantId/configuration',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:config:read'),
    createEndpointRateLimit(redisClient, 'tenant-config', { maxRequests: 20 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenantId } = req.params;

      // Try cache first
      let configuration = await redisClient.getCachedTenantConfig(tenantId);

      if (!configuration) {
        // Fetch from API
        configuration = await apiClient.getTenantConfiguration(
          tenantId,
          req.headers.authorization?.substring(7) || ''
        );

        // Cache for 30 minutes (configuration changes infrequently)
        await redisClient.cacheTenantConfig(tenantId, configuration, 1800);
      }

      res.json({
        configuration,
        cached: !!configuration,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Update tenant configuration
  router.put(
    '/:tenantId/configuration',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:config:write'),
    createRateLimitMiddleware(redisClient, rateLimitConfigs.configuration),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenantId } = req.params;
      const configuration = req.body;

      // Update configuration via API
      const updatedConfiguration = await apiClient.updateTenantConfiguration(
        tenantId,
        configuration,
        req.headers.authorization?.substring(7) || ''
      );

      // Invalidate cache
      await redisClient.invalidateTenantConfig(tenantId);

      res.json({
        configuration: updatedConfiguration,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Invalidate tenant cache (admin only)
  router.post(
    '/:tenantId/cache/invalidate',
    authMiddleware,
    requireTenantRole('admin'),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenantId } = req.params;
      const { cacheType } = req.body;

      let invalidatedKeys = 0;

      switch (cacheType) {
        case 'all':
          await redisClient.invalidateTenant(tenantId);
          invalidatedKeys = 1; // Simplified count
          break;
        case 'analytics':
          await redisClient.invalidateTenantAnalytics(tenantId);
          invalidatedKeys = 1;
          break;
        case 'memberships':
          await redisClient.invalidateTenantMemberships(tenantId);
          invalidatedKeys = 1;
          break;
        case 'configuration':
          await redisClient.invalidateTenantConfig(tenantId);
          invalidatedKeys = 1;
          break;
        default:
          return res.status(400).json({
            error: {
              code: 'INVALID_CACHE_TYPE',
              message: 'Invalid cache type',
              validTypes: ['all', 'analytics', 'memberships', 'configuration'],
            },
          });
      }

      res.json({
        message: 'Cache invalidated successfully',
        cacheType,
        invalidatedKeys,
        timestamp: new Date().toISOString(),
      });
    })
  );

  return router;
}