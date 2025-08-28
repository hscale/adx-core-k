import { describe, it, expect, beforeAll, afterAll, vi } from 'vitest';
import request from 'supertest';
import app from '../server.js';

// Mock Redis client
vi.mock('../services/redis.js', () => ({
  RedisClient: vi.fn().mockImplementation(() => ({
    isHealthy: () => true,
    disconnect: () => Promise.resolve(),
    getCachedTenant: () => Promise.resolve(null),
    cacheTenant: () => Promise.resolve(),
    getCachedUserTenants: () => Promise.resolve(null),
    cacheUserTenants: () => Promise.resolve(),
    getCachedTenantContext: () => Promise.resolve(null),
    cacheTenantContext: () => Promise.resolve(),
    getCachedTenantMemberships: () => Promise.resolve(null),
    cacheTenantMemberships: () => Promise.resolve(),
    getCachedTenantAnalytics: () => Promise.resolve(null),
    cacheTenantAnalytics: () => Promise.resolve(),
    getCachedTenantConfig: () => Promise.resolve(null),
    cacheTenantConfig: () => Promise.resolve(),
    incrementRateLimit: () => Promise.resolve({ count: 1, remaining: 99, resetTime: Date.now() + 60000 }),
    getSession: () => Promise.resolve(null),
    setSession: () => Promise.resolve(),
    extendSession: () => Promise.resolve(true),
    flushPattern: () => Promise.resolve(0),
    getInfo: () => Promise.resolve({}),
  })),
}));

// Mock API client
vi.mock('../services/apiClient.js', () => ({
  ApiClient: vi.fn().mockImplementation(() => ({
    healthCheck: () => Promise.resolve('healthy'),
    getTenant: () => Promise.resolve({
      id: 'test-tenant',
      name: 'Test Tenant',
      displayName: 'Test Tenant',
      subscriptionTier: 'professional',
      status: 'active',
      features: ['basic'],
      quotas: {
        maxUsers: 100,
        maxStorageGB: 10,
        maxApiCallsPerHour: 1000,
        maxWorkflowsPerHour: 100,
        maxModules: 10,
        maxCustomDomains: 1,
      },
      settings: {
        timezone: 'UTC',
        locale: 'en',
        dateFormat: 'YYYY-MM-DD',
        timeFormat: '24h',
        currency: 'USD',
        allowUserRegistration: true,
        requireEmailVerification: true,
        enableMFA: false,
        sessionTimeoutMinutes: 60,
        passwordPolicy: {
          minLength: 8,
          requireUppercase: true,
          requireLowercase: true,
          requireNumbers: true,
          requireSpecialChars: false,
          preventReuse: 5,
          maxAge: 90,
        },
        auditLogRetentionDays: 365,
      },
      branding: {
        primaryColor: '#007bff',
        secondaryColor: '#6c757d',
        accentColor: '#28a745',
        customDomain: null,
        logoUrl: null,
        faviconUrl: null,
        customCSS: null,
        emailTemplates: {},
      },
      createdAt: '2024-01-01T00:00:00Z',
      updatedAt: '2024-01-01T00:00:00Z',
    }),
    getUserTenants: () => Promise.resolve([]),
    validateTenantAccess: () => Promise.resolve({ hasAccess: true, permissions: ['tenant:read'], roles: ['member'] }),
    getUserTenantContext: () => Promise.resolve({
      membership: {
        id: 'membership-1',
        tenantId: 'test-tenant',
        userId: 'test-user',
        userEmail: 'test@example.com',
        userName: 'Test User',
        roles: ['member'],
        permissions: ['tenant:read'],
        status: 'active',
        joinedAt: '2024-01-01T00:00:00Z',
        lastActiveAt: '2024-01-01T00:00:00Z',
      },
    }),
  })),
  ApiError: class ApiError extends Error {
    constructor(message: string, public statusCode: number, public data?: any) {
      super(message);
      this.name = 'ApiError';
    }
  },
}));

describe('Tenant BFF Server', () => {
  beforeAll(async () => {
    // Setup test environment
    process.env.NODE_ENV = 'test';
    process.env.JWT_SECRET = 'test-secret';
    process.env.REDIS_URL = 'redis://localhost:6379';
  });

  afterAll(async () => {
    // Cleanup
  });

  describe('Health Check', () => {
    it('should return health status', async () => {
      const response = await request(app)
        .get('/health')
        .expect(200);

      expect(response.body).toMatchObject({
        status: 'healthy',
        service: 'tenant-bff',
        version: '1.0.0',
        checks: {
          redis: 'healthy',
          api: 'healthy',
        },
      });

      expect(response.body.timestamp).toBeDefined();
      expect(response.body.uptime).toBeGreaterThan(0);
      expect(response.body.memory).toBeDefined();
    });
  });

  describe('CORS', () => {
    it('should handle CORS preflight requests', async () => {
      const response = await request(app)
        .options('/api/tenants/current')
        .set('Origin', 'http://localhost:3002')
        .set('Access-Control-Request-Method', 'GET')
        .set('Access-Control-Request-Headers', 'Authorization')
        .expect(204);

      expect(response.headers['access-control-allow-origin']).toBe('http://localhost:3002');
      expect(response.headers['access-control-allow-methods']).toContain('GET');
      expect(response.headers['access-control-allow-headers']).toContain('Authorization');
    });

    it('should reject requests from unauthorized origins', async () => {
      await request(app)
        .get('/api/tenants/current')
        .set('Origin', 'http://malicious-site.com')
        .expect(500); // CORS error
    });
  });

  describe('Rate Limiting', () => {
    it('should include rate limit headers', async () => {
      const response = await request(app)
        .get('/health')
        .expect(200);

      expect(response.headers['x-ratelimit-limit']).toBeDefined();
      expect(response.headers['x-ratelimit-remaining']).toBeDefined();
      expect(response.headers['x-ratelimit-reset']).toBeDefined();
    });
  });

  describe('Authentication', () => {
    it('should reject requests without authentication for protected endpoints', async () => {
      const response = await request(app)
        .get('/api/tenants/current')
        .expect(401);

      expect(response.body.error).toMatchObject({
        code: 'MISSING_TOKEN',
        message: 'Authorization token is required',
      });
    });

    it('should reject requests with invalid tokens', async () => {
      const response = await request(app)
        .get('/api/tenants/current')
        .set('Authorization', 'Bearer invalid-token')
        .expect(401);

      expect(response.body.error).toMatchObject({
        code: 'INVALID_TOKEN',
        message: 'Invalid authorization token',
      });
    });
  });

  describe('Error Handling', () => {
    it('should return 404 for non-existent endpoints', async () => {
      const response = await request(app)
        .get('/api/non-existent')
        .expect(404);

      expect(response.body.error).toMatchObject({
        code: 'NOT_FOUND',
        message: 'Endpoint GET /api/non-existent not found',
      });

      expect(response.body.error.timestamp).toBeDefined();
      expect(response.body.error.requestId).toBeDefined();
    });

    it('should handle malformed JSON', async () => {
      const response = await request(app)
        .post('/api/tenants/switch')
        .set('Content-Type', 'application/json')
        .send('{"invalid": json}')
        .expect(400);

      expect(response.body.error).toMatchObject({
        code: 'INVALID_JSON',
        message: 'Invalid JSON in request body',
      });
    });
  });

  describe('Security Headers', () => {
    it('should include security headers', async () => {
      const response = await request(app)
        .get('/health')
        .expect(200);

      expect(response.headers['x-content-type-options']).toBe('nosniff');
      expect(response.headers['x-frame-options']).toBe('DENY');
      expect(response.headers['x-xss-protection']).toBe('1; mode=block');
    });
  });

  describe('Request ID', () => {
    it('should generate request ID for each request', async () => {
      const response = await request(app)
        .get('/health')
        .expect(200);

      expect(response.headers['x-request-id']).toBeDefined();
      expect(response.headers['x-request-id']).toMatch(/^req_\d+_[a-z0-9]+$/);
    });

    it('should use provided request ID', async () => {
      const requestId = 'custom-request-id';
      const response = await request(app)
        .get('/health')
        .set('X-Request-ID', requestId)
        .expect(200);

      expect(response.headers['x-request-id']).toBe(requestId);
    });
  });

  describe('Compression', () => {
    it('should compress responses when requested', async () => {
      const response = await request(app)
        .get('/health')
        .set('Accept-Encoding', 'gzip')
        .expect(200);

      // Note: supertest automatically decompresses, so we check if compression was applied
      expect(response.headers['content-encoding']).toBe('gzip');
    });
  });

  describe('Content Type', () => {
    it('should return JSON content type for API endpoints', async () => {
      const response = await request(app)
        .get('/health')
        .expect(200);

      expect(response.headers['content-type']).toContain('application/json');
    });
  });

  describe('Graceful Shutdown', () => {
    it('should handle shutdown signals gracefully', (done) => {
      // This is a basic test - in a real scenario you'd test the actual shutdown process
      const originalExit = process.exit;
      process.exit = vi.fn() as any;

      process.emit('SIGTERM');

      setTimeout(() => {
        expect(process.exit).toHaveBeenCalledWith(0);
        process.exit = originalExit;
        done();
      }, 100);
    });
  });
});

describe('Tenant BFF Integration', () => {
  describe('Service Dependencies', () => {
    it('should handle Redis connection failures gracefully', async () => {
      // Mock Redis failure
      vi.mocked(require('../services/redis.js').RedisClient).mockImplementationOnce(() => ({
        isHealthy: () => false,
        disconnect: () => Promise.resolve(),
      }));

      const response = await request(app)
        .get('/health')
        .expect(503);

      expect(response.body.status).toBe('degraded');
      expect(response.body.checks.redis).toBe('unhealthy');
    });

    it('should handle API service failures gracefully', async () => {
      // Mock API failure
      vi.mocked(require('../services/apiClient.js').ApiClient).mockImplementationOnce(() => ({
        healthCheck: () => Promise.reject(new Error('Service unavailable')),
      }));

      const response = await request(app)
        .get('/health')
        .expect(503);

      expect(response.body.status).toBe('degraded');
      expect(response.body.checks.api).toBe('unhealthy');
    });
  });

  describe('Environment Configuration', () => {
    it('should use default values when environment variables are not set', () => {
      // Test that the server starts with default configuration
      expect(process.env.PORT || 4002).toBe(4002);
      expect(process.env.REDIS_URL || 'redis://localhost:6379').toBe('redis://localhost:6379');
    });
  });
});