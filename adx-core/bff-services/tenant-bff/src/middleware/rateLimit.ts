import { Request, Response, NextFunction } from 'express';
import { RedisClient } from '../services/redis.js';
import { AuthenticatedRequest } from './auth.js';

export interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
  keyGenerator?: (req: Request) => string;
  skipSuccessfulRequests?: boolean;
  skipFailedRequests?: boolean;
  message?: string;
}

export const rateLimitConfigs = {
  general: {
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 100,
    message: 'Too many requests from this IP, please try again later',
  },
  tenantSwitch: {
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 10,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return `tenant_switch:${authReq.user?.id || req.ip}`;
    },
    message: 'Too many tenant switch attempts, please try again later',
  },
  analytics: {
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 30,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return `analytics:${authReq.user?.id || req.ip}`;
    },
    message: 'Too many analytics requests, please try again later',
  },
  workflows: {
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 20,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return `workflows:${authReq.user?.id || req.ip}`;
    },
    message: 'Too many workflow requests, please try again later',
  },
  configuration: {
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 15,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return `config:${authReq.user?.id || req.ip}`;
    },
    message: 'Too many configuration requests, please try again later',
  },
};

export function createRateLimitMiddleware(
  redisClient: RedisClient,
  config: RateLimitConfig
) {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      // Generate rate limit key
      const key = config.keyGenerator ? config.keyGenerator(req) : req.ip;
      
      // Get rate limit info
      const { count, remaining, resetTime } = await redisClient.incrementRateLimit(
        key,
        config.windowMs,
        config.maxRequests
      );

      // Set rate limit headers
      res.set({
        'X-RateLimit-Limit': config.maxRequests.toString(),
        'X-RateLimit-Remaining': remaining.toString(),
        'X-RateLimit-Reset': new Date(resetTime).toISOString(),
        'X-RateLimit-Window': config.windowMs.toString(),
      });

      // Check if rate limit exceeded
      if (count > config.maxRequests) {
        const retryAfter = Math.ceil((resetTime - Date.now()) / 1000);
        
        res.set('Retry-After', retryAfter.toString());
        
        return res.status(429).json({
          error: {
            code: 'RATE_LIMIT_EXCEEDED',
            message: config.message || 'Rate limit exceeded',
            details: {
              limit: config.maxRequests,
              windowMs: config.windowMs,
              retryAfter,
              resetTime: new Date(resetTime).toISOString(),
            },
          },
        });
      }

      next();
    } catch (error) {
      console.error('Rate limit middleware error:', error);
      // Continue without rate limiting if Redis is down
      next();
    }
  };
}

export function createTenantRateLimit(redisClient: RedisClient) {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      const authReq = req as AuthenticatedRequest;
      
      if (!authReq.user) {
        return next(); // Skip rate limiting for unauthenticated requests
      }

      const tenantId = authReq.user.tenantId || req.headers['x-tenant-id'] as string;
      
      if (!tenantId) {
        return next(); // Skip if no tenant context
      }

      // Different rate limits based on tenant tier
      const tenantKey = `tenant_rate_limit:${tenantId}`;
      
      // Get tenant info from cache or set default limits
      let maxRequests = 1000; // Default for professional tier
      let windowMs = 60 * 1000; // 1 minute
      
      // You could fetch tenant tier from cache and adjust limits accordingly
      // const tenant = await redisClient.getCachedTenant(tenantId);
      // if (tenant) {
      //   switch (tenant.subscriptionTier) {
      //     case 'free':
      //       maxRequests = 100;
      //       break;
      //     case 'professional':
      //       maxRequests = 1000;
      //       break;
      //     case 'enterprise':
      //       maxRequests = 10000;
      //       break;
      //   }
      // }

      const { count, remaining, resetTime } = await redisClient.incrementRateLimit(
        tenantKey,
        windowMs,
        maxRequests
      );

      // Set tenant-specific rate limit headers
      res.set({
        'X-Tenant-RateLimit-Limit': maxRequests.toString(),
        'X-Tenant-RateLimit-Remaining': remaining.toString(),
        'X-Tenant-RateLimit-Reset': new Date(resetTime).toISOString(),
      });

      if (count > maxRequests) {
        const retryAfter = Math.ceil((resetTime - Date.now()) / 1000);
        
        res.set('Retry-After', retryAfter.toString());
        
        return res.status(429).json({
          error: {
            code: 'TENANT_RATE_LIMIT_EXCEEDED',
            message: 'Tenant rate limit exceeded',
            details: {
              tenantId,
              limit: maxRequests,
              windowMs,
              retryAfter,
              resetTime: new Date(resetTime).toISOString(),
            },
          },
        });
      }

      next();
    } catch (error) {
      console.error('Tenant rate limit middleware error:', error);
      next(); // Continue without rate limiting if there's an error
    }
  };
}

export function createUserRateLimit(redisClient: RedisClient) {
  return async (req: Request, res: Response, next: NextFunction) => {
    try {
      const authReq = req as AuthenticatedRequest;
      
      if (!authReq.user) {
        return next(); // Skip rate limiting for unauthenticated requests
      }

      const userId = authReq.user.id;
      const userKey = `user_rate_limit:${userId}`;
      
      // User-specific rate limits
      const maxRequests = 500; // Per user per minute
      const windowMs = 60 * 1000; // 1 minute

      const { count, remaining, resetTime } = await redisClient.incrementRateLimit(
        userKey,
        windowMs,
        maxRequests
      );

      // Set user-specific rate limit headers
      res.set({
        'X-User-RateLimit-Limit': maxRequests.toString(),
        'X-User-RateLimit-Remaining': remaining.toString(),
        'X-User-RateLimit-Reset': new Date(resetTime).toISOString(),
      });

      if (count > maxRequests) {
        const retryAfter = Math.ceil((resetTime - Date.now()) / 1000);
        
        res.set('Retry-After', retryAfter.toString());
        
        return res.status(429).json({
          error: {
            code: 'USER_RATE_LIMIT_EXCEEDED',
            message: 'User rate limit exceeded',
            details: {
              userId,
              limit: maxRequests,
              windowMs,
              retryAfter,
              resetTime: new Date(resetTime).toISOString(),
            },
          },
        });
      }

      next();
    } catch (error) {
      console.error('User rate limit middleware error:', error);
      next(); // Continue without rate limiting if there's an error
    }
  };
}

export function createEndpointRateLimit(
  redisClient: RedisClient,
  endpoint: string,
  config: Partial<RateLimitConfig> = {}
) {
  const fullConfig: RateLimitConfig = {
    windowMs: 60 * 1000, // 1 minute
    maxRequests: 60,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return `endpoint:${endpoint}:${authReq.user?.id || req.ip}`;
    },
    message: `Too many requests to ${endpoint}, please try again later`,
    ...config,
  };

  return createRateLimitMiddleware(redisClient, fullConfig);
}