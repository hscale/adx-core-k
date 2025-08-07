# ADX CORE - DevOps & Deployment Strategy

## Overview

ADX CORE deployment strategy designed for **AI Coder Teams** (Kiro/Claude/Gemini/GitHub Copilot) with simple DevOps, 2-week sprints, and multi-cloud marketplace distribution.

## Deployment Targets

### 1. Development Environment (macOS + Docker Desktop)
- **Local development** with Docker Compose
- **Temporal.io cluster** for workflow development
- **Hot reload** for rapid development
- **Basic logging** for debugging

### 2. VPS Deployment (Initial Production)
- **Single server** Docker Compose setup
- **Simple monitoring** with basic logs
- **Automated backups** for data protection
- **Basic CI/CD** with GitHub Actions

### 3. Cloud Marketplace Distribution
- **AWS Marketplace** - ECS/EKS deployment
- **Google Cloud Marketplace** - GKE deployment  
- **Azure Marketplace** - AKS deployment
- **One-click deployment** for customers

## Development Environment Setup (IDE-Optimized)

### Prerequisites
- **macOS** with Docker Desktop installed
- **Rust 1.70+** installed via rustup
- **Node.js 18+** and npm/yarn for frontend
- **Git** account configured
- **IDE**: VS Code, RustRover, or similar with Rust/TypeScript support
- **8GB+ RAM**, 50GB+ disk space

### Hybrid Development Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    macOS Development                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  Rust Backend   │  │   Frontend      │  │      IDE        │  │
│  │  (Native macOS) │  │  (Native macOS) │  │   Debugging     │  │
│  │                 │  │                 │  │                 │  │
│  │ • cargo run     │  │ • npm run dev   │  │ • Breakpoints   │  │
│  │ • IDE debugging │  │ • Hot reload    │  │ • Step through │  │
│  │ • Fast compile  │  │ • React DevTools│  │ • Variable inspect│  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│                                │                               │
│  ┌─────────────────────────────┼─────────────────────────────┐  │
│  │            Docker Services (Infrastructure Only)         │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐    │  │
│  │  │  Temporal   │  │ PostgreSQL  │  │     Redis       │    │  │
│  │  │   Cluster   │  │  Database   │  │     Cache       │    │  │
│  │  │             │  │             │  │                 │    │  │
│  │  │ Port: 7233  │  │ Port: 5432  │  │   Port: 6379    │    │  │
│  │  │ UI: 8233    │  │             │  │                 │    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘    │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Docker Compose (Infrastructure Only)
```yaml
# docker-compose.dev.yml - Infrastructure services only
version: '3.8'
services:
  # Temporal.io Cluster
  temporal:
    image: temporalio/auto-setup:1.22
    ports:
      - "7233:7233"    # Temporal gRPC
      - "8233:8233"    # Temporal Web UI
    environment:
      - DB=postgresql
      - DB_PORT=5432
      - POSTGRES_USER=temporal
      - POSTGRES_PWD=temporal
      - POSTGRES_SEEDS=postgres
    depends_on:
      - postgres
    volumes:
      - ./temporal:/etc/temporal/config/dynamicconfig

  # PostgreSQL Database
  postgres:
    image: postgres:15
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: adx_core
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init-db.sql

  # Redis Cache
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

volumes:
  postgres_data:
  redis_data:
```

### Development Commands (IDE-Optimized)
```makefile
# Makefile for AI Coder Teams - IDE-friendly development
.PHONY: infra dev-backend dev-frontend test clean build deploy

# Start infrastructure services only (Docker)
infra:
	docker-compose -f docker-compose.dev.yml up -d
	@echo "🚀 Infrastructure services started"
	@echo "📊 Temporal UI: http://localhost:8233"
	@echo "🗄️  PostgreSQL: localhost:5432"
	@echo "🔴 Redis: localhost:6379"
	@echo ""
	@echo "✅ Ready for native Rust/Frontend development!"
	@echo "💡 Run 'make dev-backend' and 'make dev-frontend' in separate terminals"

# Start Rust backend (native macOS for IDE debugging)
dev-backend:
	@echo "🦀 Starting Rust backend (native macOS)"
	@echo "🔧 IDE debugging enabled - set breakpoints in your IDE"
	cd backend && cargo run --bin api-gateway

# Start frontend (native macOS for hot reload)
dev-frontend:
	@echo "⚛️  Starting React frontend (native macOS)"
	@echo "🔥 Hot reload enabled - changes reflect immediately"
	cd frontend && npm run dev

# Run all tests (native for better IDE integration)
test:
	@echo "🧪 Running Rust tests (native)"
	cd backend && cargo test --all
	@echo "🧪 Running Frontend tests (native)"
	cd frontend && npm test
	@echo "✅ All tests completed"

# Run specific test with debugging
test-debug:
	@echo "🐛 Running tests with debugging enabled"
	cd backend && cargo test --all -- --nocapture

# Clean development environment
clean:
	docker-compose -f docker-compose.dev.yml down -v
	docker system prune -f
	@echo "🧹 Infrastructure cleaned"
	@echo "💡 Your native code remains untouched"

# Build production images
build:
	docker-compose -f docker-compose.prod.yml build
	@echo "🏗️  Production images built"

# Deploy to VPS
deploy:
	./scripts/deploy-vps.sh
	@echo "🚀 Deployed to VPS"

# Full development setup (infrastructure + instructions)
dev:
	make infra
	@echo ""
	@echo "🎯 Next steps for AI Coder Teams:"
	@echo "1. Open your IDE (VS Code, RustRover, etc.)"
	@echo "2. Run 'make dev-backend' in terminal 1"
	@echo "3. Run 'make dev-frontend' in terminal 2"
	@echo "4. Set breakpoints and debug natively!"
```

## VPS Production Deployment

### Server Requirements
- **CPU**: 4 cores minimum (8 cores recommended)
- **RAM**: 16GB minimum (32GB recommended)
- **Storage**: 100GB SSD minimum (500GB recommended)
- **Network**: 1Gbps connection
- **OS**: Ubuntu 22.04 LTS

### Production Docker Compose
```yaml
# docker-compose.prod.yml
version: '3.8'
services:
  # Nginx Reverse Proxy
  nginx:
    image: nginx:alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./nginx/nginx.conf:/etc/nginx/nginx.conf
      - ./nginx/ssl:/etc/nginx/ssl
    depends_on:
      - api-gateway
    restart: unless-stopped

  # Temporal.io Production Cluster
  temporal:
    image: temporalio/server:1.22
    ports:
      - "7233:7233"
    environment:
      - DB=postgresql
      - POSTGRES_SEEDS=postgres
      - DYNAMIC_CONFIG_FILE_PATH=/etc/temporal/config/dynamicconfig/development.yaml
    volumes:
      - ./temporal/config:/etc/temporal/config
    depends_on:
      - postgres
    restart: unless-stopped

  # Production PostgreSQL
  postgres:
    image: postgres:15
    environment:
      POSTGRES_DB: adx_core
      POSTGRES_USER: ${POSTGRES_USER}
      POSTGRES_PASSWORD: ${POSTGRES_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./backups:/backups
    restart: unless-stopped

  # Production Redis
  redis:
    image: redis:7-alpine
    command: redis-server --appendonly yes --requirepass ${REDIS_PASSWORD}
    volumes:
      - redis_data:/data
    restart: unless-stopped

  # ADX Core Services
  api-gateway:
    image: adx-core/api-gateway:latest
    environment:
      - DATABASE_URL=postgresql://${POSTGRES_USER}:${POSTGRES_PASSWORD}@postgres:5432/adx_core
      - REDIS_URL=redis://:${REDIS_PASSWORD}@redis:6379
      - TEMPORAL_URL=temporal:7233
      - RUST_LOG=info
    depends_on:
      - postgres
      - redis
      - temporal
    restart: unless-stopped

volumes:
  postgres_data:
  redis_data:
```

## CI/CD Pipeline

### GitHub Actions Workflow
```yaml
# .github/workflows/ci-cd.yml
name: ADX CORE CI/CD

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run tests
        run: cargo test --all
      - name: Run Temporal workflow tests
        run: cargo test --features temporal-testing

  build:
    needs: test
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker images
        run: |
          docker build -t adx-core/api-gateway:${{ github.sha }} ./services/api-gateway
          docker build -t adx-core/frontend:${{ github.sha }} ./frontend
      - name: Push to registry
        run: |
          echo ${{ secrets.DOCKER_PASSWORD }} | docker login -u ${{ secrets.DOCKER_USERNAME }} --password-stdin
          docker push adx-core/api-gateway:${{ github.sha }}
          docker push adx-core/frontend:${{ github.sha }}

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to VPS
        uses: appleboy/ssh-action@v0.1.5
        with:
          host: ${{ secrets.VPS_HOST }}
          username: ${{ secrets.VPS_USER }}
          key: ${{ secrets.VPS_SSH_KEY }}
          script: |
            cd /opt/adx-core
            docker-compose -f docker-compose.prod.yml pull
            docker-compose -f docker-compose.prod.yml up -d
            docker system prune -f
```

## Cloud Marketplace Deployment

### AWS Marketplace
- **ECS Fargate** deployment with CloudFormation
- **RDS PostgreSQL** for database
- **ElastiCache Redis** for caching
- **Application Load Balancer** for traffic distribution
- **CloudWatch** for monitoring and logging

### Google Cloud Marketplace
- **GKE Autopilot** for container orchestration
- **Cloud SQL PostgreSQL** for database
- **Memorystore Redis** for caching
- **Cloud Load Balancing** for traffic distribution
- **Cloud Logging** for centralized logs

### Azure Marketplace
- **AKS** for container orchestration
- **Azure Database for PostgreSQL** for database
- **Azure Cache for Redis** for caching
- **Azure Load Balancer** for traffic distribution
- **Azure Monitor** for logging and metrics

## Monitoring & Logging

### Basic Logging Setup
```yaml
# logging/docker-compose.logging.yml
version: '3.8'
services:
  # Grafana for dashboards
  grafana:
    image: grafana/grafana:latest
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
    volumes:
      - grafana_data:/var/lib/grafana
      - ./grafana/dashboards:/etc/grafana/provisioning/dashboards
      - ./grafana/datasources:/etc/grafana/provisioning/datasources

  # Prometheus for metrics
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus

  # Loki for log aggregation
  loki:
    image: grafana/loki:latest
    ports:
      - "3100:3100"
    volumes:
      - ./loki/loki.yml:/etc/loki/local-config.yaml
      - loki_data:/loki

volumes:
  grafana_data:
  prometheus_data:
  loki_data:
```

### Log Configuration
- **Structured JSON logging** for all services
- **Correlation IDs** for request tracing
- **Error aggregation** with alerting
- **Performance metrics** collection
- **Temporal workflow monitoring**

## Backup Strategy

### Database Backups
```bash
#!/bin/bash
# scripts/backup-database.sh
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups"
DB_NAME="adx_core"

# Create backup
docker exec postgres pg_dump -U postgres $DB_NAME > $BACKUP_DIR/backup_$DATE.sql

# Compress backup
gzip $BACKUP_DIR/backup_$DATE.sql

# Keep only last 30 days of backups
find $BACKUP_DIR -name "backup_*.sql.gz" -mtime +30 -delete

echo "Backup completed: backup_$DATE.sql.gz"
```

### Automated Backup Schedule
```yaml
# Cron job for automated backups
0 2 * * * /opt/adx-core/scripts/backup-database.sh
0 3 * * 0 /opt/adx-core/scripts/backup-full-system.sh
```

## Security Considerations

### Basic Security Measures
- **Environment variables** for sensitive configuration
- **Docker secrets** for production passwords
- **Network isolation** between services
- **Regular security updates** for base images
- **Backup encryption** for data protection

### Production Security Checklist
- [ ] All default passwords changed
- [ ] Environment variables configured
- [ ] Firewall rules configured
- [ ] SSL certificates installed (when ready)
- [ ] Database access restricted
- [ ] Regular backup testing
- [ ] Security monitoring enabled

## IDE Setup and Debugging

### Recommended IDE Configuration

#### VS Code Setup
```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.features": ["temporal-testing"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "rust-analyzer.cargo.loadOutDirsFromCheck": true,
  "typescript.preferences.importModuleSpecifier": "relative",
  "editor.formatOnSave": true,
  "editor.codeActionsOnSave": {
    "source.fixAll.eslint": true
  }
}

// .vscode/launch.json - Rust debugging
{
  "version": "0.2.0",
  "configurations": [
    {
      "type": "lldb",
      "request": "launch",
      "name": "Debug API Gateway",
      "cargo": {
        "args": ["build", "--bin=api-gateway"],
        "filter": {
          "name": "api-gateway",
          "kind": "bin"
        }
      },
      "args": [],
      "cwd": "${workspaceFolder}/backend",
      "env": {
        "DATABASE_URL": "postgresql://postgres:postgres@localhost:5432/adx_core",
        "REDIS_URL": "redis://localhost:6379",
        "TEMPORAL_URL": "localhost:7233",
        "RUST_LOG": "debug"
      }
    }
  ]
}
```

#### RustRover/IntelliJ Setup
- **Rust Plugin**: Enable Rust plugin with Cargo integration
- **Database Plugin**: Connect to PostgreSQL for schema inspection
- **Temporal Plugin**: If available, for workflow visualization
- **Debugger**: Native LLDB debugging with breakpoints

### Development Workflow (IDE-Optimized)

#### Daily Development Process
1. **Start Infrastructure**: `make infra` (starts Docker services)
2. **Open IDE**: VS Code, RustRover, or preferred IDE
3. **Start Backend**: `make dev-backend` (native Rust with debugging)
4. **Start Frontend**: `make dev-frontend` (native React with hot reload)
5. **Set Breakpoints**: Use IDE debugging features
6. **Write Tests**: TDD with native test runner
7. **Debug Workflows**: Use Temporal UI + IDE breakpoints

#### Debugging Temporal Workflows
```rust
// Enable debug logging for Temporal workflows
#[workflow]
pub async fn debug_workflow(input: WorkflowInput) -> WorkflowResult<WorkflowOutput> {
    // Set breakpoint here - will work in IDE!
    tracing::debug!("Workflow started with input: {:?}", input);
    
    // Step through activities in debugger
    let result = my_activity(input.data).await?;
    
    // Inspect variables in IDE
    tracing::debug!("Activity result: {:?}", result);
    
    Ok(WorkflowOutput { result })
}
```

#### Frontend Debugging
```typescript
// React DevTools + browser debugging
export const MyComponent: React.FC = () => {
  const { execute, status } = useTemporalWorkflow('my-workflow');
  
  // Set breakpoint here - works in browser DevTools
  const handleClick = async () => {
    console.log('Starting workflow...'); // Debug in browser console
    await execute({ data: 'test' });
  };
  
  return <button onClick={handleClick}>Execute Workflow</button>;
};
```

## AI Coder Team Guidelines (IDE-Optimized)

### Development Workflow for AI Coders
1. **Start infrastructure**: `make infra` for Docker services only
2. **Use native development**: Run Rust/Frontend natively for IDE debugging
3. **Set breakpoints**: Use IDE debugging features extensively
4. **Follow Temporal-First Principle** for all complex operations
5. **Write tests first** using TDD with native test runner
6. **Debug workflows**: Combine Temporal UI with IDE breakpoints
7. **Use structured logging** with correlation IDs
8. **Hot reload**: Frontend changes reflect immediately

### Code Quality Standards
- **Rust clippy** linting with no warnings
- **Unit test coverage** > 80%
- **Integration tests** for all workflows
- **Documentation** for all public APIs
- **Error handling** for all failure scenarios
- **Performance benchmarks** for critical paths

This DevOps strategy provides a **simple, scalable foundation** for AI Coder Teams to build ADX CORE efficiently with 2-week sprint cycles and multi-cloud deployment capabilities.