# 🏗️ Temporal-First Architecture Rules

## Core Principle: Temporal-First Everything

> **"If it's complex, it's a workflow. If it's simple, it's an activity."**

Every ADX Core service follows the Temporal-First architecture pattern where business logic is implemented as durable, reliable Temporal workflows.

## 🎯 Architectural Mandates

### 1. Workflow-Centric Design
```rust
// ✅ CORRECT: Complex logic as workflow
#[workflow]
pub async fn user_onboarding_workflow(
    ctx: &mut WfContext,
    input: UserOnboardingInput,
) -> WorkflowResult<UserOnboardingOutput> {
    // Step 1: Create user account
    let user = ctx.activity(create_user_activity, input.user_data).await?;
    
    // Step 2: Send welcome email
    ctx.activity(send_welcome_email_activity, user.id).await?;
    
    // Step 3: Setup tenant resources (if needed)
    if input.create_tenant {
        let tenant = ctx.activity(create_tenant_activity, input.tenant_data).await?;
        ctx.activity(assign_user_to_tenant_activity, (user.id, tenant.id)).await?;
    }
    
    // Step 4: Trigger post-onboarding workflow
    ctx.start_child_workflow(post_onboarding_workflow, user.id).await?;
    
    Ok(UserOnboardingOutput { user_id: user.id })
}

// ❌ WRONG: Complex logic in HTTP handler
pub async fn create_user(req: CreateUserRequest) -> Result<UserResponse> {
    let user = create_user_in_db(req).await?;
    send_email(user.email).await?; // This could fail and leave inconsistent state
    Ok(UserResponse::from(user))
}
```

### 2. Activity-Based Operations
```rust
// ✅ CORRECT: Single responsibility activities
#[activity]
pub async fn validate_user_data_activity(
    user_data: UserData,
) -> Result<ValidatedUserData, ActivityError> {
    // Validate email format
    if !is_valid_email(&user_data.email) {
        return Err(ActivityError::ValidationError("Invalid email format".to_string()));
    }
    
    // Check password strength
    if !is_strong_password(&user_data.password) {
        return Err(ActivityError::ValidationError("Password too weak".to_string()));
    }
    
    Ok(ValidatedUserData::from(user_data))
}

#[activity]
pub async fn create_user_in_database_activity(
    validated_data: ValidatedUserData,
) -> Result<User, ActivityError> {
    let db = get_database_connection().await?;
    
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email, password_hash, name, tenant_id) VALUES ($1, $2, $3, $4) RETURNING *",
        validated_data.email,
        hash_password(&validated_data.password)?,
        validated_data.name,
        validated_data.tenant_id
    )
    .fetch_one(&db)
    .await
    .map_err(|e| ActivityError::DatabaseError(e.to_string()))?;
    
    Ok(user)
}
```

### 3. Service Architecture Pattern
```rust
// Required structure for all services
pub struct ServiceConfig {
    pub service_name: String,
    pub port: u16,
    pub database_url: String,
    pub temporal_url: String,
    pub redis_url: String,
}

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub temporal: Arc<TemporalClient>,
    pub cache: Arc<RedisManager>,
    pub config: Arc<ServiceConfig>,
}

// Every service must implement this pattern
pub async fn create_app(config: ServiceConfig) -> Result<Router, AppError> {
    let state = AppState {
        db: Arc::new(DatabaseManager::new(&config.database_url).await?),
        temporal: Arc::new(TemporalClient::new(&config.temporal_url).await?),
        cache: Arc::new(RedisManager::new(&config.redis_url).await?),
        config: Arc::new(config),
    };
    
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/*path", any(handle_api_request))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
        .layer(CorsLayer::permissive());
    
    Ok(app)
}
```

### 4. Multi-Tenant Isolation
```rust
// ✅ ALWAYS enforce tenant isolation
pub async fn check_tenant_access(
    user_id: Uuid,
    tenant_id: Uuid,
    resource_id: Uuid,
) -> Result<bool, AuthError> {
    // Verify user belongs to tenant
    let user_tenant = get_user_tenant(user_id).await?;
    if user_tenant != tenant_id {
        return Ok(false);
    }
    
    // Verify resource belongs to tenant
    let resource_tenant = get_resource_tenant(resource_id).await?;
    if resource_tenant != tenant_id {
        return Ok(false);
    }
    
    Ok(true)
}

// ✅ Tenant context in every workflow
#[workflow]
pub async fn process_file_workflow(
    ctx: &mut WfContext,
    input: FileProcessingInput,
) -> WorkflowResult<FileProcessingOutput> {
    // ALWAYS validate tenant access first
    let access_valid = ctx.activity(
        validate_tenant_access_activity,
        (input.user_id, input.tenant_id, input.file_id)
    ).await?;
    
    if !access_valid {
        return Err(WorkflowError::AccessDenied);
    }
    
    // Continue with business logic...
}
```

## 🛡️ Security Architecture

### 1. Zero-Trust Multi-Tenancy
```rust
// Every database query MUST include tenant_id
// ❌ NEVER do this
let users = sqlx::query!("SELECT * FROM users WHERE id = $1", user_id);

// ✅ ALWAYS do this
let users = sqlx::query!(
    "SELECT * FROM users WHERE id = $1 AND tenant_id = $2",
    user_id,
    tenant_id
);
```

### 2. JWT Token Validation
```rust
// Required middleware for all protected endpoints
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut req: Request<Body>,
    next: Next<Body>,
) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(StatusCode::UNAUTHORIZED)?;
    
    let claims = validate_jwt_token(auth_header)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;
    
    // Add validated claims to request
    req.extensions_mut().insert(claims);
    
    Ok(next.run(req).await)
}
```

## ⚡ Performance Requirements

### 1. Response Time Targets
- **API Responses**: < 100ms (p95)
- **Authentication**: < 50ms (p95)
- **Database Queries**: < 10ms (p95)
- **Workflow Start**: < 200ms (p95)

### 2. Caching Strategy
```rust
// Implement caching for frequently accessed data
pub async fn get_user_permissions(
    user_id: Uuid,
    tenant_id: Uuid,
    cache: &RedisManager,
    db: &DatabaseManager,
) -> Result<Vec<Permission>, AppError> {
    let cache_key = format!("permissions:{}:{}", user_id, tenant_id);
    
    // Try cache first
    if let Ok(cached) = cache.get::<Vec<Permission>>(&cache_key).await {
        return Ok(cached);
    }
    
    // Fallback to database
    let permissions = db.get_user_permissions(user_id, tenant_id).await?;
    
    // Cache for 5 minutes
    cache.set_with_ttl(&cache_key, &permissions, 300).await?;
    
    Ok(permissions)
}
```

### 3. Database Connection Pooling
```rust
// Required database configuration
pub struct DatabaseConfig {
    pub max_connections: u32,      // 10 for development, 100+ for production
    pub min_connections: u32,      // 1 for development, 10+ for production
    pub acquire_timeout: Duration, // 30 seconds
    pub idle_timeout: Duration,    // 10 minutes
}
```

## 🧪 Testing Architecture

### 1. Workflow Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use temporal_sdk_test::WorkflowTestContext;
    
    #[tokio::test]
    async fn test_user_onboarding_workflow() {
        let mut ctx = WorkflowTestContext::new();
        
        // Mock activities
        ctx.mock_activity(create_user_activity, |input: UserData| {
            Ok(User { id: Uuid::new_v4(), email: input.email, ..Default::default() })
        });
        
        ctx.mock_activity(send_welcome_email_activity, |_: Uuid| {
            Ok(EmailSent { message_id: "test".to_string() })
        });
        
        // Execute workflow
        let result = ctx.execute_workflow(
            user_onboarding_workflow,
            UserOnboardingInput {
                user_data: UserData {
                    email: "test@example.com".to_string(),
                    name: "Test User".to_string(),
                    password: "secure_password".to_string(),
                },
                create_tenant: false,
            }
        ).await;
        
        assert!(result.is_ok());
    }
}
```

### 2. Integration Testing
```rust
// Required integration test structure
#[tokio::test]
async fn test_api_endpoint_integration() {
    // Setup test environment
    let test_env = setup_test_environment().await;
    
    // Create test data
    let tenant = test_env.create_test_tenant().await;
    let user = test_env.create_test_user(tenant.id).await;
    let token = test_env.generate_auth_token(user.id, tenant.id).await;
    
    // Test API endpoint
    let response = test_env
        .client
        .post("/api/v1/users")
        .header("Authorization", format!("Bearer {}", token))
        .json(&CreateUserRequest {
            email: "new@example.com".to_string(),
            name: "New User".to_string(),
        })
        .send()
        .await;
    
    assert_eq!(response.status(), 201);
    
    // Verify workflow was triggered
    let workflows = test_env.temporal.list_workflows().await;
    assert!(workflows.iter().any(|w| w.workflow_type == "user_onboarding_workflow"));
    
    // Cleanup
    test_env.cleanup().await;
}
```

## 📚 Documentation Requirements

### 1. Workflow Documentation
```rust
/// User Onboarding Workflow
/// 
/// Orchestrates the complete user registration and setup process.
/// 
/// # Input
/// - `UserOnboardingInput`: User data and configuration
/// 
/// # Steps
/// 1. Validate user data (email, password strength)
/// 2. Create user account in database
/// 3. Send welcome email
/// 4. Setup tenant resources (if creating new tenant)
/// 5. Assign user to tenant
/// 6. Trigger post-onboarding workflows
/// 
/// # Error Handling
/// - Invalid email: Returns `ValidationError`
/// - Duplicate email: Returns `ConflictError`
/// - Email send failure: Retries 3 times, then continues
/// 
/// # Duration
/// Typical execution: 2-5 seconds
/// Maximum duration: 30 seconds (with retries)
#[workflow]
pub async fn user_onboarding_workflow(/* ... */) { /* ... */ }
```

### 2. API Documentation
```rust
/// Create New User
/// 
/// Creates a new user account and triggers the onboarding workflow.
/// 
/// # Authentication
/// Requires valid JWT token with `user:create` permission.
/// 
/// # Rate Limiting
/// 10 requests per minute per user.
/// 
/// # Request Body
/// ```json
/// {
///   "email": "user@example.com",
///   "name": "John Doe",
///   "password": "secure_password",
///   "tenant_id": "optional-tenant-id"
/// }
/// ```
/// 
/// # Response
/// - `201 Created`: User created successfully
/// - `400 Bad Request`: Invalid input data
/// - `409 Conflict`: Email already exists
/// - `429 Too Many Requests`: Rate limit exceeded
pub async fn create_user(/* ... */) { /* ... */ }
```

## 🔧 Code Quality Standards

### 1. Error Handling
```rust
// ✅ Use proper error types
#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("User not found: {user_id}")]
    UserNotFound { user_id: Uuid },
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("Database error: {source}")]
    DatabaseError { #[from] source: sqlx::Error },
    
    #[error("Temporal error: {source}")]
    TemporalError { #[from] source: temporal_sdk::Error },
}

// ✅ Proper error propagation
pub async fn get_user(user_id: Uuid, tenant_id: Uuid) -> Result<User, UserServiceError> {
    let user = sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2",
        user_id,
        tenant_id
    )
    .fetch_optional(&self.db)
    .await?
    .ok_or(UserServiceError::UserNotFound { user_id })?;
    
    Ok(user)
}
```

### 2. Logging and Observability
```rust
use tracing::{info, warn, error, instrument};

#[instrument(skip(self), fields(user_id = %user_id, tenant_id = %tenant_id))]
pub async fn create_user(&self, user_data: CreateUserRequest) -> Result<User, UserServiceError> {
    info!("Creating new user");
    
    let result = sqlx::query_as!(/* ... */)
        .fetch_one(&self.db)
        .await;
    
    match result {
        Ok(user) => {
            info!(user_id = %user.id, "User created successfully");
            Ok(user)
        }
        Err(e) => {
            error!(error = %e, "Failed to create user");
            Err(UserServiceError::DatabaseError { source: e })
        }
    }
}
```

---

**Remember: Every line of code should follow these architectural principles. When in doubt, make it a workflow!** 🚀
