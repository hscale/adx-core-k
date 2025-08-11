use async_trait::async_trait;
use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::models::*;
use crate::services::TenantService;
use adx_shared::types::{TenantId, UserId, SubscriptionTier, TenantQuotas};

// Activity request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTenantCreationRequest {
    pub tenant_name: String,
    pub admin_email: String,
    pub subscription_tier: adx_shared::types::SubscriptionTier,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantValidationResult {
    pub is_valid: bool,
    pub tenant_id: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupTenantDatabaseRequest {
    pub tenant_id: TenantId,
    pub isolation_level: adx_shared::types::TenantIsolationLevel,
    pub initial_schema: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSetupResult {
    pub connection_string: String,
    pub schema_created: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantConfigRequest {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub subscription_tier: adx_shared::types::SubscriptionTier,
    pub quotas: adx_shared::types::TenantQuotas,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateUserTenantAccessRequest {
    pub user_id: UserId,
    pub target_tenant_id: TenantId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTenantAccessResult {
    pub has_access: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSessionStateRequest {
    pub user_id: UserId,
    pub current_tenant_id: TenantId,
    pub session_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStateResult {
    pub session_id: String,
    pub saved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadTenantContextRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantSessionRequest {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub tenant_context: TenantContext,
    pub session_duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantSessionResult {
    pub session_id: String,
    pub available_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserActiveTenantRequest {
    pub user_id: UserId,
    pub new_active_tenant_id: TenantId,
}

// New activity request/response types for Task 13

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantActivityRequest {
    pub tenant_name: String,
    pub admin_email: String,
    pub subscription_tier: SubscriptionTier,
    pub isolation_level: adx_shared::types::TenantIsolationLevel,
    pub quotas: TenantQuotas,
    pub features: Vec<String>,
    pub infrastructure_config: InfrastructureConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfrastructureConfig {
    pub database_config: DatabaseConfig,
    pub storage_config: StorageConfig,
    pub compute_config: ComputeConfig,
    pub network_config: NetworkConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub isolation_level: adx_shared::types::TenantIsolationLevel,
    pub backup_enabled: bool,
    pub backup_retention_days: u32,
    pub encryption_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StorageConfig {
    pub storage_type: String, // "s3", "gcs", "azure", "local"
    pub bucket_name: Option<String>,
    pub encryption_enabled: bool,
    pub versioning_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComputeConfig {
    pub cpu_limit: f64,
    pub memory_limit_gb: u32,
    pub auto_scaling_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub custom_domain: Option<String>,
    pub ssl_enabled: bool,
    pub cdn_enabled: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantActivityResult {
    pub tenant_id: TenantId,
    pub infrastructure_status: InfrastructureStatus,
    pub connection_details: ConnectionDetails,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InfrastructureStatus {
    pub database_ready: bool,
    pub storage_ready: bool,
    pub compute_ready: bool,
    pub network_ready: bool,
    pub provisioning_time_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectionDetails {
    pub database_connection: String,
    pub storage_endpoint: String,
    pub api_endpoint: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupTenantPermissionsRequest {
    pub tenant_id: TenantId,
    pub admin_user_id: UserId,
    pub role_definitions: Vec<RoleDefinition>,
    pub default_permissions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoleDefinition {
    pub name: String,
    pub description: String,
    pub permissions: Vec<String>,
    pub is_default: bool,
    pub hierarchy_level: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupTenantPermissionsResult {
    pub roles_created: Vec<String>,
    pub permissions_assigned: Vec<String>,
    pub admin_role_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorTenantUsageRequest {
    pub tenant_id: TenantId,
    pub monitoring_period: MonitoringPeriod,
    pub metrics_to_track: Vec<UsageMetric>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MonitoringPeriod {
    Hourly,
    Daily,
    Weekly,
    Monthly,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UsageMetric {
    ApiCalls,
    StorageUsage,
    ComputeTime,
    BandwidthUsage,
    ActiveUsers,
    WorkflowExecutions,
    DatabaseQueries,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MonitorTenantUsageResult {
    pub tenant_id: TenantId,
    pub usage_data: HashMap<String, UsageData>,
    pub quota_status: QuotaStatus,
    pub alerts: Vec<UsageAlert>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageData {
    pub metric_name: String,
    pub current_value: f64,
    pub previous_value: f64,
    pub percentage_change: f64,
    pub trend: UsageTrend,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum UsageTrend {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaStatus {
    pub api_calls: QuotaInfo,
    pub storage: QuotaInfo,
    pub compute: QuotaInfo,
    pub users: QuotaInfo,
    pub workflows: QuotaInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaInfo {
    pub used: f64,
    pub limit: f64,
    pub percentage_used: f64,
    pub is_exceeded: bool,
    pub warning_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageAlert {
    pub alert_type: AlertType,
    pub metric: String,
    pub current_value: f64,
    pub threshold: f64,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertType {
    QuotaExceeded,
    QuotaWarning,
    UnusualUsage,
    PerformanceIssue,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessTenantBillingRequest {
    pub tenant_id: TenantId,
    pub billing_period: BillingPeriod,
    pub usage_data: HashMap<String, f64>,
    pub pricing_model: PricingModel,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingPeriod {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub period_type: PeriodType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PeriodType {
    Monthly,
    Quarterly,
    Yearly,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PricingModel {
    pub base_price: Decimal,
    pub usage_rates: HashMap<String, Decimal>,
    pub tier_discounts: HashMap<String, Decimal>,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessTenantBillingResult {
    pub tenant_id: TenantId,
    pub billing_period: BillingPeriod,
    pub line_items: Vec<BillingLineItem>,
    pub subtotal: Decimal,
    pub taxes: Decimal,
    pub total_amount: Decimal,
    pub invoice_id: String,
    pub payment_due_date: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingLineItem {
    pub description: String,
    pub quantity: f64,
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub metric_type: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupTenantDataRequest {
    pub tenant_id: TenantId,
    pub cleanup_type: CleanupType,
    pub data_retention_policy: DataRetentionPolicy,
    pub backup_before_cleanup: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CleanupType {
    SoftDelete,
    HardDelete,
    Archive,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    pub retain_audit_logs: bool,
    pub retain_user_data: bool,
    pub retain_file_metadata: bool,
    pub retention_period_days: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupTenantDataResult {
    pub tenant_id: TenantId,
    pub cleanup_summary: CleanupSummary,
    pub backup_info: Option<BackupInfo>,
    pub cleanup_timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupSummary {
    pub tables_cleaned: Vec<String>,
    pub records_deleted: u64,
    pub storage_freed_gb: f64,
    pub cleanup_duration_ms: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BackupInfo {
    pub backup_id: String,
    pub backup_location: String,
    pub backup_size_gb: f64,
    pub backup_timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateTenantDataRequest {
    pub tenant_id: TenantId,
    pub migration_type: MigrationType,
    pub source_config: MigrationSourceConfig,
    pub target_config: MigrationTargetConfig,
    pub migration_options: MigrationOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationType {
    TierUpgrade,
    TierDowngrade,
    RegionMigration,
    IsolationLevelChange,
    StorageProviderChange,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationSourceConfig {
    pub current_tier: SubscriptionTier,
    pub current_isolation: adx_shared::types::TenantIsolationLevel,
    pub current_region: String,
    pub current_storage_provider: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationTargetConfig {
    pub target_tier: SubscriptionTier,
    pub target_isolation: adx_shared::types::TenantIsolationLevel,
    pub target_region: String,
    pub target_storage_provider: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationOptions {
    pub validate_before_migration: bool,
    pub create_backup: bool,
    pub rollback_on_failure: bool,
    pub migration_batch_size: u32,
    pub max_downtime_minutes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateTenantDataResult {
    pub tenant_id: TenantId,
    pub migration_id: String,
    pub migration_summary: MigrationSummary,
    pub new_configuration: TenantConfiguration,
    pub rollback_info: Option<RollbackInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationSummary {
    pub migration_type: MigrationType,
    pub records_migrated: u64,
    pub data_size_gb: f64,
    pub migration_duration_ms: u64,
    pub downtime_ms: u64,
    pub success: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantConfiguration {
    pub tier: SubscriptionTier,
    pub isolation_level: adx_shared::types::TenantIsolationLevel,
    pub region: String,
    pub storage_provider: String,
    pub quotas: TenantQuotas,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackInfo {
    pub rollback_id: String,
    pub rollback_available_until: DateTime<Utc>,
    pub rollback_data_location: String,
}

// Activity trait definition
#[async_trait]
pub trait TenantActivities: Send + Sync {
    // Tenant creation activities
    async fn validate_tenant_creation(&self, request: ValidateTenantCreationRequest) -> Result<TenantValidationResult>;
    async fn setup_tenant_database(&self, request: SetupTenantDatabaseRequest) -> Result<DatabaseSetupResult>;
    async fn create_tenant_config(&self, request: CreateTenantConfigRequest) -> Result<Tenant>;
    async fn cleanup_tenant_database(&self, tenant_id: &TenantId) -> Result<()>;

    // Tenant switching activities
    async fn validate_user_tenant_access(&self, request: ValidateUserTenantAccessRequest) -> Result<UserTenantAccessResult>;
    async fn save_session_state(&self, request: SaveSessionStateRequest) -> Result<SessionStateResult>;
    async fn load_tenant_context(&self, request: LoadTenantContextRequest) -> Result<TenantContext>;
    async fn create_tenant_session(&self, request: CreateTenantSessionRequest) -> Result<TenantSessionResult>;
    async fn update_user_active_tenant(&self, request: UpdateUserActiveTenantRequest) -> Result<()>;

    // New activities for Task 13: Tenant Activities and RBAC
    async fn create_tenant_activity(&self, request: CreateTenantActivityRequest) -> Result<CreateTenantActivityResult>;
    async fn setup_tenant_permissions_activity(&self, request: SetupTenantPermissionsRequest) -> Result<SetupTenantPermissionsResult>;
    async fn monitor_tenant_usage_activity(&self, request: MonitorTenantUsageRequest) -> Result<MonitorTenantUsageResult>;
    async fn process_tenant_billing_activity(&self, request: ProcessTenantBillingRequest) -> Result<ProcessTenantBillingResult>;
    async fn cleanup_tenant_data_activity(&self, request: CleanupTenantDataRequest) -> Result<CleanupTenantDataResult>;
    async fn migrate_tenant_data_activity(&self, request: MigrateTenantDataRequest) -> Result<MigrateTenantDataResult>;
}

// Implementation of tenant activities
pub struct TenantActivitiesImpl {
    tenant_service: Arc<TenantService>,
}

impl TenantActivitiesImpl {
    pub fn new(tenant_service: Arc<TenantService>) -> Self {
        Self { tenant_service }
    }

    #[cfg(test)]
    pub fn tenant_service(&self) -> &Arc<TenantService> {
        &self.tenant_service
    }

    // Helper methods for infrastructure provisioning
    async fn provision_database_infrastructure(&self, tenant_id: &str, config: &DatabaseConfig) -> Result<bool> {
        tracing::info!("Provisioning database infrastructure for tenant: {}", tenant_id);
        
        // Simulate database provisioning
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        match config.isolation_level {
            adx_shared::types::TenantIsolationLevel::Database => {
                // Create separate database
                tracing::info!("Creating separate database for tenant: {}", tenant_id);
            }
            adx_shared::types::TenantIsolationLevel::Schema => {
                // Create separate schema
                tracing::info!("Creating separate schema for tenant: {}", tenant_id);
            }
            adx_shared::types::TenantIsolationLevel::Row => {
                // Set up row-level security
                tracing::info!("Setting up row-level security for tenant: {}", tenant_id);
            }
        }

        if config.encryption_enabled {
            tracing::info!("Enabling database encryption for tenant: {}", tenant_id);
        }

        if config.backup_enabled {
            tracing::info!("Setting up automated backups for tenant: {}", tenant_id);
        }

        Ok(true)
    }

    async fn provision_storage_infrastructure(&self, tenant_id: &str, config: &StorageConfig) -> Result<bool> {
        tracing::info!("Provisioning storage infrastructure for tenant: {}", tenant_id);
        
        // Simulate storage provisioning
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        
        match config.storage_type.as_str() {
            "s3" => tracing::info!("Setting up S3 storage for tenant: {}", tenant_id),
            "gcs" => tracing::info!("Setting up Google Cloud Storage for tenant: {}", tenant_id),
            "azure" => tracing::info!("Setting up Azure Storage for tenant: {}", tenant_id),
            "local" => tracing::info!("Setting up local storage for tenant: {}", tenant_id),
            _ => tracing::warn!("Unknown storage type: {}", config.storage_type),
        }

        if config.encryption_enabled {
            tracing::info!("Enabling storage encryption for tenant: {}", tenant_id);
        }

        if config.versioning_enabled {
            tracing::info!("Enabling storage versioning for tenant: {}", tenant_id);
        }

        Ok(true)
    }

    async fn provision_compute_infrastructure(&self, tenant_id: &str, config: &ComputeConfig) -> Result<bool> {
        tracing::info!("Provisioning compute infrastructure for tenant: {}", tenant_id);
        
        // Simulate compute provisioning
        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
        
        tracing::info!("Setting CPU limit to {} cores for tenant: {}", config.cpu_limit, tenant_id);
        tracing::info!("Setting memory limit to {} GB for tenant: {}", config.memory_limit_gb, tenant_id);

        if config.auto_scaling_enabled {
            tracing::info!("Enabling auto-scaling for tenant: {}", tenant_id);
        }

        Ok(true)
    }

    async fn provision_network_infrastructure(&self, tenant_id: &str, config: &NetworkConfig) -> Result<bool> {
        tracing::info!("Provisioning network infrastructure for tenant: {}", tenant_id);
        
        // Simulate network provisioning
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        if let Some(domain) = &config.custom_domain {
            tracing::info!("Setting up custom domain {} for tenant: {}", domain, tenant_id);
        }

        if config.ssl_enabled {
            tracing::info!("Enabling SSL for tenant: {}", tenant_id);
        }

        if config.cdn_enabled {
            tracing::info!("Enabling CDN for tenant: {}", tenant_id);
        }

        Ok(true)
    }

    // Helper methods for RBAC
    async fn create_role(&self, tenant_id: &str, role_def: &RoleDefinition) -> Result<String> {
        tracing::info!("Creating role '{}' for tenant: {}", role_def.name, tenant_id);
        
        // Simulate role creation
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        let role_id = uuid::Uuid::new_v4().to_string();
        Ok(role_id)
    }

    async fn assign_permission_to_role(&self, role_id: &str, permission: &str) -> Result<()> {
        tracing::info!("Assigning permission '{}' to role: {}", permission, role_id);
        
        // Simulate permission assignment
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(())
    }

    async fn assign_role_to_user(&self, user_id: &str, role_id: &str) -> Result<()> {
        tracing::info!("Assigning role {} to user: {}", role_id, user_id);
        
        // Simulate role assignment
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        
        Ok(())
    }

    async fn create_default_permission(&self, tenant_id: &str, permission: &str) -> Result<()> {
        tracing::info!("Creating default permission '{}' for tenant: {}", permission, tenant_id);
        
        // Simulate permission creation
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(())
    }

    // Helper methods for usage monitoring
    async fn collect_usage_metric(&self, tenant_id: &str, metric: &UsageMetric, period: &MonitoringPeriod) -> Result<UsageData> {
        tracing::info!("Collecting {:?} metric for tenant: {} over {:?} period", metric, tenant_id, period);
        
        // Simulate metric collection
        tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
        
        // Generate mock usage data
        let (current_value, previous_value) = match metric {
            UsageMetric::ApiCalls => (850.0, 720.0),
            UsageMetric::StorageUsage => (4.2, 3.8),
            UsageMetric::ComputeTime => (120.5, 110.2),
            UsageMetric::BandwidthUsage => (15.6, 14.1),
            UsageMetric::ActiveUsers => (25.0, 23.0),
            UsageMetric::WorkflowExecutions => (45.0, 38.0),
            UsageMetric::DatabaseQueries => (1250.0, 1100.0),
        };

        let percentage_change = ((current_value - previous_value) / previous_value) * 100.0;
        let trend = if percentage_change > 5.0 {
            UsageTrend::Increasing
        } else if percentage_change < -5.0 {
            UsageTrend::Decreasing
        } else {
            UsageTrend::Stable
        };

        Ok(UsageData {
            metric_name: format!("{:?}", metric),
            current_value,
            previous_value,
            percentage_change,
            trend,
            timestamp: Utc::now(),
        })
    }

    async fn calculate_quota_status(&self, usage_data: &HashMap<String, UsageData>, quotas: &TenantQuotas) -> Result<QuotaStatus> {
        let api_calls_used = usage_data.get("ApiCalls").map(|u| u.current_value).unwrap_or(0.0);
        let storage_used = usage_data.get("StorageUsage").map(|u| u.current_value).unwrap_or(0.0);
        let compute_used = usage_data.get("ComputeTime").map(|u| u.current_value).unwrap_or(0.0);
        let users_used = usage_data.get("ActiveUsers").map(|u| u.current_value).unwrap_or(0.0);
        let workflows_used = usage_data.get("WorkflowExecutions").map(|u| u.current_value).unwrap_or(0.0);

        Ok(QuotaStatus {
            api_calls: QuotaInfo {
                used: api_calls_used,
                limit: quotas.max_api_calls_per_hour.unwrap_or(1000) as f64,
                percentage_used: (api_calls_used / quotas.max_api_calls_per_hour.unwrap_or(1000) as f64) * 100.0,
                is_exceeded: api_calls_used > quotas.max_api_calls_per_hour.unwrap_or(1000) as f64,
                warning_threshold: 80.0,
            },
            storage: QuotaInfo {
                used: storage_used,
                limit: quotas.max_storage_gb.unwrap_or(5) as f64,
                percentage_used: (storage_used / quotas.max_storage_gb.unwrap_or(5) as f64) * 100.0,
                is_exceeded: storage_used > quotas.max_storage_gb.unwrap_or(5) as f64,
                warning_threshold: 90.0,
            },
            compute: QuotaInfo {
                used: compute_used,
                limit: 200.0, // Default compute limit
                percentage_used: (compute_used / 200.0) * 100.0,
                is_exceeded: compute_used > 200.0,
                warning_threshold: 85.0,
            },
            users: QuotaInfo {
                used: users_used,
                limit: quotas.max_users.unwrap_or(10) as f64,
                percentage_used: (users_used / quotas.max_users.unwrap_or(10) as f64) * 100.0,
                is_exceeded: users_used > quotas.max_users.unwrap_or(10) as f64,
                warning_threshold: 90.0,
            },
            workflows: QuotaInfo {
                used: workflows_used,
                limit: quotas.max_workflows_per_hour.unwrap_or(100) as f64,
                percentage_used: (workflows_used / quotas.max_workflows_per_hour.unwrap_or(100) as f64) * 100.0,
                is_exceeded: workflows_used > quotas.max_workflows_per_hour.unwrap_or(100) as f64,
                warning_threshold: 80.0,
            },
        })
    }

    // Helper methods for data cleanup
    async fn create_tenant_backup(&self, tenant_id: &str) -> Result<BackupInfo> {
        tracing::info!("Creating backup for tenant: {}", tenant_id);
        
        // Simulate backup creation
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        Ok(BackupInfo {
            backup_id: uuid::Uuid::new_v4().to_string(),
            backup_location: format!("s3://backups/tenant-{}/{}", tenant_id, Utc::now().format("%Y%m%d")),
            backup_size_gb: 2.5,
            backup_timestamp: Utc::now(),
        })
    }

    async fn soft_delete_tenant_data(&self, tenant_id: &str, policy: &DataRetentionPolicy) -> Result<CleanupSummary> {
        tracing::info!("Performing soft delete for tenant: {}", tenant_id);
        
        // Simulate soft delete
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        
        let mut tables_cleaned = vec!["users".to_string(), "files".to_string(), "workflows".to_string()];
        
        if !policy.retain_audit_logs {
            tables_cleaned.push("audit_logs".to_string());
        }
        
        if !policy.retain_user_data {
            tables_cleaned.push("user_profiles".to_string());
        }

        Ok(CleanupSummary {
            tables_cleaned,
            records_deleted: 1250,
            storage_freed_gb: 0.0, // Soft delete doesn't free storage immediately
            cleanup_duration_ms: 200,
        })
    }

    async fn hard_delete_tenant_data(&self, tenant_id: &str, policy: &DataRetentionPolicy) -> Result<CleanupSummary> {
        tracing::info!("Performing hard delete for tenant: {}", tenant_id);
        
        // Simulate hard delete
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
        
        let mut tables_cleaned = vec!["users".to_string(), "files".to_string(), "workflows".to_string()];
        
        if !policy.retain_audit_logs {
            tables_cleaned.push("audit_logs".to_string());
        }
        
        if !policy.retain_user_data {
            tables_cleaned.push("user_profiles".to_string());
        }

        if !policy.retain_file_metadata {
            tables_cleaned.push("file_metadata".to_string());
        }

        Ok(CleanupSummary {
            tables_cleaned,
            records_deleted: 1250,
            storage_freed_gb: 4.2,
            cleanup_duration_ms: 800,
        })
    }

    async fn archive_tenant_data(&self, tenant_id: &str, _policy: &DataRetentionPolicy) -> Result<CleanupSummary> {
        tracing::info!("Archiving data for tenant: {}", tenant_id);
        
        // Simulate archiving
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        
        let tables_cleaned = vec![
            "users".to_string(), 
            "files".to_string(), 
            "workflows".to_string(),
            "audit_logs".to_string(),
        ];

        Ok(CleanupSummary {
            tables_cleaned,
            records_deleted: 0, // Archiving doesn't delete records
            storage_freed_gb: 3.1, // Some storage freed by compression
            cleanup_duration_ms: 600,
        })
    }

    // Helper methods for data migration
    async fn validate_migration(&self, request: &MigrateTenantDataRequest) -> Result<()> {
        tracing::info!("Validating migration for tenant: {}", request.tenant_id);
        
        // Simulate validation
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        
        // Check if target configuration is valid
        match request.migration_type {
            MigrationType::TierUpgrade => {
                // Validate upgrade path
                if request.source_config.current_tier == SubscriptionTier::Enterprise 
                    && request.target_config.target_tier == SubscriptionTier::Free {
                    return Err(anyhow::anyhow!("Cannot downgrade from Enterprise to Free"));
                }
            }
            MigrationType::RegionMigration => {
                // Validate region availability
                if request.target_config.target_region == "unsupported-region" {
                    return Err(anyhow::anyhow!("Target region is not supported"));
                }
            }
            _ => {}
        }

        Ok(())
    }

    async fn migrate_tier_upgrade(&self, request: &MigrateTenantDataRequest) -> Result<MigrationSummary> {
        tracing::info!("Performing tier upgrade migration for tenant: {}", request.tenant_id);
        
        // Simulate tier upgrade
        tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;
        
        Ok(MigrationSummary {
            migration_type: request.migration_type.clone(),
            records_migrated: 1500,
            data_size_gb: 3.2,
            migration_duration_ms: 400,
            downtime_ms: 0, // No downtime for tier upgrades
            success: true,
            errors: vec![],
        })
    }

    async fn migrate_tier_downgrade(&self, request: &MigrateTenantDataRequest) -> Result<MigrationSummary> {
        tracing::info!("Performing tier downgrade migration for tenant: {}", request.tenant_id);
        
        // Simulate tier downgrade
        tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;
        
        Ok(MigrationSummary {
            migration_type: request.migration_type.clone(),
            records_migrated: 1200,
            data_size_gb: 2.8,
            migration_duration_ms: 600,
            downtime_ms: 30000, // 30 seconds downtime for downgrades
            success: true,
            errors: vec![],
        })
    }

    async fn migrate_region(&self, request: &MigrateTenantDataRequest) -> Result<MigrationSummary> {
        tracing::info!("Performing region migration for tenant: {}", request.tenant_id);
        
        // Simulate region migration
        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
        
        Ok(MigrationSummary {
            migration_type: request.migration_type.clone(),
            records_migrated: 1500,
            data_size_gb: 3.2,
            migration_duration_ms: 2000,
            downtime_ms: 120000, // 2 minutes downtime for region migration
            success: true,
            errors: vec![],
        })
    }

    async fn migrate_isolation_level(&self, request: &MigrateTenantDataRequest) -> Result<MigrationSummary> {
        tracing::info!("Performing isolation level migration for tenant: {}", request.tenant_id);
        
        // Simulate isolation level migration
        tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;
        
        Ok(MigrationSummary {
            migration_type: request.migration_type.clone(),
            records_migrated: 1500,
            data_size_gb: 3.2,
            migration_duration_ms: 800,
            downtime_ms: 60000, // 1 minute downtime for isolation changes
            success: true,
            errors: vec![],
        })
    }

    async fn migrate_storage_provider(&self, request: &MigrateTenantDataRequest) -> Result<MigrationSummary> {
        tracing::info!("Performing storage provider migration for tenant: {}", request.tenant_id);
        
        // Simulate storage provider migration
        tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
        
        Ok(MigrationSummary {
            migration_type: request.migration_type.clone(),
            records_migrated: 800, // Only file metadata
            data_size_gb: 5.1, // All file data
            migration_duration_ms: 1500,
            downtime_ms: 90000, // 1.5 minutes downtime for storage migration
            success: true,
            errors: vec![],
        })
    }

    async fn get_quotas_for_tier(&self, tier: &SubscriptionTier) -> Result<TenantQuotas> {
        match tier {
            SubscriptionTier::Free => Ok(TenantQuotas {
                max_users: Some(5),
                max_storage_gb: Some(1),
                max_api_calls_per_hour: Some(100),
                max_workflows_per_hour: Some(10),
            }),
            SubscriptionTier::Professional => Ok(TenantQuotas {
                max_users: Some(25),
                max_storage_gb: Some(10),
                max_api_calls_per_hour: Some(1000),
                max_workflows_per_hour: Some(100),
            }),
            SubscriptionTier::Enterprise => Ok(TenantQuotas {
                max_users: Some(100),
                max_storage_gb: Some(100),
                max_api_calls_per_hour: Some(10000),
                max_workflows_per_hour: Some(1000),
            }),
            SubscriptionTier::Custom => Ok(TenantQuotas {
                max_users: None, // Unlimited
                max_storage_gb: None, // Unlimited
                max_api_calls_per_hour: None, // Unlimited
                max_workflows_per_hour: None, // Unlimited
            }),
        }
    }

    async fn get_features_for_tier(&self, tier: &SubscriptionTier) -> Result<Vec<String>> {
        match tier {
            SubscriptionTier::Free => Ok(vec![
                "basic_auth".to_string(),
                "file_storage".to_string(),
            ]),
            SubscriptionTier::Professional => Ok(vec![
                "basic_auth".to_string(),
                "file_storage".to_string(),
                "advanced_workflows".to_string(),
                "api_access".to_string(),
                "email_support".to_string(),
            ]),
            SubscriptionTier::Enterprise => Ok(vec![
                "basic_auth".to_string(),
                "file_storage".to_string(),
                "advanced_workflows".to_string(),
                "api_access".to_string(),
                "email_support".to_string(),
                "sso_integration".to_string(),
                "custom_branding".to_string(),
                "priority_support".to_string(),
                "audit_logs".to_string(),
            ]),
            SubscriptionTier::Custom => Ok(vec![
                "all_features".to_string(),
            ]),
        }
    }
}

#[async_trait]
impl TenantActivities for TenantActivitiesImpl {
    async fn validate_tenant_creation(&self, request: ValidateTenantCreationRequest) -> Result<TenantValidationResult> {
        let mut errors = Vec::new();

        // Validate tenant name
        if request.tenant_name.trim().is_empty() {
            errors.push("Tenant name cannot be empty".to_string());
        }

        if request.tenant_name.len() < 3 {
            errors.push("Tenant name must be at least 3 characters long".to_string());
        }

        if request.tenant_name.len() > 100 {
            errors.push("Tenant name cannot exceed 100 characters".to_string());
        }

        // Check if tenant name already exists
        if let Ok(Some(_)) = self.tenant_service.get_tenant_by_slug(&request.tenant_name.to_lowercase()).await {
            errors.push("Tenant with this name already exists".to_string());
        }

        // Validate admin email
        if !request.admin_email.contains('@') {
            errors.push("Invalid admin email format".to_string());
        }

        let is_valid = errors.is_empty();
        let tenant_id = if is_valid {
            uuid::Uuid::new_v4().to_string()
        } else {
            String::new()
        };

        Ok(TenantValidationResult {
            is_valid,
            tenant_id,
            errors,
        })
    }

    async fn setup_tenant_database(&self, request: SetupTenantDatabaseRequest) -> Result<DatabaseSetupResult> {
        // In a real implementation, this would:
        // 1. Create a new database schema or database based on isolation level
        // 2. Run migrations for the tenant
        // 3. Set up initial data
        
        tracing::info!("Setting up database for tenant: {}", request.tenant_id);
        
        // Simulate database setup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let connection_string = match request.isolation_level {
            adx_shared::types::TenantIsolationLevel::Database => {
                format!("postgresql://user:pass@localhost/tenant_{}", request.tenant_id)
            }
            adx_shared::types::TenantIsolationLevel::Schema => {
                format!("postgresql://user:pass@localhost/adx_core?search_path=tenant_{}", request.tenant_id)
            }
            adx_shared::types::TenantIsolationLevel::Row => {
                "postgresql://user:pass@localhost/adx_core".to_string()
            }
        };

        Ok(DatabaseSetupResult {
            connection_string,
            schema_created: true,
        })
    }

    async fn create_tenant_config(&self, request: CreateTenantConfigRequest) -> Result<Tenant> {
        let create_request = CreateTenantRequest {
            name: request.tenant_name,
            admin_email: "admin@example.com".to_string(), // This would come from the workflow
            subscription_tier: Some(request.subscription_tier),
            isolation_level: None,
            features: Some(request.features),
            settings: None,
        };

        self.tenant_service.create_tenant(create_request).await
    }

    async fn cleanup_tenant_database(&self, tenant_id: &TenantId) -> Result<()> {
        // In a real implementation, this would clean up tenant-specific database resources
        tracing::info!("Cleaning up database for tenant: {}", tenant_id);
        
        // Simulate cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        Ok(())
    }

    async fn validate_user_tenant_access(&self, request: ValidateUserTenantAccessRequest) -> Result<UserTenantAccessResult> {
        match self.tenant_service.validate_tenant_access(&request.target_tenant_id, &request.user_id).await {
            Ok(has_access) => {
                let reason = if !has_access {
                    Some("User does not have access to the target tenant".to_string())
                } else {
                    None
                };

                Ok(UserTenantAccessResult {
                    has_access,
                    reason,
                })
            }
            Err(e) => Ok(UserTenantAccessResult {
                has_access: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn save_session_state(&self, request: SaveSessionStateRequest) -> Result<SessionStateResult> {
        // In a real implementation, this would save the current session state
        // to Redis or another session store
        tracing::info!("Saving session state for user: {}", request.user_id);
        
        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Simulate saving session state
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(SessionStateResult {
            session_id,
            saved: true,
        })
    }

    async fn load_tenant_context(&self, request: LoadTenantContextRequest) -> Result<TenantContext> {
        self.tenant_service.get_tenant_context(&request.tenant_id, &request.user_id).await
    }

    async fn create_tenant_session(&self, request: CreateTenantSessionRequest) -> Result<TenantSessionResult> {
        // In a real implementation, this would create a new session in the session store
        tracing::info!("Creating tenant session for user: {} in tenant: {}", request.user_id, request.tenant_id);
        
        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Simulate session creation
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(TenantSessionResult {
            session_id,
            available_features: request.tenant_context.features,
        })
    }

    async fn update_user_active_tenant(&self, request: UpdateUserActiveTenantRequest) -> Result<()> {
        // In a real implementation, this would update the user's active tenant
        // in the user service or user database
        tracing::info!("Updating active tenant for user: {} to tenant: {}", request.user_id, request.new_active_tenant_id);
        
        // Simulate user update
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(())
    }

    // Implementation of new activities for Task 13

    async fn create_tenant_activity(&self, request: CreateTenantActivityRequest) -> Result<CreateTenantActivityResult> {
        tracing::info!("Creating tenant with infrastructure provisioning: {}", request.tenant_name);
        
        let start_time = std::time::Instant::now();
        let tenant_id = uuid::Uuid::new_v4().to_string();

        // Step 1: Provision database infrastructure
        let database_ready = self.provision_database_infrastructure(&tenant_id, &request.infrastructure_config.database_config).await?;
        
        // Step 2: Provision storage infrastructure
        let storage_ready = self.provision_storage_infrastructure(&tenant_id, &request.infrastructure_config.storage_config).await?;
        
        // Step 3: Provision compute infrastructure
        let compute_ready = self.provision_compute_infrastructure(&tenant_id, &request.infrastructure_config.compute_config).await?;
        
        // Step 4: Provision network infrastructure
        let network_ready = self.provision_network_infrastructure(&tenant_id, &request.infrastructure_config.network_config).await?;

        // Step 5: Create tenant record
        let create_request = CreateTenantRequest {
            name: request.tenant_name.clone(),
            admin_email: request.admin_email,
            subscription_tier: Some(request.subscription_tier),
            isolation_level: Some(request.isolation_level),
            features: Some(request.features),
            settings: None,
        };

        let _tenant = self.tenant_service.create_tenant(create_request).await?;

        let provisioning_time = start_time.elapsed().as_millis() as u64;

        Ok(CreateTenantActivityResult {
            tenant_id: tenant_id.clone(),
            infrastructure_status: InfrastructureStatus {
                database_ready,
                storage_ready,
                compute_ready,
                network_ready,
                provisioning_time_ms: provisioning_time,
            },
            connection_details: ConnectionDetails {
                database_connection: format!("postgresql://user:pass@localhost/tenant_{}", tenant_id),
                storage_endpoint: format!("https://storage.adxcore.com/tenant/{}", tenant_id),
                api_endpoint: format!("https://api.adxcore.com/tenant/{}", tenant_id),
            },
        })
    }

    async fn setup_tenant_permissions_activity(&self, request: SetupTenantPermissionsRequest) -> Result<SetupTenantPermissionsResult> {
        tracing::info!("Setting up RBAC permissions for tenant: {}", request.tenant_id);

        let mut roles_created = Vec::new();
        let mut permissions_assigned = Vec::new();
        let mut admin_role_id = String::new();

        // Create role definitions
        for role_def in &request.role_definitions {
            let role_id = self.create_role(&request.tenant_id, role_def).await?;
            roles_created.push(role_def.name.clone());
            
            let is_admin = role_def.name.to_lowercase() == "admin";
            if is_admin {
                admin_role_id = role_id.clone();
            }

            // Assign permissions to role
            for permission in &role_def.permissions {
                self.assign_permission_to_role(&role_id, permission).await?;
                permissions_assigned.push(permission.clone());
            }
        }

        // Assign admin role to the admin user
        if !admin_role_id.is_empty() {
            self.assign_role_to_user(&request.admin_user_id, &admin_role_id).await?;
        }

        // Set up default permissions
        for permission in &request.default_permissions {
            self.create_default_permission(&request.tenant_id, permission).await?;
            permissions_assigned.push(permission.clone());
        }

        Ok(SetupTenantPermissionsResult {
            roles_created,
            permissions_assigned,
            admin_role_id,
        })
    }

    async fn monitor_tenant_usage_activity(&self, request: MonitorTenantUsageRequest) -> Result<MonitorTenantUsageResult> {
        tracing::info!("Monitoring usage for tenant: {}", request.tenant_id);

        let mut usage_data = HashMap::new();
        let mut alerts = Vec::new();
        let mut recommendations = Vec::new();

        // Collect usage metrics
        for metric in &request.metrics_to_track {
            let usage = self.collect_usage_metric(&request.tenant_id, metric, &request.monitoring_period).await?;
            usage_data.insert(format!("{:?}", metric), usage);
        }

        // Get current quotas for the tenant
        let tenant = self.tenant_service.get_tenant(&request.tenant_id).await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {}", request.tenant_id))?;
        let quotas = &tenant.quotas;

        // Calculate quota status
        let quota_status = self.calculate_quota_status(&usage_data, quotas).await?;

        // Generate alerts based on usage
        if quota_status.api_calls.percentage_used > 90.0 {
            alerts.push(UsageAlert {
                alert_type: AlertType::QuotaWarning,
                metric: "api_calls".to_string(),
                current_value: quota_status.api_calls.used,
                threshold: quota_status.api_calls.limit * 0.9,
                severity: AlertSeverity::High,
                message: "API call quota is approaching limit".to_string(),
                timestamp: Utc::now(),
            });
        }

        if quota_status.storage.percentage_used > 95.0 {
            alerts.push(UsageAlert {
                alert_type: AlertType::QuotaExceeded,
                metric: "storage".to_string(),
                current_value: quota_status.storage.used,
                threshold: quota_status.storage.limit,
                severity: AlertSeverity::Critical,
                message: "Storage quota exceeded".to_string(),
                timestamp: Utc::now(),
            });
        }

        // Generate recommendations
        if quota_status.compute.percentage_used > 80.0 {
            recommendations.push("Consider upgrading to a higher tier for better compute resources".to_string());
        }

        if usage_data.get("ActiveUsers").map(|u| u.current_value).unwrap_or(0.0) > quota_status.users.limit * 0.8 {
            recommendations.push("User count is approaching limit, consider adding more user licenses".to_string());
        }

        Ok(MonitorTenantUsageResult {
            tenant_id: request.tenant_id,
            usage_data,
            quota_status,
            alerts,
            recommendations,
        })
    }

    async fn process_tenant_billing_activity(&self, request: ProcessTenantBillingRequest) -> Result<ProcessTenantBillingResult> {
        tracing::info!("Processing billing for tenant: {}", request.tenant_id);

        let mut line_items = Vec::new();
        let mut subtotal = Decimal::new(0, 0);

        // Add base subscription fee
        line_items.push(BillingLineItem {
            description: "Base subscription fee".to_string(),
            quantity: 1.0,
            unit_price: request.pricing_model.base_price,
            total_price: request.pricing_model.base_price,
            metric_type: "subscription".to_string(),
        });
        subtotal += request.pricing_model.base_price;

        // Calculate usage-based charges
        for (metric, usage_amount) in &request.usage_data {
            if let Some(rate) = request.pricing_model.usage_rates.get(metric) {
                let total_price = rate * Decimal::from_f64_retain(*usage_amount).unwrap_or_default();
                
                line_items.push(BillingLineItem {
                    description: format!("{} usage", metric),
                    quantity: *usage_amount,
                    unit_price: *rate,
                    total_price,
                    metric_type: metric.clone(),
                });
                
                subtotal += total_price;
            }
        }

        // Apply tier discounts
        let tenant = self.tenant_service.get_tenant(&request.tenant_id).await?
            .ok_or_else(|| anyhow::anyhow!("Tenant not found: {}", request.tenant_id))?;
        let tier_key = format!("{:?}", tenant.subscription_tier);
        if let Some(discount) = request.pricing_model.tier_discounts.get(&tier_key) {
            let discount_amount = subtotal * discount;
            subtotal -= discount_amount;
            
            line_items.push(BillingLineItem {
                description: format!("{} tier discount", tier_key),
                quantity: 1.0,
                unit_price: -discount_amount,
                total_price: -discount_amount,
                metric_type: "discount".to_string(),
            });
        }

        // Calculate taxes (simplified - 10% tax rate)
        let tax_rate = Decimal::new(10, 2); // 0.10
        let taxes = subtotal * tax_rate;
        let total_amount = subtotal + taxes;

        // Generate invoice ID
        let invoice_id = format!("INV-{}-{}", request.tenant_id, Utc::now().format("%Y%m%d"));

        // Set payment due date (30 days from now)
        let payment_due_date = Utc::now() + chrono::Duration::days(30);

        Ok(ProcessTenantBillingResult {
            tenant_id: request.tenant_id,
            billing_period: request.billing_period,
            line_items,
            subtotal,
            taxes,
            total_amount,
            invoice_id,
            payment_due_date,
        })
    }

    async fn cleanup_tenant_data_activity(&self, request: CleanupTenantDataRequest) -> Result<CleanupTenantDataResult> {
        tracing::info!("Cleaning up data for tenant: {} with type: {:?}", request.tenant_id, request.cleanup_type);

        let start_time = std::time::Instant::now();
        let mut backup_info = None;

        // Create backup if requested
        if request.backup_before_cleanup {
            backup_info = Some(self.create_tenant_backup(&request.tenant_id).await?);
        }

        // Perform cleanup based on type
        let cleanup_summary = match request.cleanup_type {
            CleanupType::SoftDelete => self.soft_delete_tenant_data(&request.tenant_id, &request.data_retention_policy).await?,
            CleanupType::HardDelete => self.hard_delete_tenant_data(&request.tenant_id, &request.data_retention_policy).await?,
            CleanupType::Archive => self.archive_tenant_data(&request.tenant_id, &request.data_retention_policy).await?,
        };

        let cleanup_duration = start_time.elapsed().as_millis() as u64;

        Ok(CleanupTenantDataResult {
            tenant_id: request.tenant_id,
            cleanup_summary: CleanupSummary {
                tables_cleaned: cleanup_summary.tables_cleaned,
                records_deleted: cleanup_summary.records_deleted,
                storage_freed_gb: cleanup_summary.storage_freed_gb,
                cleanup_duration_ms: cleanup_duration,
            },
            backup_info,
            cleanup_timestamp: Utc::now(),
        })
    }

    async fn migrate_tenant_data_activity(&self, request: MigrateTenantDataRequest) -> Result<MigrateTenantDataResult> {
        tracing::info!("Migrating data for tenant: {} with type: {:?}", request.tenant_id, request.migration_type);

        let start_time = std::time::Instant::now();
        let migration_id = uuid::Uuid::new_v4().to_string();
        let mut rollback_info = None;

        // Validate migration if requested
        if request.migration_options.validate_before_migration {
            self.validate_migration(&request).await?;
        }

        // Create backup for rollback if requested
        if request.migration_options.create_backup {
            let backup = self.create_tenant_backup(&request.tenant_id).await?;
            rollback_info = Some(RollbackInfo {
                rollback_id: format!("rollback-{}", migration_id),
                rollback_available_until: Utc::now() + chrono::Duration::days(7),
                rollback_data_location: backup.backup_location,
            });
        }

        // Perform migration based on type
        let migration_summary = match request.migration_type {
            MigrationType::TierUpgrade => self.migrate_tier_upgrade(&request).await?,
            MigrationType::TierDowngrade => self.migrate_tier_downgrade(&request).await?,
            MigrationType::RegionMigration => self.migrate_region(&request).await?,
            MigrationType::IsolationLevelChange => self.migrate_isolation_level(&request).await?,
            MigrationType::StorageProviderChange => self.migrate_storage_provider(&request).await?,
        };

        let migration_duration = start_time.elapsed().as_millis() as u64;

        // Update tenant configuration
        let new_configuration = TenantConfiguration {
            tier: request.target_config.target_tier.clone(),
            isolation_level: request.target_config.target_isolation.clone(),
            region: request.target_config.target_region.clone(),
            storage_provider: request.target_config.target_storage_provider.clone(),
            quotas: self.get_quotas_for_tier(&request.target_config.target_tier).await?,
            features: self.get_features_for_tier(&request.target_config.target_tier).await?,
        };

        Ok(MigrateTenantDataResult {
            tenant_id: request.tenant_id,
            migration_id,
            migration_summary: MigrationSummary {
                migration_type: request.migration_type,
                records_migrated: migration_summary.records_migrated,
                data_size_gb: migration_summary.data_size_gb,
                migration_duration_ms: migration_duration,
                downtime_ms: migration_summary.downtime_ms,
                success: migration_summary.success,
                errors: migration_summary.errors,
            },
            new_configuration,
            rollback_info,
        })
    }
}