# ADX CORE

A modern, scalable platform built with a **Temporal-first architecture** that prioritizes reliability, observability, and maintainability for complex business operations.

## 🏗️ Architecture Overview

ADX CORE follows a **Temporal-first** approach where Temporal workflows are the PRIMARY mechanism for implementing all multi-step business operations. This provides:

- **Reliability**: Automatic retry, timeout, and error handling for distributed operations
- **Observability**: Complete visibility into business process execution through Temporal UI
- **Maintainability**: Clear separation between business logic (workflows) and infrastructure concerns
- **Scalability**: Horizontal scaling of workflow workers independent of HTTP services

### Core Architecture Patterns

- **Workflow-Driven Microservices**: Services provide both direct endpoints (simple operations) and Temporal activities (complex workflows)
- **Frontend Microservices**: Domain-aligned micro-frontends with Module Federation
- **Multi-tenant**: Tenant isolation at database, application, and workflow levels
- **BFF Pattern (Optional)**: Backend for Frontend services act as Temporal clients for data aggregation and caching

## 🛠️ Technology Stack

### Backend (Temporal-First)
- **Language**: Rust 1.88+ with async/await
- **Framework**: Axum for HTTP services
- **Workflow Engine**: Temporal.io for ALL multi-step operations
- **Database**: PostgreSQL (primary) + Redis (caching/sessions)
- **Authentication**: JWT tokens with bcrypt password hashing

### Frontend (Micro-Frontend Architecture)
- **Shell Application**: React 18+ with TypeScript, Vite Module Federation
- **Micro-Frontends**: Domain-specific apps (Auth, Tenant, File, User, Workflow)
- **Cross-Platform**: Tauri 2.0 for native desktop and mobile
- **Styling**: TailwindCSS with shared design system
- **State Management**: Zustand, React Query (@tanstack/react-query)

## 🚀 Quick Start

### Prerequisites
- Rust 1.88+
- Node.js 18+
- Docker & Docker Compose
- PostgreSQL 14+
- Redis 6+

### Development Setup

1. **Clone and setup the project**:
```bash
git clone <repository-url>
cd adx-core
```

2. **Start the entire development environment**:
```bash
./scripts/dev-start.sh
```

3. **Or start infrastructure only**:
```bash
docker compose -f infrastructure/docker/docker-compose.dev.yml up -d
```

### Service Architecture

#### Backend Services (Temporal-Enabled)
Each service runs in dual-mode: HTTP server + Temporal worker

```bash
# HTTP server mode (direct endpoints)
cargo run --bin auth-service

# Workflow worker mode (Temporal activities)
cargo run --bin auth-service -- --mode worker
```

#### Service Ports

**Backend Services**:
- API Gateway: http://localhost:8080
- Auth Service: http://localhost:8081
- User Service: http://localhost:8082
- File Service: http://localhost:8083
- Workflow Service: http://localhost:8084
- Tenant Service: http://localhost:8085
- Module Service: http://localhost:8086

**Temporal Infrastructure**:
- Temporal Server: http://localhost:7233
- Temporal UI: http://localhost:8088

**Frontend Micro-Services**:
- Shell Application: http://localhost:3000
- Auth Micro-App: http://localhost:3001
- Tenant Micro-App: http://localhost:3002
- File Micro-App: http://localhost:3003
- User Micro-App: http://localhost:3004
- Workflow Micro-App: http://localhost:3005

## 🔄 Workflow-First Development

### When to Use Workflows vs Direct Endpoints

**Use Temporal Workflows for**:
- Operations involving multiple microservices
- Long-running operations (>5 seconds expected duration)
- Operations requiring rollback/compensation logic
- Complex business processes with multiple steps
- Operations that need progress tracking

**Use Direct Endpoints for**:
- Single-service CRUD operations
- Simple data retrieval
- Operations that complete in <1 second
- Health checks and status endpoints

### Example Workflow Implementation

```rust
#[workflow]
pub async fn business_process_workflow(
    request: BusinessProcessRequest,
    context: WorkflowContext,
) -> Result<BusinessProcessResult, WorkflowError> {
    // Step 1: Validate preconditions
    let validation = call_activity(
        ServiceActivities::validate_request,
        ValidationRequest {
            request: request.clone(),
            user_context: context.user_context.clone(),
            tenant_context: context.tenant_context.clone(),
        },
    ).await?;
    
    // Step 2: Execute main business logic with compensation tracking
    let entity = call_activity(
        ServiceActivities::create_entity,
        CreateEntityRequest::from(request.clone()),
    ).await?;
    
    // Step 3: Execute dependent operations
    let related_data = call_activity(
        RelatedServiceActivities::create_related_data,
        CreateRelatedDataRequest {
            entity_id: entity.id.clone(),
            data: request.related_data,
        },
    ).await?;
    
    Ok(BusinessProcessResult {
        entity_id: entity.id,
        related_data_id: related_data.id,
        completed_at: Utc::now(),
    })
}
```

## 🧪 Development Commands

### Backend Development
```bash
# Build all services
cargo build --workspace

# Run tests
cargo test --workspace
cargo test --test workflow_tests
cargo test --test integration_workflows

# Test specific workflows
cargo test tenant_switch_workflow
cargo test file_upload_workflow

# Code quality
cargo clippy --workspace
cargo fmt --all
```

### Frontend Development
```bash
# Start all micro-frontends
npm run dev:all

# Start individual micro-frontends
npm run dev:shell        # Shell application
npm run dev:auth         # Auth micro-frontend
npm run dev:tenant       # Tenant micro-frontend

# Building
npm run build:all        # All micro-frontends
npm run build:web        # Web builds
npm run build:desktop    # Desktop builds
npm run build:mobile     # Mobile builds

# Testing
npm run test:unit        # Unit tests
npm run test:integration # Integration tests
npm run test:e2e         # End-to-end tests
```

## 📊 Monitoring & Observability

### Temporal UI
Access the Temporal UI at http://localhost:8088 to monitor:
- Workflow executions and their status
- Activity retries and failures
- Workflow history and timeline
- Performance metrics

### Metrics & Logging
- **Structured Logging**: serde_json with workflow context
- **Metrics**: Prometheus metrics for workflows and activities
- **Tracing**: OpenTelemetry for distributed tracing
- **Workflow Metrics**: Built-in Temporal metrics and custom business metrics

## 🏢 Multi-Tenant Architecture

ADX CORE provides complete tenant isolation at multiple levels:
- **Database Level**: Tenant-scoped data access
- **Application Level**: Tenant context propagation through workflows
- **Workflow Level**: Tenant-aware workflow execution
- **Frontend Level**: Tenant-specific UI and data

## 🔧 Service Development Pattern

Each service implements a dual-mode pattern:

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("server");
    
    match mode {
        "server" => start_http_server().await,
        "worker" => start_workflow_worker().await,
        _ => {
            eprintln!("Usage: {} [server|worker]", args[0]);
            std::process::exit(1);
        }
    }
}
```

## 📚 Documentation

- [Architecture Decision Records](./docs/adr/)
- [API Documentation](./docs/api/)
- [Workflow Patterns](./docs/workflows/)
- [Frontend Architecture](./docs/frontend/)
- [Deployment Guide](./docs/deployment/)

## 🤝 Contributing

1. Follow the Temporal-first architecture principles
2. Implement complex operations as workflows
3. Maintain dual-mode service pattern
4. Write comprehensive workflow tests
5. Update documentation for new workflows

## 📄 License

[License information]

---

Built with ❤️ using Temporal.io, Rust, and React