import { Request, Response, NextFunction } from 'express';
import { createLogger } from '../utils/logger.js';

const logger = createLogger('auth-middleware');

export interface AuthenticatedRequest extends Request {
  user?: {
    id: string;
    email: string;
    tenantId: string;
    roles: string[];
    permissions: string[];
  };
  requestId?: string;
}

export const authMiddleware = async (
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
): Promise<void> => {
  try {
    const authHeader = req.headers.authorization;
    
    if (!authHeader || !authHeader.startsWith('Bearer ')) {
      res.status(401).json({
        error: 'Unauthorized',
        message: 'Missing or invalid authorization header',
        timestamp: new Date().toISOString(),
      });
      return;
    }

    const token = authHeader.substring(7);
    
    if (!token) {
      res.status(401).json({
        error: 'Unauthorized',
        message: 'Missing authentication token',
        timestamp: new Date().toISOString(),
      });
      return;
    }

    // In a real implementation, you would validate the JWT token here
    // For now, we'll extract basic info from the token (this is a simplified version)
    try {
      // This is a placeholder - in production, use proper JWT validation
      const payload = JSON.parse(Buffer.from(token.split('.')[1], 'base64').toString());
      
      req.user = {
        id: payload.sub || payload.userId,
        email: payload.email,
        tenantId: payload.tenantId,
        roles: payload.roles || [],
        permissions: payload.permissions || [],
      };

      logger.debug('User authenticated', {
        userId: req.user.id,
        tenantId: req.user.tenantId,
        requestId: req.requestId,
      });

      next();
    } catch (tokenError) {
      logger.warn('Invalid token format', { 
        error: tokenError,
        requestId: req.requestId,
      });
      
      res.status(401).json({
        error: 'Unauthorized',
        message: 'Invalid authentication token',
        timestamp: new Date().toISOString(),
      });
    }
  } catch (error) {
    logger.error('Authentication middleware error', { 
      error,
      requestId: req.requestId,
    });
    
    res.status(500).json({
      error: 'Internal Server Error',
      message: 'Authentication failed',
      timestamp: new Date().toISOString(),
    });
  }
};