# ADX CORE Quick Start Guide

## 🚀 Quick Development Setup

### Prerequisites
- Docker and Docker Compose
- Rust 1.88+ with Cargo
- Node.js 18+ and npm (for frontend)

### 1. Start Development Environment
```bash
# Start everything (backend + infrastructure)
./quick-dev.sh

# Or start manually:
# 1. Start infrastructure
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml up -d

# 2. Build and start backend services
cd adx-core
cargo build --workspace
RUST_LOG=info cargo run -p auth-service &
RUST_LOG=info cargo run -p user-service &
RUST_LOG=info cargo run -p file-service &
RUST_LOG=info cargo run -p workflow-service &
RUST_LOG=info cargo run -p tenant-service &
RUST_LOG=info cargo run -p api-gateway &
```

### 2. Test API Endpoints
```bash
# Run comprehensive API tests
./test-api.sh

# Or test manually:
curl http://localhost:8080/health
curl http://localhost:8081/health
```

### 3. Frontend Development (when ready)
```bash
cd frontend
npm install
npm run dev
# Frontend will be available at http://localhost:1420
```

## 🌐 Service URLs

| Service | URL | Description |
|---------|-----|-------------|
| API Gateway | http://localhost:8080 | Main entry point |
| Auth Service | http://localhost:8081 | Authentication |
| User Service | http://localhost:8082 | User management |
| File Service | http://localhost:8083 | File operations |
| Workflow Service | http://localhost:8084 | Business workflows |
| Tenant Service | http://localhost:8085 | Multi-tenancy |
| Temporal UI | http://localhost:8088 | Workflow monitoring |

## 🔧 Infrastructure

| Service | URL | Credentials |
|---------|-----|-------------|
| PostgreSQL | localhost:5432 | adx_user / dev_password |
| Redis | localhost:6379 | No auth |
| Temporal | localhost:7233 | No auth |

## 🔐 Demo Login Credentials

### Frontend Login
- **URL**: http://localhost:1420
- **Email**: `admin@example.com`
- **Password**: `password`

### API Authentication
```bash
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{"email":"admin@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}'
```

## 🧪 Quick API Tests

### User Management
```bash
# List users (through API Gateway)
curl http://localhost:8080/api/v1/users

# Create user
curl -X POST http://localhost:8080/api/v1/users \
  -H 'Content-Type: application/json' \
  -d '{"email":"test@example.com","password":"password","tenant_id":"550e8400-e29b-41d4-a716-446655440000"}'
```

### Health Checks
```bash
# Check all services
for port in 8080 8081 8082 8083 8084 8085; do
  echo "Port $port: $(curl -s http://localhost:$port/health)"
done
```

## 📋 Development Commands

### View Logs
```bash
# Service logs
tail -f logs/auth-service.log
tail -f logs/api-gateway.log

# Infrastructure logs
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml logs -f
```

### Stop Services
```bash
# Stop backend services
pkill -f "cargo run"

# Stop infrastructure
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml down
```

### Restart Services
```bash
# Restart specific service
pkill -f "auth-service"
cd adx-core && RUST_LOG=info cargo run -p auth-service > ../logs/auth-service.log 2>&1 &

# Restart infrastructure
docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml restart
```

## 🐛 Troubleshooting

### Common Issues

1. **Port already in use**
   ```bash
   # Find and kill process using port
   lsof -ti:8080 | xargs kill -9
   ```

2. **Database connection failed**
   ```bash
   # Check PostgreSQL status
   docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml ps postgres
   
   # Restart PostgreSQL
   docker compose -f adx-core/infrastructure/docker/docker-compose.dev.yml restart postgres
   ```

3. **Service compilation errors**
   ```bash
   # Clean and rebuild
   cd adx-core
   cargo clean
   cargo build --workspace
   ```

4. **Frontend build errors**
   ```bash
   cd frontend
   rm -rf node_modules package-lock.json
   npm install
   ```

### Log Locations
- Backend services: `./logs/`
- Infrastructure: `docker compose logs`
- Frontend: `./logs/frontend.log`

## 📚 Next Steps

1. **API Documentation**: Check service endpoints in each service's `main.rs`
2. **Database Schema**: Review migrations in `adx-core/infrastructure/docker/init.sql`
3. **Workflow Definitions**: Explore Temporal workflows in each service
4. **Frontend Development**: Fix missing context files and continue React development

## 🔗 Useful Links

- [Temporal UI](http://localhost:8088) - Workflow monitoring
- [PostgreSQL Admin](postgresql://adx_user:dev_password@localhost:5432/adx_core)
- [Redis CLI](redis://localhost:6379)

---

**Quick Commands Summary:**
```bash
./quick-dev.sh    # Start everything
./test-api.sh     # Test all APIs
pkill -f "cargo"  # Stop all services
```