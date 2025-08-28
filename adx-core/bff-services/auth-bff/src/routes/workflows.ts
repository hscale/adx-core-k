import { Router } from 'express';
import { z } from 'zod';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { WebSocketService } from '../services/websocket.js';
import { AuthenticatedRequest, requireAuth } from '../middleware/auth.js';
import { TenantRequest, requireTenant } from '../middleware/tenant.js';
import { createRateLimitMiddleware, rateLimitConfigs } from '../middleware/rateLimit.js';
import { asyncHandler, BFFError } from '../middleware/errorHandler.js';
import { WorkflowResponse, WorkflowStatus } from '../types/auth.js';

// Validation schemas
const workflowRequestSchema = z.object({
  workflowType: z.string().min(1, 'Workflow type is required'),
  request: z.record(z.any()),
  synchronous: z.boolean().optional().default(false),
});

const batchWorkflowSchema = z.object({
  workflows: z.array(z.object({
    id: z.string(),
    workflowType: z.string(),
    request: z.record(z.any()),
  })).min(1, 'At least one workflow is required').max(10, 'Maximum 10 workflows allowed'),
});

export function createWorkflowRoutes(
  redisClient: RedisClient,
  apiClient: ApiClient,
  wsService: WebSocketService
): Router {
  const router = Router();

  // Apply rate limiting to workflow endpoints
  const workflowRateLimit = createRateLimitMiddleware(redisClient, rateLimitConfigs.workflow);

  // Generic workflow initiation endpoint
  router.post('/execute', requireAuth, requireTenant, workflowRateLimit, asyncHandler(async (req: TenantRequest, res) => {
    const { workflowType, request: workflowRequest, synchronous } = workflowRequestSchema.parse(req.body);

    try {
      // Add user and tenant context to workflow request
      const enrichedRequest = {
        ...workflowRequest,
        userContext: {
          userId: req.user!.id,
          email: req.user!.email,
          roles: req.user!.roles,
          permissions: req.user!.permissions,
        },
        tenantContext: {
          tenantId: req.tenantId!,
          tenantName: req.tenant!.name,
          subscriptionTier: req.tenant!.subscriptionTier,
          features: req.tenant!.features,
        },
      };

      const result = await apiClient.initiateWorkflow(
        workflowType,
        enrichedRequest,
        req.token!,
        req.tenantId!,
        synchronous
      );

      if (result.type === 'sync') {
        res.json(result);
      } else {
        // Cache workflow info for status tracking
        await redisClient.set(
          `workflow:${result.operationId}`,
          {
            operationId: result.operationId,
            workflowType,
            userId: req.user!.id,
            tenantId: req.tenantId!,
            startedAt: new Date().toISOString(),
            status: 'pending',
          },
          3600 // 1 hour
        );

        res.status(202).json(result);
      }
    } catch (error) {
      if (error.status === 400) {
        throw new BFFError(400, 'Invalid workflow request', 'INVALID_WORKFLOW_REQUEST', error.data);
      } else if (error.status === 403) {
        throw new BFFError(403, 'Workflow execution not allowed', 'WORKFLOW_EXECUTION_DENIED');
      }
      throw error;
    }
  }));

  // Batch workflow execution
  router.post('/batch', requireAuth, requireTenant, workflowRateLimit, asyncHandler(async (req: TenantRequest, res) => {
    const { workflows } = batchWorkflowSchema.parse(req.body);

    try {
      const results = await Promise.allSettled(
        workflows.map(async (workflow) => {
          const enrichedRequest = {
            ...workflow.request,
            userContext: {
              userId: req.user!.id,
              email: req.user!.email,
              roles: req.user!.roles,
              permissions: req.user!.permissions,
            },
            tenantContext: {
              tenantId: req.tenantId!,
              tenantName: req.tenant!.name,
              subscriptionTier: req.tenant!.subscriptionTier,
              features: req.tenant!.features,
            },
          };

          const result = await apiClient.initiateWorkflow(
            workflow.workflowType,
            enrichedRequest,
            req.token!,
            req.tenantId!,
            false // Always async for batch
          );

          // Cache workflow info
          if (result.operationId) {
            await redisClient.set(
              `workflow:${result.operationId}`,
              {
                operationId: result.operationId,
                workflowType: workflow.workflowType,
                batchId: workflow.id,
                userId: req.user!.id,
                tenantId: req.tenantId!,
                startedAt: new Date().toISOString(),
                status: 'pending',
              },
              3600
            );
          }

          return {
            id: workflow.id,
            ...result,
          };
        })
      );

      const response = results.map((result, index) => {
        const baseResponse = {
          id: workflows[index].id,
        };
        
        if (result.status === 'fulfilled') {
          return { ...baseResponse, success: true, ...result.value };
        } else {
          return { ...baseResponse, success: false, error: result.reason.message };
        }
      });

      res.status(202).json({ workflows: response });
    } catch (error) {
      throw error;
    }
  }));

  // Get workflow status
  router.get('/:operationId/status', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    const { operationId } = req.params;

    try {
      // Check if user has access to this workflow
      const workflowInfo = await redisClient.get(`workflow:${operationId}`);
      if (workflowInfo && workflowInfo.userId !== req.user!.id) {
        throw new BFFError(403, 'Access denied to workflow', 'WORKFLOW_ACCESS_DENIED');
      }

      const status = await apiClient.getWorkflowStatus(operationId, req.token!, req.user!.tenantId);

      // Update cached workflow info
      if (workflowInfo) {
        await redisClient.set(
          `workflow:${operationId}`,
          {
            ...workflowInfo,
            status: status.status,
            updatedAt: new Date().toISOString(),
            ...(status.result && { result: status.result }),
            ...(status.error && { error: status.error }),
          },
          3600
        );
      }

      res.json(status);
    } catch (error) {
      if (error.status === 404) {
        throw new BFFError(404, 'Workflow not found', 'WORKFLOW_NOT_FOUND');
      }
      throw error;
    }
  }));

  // Cancel workflow
  router.post('/:operationId/cancel', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    const { operationId } = req.params;

    try {
      // Check if user has access to this workflow
      const workflowInfo = await redisClient.get(`workflow:${operationId}`);
      if (workflowInfo && workflowInfo.userId !== req.user!.id) {
        throw new BFFError(403, 'Access denied to workflow', 'WORKFLOW_ACCESS_DENIED');
      }

      await apiClient.cancelWorkflow(operationId, req.token!, req.user!.tenantId);

      // Update cached workflow info
      if (workflowInfo) {
        await redisClient.set(
          `workflow:${operationId}`,
          {
            ...workflowInfo,
            status: 'cancelled',
            updatedAt: new Date().toISOString(),
          },
          3600
        );
      }

      res.json({ message: 'Workflow cancelled successfully' });
    } catch (error) {
      if (error.status === 404) {
        throw new BFFError(404, 'Workflow not found', 'WORKFLOW_NOT_FOUND');
      } else if (error.status === 409) {
        throw new BFFError(409, 'Workflow cannot be cancelled', 'WORKFLOW_CANCELLATION_FAILED');
      }
      throw error;
    }
  }));

  // Get user's workflow history
  router.get('/history', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    const page = parseInt(req.query.page as string) || 1;
    const limit = Math.min(parseInt(req.query.limit as string) || 20, 100);
    const status = req.query.status as string;
    const workflowType = req.query.workflowType as string;

    try {
      // Get workflow IDs from cache
      const pattern = `workflow:*`;
      const workflowKeys = await redisClient.keys(pattern);
      
      let workflows = await Promise.all(
        workflowKeys.map(async (key) => {
          const workflow = await redisClient.get(key);
          return workflow;
        })
      );

      // Filter by user
      workflows = workflows.filter(w => w && w.userId === req.user!.id);

      // Apply filters
      if (status) {
        workflows = workflows.filter(w => w.status === status);
      }
      if (workflowType) {
        workflows = workflows.filter(w => w.workflowType === workflowType);
      }

      // Sort by start time (newest first)
      workflows.sort((a, b) => new Date(b.startedAt).getTime() - new Date(a.startedAt).getTime());

      // Paginate
      const total = workflows.length;
      const startIndex = (page - 1) * limit;
      const paginatedWorkflows = workflows.slice(startIndex, startIndex + limit);

      res.json({
        workflows: paginatedWorkflows,
        pagination: {
          page,
          limit,
          total,
          totalPages: Math.ceil(total / limit),
          hasNext: startIndex + limit < total,
          hasPrev: page > 1,
        },
      });
    } catch (error) {
      throw error;
    }
  }));

  // Get workflow statistics
  router.get('/stats', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    try {
      const cacheKey = `workflow_stats:${req.user!.id}:${req.user!.tenantId}`;
      let stats = await redisClient.getAggregatedData(cacheKey);

      if (!stats) {
        // Get all user workflows
        const pattern = `workflow:*`;
        const workflowKeys = await redisClient.keys(pattern);
        
        const workflows = await Promise.all(
          workflowKeys.map(async (key) => {
            const workflow = await redisClient.get(key);
            return workflow;
          })
        );

        const userWorkflows = workflows.filter(w => w && w.userId === req.user!.id);

        // Calculate statistics
        const total = userWorkflows.length;
        const byStatus = userWorkflows.reduce((acc, w) => {
          acc[w.status] = (acc[w.status] || 0) + 1;
          return acc;
        }, {} as Record<string, number>);

        const byType = userWorkflows.reduce((acc, w) => {
          acc[w.workflowType] = (acc[w.workflowType] || 0) + 1;
          return acc;
        }, {} as Record<string, number>);

        // Recent activity (last 24 hours)
        const oneDayAgo = new Date(Date.now() - 24 * 60 * 60 * 1000);
        const recentWorkflows = userWorkflows.filter(w => 
          new Date(w.startedAt) > oneDayAgo
        );

        stats = {
          total,
          byStatus,
          byType,
          recent: {
            count: recentWorkflows.length,
            workflows: recentWorkflows.slice(0, 10), // Last 10
          },
          summary: {
            completed: byStatus.completed || 0,
            failed: byStatus.failed || 0,
            running: byStatus.running || 0,
            pending: byStatus.pending || 0,
          },
        };

        // Cache for 5 minutes
        await redisClient.cacheAggregatedData(cacheKey, stats, 300);
      }

      res.json(stats);
    } catch (error) {
      throw error;
    }
  }));

  // WebSocket endpoint for real-time workflow updates
  router.get('/:operationId/stream', requireAuth, asyncHandler(async (req: AuthenticatedRequest, res) => {
    const { operationId } = req.params;

    try {
      // Check if user has access to this workflow
      const workflowInfo = await redisClient.get(`workflow:${operationId}`);
      if (workflowInfo && workflowInfo.userId !== req.user!.id) {
        throw new BFFError(403, 'Access denied to workflow', 'WORKFLOW_ACCESS_DENIED');
      }

      // Set up Server-Sent Events
      res.writeHead(200, {
        'Content-Type': 'text/event-stream',
        'Cache-Control': 'no-cache',
        'Connection': 'keep-alive',
        'Access-Control-Allow-Origin': '*',
        'Access-Control-Allow-Headers': 'Cache-Control',
      });

      // Send initial status
      const initialStatus = await apiClient.getWorkflowStatus(operationId, req.token!, req.user!.tenantId);
      res.write(`data: ${JSON.stringify(initialStatus)}\n\n`);

      // Set up polling for status updates
      const pollInterval = setInterval(async () => {
        try {
          const status = await apiClient.getWorkflowStatus(operationId, req.token!, req.user!.tenantId);
          res.write(`data: ${JSON.stringify(status)}\n\n`);

          // Stop polling if workflow is complete
          if (['completed', 'failed', 'cancelled'].includes(status.status)) {
            clearInterval(pollInterval);
            res.write('event: close\ndata: Workflow completed\n\n');
            res.end();
          }
        } catch (error) {
          clearInterval(pollInterval);
          res.write(`event: error\ndata: ${JSON.stringify({ error: error.message })}\n\n`);
          res.end();
        }
      }, 2000); // Poll every 2 seconds

      // Clean up on client disconnect
      req.on('close', () => {
        clearInterval(pollInterval);
      });

    } catch (error) {
      if (error.status === 404) {
        throw new BFFError(404, 'Workflow not found', 'WORKFLOW_NOT_FOUND');
      }
      throw error;
    }
  }));

  // Workflow templates/presets
  router.get('/templates', requireAuth, requireTenant, asyncHandler(async (req: TenantRequest, res) => {
    try {
      const cacheKey = `workflow_templates:${req.tenantId}`;
      let templates = await redisClient.getAggregatedData(cacheKey);

      if (!templates) {
        // Define common workflow templates based on tenant features
        templates = [
          {
            id: 'user-onboarding',
            name: 'User Onboarding',
            description: 'Complete user onboarding process',
            workflowType: 'user-onboarding',
            category: 'user-management',
            estimatedDuration: 300, // 5 minutes
            requiredPermissions: ['user:create'],
            parameters: [
              { name: 'email', type: 'string', required: true },
              { name: 'firstName', type: 'string', required: false },
              { name: 'lastName', type: 'string', required: false },
              { name: 'role', type: 'string', required: true, default: 'user' },
            ],
          },
          {
            id: 'tenant-switch',
            name: 'Tenant Switch',
            description: 'Switch user to different tenant',
            workflowType: 'tenant-switch',
            category: 'tenant-management',
            estimatedDuration: 30, // 30 seconds
            requiredPermissions: ['tenant:switch'],
            parameters: [
              { name: 'targetTenantId', type: 'string', required: true },
            ],
          },
          {
            id: 'bulk-user-import',
            name: 'Bulk User Import',
            description: 'Import multiple users from CSV',
            workflowType: 'bulk-user-import',
            category: 'user-management',
            estimatedDuration: 600, // 10 minutes
            requiredPermissions: ['user:bulk-create'],
            parameters: [
              { name: 'csvData', type: 'string', required: true },
              { name: 'defaultRole', type: 'string', required: false, default: 'user' },
              { name: 'sendInvites', type: 'boolean', required: false, default: true },
            ],
          },
        ];

        // Filter templates based on tenant features and user permissions
        templates = templates.filter((template: any) => {
          const hasFeature = template.category === 'user-management' 
            ? req.tenant!.features.includes('user_management')
            : true;
          
          const hasPermission = template.requiredPermissions.every((permission: string) =>
            req.user!.permissions.includes(permission)
          );

          return hasFeature && hasPermission;
        });

        // Cache for 10 minutes
        await redisClient.cacheAggregatedData(cacheKey, templates, 600);
      }

      res.json({ templates });
    } catch (error) {
      throw error;
    }
  }));

  return router;
}