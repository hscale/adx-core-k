# 🛡️ Security Rules - Zero Trust Multi-Tenancy

## Core Security Principle

> **"Trust nothing. Verify everything. Isolate by default."**

Every line of code must assume it's under attack and protect against multi-tenant data leakage.

## 🎯 Multi-Tenant Isolation

### 1. Database Query Isolation
```rust
// ❌ CRITICAL SECURITY VIOLATION - NEVER DO THIS
async fn get_user_dangerous(user_id: Uuid) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1",  // Missing tenant_id check!
        user_id
    ).fetch_one(&db).await
}

// ✅ REQUIRED: Always include tenant_id in queries
async fn get_user_secure(user_id: Uuid, tenant_id: Uuid) -> Result<User> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2",
        user_id,
        tenant_id
    ).fetch_one(&db).await
}

// ✅ MANDATORY: Tenant isolation middleware
async fn enforce_tenant_isolation(
    req: &mut Request,
    claims: &JwtClaims,
) -> Result<(), SecurityError> {
    // Extract tenant_id from path or body
    let requested_tenant = extract_tenant_from_request(req).await?;
    
    // Verify user has access to this tenant
    if claims.tenant_id != requested_tenant {
        tracing::warn!(
            user_id = %claims.user_id,
            user_tenant = %claims.tenant_id,
            requested_tenant = %requested_tenant,
            "Tenant isolation violation attempt"
        );
        return Err(SecurityError::TenantViolation);
    }
    
    Ok(())
}
```

### 2. Row-Level Security (RLS)
```sql
-- ✅ REQUIRED: Enable RLS on ALL multi-tenant tables
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE files ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflows ENABLE ROW LEVEL SECURITY;

-- ✅ REQUIRED: Create tenant isolation policies
CREATE POLICY tenant_isolation_users ON users
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

CREATE POLICY tenant_isolation_files ON files
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);

-- ✅ REQUIRED: Set tenant context in every connection
SET SESSION app.current_tenant_id = 'tenant-uuid-here';
```

### 3. API Gateway Tenant Enforcement
```rust
// ✅ REQUIRED: Tenant validation in API Gateway
#[workflow]
pub async fn api_request_workflow(
    ctx: &mut WfContext,
    request: APIRequest,
) -> WorkflowResult<APIResponse> {
    // Step 1: Authentication (get JWT claims)
    let auth_result = ctx.activity(authenticate_request_activity, request.clone()).await?;
    
    // Step 2: CRITICAL - Tenant isolation check
    let tenant_check = ctx.activity(
        validate_tenant_access_activity,
        TenantAccessCheck {
            user_id: auth_result.user_id,
            user_tenant_id: auth_result.tenant_id,
            requested_tenant_id: request.tenant_id,
            resource_path: request.path.clone(),
        }
    ).await?;
    
    if !tenant_check.is_allowed {
        tracing::security_event!(
            event = "tenant_violation_blocked",
            user_id = %auth_result.user_id,
            user_tenant = %auth_result.tenant_id,
            requested_tenant = %request.tenant_id,
            resource = %request.path
        );
        return Ok(APIResponse::forbidden("Tenant access denied"));
    }
    
    // Continue with request processing...
}
```

## 🔐 Authentication & Authorization

### 1. JWT Security Standards
```rust
// ✅ REQUIRED: Secure JWT configuration
pub struct JwtConfig {
    // MUST use RS256 (asymmetric) for production
    pub algorithm: Algorithm::RS256,
    
    // MUST be at least 2048 bits
    pub rsa_key_size: usize = 2048,
    
    // Short-lived access tokens
    pub access_token_expiry: Duration = Duration::minutes(15),
    
    // Longer-lived refresh tokens
    pub refresh_token_expiry: Duration = Duration::days(30),
    
    // Token issuer (your domain)
    pub issuer: String,
    
    // Valid audiences
    pub audiences: Vec<String>,
}

// ✅ REQUIRED: Comprehensive JWT validation
pub async fn validate_jwt_token(
    token: &str,
    config: &JwtConfig,
) -> Result<JwtClaims, AuthError> {
    let validation = Validation {
        algorithms: vec![Algorithm::RS256],
        iss: Some(config.issuer.clone()),
        aud: Some(config.audiences.clone()),
        leeway: 30, // 30 second clock skew tolerance
        validate_exp: true,
        validate_nbf: true,
        ..Default::default()
    };
    
    let token_data = decode::<JwtClaims>(
        token,
        &config.decoding_key,
        &validation,
    ).map_err(|e| {
        tracing::warn!(error = %e, "JWT validation failed");
        AuthError::InvalidToken
    })?;
    
    // Additional custom validations
    validate_token_claims(&token_data.claims)?;
    
    Ok(token_data.claims)
}

// ✅ REQUIRED: Validate token claims
fn validate_token_claims(claims: &JwtClaims) -> Result<(), AuthError> {
    // Check if user is still active
    if claims.is_suspended {
        return Err(AuthError::UserSuspended);
    }
    
    // Check if tenant is still active
    if claims.tenant_status != TenantStatus::Active {
        return Err(AuthError::TenantInactive);
    }
    
    // Validate token type
    if claims.token_type != TokenType::Access {
        return Err(AuthError::InvalidTokenType);
    }
    
    Ok(())
}
```

### 2. RBAC Security
```rust
// ✅ REQUIRED: Secure permission checking
#[activity]
pub async fn check_permission_activity(
    input: PermissionCheck,
) -> Result<PermissionResult, ActivityError> {
    // Step 1: Validate tenant context
    if input.user_tenant_id != input.resource_tenant_id {
        tracing::security_event!(
            event = "cross_tenant_permission_attempt",
            user_id = %input.user_id,
            user_tenant = %input.user_tenant_id,
            resource_tenant = %input.resource_tenant_id,
            resource = %input.resource,
            action = %input.action
        );
        return Ok(PermissionResult::denied("Cross-tenant access denied"));
    }
    
    // Step 2: Get user roles (with caching)
    let roles = get_user_roles_cached(input.user_id, input.user_tenant_id).await?;
    
    // Step 3: Get role permissions
    let permissions = get_role_permissions_cached(&roles).await?;
    
    // Step 4: Check if permission is granted
    let is_allowed = permissions.iter().any(|p| {
        p.resource_pattern.matches(&input.resource) &&
        p.actions.contains(&input.action) &&
        p.evaluate_conditions(&input.context)
    });
    
    // Step 5: Audit log the permission check
    audit_log_permission_check(input, is_allowed).await?;
    
    Ok(PermissionResult {
        is_allowed,
        matched_permissions: permissions,
        audit_id: Uuid::new_v4(),
    })
}

// ✅ REQUIRED: Permission caching with security
async fn get_user_roles_cached(
    user_id: Uuid,
    tenant_id: Uuid,
) -> Result<Vec<Role>, ActivityError> {
    let cache_key = format!("user_roles:{}:{}", user_id, tenant_id);
    
    // Try cache first (5 minute TTL for security)
    if let Ok(cached_roles) = REDIS.get::<Vec<Role>>(&cache_key).await {
        return Ok(cached_roles);
    }
    
    // Fetch from database with tenant isolation
    let roles = sqlx::query_as!(
        Role,
        r#"
        SELECT r.* FROM roles r
        JOIN user_roles ur ON ur.role_id = r.id
        WHERE ur.user_id = $1 
        AND ur.tenant_id = $2 
        AND r.tenant_id = $2
        AND ur.is_active = true
        AND (ur.expires_at IS NULL OR ur.expires_at > NOW())
        "#,
        user_id,
        tenant_id
    ).fetch_all(&DB).await?;
    
    // Cache for 5 minutes (security vs performance balance)
    REDIS.set_with_ttl(&cache_key, &roles, 300).await?;
    
    Ok(roles)
}
```

## 🔒 Data Protection

### 1. Encryption Standards
```rust
// ✅ REQUIRED: Strong encryption for sensitive data
use aes_gcm::{Aes256Gcm, Key, Nonce};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::rngs::OsRng;

pub struct EncryptionService {
    cipher: Aes256Gcm,
    argon2: Argon2<'static>,
}

impl EncryptionService {
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: Aes256Gcm::new(Key::from_slice(key)),
            argon2: Argon2::default(),
        }
    }
    
    // ✅ REQUIRED: Encrypt sensitive data at rest
    pub fn encrypt_sensitive_data(&self, data: &str) -> Result<EncryptedData, CryptoError> {
        let nonce = Nonce::from_slice(&generate_nonce());
        let ciphertext = self.cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|_| CryptoError::EncryptionFailed)?;
        
        Ok(EncryptedData {
            ciphertext,
            nonce: nonce.to_vec(),
            algorithm: "AES-256-GCM".to_string(),
        })
    }
    
    // ✅ REQUIRED: Secure password hashing
    pub fn hash_password(&self, password: &str) -> Result<String, CryptoError> {
        // Generate random salt
        let salt = generate_salt();
        
        // Hash with Argon2id (recommended for password hashing)
        let password_hash = self.argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| CryptoError::HashingFailed)?;
        
        Ok(password_hash.to_string())
    }
    
    // ✅ REQUIRED: Constant-time password verification
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool, CryptoError> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| CryptoError::InvalidHash)?;
        
        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(_) => Ok(false),
        }
    }
}
```

### 2. PII Data Handling
```rust
// ✅ REQUIRED: PII data types with automatic protection
#[derive(Debug, Clone)]
pub struct PiiString {
    encrypted_value: EncryptedData,
    classification: DataClassification,
}

#[derive(Debug, Clone)]
pub enum DataClassification {
    Public,
    Internal,
    Confidential,
    Restricted,
}

impl PiiString {
    pub fn new(value: &str, classification: DataClassification) -> Result<Self, CryptoError> {
        let encrypted_value = ENCRYPTION_SERVICE.encrypt_sensitive_data(value)?;
        Ok(Self { encrypted_value, classification })
    }
    
    pub fn decrypt(&self, authorized_user: Uuid, purpose: AccessPurpose) -> Result<String, CryptoError> {
        // Audit access to PII
        audit_pii_access(authorized_user, purpose, &self.classification).await?;
        
        ENCRYPTION_SERVICE.decrypt_sensitive_data(&self.encrypted_value)
    }
}

// ✅ REQUIRED: Automatic PII redaction in logs
impl std::fmt::Display for PiiString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[REDACTED-{}]", self.classification.as_str())
    }
}

// ✅ REQUIRED: PII in database models
#[derive(Debug, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    
    // PII fields are encrypted
    pub email: PiiString,
    pub name: PiiString,
    pub phone: Option<PiiString>,
    
    // Non-PII fields
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
```

## 🚨 Security Monitoring

### 1. Security Event Logging
```rust
// ✅ REQUIRED: Security event macro
macro_rules! security_event {
    ($($key:ident = $value:expr),* $(,)?) => {
        tracing::warn!(
            target: "security",
            event_type = "security_event",
            timestamp = %chrono::Utc::now(),
            $($key = $value),*
        );
    };
}

// ✅ REQUIRED: Critical security events to monitor
pub async fn monitor_security_events() {
    // Failed authentication attempts
    security_event!(
        event = "auth_failure",
        user_id = %user_id,
        ip_address = %request_ip,
        user_agent = %user_agent,
        reason = "invalid_credentials"
    );
    
    // Suspicious tenant access patterns
    security_event!(
        event = "tenant_violation_attempt",
        user_id = %user_id,
        user_tenant = %user_tenant_id,
        requested_tenant = %requested_tenant_id,
        endpoint = %request.path
    );
    
    // Privilege escalation attempts
    security_event!(
        event = "privilege_escalation_attempt",
        user_id = %user_id,
        current_roles = ?current_roles,
        requested_permission = %permission,
        resource = %resource
    );
    
    // Data export activities
    security_event!(
        event = "data_export",
        user_id = %user_id,
        tenant_id = %tenant_id,
        export_type = %export_type,
        record_count = record_count,
        file_size = file_size
    );
}
```

### 2. Rate Limiting & DDoS Protection
```rust
// ✅ REQUIRED: Multi-level rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    // Per-user limits
    pub user_requests_per_minute: u32,
    pub user_requests_per_hour: u32,
    
    // Per-tenant limits
    pub tenant_requests_per_minute: u32,
    pub tenant_requests_per_hour: u32,
    
    // Per-IP limits (DDoS protection)
    pub ip_requests_per_minute: u32,
    pub ip_requests_per_second: u32,
    
    // Endpoint-specific limits
    pub auth_attempts_per_minute: u32,
    pub file_uploads_per_hour: u32,
}

#[activity]
pub async fn check_rate_limits_activity(
    input: RateLimitCheck,
) -> Result<RateLimitResult, ActivityError> {
    let config = get_rate_limit_config().await?;
    
    // Check multiple rate limit dimensions
    let checks = vec![
        check_user_rate_limit(input.user_id, &config).await?,
        check_tenant_rate_limit(input.tenant_id, &config).await?,
        check_ip_rate_limit(input.ip_address, &config).await?,
        check_endpoint_rate_limit(input.endpoint, &config).await?,
    ];
    
    // Find the most restrictive limit
    let most_restrictive = checks.into_iter()
        .min_by_key(|c| c.remaining_requests)
        .unwrap();
    
    if most_restrictive.is_exceeded {
        security_event!(
            event = "rate_limit_exceeded",
            user_id = %input.user_id,
            tenant_id = %input.tenant_id,
            ip_address = %input.ip_address,
            endpoint = %input.endpoint,
            limit_type = %most_restrictive.limit_type
        );
    }
    
    Ok(most_restrictive)
}
```

## 🔍 Security Testing

### 1. Security Test Requirements
```rust
#[cfg(test)]
mod security_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tenant_isolation_enforcement() {
        let test_env = setup_security_test_env().await;
        
        // Create two tenants with users
        let tenant1 = test_env.create_tenant().await;
        let tenant2 = test_env.create_tenant().await;
        let user1 = test_env.create_user(tenant1.id).await;
        let user2 = test_env.create_user(tenant2.id).await;
        
        // User 1 tries to access User 2's data (different tenant)
        let token1 = test_env.generate_token(user1.id, tenant1.id).await;
        
        let response = test_env.client
            .get(&format!("/api/v1/users/{}", user2.id))
            .header("Authorization", format!("Bearer {}", token1))
            .send()
            .await;
        
        // Should be forbidden
        assert_eq!(response.status(), 403);
        
        // Check security event was logged
        let events = test_env.get_security_events().await;
        assert!(events.iter().any(|e| e.event_type == "tenant_violation_attempt"));
    }
    
    #[tokio::test]
    async fn test_sql_injection_prevention() {
        let test_env = setup_security_test_env().await;
        
        // Attempt SQL injection in user creation
        let malicious_email = "test@example.com'; DROP TABLE users; --";
        
        let response = test_env.client
            .post("/api/v1/users")
            .json(&CreateUserRequest {
                email: malicious_email.to_string(),
                name: "Test".to_string(),
                password: "password".to_string(),
                tenant_id: None,
            })
            .send()
            .await;
        
        // Should fail validation, not affect database
        assert_eq!(response.status(), 400);
        
        // Verify users table still exists
        let user_count = test_env.count_users().await;
        assert!(user_count >= 0); // Should not have been dropped
    }
    
    #[tokio::test]
    async fn test_jwt_token_validation() {
        let test_env = setup_security_test_env().await;
        
        // Test expired token
        let expired_token = test_env.generate_expired_token().await;
        let response = test_env.make_authenticated_request(expired_token).await;
        assert_eq!(response.status(), 401);
        
        // Test tampered token
        let tampered_token = test_env.tamper_with_token().await;
        let response = test_env.make_authenticated_request(tampered_token).await;
        assert_eq!(response.status(), 401);
        
        // Test wrong issuer
        let wrong_issuer_token = test_env.generate_token_wrong_issuer().await;
        let response = test_env.make_authenticated_request(wrong_issuer_token).await;
        assert_eq!(response.status(), 401);
    }
}
```

## ⚡ Security Performance

### 1. Security vs Performance Balance
```rust
// ✅ REQUIRED: Optimized security checks
pub struct SecurityCache {
    // Cache permission checks for 5 minutes
    permission_cache: Arc<RwLock<LruCache<String, PermissionResult>>>,
    
    // Cache role lookups for 10 minutes
    role_cache: Arc<RwLock<LruCache<String, Vec<Role>>>>,
    
    // Rate limit windows
    rate_limit_windows: Arc<RwLock<HashMap<String, RateLimitWindow>>>,
}

impl SecurityCache {
    // ✅ Fast permission check with caching
    pub async fn check_permission_cached(
        &self,
        user_id: Uuid,
        tenant_id: Uuid,
        resource: &str,
        action: &str,
    ) -> Result<bool, SecurityError> {
        let cache_key = format!("{}:{}:{}:{}", user_id, tenant_id, resource, action);
        
        // Check cache first (must be fast - < 1ms)
        {
            let cache = self.permission_cache.read().await;
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok(cached_result.is_allowed);
            }
        }
        
        // Fallback to full check
        let result = self.check_permission_full(user_id, tenant_id, resource, action).await?;
        
        // Cache the result
        {
            let mut cache = self.permission_cache.write().await;
            cache.put(cache_key, result.clone());
        }
        
        Ok(result.is_allowed)
    }
}
```

---

**Security is not optional. Every feature must pass these security requirements!** 🛡️
