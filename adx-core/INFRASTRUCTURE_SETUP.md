# ADX CORE Infrastructure Setup

This document describes how to set up and run the complete ADX CORE development environment.

## Prerequisites

- Docker and Docker Compose
- Rust 1.88+ (for backend services)
- Node.js 18+ (for frontend services)

## Quick Start

### 1. Start Complete Environment

```bash
# Start all infrastructure services (PostgreSQL, Redis, Temporal, monitoring)
./scripts/dev-start-all.sh
```

This will start:
- PostgreSQL database (port 5432)
- Redis cache (port 6379)
- Temporal server (port 7233)
- Temporal UI (http://localhost:8088)
- Prometheus monitoring (http://localhost:9090)
- Grafana dashboards (http://localhost:3001, admin/admin)
- Jaeger tracing (http://localhost:16686)

### 2. Start Backend Services

After infrastructure is running, start the backend services:

```bash
# Start services in HTTP server mode
cargo run --bin auth-service server &
cargo run --bin user-service server &
cargo run --bin file-service server &
cargo run --bin tenant-service server &
cargo run --bin workflow-service server &
cargo run --bin api-gateway &

# Start services in Temporal worker mode (in separate terminals)
cargo run --bin auth-service worker &
cargo run --bin user-service worker &
cargo run --bin file-service worker &
cargo run --bin tenant-service worker &
cargo run --bin workflow-service worker &
```

### 3. Start Frontend Services

```bash
# Start Shell application (Module Federation host)
cd apps/shell && npm run dev &

# Start micro-frontends
cd apps/auth && npm run dev &
cd apps/tenant && npm run dev &
cd apps/user && npm run dev &
cd apps/file && npm run dev &
cd apps/workflow && npm run dev &
```

### 4. Stop Everything

```bash
# Stop all infrastructure services
./scripts/dev-stop-all.sh

# Optional: Clean up Docker resources
./scripts/dev-stop-all.sh --clean

# Optional: Reset all data (removes volumes)
./scripts/dev-stop-all.sh --reset
```

## Service Ports

### Infrastructure Services
- PostgreSQL: `localhost:5432`
- Redis: `localhost:6379`
- Temporal Server: `localhost:7233`
- Temporal UI: `http://localhost:8088`
- Prometheus: `http://localhost:9090`
- Grafana: `http://localhost:3001`
- Jaeger: `http://localhost:16686`

### Backend Services
- API Gateway: `localhost:8080`
- Auth Service: `localhost:8081`
- User Service: `localhost:8082`
- File Service: `localhost:8083`
- Workflow Service: `localhost:8084`
- Tenant Service: `localhost:8085`
- AI Service: `localhost:8086`
- White Label Service: `localhost:8087`

### Frontend Services
- Shell Application: `localhost:3000`
- Auth Micro-App: `localhost:3001`
- Tenant Micro-App: `localhost:3002`
- File Micro-App: `localhost:3003`
- User Micro-App: `localhost:3004`
- Workflow Micro-App: `localhost:3005`

### BFF Services
- Auth BFF: `localhost:4001`
- Tenant BFF: `localhost:4002`
- File BFF: `localhost:4003`
- User BFF: `localhost:4004`
- Workflow BFF: `localhost:4005`

## Environment Configuration

### Backend Services

Each service needs a `.env` file. Example for auth-service:

```env
# Database
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/adx_core

# Redis
REDIS_URL=redis://localhost:6379

# Temporal
TEMPORAL_SERVER_URL=localhost:7233
TEMPORAL_NAMESPACE=adx-core-development

# Server
SERVER_HOST=0.0.0.0
SERVER_PORT=8081

# JWT
JWT_SECRET=your-jwt-secret-key-here
JWT_EXPIRATION=24h

# Logging
RUST_LOG=info,auth_service=debug
```

### Frontend Services

Each micro-frontend needs environment configuration. Example for auth micro-app:

```env
# API Endpoints
VITE_API_GATEWAY_URL=http://localhost:8080
VITE_AUTH_BFF_URL=http://localhost:4001

# Module Federation
VITE_SHELL_URL=http://localhost:3000
```

## Database Setup

The infrastructure includes automatic database initialization with:
- Default tenant and demo tenant
- Admin user (admin@adxcore.local / admin123)
- Demo user (demo@adxcore.local / demo123)
- Basic schema structure

For production, run proper migrations:

```bash
# Run database migrations
cargo run --bin db-manager migrate

# Seed development data
cargo run --bin db-manager seed
```

## Temporal Configuration

The setup includes three Temporal namespaces:
- `adx-core-development` (72h retention)
- `adx-core-staging` (168h retention)
- `adx-core-production` (8760h retention)

Custom search attributes are configured for:
- TenantId, UserId, WorkflowType
- BusinessProcess, Priority, Environment
- Version, ModuleId, CorrelationId

## Monitoring and Observability

### Prometheus Metrics
- All services expose metrics at `/metrics`
- Prometheus scrapes every 5-15 seconds
- Custom ADX Core metrics included

### Grafana Dashboards
- Access: http://localhost:3001 (admin/admin)
- Pre-configured dashboards for all services
- Real-time monitoring of workflows and performance

### Jaeger Tracing
- Access: http://localhost:16686
- Distributed tracing across all services
- Temporal workflow tracing integration

### Temporal UI
- Access: http://localhost:8088
- Workflow execution monitoring
- Workflow history and replay
- Real-time workflow status

## Troubleshooting

### Common Issues

1. **Port conflicts**: Check if ports are already in use
2. **Docker not running**: Ensure Docker daemon is started
3. **Services not connecting**: Check if infrastructure is fully started
4. **Database connection errors**: Verify PostgreSQL is ready

### Debug Commands

```bash
# Check service status
docker-compose -f infrastructure/docker/docker-compose.dev.yml ps

# View service logs
docker-compose -f infrastructure/docker/docker-compose.dev.yml logs temporal
docker-compose -f infrastructure/docker/docker-compose.dev.yml logs postgres

# Test database connection
docker exec -it adx-postgres psql -U postgres -d adx_core

# Test Redis connection
docker exec -it adx-redis redis-cli ping

# Test Temporal connection
docker exec temporal tctl cluster health
```

### Reset Environment

```bash
# Complete reset (removes all data)
./scripts/dev-stop-all.sh --reset
./scripts/dev-start-all.sh
```

## Production Deployment

For production deployment, use:
- `infrastructure/docker/docker-compose.prod.yml`
- `infrastructure/kubernetes/` manifests
- Proper secrets management
- SSL/TLS certificates
- Load balancing and scaling

See `infrastructure/production/` for production-specific configurations.