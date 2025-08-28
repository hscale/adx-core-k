import { Request, Response, NextFunction } from 'express';
import { AuthenticatedRequest } from './auth.js';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { BFFError } from './errorHandler.js';
import { Tenant } from '../types/auth.js';

export interface TenantRequest extends AuthenticatedRequest {
  tenant?: Tenant;
  tenantId?: string;
}

export function createTenantMiddleware(redisClient: RedisClient, apiClient: ApiClient) {
  return async (req: TenantRequest, res: Response, next: NextFunction): Promise<void> => {
    try {
      // Skip for public endpoints
      if (isPublicEndpoint(req.path)) {
        return next();
      }

      // Skip if no user (will be handled by auth middleware)
      if (!req.user) {
        return next();
      }

      const tenantId = extractTenantId(req);
      if (!tenantId) {
        throw new BFFError(400, 'Tenant ID required', 'MISSING_TENANT_ID');
      }

      // Validate user has access to this tenant
      if (req.user.tenantId !== tenantId) {
        throw new BFFError(403, 'Access denied to tenant', 'TENANT_ACCESS_DENIED', {
          requestedTenant: tenantId,
          userTenant: req.user.tenantId,
        });
      }

      // Try to get tenant from cache first
      let tenant = await redisClient.getTenantData(tenantId);
      
      if (!tenant) {
        // Fetch from API and cache
        try {
          tenant = await apiClient.getTenant(tenantId, req.token!);
          await redisClient.cacheTenantData(tenantId, tenant, 300); // 5 minutes cache
        } catch (error) {
          if (error.status === 404) {
            throw new BFFError(404, 'Tenant not found', 'TENANT_NOT_FOUND', { tenantId });
          } else if (error.status === 403) {
            throw new BFFError(403, 'Access denied to tenant', 'TENANT_ACCESS_DENIED', { tenantId });
          }
          throw error;
        }
      }

      // Check if tenant is active
      if (!tenant.isActive) {
        throw new BFFError(403, 'Tenant is inactive', 'TENANT_INACTIVE', { tenantId });
      }

      // Attach tenant to request
      req.tenant = tenant;
      req.tenantId = tenantId;

      next();
    } catch (error) {
      next(error);
    }
  };
}

export function requireTenant(req: TenantRequest, res: Response, next: NextFunction): void {
  if (!req.tenant) {
    throw new BFFError(400, 'Tenant context required', 'TENANT_CONTEXT_REQUIRED');
  }
  next();
}

export function requireTenantFeature(feature: string) {
  return (req: TenantRequest, res: Response, next: NextFunction): void => {
    if (!req.tenant) {
      throw new BFFError(400, 'Tenant context required', 'TENANT_CONTEXT_REQUIRED');
    }

    if (!req.tenant.features.includes(feature)) {
      throw new BFFError(403, `Tenant feature '${feature}' not available`, 'TENANT_FEATURE_UNAVAILABLE', {
        required: feature,
        available: req.tenant.features,
      });
    }

    next();
  };
}

export function requireTenantSubscription(minTier: string) {
  const tierHierarchy = ['free', 'basic', 'professional', 'enterprise'];
  
  return (req: TenantRequest, res: Response, next: NextFunction): void => {
    if (!req.tenant) {
      throw new BFFError(400, 'Tenant context required', 'TENANT_CONTEXT_REQUIRED');
    }

    const currentTierIndex = tierHierarchy.indexOf(req.tenant.subscriptionTier.toLowerCase());
    const requiredTierIndex = tierHierarchy.indexOf(minTier.toLowerCase());

    if (currentTierIndex === -1 || requiredTierIndex === -1) {
      throw new BFFError(500, 'Invalid subscription tier configuration', 'INVALID_SUBSCRIPTION_TIER');
    }

    if (currentTierIndex < requiredTierIndex) {
      throw new BFFError(403, `Subscription tier '${minTier}' or higher required`, 'INSUFFICIENT_SUBSCRIPTION_TIER', {
        required: minTier,
        current: req.tenant.subscriptionTier,
      });
    }

    next();
  };
}

export function checkTenantQuota(quotaType: string, requestedAmount = 1) {
  return (req: TenantRequest, res: Response, next: NextFunction): void => {
    if (!req.tenant) {
      throw new BFFError(400, 'Tenant context required', 'TENANT_CONTEXT_REQUIRED');
    }

    const quota = req.tenant.quotas[quotaType];
    if (!quota) {
      // If quota doesn't exist, assume unlimited
      return next();
    }

    if (quota.used + requestedAmount > quota.limit) {
      throw new BFFError(429, `Tenant quota exceeded for '${quotaType}'`, 'TENANT_QUOTA_EXCEEDED', {
        quotaType,
        used: quota.used,
        limit: quota.limit,
        requested: requestedAmount,
        available: quota.limit - quota.used,
      });
    }

    next();
  };
}

function extractTenantId(req: Request): string | null {
  // Priority order for tenant ID extraction:
  
  // 1. X-Tenant-ID header
  const tenantHeader = req.headers['x-tenant-id'] as string;
  if (tenantHeader) {
    return tenantHeader;
  }

  // 2. URL parameter
  if (req.params.tenantId) {
    return req.params.tenantId;
  }

  // 3. Query parameter
  if (req.query.tenantId && typeof req.query.tenantId === 'string') {
    return req.query.tenantId;
  }

  // 4. From authenticated user context
  const authReq = req as AuthenticatedRequest;
  if (authReq.user?.tenantId) {
    return authReq.user.tenantId;
  }

  // 5. Subdomain extraction (tenant.domain.com)
  const host = req.headers.host;
  if (host) {
    const subdomain = extractSubdomain(host);
    if (subdomain && subdomain !== 'www' && subdomain !== 'api') {
      return subdomain;
    }
  }

  return null;
}

function extractSubdomain(host: string): string | null {
  const parts = host.split('.');
  if (parts.length >= 3) {
    return parts[0];
  }
  return null;
}

function isPublicEndpoint(path: string): boolean {
  const publicPaths = [
    '/health',
    '/api/auth/login',
    '/api/auth/register',
    '/api/auth/refresh',
    '/api/auth/password-reset',
    '/api/auth/password-reset/confirm',
    '/api/auth/verify-email',
  ];

  return publicPaths.some(publicPath => path.startsWith(publicPath));
}

// Middleware to inject tenant ID into outgoing API requests
export function injectTenantContext(req: TenantRequest, res: Response, next: NextFunction): void {
  // Add tenant ID to response headers for client-side context
  if (req.tenantId) {
    res.set('X-Tenant-ID', req.tenantId);
  }

  if (req.tenant) {
    res.set('X-Tenant-Name', req.tenant.name);
    res.set('X-Tenant-Tier', req.tenant.subscriptionTier);
  }

  next();
}

// Helper function to validate tenant access for specific operations
export async function validateTenantAccess(
  userId: string,
  tenantId: string,
  operation: string,
  redisClient: RedisClient,
  apiClient: ApiClient,
  authToken: string
): Promise<boolean> {
  try {
    // Check cache first
    const cacheKey = `tenant_access:${userId}:${tenantId}:${operation}`;
    const cachedResult = await redisClient.get<boolean>(cacheKey);
    
    if (cachedResult !== null) {
      return cachedResult;
    }

    // Validate with backend service
    const userTenants = await apiClient.getUserTenants(userId, authToken);
    const hasAccess = userTenants.some((tenant: any) => 
      tenant.id === tenantId && tenant.permissions.includes(operation)
    );

    // Cache result for 5 minutes
    await redisClient.set(cacheKey, hasAccess, 300);
    
    return hasAccess;
  } catch (error) {
    console.error('Error validating tenant access:', error);
    return false;
  }
}