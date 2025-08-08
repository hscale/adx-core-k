# ADX CORE - Temporal-First Microservices Platform

ADX CORE is a temporal-first, multi-tenant SaaS platform built with microservices architecture for both backend and frontend. The platform uses Temporal.io workflows as the PRIMARY orchestration mechanism, with domain-aligned microservices, Module Federation-based frontend microservices, and optional Backend-for-Frontend (BFF) services.

## Architecture Overview

### Core Principles
1. **"If it's more complex than a simple CRUD operation, it MUST be a Temporal workflow."**
2. **"Frontend micro-apps mirror backend service boundaries for team autonomy."**
3. **"Services communicate through Temporal workflows, never direct calls."**
4. **"Each team owns a complete vertical slice: backend service + micro-frontend + optional BFF."**

### Backend Services (Temporal-First)
- **API Gateway** (Port 8080): Workflow orchestration + Direct endpoints + Rate limiting
- **Auth Service** (Port 8081): Activities + Direct endpoints + Workflow worker + SSO/MFA
- **User Service** (Port 8082): Activities + Direct endpoints + Workflow worker + Multi-tenant
- **File Service** (Port 8083): Activities + Direct endpoints + Workflow worker + Storage backends
- **Workflow Service** (Port 8084): Cross-service workflow orchestration + AI integration
- **Tenant Service** (Port 8085): Activities + Direct endpoints + Workflow worker + Isolation

### Infrastructure
- **Temporal Server**: http://localhost:7233
- **Temporal UI**: http://localhost:8088
- **PostgreSQL**: Primary database with per-service schemas
- **Redis**: Caching layer for BFF services, sessions, and rate limiting

## Project Structure

```
adx-core/
├── Cargo.toml                 # Workspace configuration
├── README.md                  # This file
├── services/                  # Backend microservices
│   ├── shared/               # Common utilities and Temporal abstractions
│   │   ├── src/
│   │   │   ├── lib.rs        # Module exports
│   │   │   ├── config.rs     # Configuration management
│   │   │   ├── database.rs   # Database abstractions
│   │   │   ├── temporal.rs   # Temporal client and utilities
│   │   │   ├── types.rs      # Common types and structures
│   │   │   ├── auth.rs       # Authentication utilities
│   │   │   ├── logging.rs    # Structured logging setup
│   │   │   ├── health.rs     # Health check utilities
│   │   │   └── error.rs      # Error handling
│   │   └── Cargo.toml
│   ├── auth-service/         # Authentication and authorization
│   ├── user-service/         # User management
│   ├── file-service/         # File storage and management
│   ├── tenant-service/       # Multi-tenant management
│   └── workflow-service/     # Cross-service workflow orchestration
├── infrastructure/           # Infrastructure and deployment
│   └── docker/              # Docker configurations
│       ├── docker-compose.dev.yml  # Development environment
│       ├── init-db.sql             # Database initialization
│       ├── prometheus.yml          # Metrics configuration
│       └── temporal-config/        # Temporal server configuration
└── scripts/                 # Development and deployment scripts
    ├── README.md            # Scripts documentation
    ├── dev-start.sh         # Start development environment
    ├── dev-stop.sh          # Stop development environment
    ├── build.sh             # Build all services
    ├── test.sh              # Run test suite
    └── deploy.sh            # Deployment script
```

## Getting Started

### Prerequisites
- Docker and Docker Compose
- Rust toolchain (latest stable)
- Git

### Quick Start

1. **Start the development environment:**
   ```bash
   ./scripts/dev-start.sh
   ```

2. **Build all services:**
   ```bash
   ./scripts/build.sh
   ```

3. **Run tests:**
   ```bash
   ./scripts/test.sh
   ```

4. **Start individual services:**
   ```bash
   # HTTP server mode
   cargo run --bin auth-service
   cargo run --bin user-service
   
   # Temporal worker mode
   cargo run --bin auth-service -- --mode worker
   cargo run --bin user-service -- --mode worker
   ```

### Service Architecture

Each backend service follows a **dual-mode pattern**:

- **HTTP Server Mode**: Handles direct endpoints for simple CRUD operations
- **Temporal Worker Mode**: Executes Temporal activities and workflows for complex operations

### Development Workflow

1. **Daily Development:**
   ```bash
   ./scripts/dev-start.sh    # Start infrastructure
   cargo run --bin <service> # Run specific service
   ./scripts/test.sh         # Run tests
   ```

2. **Before Committing:**
   ```bash
   ./scripts/test.sh --coverage  # Full test suite with coverage
   ./scripts/build.sh --clean    # Clean build
   ```

3. **Deployment:**
   ```bash
   ./scripts/deploy.sh --env development  # Deploy to dev
   ./scripts/deploy.sh --env staging      # Deploy to staging
   ```

## Service Communication

### Temporal-First Communication
- **Simple Operations**: Direct HTTP calls between services
- **Complex Operations**: Temporal workflows coordinate multiple services
- **Cross-Service Operations**: Always use Temporal workflows, never direct calls

### Example Workflow
```rust
#[workflow]
pub async fn user_onboarding_workflow(request: UserOnboardingRequest) -> Result<OnboardingResult, WorkflowError> {
    // Step 1: Create user account (Auth Service activity)
    let user = call_activity(AuthServiceActivities::create_user, request.clone()).await?;
    
    // Step 2: Create default tenant (Tenant Service activity)
    let tenant = call_activity(TenantServiceActivities::create_default_tenant, user.id).await?;
    
    // Step 3: Setup file storage (File Service activity)
    let storage = call_activity(FileServiceActivities::setup_user_storage, user.id).await?;
    
    Ok(OnboardingResult { user, tenant, storage })
}
```

## Multi-Tenancy

ADX CORE implements comprehensive multi-tenancy with complete isolation:

- **Database Level**: Separate schemas or databases per tenant
- **Application Level**: Tenant context propagation through all layers
- **Workflow Level**: Tenant-aware workflow execution and isolation

## Testing Strategy

### Test Types
- **Unit Tests**: Individual component testing with mocks
- **Integration Tests**: Cross-service integration with test containers
- **Workflow Tests**: Temporal workflow testing with replay capabilities
- **End-to-End Tests**: Complete user journey testing

### Running Tests
```bash
./scripts/test.sh --unit         # Unit tests only
./scripts/test.sh --integration  # Integration tests only
./scripts/test.sh --workflow     # Workflow tests only
./scripts/test.sh --coverage     # All tests with coverage
```

## Monitoring and Observability

### Built-in Observability
- **Structured Logging**: JSON-formatted logs with correlation IDs
- **Distributed Tracing**: OpenTelemetry integration
- **Metrics**: Prometheus metrics for all services
- **Workflow Monitoring**: Complete visibility through Temporal UI

### Accessing Monitoring
- **Temporal UI**: http://localhost:8088
- **Service Logs**: `docker-compose logs <service-name>`
- **Metrics**: Prometheus endpoint on each service

## Configuration

### Environment Variables
Each service uses environment-specific configuration:

```bash
# Database
DATABASE_URL=postgres://postgres:postgres@localhost:5432/adx_core
REDIS_URL=redis://localhost:6379

# Temporal
TEMPORAL_SERVER_URL=localhost:7233

# Service-specific
AUTH_SERVICE_PORT=8081
USER_SERVICE_PORT=8082
# ... etc
```

### Configuration Files
- Service configuration in `services/<service>/config/`
- Infrastructure configuration in `infrastructure/docker/`

## Contributing

### Development Guidelines
1. **Temporal-First**: Implement complex operations as Temporal workflows
2. **Multi-Tenant Aware**: Always validate and propagate tenant context
3. **Error Handling**: Use structured error types with proper context
4. **Testing**: Write comprehensive tests for all functionality
5. **Documentation**: Update documentation for any API changes

### Code Style
- Follow Rust standard formatting (`cargo fmt`)
- Use Clippy for linting (`cargo clippy`)
- Write descriptive commit messages
- Include tests for new functionality

## Troubleshooting

### Common Issues

1. **Services won't start:**
   ```bash
   ./scripts/dev-stop.sh --clean
   ./scripts/dev-start.sh
   ```

2. **Database connection issues:**
   ```bash
   docker-compose -f infrastructure/docker/docker-compose.dev.yml restart postgres
   ```

3. **Temporal workflows not executing:**
   - Check Temporal UI at http://localhost:8088
   - Verify worker services are running
   - Check service logs for errors

### Getting Help
- Check service logs: `docker-compose logs <service-name>`
- Review Temporal UI for workflow execution details
- Run tests to identify issues: `./scripts/test.sh --verbose`

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support and questions:
- Create an issue in the repository
- Check the documentation in each service directory
- Review the Temporal documentation for workflow-related questions