# File BFF Service

The File Backend-for-Frontend (BFF) service for ADX Core, implemented in Rust with Axum. This service acts as a Temporal workflow client and provides optimized APIs for file management micro-frontends.

## Features

- **Temporal Workflow Integration**: Acts as a Temporal client for file workflow coordination
- **Redis Caching**: Implements comprehensive caching for file metadata, permissions, and search results
- **Aggregated Endpoints**: Combines file data, permissions, and storage info in single requests
- **Upload Progress Tracking**: Real-time file upload progress monitoring
- **File Search Optimization**: Advanced file search with caching and filtering
- **Multi-tenant Support**: Complete tenant isolation and context management
- **Authentication & Authorization**: JWT-based auth with role-based permissions
- **Performance Optimized**: Request batching, response shaping, and intelligent caching

## Architecture

### Service Components

```
File BFF Service (Port 4003)
├── Routes
│   ├── /api/files          # File management endpoints
│   ├── /api/workflows      # Workflow initiation and status
│   └── /api/aggregated     # Combined data endpoints
├── Services
│   ├── ApiClient          # File Service & API Gateway communication
│   ├── RedisService       # Caching and session management
│   └── TemporalClient     # Workflow coordination (placeholder)
└── Middleware
    ├── Authentication     # JWT token validation
    ├── Tenant Context     # Multi-tenant isolation
    └── Error Handling     # Standardized error responses
```

### Workflow Integration

The File BFF service initiates and coordinates file-related Temporal workflows:

- **File Upload Workflow**: Handles file uploads with virus scanning and processing
- **File Processing Workflow**: Thumbnail generation, metadata extraction, OCR
- **File Migration Workflow**: Storage provider migrations with rollback
- **Bulk File Operations**: Batch operations on multiple files
- **File Cleanup Workflow**: Automated lifecycle management and archival

## API Endpoints

### File Management

```http
GET    /api/files                    # List files with caching
POST   /api/files/search             # Advanced file search
GET    /api/files/:id                # Get file metadata (cached)
PUT    /api/files/:id                # Update file (workflow)
DELETE /api/files/:id                # Delete file (workflow)
POST   /api/files/upload             # Initiate upload workflow
```

### File Permissions

```http
GET    /api/files/:id/permissions    # Get file permissions (cached)
PUT    /api/files/:id/permissions    # Update permissions (workflow)
POST   /api/files/:id/share          # Share file (workflow)
```

### Upload Management

```http
GET    /api/files/uploads/:id/progress  # Get upload progress
POST   /api/files/uploads/:id/cancel    # Cancel upload (workflow)
```

### Workflow Operations

```http
POST   /api/workflows/file-upload           # File upload workflow
POST   /api/workflows/file-processing       # File processing workflow
POST   /api/workflows/file-migration        # File migration workflow
POST   /api/workflows/bulk-file-operation   # Bulk operations workflow
POST   /api/workflows/file-cleanup          # File cleanup workflow
GET    /api/workflows/:id/status            # Get workflow status
POST   /api/workflows/:id/cancel            # Cancel workflow
GET    /api/workflows/:id/stream            # Stream workflow progress
```

### Aggregated Data

```http
GET    /api/aggregated/file/:id         # Complete file data
GET    /api/aggregated/files            # Enhanced file list
GET    /api/aggregated/dashboard        # File dashboard data
GET    /api/aggregated/storage-summary  # Storage analytics
GET    /api/aggregated/recent-activity  # Recent file activity
GET    /api/aggregated/upload-status    # Upload status summary
```

## Configuration

### Environment Variables

```bash
# Server Configuration
PORT=4003
HOST=0.0.0.0

# Service URLs
API_GATEWAY_URL=http://localhost:8080
FILE_SERVICE_URL=http://localhost:8083

# Redis Configuration
REDIS_URL=redis://localhost:6379

# Temporal Configuration
TEMPORAL_SERVER_URL=localhost:7233
TEMPORAL_NAMESPACE=default

# Authentication
JWT_SECRET=your-secret-key

# Caching TTL (seconds)
FILE_METADATA_CACHE_TTL=600
PERMISSIONS_CACHE_TTL=300
SEARCH_CACHE_TTL=300
UPLOAD_PROGRESS_CACHE_TTL=30
```

### Cache Strategy

The service implements intelligent caching with different TTL values:

- **File Metadata**: 10 minutes (600s) - relatively stable
- **File Permissions**: 5 minutes (300s) - changes moderately
- **Search Results**: 5 minutes (300s) - can be cached safely
- **Upload Progress**: 30 seconds - changes frequently
- **Workflow Status**: 5 minutes for completed, 30s for active

## Development

### Prerequisites

- Rust 1.70+
- Redis server
- Access to ADX Core API Gateway and File Service
- Temporal server (for workflow integration)

### Setup

1. **Clone and navigate to the service directory**:
   ```bash
   cd bff-services/file-bff
   ```

2. **Copy environment configuration**:
   ```bash
   cp .env.example .env
   # Edit .env with your configuration
   ```

3. **Install dependencies and build**:
   ```bash
   cargo build
   ```

4. **Run the service**:
   ```bash
   cargo run
   ```

5. **Run tests**:
   ```bash
   cargo test
   ```

### Development Commands

```bash
# Development with auto-reload
cargo watch -x run

# Run with debug logging
RUST_LOG=file_bff=debug cargo run

# Run specific tests
cargo test test_name

# Check code formatting
cargo fmt --check

# Run clippy for linting
cargo clippy -- -D warnings

# Build for release
cargo build --release
```

## Caching Strategy

### Cache Keys

The service uses structured cache keys for efficient data retrieval:

```
file:metadata:{tenant_id}:{file_id}
file:permissions:{tenant_id}:{file_id}
file:storage:{tenant_id}:{file_id}
file:search:{tenant_id}:{search_hash}
upload:progress:{tenant_id}:{upload_id}
workflow:status:{tenant_id}:{operation_id}
aggregated:file:{tenant_id}:{file_id}
dashboard:files:{tenant_id}
```

### Cache Invalidation

- **File Updates**: Invalidates all file-related cache entries
- **Permission Changes**: Invalidates permission and aggregated caches
- **Tenant Changes**: Bulk invalidation of tenant-specific caches
- **Upload Completion**: Removes upload progress cache entries

## Performance Optimizations

### Request Batching

The service batches multiple API calls when fetching aggregated data:

```rust
// Parallel data fetching
let (metadata, permissions, storage) = 
    futures::future::try_join3(
        get_metadata_future,
        get_permissions_future,
        get_storage_future
    ).await?;
```

### Response Shaping

Aggregated endpoints combine multiple data sources into optimized responses:

- File metadata + permissions + storage info
- Dashboard data with analytics and summaries
- Enhanced file lists with optional additional data

### Intelligent Caching

- **Cache-first strategy** for stable data (metadata, permissions)
- **Short TTL** for frequently changing data (upload progress)
- **Cache warming** for dashboard and summary data
- **Conditional caching** based on data volatility

## Error Handling

The service implements comprehensive error handling:

```rust
#[derive(Debug, thiserror::Error)]
pub enum BffError {
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    #[error("Tenant validation failed: {0}")]
    TenantValidation(String),
    
    #[error("API client error: {0}")]
    ApiClient(#[from] anyhow::Error),
    
    // ... other error types
}
```

### Error Response Format

```json
{
  "error": "VALIDATION_ERROR",
  "message": "File name cannot be empty",
  "details": null
}
```

## Security

### Authentication

- **JWT Token Validation**: All requests require valid JWT tokens
- **Permission Checking**: Role-based and permission-based authorization
- **Tenant Isolation**: Strict tenant context validation

### Authorization Levels

- **Admin**: Full access to all file operations and workflows
- **User**: Standard file operations (read, write, share)
- **Viewer**: Read-only access to files

### Security Headers

The service implements security best practices:

- CORS configuration for cross-origin requests
- Request timeout and rate limiting
- Input validation and sanitization
- Secure error messages (no sensitive data leakage)

## Monitoring and Observability

### Logging

Structured logging with different levels:

```rust
tracing::info!("File uploaded successfully: {}", file_id);
tracing::warn!("Cache miss for file metadata: {}", file_id);
tracing::error!("Failed to initiate workflow: {}", error);
```

### Metrics

Key metrics to monitor:

- Request latency and throughput
- Cache hit/miss ratios
- Workflow initiation success rates
- Error rates by endpoint
- Redis connection health

### Health Checks

```http
GET /health
```

Returns service health status including:
- Service availability
- Redis connectivity
- API Gateway connectivity
- File Service connectivity

## Deployment

### Docker

```dockerfile
FROM rust:1.70 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates
COPY --from=builder /app/target/release/file-bff /usr/local/bin/
EXPOSE 4003
CMD ["file-bff"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: file-bff
spec:
  replicas: 3
  selector:
    matchLabels:
      app: file-bff
  template:
    metadata:
      labels:
        app: file-bff
    spec:
      containers:
      - name: file-bff
        image: adx-core/file-bff:latest
        ports:
        - containerPort: 4003
        env:
        - name: REDIS_URL
          value: "redis://redis-service:6379"
        - name: API_GATEWAY_URL
          value: "http://api-gateway-service:8080"
```

## Contributing

1. Follow Rust coding standards and use `cargo fmt`
2. Add tests for new functionality
3. Update documentation for API changes
4. Ensure all tests pass before submitting PRs
5. Follow the existing error handling patterns
6. Add appropriate logging and metrics

## License

This service is part of the ADX Core platform and follows the same licensing terms.