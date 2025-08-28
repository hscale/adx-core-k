import { Response, NextFunction } from 'express';
import { AuthenticatedRequest } from './auth.js';
import { createLogger } from '../utils/logger.js';

const logger = createLogger('tenant-middleware');

export interface TenantRequest extends AuthenticatedRequest {
  tenantId?: string;
  tenantContext?: {
    id: string;
    name: string;
    features: string[];
    quotas: Record<string, any>;
    settings: Record<string, any>;
  };
}

export const tenantMiddleware = async (
  req: TenantRequest,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    // Extract tenant ID from various sources
    let tenantId: string | undefined;

    // 1. X-Tenant-ID header (highest priority)
    tenantId = req.headers['x-tenant-id'] as string;

    // 2. User's current tenant from JWT
    if (!tenantId && req.user?.tenantId) {
      tenantId = req.user.tenantId;
    }

    // 3. Query parameter (lowest priority)
    if (!tenantId && req.query.tenantId) {
      tenantId = req.query.tenantId as string;
    }

    if (!tenantId) {
      logger.warn('No tenant ID found in request', {
        userId: req.user?.id,
        requestId: req.requestId,
        headers: req.headers,
      });

      res.status(400).json({
        error: 'Bad Request',
        message: 'Tenant ID is required',
        timestamp: new Date().toISOString(),
      });
      return;
    }

    // Validate that user has access to this tenant
    if (req.user && req.user.tenantId !== tenantId) {
      logger.warn('User attempting to access unauthorized tenant', {
        userId: req.user.id,
        userTenantId: req.user.tenantId,
        requestedTenantId: tenantId,
        requestId: req.requestId,
      });

      res.status(403).json({
        error: 'Forbidden',
        message: 'Access denied to requested tenant',
        timestamp: new Date().toISOString(),
      });
      return;
    }

    // Set tenant context
    req.tenantId = tenantId;
    
    // In a real implementation, you would load tenant context from database/cache
    // For now, we'll use a simplified version
    req.tenantContext = {
      id: tenantId,
      name: `Tenant ${tenantId}`,
      features: ['file_management', 'sharing', 'workflows'],
      quotas: {
        storage: 10 * 1024 * 1024 * 1024, // 10GB
        files: 10000,
        apiCalls: 1000,
      },
      settings: {
        maxFileSize: 100 * 1024 * 1024, // 100MB
        allowedFileTypes: ['*'],
        retentionDays: 365,
      },
    };

    logger.debug('Tenant context set', {
      tenantId,
      userId: req.user?.id,
      requestId: req.requestId,
    });

    next();
  } catch (error) {
    logger.error('Tenant middleware error', {
      error,
      requestId: req.requestId,
    });

    res.status(500).json({
      error: 'Internal Server Error',
      message: 'Failed to process tenant context',
      timestamp: new Date().toISOString(),
    });
  }
};