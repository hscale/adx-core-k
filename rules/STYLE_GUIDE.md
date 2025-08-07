# 🎨 AI Coding Style Guide

## Code Personality: Enterprise Rust Excellence

This style guide defines the "voice" and approach for AI-generated code in ADX Core.

## 🏆 Code Quality Philosophy

### The "Top 1%" Standard
- **Readable**: Code reads like well-written prose
- **Robust**: Handles edge cases gracefully
- **Performant**: Optimized for real-world usage
- **Secure**: Security is built-in, not bolted-on
- **Maintainable**: Easy to extend and modify
- **Testable**: Designed for comprehensive testing

## 🎯 Writing Style Principles

### 1. Clarity Over Cleverness
```rust
// ❌ Clever but unclear
fn proc_usr(u: &U, t: T) -> R { /* ... */ }

// ✅ Clear and descriptive
async fn process_user_registration(
    user_data: &UserRegistrationData,
    tenant_context: TenantContext,
) -> Result<UserRegistrationResult, UserError> {
    // Implementation is immediately understandable
}
```

### 2. Express Intent, Not Just Implementation
```rust
// ❌ What, but not why
let users = db.query("SELECT * FROM users WHERE active = true").await?;

// ✅ Intent-revealing with business context
let active_users_for_billing = repository
    .find_billable_users_by_tenant(tenant_id)
    .await
    .context("Failed to retrieve active users for billing calculation")?;
```

### 3. Anticipate and Handle Edge Cases
```rust
// ✅ Comprehensive edge case handling
pub async fn validate_user_email(email: &str) -> Result<ValidatedEmail, ValidationError> {
    // Handle empty/whitespace
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return Err(ValidationError::EmptyEmail);
    }
    
    // Handle length limits (common email max length)
    if trimmed.len() > 254 {
        return Err(ValidationError::EmailTooLong);
    }
    
    // Handle basic format validation
    if !EMAIL_REGEX.is_match(trimmed) {
        return Err(ValidationError::InvalidFormat);
    }
    
    // Handle domain validation (if required)
    if let Some(domain) = trimmed.split('@').nth(1) {
        if BLOCKED_DOMAINS.contains(&domain.to_lowercase().as_str()) {
            return Err(ValidationError::BlockedDomain);
        }
    }
    
    Ok(ValidatedEmail(trimmed.to_lowercase()))
}
```

## 🏗️ Architecture Expression

### Business Logic as Workflows
```rust
// ✅ Complex business process as durable workflow
#[temporal_sdk::workflow]
pub async fn subscription_lifecycle_workflow(
    input: SubscriptionLifecycleInput,
) -> WorkflowResult<SubscriptionLifecycleOutput> {
    // Step 1: Validate subscription eligibility
    let eligibility_check = execute_activity!(
        validate_subscription_eligibility_activity,
        ValidationInput {
            tenant_id: input.tenant_id,
            plan_id: input.plan_id,
            user_id: input.user_id,
        },
        ActivityOptions::default().start_to_close_timeout(Duration::from_secs(30))
    ).await?;
    
    if !eligibility_check.is_eligible {
        return Ok(SubscriptionLifecycleOutput::Rejected {
            reason: eligibility_check.rejection_reason,
        });
    }
    
    // Step 2: Process payment (with retries for transient failures)
    let payment_result = execute_activity!(
        process_subscription_payment_activity,
        PaymentInput {
            amount: eligibility_check.pricing.amount,
            currency: eligibility_check.pricing.currency,
            payment_method_id: input.payment_method_id,
            tenant_id: input.tenant_id,
        },
        ActivityOptions::default()
            .start_to_close_timeout(Duration::from_secs(60))
            .retry_policy(RetryPolicy::default().maximum_attempts(3))
    ).await?;
    
    // Step 3: Activate subscription (only after successful payment)
    let activation_result = execute_activity!(
        activate_subscription_activity,
        ActivationInput {
            tenant_id: input.tenant_id,
            plan_id: input.plan_id,
            payment_transaction_id: payment_result.transaction_id,
            effective_date: Utc::now(),
        },
        ActivityOptions::default()
    ).await?;
    
    // Step 4: Schedule recurring billing
    let billing_schedule = schedule_recurring_billing_workflow(
        input.tenant_id,
        activation_result.subscription_id,
        eligibility_check.pricing.billing_cycle,
    ).await?;
    
    Ok(SubscriptionLifecycleOutput::Success {
        subscription_id: activation_result.subscription_id,
        next_billing_date: billing_schedule.next_billing_date,
    })
}
```

### Multi-Tenant Security Expression
```rust
// ✅ Security-first data access pattern
#[derive(Debug)]
pub struct SecureUserRepository {
    pool: PgPool,
}

impl SecureUserRepository {
    /// Retrieves a user with guaranteed tenant isolation.
    /// 
    /// # Security
    /// This method enforces row-level security by requiring tenant_id
    /// and using it in the WHERE clause. This prevents cross-tenant data access.
    pub async fn find_user_by_id(
        &self,
        user_id: Uuid,
        requesting_tenant_id: Uuid,
    ) -> Result<Option<User>, DatabaseError> {
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT 
                id,
                tenant_id,
                email,
                name,
                created_at,
                updated_at,
                is_active,
                -- Sensitive fields excluded from general queries
                password_hash  -- Only include when specifically needed
            FROM users 
            WHERE id = $1 
              AND tenant_id = $2  -- CRITICAL: tenant isolation
              AND is_active = true  -- Business rule: only active users
            "#,
            user_id,
            requesting_tenant_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| {
            // Log security-relevant errors without exposing internals
            tracing::warn!(
                user_id = %user_id,
                tenant_id = %requesting_tenant_id,
                error = %e,
                "Failed to retrieve user - potential security violation or system error"
            );
            DatabaseError::QueryFailed
        })?;
        
        Ok(user)
    }
}
```

## 🔧 Error Handling Excellence

### Comprehensive Error Types
```rust
// ✅ Rich error types that help debugging and user experience
#[derive(Debug, thiserror::Error)]
pub enum UserServiceError {
    #[error("User not found: {user_id} in tenant {tenant_id}")]
    UserNotFound { user_id: Uuid, tenant_id: Uuid },
    
    #[error("User already exists with email: {email}")]
    UserAlreadyExists { email: String },
    
    #[error("Invalid user data: {field} - {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Database operation failed")]
    DatabaseError(#[from] DatabaseError),
    
    #[error("Authentication failed: {reason}")]
    AuthenticationError { reason: String },
    
    #[error("Authorization failed: user {user_id} lacks permission {permission}")]
    AuthorizationError { user_id: Uuid, permission: String },
    
    #[error("Rate limit exceeded: {requests} requests in {window} seconds")]
    RateLimitExceeded { requests: u32, window: u32 },
    
    #[error("External service unavailable: {service}")]
    ExternalServiceError { service: String },
    
    #[error("Workflow execution failed: {workflow_id}")]
    WorkflowError { workflow_id: String },
}

impl UserServiceError {
    /// Convert to appropriate HTTP status code
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::UserNotFound { .. } => StatusCode::NOT_FOUND,
            Self::UserAlreadyExists { .. } => StatusCode::CONFLICT,
            Self::ValidationError { .. } => StatusCode::BAD_REQUEST,
            Self::AuthenticationError { .. } => StatusCode::UNAUTHORIZED,
            Self::AuthorizationError { .. } => StatusCode::FORBIDDEN,
            Self::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::DatabaseError(_) 
            | Self::ExternalServiceError { .. } 
            | Self::WorkflowError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
    
    /// Get user-safe error message (no internal details)
    pub fn user_message(&self) -> String {
        match self {
            Self::UserNotFound { .. } => "User not found".to_string(),
            Self::UserAlreadyExists { .. } => "A user with this email already exists".to_string(),
            Self::ValidationError { field, reason } => format!("Invalid {}: {}", field, reason),
            Self::AuthenticationError { .. } => "Authentication required".to_string(),
            Self::AuthorizationError { .. } => "Insufficient permissions".to_string(),
            Self::RateLimitExceeded { .. } => "Too many requests. Please try again later".to_string(),
            Self::DatabaseError(_) 
            | Self::ExternalServiceError { .. } 
            | Self::WorkflowError { .. } => "An internal error occurred. Please try again".to_string(),
        }
    }
}
```

## 📝 Documentation Style

### Self-Documenting Code with Strategic Comments
```rust
/// Processes a user registration workflow with comprehensive validation,
/// security checks, and audit logging.
/// 
/// # Arguments
/// * `registration_data` - Validated user registration information
/// * `tenant_context` - Tenant-specific configuration and limits
/// * `request_metadata` - Request tracking and audit information
/// 
/// # Returns
/// * `Ok(UserRegistrationResult)` - Successful registration with user ID and workflow tracking
/// * `Err(UserServiceError)` - Registration failed with specific error details
/// 
/// # Security
/// This function enforces:
/// - Email uniqueness within tenant boundaries
/// - Password strength requirements per tenant policy
/// - Rate limiting based on IP and tenant
/// - Audit logging of all registration attempts
/// 
/// # Business Rules
/// - New users start with 'pending' status until email verification
/// - Default role assignment based on tenant configuration
/// - Welcome email workflow is triggered asynchronously
/// 
/// # Example
/// ```rust
/// let result = user_service.register_user(
///     UserRegistrationData {
///         email: "user@example.com".to_string(),
///         password: "secure_password123".to_string(),
///         name: "John Doe".to_string(),
///     },
///     tenant_context,
///     request_metadata,
/// ).await?;
/// 
/// println!("User registered with ID: {}", result.user_id);
/// ```
pub async fn register_user(
    &self,
    registration_data: UserRegistrationData,
    tenant_context: TenantContext,
    request_metadata: RequestMetadata,
) -> Result<UserRegistrationResult, UserServiceError> {
    // Input validation with business context
    self.validate_registration_data(&registration_data, &tenant_context)
        .await
        .context("Registration data validation failed")?;
    
    // Check rate limits to prevent abuse
    self.enforce_registration_rate_limits(&request_metadata, &tenant_context)
        .await
        .context("Rate limit check failed")?;
    
    // Start the registration workflow (durable process)
    let workflow_result = self.temporal_client
        .start_workflow(
            WorkflowOptions::default()
                .task_queue("user-registration")
                .workflow_id(format!("user-reg-{}-{}", 
                    tenant_context.id, 
                    Uuid::new_v4()
                )),
            "user_registration_workflow",
            UserRegistrationWorkflowInput {
                registration_data,
                tenant_context,
                request_metadata,
            },
        )
        .await
        .context("Failed to start user registration workflow")?;
    
    Ok(UserRegistrationResult {
        workflow_id: workflow_result.workflow_id,
        status: RegistrationStatus::Processing,
        estimated_completion: Utc::now() + Duration::minutes(2),
    })
}
```

## 🧪 Testing Excellence

### Comprehensive Test Coverage
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;
    use crate::test_utils::*;
    
    /// Test fixture for common user registration scenarios
    #[fixture]
    fn valid_registration_data() -> UserRegistrationData {
        UserRegistrationData {
            email: "test@example.com".to_string(),
            password: "SecurePassword123!".to_string(),
            name: "Test User".to_string(),
        }
    }
    
    /// Test fixture for tenant context with default policies
    #[fixture]
    fn default_tenant_context() -> TenantContext {
        TenantContext {
            id: Uuid::new_v4(),
            name: "Test Tenant".to_string(),
            password_policy: PasswordPolicy::default(),
            rate_limits: RateLimits::default(),
            features: TenantFeatures::default(),
        }
    }
    
    #[rstest]
    #[tokio::test]
    async fn test_successful_user_registration(
        valid_registration_data: UserRegistrationData,
        default_tenant_context: TenantContext,
    ) {
        // Arrange
        let test_env = TestEnvironment::setup().await;
        let user_service = test_env.user_service();
        let request_metadata = RequestMetadata::test_default();
        
        // Act
        let result = user_service
            .register_user(
                valid_registration_data.clone(),
                default_tenant_context.clone(),
                request_metadata,
            )
            .await;
        
        // Assert
        assert!(result.is_ok());
        let registration_result = result.unwrap();
        assert!(!registration_result.workflow_id.is_empty());
        assert_eq!(registration_result.status, RegistrationStatus::Processing);
        
        // Verify workflow was created
        let workflow_info = test_env
            .temporal_client()
            .describe_workflow_execution(&registration_result.workflow_id)
            .await
            .expect("Workflow should exist");
        assert_eq!(workflow_info.workflow_type, "user_registration_workflow");
        
        // Verify audit log entry
        let audit_logs = test_env
            .audit_repository()
            .find_logs_by_tenant(default_tenant_context.id)
            .await
            .expect("Should retrieve audit logs");
        assert!(audit_logs.iter().any(|log| 
            log.action == "user_registration_attempted" &&
            log.metadata.contains(&valid_registration_data.email)
        ));
        
        test_env.cleanup().await;
    }
    
    #[rstest]
    #[case("", "Email cannot be empty")]
    #[case("invalid-email", "Invalid email format")]
    #[case("user@blocked-domain.com", "Email domain not allowed")]
    #[tokio::test]
    async fn test_registration_validation_errors(
        #[case] invalid_email: &str,
        #[case] expected_error: &str,
        default_tenant_context: TenantContext,
    ) {
        // Arrange
        let test_env = TestEnvironment::setup().await;
        let user_service = test_env.user_service();
        let mut registration_data = UserRegistrationData {
            email: invalid_email.to_string(),
            password: "SecurePassword123!".to_string(),
            name: "Test User".to_string(),
        };
        let request_metadata = RequestMetadata::test_default();
        
        // Act
        let result = user_service
            .register_user(registration_data, default_tenant_context, request_metadata)
            .await;
        
        // Assert
        assert!(result.is_err());
        let error = result.unwrap_err();
        match error {
            UserServiceError::ValidationError { field, reason } => {
                assert_eq!(field, "email");
                assert!(reason.contains(expected_error));
            }
            _ => panic!("Expected ValidationError, got: {:?}", error),
        }
        
        test_env.cleanup().await;
    }
    
    #[rstest]
    #[tokio::test]
    async fn test_duplicate_email_registration(
        valid_registration_data: UserRegistrationData,
        default_tenant_context: TenantContext,
    ) {
        // Arrange
        let test_env = TestEnvironment::setup().await;
        let user_service = test_env.user_service();
        let request_metadata = RequestMetadata::test_default();
        
        // Create initial user
        test_env
            .create_test_user(&valid_registration_data.email, default_tenant_context.id)
            .await;
        
        // Act - Try to register with same email
        let result = user_service
            .register_user(
                valid_registration_data.clone(),
                default_tenant_context,
                request_metadata,
            )
            .await;
        
        // Assert
        assert!(result.is_err());
        match result.unwrap_err() {
            UserServiceError::UserAlreadyExists { email } => {
                assert_eq!(email, valid_registration_data.email);
            }
            error => panic!("Expected UserAlreadyExists, got: {:?}", error),
        }
        
        test_env.cleanup().await;
    }
}
```

## 🎯 Key Style Guidelines

### 1. **Function Naming**: Action + Subject + Context
- `validate_user_registration_data()` not `validate_data()`
- `find_active_users_by_tenant()` not `get_users()`
- `schedule_billing_workflow()` not `schedule()`

### 2. **Error Messages**: Specific and Actionable
- ❌ "Invalid input"
- ✅ "Email address must be between 3 and 254 characters"

### 3. **Logging**: Context-Rich and Structured
```rust
tracing::info!(
    user_id = %user.id,
    tenant_id = %tenant.id,
    workflow_id = %workflow.id,
    duration_ms = elapsed.as_millis(),
    "User registration workflow completed successfully"
);
```

### 4. **Comments**: Why, Not What
```rust
// ❌ What
// Increment counter
counter += 1;

// ✅ Why
// Track failed attempts for rate limiting and security monitoring
failed_attempts.increment(&user_key);
```

### 5. **Configuration**: Explicit and Documented
```rust
/// Rate limiting configuration for user registration
#[derive(Debug, Clone)]
pub struct RegistrationRateLimits {
    /// Maximum registration attempts per IP address per hour
    /// Prevents automated account creation attacks
    pub per_ip_per_hour: u32,
    
    /// Maximum registration attempts per tenant per day
    /// Prevents abuse of tenant resources
    pub per_tenant_per_day: u32,
    
    /// Cooldown period after failed attempts (in seconds)
    /// Provides protection against brute force attacks
    pub cooldown_after_failures: u64,
}
```

This style guide ensures that all AI-generated code maintains enterprise-grade quality, readability, and maintainability while following ADX Core's architectural principles.
