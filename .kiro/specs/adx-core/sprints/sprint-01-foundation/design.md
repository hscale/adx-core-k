# Sprint 1: Foundation Infrastructure - Design

## Architecture Overview

### Project Structure
```
adx-core/
├── backend/
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── lib.rs
│   │   ├── config/
│   │   ├── database/
│   │   ├── repositories/
│   │   └── models/
│   └── migrations/
├── frontend/
│   ├── package.json
│   ├── vite.config.ts
│   ├── src/
│   │   ├── main.tsx
│   │   ├── App.tsx
│   │   ├── components/
│   │   └── styles/
│   └── public/
├── docker-compose.yml
└── README.md
```

### Database Schema Design

#### Core Tables
```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255),
    preferences JSONB DEFAULT '{}',
    email_verified BOOLEAN DEFAULT FALSE,
    mfa_enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tenants (Companies) table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE,
    settings JSONB DEFAULT '{}',
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);
```

### Repository Pattern Implementation

#### Repository Traits
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: CreateUser) -> Result<User, Error>;
    async fn get_user_by_id(&self, id: UserId) -> Result<Option<User>, Error>;
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>, Error>;
    async fn update_user(&self, id: UserId, updates: UpdateUser) -> Result<User, Error>;
    async fn delete_user(&self, id: UserId) -> Result<(), Error>;
}

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn create_tenant(&self, tenant: CreateTenant) -> Result<Tenant, Error>;
    async fn get_tenant_by_id(&self, id: TenantId) -> Result<Option<Tenant>, Error>;
    async fn list_user_tenants(&self, user_id: UserId) -> Result<Vec<Tenant>, Error>;
    async fn update_tenant(&self, id: TenantId, updates: UpdateTenant) -> Result<Tenant, Error>;
}
```

## Implementation Details

### Database Configuration
- Connection pooling with 10-50 connections
- SSL required for production
- Migration system using sqlx-migrate
- Proper indexing for performance

### Error Handling Strategy
- Custom error types for different domains
- Proper error propagation with context
- Logging for debugging and monitoring

### Testing Strategy
- Unit tests for repository implementations
- Integration tests with test database
- Mock repositories for service testing