# ADX CORE API Documentation

## Overview

ADX CORE provides a comprehensive API ecosystem built on Temporal-first microservices architecture. This documentation covers all services, BFF endpoints, and integration patterns.

## Architecture

### Temporal-First Design
- **Complex Operations**: Implemented as Temporal workflows for reliability and observability
- **Simple Operations**: Direct HTTP endpoints for optimal performance
- **Cross-Service Communication**: Coordinated through Temporal workflows only

### Service Ecosystem
- **API Gateway** (Port 8080): Single entry point with intelligent routing
- **Backend Services**: Dual-mode services (HTTP + Temporal worker)
- **BFF Services**: Optional optimization layer for micro-frontends
- **Frontend Services**: Module Federation-based micro-frontends

## Quick Start

### Authentication
All API requests require authentication via JWT tokens:

```bash
# Login to get token
curl -X POST https://api.adxcore.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email": "user@example.com", "password": "password"}'

# Use token in subsequent requests
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  https://api.adxcore.com/api/v1/tenants
```

### Multi-Tenant Context
Include tenant context in requests:

```bash
# Via header (recommended)
curl -H "X-Tenant-ID: tenant-123" \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  https://api.adxcore.com/api/v1/users

# Via subdomain
curl -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  https://tenant-123.adxcore.com/api/v1/users
```

## Service Documentation

### Core Services
- [API Gateway](./api-gateway.md) - Central routing and workflow orchestration
- [Auth Service](./auth-service.md) - Authentication, authorization, SSO
- [Tenant Service](./tenant-service.md) - Multi-tenant management
- [User Service](./user-service.md) - User management and profiles
- [File Service](./file-service.md) - File storage and processing
- [Workflow Service](./workflow-service.md) - Cross-service orchestration

### BFF Services
- [Auth BFF](./bff/auth-bff.md) - Authentication data aggregation
- [Tenant BFF](./bff/tenant-bff.md) - Tenant management optimization
- [File BFF](./bff/file-bff.md) - File operations optimization
- [User BFF](./bff/user-bff.md) - User data aggregation
- [Workflow BFF](./bff/workflow-bff.md) - Workflow monitoring optimization

## API Patterns

### Direct Operations (Simple CRUD)
```bash
# Get resources
GET /api/v1/{resource}
GET /api/v1/{resource}/{id}

# Create resources
POST /api/v1/{resource}

# Update resources
PUT /api/v1/{resource}/{id}
PATCH /api/v1/{resource}/{id}

# Delete resources
DELETE /api/v1/{resource}/{id}
```

### Workflow Operations (Complex Processes)
```bash
# Initiate workflow
POST /api/v1/workflows/{workflow-type}

# Check workflow status
GET /api/v1/workflows/{operation-id}/status

# Stream workflow progress
GET /api/v1/workflows/{operation-id}/stream

# Cancel workflow
POST /api/v1/workflows/{operation-id}/cancel
```

## Response Formats

### Standard Response
```json
{
  "data": {...},
  "meta": {
    "timestamp": "2024-01-15T10:30:00Z",
    "request_id": "req_123456789"
  }
}
```

### Workflow Response (Asynchronous)
```json
{
  "operation_id": "wf_123456789",
  "status_url": "/api/v1/workflows/wf_123456789/status",
  "stream_url": "/api/v1/workflows/wf_123456789/stream",
  "estimated_duration_seconds": 30
}
```

### Error Response
```json
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Request validation failed",
    "details": {...},
    "documentation_url": "https://docs.adxcore.com/api/errors"
  },
  "request_id": "req_123456789",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Rate Limits

### Default Limits
- **Per User**: 1000 requests/hour
- **Per Tenant**: 10000 requests/hour
- **Workflow Operations**: 100 workflows/hour per tenant

### Headers
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1642248600
```

## Webhooks

### Event Types
- `tenant.created`
- `user.registered`
- `file.uploaded`
- `workflow.completed`
- `module.installed`

### Webhook Format
```json
{
  "event": "tenant.created",
  "data": {...},
  "timestamp": "2024-01-15T10:30:00Z",
  "tenant_id": "tenant-123"
}
```

## SDKs and Libraries

### Official SDKs
- [JavaScript/TypeScript SDK](./sdks/javascript.md)
- [Python SDK](./sdks/python.md)
- [Go SDK](./sdks/go.md)
- [Rust SDK](./sdks/rust.md)

### Community SDKs
- [PHP SDK](./sdks/php.md)
- [Ruby SDK](./sdks/ruby.md)
- [Java SDK](./sdks/java.md)

## OpenAPI Specifications

### Download Specifications
- [API Gateway OpenAPI](./openapi/api-gateway.yaml)
- [Auth Service OpenAPI](./openapi/auth-service.yaml)
- [Tenant Service OpenAPI](./openapi/tenant-service.yaml)
- [User Service OpenAPI](./openapi/user-service.yaml)
- [File Service OpenAPI](./openapi/file-service.yaml)
- [Workflow Service OpenAPI](./openapi/workflow-service.yaml)

### Interactive Documentation
- [Swagger UI](https://api.adxcore.com/docs)
- [Redoc](https://api.adxcore.com/redoc)

## Support

### Resources
- [API Status Page](https://status.adxcore.com)
- [Developer Forum](https://forum.adxcore.com)
- [GitHub Issues](https://github.com/adxcore/adx-core/issues)

### Contact
- **Email**: api-support@adxcore.com
- **Slack**: [ADX Core Community](https://adxcore.slack.com)
- **Discord**: [ADX Core Discord](https://discord.gg/adxcore)