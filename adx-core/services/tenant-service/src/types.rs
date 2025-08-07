use adx_shared::TenantId;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ==================== Core Tenant Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantDetails {
    pub id: TenantId,
    pub name: String,
    pub slug: String, // URL-friendly identifier
    pub status: TenantStatus,
    pub plan: TenantPlan,
    pub owner_email: String,
    pub billing_email: String,
    pub settings: TenantSettings,
    pub resource_limits: ResourceLimits,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_activity: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantStatus {
    Active,
    Suspended,
    Deleted,
    Provisioning,
    Upgrading,
    Downgrading,
    Maintenance,
}

impl std::fmt::Display for TenantStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantStatus::Active => write!(f, "active"),
            TenantStatus::Suspended => write!(f, "suspended"),
            TenantStatus::Deleted => write!(f, "deleted"),
            TenantStatus::Provisioning => write!(f, "provisioning"),
            TenantStatus::Upgrading => write!(f, "upgrading"),
            TenantStatus::Downgrading => write!(f, "downgrading"),
            TenantStatus::Maintenance => write!(f, "maintenance"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TenantPlan {
    Free,
    Starter,
    Professional,
    Enterprise,
    Custom,
}

impl std::fmt::Display for TenantPlan {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TenantPlan::Free => write!(f, "free"),
            TenantPlan::Starter => write!(f, "starter"),
            TenantPlan::Professional => write!(f, "professional"),
            TenantPlan::Enterprise => write!(f, "enterprise"),
            TenantPlan::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    pub timezone: String,
    pub locale: String,
    pub features_enabled: Vec<String>,
    pub integrations: Vec<Integration>,
    pub security_settings: SecuritySettings,
    pub notification_preferences: NotificationPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_users: u32,
    pub max_storage_gb: u32,
    pub max_api_calls_per_hour: u32,
    pub max_workflows_per_month: u32,
    pub max_files: u32,
    pub max_file_size_mb: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Integration {
    pub name: String,
    pub enabled: bool,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySettings {
    pub enforce_2fa: bool,
    pub session_timeout_minutes: u32,
    pub password_policy: PasswordPolicy,
    pub ip_whitelist: Vec<String>,
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordPolicy {
    pub min_length: u8,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
    pub max_age_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub sms_notifications: bool,
    pub webhook_url: Option<String>,
    pub notification_types: Vec<String>,
}

// ==================== Request/Response Types ====================

#[derive(Debug, Deserialize)]
pub struct TenantCreationRequest {
    pub name: String,
    pub slug: String,
    pub owner_email: String,
    pub billing_email: String,
    pub plan: TenantPlan,
    pub initial_settings: Option<TenantSettings>,
}

#[derive(Debug, Serialize)]
pub struct TenantCreationResponse {
    pub tenant: TenantDetails,
    pub workflow_id: String,
    pub provisioning_status: String,
    pub estimated_completion_minutes: u32,
}

#[derive(Debug, Deserialize)]
pub struct TenantUpdateRequest {
    pub name: Option<String>,
    pub billing_email: Option<String>,
    pub settings: Option<TenantSettings>,
}

#[derive(Debug, Deserialize)]
pub struct TenantListQuery {
    pub status: Option<TenantStatus>,
    pub plan: Option<TenantPlan>,
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TenantListResponse {
    pub tenants: Vec<TenantSummary>,
    pub total_count: u32,
    pub has_more: bool,
}

#[derive(Debug, Serialize)]
pub struct TenantSummary {
    pub id: TenantId,
    pub name: String,
    pub slug: String,
    pub status: TenantStatus,
    pub plan: TenantPlan,
    pub created_at: DateTime<Utc>,
    pub last_activity: Option<DateTime<Utc>>,
    pub user_count: u32,
}

// ==================== Workflow Input Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantProvisioningInput {
    pub tenant_id: TenantId,
    pub plan: TenantPlan,
    pub resource_requirements: ResourceRequirements,
    pub features_to_enable: Vec<String>,
    pub notification_settings: NotificationPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub database_schema: bool,
    pub storage_bucket: bool,
    pub cdn_setup: bool,
    pub monitoring_setup: bool,
    pub backup_setup: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUpgradeInput {
    pub tenant_id: TenantId,
    pub from_plan: TenantPlan,
    pub to_plan: TenantPlan,
    pub effective_date: DateTime<Utc>,
    pub prorated_billing: bool,
    pub feature_migration: FeatureMigration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureMigration {
    pub features_to_enable: Vec<String>,
    pub features_to_disable: Vec<String>,
    pub data_migration_required: bool,
    pub backup_before_upgrade: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMonitoringInput {
    pub tenant_id: TenantId,
    pub monitoring_type: MonitoringType,
    pub metrics_to_collect: Vec<String>,
    pub alert_thresholds: AlertThresholds,
    pub reporting_frequency: ReportingFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MonitoringType {
    HealthCheck,
    PerformanceMonitoring,
    SecurityAudit,
    UsageAnalysis,
    ComplianceCheck,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub cpu_usage_percent: Option<f32>,
    pub memory_usage_percent: Option<f32>,
    pub disk_usage_percent: Option<f32>,
    pub api_error_rate_percent: Option<f32>,
    pub response_time_ms: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportingFrequency {
    RealTime,
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

// ==================== Workflow Output Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantProvisioningOutput {
    pub tenant_id: TenantId,
    pub provisioning_status: ProvisioningStatus,
    pub resources_created: Vec<ProvisionedResource>,
    pub configuration_applied: bool,
    pub estimated_completion: DateTime<Utc>,
    pub rollback_plan: Option<RollbackPlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProvisioningStatus {
    InProgress,
    Completed,
    Failed,
    RolledBack,
    RequiresManualIntervention,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionedResource {
    pub resource_type: String,
    pub resource_id: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackPlan {
    pub steps: Vec<RollbackStep>,
    pub estimated_duration_minutes: u32,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackStep {
    pub action: String,
    pub resource_id: String,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantUpgradeOutput {
    pub tenant_id: TenantId,
    pub upgrade_status: UpgradeStatus,
    pub features_migrated: Vec<String>,
    pub billing_updated: bool,
    pub rollback_available: bool,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpgradeStatus {
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
    RolledBack,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMonitoringOutput {
    pub tenant_id: TenantId,
    pub monitoring_session_id: Uuid,
    pub metrics_collected: Vec<MetricData>,
    pub alerts_triggered: Vec<Alert>,
    pub health_score: f32,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricData {
    pub metric_name: String,
    pub value: f64,
    pub unit: String,
    pub timestamp: DateTime<Utc>,
    pub tags: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub alert_type: String,
    pub severity: AlertSeverity,
    pub message: String,
    pub threshold_exceeded: f64,
    pub current_value: f64,
    pub triggered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    pub category: String,
    pub title: String,
    pub description: String,
    pub priority: RecommendationPriority,
    pub estimated_impact: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationPriority {
    Low,
    Medium,
    High,
    Urgent,
}

// ==================== Billing and Usage Types ====================

#[derive(Debug, Serialize)]
pub struct TenantBillingInfo {
    pub tenant_id: TenantId,
    pub plan: TenantPlan,
    pub billing_cycle: BillingCycle,
    pub current_period: BillingPeriod,
    pub charges: Vec<BillingCharge>,
    pub payment_method: Option<PaymentMethod>,
    pub billing_status: BillingStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BillingCycle {
    Monthly,
    Quarterly,
    Annual,
}

#[derive(Debug, Serialize)]
pub struct BillingPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub is_current: bool,
}

#[derive(Debug, Serialize)]
pub struct BillingCharge {
    pub description: String,
    pub amount: f64,
    pub currency: String,
    pub charge_type: ChargeType,
    pub date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ChargeType {
    Subscription,
    Usage,
    Overage,
    OneTime,
    Credit,
}

#[derive(Debug, Serialize)]
pub struct PaymentMethod {
    pub method_type: String,
    pub last_four: String,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BillingStatus {
    Current,
    Overdue,
    Suspended,
    Cancelled,
}

#[derive(Debug, Serialize)]
pub struct TenantUsageStats {
    pub tenant_id: TenantId,
    pub period: BillingPeriod,
    pub usage_metrics: Vec<UsageMetric>,
    pub resource_consumption: ResourceConsumption,
    pub api_usage: ApiUsage,
    pub storage_usage: StorageUsage,
}

#[derive(Debug, Serialize)]
pub struct UsageMetric {
    pub metric_name: String,
    pub current_value: f64,
    pub limit: Option<f64>,
    pub unit: String,
    pub percentage_of_limit: Option<f32>,
}

#[derive(Debug, Serialize)]
pub struct ResourceConsumption {
    pub cpu_hours: f64,
    pub memory_gb_hours: f64,
    pub network_gb: f64,
    pub database_queries: u64,
}

#[derive(Debug, Serialize)]
pub struct ApiUsage {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub average_response_time_ms: f32,
    pub requests_by_endpoint: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct StorageUsage {
    pub total_files: u32,
    pub total_size_gb: f64,
    pub files_by_type: serde_json::Value,
    pub largest_files: Vec<FileInfo>,
}

#[derive(Debug, Serialize)]
pub struct FileInfo {
    pub name: String,
    pub size_mb: f64,
    pub created_at: DateTime<Utc>,
    pub last_accessed: Option<DateTime<Utc>>,
}

// ==================== Backup Related Types ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupType {
    Full,
    Incremental,
    Differential,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionSettings {
    pub enabled: bool,
    pub algorithm: String,
    pub key_rotation_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub retention_days: u32,
    pub max_backups: u32,
    pub compress: bool,
}
