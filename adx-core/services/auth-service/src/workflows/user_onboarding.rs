use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use adx_shared::temporal::{
    WorkflowContext, ActivityContext, AdxActivity, TenantAwareActivity,
    ActivityError, WorkflowError, utils as activity_utils,
};
use adx_shared::types::{UserId, TenantId, SubscriptionTier};

/// User onboarding workflow input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingRequest {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub onboarding_type: OnboardingType,
    pub user_preferences: UserOnboardingPreferences,
    pub tenant_setup: Option<TenantSetupPreferences>,
}

/// Onboarding type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnboardingType {
    NewUser,           // First-time user registration
    InvitedUser,       // User invited to existing tenant
    TenantAdmin,       // User creating new tenant
    Migration,         // User migrating from another system
}

/// User onboarding preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingPreferences {
    pub display_name: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub notification_preferences: NotificationPreferences,
    pub role_preferences: Vec<String>,
    pub department: Option<String>,
    pub job_title: Option<String>,
}

/// Notification preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub sms_notifications: bool,
    pub digest_frequency: DigestFrequency,
}

/// Digest frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DigestFrequency {
    Immediate,
    Hourly,
    Daily,
    Weekly,
    Never,
}

/// Tenant setup preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSetupPreferences {
    pub tenant_name: String,
    pub industry: Option<String>,
    pub company_size: Option<CompanySize>,
    pub use_cases: Vec<String>,
    pub default_modules: Vec<String>,
    pub branding: Option<BrandingPreferences>,
}

/// Company size
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompanySize {
    Solo,
    Small,      // 2-10
    Medium,     // 11-50
    Large,      // 51-200
    Enterprise, // 200+
}

/// Branding preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingPreferences {
    pub primary_color: Option<String>,
    pub logo_url: Option<String>,
    pub custom_domain: Option<String>,
}

/// User onboarding workflow result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingResult {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub onboarding_completed: bool,
    pub user_profile_created: bool,
    pub tenant_configured: bool,
    pub modules_installed: Vec<String>,
    pub welcome_email_sent: bool,
    pub next_steps: Vec<OnboardingStep>,
    pub completed_at: DateTime<Utc>,
}

/// Onboarding step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    pub step_id: String,
    pub title: String,
    pub description: String,
    pub action_url: Option<String>,
    pub completed: bool,
    pub required: bool,
}

/// Setup user profile activity
pub struct SetupUserProfileActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupUserProfileInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub preferences: UserOnboardingPreferences,
    pub onboarding_type: OnboardingType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupUserProfileOutput {
    pub profile_created: bool,
    pub user_roles: Vec<String>,
    pub permissions: Vec<String>,
    pub created_at: DateTime<Utc>,
}

impl AdxActivity<SetupUserProfileInput, SetupUserProfileOutput> for SetupUserProfileActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: SetupUserProfileInput,
    ) -> Result<SetupUserProfileOutput, ActivityError> {
        let created_at = Utc::now();

        // Determine user roles based on onboarding type
        let user_roles = match input.onboarding_type {
            OnboardingType::TenantAdmin => vec!["admin".to_string(), "user".to_string()],
            OnboardingType::InvitedUser => input.preferences.role_preferences.clone(),
            OnboardingType::NewUser => vec!["user".to_string()],
            OnboardingType::Migration => vec!["user".to_string()],
        };

        // Determine permissions based on roles
        let permissions = derive_permissions_from_roles(&user_roles);

        // TODO: Create user profile in database
        tracing::info!(
            user_id = %input.user_id,
            tenant_id = %input.tenant_id,
            roles = ?user_roles,
            "Setting up user profile"
        );

        Ok(SetupUserProfileOutput {
            profile_created: true,
            user_roles,
            permissions,
            created_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "setup_user_profile"
    }
}

impl TenantAwareActivity<SetupUserProfileInput, SetupUserProfileOutput> for SetupUserProfileActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        _user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // User profile setup is allowed during onboarding
        Ok(())
    }
}

/// Configure tenant settings activity
pub struct ConfigureTenantSettingsActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureTenantSettingsInput {
    pub tenant_id: TenantId,
    pub admin_user_id: UserId,
    pub setup_preferences: TenantSetupPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureTenantSettingsOutput {
    pub tenant_configured: bool,
    pub settings_applied: Vec<String>,
    pub branding_configured: bool,
    pub configured_at: DateTime<Utc>,
}

impl AdxActivity<ConfigureTenantSettingsInput, ConfigureTenantSettingsOutput> for ConfigureTenantSettingsActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: ConfigureTenantSettingsInput,
    ) -> Result<ConfigureTenantSettingsOutput, ActivityError> {
        let configured_at = Utc::now();
        let mut settings_applied = Vec::new();

        // Apply tenant name
        settings_applied.push("tenant_name".to_string());

        // Apply industry settings if provided
        if input.setup_preferences.industry.is_some() {
            settings_applied.push("industry".to_string());
        }

        // Apply company size settings if provided
        if input.setup_preferences.company_size.is_some() {
            settings_applied.push("company_size".to_string());
        }

        // Configure branding if provided
        let branding_configured = input.setup_preferences.branding.is_some();
        if branding_configured {
            settings_applied.push("branding".to_string());
        }

        // TODO: Update tenant settings in database
        tracing::info!(
            tenant_id = %input.tenant_id,
            admin_user_id = %input.admin_user_id,
            settings_applied = ?settings_applied,
            "Configuring tenant settings"
        );

        Ok(ConfigureTenantSettingsOutput {
            tenant_configured: true,
            settings_applied,
            branding_configured,
            configured_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "configure_tenant_settings"
    }
}

impl TenantAwareActivity<ConfigureTenantSettingsInput, ConfigureTenantSettingsOutput> for ConfigureTenantSettingsActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Only tenant admins can configure tenant settings
        if !user_context.roles.contains(&"admin".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "Only tenant administrators can configure tenant settings".to_string(),
            });
        }

        Ok(())
    }
}

/// Install default modules activity
pub struct InstallDefaultModulesActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallDefaultModulesInput {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub modules: Vec<String>,
    pub subscription_tier: SubscriptionTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallDefaultModulesOutput {
    pub modules_installed: Vec<String>,
    pub modules_failed: Vec<ModuleInstallationError>,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInstallationError {
    pub module_id: String,
    pub error_code: String,
    pub error_message: String,
}

impl AdxActivity<InstallDefaultModulesInput, InstallDefaultModulesOutput> for InstallDefaultModulesActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: InstallDefaultModulesInput,
    ) -> Result<InstallDefaultModulesOutput, ActivityError> {
        let installed_at = Utc::now();
        let mut modules_installed = Vec::new();
        let mut modules_failed = Vec::new();

        // Filter modules based on subscription tier
        let available_modules = filter_modules_by_subscription(&input.modules, &input.subscription_tier);

        for module_id in available_modules {
            // TODO: Install module using module service
            // For now, simulate installation
            match simulate_module_installation(&module_id).await {
                Ok(_) => {
                    modules_installed.push(module_id.clone());
                    tracing::info!(
                        tenant_id = %input.tenant_id,
                        module_id = %module_id,
                        "Module installed successfully"
                    );
                }
                Err(error) => {
                    modules_failed.push(ModuleInstallationError {
                        module_id: module_id.clone(),
                        error_code: "INSTALLATION_FAILED".to_string(),
                        error_message: error.clone(),
                    });
                    tracing::warn!(
                        tenant_id = %input.tenant_id,
                        module_id = %module_id,
                        error = %error,
                        "Module installation failed"
                    );
                }
            }
        }

        Ok(InstallDefaultModulesOutput {
            modules_installed,
            modules_failed,
            installed_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "install_default_modules"
    }
}

impl TenantAwareActivity<InstallDefaultModulesInput, InstallDefaultModulesOutput> for InstallDefaultModulesActivity {
    async fn validate_tenant_access(
        &self,
        _tenant_context: &adx_shared::temporal::TenantContext,
        user_context: &adx_shared::temporal::UserContext,
    ) -> Result<(), ActivityError> {
        // Only users with module installation permissions can install modules
        if !user_context.permissions.contains(&"module:install".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "Insufficient permissions to install modules".to_string(),
            });
        }

        Ok(())
    }
}

/// Send welcome email activity
pub struct SendWelcomeEmailActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendWelcomeEmailInput {
    pub user_id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub tenant_name: String,
    pub onboarding_type: OnboardingType,
    pub next_steps: Vec<OnboardingStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendWelcomeEmailOutput {
    pub email_sent: bool,
    pub message_id: String,
    pub sent_at: DateTime<Utc>,
}

impl AdxActivity<SendWelcomeEmailInput, SendWelcomeEmailOutput> for SendWelcomeEmailActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: SendWelcomeEmailInput,
    ) -> Result<SendWelcomeEmailOutput, ActivityError> {
        let message_id = Uuid::new_v4().to_string();
        let sent_at = Utc::now();

        // TODO: Send welcome email using email service
        // Email content would vary based on onboarding type
        tracing::info!(
            user_id = %input.user_id,
            email = %input.email,
            tenant_name = %input.tenant_name,
            onboarding_type = ?input.onboarding_type,
            message_id = %message_id,
            "Sending welcome email"
        );

        // Simulate email sending
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        Ok(SendWelcomeEmailOutput {
            email_sent: true,
            message_id,
            sent_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "send_welcome_email"
    }
}

/// Create onboarding checklist activity
pub struct CreateOnboardingChecklistActivity;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboardingChecklistInput {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub onboarding_type: OnboardingType,
    pub subscription_tier: SubscriptionTier,
    pub installed_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateOnboardingChecklistOutput {
    pub checklist_created: bool,
    pub next_steps: Vec<OnboardingStep>,
    pub created_at: DateTime<Utc>,
}

impl AdxActivity<CreateOnboardingChecklistInput, CreateOnboardingChecklistOutput> for CreateOnboardingChecklistActivity {
    async fn execute(
        &self,
        _context: ActivityContext,
        input: CreateOnboardingChecklistInput,
    ) -> Result<CreateOnboardingChecklistOutput, ActivityError> {
        let created_at = Utc::now();

        // Generate next steps based on onboarding type and installed modules
        let next_steps = generate_onboarding_steps(
            &input.onboarding_type,
            &input.subscription_tier,
            &input.installed_modules,
        );

        // TODO: Store onboarding checklist in database
        tracing::info!(
            user_id = %input.user_id,
            tenant_id = %input.tenant_id,
            steps_count = next_steps.len(),
            "Creating onboarding checklist"
        );

        Ok(CreateOnboardingChecklistOutput {
            checklist_created: true,
            next_steps,
            created_at,
        })
    }

    fn activity_type(&self) -> &'static str {
        "create_onboarding_checklist"
    }
}

/// User onboarding workflow implementation
pub async fn user_onboarding_workflow(
    _context: WorkflowContext,
    request: UserOnboardingRequest,
) -> Result<UserOnboardingResult, WorkflowError> {
    let completed_at = Utc::now();

    // Step 1: Setup user profile
    let setup_profile_activity = SetupUserProfileActivity;
    let setup_profile_input = SetupUserProfileInput {
        user_id: request.user_id.clone(),
        tenant_id: request.tenant_id.clone(),
        preferences: request.user_preferences.clone(),
        onboarding_type: request.onboarding_type.clone(),
    };

    let profile_result = setup_profile_activity.execute(
        create_activity_context("setup_user_profile", "user-onboarding-workflow"),
        setup_profile_input,
    ).await?;

    // Step 2: Configure tenant settings (if tenant admin)
    let tenant_configured = if matches!(request.onboarding_type, OnboardingType::TenantAdmin) {
        if let Some(tenant_setup) = request.tenant_setup {
            let configure_tenant_activity = ConfigureTenantSettingsActivity;
            let configure_tenant_input = ConfigureTenantSettingsInput {
                tenant_id: request.tenant_id.clone(),
                admin_user_id: request.user_id.clone(),
                setup_preferences: tenant_setup,
            };

            let tenant_result = configure_tenant_activity.execute(
                create_activity_context("configure_tenant_settings", "user-onboarding-workflow"),
                configure_tenant_input,
            ).await?;

            tenant_result.tenant_configured
        } else {
            false
        }
    } else {
        false
    };

    // Step 3: Install default modules
    let default_modules = get_default_modules_for_onboarding(&request.onboarding_type);
    let install_modules_activity = InstallDefaultModulesActivity;
    let install_modules_input = InstallDefaultModulesInput {
        tenant_id: request.tenant_id.clone(),
        user_id: request.user_id.clone(),
        modules: default_modules,
        subscription_tier: SubscriptionTier::Professional, // TODO: Get from tenant context
    };

    let modules_result = install_modules_activity.execute(
        create_activity_context("install_default_modules", "user-onboarding-workflow"),
        install_modules_input,
    ).await?;

    // Step 4: Create onboarding checklist
    let create_checklist_activity = CreateOnboardingChecklistActivity;
    let create_checklist_input = CreateOnboardingChecklistInput {
        user_id: request.user_id.clone(),
        tenant_id: request.tenant_id.clone(),
        onboarding_type: request.onboarding_type.clone(),
        subscription_tier: SubscriptionTier::Professional, // TODO: Get from tenant context
        installed_modules: modules_result.modules_installed.clone(),
    };

    let checklist_result = create_checklist_activity.execute(
        create_activity_context("create_onboarding_checklist", "user-onboarding-workflow"),
        create_checklist_input,
    ).await?;

    // Step 5: Send welcome email
    let send_welcome_activity = SendWelcomeEmailActivity;
    let send_welcome_input = SendWelcomeEmailInput {
        user_id: request.user_id.clone(),
        email: "user@example.com".to_string(), // TODO: Get from user context
        display_name: request.user_preferences.display_name.clone(),
        tenant_name: "Default Tenant".to_string(), // TODO: Get from tenant context
        onboarding_type: request.onboarding_type.clone(),
        next_steps: checklist_result.next_steps.clone(),
    };

    let welcome_result = send_welcome_activity.execute(
        create_activity_context("send_welcome_email", "user-onboarding-workflow"),
        send_welcome_input,
    ).await?;

    Ok(UserOnboardingResult {
        user_id: request.user_id,
        tenant_id: request.tenant_id,
        onboarding_completed: true,
        user_profile_created: profile_result.profile_created,
        tenant_configured,
        modules_installed: modules_result.modules_installed,
        welcome_email_sent: welcome_result.email_sent,
        next_steps: checklist_result.next_steps,
        completed_at,
    })
}

// Helper functions
fn derive_permissions_from_roles(roles: &[String]) -> Vec<String> {
    let mut permissions = Vec::new();

    for role in roles {
        match role.as_str() {
            "admin" => {
                permissions.extend_from_slice(&[
                    "tenant:admin".to_string(),
                    "user:admin".to_string(),
                    "file:admin".to_string(),
                    "module:install".to_string(),
                    "module:admin".to_string(),
                ]);
            }
            "user" => {
                permissions.extend_from_slice(&[
                    "tenant:read".to_string(),
                    "user:read".to_string(),
                    "user:write".to_string(),
                    "file:read".to_string(),
                    "file:write".to_string(),
                ]);
            }
            "viewer" => {
                permissions.extend_from_slice(&[
                    "tenant:read".to_string(),
                    "user:read".to_string(),
                    "file:read".to_string(),
                ]);
            }
            _ => {}
        }
    }

    permissions.sort();
    permissions.dedup();
    permissions
}

fn filter_modules_by_subscription(modules: &[String], subscription_tier: &SubscriptionTier) -> Vec<String> {
    // Filter modules based on subscription tier
    modules.iter()
        .filter(|module_id| {
            match subscription_tier {
                SubscriptionTier::Free => {
                    // Only basic modules for free tier
                    matches!(module_id.as_str(), "basic_dashboard" | "basic_files")
                }
                SubscriptionTier::Professional => {
                    // Most modules for professional tier
                    !matches!(module_id.as_str(), "enterprise_analytics" | "advanced_security")
                }
                SubscriptionTier::Enterprise | SubscriptionTier::Custom => {
                    // All modules for enterprise tier
                    true
                }
            }
        })
        .cloned()
        .collect()
}

async fn simulate_module_installation(module_id: &str) -> Result<(), String> {
    // Simulate module installation with potential failures
    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    
    match module_id {
        "failing_module" => Err("Module installation failed".to_string()),
        _ => Ok(()),
    }
}

fn get_default_modules_for_onboarding(onboarding_type: &OnboardingType) -> Vec<String> {
    match onboarding_type {
        OnboardingType::TenantAdmin => vec![
            "basic_dashboard".to_string(),
            "user_management".to_string(),
            "basic_files".to_string(),
            "tenant_settings".to_string(),
        ],
        OnboardingType::NewUser => vec![
            "basic_dashboard".to_string(),
            "basic_files".to_string(),
        ],
        OnboardingType::InvitedUser => vec![
            "basic_dashboard".to_string(),
            "basic_files".to_string(),
        ],
        OnboardingType::Migration => vec![
            "basic_dashboard".to_string(),
            "basic_files".to_string(),
            "data_import".to_string(),
        ],
    }
}

fn generate_onboarding_steps(
    onboarding_type: &OnboardingType,
    subscription_tier: &SubscriptionTier,
    installed_modules: &[String],
) -> Vec<OnboardingStep> {
    let mut steps = Vec::new();

    // Common steps for all users
    steps.push(OnboardingStep {
        step_id: "complete_profile".to_string(),
        title: "Complete Your Profile".to_string(),
        description: "Add your profile picture and personal information".to_string(),
        action_url: Some("/profile/edit".to_string()),
        completed: false,
        required: false,
    });

    steps.push(OnboardingStep {
        step_id: "explore_dashboard".to_string(),
        title: "Explore Your Dashboard".to_string(),
        description: "Take a tour of your personalized dashboard".to_string(),
        action_url: Some("/dashboard/tour".to_string()),
        completed: false,
        required: false,
    });

    // Steps specific to tenant admins
    if matches!(onboarding_type, OnboardingType::TenantAdmin) {
        steps.push(OnboardingStep {
            step_id: "invite_team_members".to_string(),
            title: "Invite Team Members".to_string(),
            description: "Invite your colleagues to join your organization".to_string(),
            action_url: Some("/users/invite".to_string()),
            completed: false,
            required: false,
        });

        steps.push(OnboardingStep {
            step_id: "configure_settings".to_string(),
            title: "Configure Organization Settings".to_string(),
            description: "Set up your organization preferences and branding".to_string(),
            action_url: Some("/tenant/settings".to_string()),
            completed: false,
            required: false,
        });
    }

    // Steps based on installed modules
    if installed_modules.contains(&"basic_files".to_string()) {
        steps.push(OnboardingStep {
            step_id: "upload_first_file".to_string(),
            title: "Upload Your First File".to_string(),
            description: "Try out the file management features".to_string(),
            action_url: Some("/files/upload".to_string()),
            completed: false,
            required: false,
        });
    }

    // Steps based on subscription tier
    if matches!(subscription_tier, SubscriptionTier::Professional | SubscriptionTier::Enterprise | SubscriptionTier::Custom) {
        steps.push(OnboardingStep {
            step_id: "explore_advanced_features".to_string(),
            title: "Explore Advanced Features".to_string(),
            description: "Discover the advanced capabilities available in your plan".to_string(),
            action_url: Some("/features/advanced".to_string()),
            completed: false,
            required: false,
        });
    }

    steps
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
            permissions: vec!["user:create".to_string(), "module:install".to_string()],
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
            tags: vec!["user_onboarding".to_string()],
            custom: std::collections::HashMap::new(),
        },
        heartbeat_details: None,
    }
}