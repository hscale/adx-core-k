import { Router } from 'express';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { AuthenticatedRequest, requireAuth } from '../middleware/auth.js';
import { TenantRequest, requireTenant } from '../middleware/tenant.js';
import { createRateLimitMiddleware, rateLimitConfigs } from '../middleware/rateLimit.js';
import { asyncHandler, AggregationError } from '../middleware/errorHandler.js';
import { AggregatedDashboardData, ActivityItem, NotificationItem, QuickStats } from '../types/auth.js';

export function createAggregatedRoutes(
  redisClient: RedisClient,
  apiClient: ApiClient
): Router {
  const router = Router();

  // Apply rate limiting to aggregated endpoints
  const aggregationRateLimit = createRateLimitMiddleware(redisClient, rateLimitConfigs.aggregation);

  // Dashboard data aggregation
  router.get('/dashboard', requireAuth, requireTenant, aggregationRateLimit, asyncHandler(async (req: TenantRequest, res) => {
    try {
      const cacheKey = `dashboard:${req.user!.id}:${req.tenantId}`;
      let dashboardData = await redisClient.getAggregatedData(cacheKey);

      if (!dashboardData) {
        // Fetch data from multiple sources in parallel
        const [
          userProfile,
          tenantData,
          recentActivity,
          notifications,
          tenantStats,
        ] = await Promise.allSettled([
          apiClient.getUserProfile(req.user!.id, req.token!, req.tenantId!),
          apiClient.getTenant(req.tenantId!, req.token!),
          apiClient.getUserActivity(req.user!.id, req.token!, req.tenantId!, 10),
          apiClient.getUserNotifications(req.user!.id, req.token!, req.tenantId!, 5),
          apiClient.getTenantStats(req.tenantId!, req.token!),
        ]);

        // Handle partial failures gracefully
        const user = userProfile.status === 'fulfilled' ? userProfile.value : req.user;
        const tenant = tenantData.status === 'fulfilled' ? tenantData.value : req.tenant;
        const activity = recentActivity.status === 'fulfilled' ? recentActivity.value : [];
        const notifs = notifications.status === 'fulfilled' ? notifications.value : [];
        const stats = tenantStats.status === 'fulfilled' ? tenantStats.value : getDefaultStats();

        // Get available tenants
        let availableTenants = [];
        try {
          availableTenants = await apiClient.getUserTenants(req.user!.id, req.token!);
        } catch (error) {
          console.warn('Failed to fetch available tenants:', error.message);
          availableTenants = [tenant];
        }

        dashboardData = {
          user,
          tenant,
          recentActivity: activity,
          notifications: notifs,
          quickStats: stats,
          availableTenants,
        } as AggregatedDashboardData;

        // Cache for 2 minutes
        await redisClient.cacheAggregatedData(cacheKey, dashboardData, 120);
      }

      res.json(dashboardData);
    } catch (error) {
      throw new AggregationError('Failed to aggregate dashboard data', {
        userId: req.user!.id,
        tenantId: req.tenantId,
        error: error.message,
      });
    }
  }));

  // User profile with extended data
  router.get('/profile', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    try {
      const cacheKey = `profile_extended:${req.user!.id}:${req.tenantId}`;
      let profileData = await redisClient.getAggregatedData(cacheKey);

      if (!profileData) {
        const [
          userProfile,
          userTenants,
          userActivity,
          userStats,
        ] = await Promise.allSettled([
          apiClient.getUserProfile(req.user!.id, req.token!, req.tenantId!),
          apiClient.getUserTenants(req.user!.id, req.token!),
          apiClient.getUserActivity(req.user!.id, req.token!, req.tenantId!, 20),
          // Get user-specific stats (could be from analytics service)
          Promise.resolve({
            totalLogins: 0,
            lastLogin: null,
            filesUploaded: 0,
            workflowsExecuted: 0,
          }),
        ]);

        profileData = {
          profile: userProfile.status === 'fulfilled' ? userProfile.value : req.user,
          tenants: userTenants.status === 'fulfilled' ? userTenants.value : [],
          recentActivity: userActivity.status === 'fulfilled' ? userActivity.value : [],
          stats: userStats.status === 'fulfilled' ? userStats.value : {},
        };

        // Cache for 5 minutes
        await redisClient.cacheAggregatedData(cacheKey, profileData, 300);
      }

      res.json(profileData);
    } catch (error) {
      throw new AggregationError('Failed to aggregate profile data', {
        userId: req.user!.id,
        tenantId: req.tenantId,
        error: error.message,
      });
    }
  }));

  // Tenant overview with user context
  router.get('/tenant-overview', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    try {
      const cacheKey = `tenant_overview:${req.tenantId}:${req.user!.id}`;
      let tenantOverview = await redisClient.getAggregatedData(cacheKey);

      if (!tenantOverview) {
        const [
          tenantData,
          tenantStats,
          userRole,
          tenantActivity,
        ] = await Promise.allSettled([
          apiClient.getTenant(req.tenantId!, req.token!),
          apiClient.getTenantStats(req.tenantId!, req.token!),
          // Get user's role in this tenant
          Promise.resolve({
            role: req.user!.roles[0] || 'user',
            permissions: req.user!.permissions,
            joinedAt: new Date().toISOString(),
          }),
          // Get tenant activity (if user has permission)
          req.user!.permissions.includes('tenant:read_activity')
            ? apiClient.getUserActivity(req.user!.id, req.token!, req.tenantId!, 10)
            : Promise.resolve([]),
        ]);

        tenantOverview = {
          tenant: tenantData.status === 'fulfilled' ? tenantData.value : req.tenant,
          stats: tenantStats.status === 'fulfilled' ? tenantStats.value : getDefaultStats(),
          userRole: userRole.status === 'fulfilled' ? userRole.value : {},
          recentActivity: tenantActivity.status === 'fulfilled' ? tenantActivity.value : [],
        };

        // Cache for 3 minutes
        await redisClient.cacheAggregatedData(cacheKey, tenantOverview, 180);
      }

      res.json(tenantOverview);
    } catch (error) {
      throw new AggregationError('Failed to aggregate tenant overview', {
        userId: req.user!.id,
        tenantId: req.tenantId,
        error: error.message,
      });
    }
  }));

  // Activity feed with context
  router.get('/activity', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);
    const offset = parseInt(req.query.offset as string) || 0;
    const type = req.query.type as string;

    try {
      const cacheKey = `activity_feed:${req.user!.id}:${req.tenantId}:${limit}:${offset}:${type || 'all'}`;
      let activityData = await redisClient.getAggregatedData(cacheKey);

      if (!activityData) {
        // Fetch activity from multiple sources
        const [
          userActivity,
          workflowActivity,
          // Could add more activity sources here
        ] = await Promise.allSettled([
          apiClient.getUserActivity(req.user!.id, req.token!, req.tenantId!, limit),
          // Get workflow activity from cache
          getWorkflowActivity(req.user!.id, req.tenantId!, redisClient, limit),
        ]);

        let allActivity: ActivityItem[] = [];

        if (userActivity.status === 'fulfilled') {
          allActivity = allActivity.concat(userActivity.value);
        }

        if (workflowActivity.status === 'fulfilled') {
          allActivity = allActivity.concat(workflowActivity.value);
        }

        // Filter by type if specified
        if (type) {
          allActivity = allActivity.filter(item => item.type === type);
        }

        // Sort by timestamp (newest first)
        allActivity.sort((a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime());

        // Apply pagination
        const paginatedActivity = allActivity.slice(offset, offset + limit);

        activityData = {
          activity: paginatedActivity,
          total: allActivity.length,
          hasMore: offset + limit < allActivity.length,
        };

        // Cache for 1 minute
        await redisClient.cacheAggregatedData(cacheKey, activityData, 60);
      }

      res.json(activityData);
    } catch (error) {
      throw new AggregationError('Failed to aggregate activity data', {
        userId: req.user!.id,
        tenantId: req.tenantId,
        error: error.message,
      });
    }
  }));

  // Notifications with context
  router.get('/notifications', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 50);
    const unreadOnly = req.query.unreadOnly === 'true';

    try {
      const cacheKey = `notifications:${req.user!.id}:${req.tenantId}:${limit}:${unreadOnly}`;
      let notificationData = await redisClient.getAggregatedData(cacheKey);

      if (!notificationData) {
        // Fetch notifications from API
        let notifications = await apiClient.getUserNotifications(req.user!.id, req.token!, req.tenantId!, limit * 2);

        // Filter unread if requested
        if (unreadOnly) {
          notifications = notifications.filter((n: NotificationItem) => !n.isRead);
        }

        // Limit results
        notifications = notifications.slice(0, limit);

        // Add workflow-related notifications from cache
        const workflowNotifications = await getWorkflowNotifications(req.user!.id, redisClient);
        
        const allNotifications = [...notifications, ...workflowNotifications]
          .sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime())
          .slice(0, limit);

        const unreadCount = allNotifications.filter(n => !n.isRead).length;

        notificationData = {
          notifications: allNotifications,
          unreadCount,
          total: allNotifications.length,
        };

        // Cache for 30 seconds
        await redisClient.cacheAggregatedData(cacheKey, notificationData, 30);
      }

      res.json(notificationData);
    } catch (error) {
      throw new AggregationError('Failed to aggregate notification data', {
        userId: req.user!.id,
        tenantId: req.tenantId,
        error: error.message,
      });
    }
  }));

  // Quick stats aggregation
  router.get('/stats', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    try {
      const cacheKey = `quick_stats:${req.user!.id}:${req.tenantId}`;
      let statsData = await redisClient.getAggregatedData(cacheKey);

      if (!statsData) {
        const [
          tenantStats,
          userWorkflowStats,
          userActivityCount,
        ] = await Promise.allSettled([
          apiClient.getTenantStats(req.tenantId!, req.token!),
          getUserWorkflowStats(req.user!.id, redisClient),
          getUserActivityCount(req.user!.id, req.tenantId!, redisClient),
        ]);

        statsData = {
          tenant: tenantStats.status === 'fulfilled' ? tenantStats.value : getDefaultStats(),
          workflows: userWorkflowStats.status === 'fulfilled' ? userWorkflowStats.value : {},
          activity: userActivityCount.status === 'fulfilled' ? userActivityCount.value : 0,
        };

        // Cache for 5 minutes
        await redisClient.cacheAggregatedData(cacheKey, statsData, 300);
      }

      res.json(statsData);
    } catch (error) {
      throw new AggregationError('Failed to aggregate stats data', {
        userId: req.user!.id,
        tenantId: req.tenantId,
        error: error.message,
      });
    }
  }));

  // Batch data endpoint for multiple aggregations
  router.post('/batch', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    const { requests } = req.body;
    
    if (!Array.isArray(requests) || requests.length === 0) {
      throw new AggregationError('Invalid batch request format');
    }

    if (requests.length > 10) {
      throw new AggregationError('Maximum 10 batch requests allowed');
    }

    try {
      const results = await Promise.allSettled(
        requests.map(async (request: { id: string; endpoint: string; params?: any }) => {
          const cacheKey = `batch:${request.id}:${req.user!.id}:${req.tenantId}`;
          let data = await redisClient.getAggregatedData(cacheKey);

          if (!data) {
            // Route to appropriate aggregation logic based on endpoint
            switch (request.endpoint) {
              case 'dashboard':
                data = await aggregateDashboardData(req, apiClient);
                break;
              case 'profile':
                data = await aggregateProfileData(req, apiClient);
                break;
              case 'notifications':
                data = await aggregateNotificationData(req, apiClient, request.params);
                break;
              case 'activity':
                data = await aggregateActivityData(req, apiClient, redisClient, request.params);
                break;
              default:
                throw new Error(`Unknown endpoint: ${request.endpoint}`);
            }

            // Cache for 1 minute
            await redisClient.cacheAggregatedData(cacheKey, data, 60);
          }

          return { id: request.id, data };
        })
      );

      const response = results.map((result, index) => {
        const baseResponse = {
          id: requests[index].id,
        };
        
        if (result.status === 'fulfilled') {
          return { ...baseResponse, success: true, ...result.value };
        } else {
          return { ...baseResponse, success: false, error: result.reason.message };
        }
      });

      res.json({ results: response });
    } catch (error) {
      throw new AggregationError('Batch aggregation failed', {
        userId: req.user!.id,
        tenantId: req.tenantId,
        error: error.message,
      });
    }
  }));

  return router;
}

// Helper functions
function getDefaultStats(): QuickStats {
  return {
    activeUsers: 0,
    totalFiles: 0,
    storageUsed: 0,
    workflowsRunning: 0,
  };
}

async function getWorkflowActivity(userId: string, tenantId: string, redisClient: RedisClient, limit: number): Promise<ActivityItem[]> {
  try {
    const pattern = `workflow:*`;
    const workflowKeys = await redisClient.keys(pattern);
    
    const workflows = await Promise.all(
      workflowKeys.map(async (key) => {
        const workflow = await redisClient.get(key);
        return workflow;
      })
    );

    return workflows
      .filter(w => w && w.userId === userId)
      .slice(0, limit)
      .map(w => ({
        id: w.operationId,
        type: 'workflow',
        title: `${w.workflowType} workflow`,
        description: `Status: ${w.status}`,
        timestamp: w.updatedAt || w.startedAt,
        metadata: {
          workflowType: w.workflowType,
          status: w.status,
        },
      }));
  } catch (error) {
    console.warn('Failed to get workflow activity:', error.message);
    return [];
  }
}

async function getWorkflowNotifications(userId: string, redisClient: RedisClient): Promise<NotificationItem[]> {
  try {
    const pattern = `workflow:*`;
    const workflowKeys = await redisClient.keys(pattern);
    
    const workflows = await Promise.all(
      workflowKeys.map(async (key) => {
        const workflow = await redisClient.get(key);
        return workflow;
      })
    );

    return workflows
      .filter(w => w && w.userId === userId && ['completed', 'failed'].includes(w.status))
      .slice(0, 5)
      .map(w => ({
        id: `workflow_${w.operationId}`,
        type: w.status === 'completed' ? 'success' : 'error',
        title: `Workflow ${w.status}`,
        message: `${w.workflowType} workflow has ${w.status}`,
        isRead: false,
        createdAt: w.updatedAt || w.startedAt,
        actionUrl: `/workflows/${w.operationId}`,
      }));
  } catch (error) {
    console.warn('Failed to get workflow notifications:', error.message);
    return [];
  }
}

async function getUserWorkflowStats(userId: string, redisClient: RedisClient): Promise<any> {
  try {
    const pattern = `workflow:*`;
    const workflowKeys = await redisClient.keys(pattern);
    
    const workflows = await Promise.all(
      workflowKeys.map(async (key) => {
        const workflow = await redisClient.get(key);
        return workflow;
      })
    );

    const userWorkflows = workflows.filter(w => w && w.userId === userId);
    
    return {
      total: userWorkflows.length,
      completed: userWorkflows.filter(w => w.status === 'completed').length,
      failed: userWorkflows.filter(w => w.status === 'failed').length,
      running: userWorkflows.filter(w => w.status === 'running').length,
    };
  } catch (error) {
    return { total: 0, completed: 0, failed: 0, running: 0 };
  }
}

async function getUserActivityCount(userId: string, tenantId: string, redisClient: RedisClient): Promise<number> {
  try {
    // This would typically come from an analytics service
    // For now, return a placeholder
    return 0;
  } catch (error) {
    return 0;
  }
}

// Aggregation helper functions
async function aggregateDashboardData(req: TenantRequest, apiClient: ApiClient): Promise<any> {
  // Implementation similar to the dashboard endpoint
  return {};
}

async function aggregateProfileData(req: TenantRequest, apiClient: ApiClient): Promise<any> {
  // Implementation similar to the profile endpoint
  return {};
}

async function aggregateNotificationData(req: TenantRequest, apiClient: ApiClient, params?: any): Promise<any> {
  // Implementation similar to the notifications endpoint
  return {};
}

async function aggregateActivityData(req: TenantRequest, apiClient: ApiClient, redisClient: RedisClient, params?: any): Promise<any> {
  // Implementation similar to the activity endpoint
  return {};
}