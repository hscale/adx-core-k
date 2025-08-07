# 🦀 Rust Coding Standards - Top 1% Quality

## Core Principles

> **"Code like your life depends on it. Document like you're teaching your future self."**

These standards ensure world-class Rust code that's maintainable, performant, and secure.

## 🏆 Code Organization

### 1. Project Structure
```rust
// ✅ Standard service structure
src/
├── main.rs              // Service entry point
├── lib.rs               // Public API exports
├── config.rs            // Configuration management
├── error.rs             // Error types and handling
├── handlers/            // HTTP request handlers
│   ├── mod.rs
│   ├── auth.rs
│   └── users.rs
├── workflows/           // Temporal workflows
│   ├── mod.rs
│   ├── user_onboarding.rs
│   └── file_processing.rs
├── activities/          // Temporal activities
│   ├── mod.rs
│   ├── database.rs
│   └── email.rs
├── models/              // Data models
│   ├── mod.rs
│   ├── user.rs
│   └── tenant.rs
├── services/            // Business logic
│   ├── mod.rs
│   ├── user_service.rs
│   └── auth_service.rs
└── utils/               // Utility functions
    ├── mod.rs
    ├── validation.rs
    └── crypto.rs
```

### 2. Module Declaration Order
```rust
// ✅ Required order in lib.rs and mod.rs
pub mod config;
pub mod error;
pub mod models;
pub mod services;
pub mod workflows;
pub mod activities;
pub mod handlers;
pub mod utils;

// Re-exports
pub use config::*;
pub use error::*;
```

## 📝 Naming Conventions

### 1. Identifiers
```rust
// ✅ Types: PascalCase
pub struct UserService;
pub enum AuthError;
pub trait DatabaseProvider;

// ✅ Functions, variables: snake_case
pub async fn create_user_account() {}
let user_data = get_user_data();
const MAX_RETRY_COUNT: u32 = 3;

// ✅ Constants: SCREAMING_SNAKE_CASE
pub const DATABASE_TIMEOUT_SECONDS: u64 = 30;
pub const DEFAULT_PAGE_SIZE: usize = 50;

// ✅ Modules: snake_case
mod user_service;
mod auth_middleware;
```

### 2. Temporal Naming
```rust
// ✅ Workflows: verb_noun_workflow
pub async fn user_onboarding_workflow() {}
pub async fn file_processing_workflow() {}
pub async fn payment_confirmation_workflow() {}

// ✅ Activities: verb_noun_activity
pub async fn create_user_activity() {}
pub async fn send_email_activity() {}
pub async fn validate_payment_activity() {}

// ✅ Task queues: service-name-queue
const USER_SERVICE_QUEUE: &str = "user-service-queue";
const FILE_SERVICE_QUEUE: &str = "file-service-queue";
```

## 🛡️ Error Handling

### 1. Error Type Design
```rust
// ✅ Use thiserror for all custom errors
#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("User not found with ID: {user_id}")]
    UserNotFound { user_id: Uuid },
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("User already exists with email: {email}")]
    UserAlreadyExists { email: String },
    
    #[error("Permission denied for user {user_id} on resource {resource}")]
    PermissionDenied { user_id: Uuid, resource: String },
    
    #[error("Database operation failed")]
    DatabaseError(#[from] sqlx::Error),
    
    #[error("Authentication failed")]
    AuthenticationError(#[from] AuthError),
    
    #[error("Temporal workflow error")]
    WorkflowError(#[from] temporal_sdk::Error),
}

// ✅ Implement common traits
impl UserServiceError {
    pub fn is_retryable(&self) -> bool {
        matches!(self, 
            Self::DatabaseError(_) |
            Self::WorkflowError(_)
        )
    }
    
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::UserNotFound { .. } => StatusCode::NOT_FOUND,
            Self::InvalidEmail { .. } => StatusCode::BAD_REQUEST,
            Self::UserAlreadyExists { .. } => StatusCode::CONFLICT,
            Self::PermissionDenied { .. } => StatusCode::FORBIDDEN,
            Self::DatabaseError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AuthenticationError(_) => StatusCode::UNAUTHORIZED,
            Self::WorkflowError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

### 2. Error Propagation
```rust
// ✅ Use ? operator with proper context
pub async fn get_user_with_permissions(
    user_id: Uuid,
    tenant_id: Uuid,
) -> Result<UserWithPermissions, UserServiceError> {
    // Get user (handles not found)
    let user = self.get_user(user_id, tenant_id).await?;
    
    // Get permissions (may fail with database error)
    let permissions = self.auth_service
        .get_user_permissions(user_id, tenant_id)
        .await
        .map_err(UserServiceError::AuthenticationError)?;
    
    Ok(UserWithPermissions { user, permissions })
}

// ✅ Handle errors at appropriate boundaries
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, UserServiceError> {
    match state.user_service.create_user(req).await {
        Ok(user) => Ok(Json(UserResponse::from(user))),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create user");
            Err(e)
        }
    }
}
```

## 📊 Data Models

### 1. Struct Design
```rust
// ✅ Use proper derives and documentation
/// Represents a user in the system
/// 
/// Users are always associated with a tenant and have role-based permissions.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct User {
    /// Unique user identifier
    pub id: Uuid,
    
    /// Tenant this user belongs to
    pub tenant_id: Uuid,
    
    /// User's email address (unique within tenant)
    #[serde(deserialize_with = "deserialize_email")]
    pub email: String,
    
    /// User's display name
    pub name: String,
    
    /// Whether the user account is active
    #[serde(default = "default_true")]
    pub is_active: bool,
    
    /// When the user was created
    pub created_at: DateTime<Utc>,
    
    /// When the user was last updated
    pub updated_at: DateTime<Utc>,
}

// ✅ Implement Display for important types
impl std::fmt::Display for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "User(id={}, email={})", self.id, self.email)
    }
}

// ✅ Helper functions
fn default_true() -> bool { true }

fn deserialize_email<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let email = String::deserialize(deserializer)?;
    validate_email(&email)
        .map_err(serde::de::Error::custom)?;
    Ok(email.to_lowercase())
}
```

### 2. Request/Response Types
```rust
// ✅ Separate types for requests and responses
#[derive(Debug, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(email)]
    pub email: String,
    
    #[validate(length(min = 2, max = 100))]
    pub name: String,
    
    #[validate(length(min = 8))]
    pub password: String,
    
    pub tenant_id: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct UserResponse {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    // Note: No password or sensitive data
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            name: user.name,
            is_active: user.is_active,
            created_at: user.created_at,
        }
    }
}
```

## 🚀 Performance Standards

### 1. Memory Management
```rust
// ✅ Use Arc for shared data
#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DatabaseManager>,
    pub cache: Arc<RedisManager>,
    pub temporal: Arc<TemporalClient>,
}

// ✅ Use Cow for potentially borrowed data
pub fn format_user_display_name(user: &User) -> Cow<str> {
    if user.name.is_empty() {
        Cow::Borrowed(&user.email)
    } else {
        Cow::Owned(format!("{} <{}>", user.name, user.email))
    }
}

// ✅ Avoid unnecessary allocations
pub fn validate_email(email: &str) -> Result<(), ValidationError> {
    // Use string methods instead of regex for simple validation
    if !email.contains('@') || !email.contains('.') {
        return Err(ValidationError::InvalidFormat);
    }
    
    if email.len() > 320 {
        return Err(ValidationError::TooLong);
    }
    
    Ok(())
}
```

### 2. Database Queries
```rust
// ✅ Use prepared statements with sqlx
pub async fn get_users_by_tenant(
    db: &PgPool,
    tenant_id: Uuid,
    limit: i64,
    offset: i64,
) -> Result<Vec<User>, sqlx::Error> {
    sqlx::query_as!(
        User,
        r#"
        SELECT id, tenant_id, email, name, is_active, created_at, updated_at
        FROM users 
        WHERE tenant_id = $1 
        ORDER BY created_at DESC 
        LIMIT $2 OFFSET $3
        "#,
        tenant_id,
        limit,
        offset
    )
    .fetch_all(db)
    .await
}

// ✅ Use transactions for consistency
pub async fn create_user_with_profile(
    db: &PgPool,
    user_data: CreateUserRequest,
    profile_data: CreateProfileRequest,
) -> Result<(User, Profile), UserServiceError> {
    let mut tx = db.begin().await?;
    
    let user = sqlx::query_as!(
        User,
        "INSERT INTO users (email, name, tenant_id) VALUES ($1, $2, $3) RETURNING *",
        user_data.email,
        user_data.name,
        user_data.tenant_id
    )
    .fetch_one(&mut *tx)
    .await?;
    
    let profile = sqlx::query_as!(
        Profile,
        "INSERT INTO profiles (user_id, bio, avatar_url) VALUES ($1, $2, $3) RETURNING *",
        user.id,
        profile_data.bio,
        profile_data.avatar_url
    )
    .fetch_one(&mut *tx)
    .await?;
    
    tx.commit().await?;
    
    Ok((user, profile))
}
```

## 🧪 Testing Standards

### 1. Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    
    #[fixture]
    fn sample_user() -> User {
        User {
            id: Uuid::new_v4(),
            tenant_id: Uuid::new_v4(),
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            is_active: true,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }
    }
    
    #[rstest]
    #[case("valid@example.com", true)]
    #[case("invalid-email", false)]
    #[case("", false)]
    fn test_email_validation(#[case] email: &str, #[case] expected: bool) {
        assert_eq!(validate_email(email).is_ok(), expected);
    }
    
    #[tokio::test]
    async fn test_create_user_success() {
        let mut mock_db = MockDatabase::new();
        mock_db
            .expect_create_user()
            .with(predicate::eq("test@example.com"))
            .times(1)
            .returning(|_| Ok(sample_user()));
        
        let service = UserService::new(Arc::new(mock_db));
        let result = service.create_user(CreateUserRequest {
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password: "secure_password".to_string(),
            tenant_id: None,
        }).await;
        
        assert!(result.is_ok());
    }
}
```

### 2. Integration Tests
```rust
// tests/integration_test.rs
use adx_core::*;
use testcontainers::*;

#[tokio::test]
async fn test_user_creation_workflow() {
    let docker = clients::Cli::default();
    let postgres_container = docker.run(images::postgres::Postgres::default());
    let redis_container = docker.run(images::redis::Redis::default());
    
    let config = TestConfig {
        database_url: format!(
            "postgresql://postgres:postgres@localhost:{}/postgres",
            postgres_container.get_host_port_ipv4(5432)
        ),
        redis_url: format!(
            "redis://localhost:{}",
            redis_container.get_host_port_ipv4(6379)
        ),
    };
    
    let app = create_test_app(config).await;
    
    // Test the actual API
    let response = app
        .post("/api/v1/users")
        .json(&CreateUserRequest {
            email: "test@example.com".to_string(),
            name: "Test User".to_string(),
            password: "secure_password".to_string(),
            tenant_id: None,
        })
        .send()
        .await;
    
    assert_eq!(response.status(), 201);
}
```

## 📝 Documentation Standards

### 1. Function Documentation
```rust
/// Creates a new user account and triggers the onboarding workflow
/// 
/// This function performs the following steps:
/// 1. Validates the user data (email format, password strength)
/// 2. Checks for existing users with the same email
/// 3. Creates the user in the database
/// 4. Triggers the onboarding workflow asynchronously
/// 
/// # Arguments
/// 
/// * `user_data` - The user information for the new account
/// 
/// # Returns
/// 
/// Returns the created `User` on success, or a `UserServiceError` on failure.
/// 
/// # Errors
/// 
/// This function will return an error if:
/// * The email format is invalid
/// * A user with the same email already exists
/// * The database operation fails
/// * The workflow trigger fails
/// 
/// # Examples
/// 
/// ```rust
/// let user_data = CreateUserRequest {
///     email: "john@example.com".to_string(),
///     name: "John Doe".to_string(),
///     password: "secure_password".to_string(),
///     tenant_id: Some(tenant_id),
/// };
/// 
/// let user = user_service.create_user(user_data).await?;
/// println!("Created user: {}", user.email);
/// ```
pub async fn create_user(
    &self,
    user_data: CreateUserRequest,
) -> Result<User, UserServiceError> {
    // Implementation...
}
```

### 2. Module Documentation
```rust
//! User Service Module
//! 
//! This module provides functionality for managing user accounts within the ADX Core platform.
//! It handles user creation, authentication, profile management, and integration with the
//! Temporal workflow engine for complex user operations.
//! 
//! # Features
//! 
//! * Multi-tenant user isolation
//! * Secure password handling with bcrypt
//! * Email validation and normalization
//! * Integration with authentication service
//! * Workflow-based user onboarding
//! 
//! # Examples
//! 
//! ```rust
//! use adx_core::user_service::UserService;
//! 
//! let user_service = UserService::new(db_pool, temporal_client);
//! let user = user_service.create_user(user_data).await?;
//! ```

use crate::models::User;
use crate::workflows::user_onboarding_workflow;
```

## 🔧 Code Quality Tools

### 1. Required Cargo.toml Configuration
```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
unused_imports = "warn"

[lints.clippy]
all = "warn"
pedantic = "warn"
nursery = "warn"
cargo = "warn"

# Performance lints
missing_inline_in_public_items = "allow"  # Can be noisy
single_match_else = "allow"               # Sometimes clearer

# Style preferences
module_name_repetitions = "allow"         # Sometimes necessary
similar_names = "allow"                   # Context usually makes it clear
```

### 2. CI/CD Quality Checks
```bash
# Required commands that must pass
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo audit
cargo deny check
```

---

**Remember: Code quality is not negotiable. Every line should meet these standards!** 🦀
