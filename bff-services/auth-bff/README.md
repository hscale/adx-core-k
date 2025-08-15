# Auth BFF Service

Backend-for-Frontend service for authentication and user management in ADX Core. This service acts as a Temporal workflow client and provides optimized APIs for the Auth micro-frontend.

## Features

- **Temporal Workflow Integration**: Acts as a client for Temporal workflows
- **Redis Caching**: Caches authentication data and user sessions
- **Aggregated Endpoints**: Combines auth, user, and tenant data
- **Real-time Updates**: WebSocket support for authentication status updates
- **Request Batching**: Optimized batch operations for multiple requests
- **Rate Limiting**: Comprehensive rate limiting with tenant awareness
- **Multi-tenant Support**: Full tenant isolation and context management

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Auth Micro-   │    │   Auth BFF      │    │   Backend       │
│   Frontend      │◄──►│   Service       │◄──►│   Services      │
│  (Port 3001)    │    │  (Port 4001)    │    │                 │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │     Redis       │
                       │    Cache        │
                       └─────────────────┘
                              │
                              ▼
                       ┌─────────────────┐
                       │   WebSocket     │
                       │   Service       │
                       └─────────────────┘
```

## API Endpoints

### Authentication
- `POST /api/auth/login` - User login with workflow support
- `POST /api/auth/register` - User registration
- `POST /api/auth/logout` - User logout
- `POST /api/auth/refresh` - Token refresh
- `POST /api/auth/password-reset` - Password reset request
- `POST /api/auth/password-reset/confirm` - Password reset confirmation
- `POST /api/auth/verify-email` - Email verification
- `POST /api/auth/tenant-switch` - Tenant switching (workflow-based)
- `PUT /api/auth/profile` - Profile updates
- `GET /api/auth/me` - Current user info (aggregated)

### Multi-Factor Authentication
- `POST /api/auth/mfa/setup` - MFA setup workflow
- `POST /api/auth/mfa/verify` - MFA verification

### Workflows
- `POST /api/workflows/execute` - Generic workflow execution
- `POST /api/workflows/batch` - Batch workflow execution
- `GET /api/workflows/:operationId/status` - Workflow status
- `POST /api/workflows/:operationId/cancel` - Cancel workflow
- `GET /api/workflows/history` - User workflow history
- `GET /api/workflows/stats` - Workflow statistics
- `GET /api/workflows/:operationId/stream` - Real-time workflow updates (SSE)
- `GET /api/workflows/templates` - Available workflow templates

### Aggregated Data
- `GET /api/aggregated/dashboard` - Dashboard data aggregation
- `GET /api/aggregated/profile` - Extended profile data
- `GET /api/aggregated/tenant-overview` - Tenant overview with user context
- `GET /api/aggregated/activity` - Activity feed with context
- `GET /api/aggregated/notifications` - Notifications with context
- `GET /api/aggregated/stats` - Quick stats aggregation
- `POST /api/aggregated/batch` - Batch data aggregation

### System
- `GET /health` - Health check
- `GET /api/ws/stats` - WebSocket connection statistics
- `POST /api/cache/clear` - Cache management (admin only)
- `GET /api/metrics` - System metrics (admin only)

## WebSocket Support

Real-time authentication status updates via WebSocket:

```javascript
const ws = new WebSocket('ws://localhost:4001/ws?token=your-jwt-token');

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  
  switch (message.type) {
    case 'auth_status_update':
      // Handle authentication status changes
      break;
    case 'connected':
      // Connection established
      break;
    // ... other message types
  }
};

// Subscribe to specific channels
ws.send(JSON.stringify({
  type: 'subscribe',
  channels: ['auth_updates', 'workflow_updates']
}));
```

## Configuration

### Environment Variables

```bash
# Server Configuration
PORT=4001
NODE_ENV=development

# CORS Configuration
CORS_ORIGIN=http://localhost:3000,http://localhost:3001

# Redis Configuration
REDIS_URL=redis://localhost:6379

# API Gateway Configuration
API_GATEWAY_URL=http://localhost:8080
AUTH_SERVICE_URL=http://localhost:8081
USER_SERVICE_URL=http://localhost:8082
TENANT_SERVICE_URL=http://localhost:8085

# JWT Configuration
JWT_SECRET=your-jwt-secret-key
JWT_EXPIRES_IN=24h

# Rate Limiting
RATE_LIMIT_WINDOW_MS=900000
RATE_LIMIT_MAX_REQUESTS=100

# WebSocket Configuration
WS_HEARTBEAT_INTERVAL=30000
WS_MAX_CONNECTIONS=1000

# Cache Configuration
CACHE_TTL_SECONDS=300
CACHE_MAX_SIZE=1000
```

## Development

### Prerequisites
- Node.js 18+
- Redis server
- Backend services running (Auth, User, Tenant, API Gateway)

### Installation

```bash
cd bff-services/auth-bff
npm install
```

### Development Server

```bash
npm run dev
```

### Build

```bash
npm run build
npm start
```

### Testing

```bash
npm test
npm run test:watch
```

## Caching Strategy

The Auth BFF implements a multi-layered caching strategy:

### Session Caching
- **Key Pattern**: `session:{sessionId}`
- **TTL**: 1 hour
- **Data**: Complete session information

### User Data Caching
- **Key Pattern**: `user:{userId}`
- **TTL**: 5 minutes
- **Data**: User profile and permissions

### Tenant Data Caching
- **Key Pattern**: `tenant:{tenantId}`
- **TTL**: 5 minutes
- **Data**: Tenant configuration and features

### Aggregated Data Caching
- **Key Pattern**: `aggregated:{type}:{userId}:{tenantId}`
- **TTL**: 1-5 minutes (varies by data type)
- **Data**: Pre-computed aggregated responses

### Workflow Caching
- **Key Pattern**: `workflow:{operationId}`
- **TTL**: 1 hour
- **Data**: Workflow status and metadata

## Rate Limiting

Multiple rate limiting strategies are implemented:

### General API Rate Limiting
- **Window**: 15 minutes
- **Limit**: 1000 requests per user/IP
- **Key**: `user:{userId}` or `ip:{ip}`

### Authentication Rate Limiting
- **Window**: 15 minutes
- **Limit**: 10 requests per IP
- **Key**: `auth:{ip}`

### Password Reset Rate Limiting
- **Window**: 1 hour
- **Limit**: 3 requests per IP
- **Key**: `password_reset:{ip}`

### Workflow Rate Limiting
- **Window**: 5 minutes
- **Limit**: 50 requests per user
- **Key**: `workflow:{userId}`

## Error Handling

Comprehensive error handling with standardized error responses:

```json
{
  "error": {
    "code": "ERROR_CODE",
    "message": "Human readable message",
    "details": {},
    "validationErrors": [],
    "retryAfter": 60,
    "documentationUrl": "https://docs.adxcore.com/api/errors/error-code"
  },
  "requestId": "req_123456789",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Security Features

- **Helmet.js**: Security headers
- **CORS**: Configurable cross-origin resource sharing
- **JWT Validation**: Token verification and session management
- **Rate Limiting**: Multiple rate limiting strategies
- **Input Validation**: Zod schema validation
- **Tenant Isolation**: Multi-tenant security boundaries
- **Permission Checking**: Role and permission-based access control

## Monitoring and Observability

- **Health Checks**: Comprehensive health monitoring
- **Metrics**: System and application metrics
- **Logging**: Structured logging with request correlation
- **WebSocket Stats**: Real-time connection monitoring
- **Cache Metrics**: Redis performance monitoring

## Deployment

### Docker

```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY dist ./dist
EXPOSE 4001
CMD ["npm", "start"]
```

### Docker Compose

```yaml
version: '3.8'
services:
  auth-bff:
    build: .
    ports:
      - "4001:4001"
    environment:
      - NODE_ENV=production
      - REDIS_URL=redis://redis:6379
      - API_GATEWAY_URL=http://api-gateway:8080
    depends_on:
      - redis
      - api-gateway
```

## Performance Considerations

- **Connection Pooling**: Efficient Redis connection management
- **Request Batching**: Batch multiple API calls
- **Caching**: Multi-layered caching strategy
- **Compression**: Response compression
- **Keep-Alive**: HTTP keep-alive connections
- **WebSocket Optimization**: Efficient real-time communication

## Contributing

1. Follow TypeScript best practices
2. Add comprehensive error handling
3. Include unit tests for new features
4. Update documentation
5. Follow the established caching patterns
6. Ensure proper tenant isolation

## License

MIT License - see LICENSE file for details