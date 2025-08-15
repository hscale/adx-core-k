import express from 'express';
import axios from 'axios';
import { z } from 'zod';
import { TenantRequest } from '../middleware/tenant.js';
import { workflowRateLimit } from '../middleware/rateLimit.js';
import { cacheService } from '../services/redis.js';
import { createLogger } from '../utils/logger.js';
import { ValidationError, NotFoundError } from '../middleware/errorHandler.js';

const router = express.Router();
const logger = createLogger('workflows-routes');

// Validation schemas
const workflowRequestSchema = z.object({
  synchronous: z.boolean().optional().default(false),
});

const fileUploadWorkflowSchema = workflowRequestSchema.extend({
  files: z.array(z.any()).min(1),
  path: z.string().optional().default('/'),
  options: z.object({
    virusScan: z.boolean().optional().default(true),
    generateThumbnails: z.boolean().optional().default(true),
    extractMetadata: z.boolean().optional().default(true),
    processAI: z.boolean().optional().default(false),
  }).optional(),
});

const fileProcessingWorkflowSchema = workflowRequestSchema.extend({
  fileIds: z.array(z.string()).min(1),
  operations: z.array(z.enum(['thumbnail', 'metadata', 'virus_scan', 'ai_analysis'])).min(1),
});

const fileSharingWorkflowSchema = workflowRequestSchema.extend({
  fileIds: z.array(z.string()).min(1),
  shareSettings: z.object({
    shareType: z.enum(['public', 'email', 'team']),
    permissions: z.enum(['read', 'write', 'admin']),
    sharedWith: z.string().optional(),
    expiresAt: z.string().optional(),
    password: z.string().optional(),
    downloadAllowed: z.boolean().optional().default(true),
    message: z.string().optional(),
    notifyUsers: z.boolean().optional().default(true),
  }),
});

const fileMigrationWorkflowSchema = workflowRequestSchema.extend({
  fileIds: z.array(z.string()).min(1),
  targetStorage: z.enum(['local', 's3', 'gcs', 'azure']),
  migrationOptions: z.object({
    preserveMetadata: z.boolean().optional().default(true),
    createBackup: z.boolean().optional().default(true),
    verifyIntegrity: z.boolean().optional().default(true),
    deleteSource: z.boolean().optional().default(false),
  }).optional(),
});

const bulkFileOperationSchema = workflowRequestSchema.extend({
  operation: z.enum(['move', 'copy', 'delete', 'archive', 'restore']),
  fileIds: z.array(z.string()).min(1),
  targetPath: z.string().optional(),
  options: z.object({
    preserveStructure: z.boolean().optional().default(true),
    overwriteExisting: z.boolean().optional().default(false),
    createBackup: z.boolean().optional().default(false),
  }).optional(),
});

// Helper function to make API Gateway requests
const makeWorkflowRequest = async (req: TenantRequest, workflowType: string, data: any) => {
  const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
  const timeout = parseInt(process.env.API_GATEWAY_TIMEOUT || '30000');

  try {
    const response = await axios({
      method: 'POST',
      url: `${apiGatewayUrl}/api/v1/workflows/${workflowType}`,
      data,
      timeout,
      headers: {
        'Authorization': req.headers.authorization,
        'X-Tenant-ID': req.tenantId,
        'X-Request-ID': req.requestId,
        'Content-Type': 'application/json',
      },
    });

    return response.data;
  } catch (error: any) {
    logger.error('Workflow request failed', {
      workflowType,
      error: error.message,
      status: error.response?.status,
      requestId: req.requestId,
    });

    if (error.response?.status === 404) {
      throw new NotFoundError(`Workflow type '${workflowType}' not found`);
    }

    throw error;
  }
};

// POST /api/workflows/file-upload - File upload workflow
router.post('/file-upload', workflowRateLimit, async (req: TenantRequest, res) => {
  try {
    const workflowData = fileUploadWorkflowSchema.parse(req.body);

    const result = await makeWorkflowRequest(req, 'file-upload', workflowData);

    // Invalidate file cache
    await cacheService.invalidateFileCache(req.tenantId!, workflowData.path);

    logger.info('File upload workflow initiated', {
      tenantId: req.tenantId,
      fileCount: workflowData.files.length,
      path: workflowData.path,
      synchronous: workflowData.synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (workflowData.synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('File upload workflow error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/workflows/file-processing - File processing workflow
router.post('/file-processing', workflowRateLimit, async (req: TenantRequest, res) => {
  try {
    const workflowData = fileProcessingWorkflowSchema.parse(req.body);

    const result = await makeWorkflowRequest(req, 'file-processing', workflowData);

    logger.info('File processing workflow initiated', {
      tenantId: req.tenantId,
      fileCount: workflowData.fileIds.length,
      operations: workflowData.operations,
      synchronous: workflowData.synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (workflowData.synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('File processing workflow error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/workflows/file-sharing - File sharing workflow
router.post('/file-sharing', workflowRateLimit, async (req: TenantRequest, res) => {
  try {
    const workflowData = fileSharingWorkflowSchema.parse(req.body);

    const result = await makeWorkflowRequest(req, 'file-sharing', workflowData);

    logger.info('File sharing workflow initiated', {
      tenantId: req.tenantId,
      fileCount: workflowData.fileIds.length,
      shareType: workflowData.shareSettings.shareType,
      synchronous: workflowData.synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (workflowData.synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('File sharing workflow error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/workflows/file-migration - File migration workflow
router.post('/file-migration', workflowRateLimit, async (req: TenantRequest, res) => {
  try {
    const workflowData = fileMigrationWorkflowSchema.parse(req.body);

    const result = await makeWorkflowRequest(req, 'file-migration', workflowData);

    // Invalidate file cache since files might change location
    await cacheService.invalidateFileCache(req.tenantId!);

    logger.info('File migration workflow initiated', {
      tenantId: req.tenantId,
      fileCount: workflowData.fileIds.length,
      targetStorage: workflowData.targetStorage,
      synchronous: workflowData.synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (workflowData.synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('File migration workflow error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/workflows/bulk-file-operation - Bulk file operations workflow
router.post('/bulk-file-operation', workflowRateLimit, async (req: TenantRequest, res) => {
  try {
    const workflowData = bulkFileOperationSchema.parse(req.body);

    const result = await makeWorkflowRequest(req, 'bulk-file-operation', workflowData);

    // Invalidate file cache
    await cacheService.invalidateFileCache(req.tenantId!);

    logger.info('Bulk file operation workflow initiated', {
      tenantId: req.tenantId,
      operation: workflowData.operation,
      fileCount: workflowData.fileIds.length,
      targetPath: workflowData.targetPath,
      synchronous: workflowData.synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (workflowData.synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('Bulk file operation workflow error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/workflows/:operationId/status - Get workflow status
router.get('/:operationId/status', async (req: TenantRequest, res) => {
  try {
    const { operationId } = req.params;

    // Try cache first for frequently checked operations
    const cacheKey = `workflow_status:${req.tenantId}:${operationId}`;
    const cachedStatus = await cacheService.get(cacheKey);
    
    if (cachedStatus && cachedStatus.status !== 'running') {
      logger.debug('Returning cached workflow status', {
        operationId,
        status: cachedStatus.status,
        requestId: req.requestId,
      });
      return res.json(cachedStatus);
    }

    const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
    const response = await axios({
      method: 'GET',
      url: `${apiGatewayUrl}/api/v1/workflows/${operationId}/status`,
      headers: {
        'Authorization': req.headers.authorization,
        'X-Tenant-ID': req.tenantId,
        'X-Request-ID': req.requestId,
      },
    });

    const status = response.data;

    // Cache completed or failed statuses
    if (status.status === 'completed' || status.status === 'failed') {
      await cacheService.set(cacheKey, status, 3600); // Cache for 1 hour
    } else {
      // Cache running status for shorter time
      await cacheService.set(cacheKey, status, 10); // Cache for 10 seconds
    }

    logger.debug('Workflow status retrieved', {
      operationId,
      status: status.status,
      requestId: req.requestId,
    });

    res.json(status);
  } catch (error: any) {
    if (error.response?.status === 404) {
      throw new NotFoundError('Workflow operation not found');
    }
    
    logger.error('Get workflow status error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/workflows/:operationId/stream - Stream workflow progress (SSE)
router.get('/:operationId/stream', async (req: TenantRequest, res) => {
  try {
    const { operationId } = req.params;

    // Set up Server-Sent Events
    res.writeHead(200, {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Headers': 'Cache-Control',
    });

    logger.info('Workflow stream started', {
      operationId,
      tenantId: req.tenantId,
      requestId: req.requestId,
    });

    // Poll workflow status and stream updates
    const pollInterval = setInterval(async () => {
      try {
        const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
        const response = await axios({
          method: 'GET',
          url: `${apiGatewayUrl}/api/v1/workflows/${operationId}/status`,
          headers: {
            'Authorization': req.headers.authorization,
            'X-Tenant-ID': req.tenantId,
            'X-Request-ID': req.requestId,
          },
        });

        const status = response.data;
        
        // Send status update
        res.write(`data: ${JSON.stringify(status)}\n\n`);

        // Stop polling if workflow is complete
        if (status.status === 'completed' || status.status === 'failed' || status.status === 'cancelled') {
          clearInterval(pollInterval);
          res.end();
          
          logger.info('Workflow stream ended', {
            operationId,
            finalStatus: status.status,
            requestId: req.requestId,
          });
        }
      } catch (error) {
        logger.error('Workflow stream polling error', {
          operationId,
          error,
          requestId: req.requestId,
        });
        
        res.write(`data: ${JSON.stringify({ error: 'Failed to get workflow status' })}\n\n`);
        clearInterval(pollInterval);
        res.end();
      }
    }, 1000); // Poll every second

    // Clean up on client disconnect
    req.on('close', () => {
      clearInterval(pollInterval);
      logger.info('Workflow stream client disconnected', {
        operationId,
        requestId: req.requestId,
      });
    });

  } catch (error) {
    logger.error('Workflow stream setup error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/workflows/:operationId/cancel - Cancel workflow
router.post('/:operationId/cancel', async (req: TenantRequest, res) => {
  try {
    const { operationId } = req.params;

    const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
    await axios({
      method: 'POST',
      url: `${apiGatewayUrl}/api/v1/workflows/${operationId}/cancel`,
      headers: {
        'Authorization': req.headers.authorization,
        'X-Tenant-ID': req.tenantId,
        'X-Request-ID': req.requestId,
      },
    });

    // Invalidate cached status
    const cacheKey = `workflow_status:${req.tenantId}:${operationId}`;
    await cacheService.del(cacheKey);

    logger.info('Workflow cancelled', {
      operationId,
      tenantId: req.tenantId,
      requestId: req.requestId,
    });

    res.json({ 
      message: 'Workflow cancellation requested',
      operationId,
      timestamp: new Date().toISOString(),
    });
  } catch (error: any) {
    if (error.response?.status === 404) {
      throw new NotFoundError('Workflow operation not found');
    }
    
    logger.error('Cancel workflow error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/workflows - List workflow operations for tenant
router.get('/', async (req: TenantRequest, res) => {
  try {
    const { status, limit = 50, offset = 0 } = req.query;

    const params = new URLSearchParams({
      limit: limit.toString(),
      offset: offset.toString(),
    });

    if (status) {
      params.append('status', status as string);
    }

    const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
    const response = await axios({
      method: 'GET',
      url: `${apiGatewayUrl}/api/v1/workflows?${params.toString()}`,
      headers: {
        'Authorization': req.headers.authorization,
        'X-Tenant-ID': req.tenantId,
        'X-Request-ID': req.requestId,
      },
    });

    logger.info('Workflow operations listed', {
      tenantId: req.tenantId,
      count: response.data.items?.length || 0,
      status,
      requestId: req.requestId,
    });

    res.json(response.data);
  } catch (error) {
    logger.error('List workflows error', { error, requestId: req.requestId });
    throw error;
  }
});

export { router as workflowRoutes };