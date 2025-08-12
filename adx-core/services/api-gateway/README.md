# ADX Core API Gateway

The ADX Core API Gateway is a Temporal-first intelligent routing service that provides a single entry point for all client requests. It implements intelligent routing between direct service calls and Temporal workflow orchestration based on operation complexity.

## Features

### Temporal-First Architecture
- **Intelligent Routing**: Automatically routes simple operations directly to services and complex operations through Temporal workflows
- **Workflow Orchestration**: Initiates and manages Temporal workflows for multi-step business processes
- **Operation Tracking**: Provides real-time status and progress tracking for long-running workflows
- **Workflow Management**: Supports workflow cancellation, signaling, and querying

### Security & Authentication
- **JWT Authentication**: Validates JWT tokens and extracts user/tenant context
- **Multi-Tenant Support**: Enforces tenant isolation and context propagation
- **Permission-Based Authorization**: Validates user permissions for requested operations
- **Rate Limiting**: Implements tenant and user-aware rate limiting with Redis backend

### Operational Excellence
- **Health Monitoring**: Comprehensive health checks for all downstream services
- **Request Tracing**: Distributed tracing with unique request IDs
- **Structured Logging**: Detailed logging for debugging and monitoring
- **Metrics & Observability**: Prometheus metrics and OpenTelemetry integration

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway (Port 8080)                 │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │                Middleware Stack                         │ │
│  │  • Request ID Generation                                │ │
│  │  • Authentication & Authorization                       │ │
│  │  • Rate Limiting                                        │ │
│  │  • Tenant Context Validation                            │ │
│  │  • Distributed Tracing                                  │ │
│  │  • Request/Response Logging                             │ │
│  │  • CORS & Compression                                   │ │
│  └─────────────────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Intelligent Router                         │ │
│  │  • Operation Classification                             │ │
│  │  • Direct vs Workflow Routing                           │ │
│  │  • Service Discovery                                    │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
                              │
                ┌─────────────┴─────────────┐
                │                           │
        ┌───────▼────────┐         ┌────────▼────────┐
        │ Direct Service │         │ Temporal Client │
        │ Proxy          │         │ (Workflows)     │
        └────────────────┘         └─────────────────┘
                │                           │
    ┌───────────┼───────────┐              │
    │           │           │              │
┌───▼───┐  ┌───▼───┐  ┌───▼───┐      ┌────▼────┐
│ Auth  │  │ User  │  │ File  │      │Temporal │
│Service│  │Service│  │Service│      │ Server  │
└───────┘  └───────┘  └───────┘      └─────────┘
```

## Operation Classification

### Direct Operations (Simple CRUD)
- `GET /api/v1/users/{id}` → User Service
- `PUT /api/v1/users/{id}` → User Service  
- `GET /api/v1/tenants` → Tenant Service
- `GET /api/v1/files/{id}` → File Service

### Workflow Operations (Complex Processes)
- `POST /api/v1/users` → User Registration Workflow
- `POST /api/v1/tenants` → Tenant Creation Workflow
- `POST /api/v1/files` → File Upload Workflow
- `POST /api/v1/workflows/switch-tenant` → Tenant Switch Workflow

## API Endpoints

### Health & Status
```
GET  /health                           # API Gateway health check
GET  /api/v1/health                    # Detailed service health
```

### Workflow Management
```
GET  /api/v1/workflows/{id}/status     # Get workflow status
POST /api/v1/workflows/{id}/cancel     # Cancel workflow
POST /api/v1/workflows/{id}/signal/{signal} # Send signal to workflow
```

### Service Proxying
```
*    /api/v1/auth/*                    # Auth Service operations
*    /api/v1/users/*                   # User Service operations  
*    /api/v1/tenants/*                 # Tenant Service operations
*    /api/v1/files/*                   # File Service operations
```

## Configuration

The API Gateway uses environment variables for configuration. Copy `.env.example` to `.env` and adjust values:

### Server Configuration
- `API_GATEWAY_SERVER_HOST`: Server bind address (default: 0.0.0.0)
- `API_GATEWAY_SERVER_PORT`: Server port (default: 8080)
- `API_GATEWAY_SERVER_REQUEST_TIMEOUT_SECONDS`: Request timeout (default: 30)

### Temporal Configuration
- `API_GATEWAY_TEMPORAL_SERVER_ADDRESS`: Temporal server address (default: localhost:7233)
- `API_GATEWAY_TEMPORAL_NAMESPACE`: Temporal namespace (default: adx-core-development)

### Service Endpoints
- `API_GATEWAY_SERVICES_AUTH_SERVICE_BASE_URL`: Auth service URL
- `API_GATEWAY_SERVICES_USER_SERVICE_BASE_URL`: User service URL
- `API_GATEWAY_SERVICES_TENANT_SERVICE_BASE_URL`: Tenant service URL
- `API_GATEWAY_SERVICES_FILE_SERVICE_BASE_URL`: File service URL

### Authentication
- `API_GATEWAY_AUTH_JWT_SECRET`: JWT signing secret
- `API_GATEWAY_AUTH_REQUIRE_AUTH`: Enable authentication (default: true)

### Rate Limiting
- `API_GATEWAY_RATE_LIMITING_ENABLED`: Enable rate limiting (default: true)
- `API_GATEWAY_RATE_LIMITING_REQUESTS_PER_MINUTE`: Per-minute limit (default: 100)
- `API_GATEWAY_RATE_LIMITING_REQUESTS_PER_HOUR`: Per-hour limit (default: 1000)

## Development

### Running the API Gateway

```bash
# From the api-gateway directory
cargo run

# Or from the workspace root
cargo run --bin api-gateway
```

### Running with Docker

```bash
# Start infrastructure services
docker-compose -f infrastructure/docker/docker-compose.dev.yml up -d

# Run the API Gateway
cargo run --bin api-gateway
```

### Testing

```bash
# Run unit tests
cargo test

# Run integration tests (requires Redis and Temporal)
cargo test --test integration_tests

# Test with curl
curl http://localhost:8080/health
```

## Request Flow

### Direct Operation Example
```
1. Client → GET /api/v1/users/123
2. API Gateway → Middleware Stack (auth, rate limiting, etc.)
3. API Gateway → Classify as Direct Operation
4. API Gateway → Proxy to User Service
5. User Service → Return user data
6. API Gateway → Return response to client
```

### Workflow Operation Example
```
1. Client → POST /api/v1/users (user registration)
2. API Gateway → Middleware Stack
3. API Gateway → Classify as Workflow Operation
4. API Gateway → Start user_registration workflow
5. Temporal → Execute workflow activities
6. API Gateway → Return operation tracking info
7. Client → Poll /api/v1/workflows/{id}/status for completion
```

## Error Handling

The API Gateway provides standardized error responses:

```json
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded for this endpoint",
    "details": {
      "limit_type": "per_hour",
      "current_usage": 1001,
      "limit": 1000
    },
    "retry_after": 3600
  },
  "request_id": "req_123456789",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Monitoring

### Health Checks
- `/health` - Basic health check
- `/api/v1/health` - Detailed service health with response times

### Metrics
- Request count and duration by endpoint
- Rate limiting metrics
- Workflow execution metrics
- Service health metrics

### Logging
- Structured JSON logging
- Request/response logging with correlation IDs
- Error logging with stack traces
- Performance metrics logging

## Security

### Authentication
- JWT token validation
- Token expiration checking
- User context extraction

### Authorization
- Permission-based access control
- Tenant isolation enforcement
- Role-based operation validation

### Rate Limiting
- Per-user and per-tenant limits
- Multiple time windows (minute, hour, day)
- Burst protection
- Redis-backed counters

## Deployment

### Production Configuration
- Use strong JWT secrets
- Configure appropriate rate limits
- Set up Redis clustering
- Enable request logging
- Configure CORS properly
- Use HTTPS termination

### Scaling
- Horizontal scaling supported
- Stateless design
- Redis for shared state
- Load balancer compatible

## Troubleshooting

### Common Issues

1. **Service Unavailable Errors**
   - Check downstream service health
   - Verify service URLs in configuration
   - Check network connectivity

2. **Rate Limiting Issues**
   - Verify Redis connectivity
   - Check rate limit configuration
   - Monitor rate limit metrics

3. **Authentication Failures**
   - Verify JWT secret configuration
   - Check token expiration
   - Validate token format

4. **Workflow Execution Issues**
   - Check Temporal server connectivity
   - Verify workflow registration
   - Monitor workflow execution logs