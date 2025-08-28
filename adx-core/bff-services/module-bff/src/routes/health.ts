import { Router } from 'express';

const router = Router();

router.get('/', (req, res) => {
  res.json({
    status: 'healthy',
    service: 'module-bff',
    version: '1.0.0',
    timestamp: new Date().toISOString(),
    uptime: process.uptime(),
  });
});

router.get('/ready', (req, res) => {
  // In a real implementation, this would check dependencies
  // like database connections, external services, etc.
  res.json({
    status: 'ready',
    checks: {
      database: 'healthy',
      redis: 'healthy',
      apiGateway: 'healthy',
    },
    timestamp: new Date().toISOString(),
  });
});

export { router as healthRoutes };