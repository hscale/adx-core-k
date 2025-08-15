import express from 'express';
import cors from 'cors';
import helmet from 'helmet';
import compression from 'compression';
import dotenv from 'dotenv';
import { createLogger } from './utils/logger';
import { errorHandler } from './middleware/errorHandler';
import { authMiddleware } from './middleware/auth';
import { tenantMiddleware } from './middleware/tenant';
import { rateLimitMiddleware } from './middleware/rateLimit';
import { healthRoutes } from './routes/health';
import { marketplaceRoutes } from './routes/marketplace';
import { moduleRoutes } from './routes/modules';
import { developmentRoutes } from './routes/development';
import { workflowRoutes } from './routes/workflows';

// Load environment variables
dotenv.config();

const app = express();
const port = process.env.PORT || 4006;
const logger = createLogger('module-bff');

// Security middleware
app.use(helmet({
  contentSecurityPolicy: {
    directives: {
      defaultSrc: ["'self'"],
      styleSrc: ["'self'", "'unsafe-inline'"],
      scriptSrc: ["'self'"],
      imgSrc: ["'self'", "data:", "https:"],
      connectSrc: ["'self'", "ws:", "wss:"],
    },
  },
}));

// CORS configuration
app.use(cors({
  origin: process.env.NODE_ENV === 'production' 
    ? ['https://app.adxcore.com', 'https://staging.adxcore.com']
    : ['http://localhost:3000', 'http://localhost:3006'],
  credentials: true,
  methods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
  allowedHeaders: ['Content-Type', 'Authorization', 'X-Tenant-ID'],
}));

// Compression and parsing middleware
app.use(compression());
app.use(express.json({ limit: '10mb' }));
app.use(express.urlencoded({ extended: true, limit: '10mb' }));

// Rate limiting
app.use(rateLimitMiddleware);

// Request logging
app.use((req, res, next) => {
  logger.info('Request received', {
    method: req.method,
    url: req.url,
    userAgent: req.get('User-Agent'),
    ip: req.ip,
  });
  next();
});

// Health check routes (no auth required)
app.use('/api/health', healthRoutes);

// Authentication middleware for protected routes
app.use('/api', authMiddleware);
app.use('/api', tenantMiddleware);

// API routes
app.use('/api/marketplace', marketplaceRoutes);
app.use('/api/modules', moduleRoutes);
app.use('/api/development', developmentRoutes);
app.use('/api/workflows', workflowRoutes);

// 404 handler
app.use('*', (req, res) => {
  res.status(404).json({
    error: {
      code: 'NOT_FOUND',
      message: 'Endpoint not found',
    },
    timestamp: new Date().toISOString(),
  });
});

// Error handling middleware
app.use(errorHandler);

// Graceful shutdown
const server = app.listen(port, () => {
  logger.info(`Module BFF server running on port ${port}`, {
    port,
    nodeEnv: process.env.NODE_ENV,
  });
});

process.on('SIGTERM', () => {
  logger.info('SIGTERM received, shutting down gracefully');
  server.close(() => {
    logger.info('Server closed');
    process.exit(0);
  });
});

process.on('SIGINT', () => {
  logger.info('SIGINT received, shutting down gracefully');
  server.close(() => {
    logger.info('Server closed');
    process.exit(0);
  });
});

export default app;