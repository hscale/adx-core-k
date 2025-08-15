import { Request, Response, NextFunction } from 'express';
import { createError } from './errorHandler';

export interface AuthenticatedRequest extends Request {
  user?: {
    id: string;
    email: string;
    roles: string[];
    permissions: string[];
  };
}

export const authMiddleware = (
  req: AuthenticatedRequest,
  res: Response,
  next: NextFunction
) => {
  const authHeader = req.headers.authorization;

  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    return next(createError('Authentication required', 401, 'UNAUTHORIZED'));
  }

  const token = authHeader.substring(7);

  try {
    // In a real implementation, this would validate the JWT token
    // For now, we'll mock a user based on the token
    if (token === 'invalid') {
      return next(createError('Invalid token', 401, 'INVALID_TOKEN'));
    }

    // Mock user data - in production this would come from JWT validation
    req.user = {
      id: 'user-123',
      email: 'user@example.com',
      roles: ['user', 'developer'],
      permissions: [
        'module:read',
        'module:install',
        'module:configure',
        'module:develop',
      ],
    };

    next();
  } catch (error) {
    next(createError('Invalid token', 401, 'INVALID_TOKEN'));
  }
};

export const requirePermission = (permission: string) => {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    if (!req.user) {
      return next(createError('Authentication required', 401, 'UNAUTHORIZED'));
    }

    if (!req.user.permissions.includes(permission)) {
      return next(createError('Insufficient permissions', 403, 'FORBIDDEN'));
    }

    next();
  };
};