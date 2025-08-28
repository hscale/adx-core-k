import { Router } from 'express';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { TenantRequest } from '../middleware/tenant.js';
import { createAuthMiddleware } from '../middleware/auth.js';
import { requireTenant, requireTenantAccess, requireTenantPermission } from '../middleware/tenant.js';
import { createEndpointRateLimit } from '../middleware/rateLimit.js';
import { asyncHandler } from '../middleware/errorHandler.js';
import { AnalyticsPeriod } from '../types/tenant.js';

export function createAggregatedRoutes(
  redisClient: RedisClient,
  apiClient: ApiClient,
  jwtSecret: string
): Router {
  const router = Router();
  const authMiddleware = createAuthMiddleware(redisClient, jwtSecret);

  // Get tenant dashboard data (aggregated from multiple sources)
  router.get(
    '/dashboard',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:read'),
    createEndpointRateLimit(redisClient, 'dashboard', { maxRequests: 20 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenant, user } = req;
      
      if (!tenant || !user) {
        return res.status(400).json({ error: 'Tenant and user context required' });
      }

      const cacheKey = `dashboard:${tenant.id}:${user.id}`;
      
      // Try cache first
      let dashboardData = await redisClient.getCachedTenantConfig(cacheKey);

      if (!dashboardData) {
        // Aggregate data from multiple sources
        const [
          tenantOverview,
          tenantAnalytics,
          tenantUsage,
          tenantMemberships,
        ] = await Promise.allSettled([
          apiClient.getTenantOverview(tenant.id, req.headers.authorization?.substring(7) || ''),
          apiClient.getTenantAnalytics(tenant.id, AnalyticsPeriod.DAY, req.headers.authorization?.substring(7) || ''),
          apiClient.getTenantUsage(tenant.id, req.headers.authorization?.substring(7) || ''),
          apiClient.getTenantMemberships(tenant.id, req.headers.authorization?.substring(7) || '', 1, 10),
        ]);

        dashboardData = {
          overview: tenantOverview.status === 'fulfilled' ? tenantOverview.value : null,
          analytics: tenantAnalytics.status === 'fulfilled' ? tenantAnalytics.value : null,
          usage: tenantUsage.status === 'fulfilled' ? tenantUsage.value : null,
          recentMemberships: tenantMemberships.status === 'fulfilled' ? 
            tenantMemberships.value.memberships.slice(0, 5) : [],
          tenant: {
            id: tenant.id,
            name: tenant.name,
            subscriptionTier: tenant.subscriptionTier,
            status: tenant.status,
            features: tenant.features,
          },
          user: {
            id: user.id,
            email: user.email,
            roles: req.tenantContext?.membership.roles || [],
            permissions: req.tenantContext?.permissions || [],
          },
          errors: [
            tenantOverview.status === 'rejected' ? { source: 'overview', error: tenantOverview.reason?.message } : null,
            tenantAnalytics.status === 'rejected' ? { source: 'analytics', error: tenantAnalytics.reason?.message } : null,
            tenantUsage.status === 'rejected' ? { source: 'usage', error: tenantUsage.reason?.message } : null,
            tenantMemberships.status === 'rejected' ? { source: 'memberships', error: tenantMemberships.reason?.message } : null,
          ].filter(Boolean),
        };

        // Cache for 5 minutes
        await redisClient.cacheTenantConfig(cacheKey, dashboardData, 300);
      }

      res.json({
        ...dashboardData,
        cached: !!dashboardData,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant summary (lightweight dashboard data)
  router.get(
    '/summary',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:read'),
    createEndpointRateLimit(redisClient, 'summary', { maxRequests: 30 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenant, user, tenantContext } = req;
      
      if (!tenant || !user || !tenantContext) {
        return res.status(400).json({ error: 'Complete tenant context required' });
      }

      const cacheKey = `summary:${tenant.id}:${user.id}`;
      
      // Try cache first
      let summaryData = await redisClient.getCachedTenantConfig(cacheKey);

      if (!summaryData) {
        // Get lightweight summary data
        const [tenantUsage] = await Promise.allSettled([
          apiClient.getTenantUsage(tenant.id, req.headers.authorization?.substring(7) || ''),
        ]);

        summaryData = {
          tenant: {
            id: tenant.id,
            name: tenant.name,
            displayName: tenant.displayName || tenant.name,
            subscriptionTier: tenant.subscriptionTier,
            status: tenant.status,
            features: tenant.features.length,
            lastActivityAt: tenant.lastActivityAt,
          },
          user: {
            id: user.id,
            email: user.email,
            roles: tenantContext.membership.roles,
            permissions: tenantContext.permissions.length,
            membershipStatus: tenantContext.membership.status,
            joinedAt: tenantContext.membership.joinedAt,
            lastActiveAt: tenantContext.membership.lastActiveAt,
          },
          usage: tenantUsage.status === 'fulfilled' ? {
            users: tenantUsage.value.quotaUsage?.users || { used: 0, limit: 0, percentage: 0 },
            storage: tenantUsage.value.quotaUsage?.storage || { used: 0, limit: 0, percentage: 0 },
            apiCalls: tenantUsage.value.quotaUsage?.apiCalls || { used: 0, limit: 0, percentage: 0 },
          } : null,
          branding: {
            primaryColor: tenant.branding.primaryColor,
            secondaryColor: tenant.branding.secondaryColor,
            logoUrl: tenant.branding.logoUrl,
            customDomain: tenant.branding.customDomain,
          },
        };

        // Cache for 2 minutes (shorter cache for summary)
        await redisClient.cacheTenantConfig(cacheKey, summaryData, 120);
      }

      res.json({
        ...summaryData,
        cached: !!summaryData,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant analytics overview (multiple periods)
  router.get(
    '/analytics-overview',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:analytics:read'),
    createEndpointRateLimit(redisClient, 'analytics-overview', { maxRequests: 15 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenant } = req;
      
      if (!tenant) {
        return res.status(400).json({ error: 'Tenant context required' });
      }

      const cacheKey = `analytics_overview:${tenant.id}`;
      
      // Try cache first
      let analyticsOverview = await redisClient.getCachedTenantAnalytics(tenant.id, 'overview');

      if (!analyticsOverview) {
        // Get analytics for multiple periods
        const periods = [AnalyticsPeriod.DAY, AnalyticsPeriod.WEEK, AnalyticsPeriod.MONTH];
        const analyticsPromises = periods.map(period =>
          apiClient.getTenantAnalytics(tenant.id, period, req.headers.authorization?.substring(7) || '')
            .catch(error => ({ period, error: error.message }))
        );

        const analyticsResults = await Promise.allSettled(analyticsPromises);

        analyticsOverview = {
          tenantId: tenant.id,
          periods: {},
          summary: {
            totalUsers: 0,
            activeUsers: 0,
            apiCallsToday: 0,
            workflowsToday: 0,
            storageUsed: 0,
            errorRate: 0,
          },
          trends: {
            userGrowth: 0,
            usageGrowth: 0,
            errorTrend: 0,
          },
          generatedAt: new Date().toISOString(),
        };

        // Process results
        analyticsResults.forEach((result, index) => {
          const period = periods[index];
          if (result.status === 'fulfilled' && !result.value.error) {
            analyticsOverview.periods[period] = result.value;
            
            // Update summary with daily data
            if (period === AnalyticsPeriod.DAY) {
              const dayData = result.value;
              analyticsOverview.summary = {
                totalUsers: dayData.metrics?.totalUsers || 0,
                activeUsers: dayData.metrics?.activeUsers || 0,
                apiCallsToday: dayData.metrics?.apiCallsThisPeriod || 0,
                workflowsToday: dayData.metrics?.workflowExecutionsThisPeriod || 0,
                storageUsed: dayData.metrics?.storageUsedGB || 0,
                errorRate: dayData.metrics?.errorRate || 0,
              };
            }
          } else {
            analyticsOverview.periods[period] = { error: result.status === 'rejected' ? result.reason : result.value.error };
          }
        });

        // Cache for 10 minutes
        await redisClient.cacheTenantAnalytics(tenant.id, 'overview', analyticsOverview, 600);
      }

      res.json({
        ...analyticsOverview,
        cached: !!analyticsOverview,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant health status (aggregated health checks)
  router.get(
    '/health',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:read'),
    createEndpointRateLimit(redisClient, 'health', { maxRequests: 60 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenant } = req;
      
      if (!tenant) {
        return res.status(400).json({ error: 'Tenant context required' });
      }

      const cacheKey = `health:${tenant.id}`;
      
      // Try cache first (short cache for health data)
      const cachedHealth = await redisClient.getCachedTenantConfig(cacheKey);

      if (cachedHealth) {
        return res.json({
          ...cachedHealth,
          cached: true,
          timestamp: new Date().toISOString(),
        });
      }

      // Aggregate health information
      const healthData = {
        tenant: {
          id: tenant.id,
          name: tenant.name,
          status: tenant.status,
          healthy: tenant.status === 'active',
        },
        services: {
          api: 'healthy', // Would check actual service health
          database: 'healthy',
          cache: redisClient.isHealthy() ? 'healthy' : 'unhealthy',
          workflows: 'healthy',
        },
        quotas: {
          users: { status: 'ok', usage: 0, limit: tenant.quotas.maxUsers },
          storage: { status: 'ok', usage: 0, limit: tenant.quotas.maxStorageGB },
          apiCalls: { status: 'ok', usage: 0, limit: tenant.quotas.maxApiCallsPerHour },
        },
        alerts: [],
        lastChecked: new Date().toISOString(),
      };

      // Determine overall health
      const servicesHealthy = Object.values(healthData.services).every(status => status === 'healthy');
      const quotasOk = Object.values(healthData.quotas).every(quota => quota.status === 'ok');
      
      const overallHealth = {
        status: servicesHealthy && quotasOk && healthData.tenant.healthy ? 'healthy' : 'degraded',
        services: healthData.services,
        quotas: healthData.quotas,
        tenant: healthData.tenant,
        alerts: healthData.alerts,
        lastChecked: healthData.lastChecked,
      };

      // Cache for 30 seconds (very short cache for health data)
      await redisClient.cacheTenantConfig(cacheKey, overallHealth, 30);

      res.json({
        ...overallHealth,
        cached: false,
        timestamp: new Date().toISOString(),
      });
    })
  );

  // Get tenant quick stats (for navigation/header display)
  router.get(
    '/quick-stats',
    authMiddleware,
    requireTenant,
    requireTenantAccess,
    requireTenantPermission('tenant:read'),
    createEndpointRateLimit(redisClient, 'quick-stats', { maxRequests: 120 }),
    asyncHandler(async (req: TenantRequest, res) => {
      const { tenant, tenantContext } = req;
      
      if (!tenant || !tenantContext) {
        return res.status(400).json({ error: 'Complete tenant context required' });
      }

      const quickStats = {
        tenant: {
          id: tenant.id,
          name: tenant.name,
          displayName: tenant.displayName || tenant.name,
          subscriptionTier: tenant.subscriptionTier,
          status: tenant.status,
        },
        user: {
          roles: tenantContext.membership.roles,
          permissions: tenantContext.permissions.length,
        },
        features: {
          available: tenant.features.length,
          enabled: tenant.features.filter(feature => 
            tenantContext.features.includes(feature)
          ).length,
        },
        branding: {
          primaryColor: tenant.branding.primaryColor,
          logoUrl: tenant.branding.logoUrl,
        },
        notifications: {
          unread: 0, // Would fetch from notification service
          alerts: 0, // Would fetch from alert service
        },
      };

      res.json({
        ...quickStats,
        timestamp: new Date().toISOString(),
      });
    })
  );

  return router;
}