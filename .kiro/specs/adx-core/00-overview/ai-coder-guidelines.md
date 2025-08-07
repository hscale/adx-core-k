# ADX CORE - AI Coder Team Guidelines

## Overview

Comprehensive guidelines for AI Coding Assistants (Kiro, Claude, Gemini, GitHub Copilot) working on ADX CORE development with **Temporal-First Architecture** and **2-week sprint cycles**.

## Core Development Principles

### 1. IDE-Optimized Development Environment
**Run Rust backend and React frontend NATIVELY on macOS for optimal debugging experience.**

#### ✅ Native Development (macOS):
- **Rust Backend**: `cargo run` with full IDE debugging support
- **React Frontend**: `npm run dev` with hot reload and React DevTools
- **IDE Debugging**: Set breakpoints, step through code, inspect variables
- **Fast Compilation**: Native Rust compilation without Docker overhead
- **Hot Reload**: Instant frontend updates without container rebuilds

#### 🐳 Docker Only For Infrastructure:
- **Temporal.io**: Workflow orchestration cluster
- **PostgreSQL**: Database server
- **Redis**: Cache and session storage
- **No application code in Docker during development**

### 2. Temporal-First Mandate
**"If it's more complex than a simple CRUD operation, it MUST be a Temporal workflow."**

#### ✅ ALWAYS Use Temporal Workflows For:
- User registration and authentication flows
- File upload, processing, and sharing operations
- Tenant provisioning and lifecycle management
- Plugin installation, updates, and monitoring
- Multi-step API request processing
- Error recovery and retry scenarios
- Long-running background tasks
- Complex business logic orchestration

#### ❌ NEVER Use Temporal Workflows For:
- Simple database CRUD operations (get, create, update, delete)
- Immediate API responses (health checks, simple queries)
- Pure calculations and data transformations
- Static configuration reading
- Simple validation functions

### 3. IDE-Optimized Development Workflow

#### Daily Development Process
```bash
# 1. Start infrastructure services (Docker)
make infra

# 2. Start Rust backend (native macOS - Terminal 1)
make dev-backend
# This runs: cd backend && cargo run --bin api-gateway
# ✅ Full IDE debugging support with breakpoints

# 3. Start React frontend (native macOS - Terminal 2)  
make dev-frontend
# This runs: cd frontend && npm run dev
# ✅ Hot reload + React DevTools + browser debugging

# 4. Open IDE and set breakpoints in Rust code
# 5. Debug Temporal workflows with IDE + Temporal UI
```

#### IDE Debugging Setup
```json
// .vscode/launch.json - Native Rust debugging
{
  "type": "lldb",
  "request": "launch", 
  "name": "Debug API Gateway",
  "cargo": {
    "args": ["build", "--bin=api-gateway"],
    "filter": {"name": "api-gateway", "kind": "bin"}
  },
  "env": {
    "DATABASE_URL": "postgresql://postgres:postgres@localhost:5432/adx_core",
    "REDIS_URL": "redis://localhost:6379", 
    "TEMPORAL_URL": "localhost:7233",
    "RUST_LOG": "debug"
  }
}
```

### 4. Rust Development Standards

#### Code Quality Requirements
```rust
// ✅ CORRECT: Proper error handling with Result types
pub async fn create_user(user_data: CreateUserRequest) -> Result<User, UserError> {
    // Set breakpoint here - works perfectly in IDE!
    validate_user_data(&user_data)?;
    let user = repository.create_user(user_data).await?;
    Ok(user)
}

// ❌ INCORRECT: Using unwrap() or expect() in production code
pub async fn create_user(user_data: CreateUserRequest) -> User {
    let user = repository.create_user(user_data).await.unwrap(); // DON'T DO THIS
    user
}
```

#### Required Dependencies
```toml
# Cargo.toml - Essential dependencies for all services
[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
sqlx = { version = "0.7", features = ["runtime-tokio-rustls", "postgres", "uuid", "chrono"] }
serde = { version = "1.0", features = ["derive"] }
temporal-sdk = "0.1"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "1.0"
```

### 3. Temporal Workflow Patterns

#### Standard Workflow Structure
```rust
use temporal_sdk::{workflow, activity, WorkflowResult};

#[workflow]
pub async fn example_workflow(
    input: WorkflowInput,
) -> WorkflowResult<WorkflowOutput> {
    // Step 1: Validate input
    validate_input_activity(input.clone()).await?;
    
    // Step 2: Process in parallel if possible
    let (result_a, result_b) = temporal_sdk::join!(
        process_a_activity(input.data_a),
        process_b_activity(input.data_b)
    );
    
    // Step 3: Combine results
    let final_result = combine_results_activity(result_a?, result_b?).await?;
    
    // Step 4: Handle conditional logic
    if final_result.needs_notification {
        send_notification_activity(final_result.clone()).await?;
    }
    
    Ok(WorkflowOutput {
        result: final_result,
        processed_at: temporal_sdk::now(),
    })
}

#[activity]
pub async fn validate_input_activity(
    input: WorkflowInput,
) -> Result<(), ActivityError> {
    if input.data_a.is_empty() {
        return Err(ActivityError::InvalidInput("data_a cannot be empty".to_string()));
    }
    Ok(())
}
```

#### Error Handling Pattern
```rust
// ✅ CORRECT: Let Temporal handle retries
#[activity]
pub async fn external_api_call_activity(
    request: ApiRequest,
) -> Result<ApiResponse, ActivityError> {
    let client = reqwest::Client::new();
    
    let response = client
        .post(&request.url)
        .json(&request.body)
        .send()
        .await
        .map_err(|e| ActivityError::NetworkError(e.to_string()))?;
    
    if response.status().is_success() {
        let data = response.json().await
            .map_err(|e| ActivityError::ParseError(e.to_string()))?;
        Ok(data)
    } else {
        Err(ActivityError::ApiError(response.status().to_string()))
    }
}

// ❌ INCORRECT: Custom retry logic
#[activity]
pub async fn bad_external_api_call_activity(
    request: ApiRequest,
) -> Result<ApiResponse, ActivityError> {
    let mut retries = 0;
    loop {
        match make_api_call(&request).await {
            Ok(response) => return Ok(response),
            Err(e) if retries < 3 => {
                retries += 1;
                tokio::time::sleep(Duration::from_secs(2_u64.pow(retries))).await;
                continue;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Module-Specific Implementation Guidelines

### Authentication Service
```rust
// Required workflows for Auth Service
#[workflow]
pub async fn user_registration_workflow(
    registration_data: UserRegistrationData,
) -> WorkflowResult<User> {
    // Implementation must include:
    // 1. Data validation
    // 2. User creation
    // 3. Email verification with timeout
    // 4. Account activation or cleanup
}

#[workflow]
pub async fn password_reset_workflow(
    email: String,
) -> WorkflowResult<()> {
    // Implementation must include:
    // 1. User lookup
    // 2. Reset token generation
    // 3. Email sending
    // 4. Token expiration handling
}
```

### File Service
```rust
// Required workflows for File Service
#[workflow]
pub async fn file_upload_workflow(
    upload_request: FileUploadRequest,
) -> WorkflowResult<ProcessedFile> {
    // Implementation must include:
    // 1. File validation
    // 2. Virus scanning
    // 3. Metadata extraction
    // 4. Thumbnail generation (if applicable)
    // 5. Storage provider upload
    // 6. Database record creation
}

#[workflow]
pub async fn file_sharing_workflow(
    share_request: FileShareRequest,
) -> WorkflowResult<ShareResult> {
    // Implementation must include:
    // 1. Permission validation
    // 2. Share link generation
    // 3. Notification sending
    // 4. Expiration scheduling
}
```

### Tenant Service
```rust
// Required workflows for Tenant Service
#[workflow]
pub async fn tenant_provisioning_workflow(
    tenant_request: TenantCreationRequest,
) -> WorkflowResult<Tenant> {
    // Implementation must include:
    // 1. Tenant validation
    // 2. Database schema creation
    // 3. Storage bucket setup
    // 4. Default configuration
    // 5. Admin user creation
    // 6. Welcome email sending
}
```

## Database Patterns

### Repository Pattern Implementation
```rust
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: CreateUser) -> Result<User, DatabaseError>;
    async fn get_user_by_id(&self, id: UserId) -> Result<Option<User>, DatabaseError>;
    async fn update_user(&self, id: UserId, updates: UpdateUser) -> Result<User, DatabaseError>;
    async fn delete_user(&self, id: UserId) -> Result<(), DatabaseError>;
}

pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn create_user(&self, user: CreateUser) -> Result<User, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (id, email, password_hash, created_at)
            VALUES ($1, $2, $3, $4)
            RETURNING id, email, password_hash, created_at, updated_at
            "#,
            Uuid::new_v4(),
            user.email,
            user.password_hash,
            Utc::now()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(DatabaseError::from)?;
        
        Ok(user)
    }
}
```

### Multi-Tenant Database Pattern
```rust
// ✅ CORRECT: Tenant-scoped queries
pub async fn get_user_files(
    pool: &PgPool,
    user_id: UserId,
    tenant_id: TenantId,
) -> Result<Vec<File>, DatabaseError> {
    let files = sqlx::query_as!(
        File,
        r#"
        SELECT f.* FROM files f
        JOIN users u ON f.user_id = u.id
        WHERE f.user_id = $1 AND u.tenant_id = $2
        "#,
        user_id,
        tenant_id
    )
    .fetch_all(pool)
    .await?;
    
    Ok(files)
}

// ❌ INCORRECT: Missing tenant isolation
pub async fn get_user_files_bad(
    pool: &PgPool,
    user_id: UserId,
) -> Result<Vec<File>, DatabaseError> {
    let files = sqlx::query_as!(
        File,
        "SELECT * FROM files WHERE user_id = $1",
        user_id
    )
    .fetch_all(pool)
    .await?;
    
    Ok(files)
}
```

## Testing Requirements

### Unit Testing Pattern
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_create_user_success() {
        let mut mock_repo = MockUserRepository::new();
        mock_repo
            .expect_create_user()
            .with(eq(CreateUser {
                email: "test@example.com".to_string(),
                password_hash: "hashed_password".to_string(),
            }))
            .times(1)
            .returning(|_| Ok(User {
                id: Uuid::new_v4(),
                email: "test@example.com".to_string(),
                password_hash: "hashed_password".to_string(),
                created_at: Utc::now(),
                updated_at: Utc::now(),
            }));
        
        let service = UserService::new(Arc::new(mock_repo));
        let result = service.create_user(CreateUser {
            email: "test@example.com".to_string(),
            password_hash: "hashed_password".to_string(),
        }).await;
        
        assert!(result.is_ok());
    }
}
```

### Temporal Workflow Testing
```rust
#[cfg(test)]
mod workflow_tests {
    use super::*;
    use temporal_sdk::testing::WorkflowTestEnv;
    
    #[tokio::test]
    async fn test_user_registration_workflow() {
        let mut env = WorkflowTestEnv::new().await;
        
        // Mock activities
        env.register_activity(validate_registration_activity);
        env.register_activity(create_user_activity);
        env.register_activity(send_verification_email_activity);
        
        // Execute workflow
        let result = env.execute_workflow(
            user_registration_workflow,
            UserRegistrationData {
                email: "test@example.com".to_string(),
                password: "password123".to_string(),
            }
        ).await;
        
        assert!(result.is_ok());
    }
}
```

## API Development Guidelines

### Axum Handler Pattern
```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};

pub fn create_user_routes() -> Router<AppState> {
    Router::new()
        .route("/users", post(create_user_handler))
        .route("/users/:id", get(get_user_handler))
}

pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<User>, ApiError> {
    // Start Temporal workflow for complex operations
    let workflow_id = format!("user-registration-{}", Uuid::new_v4());
    
    let user = state.temporal_client
        .start_workflow(
            workflow_id,
            user_registration_workflow,
            UserRegistrationData {
                email: request.email,
                password: request.password,
            }
        )
        .await
        .map_err(ApiError::from)?;
    
    Ok(Json(user))
}

pub async fn get_user_handler(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<User>, ApiError> {
    // Simple CRUD - direct repository call
    let user = state.user_repository
        .get_user_by_id(user_id)
        .await
        .map_err(ApiError::from)?
        .ok_or(ApiError::NotFound)?;
    
    Ok(Json(user))
}
```

## Frontend Integration Guidelines

### React + TypeScript Patterns
```typescript
// Temporal workflow integration hook
export const useTemporalWorkflow = <T>(workflowType: string) => {
  const [status, setStatus] = useState<WorkflowStatus>('idle');
  const [result, setResult] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);

  const execute = async (data: any) => {
    try {
      setStatus('running');
      setError(null);

      // Start workflow
      const response = await api.post(`/workflows/${workflowType}`, data);
      const { workflowId } = response.data;

      // Monitor progress via WebSocket
      const ws = new WebSocket(`/ws/workflows/${workflowId}/progress`);
      
      ws.onmessage = (event) => {
        const update = JSON.parse(event.data);
        
        if (update.status === 'completed') {
          setResult(update.result);
          setStatus('completed');
          ws.close();
        } else if (update.status === 'failed') {
          setError(update.error);
          setStatus('failed');
          ws.close();
        }
      };

    } catch (err) {
      setError(err.message);
      setStatus('failed');
    }
  };

  return { execute, status, result, error };
};

// Usage in components
export const FileUploadComponent: React.FC = () => {
  const { execute, status, result, error } = useTemporalWorkflow<ProcessedFile>('file-upload');

  const handleUpload = async (files: FileList) => {
    await execute({
      files: Array.from(files).map(f => ({
        name: f.name,
        size: f.size,
        type: f.type,
      })),
      options: {
        virusScan: true,
        generateThumbnails: true,
      },
    });
  };

  return (
    <div>
      <input type="file" multiple onChange={(e) => handleUpload(e.target.files)} />
      {status === 'running' && <div>Uploading...</div>}
      {status === 'completed' && <div>Upload completed!</div>}
      {error && <div>Error: {error}</div>}
    </div>
  );
};
```

## IDE-Optimized Development Environment

### Development Architecture
```
┌─────────────────────────────────────────────────────────────┐
│                    macOS Development                        │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐  │
│  │  Rust Backend   │  │   Frontend      │  │      IDE        │  │
│  │  (Native macOS) │  │  (Native macOS) │  │   Debugging     │  │
│  │                 │  │                 │  │                 │  │
│  │ • cargo run     │  │ • npm run dev   │  │ • Breakpoints   │  │
│  │ • IDE debugging │  │ • Hot reload    │  │ • Step through │  │
│  │ • Fast compile  │  │ • React DevTools│  │ • Variable inspect│  │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘  │
│                                │                               │
│  ┌─────────────────────────────┼─────────────────────────────┐  │
│  │            Docker Services (Infrastructure Only)         │  │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐    │  │
│  │  │  Temporal   │  │ PostgreSQL  │  │     Redis       │    │  │
│  │  │   Cluster   │  │  Database   │  │     Cache       │    │  │
│  │  │             │  │             │  │                 │    │  │
│  │  │ Port: 7233  │  │ Port: 5432  │  │   Port: 6379    │    │  │
│  │  │ UI: 8233    │  │             │  │                 │    │  │
│  │  └─────────────┘  └─────────────┘  └─────────────────┘    │  │
│  └─────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
```

### Development Commands
```makefile
# Start infrastructure services only (Docker)
make infra
# ✅ Starts: Temporal, PostgreSQL, Redis in Docker

# Start Rust backend (native macOS - Terminal 1)
make dev-backend  
# ✅ Runs: cd backend && cargo run --bin api-gateway
# ✅ Full IDE debugging with breakpoints

# Start React frontend (native macOS - Terminal 2)
make dev-frontend
# ✅ Runs: cd frontend && npm run dev  
# ✅ Hot reload + React DevTools + browser debugging
```

### IDE Configuration Examples
```json
// .vscode/settings.json
{
  "rust-analyzer.cargo.features": ["temporal-testing"],
  "rust-analyzer.checkOnSave.command": "clippy",
  "typescript.preferences.importModuleSpecifier": "relative",
  "editor.formatOnSave": true
}

// .vscode/launch.json - Native Rust debugging
{
  "type": "lldb",
  "request": "launch",
  "name": "Debug API Gateway", 
  "cargo": {
    "args": ["build", "--bin=api-gateway"],
    "filter": {"name": "api-gateway", "kind": "bin"}
  },
  "env": {
    "DATABASE_URL": "postgresql://postgres:postgres@localhost:5432/adx_core",
    "TEMPORAL_URL": "localhost:7233"
  }
}
```

## Sprint Development Workflow (IDE-Optimized)

### 2-Week Sprint Structure
1. **Sprint Planning** (Day 1)
   - Review specifications and requirements
   - Set up IDE debugging environment
   - Break down tasks into Temporal workflows and activities

2. **Development** (Days 2-9)
   - Use native development with IDE debugging
   - Implement Temporal workflows with breakpoints
   - Write tests with native test runner
   - Use hot reload for rapid frontend iteration

3. **Integration & Testing** (Days 10-12)
   - Integration testing with native tools
   - End-to-end workflow testing with Temporal UI
   - Performance testing with native profiling

4. **Review & Deployment** (Days 13-14)
   - Code review with IDE integration
   - Deploy to staging environment
   - Sprint retrospective and planning

### Daily Development Checklist
- [ ] Read relevant specifications before coding
- [ ] Follow Temporal-First Principle for complex operations
- [ ] Write tests before implementation (TDD)
- [ ] Use proper error handling and logging
- [ ] Update documentation for new workflows
- [ ] Run all tests and ensure they pass
- [ ] Check code quality with clippy and formatting

## Common Pitfalls to Avoid

### ❌ Anti-Patterns
1. **Custom retry logic** instead of Temporal retries
2. **Blocking operations** in async functions
3. **Unwrap/expect** in production code
4. **Missing tenant isolation** in database queries
5. **Hardcoded configuration** instead of environment variables
6. **Missing error handling** in activities
7. **Complex workflows** that should be broken down
8. **Direct database access** instead of repository pattern

### ✅ Best Practices
1. **Use Temporal workflows** for all complex operations
2. **Implement proper error types** with thiserror
3. **Use structured logging** with correlation IDs
4. **Follow repository pattern** for database access
5. **Write comprehensive tests** including workflow replay tests
6. **Document all public APIs** with examples
7. **Use environment variables** for configuration
8. **Implement proper tenant isolation** in all operations

## Code Review Checklist

### Before Submitting Code
- [ ] All tests pass (unit, integration, workflow)
- [ ] Code follows Temporal-First Principle
- [ ] Proper error handling implemented
- [ ] Documentation updated
- [ ] No clippy warnings
- [ ] Tenant isolation verified
- [ ] Performance considerations addressed
- [ ] Security best practices followed

### Review Criteria
- [ ] Temporal workflows used appropriately
- [ ] Activities are idempotent and focused
- [ ] Error handling is comprehensive
- [ ] Tests cover all scenarios
- [ ] Code is readable and well-documented
- [ ] Performance is acceptable
- [ ] Security vulnerabilities addressed

This comprehensive guide ensures AI Coder Teams can efficiently develop ADX CORE following best practices and architectural principles.