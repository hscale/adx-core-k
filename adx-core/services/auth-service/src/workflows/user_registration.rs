use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use adx_shared::temporal::{
    WorkflowContext, ActivityContext, AdxActivity, TenantAwareActivity,
    ActivityError, WorkflowError, utils as activity_utils,
};
use adx_shared::types::{UserId, TenantId, SubscriptionTier};

/// User registration workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistrationRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub tenant_name: Option<String>,
    pub subscription_tier: Option<SubscriptionTier>,
    pub invite_token: Option<String>,
    pub referral_code: Option<String>,
}

/// User registration workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRegistrationResult {
    pub user_id: UserId,
    pub tenant_id: Option<TenantId>,
    pub verification_token: String,
    pub verification_required: bool,
    pub onboarding_required: bool,
    pub created_at: DateTime<Utc>,
}

/// Validate user registration request activity
pub struct ValidateUserRegistrationActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUserRegistrationInput {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub invite_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUserRegistrationOutput {
    pub is_valid: bool,
    pub validation_errors: Vec<ValidationError>,
    pub existing_user_id: Option<UserId>,
    pub invite_details: Option<InviteDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InviteDetails {
    pub tenant_id: TenantId,
    pub invited_by: UserId,
    pub roles: Vec<String>,
    pub expires_at: DateTime<Utc>,
}

impl AdxActivity<ValidateUserRegistrationInput, ValidateUserRegistrationOutput> for ValidateUserRegistrationActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: ValidateUserRegistrationInput,
    ) -> Result<ValidateUserRegistrationOutput, ActivityError> {
        let mut validation_errors = Vec::new();

        // Validate email format
        if !is_valid_email(&input.email) {
            validation_errors.push(ValidationError {
                field: "email".to_string(),
                code: "INVALID_FORMAT".to_string(),
                message: "Email address format is invalid".to_string(),
            });
        }

        // Validate password strength
        if !is_strong_password(&input.password) {
            validation_errors.push(ValidationError {
                field: "password".to_string(),
                code: "WEAK_PASSWORD".to_string(),
                message: "Password must be at least 8 characters long and contain uppercase, lowercase, number, and special character".to_string(),
            });
        }

        // Check if user already exists
        // TODO: Query database to check for existing user
        let existing_user_id = None;

        // Validate invite token if provided
        let invite_details = if let Some(token) = &input.invite_token {
            // TODO: Validate invite token and get details
            Some(InviteDetails {
                tenant_id: "default-tenant".to_string(),
                invited_by: "admin-user".to_string(),
                roles: vec!["user".to_string()],
                expires_at: Utc::now() + chrono::Duration::days(7),
            })
        } else {
            None
        };

        Ok(ValidateUserRegistrationOutput {
            is_valid: validation_errors.is_empty(),
            validation_errors,
            existing_user_id,
            invite_details,
        })
    }

    fn activity_type(&self) -> &'static str {
        "validate_user_registration"
    }
}

/// Create user account activity
pub struct CreateUserAccountActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserAccountInput {
    pub email: String,
    pub password_hash: String,
    pub display_name: Option<String>,
    pub verification_token: String,
    pub invite_details: Option<InviteDetails>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserAccountOutput {
    pub user_id: UserId,
    pub tenant_id: Option<TenantId>,
    pub created_at: DateTime<Utc>,
}

impl AdxActivity<CreateUserAccountInput, CreateUserAccountOutput> for CreateUserAccountActivity {
    async fn execute(
        &self,
        context: ActivityContext,
        input: CreateUserAccountInput,
    ) -> Result<CreateUserAccountOutput, ActivityError> {
        let user_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        // TODO: Insert user into database with tenant isolation
        // This would use the tenant context from the activity context
        tracing::info!(
            user_id = %user_id,
            email = %input.email,
            tenant_id = %context.tenant_context.tenant_id,
            "Creating user account"
        );

        // Determine tenant ID
        let tenant_id = if let Some(invite) = &input.invite_details {
            Some(invite.tenant_id.clone())
        } else {
            Some(context.tenant_context.tenant_id.clone())
        };

        Ok(CreateUserAccountOutput {
            user_id,
            tenant_id,
            created_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "create_user_account"
    }
}

impl TenantAwareActivity<CreateUserAccountInput, CreateUserAccountOutput> for CreateUserAccountActivity {
    async fn validate_tenant_access(
        &self,
        tenant_context: &adx_shared::temporal::TenantContext,
        user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Validate that the user has permission to create accounts in this tenant
        if !user_context.permissions.contains(&"user:create".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "Insufficient permissions to create user accounts".to_string(),
            });
        }

        Ok(())
    }

    async fn check_tenant_quotas(
        &self,
        tenant_context: &adx_shared::temporal::TenantContext,
        _resource_type: &str,
        _requested_amount: u64,
    ) -> Result<(), ActivityError> {
        // Check if tenant has reached user limit
        // TODO: Query current user count and compare with quota
        if tenant_context.quotas.max_users <= 0 {
            return Err(ActivityError::QuotaExceeded {
                message: "Tenant has reached maximum user limit".to_string(),
                current_usage: 0,
                limit: tenant_context.quotas.max_users as u64,
            });
        }

        Ok(())
    }
}

/// Send verification email activity
pub struct SendVerificationEmailActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVerificationEmailInput {
    pub user_id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub verification_token: String,
    pub tenant_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendVerificationEmailOutput {
    pub email_sent: bool,
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
}

impl AdxActivity<SendVerificationEmailInput, SendVerificationEmailOutput> for SendVerificationEmailActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: SendVerificationEmailInput,
    ) -> Result<SendVerificationEmailOutput, ActivityError> {
        // TODO: Send verification email using email service
        // This would integrate with an email provider like SendGrid, AWS SES, etc.
        
        let message_id = Uuid::new_v4().to_string();
        let sent_at = Utc::now();

        tracing::info!(
            user_id = %input.user_id,
            email = %input.email,
            message_id = %message_id,
            "Sending verification email"
        );

        // Simulate email sending
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(SendVerificationEmailOutput {
            email_sent: true,
            message_id,
            sent_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "send_verification_email"
    }
}

/// Create default tenant activity (if needed)
pub struct CreateDefaultTenantActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDefaultTenantInput {
    pub tenant_name: String,
    pub admin_user_id: UserId,
    pub subscription_tier: SubscriptionTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateDefaultTenantOutput {
    pub tenant_id: TenantId,
    pub created_at: DateTime<Utc>,
}

impl AdxActivity<CreateDefaultTenantInput, CreateDefaultTenantOutput> for CreateDefaultTenantActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: CreateDefaultTenantInput,
    ) -> Result<CreateDefaultTenantOutput, ActivityError> {
        let tenant_id = Uuid::new_v4().to_string();
        let created_at = Utc::now();

        // TODO: Create tenant in database
        tracing::info!(
            tenant_id = %tenant_id,
            tenant_name = %input.tenant_name,
            admin_user_id = %input.admin_user_id,
            "Creating default tenant"
        );

        Ok(CreateDefaultTenantOutput {
            tenant_id,
            created_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "create_default_tenant"
    }
}

/// User registration workflow implementation
pub async fn user_registration_workflow(
    _context: WorkflowContext,
    request: UserRegistrationRequest,
) -> Result<UserRegistrationResult, WorkflowError> {
    // Step 1: Validate registration request
    let validation_activity = ValidateUserRegistrationActivity;
    let validation_input = ValidateUserRegistrationInput {
        email: request.email.clone(),
        password: request.password.clone(),
        display_name: request.display_name.clone(),
        invite_token: request.invite_token.clone(),
    };

    // TODO: Call activity using Temporal SDK
    // For now, we'll simulate the activity call
    let validation_result = validation_activity.execute(
        ActivityContext {
            activity_id: activity_utils::generate_activity_id("validate_user_registration"),
            activity_type: "validate_user_registration".to_string(),
            workflow_id: "user-registration-workflow".to_string(),
            workflow_run_id: Uuid::new_v4().to_string(),
            attempt: 1,
            user_context: adx_shared::temporal::UserContext {
                user_id: "system".to_string(),
                email: "system@adxcore.com".to_string(),
                roles: vec!["system".to_string()],
                permissions: vec!["user:create".to_string()],
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
                tags: vec!["user_registration".to_string()],
                custom: std::collections::HashMap::new(),
            },
            heartbeat_details: None,
        },
        validation_input,
    ).await?;

    if !validation_result.is_valid {
        return Err(WorkflowError::ValidationFailed {
            errors: validation_result.validation_errors.into_iter()
                .map(|e| format!("{}: {}", e.field, e.message))
                .collect(),
        });
    }

    // Step 2: Hash password
    let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
        .map_err(|e| WorkflowError::ActivityFailed {
            activity_name: "hash_password".to_string(),
            error: format!("Failed to hash password: {}", e),
        })?;

    // Step 3: Generate verification token
    let verification_token = Uuid::new_v4().to_string();

    // Step 4: Create user account
    let create_user_activity = CreateUserAccountActivity;
    let create_user_input = CreateUserAccountInput {
        email: request.email.clone(),
        password_hash,
        display_name: request.display_name.clone(),
        verification_token: verification_token.clone(),
        invite_details: validation_result.invite_details.clone(),
    };

    let user_result = create_user_activity.execute(
        ActivityContext {
            activity_id: activity_utils::generate_activity_id("create_user_account"),
            activity_type: "create_user_account".to_string(),
            workflow_id: "user-registration-workflow".to_string(),
            workflow_run_id: Uuid::new_v4().to_string(),
            attempt: 1,
            user_context: adx_shared::temporal::UserContext {
                user_id: "system".to_string(),
                email: "system@adxcore.com".to_string(),
                roles: vec!["system".to_string()],
                permissions: vec!["user:create".to_string()],
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
                tags: vec!["user_registration".to_string()],
                custom: std::collections::HashMap::new(),
            },
            heartbeat_details: None,
        },
        create_user_input,
    ).await?;

    // Step 5: Create default tenant if needed
    let tenant_id = if request.tenant_name.is_some() && validation_result.invite_details.is_none() {
        let create_tenant_activity = CreateDefaultTenantActivity;
        let create_tenant_input = CreateDefaultTenantInput {
            tenant_name: request.tenant_name.unwrap_or_else(|| format!("{}'s Organization", request.email)),
            admin_user_id: user_result.user_id.clone(),
            subscription_tier: request.subscription_tier.unwrap_or(SubscriptionTier::Professional),
        };

        let tenant_result = create_tenant_activity.execute(
            ActivityContext {
                activity_id: activity_utils::generate_activity_id("create_default_tenant"),
                activity_type: "create_default_tenant".to_string(),
                workflow_id: "user-registration-workflow".to_string(),
                workflow_run_id: Uuid::new_v4().to_string(),
                attempt: 1,
                user_context: adx_shared::temporal::UserContext {
                    user_id: user_result.user_id.clone(),
                    email: request.email.clone(),
                    roles: vec!["admin".to_string()],
                    permissions: vec!["tenant:create".to_string()],
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
                    tags: vec!["user_registration".to_string()],
                    custom: std::collections::HashMap::new(),
                },
                heartbeat_details: None,
            },
            create_tenant_input,
        ).await?;

        Some(tenant_result.tenant_id)
    } else {
        user_result.tenant_id
    };

    // Step 6: Send verification email
    let send_email_activity = SendVerificationEmailActivity;
    let send_email_input = SendVerificationEmailInput {
        user_id: user_result.user_id.clone(),
        email: request.email.clone(),
        display_name: request.display_name.clone(),
        verification_token: verification_token.clone(),
        tenant_name: request.tenant_name.unwrap_or_else(|| "ADX Core".to_string()),
    };

    let _email_result = send_email_activity.execute(
        ActivityContext {
            activity_id: activity_utils::generate_activity_id("send_verification_email"),
            activity_type: "send_verification_email".to_string(),
            workflow_id: "user-registration-workflow".to_string(),
            workflow_run_id: Uuid::new_v4().to_string(),
            attempt: 1,
            user_context: adx_shared::temporal::UserContext {
                user_id: user_result.user_id.clone(),
                email: request.email.clone(),
                roles: vec!["user".to_string()],
                permissions: vec![],
                session_id: None,
                device_info: None,
            },
            tenant_context: adx_shared::temporal::TenantContext {
                tenant_id: tenant_id.clone().unwrap_or_else(|| "default".to_string()),
                tenant_name: request.tenant_name.unwrap_or_else(|| "Default".to_string()),
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
                retry_policy: Some(activity_utils::external_service_retry_policy()),
                tags: vec!["user_registration".to_string()],
                custom: std::collections::HashMap::new(),
            },
            heartbeat_details: None,
        },
        send_email_input,
    ).await?;

    Ok(UserRegistrationResult {
        user_id: user_result.user_id,
        tenant_id,
        verification_token,
        verification_required: true,
        onboarding_required: tenant_id.is_some(),
        created_at: user_result.created_at,
    })
}

// Helper functions
fn is_valid_email(email: &str) -> bool {
    email.contains('@') && email.contains('.') && email.len() > 5
}

fn is_strong_password(password: &str) -> bool {
    password.len() >= 8
        && password.chars().any(|c| c.is_uppercase())
        && password.chars().any(|c| c.is_lowercase())
        && password.chars().any(|c| c.is_numeric())
        && password.chars().any(|c| !c.is_alphanumeric())
}