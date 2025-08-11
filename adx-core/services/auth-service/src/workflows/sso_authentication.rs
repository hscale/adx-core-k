use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use base64::prelude::*;

use adx_shared::temporal::{
    WorkflowContext, ActivityContext, AdxActivity, TenantAwareActivity,
    ActivityError, WorkflowError, utils as activity_utils,
};
use adx_shared::types::{UserId, TenantId};

/// SSO authentication workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoAuthenticationRequest {
    pub provider: SsoProvider,
    pub tenant_id: Option<TenantId>,
    pub authorization_code: Option<String>,
    pub state: Option<String>,
    pub redirect_uri: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

/// SSO provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SsoProvider {
    Google,
    Microsoft,
    Okta,
    Auth0,
    Saml { entity_id: String },
    Oidc { issuer_url: String },
}

/// SSO authentication workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoAuthenticationResult {
    pub success: bool,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<TenantId>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub user_created: bool,
    pub user_profile: Option<SsoUserProfile>,
    pub session_id: Option<String>,
    pub expires_in: Option<i64>,
    pub completed_at: DateTime<Utc>,
}

/// SSO user profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoUserProfile {
    pub external_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub verified_email: bool,
    pub locale: Option<String>,
    pub timezone: Option<String>,
}

/// Exchange authorization code activity
pub struct ExchangeAuthorizationCodeActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAuthorizationCodeInput {
    pub provider: SsoProvider,
    pub authorization_code: String,
    pub redirect_uri: String,
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeAuthorizationCodeOutput {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub expires_in: i64,
    pub token_type: String,
    pub scope: Option<String>,
    pub exchanged_at: DateTime<Utc>,
}

impl AdxActivity<ExchangeAuthorizationCodeInput, ExchangeAuthorizationCodeOutput> for ExchangeAuthorizationCodeActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: ExchangeAuthorizationCodeInput,
    ) -> Result<ExchangeAuthorizationCodeOutput, ActivityError> {
        let exchanged_at = Utc::now();

        // TODO: Exchange authorization code with the SSO provider
        // This would make HTTP requests to the provider's token endpoint
        tracing::info!(
            provider = ?input.provider,
            "Exchanging authorization code for tokens"
        );

        // Simulate token exchange
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Mock response - in real implementation, this would come from the provider
        Ok(ExchangeAuthorizationCodeOutput {
            access_token: generate_mock_token(),
            refresh_token: Some(generate_mock_token()),
            id_token: Some(generate_mock_jwt()),
            expires_in: 3600, // 1 hour
            token_type: "Bearer".to_string(),
            scope: Some("openid profile email".to_string()),
            exchanged_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "exchange_authorization_code"
    }
}

/// Fetch user profile from SSO provider activity
pub struct FetchSsoUserProfileActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSsoUserProfileInput {
    pub provider: SsoProvider,
    pub access_token: String,
    pub id_token: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchSsoUserProfileOutput {
    pub user_profile: SsoUserProfile,
    pub fetched_at: DateTime<Utc>,
}

impl AdxActivity<FetchSsoUserProfileInput, FetchSsoUserProfileOutput> for FetchSsoUserProfileActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: FetchSsoUserProfileInput,
    ) -> Result<FetchSsoUserProfileOutput, ActivityError> {
        let fetched_at = Utc::now();

        // TODO: Fetch user profile from SSO provider using access token
        // This would make HTTP requests to the provider's userinfo endpoint
        tracing::info!(
            provider = ?input.provider,
            "Fetching user profile from SSO provider"
        );

        // Simulate profile fetch
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        // Mock user profile - in real implementation, this would come from the provider
        let user_profile = match input.provider {
            SsoProvider::Google => SsoUserProfile {
                external_id: "google_123456789".to_string(),
                email: "user@gmail.com".to_string(),
                display_name: Some("John Doe".to_string()),
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                avatar_url: Some("https://lh3.googleusercontent.com/a/default-user".to_string()),
                verified_email: true,
                locale: Some("en".to_string()),
                timezone: Some("America/New_York".to_string()),
            },
            SsoProvider::Microsoft => SsoUserProfile {
                external_id: "microsoft_123456789".to_string(),
                email: "user@outlook.com".to_string(),
                display_name: Some("Jane Smith".to_string()),
                first_name: Some("Jane".to_string()),
                last_name: Some("Smith".to_string()),
                avatar_url: None,
                verified_email: true,
                locale: Some("en-US".to_string()),
                timezone: Some("UTC-05:00".to_string()),
            },
            _ => SsoUserProfile {
                external_id: "sso_123456789".to_string(),
                email: "user@example.com".to_string(),
                display_name: Some("SSO User".to_string()),
                first_name: Some("SSO".to_string()),
                last_name: Some("User".to_string()),
                avatar_url: None,
                verified_email: true,
                locale: Some("en".to_string()),
                timezone: None,
            },
        };

        Ok(FetchSsoUserProfileOutput {
            user_profile,
            fetched_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "fetch_sso_user_profile"
    }
}

/// Find or create user activity
pub struct FindOrCreateSsoUserActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindOrCreateSsoUserInput {
    pub sso_profile: SsoUserProfile,
    pub provider: SsoProvider,
    pub tenant_id: Option<TenantId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindOrCreateSsoUserOutput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub user_created: bool,
    pub user_updated: bool,
    pub processed_at: DateTime<Utc>,
}

impl AdxActivity<FindOrCreateSsoUserInput, FindOrCreateSsoUserOutput> for FindOrCreateSsoUserActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: FindOrCreateSsoUserInput,
    ) -> Result<FindOrCreateSsoUserOutput, ActivityError> {
        let processed_at = Utc::now();

        // TODO: Query database to find existing user by email or external ID
        // If not found, create new user with SSO profile information
        
        tracing::info!(
            email = %input.sso_profile.email,
            external_id = %input.sso_profile.external_id,
            provider = ?input.provider,
            "Finding or creating SSO user"
        );

        // Simulate database operations
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Mock user creation - in real implementation, this would query/create in database
        let user_id = Uuid::new_v4().to_string();
        let tenant_id = input.tenant_id.unwrap_or_else(|| "default-tenant".to_string());
        let user_created = true; // Assume new user for this mock

        Ok(FindOrCreateSsoUserOutput {
            user_id,
            tenant_id,
            user_created,
            user_updated: false,
            processed_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "find_or_create_sso_user"
    }
}

impl TenantAwareActivity<FindOrCreateSsoUserInput, FindOrCreateSsoUserOutput> for FindOrCreateSsoUserActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        _user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // SSO user creation is allowed
        Ok(())
    }
}

/// Create SSO session activity
pub struct CreateSsoSessionActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSsoSessionInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub provider: SsoProvider,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSsoSessionOutput {
    pub session_id: String,
    pub jwt_token: String,
    pub refresh_token: String,
    pub expires_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

impl AdxActivity<CreateSsoSessionInput, CreateSsoSessionOutput> for CreateSsoSessionActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: CreateSsoSessionInput,
    ) -> Result<CreateSsoSessionOutput, ActivityError> {
        let created_at = Utc::now();
        let expires_at = created_at + chrono::Duration::seconds(input.expires_in);
        let session_id = Uuid::new_v4().to_string();

        // TODO: Create session in database and generate JWT token
        tracing::info!(
            user_id = %input.user_id,
            tenant_id = %input.tenant_id,
            provider = ?input.provider,
            session_id = %session_id,
            "Creating SSO session"
        );

        // Mock JWT token generation
        let jwt_token = generate_mock_jwt();
        let refresh_token = generate_mock_token();

        Ok(CreateSsoSessionOutput {
            session_id,
            jwt_token,
            refresh_token,
            expires_at,
            created_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "create_sso_session"
    }
}

impl TenantAwareActivity<CreateSsoSessionInput, CreateSsoSessionOutput> for CreateSsoSessionActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        _user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Session creation is allowed for authenticated users
        Ok(())
    }
}

/// Log SSO authentication event activity
pub struct LogSsoAuthEventActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSsoAuthEventInput {
    pub event_type: String,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<TenantId>,
    pub provider: SsoProvider,
    pub success: bool,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub error_message: Option<String>,
    pub additional_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogSsoAuthEventOutput {
    pub event_id: String,
    pub logged_at: DateTime<Utc>,
}

impl AdxActivity<LogSsoAuthEventInput, LogSsoAuthEventOutput> for LogSsoAuthEventActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: LogSsoAuthEventInput,
    ) -> Result<LogSsoAuthEventOutput, ActivityError> {
        let event_id = Uuid::new_v4().to_string();
        let logged_at = Utc::now();

        // TODO: Log authentication event to audit system
        tracing::info!(
            event_id = %event_id,
            event_type = %input.event_type,
            user_id = ?input.user_id,
            tenant_id = ?input.tenant_id,
            provider = ?input.provider,
            success = input.success,
            client_ip = ?input.client_ip,
            error_message = ?input.error_message,
            "SSO authentication event logged"
        );

        Ok(LogSsoAuthEventOutput {
            event_id,
            logged_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "log_sso_auth_event"
    }
}

/// Validate SSO state parameter activity
pub struct ValidateSsoStateActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateSsoStateInput {
    pub state: String,
    pub expected_state: Option<String>,
    pub tenant_id: Option<TenantId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateSsoStateOutput {
    pub state_valid: bool,
    pub tenant_id: Option<TenantId>,
    pub validated_at: DateTime<Utc>,
}

impl AdxActivity<ValidateSsoStateInput, ValidateSsoStateOutput> for ValidateSsoStateActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: ValidateSsoStateInput,
    ) -> Result<ValidateSsoStateOutput, ActivityError> {
        let validated_at = Utc::now();

        // TODO: Validate state parameter to prevent CSRF attacks
        // State should be stored in session or database and matched
        tracing::info!(
            state = %input.state,
            tenant_id = ?input.tenant_id,
            "Validating SSO state parameter"
        );

        // Mock validation - in real implementation, this would check against stored state
        let state_valid = !input.state.is_empty();

        Ok(ValidateSsoStateOutput {
            state_valid,
            tenant_id: input.tenant_id,
            validated_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "validate_sso_state"
    }
}

/// SSO authentication workflow implementation
pub async fn sso_authentication_workflow(
    _context: WorkflowContext,
    request: SsoAuthenticationRequest,
) -> Result<SsoAuthenticationResult, WorkflowError> {
    let completed_at = Utc::now();

    // Step 1: Validate state parameter (CSRF protection)
    if let Some(state) = &request.state {
        let validate_state_activity = ValidateSsoStateActivity;
        let validate_state_input = ValidateSsoStateInput {
            state: state.clone(),
            expected_state: None, // TODO: Get from session/database
            tenant_id: request.tenant_id.clone(),
        };

        let state_validation = validate_state_activity.execute(
            create_activity_context("validate_sso_state", "sso-authentication-workflow"),
            validate_state_input,
        ).await?;

        if !state_validation.state_valid {
            // Log security event
            let log_activity = LogSsoAuthEventActivity;
            let log_input = LogSsoAuthEventInput {
                event_type: "sso_invalid_state".to_string(),
                user_id: None,
                tenant_id: request.tenant_id.clone(),
                provider: request.provider.clone(),
                success: false,
                client_ip: request.client_ip.clone(),
                user_agent: request.user_agent.clone(),
                error_message: Some("Invalid state parameter".to_string()),
                additional_data: HashMap::new(),
            };

            let _log_result = log_activity.execute(
                create_activity_context("log_sso_auth_event", "sso-authentication-workflow"),
                log_input,
            ).await?;

            return Err(WorkflowError::SecurityViolation {
                message: "Invalid state parameter - possible CSRF attack".to_string(),
            });
        }
    }

    // Step 2: Exchange authorization code for tokens
    let authorization_code = request.authorization_code.ok_or_else(|| WorkflowError::ValidationFailed {
        errors: vec!["authorization_code field is required".to_string()],
    })?;

    let exchange_code_activity = ExchangeAuthorizationCodeActivity;
    let exchange_code_input = ExchangeAuthorizationCodeInput {
        provider: request.provider.clone(),
        authorization_code,
        redirect_uri: request.redirect_uri.clone(),
        client_id: get_sso_client_id(&request.provider),
        client_secret: get_sso_client_secret(&request.provider),
    };

    let token_result = exchange_code_activity.execute(
        create_activity_context("exchange_authorization_code", "sso-authentication-workflow"),
        exchange_code_input,
    ).await.map_err(|e| {
        // Log failed token exchange
        let log_activity = LogSsoAuthEventActivity;
        let log_input = LogSsoAuthEventInput {
            event_type: "sso_token_exchange_failed".to_string(),
            user_id: None,
            tenant_id: request.tenant_id.clone(),
            provider: request.provider.clone(),
            success: false,
            client_ip: request.client_ip.clone(),
            user_agent: request.user_agent.clone(),
            error_message: Some(format!("Token exchange failed: {}", e)),
            additional_data: HashMap::new(),
        };

        // Note: We can't await here due to error handling, so we'll skip logging in error case
        WorkflowError::ActivityFailed {
            activity_name: "exchange_authorization_code".to_string(),
            error: format!("Failed to exchange authorization code: {}", e),
        }
    })?;

    // Step 3: Fetch user profile from SSO provider
    let fetch_profile_activity = FetchSsoUserProfileActivity;
    let fetch_profile_input = FetchSsoUserProfileInput {
        provider: request.provider.clone(),
        access_token: token_result.access_token.clone(),
        id_token: token_result.id_token.clone(),
    };

    let profile_result = fetch_profile_activity.execute(
        create_activity_context("fetch_sso_user_profile", "sso-authentication-workflow"),
        fetch_profile_input,
    ).await?;

    // Step 4: Find or create user
    let find_create_user_activity = FindOrCreateSsoUserActivity;
    let find_create_user_input = FindOrCreateSsoUserInput {
        sso_profile: profile_result.user_profile.clone(),
        provider: request.provider.clone(),
        tenant_id: request.tenant_id.clone(),
    };

    let user_result = find_create_user_activity.execute(
        create_activity_context("find_or_create_sso_user", "sso-authentication-workflow"),
        find_create_user_input,
    ).await?;

    // Step 5: Create session
    let create_session_activity = CreateSsoSessionActivity;
    let create_session_input = CreateSsoSessionInput {
        user_id: user_result.user_id.clone(),
        tenant_id: user_result.tenant_id.clone(),
        provider: request.provider.clone(),
        access_token: token_result.access_token.clone(),
        refresh_token: token_result.refresh_token.clone(),
        expires_in: token_result.expires_in,
        client_ip: request.client_ip.clone(),
        user_agent: request.user_agent.clone(),
    };

    let session_result = create_session_activity.execute(
        create_activity_context("create_sso_session", "sso-authentication-workflow"),
        create_session_input,
    ).await?;

    // Step 6: Log successful authentication
    let log_activity = LogSsoAuthEventActivity;
    let log_input = LogSsoAuthEventInput {
        event_type: "sso_authentication_success".to_string(),
        user_id: Some(user_result.user_id.clone()),
        tenant_id: Some(user_result.tenant_id.clone()),
        provider: request.provider.clone(),
        success: true,
        client_ip: request.client_ip.clone(),
        user_agent: request.user_agent.clone(),
        error_message: None,
        additional_data: {
            let mut data = HashMap::new();
            data.insert("user_created".to_string(), serde_json::Value::Bool(user_result.user_created));
            data.insert("session_id".to_string(), serde_json::Value::String(session_result.session_id.clone()));
            data
        },
    };

    let _log_result = log_activity.execute(
        create_activity_context("log_sso_auth_event", "sso-authentication-workflow"),
        log_input,
    ).await?;

    Ok(SsoAuthenticationResult {
        success: true,
        user_id: Some(user_result.user_id),
        tenant_id: Some(user_result.tenant_id),
        access_token: Some(session_result.jwt_token),
        refresh_token: Some(session_result.refresh_token),
        user_created: user_result.user_created,
        user_profile: Some(profile_result.user_profile),
        session_id: Some(session_result.session_id),
        expires_in: Some(token_result.expires_in),
        completed_at,
    })
}

/// SSO provider configuration workflow
pub async fn configure_sso_provider_workflow(
    _context: WorkflowContext,
    request: ConfigureSsoProviderRequest,
) -> Result<ConfigureSsoProviderResult, WorkflowError> {
    // This workflow would handle SSO provider configuration
    // Including setting up client credentials, endpoints, etc.
    
    let configured_at = Utc::now();

    // TODO: Implement SSO provider configuration logic
    tracing::info!(
        tenant_id = %request.tenant_id,
        provider = ?request.provider,
        "Configuring SSO provider"
    );

    Ok(ConfigureSsoProviderResult {
        tenant_id: request.tenant_id,
        provider: request.provider,
        configured: true,
        configuration_id: Uuid::new_v4().to_string(),
        configured_at,
    })
}

/// Configure SSO provider workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureSsoProviderRequest {
    pub tenant_id: TenantId,
    pub provider: SsoProvider,
    pub client_id: String,
    pub client_secret: String,
    pub additional_config: HashMap<String, serde_json::Value>,
}

/// Configure SSO provider workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureSsoProviderResult {
    pub tenant_id: TenantId,
    pub provider: SsoProvider,
    pub configured: bool,
    pub configuration_id: String,
    pub configured_at: DateTime<Utc>,
}

// Helper functions
fn generate_mock_token() -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    let mut rng = rand::thread_rng();
    
    (0..64)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

fn generate_mock_jwt() -> String {
    // Mock JWT token - in real implementation, this would be properly signed
    format!(
        "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.{}.{}",
        base64::prelude::BASE64_STANDARD.encode(r#"{"sub":"1234567890","name":"John Doe","iat":1516239022}"#),
        generate_mock_token()
    )
}

fn get_sso_client_id(provider: &SsoProvider) -> String {
    // TODO: Get client ID from configuration/environment
    match provider {
        SsoProvider::Google => "google_client_id".to_string(),
        SsoProvider::Microsoft => "microsoft_client_id".to_string(),
        SsoProvider::Okta => "okta_client_id".to_string(),
        SsoProvider::Auth0 => "auth0_client_id".to_string(),
        SsoProvider::Saml { .. } => "saml_client_id".to_string(),
        SsoProvider::Oidc { .. } => "oidc_client_id".to_string(),
    }
}

fn get_sso_client_secret(provider: &SsoProvider) -> String {
    // TODO: Get client secret from secure configuration/vault
    match provider {
        SsoProvider::Google => "google_client_secret".to_string(),
        SsoProvider::Microsoft => "microsoft_client_secret".to_string(),
        SsoProvider::Okta => "okta_client_secret".to_string(),
        SsoProvider::Auth0 => "auth0_client_secret".to_string(),
        SsoProvider::Saml { .. } => "saml_client_secret".to_string(),
        SsoProvider::Oidc { .. } => "oidc_client_secret".to_string(),
    }
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
            permissions: vec!["sso:authenticate".to_string()],
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
            retry_policy: Some(activity_utils::external_service_retry_policy()),
            tags: vec!["sso_authentication".to_string()],
            custom: std::collections::HashMap::new(),
        },
        heartbeat_details: None,
    }
}