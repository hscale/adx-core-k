import { createClient, RedisClientType } from 'redis';
import { Tenant, TenantAnalytics, TenantContext, TenantMembership } from '../types/tenant.js';

export class RedisClient {
  private client: RedisClientType;
  private isConnected = false;

  constructor(private redisUrl: string) {
    this.client = createClient({
      url: this.redisUrl,
      socket: {
        reconnectStrategy: (retries) => {
          if (retries > 10) {
            console.error('Redis connection failed after 10 retries');
            return false;
          }
          return Math.min(retries * 100, 3000);
        },
      },
    });

    this.client.on('error', (err) => {
      console.error('Redis Client Error:', err);
      this.isConnected = false;
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
      throw error;
    }
  }

  public isHealthy(): boolean {
    return this.isConnected;
  }

  public async disconnect(): Promise<void> {
    if (this.isConnected) {
      await this.client.disconnect();
    }
  }

  // Tenant data caching
  public async cacheTenant(tenant: Tenant, ttl: number = 300): Promise<void> {
    const key = this.getTenantKey(tenant.id);
    await this.client.setEx(key, ttl, JSON.stringify(tenant));
  }

  public async getCachedTenant(tenantId: string): Promise<Tenant | null> {
    const key = this.getTenantKey(tenantId);
    const cached = await this.client.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  public async invalidateTenant(tenantId: string): Promise<void> {
    const key = this.getTenantKey(tenantId);
    await this.client.del(key);
    
    // Also invalidate related caches
    await this.invalidateTenantContext(tenantId);
    await this.invalidateTenantAnalytics(tenantId);
    await this.invalidateTenantMemberships(tenantId);
  }

  // Tenant context caching
  public async cacheTenantContext(
    tenantId: string,
    userId: string,
    context: TenantContext,
    ttl: number = 300
  ): Promise<void> {
    const key = this.getTenantContextKey(tenantId, userId);
    await this.client.setEx(key, ttl, JSON.stringify(context));
  }

  public async getCachedTenantContext(
    tenantId: string,
    userId: string
  ): Promise<TenantContext | null> {
    const key = this.getTenantContextKey(tenantId, userId);
    const cached = await this.client.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  public async invalidateTenantContext(tenantId: string, userId?: string): Promise<void> {
    if (userId) {
      const key = this.getTenantContextKey(tenantId, userId);
      await this.client.del(key);
    } else {
      // Invalidate all contexts for the tenant
      const pattern = this.getTenantContextKey(tenantId, '*');
      await this.deleteByPattern(pattern);
    }
  }

  // Tenant membership caching
  public async cacheTenantMemberships(
    tenantId: string,
    memberships: TenantMembership[],
    ttl: number = 300
  ): Promise<void> {
    const key = this.getTenantMembershipsKey(tenantId);
    await this.client.setEx(key, ttl, JSON.stringify(memberships));
  }

  public async getCachedTenantMemberships(tenantId: string): Promise<TenantMembership[] | null> {
    const key = this.getTenantMembershipsKey(tenantId);
    const cached = await this.client.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  public async invalidateTenantMemberships(tenantId: string): Promise<void> {
    const key = this.getTenantMembershipsKey(tenantId);
    await this.client.del(key);
  }

  // User tenant list caching
  public async cacheUserTenants(
    userId: string,
    tenants: Tenant[],
    ttl: number = 300
  ): Promise<void> {
    const key = this.getUserTenantsKey(userId);
    await this.client.setEx(key, ttl, JSON.stringify(tenants));
  }

  public async getCachedUserTenants(userId: string): Promise<Tenant[] | null> {
    const key = this.getUserTenantsKey(userId);
    const cached = await this.client.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  public async invalidateUserTenants(userId: string): Promise<void> {
    const key = this.getUserTenantsKey(userId);
    await this.client.del(key);
  }

  // Tenant analytics caching
  public async cacheTenantAnalytics(
    tenantId: string,
    period: string,
    analytics: TenantAnalytics,
    ttl: number = 600
  ): Promise<void> {
    const key = this.getTenantAnalyticsKey(tenantId, period);
    await this.client.setEx(key, ttl, JSON.stringify(analytics));
  }

  public async getCachedTenantAnalytics(
    tenantId: string,
    period: string
  ): Promise<TenantAnalytics | null> {
    const key = this.getTenantAnalyticsKey(tenantId, period);
    const cached = await this.client.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  public async invalidateTenantAnalytics(tenantId: string, period?: string): Promise<void> {
    if (period) {
      const key = this.getTenantAnalyticsKey(tenantId, period);
      await this.client.del(key);
    } else {
      // Invalidate all analytics for the tenant
      const pattern = this.getTenantAnalyticsKey(tenantId, '*');
      await this.deleteByPattern(pattern);
    }
  }

  // Tenant configuration caching
  public async cacheTenantConfig(
    tenantId: string,
    config: any,
    ttl: number = 1800
  ): Promise<void> {
    const key = this.getTenantConfigKey(tenantId);
    await this.client.setEx(key, ttl, JSON.stringify(config));
  }

  public async getCachedTenantConfig(tenantId: string): Promise<any | null> {
    const key = this.getTenantConfigKey(tenantId);
    const cached = await this.client.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  public async invalidateTenantConfig(tenantId: string): Promise<void> {
    const key = this.getTenantConfigKey(tenantId);
    await this.client.del(key);
  }

  // Rate limiting
  public async incrementRateLimit(
    key: string,
    windowMs: number,
    maxRequests: number
  ): Promise<{ count: number; remaining: number; resetTime: number }> {
    const now = Date.now();
    const window = Math.floor(now / windowMs);
    const rateLimitKey = `rate_limit:${key}:${window}`;

    const count = await this.client.incr(rateLimitKey);
    
    if (count === 1) {
      await this.client.expire(rateLimitKey, Math.ceil(windowMs / 1000));
    }

    const remaining = Math.max(0, maxRequests - count);
    const resetTime = (window + 1) * windowMs;

    return { count, remaining, resetTime };
  }

  // Session management
  public async setSession(
    sessionId: string,
    sessionData: any,
    ttl: number = 3600
  ): Promise<void> {
    const key = this.getSessionKey(sessionId);
    await this.client.setEx(key, ttl, JSON.stringify(sessionData));
  }

  public async getSession(sessionId: string): Promise<any | null> {
    const key = this.getSessionKey(sessionId);
    const cached = await this.client.get(key);
    return cached ? JSON.parse(cached) : null;
  }

  public async deleteSession(sessionId: string): Promise<void> {
    const key = this.getSessionKey(sessionId);
    await this.client.del(key);
  }

  public async extendSession(sessionId: string, ttl: number = 3600): Promise<boolean> {
    const key = this.getSessionKey(sessionId);
    const result = await this.client.expire(key, ttl);
    return result;
  }

  // Utility methods
  public async deleteByPattern(pattern: string): Promise<number> {
    const keys = await this.client.keys(pattern);
    if (keys.length === 0) return 0;
    
    return await this.client.del(keys);
  }

  public async flushPattern(pattern: string): Promise<number> {
    return await this.deleteByPattern(pattern);
  }

  public async exists(key: string): Promise<boolean> {
    const result = await this.client.exists(key);
    return result === 1;
  }

  public async ttl(key: string): Promise<number> {
    return await this.client.ttl(key);
  }

  // Key generation methods
  private getTenantKey(tenantId: string): string {
    return `tenant:${tenantId}`;
  }

  private getTenantContextKey(tenantId: string, userId: string): string {
    return `tenant_context:${tenantId}:${userId}`;
  }

  private getTenantMembershipsKey(tenantId: string): string {
    return `tenant_memberships:${tenantId}`;
  }

  private getUserTenantsKey(userId: string): string {
    return `user_tenants:${userId}`;
  }

  private getTenantAnalyticsKey(tenantId: string, period: string): string {
    return `tenant_analytics:${tenantId}:${period}`;
  }

  private getTenantConfigKey(tenantId: string): string {
    return `tenant_config:${tenantId}`;
  }

  private getSessionKey(sessionId: string): string {
    return `session:${sessionId}`;
  }

  // Health check
  public async ping(): Promise<string> {
    return await this.client.ping();
  }

  // Get Redis info
  public async getInfo(): Promise<any> {
    const info = await this.client.info();
    return this.parseRedisInfo(info);
  }

  private parseRedisInfo(info: string): any {
    const lines = info.split('\r\n');
    const result: any = {};
    let section = '';

    for (const line of lines) {
      if (line.startsWith('#')) {
        section = line.substring(2).toLowerCase();
        result[section] = {};
      } else if (line.includes(':')) {
        const [key, value] = line.split(':');
        if (section && result[section]) {
          result[section][key] = isNaN(Number(value)) ? value : Number(value);
        }
      }
    }

    return result;
  }
}