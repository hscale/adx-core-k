# Tenant BFF Service

The Tenant Backend-for-Frontend (BFF) service provides optimized APIs for tenant-related operations in the ADX Core platform. It acts as a Temporal workflow client and implements Redis caching for improved performance.

## Features

- **Temporal Workflow Integration**: Acts as a client for tenant-related workflows
- **Redis Caching**: Caches tenant data, analytics, and configuration for optimal performance
- **Multi-Tenant Support**: Full tenant isolation and context management
- **Rate Limiting**: Comprehensive rate limiting at global, tenant, and user levels
- **Aggregated APIs**: Optimized endpoints that combine data from multiple services
- **Real-time Analytics**: Tenant usage and analytics with multiple time periods
- **Tenant Switching**: Optimized tenant switching with workflow orchestration

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│  Tenant Micro-  │    │   Tenant BFF    │    │   Backend       │
│  Frontend       │◄──►│   Service       │◄──►│   Services      │
│  (Port 3002)    │    │  (Port 4002)    │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │     Redis       │
                       │    Cache        │
                       └─────────────────┘
```

## API Endpoints

### Tenant Management
- `GET /api/tenants/current` - Get current tenant information
- `GET /api/tenants/available` - Get user's available tenants
- `POST /api/tenants/switch` - Switch tenant context
- `GET /api/tenants/:tenantId/overview` - Get tenant overview
- `GET /api/tenants/:tenantId/memberships` - Get tenant memberships
- `GET /api/tenants/:tenantId/analytics` - Get tenant analytics
- `GET /api/tenants/:tenantId/usage` - Get tenant usage information
- `GET /api/tenants/:tenantId/configuration` - Get tenant configuration
- `PUT /api/tenants/:tenantId/configuration` - Update tenant configuration

### Workflow Operations
- `POST /api/workflows/initiate` - Initiate tenant-related workflow
- `GET /api/workflows/:operationId/status` - Get workflow status
- `POST /api/workflows/:operationId/cancel` - Cancel workflow
- `POST /api/workflows/provision-tenant` - Provision new tenant
- `POST /api/workflows/migrate-tenant` - Migrate tenant
- `POST /api/workflows/bulk-invite-users` - Bulk invite users
- `POST /api/workflows/bulk-update-memberships` - Bulk update memberships
- `GET /api/workflows/history` - Get workflow history

### Aggregated Data
- `GET /api/aggregated/dashboard` - Get tenant dashboard data
- `GET /api/aggregated/summary` - Get tenant summary
- `GET /api/aggregated/analytics-overview` - Get analytics overview
- `GET /api/aggregated/health` - Get tenant health status
- `GET /api/aggregated/quick-stats` - Get quick stats for navigation

## Configuration

### Environment Variables

```bash
# Server Configuration
PORT=4002
NODE_ENV=development

# JWT Configuration
JWT_SECRET=your-jwt-secret-key

# Redis Configuration
REDIS_URL=redis://localhost:6379

# Service URLs
API_GATEWAY_URL=http://localhost:8080
TENANT_SERVICE_URL=http://localhost:8085
USER_SERVICE_URL=http://localhost:8082
AUTH_SERVICE_URL=http://localhost:8081
WORKFLOW_SERVICE_URL=http://localhost:8084

# Temporal Configuration
TEMPORAL_SERVER_URL=localhost:7233
TEMPORAL_NAMESPACE=default

# CORS Configuration
CORS_ORIGIN=http://localhost:3000,http://localhost:3002

# Cache Configuration
CACHE_TTL_TENANT_DATA=300
CACHE_TTL_TENANT_ANALYTICS=600
CACHE_TTL_TENANT_CONFIG=1800

# Rate Limiting
RATE_LIMIT_WINDOW_MS=60000
RATE_LIMIT_MAX_REQUESTS=100
RATE_LIMIT_TENANT_SWITCH_MAX=10
```

## Development

### Prerequisites
- Node.js 18+
- Redis server
- Access to ADX Core backend services

### Installation
```bash
cd bff-services/tenant-bff
npm install
```

### Development Server
```bash
npm run dev
```

### Build
```bash
npm run build
```

### Production
```bash
npm start
```

### Testing
```bash
npm test
npm run test:watch
```

## Caching Strategy

### Cache Keys
- `tenant:{tenantId}` - Tenant data (5 minutes)
- `tenant_context:{tenantId}:{userId}` - User tenant context (5 minutes)
- `tenant_memberships:{tenantId}` - Tenant memberships (5 minutes)
- `user_tenants:{userId}` - User's available tenants (5 minutes)
- `tenant_analytics:{tenantId}:{period}` - Tenant analytics (10 minutes)
- `tenant_config:{tenantId}` - Tenant configuration (30 minutes)

### Cache Invalidation
- Automatic TTL expiration
- Manual invalidation on data updates
- Admin cache clearing endpoints
- Tenant context invalidation on tenant switch

## Rate Limiting

### Global Limits
- 100 requests per minute per IP

### Tenant-Specific Limits
- Free tier: 100 requests per minute
- Professional tier: 1000 requests per minute
- Enterprise tier: 10000 requests per minute

### User-Specific Limits
- 500 requests per minute per user

### Endpoint-Specific Limits
- Tenant switching: 10 requests per minute
- Analytics: 30 requests per minute
- Workflows: 20 requests per minute
- Configuration: 15 requests per minute

## Security

### Authentication
- JWT token validation
- Session management with Redis
- Optional authentication for public endpoints

### Authorization
- Role-based access control
- Permission-based access control
- Tenant-specific permissions
- Admin-only endpoints

### Security Headers
- Helmet.js security headers
- CORS configuration
- Content Security Policy
- XSS protection

## Monitoring

### Health Checks
- Service health endpoint: `/health`
- Redis connectivity check
- Backend service connectivity check

### Metrics
- Request/response metrics
- Cache hit/miss rates
- Rate limiting statistics
- Error rates and types

### Logging
- Structured JSON logging
- Request/response logging
- Error logging with stack traces
- Performance metrics

## Error Handling

### Error Types
- Validation errors (400)
- Authentication errors (401)
- Authorization errors (403)
- Not found errors (404)
- Rate limit errors (429)
- Server errors (500)
- Service unavailable (503)

### Error Response Format
```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {},
    "timestamp": "2024-01-15T10:30:00Z",
    "requestId": "req_123456789"
  }
}
```

## Performance Optimization

### Caching
- Multi-level caching strategy
- Cache warming for frequently accessed data
- Intelligent cache invalidation
- Cache compression for large objects

### Request Optimization
- Request batching where possible
- Response compression
- Efficient database queries
- Connection pooling

### Monitoring
- Response time monitoring
- Cache performance metrics
- Database query performance
- Memory usage tracking

## Deployment

### Docker
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY dist ./dist
EXPOSE 4002
CMD ["npm", "start"]
```

### Kubernetes
- Horizontal Pod Autoscaler
- Resource limits and requests
- Health check probes
- ConfigMap for environment variables
- Secret for sensitive data

## Contributing

1. Follow TypeScript best practices
2. Add comprehensive error handling
3. Include unit and integration tests
4. Update documentation
5. Follow security guidelines
6. Implement proper logging
7. Add performance monitoring