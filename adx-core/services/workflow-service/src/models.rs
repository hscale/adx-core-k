use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Workflow execution status
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum WorkflowExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
    Paused,
    Terminated,
}

// Workflow progress information
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowProgressInfo {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
    pub status_message: Option<String>,
}

// Workflow history event
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowHistoryEvent {
    pub event_id: u64,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub details: serde_json::Value,
}

// Common workflow context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    pub workflow_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub correlation_id: Option<String>,
    pub metadata: HashMap<String, String>,
    pub started_at: DateTime<Utc>,
}

// User Onboarding Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingRequest {
    pub user_email: String,
    pub user_name: String,
    pub tenant_id: String,
    pub role: String,
    pub send_welcome_email: bool,
    pub setup_default_workspace: bool,
    pub assign_default_permissions: bool,
    pub create_sample_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingResult {
    pub user_id: String,
    pub tenant_id: String,
    pub workspace_id: Option<String>,
    pub permissions_assigned: Vec<String>,
    pub welcome_email_sent: bool,
    pub sample_data_created: bool,
    pub onboarding_completed_at: DateTime<Utc>,
}

// Tenant Switching Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSwitchingRequest {
    pub user_id: String,
    pub current_tenant_id: String,
    pub target_tenant_id: String,
    pub preserve_session_data: bool,
    pub update_user_preferences: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSwitchingResult {
    pub user_id: String,
    pub new_tenant_id: String,
    pub new_session_id: String,
    pub updated_permissions: Vec<String>,
    pub tenant_context: TenantContext,
    pub switch_completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub tenant_name: String,
    pub subscription_tier: String,
    pub features: Vec<String>,
    pub quotas: HashMap<String, u64>,
    pub settings: HashMap<String, String>,
}

// Data Migration Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMigrationRequest {
    pub migration_id: String,
    pub source_tenant_id: Option<String>,
    pub target_tenant_id: String,
    pub migration_type: DataMigrationType,
    pub data_selectors: Vec<DataSelector>,
    pub migration_options: MigrationOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataMigrationType {
    TenantToTenant,
    SystemUpgrade,
    DataArchival,
    DataRestore,
    CrossServiceSync,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSelector {
    pub service: String,
    pub entity_type: String,
    pub filters: HashMap<String, String>,
    pub include_relationships: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationOptions {
    pub batch_size: usize,
    pub parallel_workers: usize,
    pub validate_data: bool,
    pub create_backup: bool,
    pub rollback_on_error: bool,
    pub dry_run: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataMigrationResult {
    pub migration_id: String,
    pub status: MigrationStatus,
    pub records_processed: u64,
    pub records_migrated: u64,
    pub records_failed: u64,
    pub services_affected: Vec<String>,
    pub backup_id: Option<String>,
    pub error_summary: Option<String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MigrationStatus {
    InProgress,
    Completed,
    Failed,
    RolledBack,
}

// Bulk Operation Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationRequest {
    pub operation_id: String,
    pub tenant_id: String,
    pub operation_type: BulkOperationType,
    pub target_entities: Vec<EntityTarget>,
    pub operation_data: serde_json::Value,
    pub batch_options: BatchOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulkOperationType {
    UserCreation,
    UserUpdate,
    UserDeactivation,
    PermissionUpdate,
    DataExport,
    DataImport,
    FileProcessing,
    NotificationSend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityTarget {
    pub service: String,
    pub entity_type: String,
    pub entity_id: String,
    pub operation_data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchOptions {
    pub batch_size: usize,
    pub parallel_batches: usize,
    pub delay_between_batches_ms: u64,
    pub continue_on_error: bool,
    pub max_retries: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulkOperationResult {
    pub operation_id: String,
    pub total_entities: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub batches_processed: u64,
    pub error_details: Vec<OperationError>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationError {
    pub entity_id: String,
    pub error_message: String,
    pub retry_count: u32,
}

// Compliance Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceWorkflowRequest {
    pub compliance_id: String,
    pub tenant_id: String,
    pub compliance_type: ComplianceType,
    pub subject_user_id: Option<String>,
    pub data_categories: Vec<String>,
    pub retention_policy: Option<RetentionPolicy>,
    pub audit_requirements: AuditRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceType {
    GdprDataExport,
    GdprDataDeletion,
    DataRetentionEnforcement,
    AuditLogGeneration,
    ComplianceReport,
    DataClassification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionPolicy {
    pub retention_period_days: u32,
    pub auto_delete: bool,
    pub archive_before_delete: bool,
    pub notification_before_deletion_days: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditRequirements {
    pub include_access_logs: bool,
    pub include_modification_logs: bool,
    pub include_deletion_logs: bool,
    pub include_export_logs: bool,
    pub date_range: Option<DateRange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceWorkflowResult {
    pub compliance_id: String,
    pub status: ComplianceStatus,
    pub data_exported: Option<DataExportSummary>,
    pub data_deleted: Option<DataDeletionSummary>,
    pub audit_report: Option<AuditReportSummary>,
    pub compliance_certificate: Option<String>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceStatus {
    InProgress,
    Completed,
    Failed,
    PartiallyCompleted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExportSummary {
    pub export_file_path: String,
    pub total_records: u64,
    pub services_included: Vec<String>,
    pub export_format: String,
    pub encryption_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataDeletionSummary {
    pub total_records_deleted: u64,
    pub services_affected: Vec<String>,
    pub backup_created: bool,
    pub backup_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditReportSummary {
    pub report_file_path: String,
    pub total_audit_entries: u64,
    pub date_range_covered: DateRange,
    pub report_format: String,
}

// Workflow Progress Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub workflow_id: String,
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
    pub status_message: Option<String>,
}

// Service Activity Results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceActivityResult<T> {
    pub service: String,
    pub activity: String,
    pub success: bool,
    pub result: Option<T>,
    pub error: Option<String>,
    pub execution_time_ms: u64,
    pub retry_count: u32,
}

impl<T> ServiceActivityResult<T> {
    pub fn success(service: &str, activity: &str, result: T, execution_time_ms: u64) -> Self {
        Self {
            service: service.to_string(),
            activity: activity.to_string(),
            success: true,
            result: Some(result),
            error: None,
            execution_time_ms,
            retry_count: 0,
        }
    }

    pub fn failure(service: &str, activity: &str, error: String, execution_time_ms: u64, retry_count: u32) -> Self {
        Self {
            service: service.to_string(),
            activity: activity.to_string(),
            success: false,
            result: None,
            error: Some(error),
            execution_time_ms,
            retry_count,
        }
    }
}