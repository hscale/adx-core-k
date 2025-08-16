# ADX CORE Development Environment Setup

This guide will help you set up and run the complete ADX CORE development environment for live testing.

## Prerequisites

Before starting, ensure you have the following installed:

- **Docker Desktop** - Running and accessible
- **Node.js 18+** - For frontend micro-apps and BFF services
- **Rust 1.70+** - For backend microservices
- **Git** - For version control

## Quick Start

### 1. Setup Development Environment

Run the setup script to install all dependencies:

```bash
./scripts/setup-dev.sh
```

This will:
- Install all npm dependencies for frontend micro-apps
- Build the Rust workspace
- Install required tools (sqlx-cli, concurrently)
- Create necessary directories and environment files

### 2. Start All Services

Start the complete development environment:

```bash
./scripts/dev-start-all.sh
```

This will start:
- **Infrastructure**: PostgreSQL, Redis, Temporal, Prometheus, Grafana, Jaeger
- **Backend Services**: API Gateway, Auth, User, File, Workflow, Tenant, Module services
- **Temporal Workers**: For each backend service
- **BFF Services**: Backend for Frontend optimization layer
- **Frontend Micro-Apps**: Shell, Auth, Tenant, File, User, Workflow, Module apps

### 3. Verify Everything is Running

Check the health of all services:

```bash
./scripts/health-check.sh
```

## Service URLs

Once everything is running, you can access:

### 🎨 Frontend Applications
- **Main Application**: http://localhost:3000
- **Auth Micro-App**: http://localhost:3001
- **Tenant Micro-App**: http://localhost:3002
- **File Micro-App**: http://localhost:3003
- **User Micro-App**: http://localhost:3004
- **Workflow Micro-App**: http://localhost:3005
- **Module Micro-App**: http://localhost:3006

### 🔧 Backend Services
- **API Gateway**: http://localhost:8080
- **Auth Service**: http://localhost:8081
- **User Service**: http://localhost:8082
- **File Service**: http://localhost:8083
- **Workflow Service**: http://localhost:8084
- **Tenant Service**: http://localhost:8085
- **Module Service**: http://localhost:8086

### 🌐 BFF Services
- **Auth BFF**: http://localhost:4001
- **Tenant BFF**: http://localhost:4002
- **File BFF**: http://localhost:4003
- **User BFF**: http://localhost:4004
- **Workflow BFF**: http://localhost:4005
- **Module BFF**: http://localhost:4006

### 🏗️ Infrastructure & Monitoring
- **Temporal UI**: http://localhost:8088
- **Grafana**: http://localhost:3001 (admin/admin)
- **Prometheus**: http://localhost:9090
- **Jaeger**: http://localhost:16686
- **PostgreSQL**: localhost:5432
- **Redis**: localhost:6379

## Default Test Accounts

The development environment comes with pre-configured test accounts:

### Default Tenant
- **Tenant**: Default Tenant
- **Admin Email**: admin@adxcore.local
- **Password**: admin123
- **Roles**: admin, user

### Demo Tenant
- **Tenant**: Demo Tenant
- **User Email**: demo@adxcore.local
- **Password**: demo123
- **Roles**: user

## Development Workflow

### Starting Individual Services

If you need to start services individually:

```bash
# Backend services (server mode)
cd adx-core
cargo run --bin auth-service
cargo run --bin user-service
cargo run --bin file-service

# Backend services (worker mode)
cargo run --bin auth-service -- --mode worker
cargo run --bin user-service -- --mode worker

# Frontend micro-apps
npm run dev:shell    # Shell application
npm run dev:auth     # Auth micro-app
npm run dev:tenant   # Tenant micro-app
```

### Viewing Logs

Service logs are stored in the `./logs/` directory:

```bash
# View specific service logs
tail -f logs/auth-service.log
tail -f logs/shell-frontend.log

# View all logs
ls logs/
```

### Database Access

Connect to PostgreSQL:

```bash
# Using psql
psql -h localhost -p 5432 -U postgres -d adx_core

# Using Docker
docker exec -it adx-postgres psql -U postgres -d adx_core
```

### Temporal Workflows

Access Temporal UI at http://localhost:8088 to:
- Monitor workflow executions
- View workflow history
- Debug workflow issues
- Manage workflow schedules

## Testing

### Run All Tests

```bash
npm run test:all
```

### Specific Test Types

```bash
# Unit tests
npm run test:unit

# Integration tests
npm run test:integration

# E2E tests
npm run test:e2e

# Workflow tests
npm run test:workflow

# Cross-platform tests
npm run test:cross-platform
```

## Troubleshooting

### Services Not Starting

1. **Check Docker**: Ensure Docker Desktop is running
2. **Check Ports**: Make sure required ports are not in use
3. **Check Logs**: Look at service logs in `./logs/` directory
4. **Restart Services**: Stop and restart the development environment

### Database Issues

1. **Reset Database**: 
   ```bash
   docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml down -v
   ./scripts/dev-start-all.sh
   ```

2. **Check Connection**: Verify PostgreSQL is accessible at localhost:5432

### Frontend Issues

1. **Clear Node Modules**:
   ```bash
   # For each micro-app
   cd apps/[app-name]
   rm -rf node_modules package-lock.json
   npm install
   ```

2. **Check Module Federation**: Ensure all micro-apps are running on correct ports

### Temporal Issues

1. **Check Temporal Server**: Visit http://localhost:8088
2. **Restart Temporal**: 
   ```bash
   docker restart adx-temporal
   ```

## Stopping Services

To stop all services:

1. **Press Ctrl+C** in the terminal running `dev-start-all.sh`
2. **Or run**: 
   ```bash
   docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml down
   ```

## Architecture Overview

The ADX CORE platform follows a microservices architecture with:

- **Temporal-First Backend**: All complex operations implemented as Temporal workflows
- **Frontend Microservices**: Module Federation for independent micro-apps
- **Multi-Tenant**: Complete isolation at database, application, and workflow levels
- **BFF Pattern**: Optional optimization layer for frontend data aggregation
- **Event-Driven**: Cross-service communication through events and workflows

## Next Steps

1. **Explore the UI**: Visit http://localhost:3000 and explore the application
2. **Check Workflows**: Monitor workflow executions at http://localhost:8088
3. **View Metrics**: Check system metrics at http://localhost:3001 (Grafana)
4. **Run Tests**: Execute the test suite to ensure everything works
5. **Start Development**: Begin implementing features using the established patterns

For more detailed information, check the documentation in the `docs/` directory.