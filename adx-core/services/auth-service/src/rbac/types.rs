use adx_shared::{TenantId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// RBAC CORE TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Role {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub tenant_id: TenantId,
    pub parent_role_id: Option<Uuid>, // For role hierarchy
    pub permissions: Vec<Permission>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permission {
    pub id: Uuid,
    pub resource: String, // e.g., "files", "workflows", "users"
    pub action: String,   // e.g., "read", "write", "delete", "execute"
    pub conditions: Vec<PermissionCondition>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PermissionCondition {
    TenantOwner,              // User owns the tenant
    ResourceOwner,            // User owns the specific resource
    DepartmentMember(String), // User is in specific department
    CustomCondition(String),  // Custom business logic
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRole {
    pub id: Uuid,
    pub user_id: UserId,
    pub role_id: Uuid,
    pub tenant_id: TenantId,
    pub assigned_by: UserId,
    pub assigned_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

// ============================================================================
// WORKFLOW INPUT/OUTPUT TYPES (Complex Operations)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignmentInput {
    pub user_id: UserId,
    pub role_id: Uuid,
    pub tenant_id: TenantId,
    pub assigned_by: UserId,
    pub reason: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub requires_approval: bool,
    pub auto_approve_conditions: Vec<AutoApprovalCondition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleAssignmentOutput {
    pub assignment_id: Uuid,
    pub user_id: UserId,
    pub role_id: Uuid,
    pub status: AssignmentStatus,
    pub assigned_at: DateTime<Utc>,
    pub approved_by: Option<UserId>,
    pub workflow_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutoApprovalCondition {
    SameRole,            // User already has a similar role
    LowerPrivilege,      // New role has fewer permissions
    TemporaryAssignment, // Assignment is temporary (<24 hours)
    DepartmentHead,      // Assigned by department head
}

// ============================================================================
// PERMISSION AUDIT WORKFLOW TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAuditInput {
    pub tenant_id: TenantId,
    pub audit_type: AuditType,
    pub user_filter: Option<UserId>,
    pub role_filter: Option<Uuid>,
    pub date_range: DateRange,
    pub requested_by: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionAuditOutput {
    pub audit_id: Uuid,
    pub report_url: String,
    pub total_users: u32,
    pub total_roles: u32,
    pub violations_found: u32,
    pub recommendations: Vec<SecurityRecommendation>,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AuditType {
    ComplianceReview,      // Regulatory compliance audit
    SecurityAssessment,    // Security posture review
    AccessCertification,   // Periodic access review
    IncidentInvestigation, // Post-incident analysis
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRecommendation {
    pub severity: SecuritySeverity,
    pub category: String,
    pub description: String,
    pub affected_users: Vec<UserId>,
    pub remediation_steps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

// ============================================================================
// ACCESS REVIEW WORKFLOW TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReviewInput {
    pub tenant_id: TenantId,
    pub review_scope: ReviewScope,
    pub reviewer_id: UserId,
    pub deadline: DateTime<Utc>,
    pub auto_approve_no_change: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReviewOutput {
    pub review_id: Uuid,
    pub total_items: u32,
    pub approved_items: u32,
    pub revoked_items: u32,
    pub pending_items: u32,
    pub completion_rate: f32,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReviewScope {
    AllUsers,
    Department(String),
    Role(Uuid),
    HighPrivilegeUsers,
}

// ============================================================================
// SECURITY INCIDENT WORKFLOW TYPES
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncidentInput {
    pub incident_id: Uuid,
    pub incident_type: IncidentType,
    pub affected_user_id: Option<UserId>,
    pub tenant_id: TenantId,
    pub severity: SecuritySeverity,
    pub detected_at: DateTime<Utc>,
    pub detection_source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityIncidentOutput {
    pub response_id: Uuid,
    pub actions_taken: Vec<SecurityAction>,
    pub access_revoked: bool,
    pub notifications_sent: Vec<NotificationTarget>,
    pub investigation_initiated: bool,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IncidentType {
    SuspiciousLogin,
    PrivilegeEscalation,
    UnauthorizedAccess,
    DataExfiltration,
    MaliciousActivity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityAction {
    SessionTermination,
    AccountLockout,
    PermissionRevocation,
    AlertSecurityTeam,
    InitiateInvestigation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationTarget {
    SecurityTeam,
    UserManager,
    ComplianceOfficer,
    AffectedUser,
}

// ============================================================================
// SIMPLE OPERATION TYPES (Direct API Calls)
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckRequest {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub resource: String,
    pub action: String,
    pub context: Option<serde_json::Value>, // Additional context for conditional permissions
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCheckResponse {
    pub allowed: bool,
    pub reason: Option<String>,
    pub matched_permissions: Vec<Permission>,
    pub check_duration_ms: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRolesResponse {
    pub user_id: UserId,
    pub roles: Vec<Role>,
    pub effective_permissions: Vec<Permission>,
    pub tenant_id: TenantId,
}

// ============================================================================
// ERROR TYPES
// ============================================================================

#[derive(Debug, thiserror::Error)]
pub enum RbacError {
    #[error("Permission denied")]
    PermissionDenied,
    #[error("Role not found: {0}")]
    RoleNotFound(Uuid),
    #[error("User not found: {0}")]
    UserNotFound(UserId),
    #[error("Invalid role hierarchy")]
    InvalidRoleHierarchy,
    #[error("Workflow execution failed: {0}")]
    WorkflowError(String),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("Temporal error: {0}")]
    Temporal(String),
}
