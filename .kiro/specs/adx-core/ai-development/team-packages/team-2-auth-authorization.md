# Team 2: Authentication & Authorization - AI Development Package

## Team Mission
Build the security foundation for ADX CORE. Every user interaction flows through your systems - make them bulletproof and fast.

## Core AI Rules for Security Development

### Rule 1: Security by Default
```
NEVER trust input - validate everything
ALWAYS use secure defaults
ENCRYPT sensitive data at rest and in transit
LOG all authentication and authorization events
```

### Rule 2: Multi-Provider Authentication
```
SUPPORT: OAuth2, SAML, LDAP, local auth
PATTERN: Provider → Normalizer → JWT → Session
REQUIREMENT: Each provider must map to common user model
```

### Rule 3: Tenant-Aware Authorization
```
EVERY permission check MUST include tenant context
USERS can only access their tenant's resources
ADMINS have elevated permissions within their tenant
SUPER_ADMINS can access multiple tenants
```

### Rule 4: Performance-First Security
```
CACHE permissions aggressively (with TTL)
USE efficient data structures for role checks
MINIMIZE database queries in auth paths
TARGET: <10ms for permission evaluation
```

## Your Specific Deliverables

### 1. Authentication Service
```rust
// YOU MUST DELIVER: Multi-provider authentication
pub trait AuthenticationProvider: Send + Sync {
    async fn authenticate(&self, credentials: Credentials) -> Result<AuthResult, AuthError>;
    async fn refresh_token(&self, refresh_token: &str) -> Result<TokenPair, AuthError>;
    async fn validate_token(&self, token: &str) -> Result<Claims, AuthError>;
}

// REQUIRED IMPLEMENTATIONS:
- OAuth2Provider (Google, Microsoft, GitHub)
- SAMLProvider (Enterprise SSO)
- LDAPProvider (Active Directory)
- LocalProvider (username/password)
- MFAProvider (TOTP, SMS)
```

### 2. Authorization Service
```rust
// YOU MUST DELIVER: Permission evaluation engine
pub trait AuthorizationService: Send + Sync {
    async fn check_permission(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
        resource: &str,
        action: &str,
    ) -> Result<bool, AuthzError>;
    
    async fn get_user_permissions(
        &self,
        user_id: Uuid,
        tenant_id: TenantId,
    ) -> Result<Vec<Permission>, AuthzError>;
}

// REQUIRED FEATURES:
- Role-based access control (RBAC)
- Resource-level permissions
- Permission caching with invalidation
- Tenant-aware permission evaluation
```

### 3. User Management
```rust
// YOU MUST DELIVER: User lifecycle management
pub struct UserService {
    // Must handle: registration, profiles, lifecycle
}

// REQUIRED OPERATIONS:
- User registration with email verification
- Profile management and preferences
- Password policies and security
- Account activation and deactivation
- User search and directory
```

### 4. Tenant Management
```rust
// YOU MUST DELIVER: Tenant provisioning and management
pub struct TenantService {
    // Must handle: tenant creation, configuration, billing context
}

// REQUIRED OPERATIONS:
- Tenant provisioning workflows
- Tenant configuration management
- Billing context and limits
- Tenant isolation enforcement
```

## AI Development Prompts

### Authentication Service Prompt
```
ROLE: Senior security engineer building enterprise authentication systems

TASK: Create multi-provider authentication service for ADX CORE

REQUIREMENTS:
- Support OAuth2, SAML, LDAP, and local authentication
- JWT token generation with secure claims
- Multi-factor authentication (TOTP, SMS)
- Session management with refresh tokens
- Tenant-aware authentication flows

CONSTRAINTS:
- Use industry-standard security libraries
- Follow OWASP security guidelines
- Include comprehensive audit logging
- Support rate limiting and brute force protection
- Implement secure password policies

DELIVERABLES:
1. Multi-provider authentication framework
2. JWT token management with refresh
3. MFA implementation with multiple methods
4. Session management with security
5. Comprehensive security audit logging

CODE STRUCTURE:
```rust
// src/auth/mod.rs
pub mod providers;
pub mod jwt;
pub mod mfa;
pub mod session;
pub mod audit;

// Usage example:
let auth_result = auth_service
    .authenticate(provider, credentials)
    .await?;
```

Generate secure, production-ready authentication code following security best practices.
```

### Authorization Service Prompt
```
ROLE: Authorization expert building high-performance access control systems

TASK: Create authorization service with RBAC for ADX CORE

REQUIREMENTS:
- Role-based access control with hierarchical roles
- Resource-level permission evaluation
- Tenant-aware permission isolation
- High-performance permission caching
- Policy-based authorization rules

CONSTRAINTS:
- Optimize for <10ms permission evaluation
- Support complex permission hierarchies
- Include comprehensive audit trails
- Handle permission updates in real-time
- Support bulk permission operations

DELIVERABLES:
1. RBAC implementation with role hierarchy
2. High-performance permission evaluation
3. Tenant-aware access control
4. Permission caching with invalidation
5. Policy engine for complex rules

CODE STRUCTURE:
```rust
// src/authz/mod.rs
pub mod rbac;
pub mod permissions;
pub mod cache;
pub mod policies;
pub mod evaluator;

// Usage example:
let has_permission = authz_service
    .check_permission(user_id, tenant_id, "files", "read")
    .await?;
```

Generate high-performance authorization code with comprehensive access control.
```

### User Management Prompt
```
ROLE: User management expert building scalable identity systems

TASK: Create user management service for ADX CORE

REQUIREMENTS:
- User registration with email verification
- Profile management and preferences
- Password policies and security measures
- User lifecycle management (activate, suspend, delete)
- User directory and search capabilities

CONSTRAINTS:
- Support multiple authentication providers
- Include comprehensive user audit trails
- Handle user data privacy requirements
- Support bulk user operations
- Implement user preference management

DELIVERABLES:
1. User registration and verification workflows
2. Profile management with preferences
3. Password policy enforcement
4. User lifecycle management
5. User directory with search and filtering

CODE STRUCTURE:
```rust
// src/users/mod.rs
pub mod registration;
pub mod profiles;
pub mod lifecycle;
pub mod directory;
pub mod preferences;

// Usage example:
let user = user_service
    .register_user(tenant_id, registration_data)
    .await?;
```

Generate comprehensive user management system with security and privacy.
```

### Tenant Management Prompt
```
ROLE: Multi-tenancy expert building tenant provisioning systems

TASK: Create tenant management service for ADX CORE

REQUIREMENTS:
- Tenant provisioning with automated setup
- Tenant configuration and customization
- Billing context and usage tracking
- Tenant isolation enforcement
- Tenant lifecycle management

CONSTRAINTS:
- Ensure complete tenant data isolation
- Support tenant-specific configurations
- Include billing integration hooks
- Handle tenant migration scenarios
- Support tenant analytics and reporting

DELIVERABLES:
1. Tenant provisioning workflows
2. Configuration management system
3. Billing context and tracking
4. Isolation enforcement mechanisms
5. Tenant analytics and reporting

CODE STRUCTURE:
```rust
// src/tenants/mod.rs
pub mod provisioning;
pub mod configuration;
pub mod billing;
pub mod isolation;
pub mod analytics;

// Usage example:
let tenant = tenant_service
    .provision_tenant(tenant_config)
    .await?;
```

Generate robust tenant management system with complete isolation.
```

## Success Criteria

### Authentication Service ✅
- [ ] Multi-provider authentication working
- [ ] JWT tokens generated and validated correctly
- [ ] MFA flows complete successfully
- [ ] Session management with refresh tokens
- [ ] Performance: <100ms authentication time

### Authorization Service ✅
- [ ] Permission evaluation <10ms (95th percentile)
- [ ] RBAC with hierarchical roles working
- [ ] Tenant isolation enforced
- [ ] Permission caching effective (>90% hit rate)
- [ ] Audit logging captures all decisions

### User Management ✅
- [ ] User registration with email verification
- [ ] Profile management functional
- [ ] Password policies enforced
- [ ] User lifecycle operations working
- [ ] User directory search performing well

### Tenant Management ✅
- [ ] Tenant provisioning automated
- [ ] Configuration management working
- [ ] Billing context tracking accurate
- [ ] Tenant isolation verified
- [ ] Analytics providing insights

## Integration Points

### What You Provide to Other Teams
```yaml
authentication:
  provides_to: [team_3, team_4, team_5, team_6, team_7, team_8, team_9]
  interface: JWT middleware + user context
  
authorization:
  provides_to: [team_3, team_4, team_5, team_6, team_7, team_8, team_9]
  interface: Permission checking + role evaluation
  
user_management:
  provides_to: [team_6, team_7, team_9]
  interface: User CRUD operations + directory
  
tenant_management:
  provides_to: [team_3, team_4, team_5, team_6, team_7, team_9]
  interface: Tenant context + configuration
```

### Dependencies
```yaml
requires_from_team_1:
  - Database infrastructure and connection pools
  - API gateway for middleware integration
  - Observability for security audit logging
```

## Quality Standards

### Security Requirements
```rust
// MANDATORY: Secure password hashing
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};

pub fn hash_password(password: &str) -> Result<String, AuthError> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

// MANDATORY: JWT with secure claims
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,           // User ID
    pub tenant_id: TenantId, // Tenant context
    pub roles: Vec<String>,  // User roles
    pub exp: i64,           // Expiration
    pub iat: i64,           // Issued at
    pub jti: Uuid,          // JWT ID for revocation
}

// MANDATORY: Audit logging
#[tracing::instrument(fields(user_id = %user_id, tenant_id = %tenant_id))]
pub async fn check_permission(&self, user_id: Uuid, tenant_id: TenantId, resource: &str, action: &str) -> Result<bool, AuthzError> {
    let result = self.evaluate_permission(user_id, tenant_id, resource, action).await?;
    
    // Log authorization decision
    tracing::info!(
        user_id = %user_id,
        tenant_id = %tenant_id,
        resource = resource,
        action = action,
        granted = result,
        "Authorization decision"
    );
    
    Ok(result)
}
```

### Performance Requirements
- Authentication: <100ms end-to-end
- Authorization: <10ms permission check
- JWT validation: <5ms
- Password hashing: Use Argon2 with appropriate cost
- Cache hit rate: >90% for permissions

### Testing Requirements
```rust
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn test_tenant_isolation() {
        // Create users in different tenants
        let tenant1_user = create_test_user(tenant1_id).await;
        let tenant2_user = create_test_user(tenant2_id).await;
        
        // Verify tenant1_user cannot access tenant2 resources
        let result = authz_service
            .check_permission(tenant1_user.id, tenant2_id, "files", "read")
            .await;
        
        assert!(result.is_err() || !result.unwrap());
    }
    
    #[tokio::test]
    async fn test_permission_caching() {
        // First call should hit database
        let start = Instant::now();
        let result1 = authz_service.check_permission(user_id, tenant_id, "files", "read").await?;
        let first_duration = start.elapsed();
        
        // Second call should hit cache
        let start = Instant::now();
        let result2 = authz_service.check_permission(user_id, tenant_id, "files", "read").await?;
        let second_duration = start.elapsed();
        
        assert_eq!(result1, result2);
        assert!(second_duration < first_duration / 2); // Cache should be much faster
    }
}
```

## Timeline
- **Week 1**: Authentication service with multi-provider support
- **Week 2**: Authorization service with RBAC and user/tenant management
- **End of Week 2**: Security foundation ready for all other teams

Security is non-negotiable - build it right! 🔒