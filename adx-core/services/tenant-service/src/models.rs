use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use adx_shared::types::{TenantId, UserId, SubscriptionTier, TenantIsolationLevel, TenantQuotas};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub slug: String,
    pub admin_email: String,
    pub subscription_tier: SubscriptionTier,
    pub isolation_level: TenantIsolationLevel,
    pub quotas: TenantQuotas,
    pub features: Vec<String>,
    pub settings: TenantSettings,
    pub status: TenantStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    pub custom_domain: Option<String>,
    pub branding: TenantBranding,
    pub security: TenantSecurity,
    pub notifications: TenantNotifications,
}

impl Default for TenantSettings {
    fn default() -> Self {
        Self {
            custom_domain: None,
            branding: TenantBranding::default(),
            security: TenantSecurity::default(),
            notifications: TenantNotifications::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub theme: String,
}

impl Default for TenantBranding {
    fn default() -> Self {
        Self {
            logo_url: None,
            primary_color: None,
            secondary_color: None,
            theme: "default".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSecurity {
    pub require_mfa: bool,
    pub password_policy: PasswordPolicy,
    pub session_timeout_minutes: u32,
    pub allowed_domains: Vec<String>,
}

impl Default for TenantSecurity {
    fn default() -> Self {
        Self {
            require_mfa: false,
            password_policy: PasswordPolicy::default(),
            session_timeout_minutes: 480, // 8 hours
            allowed_domains: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub max_age_days: Option<u32>,
}

impl Default for PasswordPolicy {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_numbers: true,
            require_symbols: false,
            max_age_days: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantNotifications {
    pub email_enabled: bool,
    pub webhook_url: Option<String>,
    pub slack_webhook: Option<String>,
}

impl Default for TenantNotifications {
    fn default() -> Self {
        Self {
            email_enabled: true,
            webhook_url: None,
            slack_webhook: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantStatus {
    Active,
    Suspended,
    Pending,
    Cancelled,
}

impl Default for TenantStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMembership {
    pub id: String,
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role: TenantRole,
    pub permissions: Vec<String>,
    pub status: MembershipStatus,
    pub invited_by: Option<UserId>,
    pub invited_at: DateTime<Utc>,
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantRole {
    Owner,
    Admin,
    Member,
    Guest,
}

impl Default for TenantRole {
    fn default() -> Self {
        Self::Member
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MembershipStatus {
    Active,
    Invited,
    Suspended,
    Removed,
}

impl Default for MembershipStatus {
    fn default() -> Self {
        Self::Active
    }
}

// Request/Response DTOs
#[derive(Debug, Deserialize)]
pub struct CreateTenantRequest {
    pub name: String,
    pub admin_email: String,
    pub subscription_tier: Option<SubscriptionTier>,
    pub isolation_level: Option<TenantIsolationLevel>,
    pub features: Option<Vec<String>>,
    pub settings: Option<TenantSettings>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTenantRequest {
    pub name: Option<String>,
    pub subscription_tier: Option<SubscriptionTier>,
    pub quotas: Option<TenantQuotas>,
    pub features: Option<Vec<String>>,
    pub settings: Option<TenantSettings>,
    pub status: Option<TenantStatus>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMembershipRequest {
    pub user_id: UserId,
    pub role: TenantRole,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMembershipRequest {
    pub role: Option<TenantRole>,
    pub permissions: Option<Vec<String>>,
    pub status: Option<MembershipStatus>,
}

#[derive(Debug, Deserialize)]
pub struct SwitchTenantRequest {
    pub target_tenant_id: TenantId,
    pub current_tenant_id: Option<TenantId>,
}

#[derive(Debug, Serialize)]
pub struct SwitchTenantResponse {
    pub success: bool,
    pub new_tenant_id: TenantId,
    pub new_session_id: Option<String>,
    pub tenant_context: TenantContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub tenant_slug: String,
    pub subscription_tier: SubscriptionTier,
    pub features: Vec<String>,
    pub quotas: TenantQuotas,
    pub settings: TenantSettings,
    pub user_role: TenantRole,
    pub user_permissions: Vec<String>,
}

// Workflow request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantWorkflowRequest {
    pub tenant_name: String,
    pub admin_email: String,
    pub subscription_tier: SubscriptionTier,
    pub isolation_level: TenantIsolationLevel,
    pub quotas: TenantQuotas,
    pub features: Vec<String>,
    pub default_modules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantWorkflowResult {
    pub tenant_id: TenantId,
    pub admin_user_id: UserId,
    pub database_connection: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchTenantWorkflowRequest {
    pub user_id: UserId,
    pub target_tenant_id: TenantId,
    pub current_tenant_id: Option<TenantId>,
    pub session_duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SwitchTenantWorkflowResult {
    pub success: bool,
    pub new_session_id: String,
    pub tenant_context: TenantContext,
    pub available_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantMonitoringConfig {
    pub metrics: Vec<String>,
    pub check_interval: u64, // seconds
    pub continuous: bool,
    pub alert_thresholds: std::collections::HashMap<String, f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantUpgradeWorkflowRequest {
    pub tenant_id: TenantId,
    pub current_tier: SubscriptionTier,
    pub target_tier: SubscriptionTier,
    pub payment_method: String,
    pub proration: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantUpgradeWorkflowResult {
    pub tenant_id: TenantId,
    pub old_tier: SubscriptionTier,
    pub new_tier: SubscriptionTier,
    pub payment_id: String,
    pub effective_date: DateTime<Utc>,
}