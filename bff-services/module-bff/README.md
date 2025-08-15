# Module BFF Service

Backend-for-Frontend service for the Module Management system in ADX Core. This service acts as a Temporal workflow client and provides optimized APIs for the Module micro-frontend.

## Features

### Module Marketplace
- Module search and filtering
- Featured, trending, and recommended modules
- Module details and metadata
- Category and tag-based browsing

### Module Management
- Installed module listing
- Module configuration management
- Module status monitoring
- Module logs and metrics

### Module Development
- Development project management
- Source code management
- Testing and validation
- Publishing workflows

### Workflow Integration
- Temporal workflow client for module operations
- Asynchronous operation tracking
- Progress monitoring and status updates
- Workflow cancellation support

## API Endpoints

### Health Check
- `GET /api/health` - Service health status
- `GET /api/health/ready` - Readiness check

### Marketplace
- `GET /api/marketplace/search` - Search modules
- `GET /api/marketplace/modules/:id` - Get module details
- `GET /api/marketplace/featured` - Get featured modules
- `GET /api/marketplace/trending` - Get trending modules
- `GET /api/marketplace/recommended` - Get recommended modules

### Module Management
- `GET /api/modules/installed` - List installed modules
- `GET /api/modules/:id/configuration` - Get module configuration
- `PUT /api/modules/:id/configuration` - Update module configuration
- `GET /api/modules/:id/status` - Get module status
- `GET /api/modules/:id/logs` - Get module logs

### Development
- `GET /api/development/projects` - List development projects
- `POST /api/development/projects` - Create new project
- `GET /api/development/projects/:id` - Get project details
- `PUT /api/development/projects/:id` - Update project
- `DELETE /api/development/projects/:id` - Delete project
- `GET /api/development/projects/:id/files` - Get project files
- `PUT /api/development/projects/:id/files/:path` - Update file

### Workflows
- `POST /api/workflows/install-module` - Install module
- `POST /api/workflows/uninstall-module` - Uninstall module
- `POST /api/workflows/activate-module` - Activate module
- `POST /api/workflows/deactivate-module` - Deactivate module
- `POST /api/workflows/test-module` - Test module
- `POST /api/workflows/publish-module` - Publish module
- `GET /api/workflows/:id/status` - Get workflow status
- `POST /api/workflows/:id/cancel` - Cancel workflow

## Configuration

### Environment Variables
```bash
# Server Configuration
PORT=4006
NODE_ENV=development

# Redis Configuration
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=
REDIS_DB=0

# API Gateway Configuration
API_GATEWAY_URL=http://localhost:8080
API_GATEWAY_TIMEOUT=30000

# Module Service Configuration
MODULE_SERVICE_URL=http://localhost:8086

# Authentication
JWT_SECRET=your-jwt-secret-here

# Logging
LOG_LEVEL=info
LOG_FORMAT=json

# Rate Limiting
RATE_LIMIT_WINDOW_MS=900000
RATE_LIMIT_MAX_REQUESTS=100

# Cache Configuration
CACHE_TTL_SECONDS=300
CACHE_MAX_SIZE=1000
```

## Development

### Setup
```bash
cd bff-services/module-bff
npm install
cp .env.example .env
# Edit .env with your configuration
```

### Running
```bash
# Development mode with hot reload
npm run dev

# Production build and start
npm run build
npm start
```

### Testing
```bash
# Run tests once
npm test

# Run tests in watch mode
npm run test:watch
```

### Linting
```bash
npm run lint
npm run type-check
```

## Architecture

### Middleware Stack
- **Helmet**: Security headers
- **CORS**: Cross-origin resource sharing
- **Compression**: Response compression
- **Rate Limiting**: Request rate limiting
- **Authentication**: JWT token validation
- **Tenant Context**: Multi-tenant request context
- **Error Handling**: Centralized error handling

### Services Integration
- **API Gateway**: Routes complex operations to Temporal workflows
- **Module Service**: Direct communication for simple operations
- **Redis**: Caching and session storage
- **Temporal**: Workflow orchestration client

### Data Flow
1. Frontend sends request to BFF
2. BFF validates authentication and tenant context
3. For simple operations: Direct API call to backend services
4. For complex operations: Initiate Temporal workflow via API Gateway
5. BFF caches responses and returns optimized data to frontend

## Security

### Authentication
- JWT token validation on all protected routes
- User context extraction and validation
- Permission-based access control

### Authorization
- Tenant-based data isolation
- Role-based permission checking
- Resource-level access control

### Rate Limiting
- Per-IP rate limiting
- Configurable limits and windows
- Graceful degradation on limit exceeded

### Input Validation
- Request payload validation
- Parameter sanitization
- SQL injection prevention

## Monitoring

### Logging
- Structured JSON logging
- Request/response logging
- Error tracking and stack traces
- Performance metrics

### Health Checks
- Service health endpoint
- Dependency health checks
- Readiness probes for Kubernetes

### Metrics
- Request count and duration
- Error rates and types
- Cache hit/miss ratios
- Workflow operation metrics

## Deployment

### Docker
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY dist ./dist
EXPOSE 4006
CMD ["node", "dist/server.js"]
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: module-bff
spec:
  replicas: 3
  selector:
    matchLabels:
      app: module-bff
  template:
    metadata:
      labels:
        app: module-bff
    spec:
      containers:
      - name: module-bff
        image: adx-core/module-bff:latest
        ports:
        - containerPort: 4006
        env:
        - name: PORT
          value: "4006"
        - name: REDIS_URL
          valueFrom:
            secretKeyRef:
              name: redis-secret
              key: url
        livenessProbe:
          httpGet:
            path: /api/health
            port: 4006
        readinessProbe:
          httpGet:
            path: /api/health/ready
            port: 4006
```

## Performance

### Caching Strategy
- Redis-based response caching
- Configurable TTL per endpoint
- Cache invalidation on data changes
- Memory-efficient cache management

### Optimization
- Response compression
- Connection pooling
- Async/await patterns
- Efficient data serialization

### Scalability
- Stateless service design
- Horizontal scaling support
- Load balancer compatibility
- Database connection pooling