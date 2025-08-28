# API Gateway Documentation

## Overview

The API Gateway serves as the single entry point for all ADX CORE services, providing intelligent routing between direct operations and Temporal workflow execution.

**Base URL**: `https://api.adxcore.com`  
**Port**: 8080

## Architecture

### Intelligent Routing
The API Gateway automatically determines whether to:
- Route directly to backend services (simple CRUD operations)
- Initiate Temporal workflows (complex multi-step operations)

### Request Classification
```rust
// Simple operations → Direct routing
GET /api/v1/users/{id}           // Direct to User Service
PUT /api/v1/tenants/{id}/profile // Direct to Tenant Service

// Complex operations → Workflow initiation
POST /api/v1/workflows/user-onboarding    // Temporal workflow
POST /api/v1/workflows/tenant-migration   // Temporal workflow
POST /api/v1/workflows/file-processing    // Temporal workflow
```

## Authentication

### JWT Token Authentication
```bash
# Login
POST /api/v1/auth/login
Content-Type: application/json

{
  "email": "user@example.com",
  "password": "password123",
  "tenant_id": "tenant-123" // Optional
}

# Response
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "refresh_token_here",
  "expires_in": 3600,
  "user": {
    "id": "user-123",
    "email": "user@example.com",
    "tenant_id": "tenant-123"
  }
}
```

### Token Usage
```bash
# Include in Authorization header
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...

# Include tenant context
X-Tenant-ID: tenant-123
```

## Direct Operations

### User Management
```bash
# List users
GET /api/v1/users
Query Parameters:
  - page: int (default: 1)
  - limit: int (default: 20, max: 100)
  - search: string
  - role: string

# Get user
GET /api/v1/users/{user_id}

# Update user profile
PUT /api/v1/users/{user_id}/profile
Content-Type: application/json

{
  "first_name": "John",
  "last_name": "Doe",
  "phone": "+1234567890"
}
```

### Tenant Management
```bash
# List tenants
GET /api/v1/tenants

# Get tenant
GET /api/v1/tenants/{tenant_id}

# Update tenant settings
PUT /api/v1/tenants/{tenant_id}/settings
Content-Type: application/json

{
  "name": "Updated Tenant Name",
  "settings": {
    "theme": "dark",
    "language": "en"
  }
}
```

### File Management
```bash
# List files
GET /api/v1/files
Query Parameters:
  - folder: string
  - type: string (image, document, video, etc.)
  - limit: int

# Get file metadata
GET /api/v1/files/{file_id}

# Download file
GET /api/v1/files/{file_id}/download

# Delete file
DELETE /api/v1/files/{file_id}
```

## Workflow Operations

### User Onboarding Workflow
```bash
POST /api/v1/workflows/user-onboarding
Content-Type: application/json

{
  "user_data": {
    "email": "newuser@example.com",
    "first_name": "Jane",
    "last_name": "Smith",
    "role": "user"
  },
  "tenant_id": "tenant-123",
  "send_welcome_email": true,
  "setup_default_permissions": true
}

# Response (Asynchronous)
{
  "operation_id": "wf_user_onboarding_123456",
  "status_url": "/api/v1/workflows/wf_user_onboarding_123456/status",
  "stream_url": "/api/v1/workflows/wf_user_onboarding_123456/stream",
  "estimated_duration_seconds": 30
}
```

### Tenant Migration Workflow
```bash
POST /api/v1/workflows/tenant-migration
Content-Type: application/json

{
  "tenant_id": "tenant-123",
  "target_tier": "enterprise",
  "migration_options": {
    "preserve_data": true,
    "update_quotas": true,
    "notify_users": true
  }
}

# Response (Asynchronous)
{
  "operation_id": "wf_tenant_migration_789012",
  "status_url": "/api/v1/workflows/wf_tenant_migration_789012/status",
  "estimated_duration_seconds": 300
}
```

### File Processing Workflow
```bash
POST /api/v1/workflows/file-processing
Content-Type: application/json

{
  "file_id": "file-123",
  "operations": [
    "virus_scan",
    "thumbnail_generation",
    "metadata_extraction",
    "ai_analysis"
  ],
  "notify_on_completion": true
}
```

### Bulk Operations Workflow
```bash
POST /api/v1/workflows/bulk-user-import
Content-Type: application/json

{
  "tenant_id": "tenant-123",
  "users": [
    {
      "email": "user1@example.com",
      "first_name": "User",
      "last_name": "One",
      "role": "user"
    },
    {
      "email": "user2@example.com",
      "first_name": "User",
      "last_name": "Two",
      "role": "admin"
    }
  ],
  "send_invitations": true,
  "default_permissions": ["read", "write"]
}
```

## Workflow Status and Monitoring

### Check Workflow Status
```bash
GET /api/v1/workflows/{operation_id}/status

# Response
{
  "operation_id": "wf_user_onboarding_123456",
  "status": "running",
  "progress": {
    "current_step": "sending_welcome_email",
    "total_steps": 5,
    "completed_steps": 3,
    "percentage": 60.0,
    "message": "Sending welcome email to user"
  },
  "started_at": "2024-01-15T10:30:00Z",
  "updated_at": "2024-01-15T10:31:30Z",
  "estimated_completion": "2024-01-15T10:32:00Z"
}
```

### Stream Workflow Progress
```bash
GET /api/v1/workflows/{operation_id}/stream
Accept: text/event-stream

# Server-Sent Events Response
data: {"step": "creating_user", "progress": 20}

data: {"step": "setting_permissions", "progress": 40}

data: {"step": "sending_welcome_email", "progress": 60}

data: {"step": "completed", "progress": 100, "result": {...}}
```

### Cancel Workflow
```bash
POST /api/v1/workflows/{operation_id}/cancel
Content-Type: application/json

{
  "reason": "User requested cancellation"
}

# Response
{
  "operation_id": "wf_user_onboarding_123456",
  "status": "cancelled",
  "cancelled_at": "2024-01-15T10:31:45Z"
}
```

## Rate Limiting

### Limits by Operation Type
- **Direct Operations**: 1000 requests/hour per user
- **Workflow Operations**: 100 workflows/hour per tenant
- **File Operations**: 500 requests/hour per user
- **Bulk Operations**: 10 workflows/hour per tenant

### Rate Limit Headers
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1642248600
X-RateLimit-Type: user_hourly
```

## Error Handling

### Common Error Codes
- `400` - Bad Request (validation errors)
- `401` - Unauthorized (authentication required)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found (resource doesn't exist)
- `409` - Conflict (resource already exists)
- `422` - Unprocessable Entity (business logic error)
- `429` - Too Many Requests (rate limit exceeded)
- `500` - Internal Server Error
- `502` - Bad Gateway (upstream service error)
- `503` - Service Unavailable

### Error Response Format
```json
{
  "error": {
    "code": "WORKFLOW_EXECUTION_FAILED",
    "message": "Workflow execution failed due to validation error",
    "details": {
      "workflow_id": "wf_user_onboarding_123456",
      "failed_step": "user_validation",
      "validation_errors": [
        {
          "field": "email",
          "code": "ALREADY_EXISTS",
          "message": "Email address already exists in tenant"
        }
      ]
    },
    "retry_after": null,
    "documentation_url": "https://docs.adxcore.com/api/errors/workflow-execution-failed"
  },
  "request_id": "req_123456789",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## Health and Monitoring

### Health Check
```bash
GET /health

# Response
{
  "status": "healthy",
  "version": "2.0.0",
  "timestamp": "2024-01-15T10:30:00Z",
  "services": {
    "auth_service": "healthy",
    "tenant_service": "healthy",
    "user_service": "healthy",
    "file_service": "healthy",
    "workflow_service": "healthy",
    "temporal_cluster": "healthy"
  }
}
```

### Metrics Endpoint
```bash
GET /metrics
Accept: text/plain

# Prometheus metrics format
# HELP api_requests_total Total number of API requests
# TYPE api_requests_total counter
api_requests_total{method="GET",endpoint="/api/v1/users",status="200"} 1234

# HELP workflow_executions_total Total number of workflow executions
# TYPE workflow_executions_total counter
workflow_executions_total{workflow_type="user_onboarding",status="completed"} 567
```

## WebSocket Connections

### Real-time Workflow Updates
```javascript
const ws = new WebSocket('wss://api.adxcore.com/ws/workflows/{operation_id}');

ws.onmessage = function(event) {
  const update = JSON.parse(event.data);
  console.log('Workflow update:', update);
  // {
  //   "operation_id": "wf_123456",
  //   "status": "running",
  //   "progress": 75,
  //   "current_step": "finalizing"
  // }
};
```

### Real-time Notifications
```javascript
const ws = new WebSocket('wss://api.adxcore.com/ws/notifications');

ws.onmessage = function(event) {
  const notification = JSON.parse(event.data);
  console.log('Notification:', notification);
  // {
  //   "type": "file_uploaded",
  //   "data": {...},
  //   "timestamp": "2024-01-15T10:30:00Z"
  // }
};
```

## Examples

### Complete User Onboarding Flow
```bash
# 1. Initiate user onboarding workflow
curl -X POST https://api.adxcore.com/api/v1/workflows/user-onboarding \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-Tenant-ID: tenant-123" \
  -H "Content-Type: application/json" \
  -d '{
    "user_data": {
      "email": "newuser@example.com",
      "first_name": "Jane",
      "last_name": "Smith"
    },
    "send_welcome_email": true
  }'

# 2. Monitor workflow progress
curl https://api.adxcore.com/api/v1/workflows/wf_123456/status \
  -H "Authorization: Bearer $TOKEN"

# 3. Get user details after completion
curl https://api.adxcore.com/api/v1/users/user-789 \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-Tenant-ID: tenant-123"
```

### File Upload and Processing
```bash
# 1. Upload file (direct operation)
curl -X POST https://api.adxcore.com/api/v1/files \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-Tenant-ID: tenant-123" \
  -F "file=@document.pdf" \
  -F "folder=documents"

# 2. Initiate file processing workflow
curl -X POST https://api.adxcore.com/api/v1/workflows/file-processing \
  -H "Authorization: Bearer $TOKEN" \
  -H "X-Tenant-ID: tenant-123" \
  -H "Content-Type: application/json" \
  -d '{
    "file_id": "file-123",
    "operations": ["virus_scan", "thumbnail_generation", "ai_analysis"]
  }'

# 3. Stream processing progress
curl https://api.adxcore.com/api/v1/workflows/wf_456789/stream \
  -H "Authorization: Bearer $TOKEN" \
  -H "Accept: text/event-stream"
```