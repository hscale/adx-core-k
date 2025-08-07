# ADX CORE - API Specification

## Overview

ADX CORE provides a comprehensive API-first architecture with multiple protocol support, extensive documentation, and robust security controls. All platform functionality is exposed through well-designed APIs that support various integration patterns.

## API Architecture

### Supported Protocols
- **REST API**: Primary HTTP-based API with JSON payloads
- **GraphQL**: Flexible query language for complex data requirements
- **gRPC**: High-performance RPC for service-to-service communication
- **WebSocket**: Real-time bidirectional communication
- **Webhooks**: Event-driven notifications to external systems

### API Gateway Structure
```
┌─────────────────────────────────────────────────────────────┐
│                    API Gateway (Axum)                      │
├─────────────────┬─────────────────┬─────────────────────────┤
│   REST Routes   │  GraphQL        │    gRPC Services        │
│   /api/v1/*     │  /graphql       │    :9090                │
├─────────────────┼─────────────────┼─────────────────────────┤
│   WebSocket     │  Webhooks       │    Health & Metrics     │
│   /ws/*         │  /webhooks/*    │    /health, /metrics    │
└─────────────────┴─────────────────┴─────────────────────────┘
```

## REST API Specification

### Base Configuration
- **Base URL**: `https://api.adxcore.com/v1`
- **Content Type**: `application/json`
- **Authentication**: Bearer token (JWT)
- **Rate Limiting**: Per-user and per-tenant limits
- **Versioning**: URL path versioning (`/v1`, `/v2`)

### Authentication Endpoints

#### POST /auth/register
Register a new user account.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "securePassword123!",
  "name": "John Doe",
  "tenant_name": "Acme Corp",
  "tenant_slug": "acme-corp"
}
```

**Response (201 Created):**
```json
{
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "status": "pending_verification",
    "created_at": "2024-01-15T10:30:00Z"
  },
  "tenant": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "Acme Corp",
    "slug": "acme-corp",
    "status": "active"
  },
  "verification_required": true
}
```

#### POST /auth/login
Authenticate user and obtain access token.

**Request Body:**
```json
{
  "email": "user@example.com",
  "password": "securePassword123!",
  "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
  "mfa_code": "123456"
}
```

**Response (200 OK):**
```json
{
  "access_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expires_in": 3600,
  "token_type": "Bearer",
  "user": {
    "id": "550e8400-e29b-41d4-a716-446655440000",
    "email": "user@example.com",
    "name": "John Doe",
    "preferences": {
      "language": "en",
      "theme": "light",
      "timezone": "UTC"
    }
  },
  "tenant": {
    "id": "660e8400-e29b-41d4-a716-446655440001",
    "name": "Acme Corp",
    "slug": "acme-corp",
    "role": "owner",
    "permissions": ["*"]
  }
}
```

#### POST /auth/refresh
Refresh access token using refresh token.

**Request Body:**
```json
{
  "refresh_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
}
```

#### POST /auth/logout
Invalidate current session and tokens.

#### POST /auth/forgot-password
Initiate password reset process.

#### POST /auth/reset-password
Complete password reset with token.

#### POST /auth/verify-email
Verify email address with verification token.

### Multi-Factor Authentication Endpoints

#### POST /auth/mfa/setup
Setup MFA for user account.

**Response (200 OK):**
```json
{
  "secret": "JBSWY3DPEHPK3PXP",
  "qr_code": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...",
  "backup_codes": [
    "12345678",
    "87654321",
    "11223344",
    "44332211",
    "55667788"
  ]
}
```

#### POST /auth/mfa/verify
Verify MFA setup with TOTP code.

#### POST /auth/mfa/disable
Disable MFA for user account.

### User Management Endpoints

#### GET /users/me
Get current user profile.

**Response (200 OK):**
```json
{
  "id": "550e8400-e29b-41d4-a716-446655440000",
  "email": "user@example.com",
  "name": "John Doe",
  "avatar_url": "https://cdn.adxcore.com/avatars/user.jpg",
  "preferences": {
    "language": "en",
    "theme": "light",
    "timezone": "America/New_York",
    "date_format": "MM/DD/YYYY",
    "number_format": "1,234.56"
  },
  "status": "active",
  "mfa_enabled": true,
  "last_login_at": "2024-01-15T10:30:00Z",
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-15T10:30:00Z"
}
```

#### PUT /users/me
Update current user profile.

#### PUT /users/me/preferences
Update user preferences.

#### POST /users/me/avatar
Upload user avatar image.

#### GET /users/me/tenants
Get user's tenant memberships.

**Response (200 OK):**
```json
{
  "tenants": [
    {
      "id": "660e8400-e29b-41d4-a716-446655440001",
      "name": "Acme Corp",
      "slug": "acme-corp",
      "role": "owner",
      "permissions": ["*"],
      "status": "active",
      "joined_at": "2024-01-01T00:00:00Z"
    },
    {
      "id": "770e8400-e29b-41d4-a716-446655440002",
      "name": "Beta Inc",
      "slug": "beta-inc",
      "role": "admin",
      "permissions": ["users:read", "users:write", "files:*"],
      "status": "active",
      "joined_at": "2024-01-10T00:00:00Z"
    }
  ],
  "total": 2
}
```

### Tenant Management Endpoints

#### GET /tenants/current
Get current tenant information.

#### PUT /tenants/current
Update current tenant settings.

#### GET /tenants/current/members
List tenant members.

**Response (200 OK):**
```json
{
  "members": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "user": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "John Doe",
        "email": "john@example.com",
        "avatar_url": "https://cdn.adxcore.com/avatars/john.jpg"
      },
      "role": "owner",
      "permissions": ["*"],
      "status": "active",
      "joined_at": "2024-01-01T00:00:00Z"
    }
  ],
  "total": 1,
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total_pages": 1
  }
}
```

#### POST /tenants/current/invitations
Invite user to tenant.

#### PUT /tenants/current/members/{user_id}
Update member role and permissions.

#### DELETE /tenants/current/members/{user_id}
Remove member from tenant.

### File Management Endpoints

#### GET /files
List files with filtering and pagination.

**Query Parameters:**
- `page`: Page number (default: 1)
- `per_page`: Items per page (default: 20, max: 100)
- `search`: Search query
- `visibility`: Filter by visibility (private, tenant, public)
- `owner_id`: Filter by owner
- `tags`: Filter by tags (comma-separated)
- `sort`: Sort field (name, size, created_at, updated_at)
- `order`: Sort order (asc, desc)

**Response (200 OK):**
```json
{
  "files": [
    {
      "id": "880e8400-e29b-41d4-a716-446655440003",
      "name": "document.pdf",
      "original_name": "Important Document.pdf",
      "size_bytes": 1048576,
      "mime_type": "application/pdf",
      "visibility": "private",
      "tags": ["important", "contract"],
      "description": "Contract document for Q1 2024",
      "version": 2,
      "is_current_version": true,
      "owner": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "John Doe",
        "email": "john@example.com"
      },
      "download_url": "https://api.adxcore.com/v1/files/880e8400-e29b-41d4-a716-446655440003/download",
      "preview_url": "https://api.adxcore.com/v1/files/880e8400-e29b-41d4-a716-446655440003/preview",
      "created_at": "2024-01-15T10:30:00Z",
      "updated_at": "2024-01-15T11:00:00Z"
    }
  ],
  "total": 1,
  "pagination": {
    "page": 1,
    "per_page": 20,
    "total_pages": 1
  }
}
```

#### POST /files/upload
Upload a new file.

**Request (multipart/form-data):**
- `file`: File data
- `name`: Optional custom name
- `description`: Optional description
- `tags`: Optional tags (JSON array)
- `visibility`: Visibility setting
- `expires_at`: Optional expiration date

#### GET /files/{file_id}
Get file metadata.

#### PUT /files/{file_id}
Update file metadata.

#### DELETE /files/{file_id}
Delete file.

#### GET /files/{file_id}/download
Download file content.

#### GET /files/{file_id}/preview
Get file preview (for supported formats).

#### GET /files/{file_id}/versions
Get file version history.

#### POST /files/{file_id}/share
Create file share link.

**Request Body:**
```json
{
  "shared_with": "user@example.com",
  "permissions": {
    "read": true,
    "download": true,
    "comment": false
  },
  "expires_at": "2024-02-15T10:30:00Z",
  "password": "optional_password",
  "download_limit": 10
}
```

### Workflow Management Endpoints

#### GET /workflows
List workflows.

**Response (200 OK):**
```json
{
  "workflows": [
    {
      "id": "990e8400-e29b-41d4-a716-446655440004",
      "name": "User Onboarding",
      "description": "Automated user onboarding process",
      "version": 3,
      "status": "published",
      "ai_enhanced": true,
      "category": "user_management",
      "tags": ["onboarding", "automation"],
      "created_by": {
        "id": "550e8400-e29b-41d4-a716-446655440000",
        "name": "John Doe"
      },
      "published_at": "2024-01-10T00:00:00Z",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-10T00:00:00Z"
    }
  ],
  "total": 1
}
```

#### POST /workflows
Create new workflow.

#### GET /workflows/{workflow_id}
Get workflow details including definition.

#### PUT /workflows/{workflow_id}
Update workflow.

#### POST /workflows/{workflow_id}/execute
Execute workflow.

**Request Body:**
```json
{
  "input": {
    "user_email": "newuser@example.com",
    "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
    "welcome_message": "Welcome to Acme Corp!"
  },
  "ai_enhanced": true
}
```

**Response (202 Accepted):**
```json
{
  "execution_id": "aa0e8400-e29b-41d4-a716-446655440005",
  "workflow_id": "990e8400-e29b-41d4-a716-446655440004",
  "status": "running",
  "temporal_workflow_id": "user-onboarding-aa0e8400",
  "started_at": "2024-01-15T10:30:00Z",
  "estimated_duration_ms": 30000
}
```

#### GET /workflows/executions
List workflow executions.

#### GET /workflows/executions/{execution_id}
Get execution details and status.

#### POST /workflows/executions/{execution_id}/cancel
Cancel running execution.

### Plugin Management Endpoints

#### GET /plugins/marketplace
Browse plugin marketplace.

**Query Parameters:**
- `category`: Filter by category
- `search`: Search query
- `featured`: Show featured plugins only
- `sort`: Sort by (popularity, rating, name, updated_at)

**Response (200 OK):**
```json
{
  "plugins": [
    {
      "id": "client-management",
      "name": "Client Management",
      "description": "Comprehensive client relationship management",
      "version": "1.2.0",
      "category": "business",
      "price_cents": 2999,
      "currency": "USD",
      "billing_cycle": "monthly",
      "rating": 4.8,
      "downloads": 15420,
      "developer": {
        "name": "ADX Core Team",
        "verified": true
      },
      "screenshots": [
        "https://cdn.adxcore.com/plugins/client-mgmt/screenshot1.jpg"
      ],
      "features": [
        "Client portal builder",
        "Project tracking",
        "File sharing",
        "Custom branding"
      ],
      "is_installed": false,
      "compatible_versions": ["1.0.0", "1.1.0"]
    }
  ],
  "total": 1
}
```

#### GET /plugins/installed
List installed plugins.

#### POST /plugins/{plugin_id}/install
Install plugin from marketplace.

#### PUT /plugins/{plugin_id}/configure
Configure plugin settings.

#### POST /plugins/{plugin_id}/activate
Activate installed plugin.

#### POST /plugins/{plugin_id}/deactivate
Deactivate plugin.

#### DELETE /plugins/{plugin_id}
Uninstall plugin.

### License and Billing Endpoints

#### GET /license/current
Get current license information.

**Response (200 OK):**
```json
{
  "id": "bb0e8400-e29b-41d4-a716-446655440006",
  "tier": "professional",
  "status": "active",
  "features": {
    "max_users": 50,
    "storage_gb": 100,
    "ai_enhanced_workflows": true,
    "sso": true,
    "api_calls_per_month": 100000
  },
  "usage": {
    "users": 12,
    "storage_gb": 23.5,
    "api_calls_this_month": 15420
  },
  "billing": {
    "price_cents": 29900,
    "currency": "USD",
    "billing_cycle": "monthly",
    "next_billing_date": "2024-02-01T00:00:00Z"
  },
  "trial_ends_at": null,
  "expires_at": "2024-02-01T00:00:00Z"
}
```

#### POST /license/upgrade
Upgrade license tier.

#### GET /license/usage
Get detailed usage statistics.

#### GET /billing/invoices
List billing invoices.

#### GET /billing/payment-methods
List payment methods.

#### POST /billing/payment-methods
Add payment method.

### Analytics and Reporting Endpoints

#### GET /analytics/dashboard
Get dashboard analytics data.

#### GET /analytics/workflows
Get workflow execution analytics.

#### GET /analytics/files
Get file usage analytics.

#### GET /analytics/users
Get user activity analytics.

#### POST /reports/generate
Generate custom report.

### System and Health Endpoints

#### GET /health
System health check.

**Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "services": {
    "database": "healthy",
    "redis": "healthy",
    "temporal": "healthy",
    "file_storage": "healthy"
  },
  "metrics": {
    "response_time_ms": 45,
    "active_connections": 127,
    "memory_usage_percent": 68.5
  }
}
```

#### GET /version
Get API version information.

#### GET /metrics
Prometheus metrics endpoint.

## GraphQL API

### Schema Overview
```graphql
type Query {
  me: User
  tenant: Tenant
  files(filter: FileFilter, pagination: Pagination): FileConnection
  workflows(filter: WorkflowFilter): [Workflow]
  plugins: [Plugin]
}

type Mutation {
  updateProfile(input: UpdateProfileInput!): User
  uploadFile(input: FileUploadInput!): File
  executeWorkflow(input: ExecuteWorkflowInput!): WorkflowExecution
  installPlugin(pluginId: String!): Plugin
}

type Subscription {
  workflowExecutionUpdates(executionId: ID!): WorkflowExecution
  fileUploadProgress(uploadId: ID!): FileUploadProgress
  notifications: Notification
}
```

### Example Queries

#### Get User with Tenant Information
```graphql
query GetUserProfile {
  me {
    id
    name
    email
    preferences {
      language
      theme
      timezone
    }
    tenants {
      id
      name
      role
      permissions
    }
  }
}
```

#### Get Files with Metadata
```graphql
query GetFiles($filter: FileFilter, $pagination: Pagination) {
  files(filter: $filter, pagination: $pagination) {
    edges {
      node {
        id
        name
        sizeBytes
        mimeType
        owner {
          name
          email
        }
        tags
        createdAt
      }
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      startCursor
      endCursor
    }
    totalCount
  }
}
```

## WebSocket API

### Connection
```javascript
const ws = new WebSocket('wss://api.adxcore.com/ws');
ws.onopen = () => {
  // Send authentication
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'your-jwt-token'
  }));
};
```

### Message Types

#### Workflow Execution Updates
```json
{
  "type": "workflow_execution_update",
  "data": {
    "execution_id": "aa0e8400-e29b-41d4-a716-446655440005",
    "status": "completed",
    "progress": 100,
    "output": {
      "user_created": true,
      "welcome_email_sent": true
    },
    "completed_at": "2024-01-15T10:31:30Z"
  }
}
```

#### File Upload Progress
```json
{
  "type": "file_upload_progress",
  "data": {
    "upload_id": "cc0e8400-e29b-41d4-a716-446655440007",
    "progress": 75,
    "bytes_uploaded": 786432,
    "total_bytes": 1048576,
    "estimated_time_remaining": 5000
  }
}
```

#### Real-time Notifications
```json
{
  "type": "notification",
  "data": {
    "id": "dd0e8400-e29b-41d4-a716-446655440008",
    "title": "File Shared",
    "message": "John Doe shared 'document.pdf' with you",
    "type": "file_share",
    "priority": "normal",
    "created_at": "2024-01-15T10:30:00Z",
    "actions": [
      {
        "label": "View File",
        "url": "/files/880e8400-e29b-41d4-a716-446655440003"
      }
    ]
  }
}
```

## Webhook API

### Webhook Configuration
```json
{
  "url": "https://your-app.com/webhooks/adx-core",
  "events": [
    "user.created",
    "file.uploaded",
    "workflow.completed",
    "license.expired"
  ],
  "secret": "your-webhook-secret",
  "active": true
}
```

### Webhook Payload Example
```json
{
  "id": "ee0e8400-e29b-41d4-a716-446655440009",
  "event": "workflow.completed",
  "timestamp": "2024-01-15T10:31:30Z",
  "data": {
    "execution_id": "aa0e8400-e29b-41d4-a716-446655440005",
    "workflow_id": "990e8400-e29b-41d4-a716-446655440004",
    "status": "completed",
    "duration_ms": 90000,
    "ai_enhanced": true,
    "tenant_id": "660e8400-e29b-41d4-a716-446655440001"
  }
}
```

## Error Handling

### Standard Error Response
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "The request contains invalid data",
    "details": [
      {
        "field": "email",
        "message": "Invalid email format"
      }
    ],
    "request_id": "ff0e8400-e29b-41d4-a716-446655440010",
    "timestamp": "2024-01-15T10:30:00Z"
  }
}
```

### HTTP Status Codes
- `200 OK`: Successful request
- `201 Created`: Resource created successfully
- `202 Accepted`: Request accepted for processing
- `400 Bad Request`: Invalid request data
- `401 Unauthorized`: Authentication required
- `403 Forbidden`: Insufficient permissions
- `404 Not Found`: Resource not found
- `409 Conflict`: Resource conflict
- `422 Unprocessable Entity`: Validation errors
- `429 Too Many Requests`: Rate limit exceeded
- `500 Internal Server Error`: Server error
- `503 Service Unavailable`: Service temporarily unavailable

## Rate Limiting

### Rate Limit Headers
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 999
X-RateLimit-Reset: 1642248600
X-RateLimit-Window: 3600
```

### Rate Limit Tiers
- **Starter**: 1,000 requests/hour
- **Professional**: 10,000 requests/hour
- **Enterprise**: 100,000 requests/hour

## Authentication and Security

### JWT Token Structure
```json
{
  "header": {
    "alg": "HS256",
    "typ": "JWT"
  },
  "payload": {
    "sub": "550e8400-e29b-41d4-a716-446655440000",
    "tenant_id": "660e8400-e29b-41d4-a716-446655440001",
    "role": "owner",
    "permissions": ["*"],
    "iat": 1642248600,
    "exp": 1642252200,
    "iss": "adx-core",
    "aud": "adx-core-api"
  }
}
```

### API Key Authentication
For server-to-server communication:
```
Authorization: Bearer api_key_your_api_key_here
```

### Webhook Signature Verification
```javascript
const crypto = require('crypto');

function verifyWebhookSignature(payload, signature, secret) {
  const expectedSignature = crypto
    .createHmac('sha256', secret)
    .update(payload)
    .digest('hex');
  
  return crypto.timingSafeEqual(
    Buffer.from(signature, 'hex'),
    Buffer.from(expectedSignature, 'hex')
  );
}
```

This comprehensive API specification provides:

1. **Complete REST API** with all major endpoints
2. **GraphQL schema** for flexible queries
3. **WebSocket support** for real-time features
4. **Webhook system** for event notifications
5. **Comprehensive error handling** with detailed responses
6. **Security controls** with authentication and rate limiting
7. **Multiple protocol support** for various integration needs
8. **Detailed examples** for easy implementation