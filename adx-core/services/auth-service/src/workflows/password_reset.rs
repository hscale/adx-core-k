use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;

use adx_shared::temporal::{
    WorkflowContext, ActivityContext, AdxActivity, TenantAwareActivity,
    ActivityError, WorkflowError, utils as activity_utils,
};
use adx_shared::types::UserId;

/// Password reset workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
    pub reset_url_base: Option<String>,
}

/// Password reset workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetResult {
    pub reset_token: String,
    pub user_id: Option<UserId>,
    pub email_sent: bool,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Validate password reset request activity
pub struct ValidatePasswordResetActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePasswordResetInput {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatePasswordResetOutput {
    pub user_exists: bool,
    pub user_id: Option<UserId>,
    pub user_active: bool,
    pub last_reset_request: Option<DateTime<Utc>>,
    pub rate_limit_exceeded: bool,
}

impl AdxActivity<ValidatePasswordResetInput, ValidatePasswordResetOutput> for ValidatePasswordResetActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: ValidatePasswordResetInput,
    ) -> Result<ValidatePasswordResetOutput, ActivityError> {
        // TODO: Query database to check if user exists and is active
        // For now, simulate user lookup
        
        tracing::info!(
            email = %input.email,
            "Validating password reset request"
        );

        // Simulate database lookup
        let user_exists = !input.email.is_empty();
        let user_id = if user_exists {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        };

        // Check rate limiting (max 3 requests per hour)
        // TODO: Check Redis or database for recent reset requests
        let rate_limit_exceeded = false;

        Ok(ValidatePasswordResetOutput {
            user_exists,
            user_id,
            user_active: true,
            last_reset_request: None,
            rate_limit_exceeded,
        })
    }

    fn activity_type(&self) -> &'static str {
        "validate_password_reset"
    }
}

/// Generate secure reset token activity
pub struct GenerateResetTokenActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResetTokenInput {
    pub user_id: UserId,
    pub email: String,
    pub expires_in_hours: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateResetTokenOutput {
    pub reset_token: String,
    pub token_hash: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AdxActivity<GenerateResetTokenInput, GenerateResetTokenOutput> for GenerateResetTokenActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: GenerateResetTokenInput,
    ) -> Result<GenerateResetTokenOutput, ActivityError> {
        let created_at = Utc::now();
        let expires_at = created_at + Duration::hours(input.expires_in_hours as i64);
        
        // Generate cryptographically secure token
        let reset_token = generate_secure_token(32);
        
        // Hash the token for storage (never store plain tokens)
        let token_hash = bcrypt::hash(&reset_token, bcrypt::DEFAULT_COST)
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to hash reset token: {}", e),
            })?;

        // TODO: Store token hash in database with expiration
        tracing::info!(
            user_id = %input.user_id,
            email = %input.email,
            expires_at = %expires_at,
            "Generated password reset token"
        );

        Ok(GenerateResetTokenOutput {
            reset_token,
            token_hash,
            expires_at,
            created_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "generate_reset_token"
    }
}

impl TenantAwareActivity<GenerateResetTokenInput, GenerateResetTokenOutput> for GenerateResetTokenActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        _user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Password reset is allowed for all users
        Ok(())
    }
}

/// Send password reset email activity
pub struct SendPasswordResetEmailActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPasswordResetEmailInput {
    pub user_id: UserId,
    pub email: String,
    pub reset_token: String,
    pub reset_url_base: String,
    pub expires_at: DateTime<Utc>,
    pub tenant_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPasswordResetEmailOutput {
    pub email_sent: bool,
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
}

impl AdxActivity<SendPasswordResetEmailInput, SendPasswordResetEmailOutput> for SendPasswordResetEmailActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: SendPasswordResetEmailInput,
    ) -> Result<SendPasswordResetEmailOutput, ActivityError> {
        let message_id = Uuid::new_v4().to_string();
        let sent_at = Utc::now();

        // Construct reset URL
        let reset_url = format!("{}?token={}&email={}", 
            input.reset_url_base, 
            input.reset_token, 
            urlencoding::encode(&input.email)
        );

        // TODO: Send email using email service provider
        // This would integrate with SendGrid, AWS SES, etc.
        tracing::info!(
            user_id = %input.user_id,
            email = %input.email,
            message_id = %message_id,
            expires_at = %input.expires_at,
            "Sending password reset email"
        );

        // Simulate email sending delay
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(SendPasswordResetEmailOutput {
            email_sent: true,
            message_id,
            sent_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "send_password_reset_email"
    }
}

/// Invalidate existing reset tokens activity
pub struct InvalidateExistingTokensActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateExistingTokensInput {
    pub user_id: UserId,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateExistingTokensOutput {
    pub tokens_invalidated: u32,
    pub invalidated_at: DateTime<Utc>,
}

impl AdxActivity<InvalidateExistingTokensInput, InvalidateExistingTokensOutput> for InvalidateExistingTokensActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: InvalidateExistingTokensInput,
    ) -> Result<InvalidateExistingTokensOutput, ActivityError> {
        let invalidated_at = Utc::now();

        // TODO: Invalidate all existing reset tokens for this user
        // This prevents multiple active reset tokens
        tracing::info!(
            user_id = %input.user_id,
            email = %input.email,
            "Invalidating existing password reset tokens"
        );

        Ok(InvalidateExistingTokensOutput {
            tokens_invalidated: 0, // Would be actual count from database
            invalidated_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "invalidate_existing_tokens"
    }
}

/// Log security event activity
pub struct LogSecurityEventActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSecurityEventInput {
    pub event_type: String,
    pub user_id: Option<UserId>,
    pub email: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSecurityEventOutput {
    pub event_id: String,
    pub logged_at: DateTime<Utc>,
}

impl AdxActivity<LogSecurityEventInput, LogSecurityEventOutput> for LogSecurityEventActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: LogSecurityEventInput,
    ) -> Result<LogSecurityEventOutput, ActivityError> {
        let event_id = Uuid::new_v4().to_string();
        let logged_at = Utc::now();

        // TODO: Log security event to audit system
        tracing::warn!(
            event_id = %event_id,
            event_type = %input.event_type,
            user_id = ?input.user_id,
            email = %input.email,
            ip_address = ?input.ip_address,
            details = %input.details,
            "Security event logged"
        );

        Ok(LogSecurityEventOutput {
            event_id,
            logged_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "log_security_event"
    }
}

/// Password reset workflow implementation
pub async fn password_reset_workflow(
    _context: WorkflowContext,
    request: PasswordResetRequest,
) -> Result<PasswordResetResult, WorkflowError> {
    let created_at = Utc::now();

    // Step 1: Validate password reset request
    let validation_activity = ValidatePasswordResetActivity;
    let validation_input = ValidatePasswordResetInput {
        email: request.email.clone(),
    };

    let validation_result = validation_activity.execute(
        create_activity_context("validate_password_reset", "password-reset-workflow"),
        validation_input,
    ).await?;

    // Check rate limiting
    if validation_result.rate_limit_exceeded {
        // Log security event for rate limit exceeded
        let log_activity = LogSecurityEventActivity;
        let log_input = LogSecurityEventInput {
            event_type: "password_reset_rate_limit_exceeded".to_string(),
            user_id: validation_result.user_id.clone(),
            email: request.email.clone(),
            ip_address: None, // TODO: Extract from workflow context
            user_agent: None, // TODO: Extract from workflow context
            details: serde_json::json!({
                "last_reset_request": validation_result.last_reset_request,
                "message": "Password reset rate limit exceeded"
            }),
        };

        let _log_result = log_activity.execute(
            create_activity_context("log_security_event", "password-reset-workflow"),
            log_input,
        ).await?;

        return Err(WorkflowError::RateLimitExceeded {
            message: "Too many password reset requests. Please try again later.".to_string(),
            retry_after: Duration::hours(1),
        });
    }

    // Always continue with the workflow to prevent email enumeration
    // Even if user doesn't exist, we'll generate a token but not send email
    
    let (reset_token, expires_at, user_id) = if validation_result.user_exists && validation_result.user_active {
        let user_id = validation_result.user_id.unwrap();

        // Step 2: Invalidate existing reset tokens
        let invalidate_activity = InvalidateExistingTokensActivity;
        let invalidate_input = InvalidateExistingTokensInput {
            user_id: user_id.clone(),
            email: request.email.clone(),
        };

        let _invalidate_result = invalidate_activity.execute(
            create_activity_context("invalidate_existing_tokens", "password-reset-workflow"),
            invalidate_input,
        ).await?;

        // Step 3: Generate secure reset token
        let generate_token_activity = GenerateResetTokenActivity;
        let generate_token_input = GenerateResetTokenInput {
            user_id: user_id.clone(),
            email: request.email.clone(),
            expires_in_hours: 24, // Token expires in 24 hours
        };

        let token_result = generate_token_activity.execute(
            create_activity_context("generate_reset_token", "password-reset-workflow"),
            generate_token_input,
        ).await?;

        // Step 4: Send password reset email
        let send_email_activity = SendPasswordResetEmailActivity;
        let send_email_input = SendPasswordResetEmailInput {
            user_id: user_id.clone(),
            email: request.email.clone(),
            reset_token: token_result.reset_token.clone(),
            reset_url_base: request.reset_url_base.unwrap_or_else(|| "https://app.adxcore.com/reset-password".to_string()),
            expires_at: token_result.expires_at,
            tenant_name: "ADX Core".to_string(), // TODO: Get from tenant context
        };

        let _email_result = send_email_activity.execute(
            create_activity_context("send_password_reset_email", "password-reset-workflow"),
            send_email_input,
        ).await?;

        // Step 5: Log successful password reset request
        let log_activity = LogSecurityEventActivity;
        let log_input = LogSecurityEventInput {
            event_type: "password_reset_requested".to_string(),
            user_id: Some(user_id.clone()),
            email: request.email.clone(),
            ip_address: None, // TODO: Extract from workflow context
            user_agent: None, // TODO: Extract from workflow context
            details: serde_json::json!({
                "reset_token_expires_at": token_result.expires_at,
                "message": "Password reset token generated and email sent"
            }),
        };

        let _log_result = log_activity.execute(
            create_activity_context("log_security_event", "password-reset-workflow"),
            log_input,
        ).await?;

        (token_result.reset_token, token_result.expires_at, Some(user_id))
    } else {
        // User doesn't exist or is inactive - generate dummy token but don't send email
        let dummy_token = generate_secure_token(32);
        let dummy_expires = created_at + Duration::hours(24);

        // Log security event for non-existent user
        let log_activity = LogSecurityEventActivity;
        let log_input = LogSecurityEventInput {
            event_type: "password_reset_nonexistent_user".to_string(),
            user_id: None,
            email: request.email.clone(),
            ip_address: None, // TODO: Extract from workflow context
            user_agent: None, // TODO: Extract from workflow context
            details: serde_json::json!({
                "message": "Password reset requested for non-existent user",
                "user_exists": validation_result.user_exists,
                "user_active": validation_result.user_active
            }),
        };

        let _log_result = log_activity.execute(
            create_activity_context("log_security_event", "password-reset-workflow"),
            log_input,
        ).await?;

        (dummy_token, dummy_expires, None)
    };

    Ok(PasswordResetResult {
        reset_token,
        user_id,
        email_sent: validation_result.user_exists && validation_result.user_active,
        expires_at,
        created_at,
    })
}

/// Confirm password reset workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmPasswordResetRequest {
    pub email: String,
    pub reset_token: String,
    pub new_password: String,
}

/// Confirm password reset workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmPasswordResetResult {
    pub success: bool,
    pub user_id: Option<UserId>,
    pub sessions_invalidated: u32,
    pub completed_at: DateTime<Utc>,
}

/// Validate reset token activity
pub struct ValidateResetTokenActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResetTokenInput {
    pub email: String,
    pub reset_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateResetTokenOutput {
    pub valid: bool,
    pub user_id: Option<UserId>,
    pub expired: bool,
    pub already_used: bool,
}

impl AdxActivity<ValidateResetTokenInput, ValidateResetTokenOutput> for ValidateResetTokenActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: ValidateResetTokenInput,
    ) -> Result<ValidateResetTokenOutput, ActivityError> {
        // TODO: Query database to validate reset token
        // Check if token exists, not expired, and not already used
        
        tracing::info!(
            email = %input.email,
            "Validating password reset token"
        );

        // Simulate token validation
        let valid = !input.reset_token.is_empty();
        let user_id = if valid {
            Some(Uuid::new_v4().to_string())
        } else {
            None
        };

        Ok(ValidateResetTokenOutput {
            valid,
            user_id,
            expired: false,
            already_used: false,
        })
    }

    fn activity_type(&self) -> &'static str {
        "validate_reset_token"
    }
}

/// Update user password activity
pub struct UpdateUserPasswordActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPasswordInput {
    pub user_id: UserId,
    pub new_password_hash: String,
    pub reset_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserPasswordOutput {
    pub password_updated: bool,
    pub updated_at: DateTime<Utc>,
}

impl AdxActivity<UpdateUserPasswordInput, UpdateUserPasswordOutput> for UpdateUserPasswordActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: UpdateUserPasswordInput,
    ) -> Result<UpdateUserPasswordOutput, ActivityError> {
        let updated_at = Utc::now();

        // TODO: Update user password in database and mark reset token as used
        tracing::info!(
            user_id = %input.user_id,
            "Updating user password"
        );

        Ok(UpdateUserPasswordOutput {
            password_updated: true,
            updated_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "update_user_password"
    }
}

impl TenantAwareActivity<UpdateUserPasswordInput, UpdateUserPasswordOutput> for UpdateUserPasswordActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        _user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Password updates are allowed for the user themselves
        Ok(())
    }
}

/// Invalidate user sessions activity
pub struct InvalidateUserSessionsActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateUserSessionsInput {
    pub user_id: UserId,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidateUserSessionsOutput {
    pub sessions_invalidated: u32,
    pub invalidated_at: DateTime<Utc>,
}

impl AdxActivity<InvalidateUserSessionsInput, InvalidateUserSessionsOutput> for InvalidateUserSessionsActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: InvalidateUserSessionsInput,
    ) -> Result<InvalidateUserSessionsOutput, ActivityError> {
        let invalidated_at = Utc::now();

        // TODO: Invalidate all active sessions for this user
        tracing::info!(
            user_id = %input.user_id,
            reason = %input.reason,
            "Invalidating user sessions"
        );

        Ok(InvalidateUserSessionsOutput {
            sessions_invalidated: 0, // Would be actual count from database
            invalidated_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "invalidate_user_sessions"
    }
}

/// Confirm password reset workflow implementation
pub async fn confirm_password_reset_workflow(
    _context: WorkflowContext,
    request: ConfirmPasswordResetRequest,
) -> Result<ConfirmPasswordResetResult, WorkflowError> {
    let completed_at = Utc::now();

    // Step 1: Validate reset token
    let validate_token_activity = ValidateResetTokenActivity;
    let validate_token_input = ValidateResetTokenInput {
        email: request.email.clone(),
        reset_token: request.reset_token.clone(),
    };

    let token_validation = validate_token_activity.execute(
        create_activity_context("validate_reset_token", "confirm-password-reset-workflow"),
        validate_token_input,
    ).await?;

    if !token_validation.valid {
        // Log security event for invalid token
        let log_activity = LogSecurityEventActivity;
        let log_input = LogSecurityEventInput {
            event_type: "password_reset_invalid_token".to_string(),
            user_id: None,
            email: request.email.clone(),
            ip_address: None,
            user_agent: None,
            details: serde_json::json!({
                "expired": token_validation.expired,
                "already_used": token_validation.already_used,
                "message": "Invalid password reset token used"
            }),
        };

        let _log_result = log_activity.execute(
            create_activity_context("log_security_event", "confirm-password-reset-workflow"),
            log_input,
        ).await?;

        return Err(WorkflowError::ValidationFailed {
            errors: vec!["Reset token is invalid, expired, or already used".to_string()],
        });
    }

    let user_id = token_validation.user_id.unwrap();

    // Step 2: Validate new password strength
    if !is_strong_password(&request.new_password) {
        return Err(WorkflowError::ValidationFailed {
            errors: vec!["Password must be at least 8 characters long and contain uppercase, lowercase, number, and special character".to_string()],
        });
    }

    // Step 3: Hash new password
    let new_password_hash = bcrypt::hash(&request.new_password, bcrypt::DEFAULT_COST)
        .map_err(|e| WorkflowError::ActivityFailed {
            activity_name: "hash_password".to_string(),
            error: format!("Failed to hash new password: {}", e),
        })?;

    // Step 4: Update user password
    let update_password_activity = UpdateUserPasswordActivity;
    let update_password_input = UpdateUserPasswordInput {
        user_id: user_id.clone(),
        new_password_hash,
        reset_token: request.reset_token.clone(),
    };

    let _password_result = update_password_activity.execute(
        create_activity_context("update_user_password", "confirm-password-reset-workflow"),
        update_password_input,
    ).await?;

    // Step 5: Invalidate all user sessions
    let invalidate_sessions_activity = InvalidateUserSessionsActivity;
    let invalidate_sessions_input = InvalidateUserSessionsInput {
        user_id: user_id.clone(),
        reason: "Password reset completed".to_string(),
    };

    let sessions_result = invalidate_sessions_activity.execute(
        create_activity_context("invalidate_user_sessions", "confirm-password-reset-workflow"),
        invalidate_sessions_input,
    ).await?;

    // Step 6: Log successful password reset
    let log_activity = LogSecurityEventActivity;
    let log_input = LogSecurityEventInput {
        event_type: "password_reset_completed".to_string(),
        user_id: Some(user_id.clone()),
        email: request.email.clone(),
        ip_address: None,
        user_agent: None,
        details: serde_json::json!({
            "sessions_invalidated": sessions_result.sessions_invalidated,
            "message": "Password reset completed successfully"
        }),
    };

    let _log_result = log_activity.execute(
        create_activity_context("log_security_event", "confirm-password-reset-workflow"),
        log_input,
    ).await?;

    Ok(ConfirmPasswordResetResult {
        success: true,
        user_id: Some(user_id),
        sessions_invalidated: sessions_result.sessions_invalidated,
        completed_at,
    })
}

// Helper functions
fn generate_secure_token(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn is_strong_password(password: &str) -> bool {
    password.len() >= 8
        && password.chars().any(|c| c.is_uppercase())
        && password.chars().any(|c| c.is_lowercase())
        && password.chars().any(|c| c.is_numeric())
        && password.chars().any(|c| !c.is_alphanumeric())
}

fn create_activity_context(activity_type: &str, workflow_id: &str) -> ActivityContext {
    ActivityContext {
        activity_id: activity_utils::generate_activity_id(activity_type),
        activity_type: activity_type.to_string(),
        workflow_id: workflow_id.to_string(),
        workflow_run_id: Uuid::new_v4().to_string(),
        attempt: 1,
        user_context: adx_shared::temporal::UserContext {
            user_id: "system".to_string(),
            email: "system@adxcore.com".to_string(),
            roles: vec!["system".to_string()],
            permissions: vec!["password:reset".to_string()],
            session_id: None,
            device_info: None,
        },
        tenant_context: adx_shared::temporal::TenantContext {
            tenant_id: "default".to_string(),
            tenant_name: "Default".to_string(),
            subscription_tier: adx_shared::temporal::SubscriptionTier::Professional,
            features: vec![],
            quotas: adx_shared::temporal::TenantQuotas {
                max_users: 100,
                max_storage_gb: 1000,
                max_api_calls_per_hour: 10000,
                max_concurrent_workflows: 50,
                max_file_upload_size_mb: 100,
            },
            settings: adx_shared::temporal::TenantSettings {
                default_language: "en".to_string(),
                timezone: "UTC".to_string(),
                date_format: "YYYY-MM-DD".to_string(),
                currency: "USD".to_string(),
                branding: None,
            },
            isolation_level: adx_shared::temporal::TenantIsolationLevel::Schema,
        },
        metadata: adx_shared::temporal::ActivityMetadata {
            start_time: Utc::now(),
            timeout: std::time::Duration::from_secs(300),
            heartbeat_timeout: Some(std::time::Duration::from_secs(30)),
            retry_policy: Some(activity_utils::database_retry_policy()),
            tags: vec!["password_reset".to_string()],
            custom: std::collections::HashMap::new(),
        },
        heartbeat_details: None,
    }
}