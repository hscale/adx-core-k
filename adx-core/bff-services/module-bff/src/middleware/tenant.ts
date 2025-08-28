import { Request, Response, NextFunction } from 'express';
import { createError } from './errorHandler';

export interface TenantRequest extends Request {
  tenant?: {
    id: string;
    name: string;
    features: string[];
    quotas: Record<string, any>;
  };
}

export const tenantMiddleware = (
  req: TenantRequest,
  res: Response,
  next: NextFunction
) => {
  const tenantId = req.headers['x-tenant-id'] as string;

  if (!tenantId) {
    return next(createError('Tenant ID required', 400, 'TENANT_ID_REQUIRED'));
  }

  try {
    // In a real implementation, this would fetch tenant data from the database
    // For now, we'll mock tenant data
    req.tenant = {
      id: tenantId,
      name: `Tenant ${tenantId}`,
      features: ['modules', 'marketplace', 'development'],
      quotas: {
        maxModules: 50,
        maxDevelopmentProjects: 10,
        storageGB: 100,
      },
    };

    next();
  } catch (error) {
    next(createError('Invalid tenant', 400, 'INVALID_TENANT'));
  }
};