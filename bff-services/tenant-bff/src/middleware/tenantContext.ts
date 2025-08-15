import { Request, Response, NextFunction } from 'express';
import { TenantContext, SubscriptionTier } from '../types/tenant.js';

// Extend Express Request type to include tenant context
declare global {
  namespace Express {
    interface Request {
      tenant?: TenantContext;
    }
  }
}

export const tenantContextMiddleware = (req: Request, res: Response, next: NextFunction) => {
  // Skip for health check and non-authenticated requests
  if (req.path === '/health' || !req.auth) {
    return next();
  }

  const tenantId = req.headers['x-tenant-id'] as string || req.auth.tenantId;
  
  if (!tenantId) {
    return res.status(400).json({
      error: {
        code: 'TENANT_ID_REQUIRED',
        message: 'Tenant ID is required',
      },
    });
  }

  // In a real implementation, you would fetch tenant context from the database or cache
  // For now, we'll create a mock tenant context
  const tenantContext: TenantContext = {
    tenantId,
    tenantName: 'Demo Tenant',
    subscriptionTier: SubscriptionTier.PROFESSIONAL,
    features: ['basic_features', 'advanced_analytics'],
    quotas: {
      maxUsers: 10,
      maxStorageGB: 100,
      maxApiCallsPerHour: 1000,
      maxWorkflowsPerHour: 50,
      currentUsers: 3,
      currentStorageGB: 25.5,
      currentApiCallsThisHour: 150,
      currentWorkflowsThisHour: 5,
    },
  };

  req.tenant = tenantContext;
  next();
};