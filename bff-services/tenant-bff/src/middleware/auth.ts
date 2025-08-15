import { Request, Response, NextFunction } from 'express';
import jwt from 'jsonwebtoken';
import { RedisClient } from '../services/redis.js';

export interface AuthenticatedRequest extends Request {
  user?: {
    id: string;
    email: string;
    tenantId?: string;
    roles: string[];
    permissions: string[];
    sessionId: string;
  };
}

export interface JwtPayload {
  sub: string; // User ID
  email: string;
  tenantId?: string;
  roles: string[];
  permissions: string[];
  sessionId: string;
  iat: number;
  exp: number;
}

export function createAuthMiddleware(redisClient: RedisClient, jwtSecret: string) {
  return async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const authHeader = req.headers.authorization;
      
      if (!authHeader || !authHeader.startsWith('Bearer ')) {
        return res.status(401).json({
          error: {
            code: 'MISSING_TOKEN',
            message: 'Authorization token is required',
          },
        });
      }

      const token = authHeader.substring(7);
      
      // Verify JWT token
      const decoded = jwt.verify(token, jwtSecret) as JwtPayload;
      
      // Check if session is still valid in Redis
      const sessionData = await redisClient.getSession(decoded.sessionId);
      if (!sessionData) {
        return res.status(401).json({
          error: {
            code: 'SESSION_EXPIRED',
            message: 'Session has expired',
          },
        });
      }

      // Attach user info to request
      req.user = {
        id: decoded.sub,
        email: decoded.email,
        tenantId: decoded.tenantId,
        roles: decoded.roles || [],
        permissions: decoded.permissions || [],
        sessionId: decoded.sessionId,
      };

      // Extend session
      await redisClient.extendSession(decoded.sessionId, 3600); // 1 hour

      next();
    } catch (error: any) {
      if (error.name === 'JsonWebTokenError') {
        return res.status(401).json({
          error: {
            code: 'INVALID_TOKEN',
            message: 'Invalid authorization token',
          },
        });
      }
      
      if (error.name === 'TokenExpiredError') {
        return res.status(401).json({
          error: {
            code: 'TOKEN_EXPIRED',
            message: 'Authorization token has expired',
          },
        });
      }

      console.error('Auth middleware error:', error);
      return res.status(500).json({
        error: {
          code: 'AUTH_ERROR',
          message: 'Authentication error',
        },
      });
    }
  };
}

export function optionalAuth(redisClient: RedisClient, jwtSecret: string) {
  return async (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    try {
      const authHeader = req.headers.authorization;
      
      if (!authHeader || !authHeader.startsWith('Bearer ')) {
        return next(); // Continue without authentication
      }

      const token = authHeader.substring(7);
      
      try {
        const decoded = jwt.verify(token, jwtSecret) as JwtPayload;
        
        // Check if session is still valid in Redis
        const sessionData = await redisClient.getSession(decoded.sessionId);
        if (sessionData) {
          req.user = {
            id: decoded.sub,
            email: decoded.email,
            tenantId: decoded.tenantId,
            roles: decoded.roles || [],
            permissions: decoded.permissions || [],
            sessionId: decoded.sessionId,
          };

          // Extend session
          await redisClient.extendSession(decoded.sessionId, 3600);
        }
      } catch (error) {
        // Invalid token, but continue without authentication
        console.warn('Optional auth failed:', error.message);
      }

      next();
    } catch (error) {
      console.error('Optional auth middleware error:', error);
      next(); // Continue even if there's an error
    }
  };
}

export function requirePermission(permission: string) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    if (!req.user) {
      return res.status(401).json({
        error: {
          code: 'AUTHENTICATION_REQUIRED',
          message: 'Authentication is required for this endpoint',
        },
      });
    }

    if (!hasPermission(req.user, permission)) {
      return res.status(403).json({
        error: {
          code: 'INSUFFICIENT_PERMISSIONS',
          message: `Permission required: ${permission}`,
        },
      });
    }

    next();
  };
}

export function requireRole(role: string) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    if (!req.user) {
      return res.status(401).json({
        error: {
          code: 'AUTHENTICATION_REQUIRED',
          message: 'Authentication is required for this endpoint',
        },
      });
    }

    if (!req.user.roles.includes(role)) {
      return res.status(403).json({
        error: {
          code: 'INSUFFICIENT_ROLE',
          message: `Role required: ${role}`,
        },
      });
    }

    next();
  };
}

export function requireAnyRole(roles: string[]) {
  return (req: AuthenticatedRequest, res: Response, next: NextFunction) => {
    if (!req.user) {
      return res.status(401).json({
        error: {
          code: 'AUTHENTICATION_REQUIRED',
          message: 'Authentication is required for this endpoint',
        },
      });
    }

    const hasAnyRole = roles.some(role => req.user!.roles.includes(role));
    if (!hasAnyRole) {
      return res.status(403).json({
        error: {
          code: 'INSUFFICIENT_ROLE',
          message: `One of these roles required: ${roles.join(', ')}`,
        },
      });
    }

    next();
  };
}

function hasPermission(user: AuthenticatedRequest['user'], requiredPermission: string): boolean {
  if (!user) return false;

  // Check direct permissions
  if (user.permissions.includes(requiredPermission)) {
    return true;
  }

  // Check wildcard permissions
  const permissionParts = requiredPermission.split(':');
  for (const permission of user.permissions) {
    if (permission.endsWith('*')) {
      const wildcardBase = permission.slice(0, -1);
      if (requiredPermission.startsWith(wildcardBase)) {
        return true;
      }
    }
  }

  // Check role-based permissions (simplified - in real implementation, 
  // you'd fetch role permissions from a service or cache)
  const rolePermissions: Record<string, string[]> = {
    admin: ['*'],
    tenant_admin: ['tenant:*'],
    tenant_member: ['tenant:read', 'tenant:switch'],
    user: ['tenant:read'],
  };

  for (const role of user.roles) {
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