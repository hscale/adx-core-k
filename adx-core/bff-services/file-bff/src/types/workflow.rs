use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRequest<T> {
    pub workflow_type: String,
    pub input: T,
    pub options: WorkflowOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOptions {
    pub synchronous: bool,
    pub timeout_seconds: Option<u64>,
    pub retry_policy: Option<RetryPolicy>,
}

impl Default for WorkflowOptions {
    fn default() -> Self {
        Self {
            synchronous: false,
            timeout_seconds: Some(300), // 5 minutes default
            retry_policy: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_interval_seconds: u64,
    pub backoff_coefficient: f64,
    pub maximum_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowResponse<T> {
    Synchronous {
        data: T,
        execution_time_ms: u64,
        workflow_id: String,
    },
    Asynchronous {
        operation_id: String,
        status_url: String,
        stream_url: Option<String>,
        estimated_duration_seconds: Option<u64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub operation_id: String,
    pub workflow_id: String,
    pub status: WorkflowExecutionStatus,
    pub progress: Option<WorkflowProgress>,
    pub result: Option<serde_json::Value>,
    pub error: Option<WorkflowError>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowExecutionStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub message: Option<String>,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    pub error_type: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub retry_after: Option<u64>,
}

// File-specific workflow types
#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadWorkflowInput {
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub storage_provider: Option<String>,
    pub virus_scan: bool,
    pub generate_thumbnails: bool,
    pub extract_metadata: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileProcessingWorkflowInput {
    pub file_id: Uuid,
    pub processing_options: FileProcessingOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileProcessingOptions {
    pub generate_thumbnails: bool,
    pub extract_metadata: bool,
    pub virus_scan: bool,
    pub ocr_processing: bool,
    pub image_optimization: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMigrationWorkflowInput {
    pub file_ids: Vec<Uuid>,
    pub source_provider: String,
    pub target_provider: String,
    pub preserve_urls: bool,
    pub verify_integrity: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BulkFileOperationWorkflowInput {
    pub operation_type: BulkOperationType,
    pub file_ids: Vec<Uuid>,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum BulkOperationType {
    Delete,
    Move,
    Copy,
    UpdatePermissions,
    UpdateTags,
    UpdateMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileCleanupWorkflowInput {
    pub tenant_id: String,
    pub cleanup_rules: Vec<CleanupRule>,
    pub dry_run: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CleanupRule {
    pub rule_type: CleanupRuleType,
    pub criteria: serde_json::Value,
    pub action: CleanupAction,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CleanupRuleType {
    Age,
    Size,
    Usage,
    Orphaned,
    Duplicate,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CleanupAction {
    Delete,
    Archive,
    Compress,
    Move,
}