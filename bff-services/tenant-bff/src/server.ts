import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import dotenv from 'dotenv';
import { createServer } from 'http';
import { RedisClient } from './services/redis.js';
import { ApiClient } from './services/apiClient.js';
import { createAuthMiddleware, optionalAuth } from './middleware/auth.js';
import { createTenantMiddleware, injectTenantContext } from './middleware/tenant.js';
import { createRateLimitMiddleware, rateLimitConfigs, createTenantRateLimit, createUserRateLimit } from './middleware/rateLimit.js';
import { errorHandler, notFoundHandler } from './middleware/errorHandler.js';
import { createTenantRoutes } from './routes/tenants.js';
import { createWorkflowRoutes } from './routes/workflows.js';
import { createAggregatedRoutes } from './routes/aggregated.js';

// Load environment variables
dotenv.config();

const app = express();
const port = process.env.PORT || 4002;
const jwtSecret = process.env.JWT_SECRET || 'your-jwt-secret-key';

// Create HTTP server
const server = createServer(app);

// Initialize services
const redisClient = new RedisClient(process.env.REDIS_URL || 'redis://localhost:6379');

const apiClient = new ApiClient({
  apiGatewayUrl: process.env.API_GATEWAY_URL || 'http://localhost:8080',
  tenantServiceUrl: process.env.TENANT_SERVICE_URL || 'http://localhost:8085',
  userServiceUrl: process.env.USER_SERVICE_URL || 'http://localhost:8082',
  authServiceUrl: process.env.AUTH_SERVICE_URL || 'http://localhost:8081',
  workflowServiceUrl: process.env.WORKFLOW_SERVICE_URL || 'http://localhost:8084',
  timeout: 10000,
});

// Security middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'", "ws:", "wss:"],
      fontSrc: ["'self'"],
      objectSrc: ["'none'"],
      mediaSrc: ["'self'"],
      frameSrc: ["'none'"],
    },
  },
  crossOriginEmbedderPolicy: false,
}));

app.use(compression());

// CORS configuration
const corsOptions = {
  origin: (origin: string | undefined, callback: (err: Error | null, allow?: boolean) => void) => {
    const allowedOrigins = process.env.CORS_ORIGIN?.split(',') || [
      'http://localhost:3000',
      'http://localhost:3002',
      'https://app.adxcore.com',
      'https://tenant.adxcore.com',
    ];
    
    // Allow requests with no origin (mobile apps, etc.)
    if (!origin) return callback(null, true);
    
    if (allowedOrigins.includes(origin)) {
      callback(null, true);
    } else {
      callback(new Error('Not allowed by CORS'));
    }
  },
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS', 'PATCH'],
  allowedHeaders: [
    'Content-Type',
    'Authorization',
    'X-Tenant-ID',
    'X-Request-ID',
    'X-Forwarded-For',
    'User-Agent',
  ],
  exposedHeaders: [
    'X-RateLimit-Limit',
    'X-RateLimit-Remaining',
    'X-RateLimit-Reset',
    'X-Tenant-RateLimit-Limit',
    'X-Tenant-RateLimit-Remaining',
    'X-Tenant-RateLimit-Reset',
    'X-User-RateLimit-Limit',
    'X-User-RateLimit-Remaining',
    'X-User-RateLimit-Reset',
    'X-Tenant-ID',
    'X-Tenant-Name',
    'X-Tenant-Tier',
    'Retry-After',
  ],
};

app.use(cors(corsOptions));

// Body parsing middleware
app.use(express.json({ 
  limit: '10mb',
  verify: (req, res, buf) => {
    // Store raw body for webhook verification if needed
    (req as any).rawBody = buf;
  },
}));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Request ID middleware
app.use((req, res, next) => {
  req.headers['x-request-id'] = req.headers['x-request-id'] || 
    `req_${Date.now()}_${Math.random().toString(36).substring(2, 9)}`;
  res.set('X-Request-ID', req.headers['x-request-id'] as string);
  next();
});

// Logging middleware
app.use((req, res, next) => {
  const start = Date.now();
  
  res.on('finish', () => {
    const duration = Date.now() - start;
    const logData = {
      method: req.method,
      url: req.url,
      status: res.statusCode,
      duration: `${duration}ms`,
      userAgent: req.headers['user-agent'],
      ip: req.ip,
      requestId: req.headers['x-request-id'],
      tenantId: req.headers['x-tenant-id'],
    };
    
    if (res.statusCode >= 400) {
      console.error('Request error:', logData);
    } else {
      console.log('Request:', logData);
    }
  });
  
  next();
});

// Global rate limiting
const globalRateLimit = createRateLimitMiddleware(redisClient, rateLimitConfigs.general);
app.use(globalRateLimit);

// Authentication middleware (optional for public endpoints)
app.use(optionalAuth(redisClient, jwtSecret));

// Tenant context middleware
const tenantMiddleware = createTenantMiddleware(redisClient, apiClient);
app.use(tenantMiddleware);

// Tenant and user rate limiting
app.use(createTenantRateLimit(redisClient));
app.use(createUserRateLimit(redisClient));

// Inject tenant context into responses
app.use(injectTenantContext);

// Health check endpoint
app.get('/health', async (req, res) => {
  try {
    // Check service health
    const healthChecks = await Promise.allSettled([
      redisClient.isHealthy() ? Promise.resolve('healthy') : Promise.reject('unhealthy'),
      apiClient.healthCheck(),
    ]);

    const redisHealth = healthChecks[0].status === 'fulfilled' ? 'healthy' : 'unhealthy';
    const apiHealth = healthChecks[1].status === 'fulfilled' ? 'healthy' : 'unhealthy';

    const overallHealth = redisHealth === 'healthy' && apiHealth === 'healthy' ? 'healthy' : 'degraded';

    const healthData = {
      status: overallHealth,
      service: 'tenant-bff',
      timestamp: new Date().toISOString(),
      version: '1.0.0',
      uptime: process.uptime(),
      memory: process.memoryUsage(),
      checks: {
        redis: redisHealth,
        api: apiHealth,
      },
    };

    res.status(overallHealth === 'healthy' ? 200 : 503).json(healthData);
  } catch (error: any) {
    res.status(503).json({
      status: 'unhealthy',
      service: 'tenant-bff',
      timestamp: new Date().toISOString(),
      error: error.message,
    });
  }
});

// API routes
app.use('/api/tenants', createTenantRoutes(redisClient, apiClient, jwtSecret));
app.use('/api/workflows', createWorkflowRoutes(redisClient, apiClient, jwtSecret));
app.use('/api/aggregated', createAggregatedRoutes(redisClient, apiClient, jwtSecret));

// Cache management endpoints (admin only)
const authMiddleware = createAuthMiddleware(redisClient, jwtSecret);

app.post('/api/cache/clear', authMiddleware, async (req, res) => {
  // Check if user has admin permissions
  const authReq = req as any;
  if (!authReq.user?.roles.includes('admin')) {
    return res.status(403).json({ error: 'Admin access required' });
  }

  try {
    const pattern = req.body.pattern || '*';
    const cleared = await redisClient.flushPattern(pattern);
    
    return res.json({
      message: 'Cache cleared successfully',
      pattern,
      keysCleared: cleared,
    });
  } catch (error: any) {
    return res.status(500).json({
      error: 'Failed to clear cache',
      message: error.message,
    });
  }
});

// Metrics endpoint
app.get('/api/metrics', authMiddleware, async (req, res) => {
  try {
    const authReq = req as any;
    if (!authReq.user?.permissions.includes('system:metrics')) {
      return res.status(403).json({ error: 'Metrics access denied' });
    }

    const metrics = {
      timestamp: new Date().toISOString(),
      uptime: process.uptime(),
      memory: process.memoryUsage(),
      redis: {
        connected: redisClient.isHealthy(),
        info: await redisClient.getInfo().catch(() => null),
      },
      // Add more metrics as needed
    };

    return res.json(metrics);
  } catch (error: any) {
    return res.status(500).json({
      error: 'Failed to get metrics',
      message: error.message,
    });
  }
});

// 404 handler
app.use('*', notFoundHandler);

// Error handling middleware (must be last)
app.use(errorHandler);

// Start server
server.listen(port, () => {
  console.log(`🚀 Tenant BFF service running on port ${port}`);
  console.log(`📊 Health check: http://localhost:${port}/health`);
  console.log(`🔧 Environment: ${process.env.NODE_ENV || 'development'}`);
  console.log(`📡 API Gateway: ${process.env.API_GATEWAY_URL || 'http://localhost:8080'}`);
  console.log(`🏢 Tenant Service: ${process.env.TENANT_SERVICE_URL || 'http://localhost:8085'}`);
  console.log(`👤 User Service: ${process.env.USER_SERVICE_URL || 'http://localhost:8082'}`);
  console.log(`🔐 Auth Service: ${process.env.AUTH_SERVICE_URL || 'http://localhost:8081'}`);
  console.log(`⚡ Workflow Service: ${process.env.WORKFLOW_SERVICE_URL || 'http://localhost:8084'}`);
  console.log(`🗄️  Redis: ${process.env.REDIS_URL || 'redis://localhost:6379'}`);
});

// Graceful shutdown
const gracefulShutdown = async (signal: string) => {
  console.log(`\n${signal} received, shutting down gracefully...`);
  
  // Stop accepting new connections
  server.close(async () => {
    console.log('HTTP server closed');
    
    try {
      // Disconnect from Redis
      await redisClient.disconnect();
      console.log('Redis client disconnected');
      
      console.log('Graceful shutdown completed');
      process.exit(0);
    } catch (error) {
      console.error('Error during shutdown:', error);
      process.exit(1);
    }
  });
  
  // Force shutdown after 30 seconds
  setTimeout(() => {
    console.error('Forced shutdown after timeout');
    process.exit(1);
  }, 30000);
};

process.on('SIGTERM', () => gracefulShutdown('SIGTERM'));
process.on('SIGINT', () => gracefulShutdown('SIGINT'));

// Handle uncaught exceptions
process.on('uncaughtException', (error) => {
  console.error('Uncaught Exception:', error);
  gracefulShutdown('UNCAUGHT_EXCEPTION');
});

process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled Rejection at:', promise, 'reason:', reason);
  gracefulShutdown('UNHANDLED_REJECTION');
});

export default app;