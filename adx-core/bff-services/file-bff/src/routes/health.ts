import express from 'express';
import { redisClient } from '../services/redis.js';
import { createLogger } from '../utils/logger.js';

const router = express.Router();
const logger = createLogger('health-routes');

// GET /health - Basic health check
router.get('/', async (req, res) => {
  try {
    const health = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      service: 'file-bff',
      version: process.env.npm_package_version || '1.0.0',
      uptime: process.uptime(),
      environment: process.env.NODE_ENV || 'development',
    };

    res.json(health);
  } catch (error) {
    logger.error('Health check error', { error });
    res.status(500).json({
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      error: 'Health check failed',
    });
  }
});

// GET /health/detailed - Detailed health check with dependencies
router.get('/detailed', async (req, res) => {
  const checks = {
    service: { status: 'healthy', timestamp: new Date().toISOString() },
    redis: { status: 'unknown', timestamp: new Date().toISOString() },
    apiGateway: { status: 'unknown', timestamp: new Date().toISOString() },
  };

  let overallStatus = 'healthy';

  // Check Redis connection
  try {
    const start = Date.now();
    await redisClient.ping();
    const duration = Date.now() - start;
    
    checks.redis = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      responseTime: `${duration}ms`,
    };
  } catch (error) {
    checks.redis = {
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      error: error instanceof Error ? error.message : 'Redis connection failed',
    };
    overallStatus = 'degraded';
  }

  // Check API Gateway connection
  try {
    const axios = (await import('axios')).default;
    const apiGatewayUrl = process.env.API_GATEWAY_URL || 'http://localhost:8080';
    const start = Date.now();
    
    await axios.get(`${apiGatewayUrl}/health`, {
      timeout: 5000,
    });
    
    const duration = Date.now() - start;
    
    checks.apiGateway = {
      status: 'healthy',
      timestamp: new Date().toISOString(),
      responseTime: `${duration}ms`,
      url: apiGatewayUrl,
    };
  } catch (error: any) {
    checks.apiGateway = {
      status: 'unhealthy',
      timestamp: new Date().toISOString(),
      error: error.message || 'API Gateway connection failed',
      url: process.env.API_GATEWAY_URL || 'http://localhost:8080',
    };
    overallStatus = 'degraded';
  }

  const response = {
    status: overallStatus,
    timestamp: new Date().toISOString(),
    service: 'file-bff',
    version: process.env.npm_package_version || '1.0.0',
    uptime: process.uptime(),
    environment: process.env.NODE_ENV || 'development',
    checks,
    system: {
      memory: {
        used: process.memoryUsage().heapUsed,
        total: process.memoryUsage().heapTotal,
        external: process.memoryUsage().external,
        rss: process.memoryUsage().rss,
      },
      cpu: process.cpuUsage(),
      platform: process.platform,
      nodeVersion: process.version,
    },
  };

  const statusCode = overallStatus === 'healthy' ? 200 : 503;
  res.status(statusCode).json(response);
});

// GET /health/ready - Readiness probe
router.get('/ready', async (req, res) => {
  try {
    // Check if all critical dependencies are available
    await redisClient.ping();
    
    res.json({
      status: 'ready',
      timestamp: new Date().toISOString(),
      service: 'file-bff',
    });
  } catch (error) {
    logger.error('Readiness check failed', { error });
    res.status(503).json({
      status: 'not ready',
      timestamp: new Date().toISOString(),
      service: 'file-bff',
      error: 'Critical dependencies unavailable',
    });
  }
});

// GET /health/live - Liveness probe
router.get('/live', (req, res) => {
  res.json({
    status: 'alive',
    timestamp: new Date().toISOString(),
    service: 'file-bff',
    uptime: process.uptime(),
  });
});

export { router as healthRoutes };