# ADX CORE - AI-Powered Performance Excellence Platform

A comprehensive multi-tenant SaaS platform ecosystem featuring AI-powered workflow automation, advanced file processing, and performance analytics.

## 🏗 Project Structure

```
adx-core/
├── adx-core/              # Core platform services
│   ├── services/          # Microservices
│   │   ├── api-gateway/   # API Gateway service
│   │   ├── auth-service/  # Authentication service
│   │   ├── user-service/  # User management service
│   │   └── shared/        # Shared libraries
│   ├── infrastructure/    # Infrastructure as code
│   ├── scripts/          # Development scripts
│   └── tests/           # Integration tests
├── scripts/              # Root-level scripts
└── .kiro/               # Kiro IDE configuration
    └── specs/           # Development specifications
```

## 🚀 Quick Start

```bash
# Start development environment (from root)
./scripts/dev-start.sh

# Or start from adx-core directly
cd adx-core
./scripts/dev-start.sh
```

## 🎯 Development Teams & Focus Areas

### ADX CORE (adx-core/)
**Mission**: Multi-tenant SaaS platform foundation
- **Team 1**: Platform Foundation (Database, Temporal, API Gateway)
- **Team 2**: Identity & Security (Auth, Authorization, User Management)
- **Team 8**: Operations (DevOps, Monitoring, Infrastructure)

### Future Modules
- **AI Engine**: Machine learning and AI processing
- **Analytics Platform**: Performance metrics and insights
- **Integration Hub**: Third-party service integrations

## 📊 Service Endpoints

### ADX Core Services
- **API Gateway**: http://localhost:8080
- **Auth Service**: http://localhost:8081
- **User Service**: http://localhost:8082
- **Temporal UI**: http://localhost:8088
- **Database**: postgresql://adx_user:dev_password@localhost:5432/adx_core
- **Redis**: redis://localhost:6379

## 🧪 Test the System

### Health Check
```bash
curl http://localhost:8080/health
# Expected: "OK"
```

### Authentication Test
```bash
# Login with demo user
curl -X POST http://localhost:8081/api/v1/auth/login \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "admin@example.com",
    "password": "password", 
    "tenant_id": "550e8400-e29b-41d4-a716-446655440000"
  }'
```

### User Management Test
```bash
# List users through API Gateway
curl http://localhost:8080/api/v1/users

# Create a new user
curl -X POST http://localhost:8080/api/v1/users \
  -H 'Content-Type: application/json' \
  -d '{
    "email": "newuser@example.com",
    "password": "password123"
  }'
```

## 🛠 Development Setup

### Prerequisites
- Rust 1.88+ (latest stable)
- Node.js 18+
- Docker & Docker Compose
- PostgreSQL 14+
- Redis 6+
- Temporal Server

### Manual Setup (ADX Core)
```bash
cd adx-core

# Install dependencies
cargo build --workspace

# Start infrastructure
docker compose -f infrastructure/docker/docker-compose.dev.yml up -d

# Run services
cargo run --bin api-gateway
cargo run --bin auth-service
cargo run --bin user-service
```

## 📋 Development Specifications

Detailed development guides and specifications are available in `.kiro/specs/`:

- **[Immediate Start Guide](.kiro/specs/adx-core/development-kickoff/immediate-start-guide.md)** - Get started immediately
- **[Environment Setup](.kiro/specs/adx-core/development-kickoff/development-environment-setup.md)** - Complete setup guide
- **[Team 1 Tasks](.kiro/specs/adx-core/development-kickoff/team-1-foundation-tasks.md)** - Platform foundation tasks

## 🧪 Testing

```bash
# Test all workspaces
cargo test --workspace

# Test specific module
cd adx-core
cargo test --workspace

# Integration tests
cargo test --test integration
```

## 🎯 Team Progress

### Team 1: Platform Foundation ✅ **COMPLETE**
- [x] Database infrastructure with multi-tenant schema
- [x] API Gateway with request proxying
- [x] Event bus foundation
- [x] Development environment setup
- [x] Observability framework
- [x] Temporal workflow integration

### Team 2: Identity & Security ✅ **COMPLETE**
- [x] JWT token service
- [x] Authentication endpoints with validation
- [x] Password hashing with bcrypt
- [x] API Gateway integration
- [x] Authorization service foundation

### Team 8: Operations ✅ **COMPLETE**
- [x] Docker development environment
- [x] Database setup with migrations
- [x] Multi-service orchestration
- [x] Temporal server integration
- [x] Infrastructure monitoring

## 🔧 Configuration

Environment variables are loaded from `.env` files:
- `.env.development` - Development settings
- `.env.production` - Production settings

## 📚 Documentation

- [Quick Start Guide](QUICK_START.md)
- [Development Specifications](.kiro/specs/)
- [Architecture Overview](docs/architecture.md)

## 🤝 Contributing

1. Choose your team focus area (ADX Core, AI Engine, etc.)
2. Review the relevant specifications in `.kiro/specs/`
3. Follow the development setup for your module
4. Create feature branches and submit pull requests

### Development Workflow
1. **Start environment**: `./scripts/dev-start.sh`
2. **Make changes** to service code
3. **Test locally** with curl commands
4. **Build and verify**: `cargo build --workspace`
5. **Commit changes** with descriptive messages

## 📞 Support

### Team Channels
- **#team-1-foundation** - Database, API Gateway, Infrastructure
- **#team-2-security** - Authentication, Authorization, Security
- **#team-8-operations** - DevOps, Monitoring, Deployment

### Emergency
- **Blockers**: #blockers channel (monitored 24/7)
- **Infrastructure Issues**: @devops-lead
- **Architecture Questions**: @system-architect

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

**Ready to build the future of performance excellence?** 🚀
Start with the [Immediate Start Guide](.kiro/specs/adx-core/development-kickoff/immediate-start-guide.md)!