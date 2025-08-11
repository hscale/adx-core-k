use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};


// Common ID types
pub type UserId = String;
pub type TenantId = String;
pub type SessionId = String;
pub type WorkflowId = String;
pub type ActivityId = String;

// Subscription tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SubscriptionTier {
    Free,
    Professional,
    Enterprise,
    Custom,
}

impl Default for SubscriptionTier {
    fn default() -> Self {
        Self::Free
    }
}

// Tenant isolation levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TenantIsolationLevel {
    Schema,    // Separate schema per tenant
    Database,  // Separate database per tenant
    Row,       // Row-level security with tenant_id column
}

impl Default for TenantIsolationLevel {
    fn default() -> Self {
        Self::Schema
    }
}

// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

// Workflow progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub message: Option<String>,
}

// API response wrapper for workflows
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowApiResponse<T> {
    #[serde(rename = "synchronous")]
    Synchronous {
        data: T,
        execution_time_ms: u64,
        workflow_id: String,
    },
    #[serde(rename = "asynchronous")]
    Asynchronous {
        operation_id: String,
        status_url: String,
        stream_url: Option<String>,
        estimated_duration_seconds: Option<u64>,
    },
}

// Workflow status response
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatusResponse {
    pub operation_id: String,
    pub status: WorkflowStatus,
    pub progress: Option<WorkflowProgress>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

// Tenant quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    pub max_users: Option<u32>,
    pub max_storage_gb: Option<u32>,
    pub max_api_calls_per_hour: Option<u32>,
    pub max_workflows_per_hour: Option<u32>,
}

impl Default for TenantQuotas {
    fn default() -> Self {
        Self {
            max_users: Some(10),
            max_storage_gb: Some(5),
            max_api_calls_per_hour: Some(1000),
            max_workflows_per_hour: Some(100),
        }
    }
}

// User quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserQuotas {
    pub api_calls_per_hour: u32,
    pub storage_gb: u32,
    pub concurrent_workflows: u32,
    pub file_upload_size_mb: u32,
}

impl Default for UserQuotas {
    fn default() -> Self {
        Self {
            api_calls_per_hour: 100,
            storage_gb: 1,
            concurrent_workflows: 5,
            file_upload_size_mb: 50,
        }
    }
}

// Health check status
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: String,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub checks: std::collections::HashMap<String, HealthCheck>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheck {
    pub status: String,
    pub message: Option<String>,
    pub duration_ms: u64,
}

// Pagination
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub page: u32,
    pub limit: u32,
    pub total: u64,
    pub total_pages: u32,
    pub has_next: bool,
    pub has_prev: bool,
}