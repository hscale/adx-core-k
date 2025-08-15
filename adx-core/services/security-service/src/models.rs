use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;

// Audit Log Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLog {
    pub id: Uuid,
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: String,
    pub event_category: AuditEventCategory,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub outcome: AuditOutcome,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub details: serde_json::Value,
    pub risk_score: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_event_category", rename_all = "lowercase")]
pub enum AuditEventCategory {
    Authentication,
    Authorization,
    DataAccess,
    DataModification,
    SystemAccess,
    Configuration,
    Security,
    Compliance,
    Privacy,
    Administrative,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "audit_outcome", rename_all = "lowercase")]
pub enum AuditOutcome {
    Success,
    Failure,
    Warning,
    Error,
}

// Compliance Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceReport {
    pub id: Uuid,
    pub tenant_id: String,
    pub report_type: ComplianceReportType,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub status: ComplianceStatus,
    pub findings: serde_json::Value,
    pub recommendations: serde_json::Value,
    pub risk_level: RiskLevel,
    pub generated_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_report_type", rename_all = "lowercase")]
pub enum ComplianceReportType {
    Gdpr,
    Soc2,
    Iso27001,
    Hipaa,
    Pci,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "compliance_status", rename_all = "lowercase")]
pub enum ComplianceStatus {
    Compliant,
    NonCompliant,
    PartiallyCompliant,
    UnderReview,
    Remediated,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "risk_level", rename_all = "lowercase")]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

// GDPR Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct GdprRequest {
    pub id: Uuid,
    pub tenant_id: String,
    pub user_id: String,
    pub request_type: GdprRequestType,
    pub status: GdprRequestStatus,
    pub requester_email: String,
    pub verification_token: Option<String>,
    pub verified_at: Option<DateTime<Utc>>,
    pub processed_at: Option<DateTime<Utc>>,
    pub data_export_url: Option<String>,
    pub deletion_confirmation: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "gdpr_request_type", rename_all = "lowercase")]
pub enum GdprRequestType {
    DataExport,
    DataDeletion,
    DataPortability,
    DataRectification,
    DataRestriction,
    DataObjection,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "gdpr_request_status", rename_all = "lowercase")]
pub enum GdprRequestStatus {
    Pending,
    Verified,
    Processing,
    Completed,
    Rejected,
    Expired,
}

// Data Retention Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRetentionPolicy {
    pub id: Uuid,
    pub tenant_id: String,
    pub resource_type: String,
    pub retention_period_days: i32,
    pub deletion_method: DeletionMethod,
    pub enabled: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "deletion_method", rename_all = "lowercase")]
pub enum DeletionMethod {
    SoftDelete,
    HardDelete,
    Anonymize,
    Archive,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DataRetentionJob {
    pub id: Uuid,
    pub tenant_id: String,
    pub policy_id: Uuid,
    pub resource_type: String,
    pub scheduled_for: DateTime<Utc>,
    pub status: RetentionJobStatus,
    pub records_processed: i64,
    pub records_deleted: i64,
    pub error_message: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "retention_job_status", rename_all = "lowercase")]
pub enum RetentionJobStatus {
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// Security Scanning Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityScan {
    pub id: Uuid,
    pub tenant_id: String,
    pub scan_type: ScanType,
    pub target: String,
    pub status: ScanStatus,
    pub severity_threshold: String,
    pub vulnerabilities_found: i32,
    pub critical_count: i32,
    pub high_count: i32,
    pub medium_count: i32,
    pub low_count: i32,
    pub scan_results: serde_json::Value,
    pub remediation_suggestions: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "scan_type", rename_all = "lowercase")]
pub enum ScanType {
    Vulnerability,
    Dependency,
    Configuration,
    Network,
    Application,
    Infrastructure,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "scan_status", rename_all = "lowercase")]
pub enum ScanStatus {
    Queued,
    Running,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub cve_id: Option<String>,
    pub title: String,
    pub description: String,
    pub severity: VulnerabilitySeverity,
    pub cvss_score: Option<f32>,
    pub affected_component: String,
    pub fixed_version: Option<String>,
    pub references: Vec<String>,
    pub discovered_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "vulnerability_severity", rename_all = "lowercase")]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

// Zero Trust Models
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ZeroTrustPolicy {
    pub id: Uuid,
    pub tenant_id: String,
    pub name: String,
    pub description: String,
    pub policy_type: ZeroTrustPolicyType,
    pub conditions: serde_json::Value,
    pub actions: serde_json::Value,
    pub enabled: bool,
    pub priority: i32,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "zero_trust_policy_type", rename_all = "lowercase")]
pub enum ZeroTrustPolicyType {
    NetworkAccess,
    DeviceVerification,
    UserAuthentication,
    ResourceAccess,
    DataProtection,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct SecurityEvent {
    pub id: Uuid,
    pub tenant_id: String,
    pub event_type: SecurityEventType,
    pub severity: SecurityEventSeverity,
    pub source_ip: Option<String>,
    pub user_id: Option<String>,
    pub device_id: Option<String>,
    pub resource: Option<String>,
    pub description: String,
    pub details: serde_json::Value,
    pub status: SecurityEventStatus,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "security_event_type", rename_all = "lowercase")]
pub enum SecurityEventType {
    UnauthorizedAccess,
    SuspiciousActivity,
    PolicyViolation,
    DataBreach,
    MalwareDetection,
    AnomalousLogin,
    PrivilegeEscalation,
    DataExfiltration,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "security_event_severity", rename_all = "lowercase")]
pub enum SecurityEventSeverity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "security_event_status", rename_all = "lowercase")]
pub enum SecurityEventStatus {
    Open,
    InProgress,
    Resolved,
    FalsePositive,
    Suppressed,
}

// Request/Response Models
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAuditLogRequest {
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub event_type: String,
    pub event_category: AuditEventCategory,
    pub resource_type: String,
    pub resource_id: Option<String>,
    pub action: String,
    pub outcome: AuditOutcome,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub request_id: Option<String>,
    pub details: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GdprExportRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub requester_email: String,
    pub include_deleted: bool,
    pub format: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GdprDeletionRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub requester_email: String,
    pub verification_required: bool,
    pub delete_backups: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityScanRequest {
    pub tenant_id: String,
    pub scan_type: ScanType,
    pub target: String,
    pub severity_threshold: String,
    pub notify_on_completion: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReportRequest {
    pub tenant_id: String,
    pub report_type: ComplianceReportType,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub include_recommendations: bool,
}

// Response Models
#[derive(Debug, Serialize, Deserialize)]
pub struct AuditLogResponse {
    pub logs: Vec<AuditLog>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReportResponse {
    pub report: ComplianceReport,
    pub summary: ComplianceSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total_checks: i32,
    pub passed_checks: i32,
    pub failed_checks: i32,
    pub compliance_percentage: f32,
    pub risk_distribution: HashMap<RiskLevel, i32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SecurityScanResponse {
    pub scan: SecurityScan,
    pub vulnerabilities: Vec<Vulnerability>,
    pub summary: ScanSummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanSummary {
    pub total_vulnerabilities: i32,
    pub by_severity: HashMap<VulnerabilitySeverity, i32>,
    pub remediation_priority: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GdprExportResponse {
    pub request_id: Uuid,
    pub status: GdprRequestStatus,
    pub download_url: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataRetentionSummary {
    pub total_policies: i32,
    pub active_policies: i32,
    pub scheduled_jobs: i32,
    pub records_to_delete: i64,
    pub next_cleanup: Option<DateTime<Utc>>,
}