// Unit tests for Auth Service
use std::sync::Arc;
use auth_service::*;
use adx_shared::testing::{TestContext, TestAssertions, mocks::*};
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

// Import test macros
use adx_shared::{test_case, parameterized_test, mock_repository};

// Mock user repository for testing
mock_repository!(MockUserRepository, User);

#[derive(Debug, Clone)]
struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub tenant_id: String,
    pub is_active: bool,
}

/// Test the user creation functionality
test_case!(test_create_user_success, |ctx: TestContext| async move {
    // Arrange
    let user_repo = MockUserRepository::new();
    let auth_service = AuthService::new(Arc::new(user_repo));
    
    let create_request = CreateUserRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        tenant_id: "tenant-123".to_string(),
    };
    
    // Act
    let result = auth_service.create_user(create_request).await;
    
    // Assert
    let user = TestAssertions::assert_ok(result);
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.tenant_id, "tenant-123");
    assert!(user.is_active);
    assert!(!user.password_hash.is_empty());
    assert_ne!(user.password_hash, "password123"); // Should be hashed
});

/// Test user creation with invalid email
test_case!(test_create_user_invalid_email, |ctx: TestContext| async move {
    // Arrange
    let user_repo = MockUserRepository::new();
    let auth_service = AuthService::new(Arc::new(user_repo));
    
    let create_request = CreateUserRequest {
        email: "invalid-email".to_string(),
        password: "password123".to_string(),
        tenant_id: "tenant-123".to_string(),
    };
    
    // Act
    let result = auth_service.create_user(create_request).await;
    
    // Assert
    TestAssertions::assert_err(result);
});

/// Test user creation with repository failure
test_case!(test_create_user_repository_failure, |ctx: TestContext| async move {
    // Arrange
    let user_repo = MockUserRepository::new();
    user_repo.set_failure(MockError::DatabaseError("Connection failed".to_string()));
    let auth_service = AuthService::new(Arc::new(user_repo));
    
    let create_request = CreateUserRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        tenant_id: "tenant-123".to_string(),
    };
    
    // Act
    let result = auth_service.create_user(create_request).await;
    
    // Assert
    TestAssertions::assert_err(result);
});

/// Test user authentication success
test_case!(test_authenticate_user_success, |ctx: TestContext| async move {
    // Arrange
    let user_repo = MockUserRepository::new();
    let auth_service = AuthService::new(Arc::new(user_repo.clone()));
    
    // Create a user first
    let user = User {
        id: Uuid::new_v4().to_string(),
        email: "test@example.com".to_string(),
        password_hash: hash_password("password123").unwrap(),
        tenant_id: "tenant-123".to_string(),
        is_active: true,
    };
    
    user_repo.create(user.clone()).await.unwrap();
    
    let auth_request = AuthenticateRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        tenant_id: Some("tenant-123".to_string()),
    };
    
    // Act
    let result = auth_service.authenticate(auth_request).await;
    
    // Assert
    let auth_result = TestAssertions::assert_ok(result);
    assert_eq!(auth_result.user_id, user.id);
    assert!(!auth_result.access_token.is_empty());
    assert!(!auth_result.refresh_token.is_empty());
});

/// Test user authentication with wrong password
test_case!(test_authenticate_user_wrong_password, |ctx: TestContext| async move {
    // Arrange
    let user_repo = MockUserRepository::new();
    let auth_service = AuthService::new(Arc::new(user_repo.clone()));
    
    // Create a user first
    let user = User {
        id: Uuid::new_v4().to_string(),
        email: "test@example.com".to_string(),
        password_hash: hash_password("password123").unwrap(),
        tenant_id: "tenant-123".to_string(),
        is_active: true,
    };
    
    user_repo.create(user.clone()).await.unwrap();
    
    let auth_request = AuthenticateRequest {
        email: "test@example.com".to_string(),
        password: "wrongpassword".to_string(),
        tenant_id: Some("tenant-123".to_string()),
    };
    
    // Act
    let result = auth_service.authenticate(auth_request).await;
    
    // Assert
    TestAssertions::assert_err(result);
});

/// Parameterized test for password validation
parameterized_test!(
    test_password_validation,
    vec![
        ("", false),                    // Empty password
        ("123", false),                 // Too short
        ("password", false),            // No numbers
        ("12345678", false),            // No letters
        ("Password1", true),            // Valid
        ("MySecurePass123", true),      // Valid long
    ],
    |ctx: TestContext, (password, should_be_valid): (String, bool)| async move {
        // Arrange
        let user_repo = MockUserRepository::new();
        let auth_service = AuthService::new(Arc::new(user_repo));
        
        let create_request = CreateUserRequest {
            email: "test@example.com".to_string(),
            password: password.to_string(),
            tenant_id: "tenant-123".to_string(),
        };
        
        // Act
        let result = auth_service.create_user(create_request).await;
        
        // Assert
        if should_be_valid {
            TestAssertions::assert_ok(result);
        } else {
            TestAssertions::assert_err(result);
        }
    }
);

/// Test JWT token generation and validation
test_case!(test_jwt_token_operations, |ctx: TestContext| async move {
    // Arrange
    let jwt_service = JwtService::new("test-secret".to_string());
    let user_id = Uuid::new_v4().to_string();
    let tenant_id = "tenant-123".to_string();
    
    // Act - Generate token
    let token = jwt_service.generate_access_token(&user_id, &tenant_id).await;
    let access_token = TestAssertions::assert_ok(token);
    
    // Act - Validate token
    let claims = jwt_service.validate_token(&access_token).await;
    let token_claims = TestAssertions::assert_ok(claims);
    
    // Assert
    assert_eq!(token_claims.user_id, user_id);
    assert_eq!(token_claims.tenant_id, tenant_id);
    assert!(token_claims.exp > Utc::now().timestamp());
});

/// Test JWT token expiration
test_case!(test_jwt_token_expiration, |ctx: TestContext| async move {
    // Arrange
    let jwt_service = JwtService::with_expiration("test-secret".to_string(), 1); // 1 second expiration
    let user_id = Uuid::new_v4().to_string();
    let tenant_id = "tenant-123".to_string();
    
    // Act - Generate token
    let token = jwt_service.generate_access_token(&user_id, &tenant_id).await;
    let access_token = TestAssertions::assert_ok(token);
    
    // Wait for token to expire
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    // Act - Validate expired token
    let claims = jwt_service.validate_token(&access_token).await;
    
    // Assert
    TestAssertions::assert_err(claims);
});

/// Test user service with mocked dependencies
test_case!(test_user_service_integration, |ctx: TestContext| async move {
    // Arrange
    let user_repo = MockUserRepository::new();
    let temporal_client = MockTemporalClient::new();
    let auth_service = AuthService::new_with_temporal(
        Arc::new(user_repo.clone()),
        Arc::new(temporal_client.clone()),
    );
    
    // Mock workflow completion
    temporal_client.complete_workflow(
        "user-registration-workflow",
        json!({
            "user_id": "user-123",
            "status": "completed"
        })
    ).await.unwrap();
    
    let create_request = CreateUserRequest {
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
        tenant_id: "tenant-123".to_string(),
    };
    
    // Act
    let result = auth_service.create_user_with_workflow(create_request).await;
    
    // Assert
    let user = TestAssertions::assert_ok(result);
    assert_eq!(user.email, "test@example.com");
    
    // Verify workflow was called
    assert_eq!(temporal_client.get_call_count("start_workflow"), 1);
});

/// Test concurrent user creation
test_case!(test_concurrent_user_creation, |ctx: TestContext| async move {
    // Arrange
    let user_repo = Arc::new(MockUserRepository::new());
    let auth_service = Arc::new(AuthService::new(user_repo.clone()));
    
    let mut handles = Vec::new();
    
    // Act - Create multiple users concurrently
    for i in 0..10 {
        let service = auth_service.clone();
        let handle = tokio::spawn(async move {
            let create_request = CreateUserRequest {
                email: format!("user{}@example.com", i),
                password: "password123".to_string(),
                tenant_id: "tenant-123".to_string(),
            };
            service.create_user(create_request).await
        });
        handles.push(handle);
    }
    
    // Wait for all to complete
    let results: Vec<_> = futures::future::join_all(handles).await;
    
    // Assert
    let mut successful_creations = 0;
    for result in results {
        let task_result = result.unwrap();
        if task_result.is_ok() {
            successful_creations += 1;
        }
    }
    
    assert_eq!(successful_creations, 10);
    assert_eq!(user_repo.get_call_count("create"), 10);
});

// Helper functions for tests
fn hash_password(password: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Mock password hashing - in real implementation would use bcrypt
    Ok(format!("hashed_{}", password))
}

// Mock service implementations for testing
struct AuthService {
    user_repo: Arc<dyn MockRepository<User>>,
    temporal_client: Option<Arc<MockTemporalClient>>,
}

impl AuthService {
    fn new(user_repo: Arc<dyn MockRepository<User>>) -> Self {
        Self {
            user_repo,
            temporal_client: None,
        }
    }
    
    fn new_with_temporal(
        user_repo: Arc<dyn MockRepository<User>>,
        temporal_client: Arc<MockTemporalClient>,
    ) -> Self {
        Self {
            user_repo,
            temporal_client: Some(temporal_client),
        }
    }
    
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, MockError> {
        // Validate email
        if !request.email.contains('@') {
            return Err(MockError::ValidationError("Invalid email format".to_string()));
        }
        
        // Validate password
        if !is_valid_password(&request.password) {
            return Err(MockError::ValidationError("Invalid password".to_string()));
        }
        
        // Hash password
        let password_hash = hash_password(&request.password)
            .map_err(|e| MockError::DatabaseError(e.to_string()))?;
        
        // Create user
        let user = User {
            id: Uuid::new_v4().to_string(),
            email: request.email,
            password_hash,
            tenant_id: request.tenant_id,
            is_active: true,
        };
        
        self.user_repo.create(user).await
    }
    
    async fn create_user_with_workflow(&self, request: CreateUserRequest) -> Result<User, MockError> {
        if let Some(temporal_client) = &self.temporal_client {
            // Start workflow
            let workflow_handle = temporal_client.start_workflow(
                "user-registration-workflow",
                &Uuid::new_v4().to_string(),
                json!(request),
            ).await?;
            
            // Wait for workflow result
            let _workflow_result = workflow_handle.get_result().await?;
        }
        
        self.create_user(request).await
    }
    
    async fn authenticate(&self, request: AuthenticateRequest) -> Result<AuthResult, MockError> {
        // Find user by email
        let users = self.user_repo.list().await?;
        let user = users.iter()
            .find(|u| u.email == request.email && u.tenant_id == request.tenant_id.as_deref().unwrap_or(""))
            .ok_or_else(|| MockError::NotFound("User not found".to_string()))?;
        
        // Verify password
        let expected_hash = hash_password(&request.password)
            .map_err(|e| MockError::DatabaseError(e.to_string()))?;
        
        if user.password_hash != expected_hash {
            return Err(MockError::ValidationError("Invalid credentials".to_string()));
        }
        
        // Generate tokens
        Ok(AuthResult {
            user_id: user.id.clone(),
            access_token: format!("access_token_{}", user.id),
            refresh_token: format!("refresh_token_{}", user.id),
            expires_in: 3600,
        })
    }
}

struct JwtService {
    secret: String,
    expiration_seconds: i64,
}

impl JwtService {
    fn new(secret: String) -> Self {
        Self {
            secret,
            expiration_seconds: 3600,
        }
    }
    
    fn with_expiration(secret: String, expiration_seconds: i64) -> Self {
        Self {
            secret,
            expiration_seconds,
        }
    }
    
    async fn generate_access_token(&self, user_id: &str, tenant_id: &str) -> Result<String, MockError> {
        // Mock JWT generation
        Ok(format!("jwt_{}_{}_exp_{}", user_id, tenant_id, 
                  Utc::now().timestamp() + self.expiration_seconds))
    }
    
    async fn validate_token(&self, token: &str) -> Result<TokenClaims, MockError> {
        // Mock JWT validation
        let parts: Vec<&str> = token.split('_').collect();
        if parts.len() < 5 {
            return Err(MockError::ValidationError("Invalid token format".to_string()));
        }
        
        let exp: i64 = parts[4].parse()
            .map_err(|_| MockError::ValidationError("Invalid expiration".to_string()))?;
        
        if exp <= Utc::now().timestamp() {
            return Err(MockError::ValidationError("Token expired".to_string()));
        }
        
        Ok(TokenClaims {
            user_id: parts[1].to_string(),
            tenant_id: parts[2].to_string(),
            exp,
        })
    }
}

// Request/Response types
#[derive(Debug, Clone)]
struct CreateUserRequest {
    email: String,
    password: String,
    tenant_id: String,
}

#[derive(Debug, Clone)]
struct AuthenticateRequest {
    email: String,
    password: String,
    tenant_id: Option<String>,
}

#[derive(Debug, Clone)]
struct AuthResult {
    user_id: String,
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Debug, Clone)]
struct TokenClaims {
    user_id: String,
    tenant_id: String,
    exp: i64,
}

// Helper functions
fn is_valid_password(password: &str) -> bool {
    password.len() >= 8 && 
    password.chars().any(|c| c.is_alphabetic()) &&
    password.chars().any(|c| c.is_numeric())
}