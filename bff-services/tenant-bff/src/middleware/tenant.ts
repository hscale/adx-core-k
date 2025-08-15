import { Request, Response, NextFunction } from 'express';
import { RedisClient } from '../services/redis.js';
import { ApiClient } from '../services/apiClient.js';
import { AuthenticatedRequest } from './auth.js';
import { 
  Tenant, 
  TenantContext, 
  TenantNotFoundError, 
  TenantAccessDeniedError, 
  TenantSuspendedError 
} from '../types/tenant.js';

export interface TenantRequest extends AuthenticatedRequest {
  tenant?: Tenant;
  tenantContext?: TenantContext;
  tenantId?: string;
}

export function createTenantMiddleware(redisClient: RedisClient, apiClient: ApiClient) {
  return async (req: TenantRequest, res: Response, next: NextFunction) => {
    try {
      // Extract tenant ID from various sources
      const tenantId = extractTenantId(req);
      
      if (!tenantId) {
        // Some endpoints might not require tenant context
        return next();
      }

      req.tenantId = tenantId;

      // Try to get tenant from cache first
      let tenant = await redisClient.getCachedTenant(tenantId);
      
      if (!tenant) {
        // Fetch from API and cache
        try {
          tenant = await apiClient.getTenant(tenantId, req.headers.authorization?.substring(7));
          await redisClient.cacheTenant(tenant, 300); // Cache for 5 minutes
        } catch (error: any) {
          if (error.statusCode === 404) {
            throw new TenantNotFoundError(tenantId);
          }
          throw error;
        }
      }

      // Check tenant status
      if (tenant.status === 'suspended') {
        throw new TenantSuspendedError(tenantId);
      }

      if (tenant.status !== 'active') {
        return res.status(403).json({
          error: {
            code: 'TENANT_INACTIVE',
            message: `Tenant is not active: ${tenant.status}`,
          },
        });
      }

      req.tenant = tenant;

      // If user is authenticated, validate access and build context
      if (req.user) {
        await validateTenantAccess(req, redisClient, apiClient);
      }

      next();
    } catch (error: any) {
      if (error instanceof TenantNotFoundError) {
        return res.status(404).json({
          error: {
            code: 'TENANT_NOT_FOUND',
            message: error.message,
          },
        });
      }

      if (error instanceof TenantAccessDeniedError) {
        return res.status(403).json({
          error: {
            code: 'TENANT_ACCESS_DENIED',
            message: error.message,
          },
        });
      }

      if (error instanceof TenantSuspendedError) {
        return res.status(403).json({
          error: {
            code: 'TENANT_SUSPENDED',
            message: error.message,
          },
        });
      }

      console.error('Tenant middleware error:', error);
      return res.status(500).json({
        error: {
          code: 'TENANT_ERROR',
          message: 'Error processing tenant context',
        },
      });
    }
  };
}

async function validateTenantAccess(
  req: TenantRequest,
  redisClient: RedisClient,
  apiClient: ApiClient
): Promise<void> {
  if (!req.user || !req.tenant) return;

  const { user, tenant } = req;
  const tenantId = tenant.id;
  const userId = user.id;

  // Try to get tenant context from cache
  let tenantContext = await redisClient.getCachedTenantContext(tenantId, userId);

  if (!tenantContext) {
    // Validate access through API
    try {
      const accessValidation = await apiClient.validateTenantAccess(
        tenantId,
        userId,
        req.headers.authorization?.substring(7) || ''
      );

      if (!accessValidation.hasAccess) {
        throw new TenantAccessDeniedError(tenantId, userId);
      }

      // Get user's tenant context
      const userTenantContext = await apiClient.getUserTenantContext(
        tenantId,
        userId,
        req.headers.authorization?.substring(7) || ''
      );

      // Build tenant context
      tenantContext = {
        tenant,
        membership: userTenantContext.membership,
        permissions: accessValidation.permissions,
        features: tenant.features,
        quotas: tenant.quotas,
        settings: tenant.settings,
        branding: tenant.branding,
      };

      // Cache the context
      await redisClient.cacheTenantContext(tenantId, userId, tenantContext, 300);
    } catch (error: any) {
      if (error.statusCode === 403) {
        throw new TenantAccessDeniedError(tenantId, userId);
      }
      throw error;
    }
  }

  req.tenantContext = tenantContext;
}

export function requireTenant(req: TenantRequest, res: Response, next: NextFunction) {
  if (!req.tenant) {
    return res.status(400).json({
      error: {
        code: 'TENANT_REQUIRED',
        message: 'Tenant context is required for this endpoint',
      },
    });
  }
  next();
}

export function requireTenantAccess(req: TenantRequest, res: Response, next: NextFunction) {
  if (!req.user) {
    return res.status(401).json({
      error: {
        code: 'AUTHENTICATION_REQUIRED',
        message: 'Authentication is required for tenant access',
      },
    });
  }

  if (!req.tenantContext) {
    return res.status(403).json({
      error: {
        code: 'TENANT_ACCESS_REQUIRED',
        message: 'Tenant access is required for this endpoint',
      },
    });
  }

  next();
}

export function requireTenantPermission(permission: string) {
  return (req: TenantRequest, res: Response, next: NextFunction) => {
    if (!req.tenantContext) {
      return res.status(403).json({
        error: {
          code: 'TENANT_ACCESS_REQUIRED',
          message: 'Tenant access is required for this endpoint',
        },
      });
    }

    if (!hasTenantPermission(req.tenantContext, permission)) {
      return res.status(403).json({
        error: {
          code: 'INSUFFICIENT_TENANT_PERMISSIONS',
          message: `Tenant permission required: ${permission}`,
        },
      });
    }

    next();
  };
}

export function requireTenantRole(role: string) {
  return (req: TenantRequest, res: Response, next: NextFunction) => {
    if (!req.tenantContext) {
      return res.status(403).json({
        error: {
          code: 'TENANT_ACCESS_REQUIRED',
          message: 'Tenant access is required for this endpoint',
        },
      });
    }

    if (!req.tenantContext.membership.roles.includes(role)) {
      return res.status(403).json({
        error: {
          code: 'INSUFFICIENT_TENANT_ROLE',
          message: `Tenant role required: ${role}`,
        },
      });
    }

    next();
  };
}

export function injectTenantContext(req: TenantRequest, res: Response, next: NextFunction) {
  // Add tenant information to response headers
  if (req.tenant) {
    res.set('X-Tenant-ID', req.tenant.id);
    res.set('X-Tenant-Name', req.tenant.name);
    res.set('X-Tenant-Tier', req.tenant.subscriptionTier);
  }

  next();
}

function extractTenantId(req: Request): string | null {
  // Priority order for tenant ID extraction:
  
  // 1. X-Tenant-ID header
  const headerTenantId = req.headers['x-tenant-id'] as string;
  if (headerTenantId) {
    return headerTenantId;
  }

  // 2. URL path parameter
  const pathTenantId = req.params.tenantId;
  if (pathTenantId) {
    return pathTenantId;
  }

  // 3. Query parameter
  const queryTenantId = req.query.tenantId as string;
  if (queryTenantId) {
    return queryTenantId;
  }

  // 4. From authenticated user context (if available)
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

function hasTenantPermission(tenantContext: TenantContext, requiredPermission: string): boolean {
  // Check direct permissions
  if (tenantContext.permissions.includes(requiredPermission)) {
    return true;
  }

  // Check wildcard permissions
  for (const permission of tenantContext.permissions) {
    if (permission.endsWith('*')) {
      const wildcardBase = permission.slice(0, -1);
      if (requiredPermission.startsWith(wildcardBase)) {
        return true;
      }
    }
  }

  // Check role-based permissions
  const rolePermissions: Record<string, string[]> = {
    admin: ['*'],
    owner: ['tenant:*'],
    manager: ['tenant:read', 'tenant:write', 'tenant:members'],
    member: ['tenant:read'],
    viewer: ['tenant:read'],
  };

  for (const role of tenantContext.membership.roles) {
    const permissions = rolePermissions[role] || [];
    for (const permission of permissions) {
      if (permission === '*' || permission === requiredPermission) {
        return true;
      }
      if (permission.endsWith('*')) {
        const wildcardBase = permission.slice(0, -1);
        if (requiredPermission.startsWith(wildcardBase)) {
          return true;
        }
      }
    }
  }

  return false;
}