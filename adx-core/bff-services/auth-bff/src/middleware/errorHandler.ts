import { Request, Response, NextFunction } from 'express';
import { ApiError } from '../services/apiClient.js';

export interface ErrorResponse {
  error: {
    code: string;
    message: string;
    details?: any;
    validationErrors?: ValidationError[];
    retryAfter?: number;
    documentationUrl?: string;
  };
  requestId: string;
  timestamp: string;
}

export interface ValidationError {
  field: string;
  code: string;
  message: string;
  rejectedValue?: any;
}

export function errorHandler(
  error: Error,
  req: Request,
  res: Response,
  next: NextFunction
): void {
  const requestId = req.headers['x-request-id'] as string || generateRequestId();
  const timestamp = new Date().toISOString();

  // Log error for monitoring
  console.error(`[${requestId}] Error:`, {
    message: error.message,
    stack: error.stack,
    url: req.url,
    method: req.method,
    userAgent: req.headers['user-agent'],
    ip: req.ip,
  });

  let statusCode = 500;
  let errorCode = 'INTERNAL_SERVER_ERROR';
  let message = 'An unexpected error occurred';
  let details: any = undefined;
  let validationErrors: ValidationError[] | undefined = undefined;
  let retryAfter: number | undefined = undefined;

  if (error instanceof ApiError) {
    statusCode = error.status;
    message = error.message;
    details = error.data;

    switch (error.status) {
      case 400:
        errorCode = 'BAD_REQUEST';
        break;
      case 401:
        errorCode = 'UNAUTHORIZED';
        break;
      case 403:
        errorCode = 'FORBIDDEN';
        break;
      case 404:
        errorCode = 'NOT_FOUND';
        break;
      case 409:
        errorCode = 'CONFLICT';
        break;
      case 422:
        errorCode = 'VALIDATION_FAILED';
        if (error.data?.validationErrors) {
          validationErrors = error.data.validationErrors;
        }
        break;
      case 429:
        errorCode = 'RATE_LIMIT_EXCEEDED';
        if (error.data?.retryAfter) {
          retryAfter = error.data.retryAfter;
        }
        break;
      case 500:
        errorCode = 'INTERNAL_SERVER_ERROR';
        break;
      case 502:
        errorCode = 'BAD_GATEWAY';
        break;
      case 503:
        errorCode = 'SERVICE_UNAVAILABLE';
        break;
      case 504:
        errorCode = 'GATEWAY_TIMEOUT';
        break;
      default:
        errorCode = 'API_ERROR';
    }
  } else if (error.name === 'ValidationError') {
    statusCode = 422;
    errorCode = 'VALIDATION_FAILED';
    message = 'Request validation failed';
    
    // Handle Zod validation errors
    if ('issues' in error) {
      validationErrors = (error as any).issues.map((issue: any) => ({
        field: issue.path.join('.'),
        code: issue.code,
        message: issue.message,
        rejectedValue: issue.received,
      }));
    }
  } else if (error.name === 'JsonWebTokenError') {
    statusCode = 401;
    errorCode = 'INVALID_TOKEN';
    message = 'Invalid authentication token';
  } else if (error.name === 'TokenExpiredError') {
    statusCode = 401;
    errorCode = 'TOKEN_EXPIRED';
    message = 'Authentication token has expired';
  } else if (error.name === 'SyntaxError' && 'body' in error) {
    statusCode = 400;
    errorCode = 'INVALID_JSON';
    message = 'Invalid JSON in request body';
  } else if (error.message.includes('timeout')) {
    statusCode = 504;
    errorCode = 'GATEWAY_TIMEOUT';
    message = 'Request timeout';
  } else if (error.message.includes('ECONNREFUSED')) {
    statusCode = 503;
    errorCode = 'SERVICE_UNAVAILABLE';
    message = 'Backend service unavailable';
  }

  const errorResponse: ErrorResponse = {
    error: {
      code: errorCode,
      message,
      ...(details && { details }),
      ...(validationErrors && { validationErrors }),
      ...(retryAfter && { retryAfter }),
      documentationUrl: getDocumentationUrl(errorCode),
    },
    requestId,
    timestamp,
  };

  // Set retry-after header for rate limiting
  if (retryAfter) {
    res.set('Retry-After', retryAfter.toString());
  }

  // Set CORS headers for error responses
  res.set('Access-Control-Allow-Origin', req.headers.origin || '*');
  res.set('Access-Control-Allow-Credentials', 'true');

  res.status(statusCode).json(errorResponse);
}

function generateRequestId(): string {
  return `req_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

function getDocumentationUrl(errorCode: string): string | undefined {
  const baseUrl = 'https://docs.adxcore.com/api/errors';
  
  const errorDocs: Record<string, string> = {
    'VALIDATION_FAILED': `${baseUrl}/validation`,
    'UNAUTHORIZED': `${baseUrl}/authentication`,
    'FORBIDDEN': `${baseUrl}/authorization`,
    'RATE_LIMIT_EXCEEDED': `${baseUrl}/rate-limiting`,
    'TOKEN_EXPIRED': `${baseUrl}/token-refresh`,
    'INVALID_TOKEN': `${baseUrl}/authentication`,
  };

  return errorDocs[errorCode];
}

// Custom error classes
export class BFFError extends Error {
  constructor(
    public statusCode: number,
    message: string,
    public code: string = 'BFF_ERROR',
    public details?: any
  ) {
    super(message);
    this.name = 'BFFError';
  }
}

export class CacheError extends BFFError {
  constructor(message: string, details?: any) {
    super(500, message, 'CACHE_ERROR', details);
    this.name = 'CacheError';
  }
}

export class AggregationError extends BFFError {
  constructor(message: string, details?: any) {
    super(500, message, 'AGGREGATION_ERROR', details);
    this.name = 'AggregationError';
  }
}

export class WorkflowError extends BFFError {
  constructor(statusCode: number, message: string, details?: any) {
    super(statusCode, message, 'WORKFLOW_ERROR', details);
    this.name = 'WorkflowError';
  }
}

// Async error wrapper
export function asyncHandler(
  fn: (req: Request, res: Response, next: NextFunction) => Promise<any>
) {
  return (req: Request, res: Response, next: NextFunction) => {
    Promise.resolve(fn(req, res, next)).catch(next);
  };
}