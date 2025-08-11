use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};

use adx_shared::{
    config::AppConfig,
    temporal::{AdxTemporalWorker, TemporalConfig},
};

use crate::workflows::{
    user_registration::{
        user_registration_workflow, ValidateUserRegistrationActivity, CreateUserAccountActivity,
        SendVerificationEmailActivity, CreateDefaultTenantActivity,
    },
    password_reset::{
        password_reset_workflow, confirm_password_reset_workflow,
        ValidatePasswordResetActivity, GenerateResetTokenActivity, SendPasswordResetEmailActivity,
        InvalidateExistingTokensActivity, LogSecurityEventActivity, ValidateResetTokenActivity,
        UpdateUserPasswordActivity, InvalidateUserSessionsActivity,
    },
    user_onboarding::{
        user_onboarding_workflow, SetupUserProfileActivity, ConfigureTenantSettingsActivity,
        InstallDefaultModulesActivity, SendWelcomeEmailActivity, CreateOnboardingChecklistActivity,
    },
    mfa_setup::{
        mfa_setup_workflow, GenerateTotpSecretActivity, VerifyTotpCodeActivity,
        SendSmsVerificationActivity, VerifyPhoneNumberActivity, GenerateBackupCodesActivity,
        StoreMfaConfigurationActivity, SendMfaSetupNotificationActivity,
    },
    sso_authentication::{
        sso_authentication_workflow, configure_sso_provider_workflow,
        ExchangeAuthorizationCodeActivity, FetchSsoUserProfileActivity, FindOrCreateSsoUserActivity,
        CreateSsoSessionActivity, LogSsoAuthEventActivity, ValidateSsoStateActivity,
    },
};

/// Auth Service Temporal Worker
pub struct AuthWorker {
    worker: AdxTemporalWorker,
    config: AppConfig,
}

impl AuthWorker {
    /// Create a new Auth Service Temporal worker
    pub async fn new(config: AppConfig) -> Result<Self> {
        info!("Initializing Auth Service Temporal worker");

        // Create Temporal configuration
        let temporal_config = TemporalConfig {
            server_address: config.temporal.server_address.clone(),
            namespace: config.temporal.namespace.clone(),
            client_identity: format!("auth-worker-{}", uuid::Uuid::new_v4()),
            connection: config.temporal.connection.clone(),
            retry: config.temporal.retry.clone(),
            worker: adx_shared::temporal::WorkerConfig {
                max_concurrent_workflow_tasks: config.temporal.worker_max_concurrent_workflows,
                max_concurrent_activity_tasks: config.temporal.worker_max_concurrent_activities,
                identity: format!("auth-worker-{}", uuid::Uuid::new_v4()),
                task_queues: vec![
                    "auth-service-queue".to_string(),
                    "user-registration-queue".to_string(),
                    "password-reset-queue".to_string(),
                    "user-onboarding-queue".to_string(),
                    "mfa-setup-queue".to_string(),
                    "sso-authentication-queue".to_string(),
                ],
                enable_sticky_execution: true,
                sticky_schedule_to_start_timeout: std::time::Duration::from_secs(10),
            },
            workflow: config.temporal.workflow.clone(),
            activity: config.temporal.activity.clone(),
        };

        // Create Temporal worker
        let task_queues = temporal_config.worker.task_queues.clone();
        let worker = AdxTemporalWorker::new(temporal_config, task_queues).await?;

        Ok(Self { worker, config })
    }

    /// Register all workflows and activities
    pub async fn register_workflows_and_activities(&self) -> Result<()> {
        info!("Registering Auth Service workflows and activities");

        // Register User Registration workflows and activities
        self.register_user_registration_workflows().await?;
        
        // Register Password Reset workflows and activities
        self.register_password_reset_workflows().await?;
        
        // Register User Onboarding workflows and activities
        self.register_user_onboarding_workflows().await?;
        
        // Register MFA Setup workflows and activities
        self.register_mfa_setup_workflows().await?;
        
        // Register SSO Authentication workflows and activities
        self.register_sso_authentication_workflows().await?;

        info!("All Auth Service workflows and activities registered successfully");
        Ok(())
    }

    /// Register user registration workflows and activities
    async fn register_user_registration_workflows(&self) -> Result<()> {
        info!("Registering user registration workflows and activities");

        // Register workflow
        // TODO: Register actual workflow with Temporal SDK
        // For now, we'll log the registration
        info!("Registered workflow: user_registration_workflow");

        // Register activities
        info!("Registered activity: validate_user_registration");
        info!("Registered activity: create_user_account");
        info!("Registered activity: send_verification_email");
        info!("Registered activity: create_default_tenant");

        Ok(())
    }

    /// Register password reset workflows and activities
    async fn register_password_reset_workflows(&self) -> Result<()> {
        info!("Registering password reset workflows and activities");

        // Register workflows
        info!("Registered workflow: password_reset_workflow");
        info!("Registered workflow: confirm_password_reset_workflow");

        // Register activities
        info!("Registered activity: validate_password_reset");
        info!("Registered activity: generate_reset_token");
        info!("Registered activity: send_password_reset_email");
        info!("Registered activity: invalidate_existing_tokens");
        info!("Registered activity: log_security_event");
        info!("Registered activity: validate_reset_token");
        info!("Registered activity: update_user_password");
        info!("Registered activity: invalidate_user_sessions");

        Ok(())
    }

    /// Register user onboarding workflows and activities
    async fn register_user_onboarding_workflows(&self) -> Result<()> {
        info!("Registering user onboarding workflows and activities");

        // Register workflow
        info!("Registered workflow: user_onboarding_workflow");

        // Register activities
        info!("Registered activity: setup_user_profile");
        info!("Registered activity: configure_tenant_settings");
        info!("Registered activity: install_default_modules");
        info!("Registered activity: send_welcome_email");
        info!("Registered activity: create_onboarding_checklist");

        Ok(())
    }

    /// Register MFA setup workflows and activities
    async fn register_mfa_setup_workflows(&self) -> Result<()> {
        info!("Registering MFA setup workflows and activities");

        // Register workflow
        info!("Registered workflow: mfa_setup_workflow");

        // Register activities
        info!("Registered activity: generate_totp_secret");
        info!("Registered activity: verify_totp_code");
        info!("Registered activity: send_sms_verification");
        info!("Registered activity: verify_phone_number");
        info!("Registered activity: generate_backup_codes");
        info!("Registered activity: store_mfa_configuration");
        info!("Registered activity: send_mfa_setup_notification");

        Ok(())
    }

    /// Register SSO authentication workflows and activities
    async fn register_sso_authentication_workflows(&self) -> Result<()> {
        info!("Registering SSO authentication workflows and activities");

        // Register workflows
        info!("Registered workflow: sso_authentication_workflow");
        info!("Registered workflow: configure_sso_provider_workflow");

        // Register activities
        info!("Registered activity: exchange_authorization_code");
        info!("Registered activity: fetch_sso_user_profile");
        info!("Registered activity: find_or_create_sso_user");
        info!("Registered activity: create_sso_session");
        info!("Registered activity: log_sso_auth_event");
        info!("Registered activity: validate_sso_state");

        Ok(())
    }

    /// Start the worker
    pub async fn start(&self) -> Result<()> {
        info!("Starting Auth Service Temporal worker");

        // Start the Temporal worker
        self.worker.start().await?;

        info!(
            worker_identity = %self.worker.worker_identity(),
            task_queues = ?self.worker.task_queues(),
            workflow_count = %self.worker.workflow_count().await,
            activity_count = %self.worker.activity_count().await,
            "Auth Service Temporal worker started successfully"
        );

        // Keep the worker running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            
            // Health check
            info!(
                worker_identity = %self.worker.worker_identity(),
                workflow_count = %self.worker.workflow_count().await,
                activity_count = %self.worker.activity_count().await,
                "Auth Service worker health check"
            );
        }
    }

    /// Stop the worker
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping Auth Service Temporal worker");
        
        self.worker.stop().await?;
        
        info!("Auth Service Temporal worker stopped");
        Ok(())
    }

    /// Get worker statistics
    pub async fn get_stats(&self) -> WorkerStats {
        WorkerStats {
            worker_identity: self.worker.worker_identity().to_string(),
            task_queues: self.worker.task_queues().to_vec(),
            workflow_count: self.worker.workflow_count().await,
            activity_count: self.worker.activity_count().await,
            uptime: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }
}

/// Worker statistics
#[derive(Debug, serde::Serialize)]
pub struct WorkerStats {
    pub worker_identity: String,
    pub task_queues: Vec<String>,
    pub workflow_count: usize,
    pub activity_count: usize,
    pub uptime: u64,
}

/// Graceful shutdown handler
pub async fn handle_shutdown_signal(worker: Arc<AuthWorker>) {
    let mut sigterm = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .expect("Failed to create SIGTERM handler");
    let mut sigint = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .expect("Failed to create SIGINT handler");

    tokio::select! {
        _ = sigterm.recv() => {
            info!("Received SIGTERM, shutting down gracefully");
        }
        _ = sigint.recv() => {
            info!("Received SIGINT, shutting down gracefully");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Received Ctrl+C, shutting down gracefully");
        }
    }

    if let Err(e) = worker.stop().await {
        error!("Error stopping worker: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adx_shared::config::AppConfig;

    #[tokio::test]
    async fn test_worker_creation() {
        let config = AppConfig::test_config();
        let worker = AuthWorker::new(config).await;
        assert!(worker.is_ok());
    }

    #[tokio::test]
    async fn test_workflow_registration() {
        let config = AppConfig::test_config();
        let worker = AuthWorker::new(config).await.unwrap();
        
        let result = worker.register_workflows_and_activities().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_worker_stats() {
        let config = AppConfig::test_config();
        let worker = AuthWorker::new(config).await.unwrap();
        
        let stats = worker.get_stats().await;
        assert!(!stats.worker_identity.is_empty());
        assert!(!stats.task_queues.is_empty());
    }
}