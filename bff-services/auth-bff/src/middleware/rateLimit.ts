import { Request, Response, NextFunction } from 'express';
import { RedisClient } from '../services/redis.js';
import { AuthenticatedRequest } from './auth.js';
import { BFFError } from './errorHandler.js';

export interface RateLimitConfig {
  windowMs: number;
  maxRequests: number;
  keyGenerator?: (req: Request) => string;
  skipSuccessfulRequests?: boolean;
  skipFailedRequests?: boolean;
  onLimitReached?: (req: Request, res: Response) => void;
}

export interface RateLimitInfo {
  limit: number;
  current: number;
  remaining: number;
  resetTime: number;
}

export function createRateLimitMiddleware(
  redisClient: RedisClient,
  config: RateLimitConfig
) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      const key = config.keyGenerator ? config.keyGenerator(req) : generateDefaultKey(req);
      const windowSeconds = Math.floor(config.windowMs / 1000);
      
      // Get current count
      const current = await redisClient.incrementRateLimit(key, windowSeconds);
      
      // Calculate remaining and reset time
      const remaining = Math.max(0, config.maxRequests - current);
      const resetTime = Date.now() + (windowSeconds * 1000);
      
      // Set rate limit headers
      res.set({
        'X-RateLimit-Limit': config.maxRequests.toString(),
        'X-RateLimit-Remaining': remaining.toString(),
        'X-RateLimit-Reset': Math.ceil(resetTime / 1000).toString(),
        'X-RateLimit-Window': windowSeconds.toString(),
      });
      
      // Check if limit exceeded
      if (current > config.maxRequests) {
        const retryAfter = Math.ceil(windowSeconds);
        
        res.set('Retry-After', retryAfter.toString());
        
        if (config.onLimitReached) {
          config.onLimitReached(req, res);
        }
        
        throw new BFFError(429, 'Rate limit exceeded', 'RATE_LIMIT_EXCEEDED', {
          limit: config.maxRequests,
          current,
          windowMs: config.windowMs,
          retryAfter,
        });
      }
      
      next();
    } catch (error) {
      next(error);
    }
  };
}

// Default rate limiting configurations
export const rateLimitConfigs = {
  // General API rate limiting
  general: {
    windowMs: 15 * 60 * 1000, // 15 minutes
    maxRequests: 1000,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return authReq.user ? `user:${authReq.user.id}` : `ip:${req.ip}`;
    },
  },
  
  // Authentication endpoints (more restrictive)
  auth: {
    windowMs: 15 * 60 * 1000, // 15 minutes
    maxRequests: 10,
    keyGenerator: (req: Request) => `auth:${req.ip}`,
  },
  
  // Password reset (very restrictive)
  passwordReset: {
    windowMs: 60 * 60 * 1000, // 1 hour
    maxRequests: 3,
    keyGenerator: (req: Request) => `password_reset:${req.ip}`,
  },
  
  // Registration (restrictive)
  registration: {
    windowMs: 60 * 60 * 1000, // 1 hour
    maxRequests: 5,
    keyGenerator: (req: Request) => `registration:${req.ip}`,
  },
  
  // Workflow operations (moderate)
  workflow: {
    windowMs: 5 * 60 * 1000, // 5 minutes
    maxRequests: 50,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return authReq.user ? `workflow:${authReq.user.id}` : `workflow:${req.ip}`;
    },
  },
  
  // Aggregated data requests (moderate)
  aggregation: {
    windowMs: 1 * 60 * 1000, // 1 minute
    maxRequests: 30,
    keyGenerator: (req: Request) => {
      const authReq = req as AuthenticatedRequest;
      return authReq.user ? `aggregation:${authReq.user.id}` : `aggregation:${req.ip}`;
    },
  },
};

// Tenant-aware rate limiting
export function createTenantRateLimitMiddleware(
  redisClient: RedisClient,
  config: RateLimitConfig & { tenantMultiplier?: Record<string, number> }
) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      const authReq = req as AuthenticatedRequest;
      const tenantId = authReq.user?.tenantId;
      
      // Apply tenant-specific multipliers
      let maxRequests = config.maxRequests;
      if (tenantId && config.tenantMultiplier) {
        const multiplier = config.tenantMultiplier[tenantId] || 1;
        maxRequests = Math.floor(config.maxRequests * multiplier);
      }
      
      const key = config.keyGenerator ? config.keyGenerator(req) : generateTenantKey(req);
      const windowSeconds = Math.floor(config.windowMs / 1000);
      
      const current = await redisClient.incrementRateLimit(key, windowSeconds);
      const remaining = Math.max(0, maxRequests - current);
      const resetTime = Date.now() + (windowSeconds * 1000);
      
      res.set({
        'X-RateLimit-Limit': maxRequests.toString(),
        'X-RateLimit-Remaining': remaining.toString(),
        'X-RateLimit-Reset': Math.ceil(resetTime / 1000).toString(),
        'X-RateLimit-Window': windowSeconds.toString(),
      });
      
      if (current > maxRequests) {
        const retryAfter = Math.ceil(windowSeconds);
        res.set('Retry-After', retryAfter.toString());
        
        throw new BFFError(429, 'Rate limit exceeded', 'RATE_LIMIT_EXCEEDED', {
          limit: maxRequests,
          current,
          windowMs: config.windowMs,
          retryAfter,
          tenantId,
        });
      }
      
      next();
    } catch (error) {
      next(error);
    }
  };
}

// Burst rate limiting (allows short bursts but limits sustained usage)
export function createBurstRateLimitMiddleware(
  redisClient: RedisClient,
  config: {
    burstLimit: number;
    sustainedLimit: number;
    burstWindowMs: number;
    sustainedWindowMs: number;
    keyGenerator?: (req: Request) => string;
  }
) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      const key = config.keyGenerator ? config.keyGenerator(req) : generateDefaultKey(req);
      
      // Check burst limit (short window)
      const burstKey = `burst:${key}`;
      const burstWindowSeconds = Math.floor(config.burstWindowMs / 1000);
      const burstCurrent = await redisClient.incrementRateLimit(burstKey, burstWindowSeconds);
      
      // Check sustained limit (long window)
      const sustainedKey = `sustained:${key}`;
      const sustainedWindowSeconds = Math.floor(config.sustainedWindowMs / 1000);
      const sustainedCurrent = await redisClient.incrementRateLimit(sustainedKey, sustainedWindowSeconds);
      
      // Set headers for both limits
      res.set({
        'X-RateLimit-Burst-Limit': config.burstLimit.toString(),
        'X-RateLimit-Burst-Remaining': Math.max(0, config.burstLimit - burstCurrent).toString(),
        'X-RateLimit-Sustained-Limit': config.sustainedLimit.toString(),
        'X-RateLimit-Sustained-Remaining': Math.max(0, config.sustainedLimit - sustainedCurrent).toString(),
      });
      
      // Check if either limit is exceeded
      if (burstCurrent > config.burstLimit) {
        throw new BFFError(429, 'Burst rate limit exceeded', 'BURST_RATE_LIMIT_EXCEEDED', {
          burstLimit: config.burstLimit,
          burstCurrent,
          retryAfter: burstWindowSeconds,
        });
      }
      
      if (sustainedCurrent > config.sustainedLimit) {
        throw new BFFError(429, 'Sustained rate limit exceeded', 'SUSTAINED_RATE_LIMIT_EXCEEDED', {
          sustainedLimit: config.sustainedLimit,
          sustainedCurrent,
          retryAfter: sustainedWindowSeconds,
        });
      }
      
      next();
    } catch (error) {
      next(error);
    }
  };
}

// Adaptive rate limiting based on system load
export function createAdaptiveRateLimitMiddleware(
  redisClient: RedisClient,
  config: {
    baseLimit: number;
    windowMs: number;
    loadThresholds: { cpu: number; memory: number; responseTime: number };
    reductionFactor: number;
    keyGenerator?: (req: Request) => string;
  }
) {
  return async (req: Request, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Get system metrics (simplified - in production, use proper monitoring)
      const systemLoad = await getSystemLoad();
      
      // Calculate adaptive limit based on system load
      let adaptiveLimit = config.baseLimit;
      if (systemLoad.cpu > config.loadThresholds.cpu ||
          systemLoad.memory > config.loadThresholds.memory ||
          systemLoad.responseTime > config.loadThresholds.responseTime) {
        adaptiveLimit = Math.floor(config.baseLimit * config.reductionFactor);
      }
      
      const key = config.keyGenerator ? config.keyGenerator(req) : generateDefaultKey(req);
      const windowSeconds = Math.floor(config.windowMs / 1000);
      
      const current = await redisClient.incrementRateLimit(key, windowSeconds);
      const remaining = Math.max(0, adaptiveLimit - current);
      
      res.set({
        'X-RateLimit-Limit': adaptiveLimit.toString(),
        'X-RateLimit-Remaining': remaining.toString(),
        'X-RateLimit-Adaptive': 'true',
        'X-System-Load': JSON.stringify(systemLoad),
      });
      
      if (current > adaptiveLimit) {
        throw new BFFError(429, 'Adaptive rate limit exceeded', 'ADAPTIVE_RATE_LIMIT_EXCEEDED', {
          limit: adaptiveLimit,
          baseLimit: config.baseLimit,
          current,
          systemLoad,
          retryAfter: windowSeconds,
        });
      }
      
      next();
    } catch (error) {
      next(error);
    }
  };
}

// Helper functions
function generateDefaultKey(req: Request): string {
  const authReq = req as AuthenticatedRequest;
  if (authReq.user) {
    return `user:${authReq.user.id}:${req.method}:${req.route?.path || req.path}`;
  }
  return `ip:${req.ip}:${req.method}:${req.route?.path || req.path}`;
}

function generateTenantKey(req: Request): string {
  const authReq = req as AuthenticatedRequest;
  if (authReq.user) {
    return `tenant:${authReq.user.tenantId}:user:${authReq.user.id}:${req.method}:${req.route?.path || req.path}`;
  }
  return `ip:${req.ip}:${req.method}:${req.route?.path || req.path}`;
}

async function getSystemLoad(): Promise<{ cpu: number; memory: number; responseTime: number }> {
  // Simplified system load calculation
  // In production, integrate with proper monitoring tools
  const memUsage = process.memoryUsage();
  const memoryPercent = (memUsage.heapUsed / memUsage.heapTotal) * 100;
  
  return {
    cpu: 0, // Would need proper CPU monitoring
    memory: memoryPercent,
    responseTime: 0, // Would track average response times
  };
}

// Rate limit status endpoint
export async function getRateLimitStatus(
  req: Request,
  redisClient: RedisClient,
  keys: string[]
): Promise<Record<string, RateLimitInfo>> {
  const status: Record<string, RateLimitInfo> = {};
  
  for (const key of keys) {
    const current = await redisClient.getRateLimitCount(key);
    // This is simplified - in practice, you'd need to track limits and reset times
    status[key] = {
      limit: 1000, // Would be stored/configured per key
      current,
      remaining: Math.max(0, 1000 - current),
      resetTime: Date.now() + (15 * 60 * 1000), // 15 minutes from now
    };
  }
  
  return status;
}