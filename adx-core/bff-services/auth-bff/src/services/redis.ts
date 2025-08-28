import { createClient, RedisClientType } from 'redis';

export class RedisClient {
  private client: RedisClientType;
  private isConnected = false;

  constructor(url: string) {
    this.client = createClient({
      url,
      socket: {
        reconnectStrategy: (retries) => Math.min(retries * 50, 500),
      },
    });

    this.client.on('error', (err) => {
      console.error('Redis Client Error:', err);
    });

    this.client.on('connect', () => {
      console.log('Redis Client Connected');
      this.isConnected = true;
    });

    this.client.on('disconnect', () => {
      console.log('Redis Client Disconnected');
      this.isConnected = false;
    });

    this.connect();
  }

  private async connect(): Promise<void> {
    try {
      await this.client.connect();
    } catch (error) {
      console.error('Failed to connect to Redis:', error);
    }
  }

  async get<T = any>(key: string): Promise<T | null> {
    try {
      const value = await this.client.get(key);
      return value ? JSON.parse(value) : null;
    } catch (error) {
      console.error(`Redis GET error for key ${key}:`, error);
      return null;
    }
  }

  async set(key: string, value: any, ttlSeconds?: number): Promise<boolean> {
    try {
      const serialized = JSON.stringify(value);
      if (ttlSeconds) {
        await this.client.setEx(key, ttlSeconds, serialized);
      } else {
        await this.client.set(key, serialized);
      }
      return true;
    } catch (error) {
      console.error(`Redis SET error for key ${key}:`, error);
      return false;
    }
  }

  async del(key: string): Promise<boolean> {
    try {
      await this.client.del(key);
      return true;
    } catch (error) {
      console.error(`Redis DEL error for key ${key}:`, error);
      return false;
    }
  }

  async exists(key: string): Promise<boolean> {
    try {
      const result = await this.client.exists(key);
      return result === 1;
    } catch (error) {
      console.error(`Redis EXISTS error for key ${key}:`, error);
      return false;
    }
  }

  async incr(key: string): Promise<number> {
    try {
      return await this.client.incr(key);
    } catch (error) {
      console.error(`Redis INCR error for key ${key}:`, error);
      return 0;
    }
  }

  async expire(key: string, seconds: number): Promise<boolean> {
    try {
      await this.client.expire(key, seconds);
      return true;
    } catch (error) {
      console.error(`Redis EXPIRE error for key ${key}:`, error);
      return false;
    }
  }

  async hget(key: string, field: string): Promise<string | null> {
    try {
      const result = await this.client.hGet(key, field);
      return result || null;
    } catch (error) {
      console.error(`Redis HGET error for key ${key}, field ${field}:`, error);
      return null;
    }
  }

  async hset(key: string, field: string, value: string): Promise<boolean> {
    try {
      await this.client.hSet(key, field, value);
      return true;
    } catch (error) {
      console.error(`Redis HSET error for key ${key}, field ${field}:`, error);
      return false;
    }
  }

  async hgetall(key: string): Promise<Record<string, string> | null> {
    try {
      return await this.client.hGetAll(key);
    } catch (error) {
      console.error(`Redis HGETALL error for key ${key}:`, error);
      return null;
    }
  }

  async sadd(key: string, ...members: string[]): Promise<number> {
    try {
      return await this.client.sAdd(key, members);
    } catch (error) {
      console.error(`Redis SADD error for key ${key}:`, error);
      return 0;
    }
  }

  async srem(key: string, ...members: string[]): Promise<number> {
    try {
      return await this.client.sRem(key, members);
    } catch (error) {
      console.error(`Redis SREM error for key ${key}:`, error);
      return 0;
    }
  }

  async smembers(key: string): Promise<string[]> {
    try {
      return await this.client.sMembers(key);
    } catch (error) {
      console.error(`Redis SMEMBERS error for key ${key}:`, error);
      return [];
    }
  }

  async sismember(key: string, member: string): Promise<boolean> {
    try {
      return await this.client.sIsMember(key, member);
    } catch (error) {
      console.error(`Redis SISMEMBER error for key ${key}, member ${member}:`, error);
      return false;
    }
  }

  async keys(pattern: string): Promise<string[]> {
    try {
      return await this.client.keys(pattern);
    } catch (error) {
      console.error(`Redis KEYS error for pattern ${pattern}:`, error);
      return [];
    }
  }

  async flushPattern(pattern: string): Promise<number> {
    try {
      const keys = await this.keys(pattern);
      if (keys.length === 0) return 0;
      
      await this.client.del(keys);
      return keys.length;
    } catch (error) {
      console.error(`Redis FLUSH PATTERN error for pattern ${pattern}:`, error);
      return 0;
    }
  }

  async disconnect(): Promise<void> {
    try {
      await this.client.disconnect();
      this.isConnected = false;
    } catch (error) {
      console.error('Redis disconnect error:', error);
    }
  }

  isHealthy(): boolean {
    return this.isConnected;
  }

  // Cache helper methods
  async cacheUserSession(sessionId: string, sessionData: any, ttlSeconds = 3600): Promise<boolean> {
    return this.set(`session:${sessionId}`, sessionData, ttlSeconds);
  }

  async getUserSession(sessionId: string): Promise<any | null> {
    return this.get(`session:${sessionId}`);
  }

  async invalidateUserSession(sessionId: string): Promise<boolean> {
    return this.del(`session:${sessionId}`);
  }

  async cacheUserData(userId: string, userData: any, ttlSeconds = 300): Promise<boolean> {
    return this.set(`user:${userId}`, userData, ttlSeconds);
  }

  async getUserData(userId: string): Promise<any | null> {
    return this.get(`user:${userId}`);
  }

  async cacheTenantData(tenantId: string, tenantData: any, ttlSeconds = 300): Promise<boolean> {
    return this.set(`tenant:${tenantId}`, tenantData, ttlSeconds);
  }

  async getTenantData(tenantId: string): Promise<any | null> {
    return this.get(`tenant:${tenantId}`);
  }

  async cacheAggregatedData(key: string, data: any, ttlSeconds = 300): Promise<boolean> {
    return this.set(`aggregated:${key}`, data, ttlSeconds);
  }

  async getAggregatedData(key: string): Promise<any | null> {
    return this.get(`aggregated:${key}`);
  }

  // Rate limiting helpers
  async incrementRateLimit(key: string, windowSeconds: number): Promise<number> {
    const count = await this.incr(`rate_limit:${key}`);
    if (count === 1) {
      await this.expire(`rate_limit:${key}`, windowSeconds);
    }
    return count;
  }

  async getRateLimitCount(key: string): Promise<number> {
    const count = await this.get(`rate_limit:${key}`);
    return count || 0;
  }
}