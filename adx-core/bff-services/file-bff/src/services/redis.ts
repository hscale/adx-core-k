import { createClient } from 'redis';
import { createLogger } from '../utils/logger.js';

const logger = createLogger('redis');

// Create Redis client
export const redisClient = createClient({
  url: process.env.REDIS_URL || 'redis://localhost:6379',
  password: process.env.REDIS_PASSWORD,
  database: parseInt(process.env.REDIS_DB || '0'),
  socket: {
    reconnectStrategy: (retries) => {
      if (retries > 10) {
        logger.error('Redis reconnection failed after 10 attempts');
        return new Error('Redis reconnection failed');
      }
      return Math.min(retries * 50, 1000);
    },
  },
});

// Redis event handlers
redisClient.on('connect', () => {
  logger.info('Redis client connected');
});

redisClient.on('ready', () => {
  logger.info('Redis client ready');
});

redisClient.on('error', (error) => {
  logger.error('Redis client error', { error });
});

redisClient.on('end', () => {
  logger.info('Redis client disconnected');
});

// Connect to Redis
redisClient.connect().catch((error) => {
  logger.error('Failed to connect to Redis', { error });
});

// Cache utility functions
export class CacheService {
  private static instance: CacheService;
  private defaultTTL = parseInt(process.env.CACHE_TTL_DEFAULT || '300'); // 5 minutes

  static getInstance(): CacheService {
    if (!CacheService.instance) {
      CacheService.instance = new CacheService();
    }
    return CacheService.instance;
  }

  async get<T>(key: string): Promise<T | null> {
    try {
      const value = await redisClient.get(key);
      if (!value) return null;
      
      return JSON.parse(value) as T;
    } catch (error) {
      logger.error('Cache get error', { key, error });
      return null;
    }
  }

  async set(key: string, value: any, ttl?: number): Promise<boolean> {
    try {
      const serialized = JSON.stringify(value);
      const expiry = ttl || this.defaultTTL;
      
      await redisClient.setEx(key, expiry, serialized);
      return true;
    } catch (error) {
      logger.error('Cache set error', { key, error });
      return false;
    }
  }

  async del(key: string): Promise<boolean> {
    try {
      await redisClient.del(key);
      return true;
    } catch (error) {
      logger.error('Cache delete error', { key, error });
      return false;
    }
  }

  async exists(key: string): Promise<boolean> {
    try {
      const result = await redisClient.exists(key);
      return result === 1;
    } catch (error) {
      logger.error('Cache exists error', { key, error });
      return false;
    }
  }

  async invalidatePattern(pattern: string): Promise<number> {
    try {
      const keys = await redisClient.keys(pattern);
      if (keys.length === 0) return 0;
      
      await redisClient.del(keys);
      return keys.length;
    } catch (error) {
      logger.error('Cache invalidate pattern error', { pattern, error });
      return 0;
    }
  }

  // File-specific cache methods
  generateFileKey(tenantId: string, path: string, filters?: any): string {
    const filterHash = filters ? Buffer.from(JSON.stringify(filters)).toString('base64') : '';
    return `files:${tenantId}:${path}:${filterHash}`;
  }

  generateQuotaKey(tenantId: string): string {
    return `quota:${tenantId}`;
  }

  generateOperationKey(tenantId: string): string {
    return `operations:${tenantId}`;
  }

  async cacheFiles(tenantId: string, path: string, files: any, filters?: any): Promise<void> {
    const key = this.generateFileKey(tenantId, path, filters);
    const ttl = parseInt(process.env.CACHE_TTL_FILES || '60'); // 1 minute for files
    await this.set(key, files, ttl);
  }

  async getCachedFiles(tenantId: string, path: string, filters?: any): Promise<any | null> {
    const key = this.generateFileKey(tenantId, path, filters);
    return this.get(key);
  }

  async invalidateFileCache(tenantId: string, path?: string): Promise<void> {
    const pattern = path ? 
      `files:${tenantId}:${path}*` : 
      `files:${tenantId}:*`;
    await this.invalidatePattern(pattern);
  }

  async cacheQuota(tenantId: string, quota: any): Promise<void> {
    const key = this.generateQuotaKey(tenantId);
    const ttl = parseInt(process.env.CACHE_TTL_QUOTA || '300'); // 5 minutes for quota
    await this.set(key, quota, ttl);
  }

  async getCachedQuota(tenantId: string): Promise<any | null> {
    const key = this.generateQuotaKey(tenantId);
    return this.get(key);
  }
}

export const cacheService = CacheService.getInstance();