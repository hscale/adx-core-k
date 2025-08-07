# ADX CORE Development Environment Setup

## 🚀 Quick Start - Get Coding in 30 Minutes

**Status**: IMMEDIATE SETUP REQUIRED
**Target**: All foundation teams (1, 2, 8) ready to code today

## Prerequisites Installation

### 1. Install Rust (Required)
```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Install required components
rustup component add clippy rustfmt
cargo install cargo-watch sqlx-cli
```

### 2. Install Node.js (Required)
```bash
# Install Node.js 18+
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt-get install -y nodejs

# Install global tools
npm install -g typescript @types/node
```

### 3. Install Docker (Required)
```bash
# Install Docker
curl -fsSL https://get.docker.com | sh
sudo usermod -aG docker $USER

# Install Docker Compose
sudo curl -L "https://github.com/docker/compose/releases/download/v2.20.0/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
sudo chmod +x /usr/local/bin/docker-compose
```

### 4. Install Kubernetes Tools (Required)
```bash
# Install kubectl
curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl

# Install minikube for local development
curl -Lo minikube https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
sudo install minikube /usr/local/bin/
```

## Repository Setup

### 1. Clone and Initialize Repository
```bash
# Create project directory
mkdir adx-core && cd adx-core
git init

# Create workspace structure
mkdir -p {services,frontend,infrastructure,docs,scripts,tests}
mkdir -p services/{shared,api-gateway,auth-service,user-service,file-service,workflow-service}
mkdir -p frontend/{web-app,admin-app,mobile-app}
mkdir -p infrastructure/{terraform,kubernetes,docker}
```

### 2. Create Cargo Workspace
```toml
# File: Cargo.toml
[workspace]
members = [
    "services/shared",
    "services/api-gateway",
    "services/auth-service",
    "services/user-service",
    "services/file-service",
    "services/workflow-service",
]

[workspace.dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }
axum = "0.7"
tracing = "0.1"
tracing-subscriber = "0.3"
async-trait = "0.1"
thiserror = "1.0"
```

## Infrastructure Setup

### 1. Development Docker Compose
```yaml
# File: infrastructure/docker/docker-compose.dev.yml
version: '3.8'
services:
  postgres:
    image: postgres:14
    environment:
      POSTGRES_DB: adx_core
      POSTGRES_USER: adx_user
      POSTGRES_PASSWORD: dev_password
    ports:
      - "5432:5432"
    volumes:
      - postgres_data:/var/lib/postgresql/data

  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data

  temporal:
    image: temporalio/auto-setup:1.20.0
    ports:
      - "7233:7233"
      - "8233:8233"
    environment:
      - DB=postgresql
      - DB_PORT=5432
      - POSTGRES_USER=adx_user
      - POSTGRES_PWD=dev_password
      - POSTGRES_SEEDS=postgres
    depends_on:
      - postgres

  temporal-ui:
    image: temporalio/ui:2.10.0
    ports:
      - "8080:8080"
    environment:
      - TEMPORAL_ADDRESS=temporal:7233
    depends_on:
      - temporal

volumes:
  postgres_data:
  redis_data:
```

### 2. Start Development Infrastructure
```bash
# Start all infrastructure services
cd infrastructure/docker
docker-compose -f docker-compose.dev.yml up -d

# Verify services are running
docker-compose -f docker-compose.dev.yml ps
```

## Database Setup

### 1. Create Database Schema
```sql
-- File: migrations/001_initial_schema.sql
-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Create tenants table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE,
    settings JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    profile JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

-- Create roles table
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(100) NOT NULL,
    permissions JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

-- Create user_roles table
CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id),
    role_id UUID NOT NULL REFERENCES roles(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(user_id, role_id)
);

-- Create events table
CREATE TABLE events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id),
    event_type VARCHAR(255) NOT NULL,
    aggregate_id UUID NOT NULL,
    aggregate_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create indexes
CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_events_tenant_id ON events(tenant_id);
CREATE INDEX idx_events_type ON events(event_type);
CREATE INDEX idx_events_aggregate ON events(aggregate_id, aggregate_type);
```

### 2. Run Database Migrations
```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli

# Set database URL
export DATABASE_URL="postgresql://adx_user:dev_password@localhost:5432/adx_core"

# Run migrations
sqlx migrate run --source migrations/
```

## Development Scripts

### 1. Development Startup Script
```bash
#!/bin/bash
# File: scripts/dev-start.sh

echo "🚀 Starting ADX CORE Development Environment"

# Start infrastructure
echo "Starting infrastructure services..."
cd infrastructure/docker
docker-compose -f docker-compose.dev.yml up -d

# Wait for services to be ready
echo "Waiting for services to be ready..."
sleep 10

# Run database migrations
echo "Running database migrations..."
export DATABASE_URL="postgresql://adx_user:dev_password@localhost:5432/adx_core"
sqlx migrate run --source ../../migrations/

# Start development servers
echo "Starting development servers..."
cd ../../

# Start API Gateway
cargo run --bin api-gateway &
API_GATEWAY_PID=$!

# Start Auth Service
cargo run --bin auth-service &
AUTH_SERVICE_PID=$!

echo "✅ Development environment ready!"
echo "API Gateway: http://localhost:8080"
echo "Temporal UI: http://localhost:8080"
echo "Database: postgresql://localhost:5432/adx_core"

# Wait for interrupt
trap "kill $API_GATEWAY_PID $AUTH_SERVICE_PID; docker-compose -f infrastructure/docker/docker-compose.dev.yml down" EXIT
wait
```

### 2. Make Script Executable
```bash
chmod +x scripts/dev-start.sh
```

## IDE Configuration

### 1. VS Code Settings
```json
// File: .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "editor.formatOnSave": true,
    "files.associations": {
        "*.sql": "sql"
    },
    "sqltools.connections": [
        {
            "name": "ADX Core Dev",
            "driver": "PostgreSQL",
            "previewLimit": 50,
            "server": "localhost",
            "port": 5432,
            "database": "adx_core",
            "username": "adx_user",
            "password": "dev_password"
        }
    ]
}
```

### 2. VS Code Extensions
```json
// File: .vscode/extensions.json
{
    "recommendations": [
        "rust-lang.rust-analyzer",
        "vadimcn.vscode-lldb",
        "serayuzgur.crates",
        "tamasfe.even-better-toml",
        "ms-vscode.vscode-typescript-next",
        "bradlc.vscode-tailwindcss",
        "mtxr.sqltools",
        "mtxr.sqltools-driver-pg"
    ]
}
```

## Environment Variables

### 1. Development Environment File
```bash
# File: .env.development
DATABASE_URL=postgresql://adx_user:dev_password@localhost:5432/adx_core
REDIS_URL=redis://localhost:6379
TEMPORAL_URL=localhost:7233
JWT_SECRET=dev_secret_key_change_in_production
LOG_LEVEL=debug
RUST_LOG=debug
```

### 2. Load Environment Variables
```bash
# Add to your shell profile (.bashrc, .zshrc)
export $(cat .env.development | xargs)
```

## Verification Steps

### 1. Test Database Connection
```bash
psql postgresql://adx_user:dev_password@localhost:5432/adx_core -c "SELECT version();"
```

### 2. Test Redis Connection
```bash
redis-cli ping
```

### 3. Test Temporal Connection
```bash
curl http://localhost:8233/api/v1/namespaces
```

### 4. Build All Services
```bash
cargo build --workspace
```

## Quick Start Commands

### Start Development Environment
```bash
# One command to start everything
./scripts/dev-start.sh
```

### Run Individual Services
```bash
# API Gateway
cargo run --bin api-gateway

# Auth Service
cargo run --bin auth-service

# Run with auto-reload
cargo watch -x "run --bin api-gateway"
```

### Run Tests
```bash
# Run all tests
cargo test --workspace

# Run integration tests
cargo test --test integration

# Run with coverage
cargo tarpaulin --workspace
```

## Troubleshooting

### Common Issues

#### Docker Permission Denied
```bash
sudo usermod -aG docker $USER
newgrp docker
```

#### Database Connection Failed
```bash
# Check if PostgreSQL is running
docker-compose -f infrastructure/docker/docker-compose.dev.yml ps postgres

# Check logs
docker-compose -f infrastructure/docker/docker-compose.dev.yml logs postgres
```

#### Port Already in Use
```bash
# Find process using port
lsof -i :8080

# Kill process
kill -9 <PID>
```

## Ready to Code!

Your development environment is now ready. Start with:

1. **Team 1**: Begin with database and Temporal infrastructure
2. **Team 2**: Start with authentication service
3. **Team 8**: Set up CI/CD and monitoring

**Next Steps**:
- Review your team's specific task file
- Start implementing using the provided code templates
- Set up daily standups and coordination calls

Let's build ADX CORE! 🚀