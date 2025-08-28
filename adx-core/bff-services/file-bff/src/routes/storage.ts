import express from 'express';
import axios from 'axios';
import { TenantRequest } from '../middleware/tenant.js';
import { cacheService } from '../services/redis.js';
import { createLogger } from '../utils/logger.js';

const router = express.Router();
const logger = createLogger('storage-routes');

// Helper function to make API Gateway requests
const makeApiRequest = async (req: TenantRequest, endpoint: string, options: any = {}) => {
  const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
  const timeout = parseInt(process.env.API_GATEWAY_TIMEOUT || '30000');

  const config = {
    ...options,
    url: `${apiGatewayUrl}${endpoint}`,
    timeout,
    headers: {
      'Authorization': req.headers.authorization,
      'X-Tenant-ID': req.tenantId,
      'X-Request-ID': req.requestId,
      'Content-Type': 'application/json',
      ...options.headers,
    },
  };

  try {
    const response = await axios(config);
    return response.data;
  } catch (error: any) {
    logger.error('API Gateway request failed', {
      endpoint,
      error: error.message,
      status: error.response?.status,
      requestId: req.requestId,
    });
    throw error;
  }
};

// GET /api/storage/quota - Get storage quota with caching
router.get('/quota', async (req: TenantRequest, res) => {
  try {
    // Try cache first
    const cachedQuota = await cacheService.getCachedQuota(req.tenantId!);
    if (cachedQuota) {
      logger.debug('Returning cached storage quota', {
        tenantId: req.tenantId,
        requestId: req.requestId,
      });
      return res.json(cachedQuota);
    }

    // Fetch from API Gateway
    const quota = await makeApiRequest(req, '/api/v1/storage/quota');

    // Cache the result
    await cacheService.cacheQuota(req.tenantId!, quota);

    logger.info('Storage quota retrieved', {
      tenantId: req.tenantId,
      used: quota.used,
      limit: quota.limit,
      percentage: quota.percentage,
      requestId: req.requestId,
    });

    res.json(quota);
  } catch (error) {
    logger.error('Get storage quota error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/storage/usage - Get detailed storage usage
router.get('/usage', async (req: TenantRequest, res) => {
  try {
    const { breakdown = true, period = '30d' } = req.query;

    const params = new URLSearchParams({
      breakdown: breakdown.toString(),
      period: period.toString(),
    });

    const usage = await makeApiRequest(req, `/api/v1/storage/usage?${params.toString()}`);

    logger.info('Storage usage retrieved', {
      tenantId: req.tenantId,
      period,
      includeBreakdown: breakdown,
      requestId: req.requestId,
    });

    res.json(usage);
  } catch (error) {
    logger.error('Get storage usage error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/storage/analytics - Get storage analytics
router.get('/analytics', async (req: TenantRequest, res) => {
  try {
    const { 
      timeframe = '7d',
      metrics = 'usage,growth,types',
      granularity = 'daily'
    } = req.query;

    const params = new URLSearchParams({
      timeframe: timeframe.toString(),
      metrics: metrics.toString(),
      granularity: granularity.toString(),
    });

    const analytics = await makeApiRequest(req, `/api/v1/storage/analytics?${params.toString()}`);

    logger.info('Storage analytics retrieved', {
      tenantId: req.tenantId,
      timeframe,
      metrics,
      granularity,
      requestId: req.requestId,
    });

    res.json(analytics);
  } catch (error) {
    logger.error('Get storage analytics error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/storage/providers - Get available storage providers
router.get('/providers', async (req: TenantRequest, res) => {
  try {
    const providers = await makeApiRequest(req, '/api/v1/storage/providers');

    logger.info('Storage providers retrieved', {
      tenantId: req.tenantId,
      providerCount: providers.length,
      requestId: req.requestId,
    });

    res.json(providers);
  } catch (error) {
    logger.error('Get storage providers error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/storage/cleanup - Initiate storage cleanup workflow
router.post('/cleanup', async (req: TenantRequest, res) => {
  try {
    const {
      rules = [],
      dryRun = true,
      synchronous = false,
    } = req.body;

    const cleanupRequest = {
      rules,
      dryRun,
      synchronous,
    };

    const result = await makeApiRequest(req, '/api/v1/workflows/storage-cleanup', {
      method: 'POST',
      data: cleanupRequest,
    });

    // Invalidate quota cache since cleanup might change usage
    if (!dryRun) {
      const quotaCacheKey = cacheService.generateQuotaKey(req.tenantId!);
      await cacheService.del(quotaCacheKey);
    }

    logger.info('Storage cleanup initiated', {
      tenantId: req.tenantId,
      ruleCount: rules.length,
      dryRun,
      synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('Storage cleanup error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/storage/optimize - Initiate storage optimization workflow
router.post('/optimize', async (req: TenantRequest, res) => {
  try {
    const {
      operations = ['deduplication', 'compression'],
      targetSavings = 10, // percentage
      synchronous = false,
    } = req.body;

    const optimizeRequest = {
      operations,
      targetSavings,
      synchronous,
    };

    const result = await makeApiRequest(req, '/api/v1/workflows/storage-optimize', {
      method: 'POST',
      data: optimizeRequest,
    });

    // Invalidate quota cache since optimization might change usage
    const quotaCacheKey = cacheService.generateQuotaKey(req.tenantId!);
    await cacheService.del(quotaCacheKey);

    logger.info('Storage optimization initiated', {
      tenantId: req.tenantId,
      operations,
      targetSavings,
      synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('Storage optimization error', { error, requestId: req.requestId });
    throw error;
  }
});

// GET /api/storage/health - Get storage system health
router.get('/health', async (req: TenantRequest, res) => {
  try {
    const health = await makeApiRequest(req, '/api/v1/storage/health');

    logger.info('Storage health retrieved', {
      tenantId: req.tenantId,
      overallStatus: health.status,
      requestId: req.requestId,
    });

    res.json(health);
  } catch (error) {
    logger.error('Get storage health error', { error, requestId: req.requestId });
    throw error;
  }
});

// POST /api/storage/migrate - Initiate storage migration workflow
router.post('/migrate', async (req: TenantRequest, res) => {
  try {
    const {
      sourceProvider,
      targetProvider,
      fileIds = [],
      migrationOptions = {},
      synchronous = false,
    } = req.body;

    if (!sourceProvider || !targetProvider) {
      return res.status(400).json({
        error: 'Bad Request',
        message: 'Source and target providers are required',
        timestamp: new Date().toISOString(),
      });
    }

    const migrationRequest = {
      sourceProvider,
      targetProvider,
      fileIds,
      migrationOptions: {
        preserveMetadata: true,
        createBackup: true,
        verifyIntegrity: true,
        deleteSource: false,
        ...migrationOptions,
      },
      synchronous,
    };

    const result = await makeApiRequest(req, '/api/v1/workflows/storage-migrate', {
      method: 'POST',
      data: migrationRequest,
    });

    // Invalidate file cache since files might change location
    await cacheService.invalidateFileCache(req.tenantId!);

    logger.info('Storage migration initiated', {
      tenantId: req.tenantId,
      sourceProvider,
      targetProvider,
      fileCount: fileIds.length,
      synchronous,
      operationId: result.operationId,
      requestId: req.requestId,
    });

    if (synchronous) {
      res.json(result);
    } else {
      res.status(202).json(result);
    }
  } catch (error) {
    logger.error('Storage migration error', { error, requestId: req.requestId });
    throw error;
  }
});

// DELETE /api/storage/cache - Clear storage-related caches
router.delete('/cache', async (req: TenantRequest, res) => {
  try {
    const { type = 'all' } = req.query;

    let clearedCount = 0;

    switch (type) {
      case 'quota':
        const quotaKey = cacheService.generateQuotaKey(req.tenantId!);
        await cacheService.del(quotaKey);
        clearedCount = 1;
        break;
      case 'files':
        clearedCount = await cacheService.invalidateFileCache(req.tenantId!);
        break;
      case 'all':
      default:
        const quotaClearCount = await cacheService.invalidatePattern(`quota:${req.tenantId}*`);
        const filesClearCount = await cacheService.invalidateFileCache(req.tenantId!);
        clearedCount = quotaClearCount + filesClearCount;
        break;
    }

    logger.info('Storage cache cleared', {
      tenantId: req.tenantId,
      type,
      clearedCount,
      requestId: req.requestId,
    });

    res.json({
      message: 'Cache cleared successfully',
      type,
      clearedCount,
      timestamp: new Date().toISOString(),
    });
  } catch (error) {
    logger.error('Clear storage cache error', { error, requestId: req.requestId });
    throw error;
  }
});

export { router as storageRoutes };