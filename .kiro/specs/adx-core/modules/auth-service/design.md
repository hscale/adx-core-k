# Auth Service - Technical Design

## Architecture Overview

The Auth Service is built as a stateless Rust microservice that handles all authentication and authorization concerns. It uses Redis for session storage and integrates with external identity providers.

```
┌─────────────────────────────────────────────────────────────┐
│                    Auth Service                             │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Authentication │  Authorization  │    Session Management   │
│     Module      │     Module      │        Module           │
│                 │                 │                         │
│ • Login/Logout  │ • RBAC Engine   │ • JWT Generation        │
│ • Password Auth │ • Permission    │ • Session Storage       │
│ • SSO Integration│   Checking     │ • Token Refresh         │
│ • MFA Handling  │ • Role Mgmt     │ • Concurrent Sessions   │
└─────────────────┴─────────────────┴─────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│     Redis     │    │   PostgreSQL  │    │  External SSO │
│   (Sessions)  │    │   (Users)     │    │   Providers   │
└───────────────┘    └───────────────┘    └───────────────┘
```

## Core Components

### 1. Authentication Module

**JWT Token Service**
- Generates and validates JWT tokens using RS256 algorithm
- Implements token refresh mechanism with sliding expiration
- Supports multiple token types (access, refresh, verification)

**Password Authentication**
- Uses bcrypt for password hashing (12+ rounds)
- Implements password strength validation
- Supports password reset workflows

**SSO Integration**
- SAML 2.0 support for enterprise identity providers
- OAuth 2.0 integration (Google, Microsoft, etc.)
- Active Directory/LDAP integration
- Automatic user provisioning from SSO

### 2. Multi-Factor Authentication

**TOTP Implementation**
- RFC 6238 compliant Time-based One-Time Password
- QR code generation for authenticator app setup
- Backup code generation and validation
- Recovery mechanisms for lost devices

**MFA Policy Engine**
- Tenant-level MFA enforcement policies
- Risk-based MFA triggering
- Device trust and remember functionality

### 3. Authorization Module

**RBAC Engine**
- Hierarchical role system with inheritance
- Resource-based permissions (CRUD operations)
- Dynamic permission evaluation
- Permission caching for performance

**Permission Model**
```
Permission = Resource + Action + Conditions
Examples:
- files:read:own (read own files)
- users:write:tenant (manage tenant users)
- admin:*:* (full admin access)
```

### 4. Session Management

**Session Storage**
- Redis-based session storage with encryption
- Configurable session timeouts
- Concurrent session limits per user
- Session invalidation on security events

**Token Management**
- JWT access tokens (short-lived, 15 minutes)
- Refresh tokens (long-lived, 30 days)
- Token blacklisting for immediate revocation
- Automatic token rotation

## Database Schema

### Users Table
```sql
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(320) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    name VARCHAR(255) NOT NULL,
    status user_status NOT NULL DEFAULT 'pending_verification',
    
    -- MFA fields
    mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mfa_secret VARCHAR(32),
    mfa_backup_codes TEXT[],
    
    -- SSO fields
    sso_provider VARCHAR(50),
    sso_external_id VARCHAR(255),
    
    -- Audit fields
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_login_at TIMESTAMPTZ,
    
    CONSTRAINT users_email_format CHECK (email ~ '^[^@]+@[^@]+\.[^@]+$')
);
```

### Roles and Permissions
```sql
CREATE TABLE roles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    tenant_id UUID NOT NULL,
    parent_role_id UUID REFERENCES roles(id),
    permissions JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(name, tenant_id)
);

CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role_id UUID NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assigned_by UUID REFERENCES users(id),
    
    PRIMARY KEY (user_id, role_id, tenant_id)
);
```

## API Endpoints

### Authentication Endpoints
- `POST /auth/register` - User registration
- `POST /auth/login` - User login
- `POST /auth/logout` - User logout
- `POST /auth/refresh` - Token refresh
- `POST /auth/forgot-password` - Password reset request
- `POST /auth/reset-password` - Password reset completion

### MFA Endpoints
- `POST /auth/mfa/setup` - MFA setup
- `POST /auth/mfa/verify` - MFA verification
- `POST /auth/mfa/disable` - MFA disable
- `GET /auth/mfa/backup-codes` - Get backup codes

### SSO Endpoints
- `GET /auth/sso/{provider}` - SSO initiation
- `POST /auth/sso/{provider}/callback` - SSO callback
- `GET /auth/sso/{provider}/metadata` - SSO metadata

### Authorization Endpoints
- `GET /auth/permissions` - Get user permissions
- `POST /auth/permissions/check` - Check specific permission
- `GET /auth/roles` - List available roles
- `POST /auth/roles` - Create role
- `PUT /auth/roles/{id}` - Update role

## Security Considerations

### Token Security
- JWT tokens signed with RS256 (asymmetric)
- Private keys stored in secure key management
- Token payload includes minimal user information
- Automatic token rotation on security events

### Session Security
- Session data encrypted with AES-256
- Secure cookie flags (HttpOnly, Secure, SameSite)
- CSRF protection for state-changing operations
- Session fixation protection

### Rate Limiting
- Login attempts: 5 per minute per IP
- Password reset: 3 per hour per email
- MFA attempts: 10 per minute per user
- API calls: 1000 per minute per user

## Performance Optimizations

### Caching Strategy
- Permission cache in Redis (5-minute TTL)
- Role hierarchy cache (1-hour TTL)
- JWT public key cache (24-hour TTL)
- Session data cache with write-through

### Database Optimizations
- Indexes on email, sso_external_id, tenant_id
- Connection pooling with 20 max connections
- Read replicas for permission checks
- Prepared statements for common queries

## Monitoring and Observability

### Metrics
- Authentication success/failure rates
- Token generation and validation times
- Session creation and expiration rates
- Permission check performance
- MFA usage statistics

### Logging
- All authentication attempts (success/failure)
- Permission changes and role assignments
- Session creation and termination
- Security events (suspicious activity)
- Performance metrics and errors

### Alerts
- High authentication failure rates
- Unusual login patterns
- Permission escalation attempts
- Service performance degradation
- External SSO provider failures

## Testing Strategy

### Unit Tests
- Authentication logic validation
- Permission checking algorithms
- Token generation and validation
- Password hashing and verification
- MFA code generation and validation

### Integration Tests
- Database operations
- Redis session storage
- External SSO provider integration
- API endpoint functionality
- Security policy enforcement

### Security Tests
- Penetration testing for auth flows
- Token manipulation attempts
- Session hijacking prevention
- SQL injection prevention
- Rate limiting effectiveness

## Deployment Considerations

### Environment Configuration
- Separate JWT signing keys per environment
- Environment-specific Redis clusters
- SSO provider configuration per tenant
- Rate limiting thresholds per environment

### Scaling Strategy
- Horizontal scaling with load balancing
- Redis cluster for session storage
- Database read replicas for performance
- CDN for static assets (QR codes, etc.)

### Disaster Recovery
- JWT key backup and rotation procedures
- Session data backup and restoration
- Database backup and point-in-time recovery
- SSO provider failover configuration