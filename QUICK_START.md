# ADX CORE - Quick Start Guide

Get up and running with the ADX CORE platform in under 5 minutes.

## 🚀 One-Command Start

```bash
# From project root
./scripts/dev-start.sh

# Or from adx-core directly
cd adx-core
./scripts/dev-start.sh
```

That's it! The development environment will start automatically.

## 🏗 Project Structure Overview

```
adx-core/
├── adx-core/              # Core platform (START HERE)
│   ├── services/          # Microservices
│   ├── infrastructure/    # Docker, K8s configs
│   └── scripts/          # Development scripts
├── scripts/              # Root-level scripts
└── .kiro/specs/         # Development specifications
```

## 📋 What's Running (ADX Core)

After running the start script, you'll have:

### Services
- **API Gateway**: `http://localhost:8080` - Main entry point
- **Auth Service**: `http://localhost:8081` - Authentication & JWT
- **User Service**: `http://localhost:8082` - User management
- **Temporal UI**: `http://localhost:8088` - Workflow monitoring

### Infrastructure
- **PostgreSQL**: `localhost:5432` - Database with sample data
- **Redis**: `localhost:6379` - Caching layer
- **Temporal**: `localhost:7233` - Workflow engine

## 🧪 Test Everything Works

### 1. Health Check
```bash
curl http://localhost:8080/health
# Expected: "OK"
```

### 2. Authentication
```bash
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "admin@example.com",
    "password": "password",
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

### 3. User Management
```bash
# List users
curl http://localhost:8080/api/v1/users

# Create user
curl -X POST http://localhost:8080/api/v1/users \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "test@example.com",
    "password": "password"
  }'
```

### 4. Temporal Workflows
- Visit **Temporal UI**: http://localhost:8088
- Check workflow executions and task queues

## 🛑 Stop Everything

Press `Ctrl+C` in the terminal where you ran the start script, or:

```bash
# From adx-core directory
docker compose -f infrastructure/docker/docker-compose.dev.yml down
```

## 🔧 Manual Setup (If Needed)

### Prerequisites
- Rust 1.88+ (latest stable)
- Docker & Docker Compose
- PostgreSQL 14+
- Redis 6+
- Temporal Server

### Step-by-Step
```bash
# Navigate to adx-core
cd adx-core

# 1. Build services
cargo build --workspace

# 2. Start infrastructure
docker compose -f infrastructure/docker/docker-compose.dev.yml up -d

# 3. Run services
cargo run --bin api-gateway &
cargo run --bin auth-service &
cargo run --bin user-service &
```

## 📊 Monitoring & Tools

- **Temporal UI**: http://localhost:8088 - Workflow monitoring
- **Database**: Connect with any PostgreSQL client to `postgresql://adx_user:dev_password@localhost:5432/adx_core`
- **Redis**: Connect to `redis://localhost:6379`
- **Logs**: All services log to stdout with structured JSON

## 🎯 Development Teams

### Choose Your Focus Area:

#### ADX Core (adx-core/)
- **Team 1**: Platform Foundation ✅ **COMPLETE**
- **Team 2**: Identity & Security ✅ **COMPLETE**  
- **Team 8**: Operations ✅ **COMPLETE**

#### Future Modules
- **AI Engine**: Machine learning processing
- **Analytics Platform**: Performance insights
- **Integration Hub**: Third-party integrations

## � Dkevelopment Specifications

Get detailed guidance from the specs:

- **[Immediate Start Guide](.kiro/specs/adx-core/development-kickoff/immediate-start-guide.md)** - Get started immediately
- **[Environment Setup](.kiro/specs/adx-core/development-kickoff/development-environment-setup.md)** - Complete setup guide
- **[Team 1 Tasks](.kiro/specs/adx-core/development-kickoff/team-1-foundation-tasks.md)** - Platform foundation tasks

## 🚨 Troubleshooting

### Port Already in Use
```bash
# Find what's using the port
lsof -i :8080

# Kill the process
kill -9 <PID>
```

### Docker Issues
```bash
# Reset Docker state
cd adx-core
docker compose -f infrastructure/docker/docker-compose.dev.yml down
docker system prune -f
```

### Database Connection Issues
```bash
# Check if PostgreSQL is running
cd adx-core
docker compose -f infrastructure/docker/docker-compose.dev.yml ps postgres

# View logs
docker compose -f infrastructure/docker/docker-compose.dev.yml logs postgres
```

### Temporal Issues
```bash
# Check Temporal health
docker exec docker-temporal-1 tctl --address temporal:7233 cluster health

# View Temporal logs
docker logs docker-temporal-1
```

## 📚 Next Steps

1. **Explore the API**: Use the curl examples above
2. **Check the code**: Look at `adx-core/services/` directory
3. **Read the specs**: Check `.kiro/specs/` for detailed documentation
4. **Choose your team**: Pick a focus area and start contributing
5. **Start developing**: Follow the team-specific task guides

## 🤝 Need Help?

- **Documentation**: Check `.kiro/specs/` directory
- **Team Channels**: 
  - #team-1-foundation
  - #team-2-security  
  - #team-8-operations
- **Issues**: Create a GitHub issue
- **Emergency**: #blockers channel

---

**You're ready to build the future of performance excellence!** 🚀 
Start with the [Immediate Start Guide](.kiro/specs/adx-core/development-kickoff/immediate-start-guide.md)