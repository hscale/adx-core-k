# Auth Service Database Layer

This document describes the database layer implementation for the Auth Service, which provides CRUD operations and tenant isolation for authentication-related data.

## Overview

The Auth Service database layer consists of three main repositories:

1. **UserRepository** - Manages user accounts with tenant isolation
2. **SessionRepository** - Handles user sessions and authentication tokens
3. **AuthTokenRepository** - Manages password reset and email verification tokens

## Features

### Multi-Tenant Support
- All repositories enforce tenant isolation at the database level
- Row-level security policies ensure data separation between tenants
- Tenant context is automatically injected into all queries

### User Management
- CRUD operations for user accounts
- Email-based user lookup
- Password management and verification
- User status tracking (active, inactive, suspended, pending_verification)
- Role and permission management

### Session Management
- Secure session token generation and validation
- Refresh token support for token renewal
- Session expiration and cleanup
- Device and IP tracking for security
- Bulk session revocation

### Authentication Tokens
- Password reset tokens with expiration
- Email verification tokens
- Token validation and one-time use enforcement
- Automatic cleanup of expired tokens

## Database Schema

The implementation uses the following tables:

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    status user_status NOT NULL DEFAULT 'pending_verification',
    roles TEXT[] DEFAULT '{"user"}',
    permissions TEXT[] DEFAULT '{}',
    preferences JSONB DEFAULT '{}',
    last_login_at TIMESTAMPTZ,
    email_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);
```

### User Sessions Table
```sql
CREATE TABLE user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    refresh_token VARCHAR(255) NOT NULL UNIQUE,
    status session_status NOT NULL DEFAULT 'active',
    ip_address INET,
    user_agent TEXT,
    device_id VARCHAR(255),
    expires_at TIMESTAMPTZ NOT NULL,
    last_activity_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Password Reset Tokens Table
```sql
CREATE TABLE password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Email Verification Tokens Table
```sql
CREATE TABLE email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

## Usage Examples

### Creating a User Repository
```rust
use adx_shared::database::create_database_pool_with_config;
use auth_service::repositories::UserRepository;

let pool = create_database_pool_with_config(db_config).await?;
let tenant_id = "tenant-uuid-here";
let user_repo = UserRepository::new(pool, tenant_id.to_string());
```

### User Operations
```rust
// Find user by email
let user = user_repo.find_by_email("user@example.com").await?;

// Update last login
user_repo.update_last_login(&user_id).await?;

// Verify email
user_repo.verify_email(&user_id).await?;

// Update password
user_repo.update_password(&user_id, &new_password_hash).await?;
```

### Session Operations
```rust
let session_repo = SessionRepository::new(pool, tenant_id.to_string());

// Create session
let request = CreateSessionRequest {
    user_id: user_id.clone(),
    session_token: "secure-token".to_string(),
    refresh_token: "refresh-token".to_string(),
    ip_address: Some("127.0.0.1".to_string()),
    user_agent: Some("Mozilla/5.0...".to_string()),
    device_id: Some("device-123".to_string()),
    expires_at: Utc::now() + Duration::hours(24),
};

let session = session_repo.create_from_request(request).await?;

// Validate session
let valid_session = session_repo.validate_session(&session_token).await?;

// Revoke session
session_repo.revoke_session(&session_id).await?;
```

### Token Operations
```rust
let token_repo = AuthTokenRepository::new(pool, tenant_id.to_string());

// Create password reset token
let request = CreatePasswordResetTokenRequest {
    user_id: user_id.clone(),
    token: "reset-token-123".to_string(),
    expires_at: Utc::now() + Duration::hours(1),
};

let reset_token = token_repo.create_password_reset_token(request).await?;

// Validate and use token
let used_token = token_repo.validate_and_use_password_reset_token(&token).await?;
```

## Performance Optimizations

### Database Indexes
The implementation includes optimized indexes for:
- User email lookups within tenants
- Session token validation
- Token expiration cleanup
- Tenant-based queries

### Connection Pooling
- Configurable connection pool settings
- Health checks and connection validation
- Automatic connection recycling

### Query Optimization
- Efficient tenant filtering
- Batch operations for cleanup tasks
- Prepared statements for security

## Security Features

### Tenant Isolation
- Row-level security policies
- Automatic tenant context injection
- Prevention of cross-tenant data access

### Token Security
- Cryptographically secure token generation
- One-time use enforcement
- Automatic expiration handling
- Secure token validation

### Session Security
- Session token uniqueness
- IP and device tracking
- Automatic session cleanup
- Bulk revocation capabilities

## Development Setup

1. Copy the environment configuration:
```bash
cp .env.example .env
```

2. Update the DATABASE_URL in .env:
```
DATABASE_URL=postgresql://username:password@localhost:5432/adx_core
```

3. Run database migrations:
```bash
cargo run --bin db-manager migrate
```

4. Build the auth service:
```bash
cargo build --package auth-service
```

## Testing

Tests are currently disabled due to SQLx macro compilation requirements. To enable tests:

1. Set up a test database
2. Configure TEST_DATABASE_URL environment variable
3. Run `cargo sqlx prepare` to generate query metadata
4. Re-enable test modules in repository files

## Future Enhancements

- [ ] Add Redis caching for frequently accessed data
- [ ] Implement audit logging for all database operations
- [ ] Add database connection health monitoring
- [ ] Implement automatic token cleanup background tasks
- [ ] Add support for database read replicas
- [ ] Implement connection pooling per tenant