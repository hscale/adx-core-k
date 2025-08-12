# Workflow Service

The Workflow Service is responsible for cross-service workflow orchestration in ADX Core. It coordinates complex operations that span multiple microservices using Temporal workflows.

## Overview

The Workflow Service implements the following core workflows:

1. **User Onboarding Workflow** - Coordinates Auth, User, Tenant, and File services for complete user setup
2. **Tenant Switching Workflow** - Handles multi-service context updates when users switch tenants
3. **Data Migration Workflow** - Manages cross-service data synchronization and migration
4. **Bulk Operation Workflow** - Executes administrative operations across multiple services
5. **Compliance Workflow** - Handles GDPR and audit requirements across all services

## Architecture

The service operates in dual-mode:

- **HTTP Server Mode** (Port 8084): Provides REST endpoints for workflow initiation and management
- **Temporal Worker Mode**: Executes Temporal workflows and activities

## Key Features

### Cross-Service Coordination
- Coordinates operations across Auth, User, Tenant, and File services
- Ensures data consistency and transaction-like behavior
- Provides automatic retry and compensation logic
- Maintains complete audit trails

### Workflow Management
- Start, monitor, cancel, and retry workflows
- Real-time progress tracking
- Comprehensive workflow history
- Error handling and rollback capabilities

### Service Health Coordination
- Cross-service health checks
- Backup and restore coordination
- Service dependency management

## API Endpoints

### Workflow Initiation
- `POST /api/v1/workflows/user-onboarding` - Start user onboarding workflow
- `POST /api/v1/workflows/tenant-switching` - Start tenant switching workflow
- `POST /api/v1/workflows/data-migration` - Start data migration workflow
- `POST /api/v1/workflows/bulk-operation` - Start bulk operation workflow
- `POST /api/v1/workflows/compliance` - Start compliance workflow

### Workflow Management
- `GET /api/v1/workflows/:workflow_id/status` - Get workflow status
- `POST /api/v1/workflows/:workflow_id/cancel` - Cancel workflow
- `POST /api/v1/workflows/:workflow_id/retry` - Retry failed workflow
- `GET /api/v1/workflows` - List workflows
- `GET /api/v1/workflows/history` - Get workflow history

### Service Coordination
- `POST /api/v1/coordination/health-check` - Coordinate service health check
- `POST /api/v1/coordination/backup` - Create cross-service backup
- `POST /api/v1/coordination/restore` - Restore from backup

## Configuration

The service uses the following configuration structure:

```toml
[server]
host = "0.0.0.0"
port = 8084
timeout_seconds = 30

[temporal]
server_url = "http://localhost:7233"
namespace = "default"
task_queue = "workflow-service-queue"
worker_identity = "workflow-service-worker"
max_concurrent_activities = 100
max_concurrent_workflows = 50

[services]
auth_service = "http://localhost:8081"
user_service = "http://localhost:8082"
tenant_service = "http://localhost:8085"
file_service = "http://localhost:8083"
api_gateway = "http://localhost:8080"

[workflows]
default_timeout = "300s"
batch_size = 100

[workflows.retry_policy]
initial_interval = "1s"
backoff_coefficient = 2.0
maximum_interval = "60s"
maximum_attempts = 3
```

## Usage

### Starting the Service

#### HTTP Server Mode
```bash
cargo run --bin workflow-service server
```

#### Temporal Worker Mode
```bash
cargo run --bin workflow-service worker
```

### Example Workflow Requests

#### User Onboarding
```bash
curl -X POST http://localhost:8084/api/v1/workflows/user-onboarding \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: tenant-123" \
  -d '{
    "user_email": "user@example.com",
    "user_name": "John Doe",
    "tenant_id": "tenant-123",
    "role": "user",
    "send_welcome_email": true,
    "setup_default_workspace": true,
    "assign_default_permissions": true,
    "create_sample_data": false
  }'
```

#### Tenant Switching
```bash
curl -X POST http://localhost:8084/api/v1/workflows/tenant-switching \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: tenant-456" \
  -d '{
    "user_id": "user-123",
    "current_tenant_id": "tenant-123",
    "target_tenant_id": "tenant-456",
    "preserve_session_data": true,
    "update_user_preferences": true
  }'
```

#### Data Migration
```bash
curl -X POST http://localhost:8084/api/v1/workflows/data-migration \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: tenant-123" \
  -d '{
    "migration_id": "migration-456",
    "target_tenant_id": "tenant-123",
    "migration_type": "CrossServiceSync",
    "data_selectors": [
      {
        "service": "user",
        "entity_type": "profile",
        "filters": {},
        "include_relationships": true
      }
    ],
    "migration_options": {
      "batch_size": 100,
      "parallel_workers": 4,
      "validate_data": true,
      "create_backup": true,
      "rollback_on_error": true,
      "dry_run": false
    }
  }'
```

#### Compliance Workflow (GDPR Data Export)
```bash
curl -X POST http://localhost:8084/api/v1/workflows/compliance \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: tenant-123" \
  -d '{
    "compliance_id": "gdpr-export-789",
    "tenant_id": "tenant-123",
    "compliance_type": "GdprDataExport",
    "subject_user_id": "user-123",
    "data_categories": ["personal", "usage"],
    "audit_requirements": {
      "include_access_logs": true,
      "include_modification_logs": true,
      "include_deletion_logs": false,
      "include_export_logs": true
    }
  }'
```

## Workflow Details

### User Onboarding Workflow
Coordinates the complete user setup process:
1. Validates tenant context
2. Creates user account in Auth Service
3. Creates user profile in User Service
4. Updates tenant membership
5. Sets up file workspace (optional)
6. Creates sample data (optional)
7. Sends welcome email (optional)

### Tenant Switching Workflow
Handles secure tenant context switching:
1. Validates user access to target tenant
2. Gets target tenant context
3. Updates user tenant context
4. Updates user session
5. Updates tenant membership

### Data Migration Workflow
Manages cross-service data operations:
1. Creates backup (optional)
2. Validates data (optional)
3. Processes migration for each service
4. Handles rollback on errors
5. Provides detailed migration results

### Bulk Operation Workflow
Executes operations across multiple entities:
1. Processes entities in configurable batches
2. Supports parallel processing
3. Continues on errors (configurable)
4. Provides detailed error reporting
5. Supports retry logic

### Compliance Workflow
Handles regulatory compliance requirements:
1. GDPR data export with encryption
2. GDPR data deletion with backup
3. Data retention enforcement
4. Audit log generation
5. Compliance reporting
6. Data classification

## Error Handling

The service provides comprehensive error handling:
- Automatic retry with exponential backoff
- Compensation logic for rollbacks
- Detailed error reporting
- Service-specific error categorization
- Cross-service error coordination

## Monitoring

The service exposes the following monitoring endpoints:
- `/health` - Basic health check
- `/ready` - Readiness check with dependency validation

Metrics are exposed for:
- Workflow execution times
- Success/failure rates
- Service communication latency
- Error rates by type

## Development

### Running Tests
```bash
cargo test
```

### Building
```bash
cargo build --release
```

### Docker
```bash
docker build -t workflow-service .
docker run -p 8084:8084 workflow-service server
```

## Dependencies

The service depends on:
- Temporal server for workflow orchestration
- Auth Service for user authentication
- User Service for user management
- Tenant Service for tenant operations
- File Service for file operations
- Redis for caching (optional)
- PostgreSQL for persistence (optional)

## Security

- All cross-service communication uses HTTPS
- Tenant isolation enforced at all levels
- Audit logging for all operations
- Input validation and sanitization
- Rate limiting and timeout protection