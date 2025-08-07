# 🎯 AI Coding Context Engine

## Quick Context Injection

This engine provides essential context for AI coding sessions on ADX Core.

## 🏗️ Architecture Summary

```yaml
Platform: ADX Core Multi-tenant SaaS
Language: Rust (latest stable)
Architecture: Temporal-first microservices
Database: PostgreSQL (multi-tenant schema)
Cache: Redis
Workflows: Temporal.io
Authentication: JWT with RBAC
Security Model: Zero-trust multi-tenancy
Performance Target: Sub-100ms API responses
```

## 🔧 Service Topology

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   API Gateway   │────▶│  Auth Service   │────▶│   User Service  │
│   Port: 8080    │     │   Port: 8081    │     │   Port: 8082    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                       │                       │
         ▼                       ▼                       ▼
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│  File Service   │     │Workflow Service │     │   Temporal      │
│   Port: 8083    │     │   Port: 8084    │     │   Port: 8088    │
└─────────────────┘     └─────────────────┘     └─────────────────┘
         │                       │                       │
         └───────────────────────┼───────────────────────┘
                                 ▼
                    ┌─────────────────┐     ┌─────────────────┐
                    │   PostgreSQL    │     │      Redis      │
                    │   Port: 5432    │     │   Port: 6379    │
                    └─────────────────┘     └─────────────────┘
```

## 🚀 Quick Start Context

```bash
# Start development environment
./scripts/dev-start.sh

# Development URLs
API Gateway: http://localhost:8080
Auth Service: http://localhost:8081  
User Service: http://localhost:8082
File Service: http://localhost:8083
Workflow Service: http://localhost:8084
Temporal UI: http://localhost:8088
```

## 🏛️ Core Patterns

### 1. Multi-Tenant Database Query Pattern
```rust
// ALWAYS include tenant_id for isolation
async fn get_user_by_id(
    pool: &PgPool, 
    user_id: Uuid, 
    tenant_id: Uuid
) -> Result<User, UserError> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE id = $1 AND tenant_id = $2",
        user_id,
        tenant_id
    )
    .fetch_one(pool)
    .await
    .map_err(UserError::Database)
}
```

### 2. Temporal Workflow Pattern
```rust
// Business logic = Temporal workflow
#[temporal_sdk::workflow]
pub async fn user_onboarding_workflow(
    input: UserOnboardingInput,
) -> WorkflowResult<UserOnboardingOutput> {
    // Execute activities with retries and timeouts
    let user = execute_activity!(
        create_user_activity,
        input.user_data,
        ActivityOptions::default()
    ).await?;
    
    let welcome_sent = execute_activity!(
        send_welcome_email_activity,
        user.email.clone(),
        ActivityOptions::default()
    ).await?;
    
    Ok(UserOnboardingOutput {
        user_id: user.id,
        welcome_email_sent: welcome_sent,
    })
}
```

### 3. API Handler Pattern
```rust
// Standard API handler with auth and error handling
#[axum::debug_handler]
pub async fn create_user(
    State(app_state): State<AppState>,
    claims: JwtClaims,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<CreateUserResponse>, ApiError> {
    // Validate request
    request.validate()?;
    
    // Start Temporal workflow for business logic
    let workflow_handle = app_state
        .temporal_client
        .start_workflow(
            WorkflowOptions::default()
                .task_queue("user-service")
                .workflow_id(format!("user-onboarding-{}", Uuid::new_v4())),
            "user_onboarding_workflow",
            UserOnboardingInput {
                user_data: request.into(),
                tenant_id: claims.tenant_id,
                created_by: claims.user_id,
            },
        )
        .await?;
    
    // Return workflow reference
    Ok(Json(CreateUserResponse {
        workflow_id: workflow_handle.workflow_id,
        status: "started".to_string(),
    }))
}
```

### 4. Error Handling Pattern
```rust
// Comprehensive error types
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("User not found")]
    NotFound,
    
    #[error("User already exists")]
    AlreadyExists,
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Unauthorized access")]
    Unauthorized,
    
    #[error("Temporal workflow error: {0}")]
    Workflow(#[from] temporal_client::WorkflowError),
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            UserError::NotFound => (StatusCode::NOT_FOUND, "User not found"),
            UserError::AlreadyExists => (StatusCode::CONFLICT, "User already exists"),
            UserError::Validation(msg) => (StatusCode::BAD_REQUEST, msg.as_str()),
            UserError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            UserError::Database(_) | UserError::Workflow(_) => {
                tracing::error!("Internal error: {}", self);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error")
            }
        };
        
        (status, Json(ErrorResponse { error: error_message.to_string() })).into_response()
    }
}
```

## 📊 Performance Context

### Response Time Targets
- API endpoints: < 100ms (p95)
- Database queries: < 10ms (p95)
- Workflow activities: < 50ms (p95)
- Cache operations: < 1ms (p95)

### Resource Limits
- Memory per service: < 512MB
- CPU per service: < 1 core
- Database connections: < 10 per service
- Redis connections: < 5 per service

## 🛡️ Security Context

### Multi-Tenant Isolation
```sql
-- ALWAYS include tenant_id in queries
CREATE POLICY tenant_isolation ON users
    FOR ALL TO app_role
    USING (tenant_id = current_setting('app.current_tenant_id')::uuid);
```

### JWT Validation
```rust
// Required JWT claims
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    pub sub: Uuid,           // user_id
    pub tenant_id: Uuid,     // tenant isolation
    pub roles: Vec<String>,  // RBAC permissions
    pub exp: i64,           // expiration
    pub iat: i64,           // issued at
}
```

### Rate Limiting
```rust
// Default rate limits
const DEFAULT_RATE_LIMIT: u32 = 1000; // requests per minute
const ADMIN_RATE_LIMIT: u32 = 5000;   // for admin users
const PUBLIC_RATE_LIMIT: u32 = 100;   // for unauthenticated
```

## 🧪 Testing Context

### Test Categories Required
1. **Unit Tests**: Individual functions/methods
2. **Integration Tests**: Service interactions
3. **Workflow Tests**: Temporal workflow execution
4. **API Tests**: HTTP endpoint testing
5. **Security Tests**: Auth/authz validation
6. **Performance Tests**: Load testing

### Test Environment
```rust
// Standard test setup
pub struct TestEnvironment {
    pub db_pool: PgPool,
    pub redis_client: redis::Client,
    pub temporal_client: TemporalClient,
    pub test_server: TestServer,
}

impl TestEnvironment {
    pub async fn setup() -> Self {
        // Setup test containers and services
    }
    
    pub async fn create_test_tenant(&self) -> Tenant {
        // Create isolated test tenant
    }
    
    pub async fn cleanup(&self) {
        // Clean up test data
    }
}
```

## 🎯 Development Workflow

### Step-by-Step Development Process
1. **Analyze Requirements**: Read specs, understand business logic
2. **Design Workflow**: Map business process to Temporal workflows
3. **Implement Activities**: Create atomic, testable activities
4. **Build API Layer**: Create HTTP handlers with validation
5. **Add Database Layer**: Implement queries with tenant isolation
6. **Write Tests**: Unit, integration, and workflow tests
7. **Add Monitoring**: Logging, metrics, and health checks
8. **Document**: Update API docs and architectural decisions

### File Organization
```
services/[service-name]/
├── src/
│   ├── main.rs              # Service entry point
│   ├── handlers/            # HTTP request handlers
│   ├── workflows/           # Temporal workflows
│   ├── activities/          # Temporal activities
│   ├── models/              # Data models and validation
│   ├── database/            # Database queries and migrations
│   ├── auth/                # Authentication/authorization
│   └── config.rs            # Configuration management
├── tests/
│   ├── integration/         # Integration tests
│   ├── workflows/           # Workflow tests
│   └── api/                 # API tests
└── Cargo.toml
```

## 🔄 Temporal Workflow Guidelines

### When to Use Workflows
- Multi-step business processes
- Long-running operations (> 1 minute)
- Operations requiring reliability guarantees
- Cross-service orchestration
- State management for complex processes

### When NOT to Use Workflows
- Simple CRUD operations
- Real-time operations (< 100ms)
- Stateless transformations
- Direct database queries

## 💡 Pro Tips

1. **Always Think Multi-Tenant**: Every query, every cache key, every log entry should include tenant context
2. **Workflows for Business Logic**: If it's more complex than a simple CRUD operation, it should be a workflow
3. **Fail Fast**: Validate input at the API boundary, fail early with clear error messages
4. **Monitor Everything**: Add metrics, logs, and health checks for every component
5. **Test Like Production**: Use real databases, real workflows, real integrations in tests
6. **Security by Default**: Implement authentication, authorization, and audit logging from day one

This context provides the foundation for building enterprise-grade features on ADX Core. Reference the specific rule files in `/rules/` for detailed implementation guidance.
