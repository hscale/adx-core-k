import rateLimit from 'express-rate-limit';
import { Response, NextFunction } from 'express';
import { TenantRequest } from './tenant.js';
import { redisClient } from '../services/redis.js';
import { createLogger } from '../utils/logger.js';

const logger = createLogger('rate-limit');

// Custom rate limit store using Redis
class RedisRateLimitStore {
  private prefix: string;
  private windowMs: number;

  constructor(prefix: string, windowMs: number) {
    this.prefix = prefix;
    this.windowMs = windowMs;
  }

  async increment(key: string): Promise<{ totalHits: number; timeToExpire?: number }> {
    const redisKey = `${this.prefix}:${key}`;
    
    try {
      const multi = redisClient.multi();
      multi.incr(redisKey);
      multi.expire(redisKey, Math.ceil(this.windowMs / 1000));
      multi.ttl(redisKey);
      
      const results = await multi.exec();
      
      if (!results) {
        throw new Error('Redis multi command failed');
      }

      const totalHits = results[0] as number;
      const ttl = results[2] as number;
      
      return {
        totalHits,
        timeToExpire: ttl > 0 ? ttl * 1000 : undefined,
      };
    } catch (error) {
      logger.error('Redis rate limit store error', { key, error });
      // Fallback to allowing the request if Redis fails
      return { totalHits: 1 };
    }
  }

  async decrement(key: string): Promise<void> {
    const redisKey = `${this.prefix}:${key}`;
    
    try {
      await redisClient.decr(redisKey);
    } catch (error) {
      logger.error('Redis rate limit decrement error', { key, error });
    }
  }

  async resetKey(key: string): Promise<void> {
    const redisKey = `${this.prefix}:${key}`;
    
    try {
      await redisClient.del(redisKey);
    } catch (error) {
      logger.error('Redis rate limit reset error', { key, error });
    }
  }
}

// Tenant-specific rate limiting
export const rateLimitMiddleware = (
  req: TenantRequest,
  res: Response,
  next: NextFunction
): void => {
  if (!req.tenantId || !req.user) {
    next();
    return;
  }

  // Create tenant-specific rate limiter
  const tenantRateLimit = rateLimit({
    windowMs: 15 * 60 * 1000, // 15 minutes
    max: async (req: TenantRequest) => {
      // Get tenant-specific limits from context
      const quotas = req.tenantContext?.quotas;
      return quotas?.apiCalls || 100; // Default to 100 requests per window
    },
    keyGenerator: (req: TenantRequest) => {
      return `${req.tenantId}:${req.user?.id}`;
    },
    store: new RedisRateLimitStore('rate_limit', 15 * 60 * 1000) as any,
    message: {
      error: 'Too Many Requests',
      message: 'Rate limit exceeded for this tenant',
      retryAfter: '15 minutes',
    },
    standardHeaders: true,
    legacyHeaders: false,
    handler: (req: TenantRequest, res: Response) => {
      logger.warn('Rate limit exceeded', {
        tenantId: req.tenantId,
        userId: req.user?.id,
        ip: req.ip,
        requestId: req.requestId,
      });

      res.status(429).json({
        error: 'Too Many Requests',
        message: 'Rate limit exceeded for this tenant',
        retryAfter: '15 minutes',
        timestamp: new Date().toISOString(),
      });
    },
  });

  tenantRateLimit(req, res, next);
};

// File upload specific rate limiting
export const uploadRateLimit = rateLimit({
  windowMs: 60 * 1000, // 1 minute
  max: 10, // 10 uploads per minute
  keyGenerator: (req: TenantRequest) => {
    return `upload:${req.tenantId}:${req.user?.id}`;
  },
  store: new RedisRateLimitStore('upload_rate_limit', 60 * 1000) as any,
  message: {
    error: 'Upload Rate Limit Exceeded',
    message: 'Too many file uploads, please wait before uploading more files',
    retryAfter: '1 minute',
  },
  standardHeaders: true,
  legacyHeaders: false,
});

// Workflow initiation rate limiting
export const workflowRateLimit = rateLimit({
  windowMs: 5 * 60 * 1000, // 5 minutes
  max: 20, // 20 workflow initiations per 5 minutes
  keyGenerator: (req: TenantRequest) => {
    return `workflow:${req.tenantId}:${req.user?.id}`;
  },
  store: new RedisRateLimitStore('workflow_rate_limit', 5 * 60 * 1000) as any,
  message: {
    error: 'Workflow Rate Limit Exceeded',
    message: 'Too many workflow initiations, please wait before starting more workflows',
    retryAfter: '5 minutes',
  },
  standardHeaders: true,
  legacyHeaders: false,
});