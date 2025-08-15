import rateLimit from 'express-rate-limit';
import { createError } from './errorHandler';

const windowMs = parseInt(process.env.RATE_LIMIT_WINDOW_MS || '900000'); // 15 minutes
const maxRequests = parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '100');

export const rateLimitMiddleware = rateLimit({
  windowMs,
  max: maxRequests,
  message: {
    error: {
      code: 'RATE_LIMIT_EXCEEDED',
      message: 'Too many requests, please try again later',
    },
    timestamp: new Date().toISOString(),
  },
  standardHeaders: true,
  legacyHeaders: false,
  handler: (req, res) => {
    const error = createError(
      'Rate limit exceeded',
      429,
      'RATE_LIMIT_EXCEEDED',
      {
        windowMs,
        maxRequests,
        retryAfter: Math.ceil(windowMs / 1000),
      }
    );
    
    res.status(429).json({
      error: {
        code: error.code,
        message: error.message,
        details: error.details,
      },
      timestamp: new Date().toISOString(),
    });
  },
});