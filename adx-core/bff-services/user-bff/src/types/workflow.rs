use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub workflow_id: String,
    pub run_id: String,
    pub workflow_type: String,
    pub status: WorkflowStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<u64>,
    pub input: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<WorkflowError>,
    pub progress: Option<WorkflowProgress>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Terminated,
    TimedOut,
    ContinuedAsNew,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    pub message: String,
    pub error_type: String,
    pub stack_trace: Option<String>,
    pub failure_info: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub message: Option<String>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartWorkflowRequest {
    pub workflow_type: String,
    pub workflow_id: Option<String>,
    pub input: serde_json::Value,
    pub task_queue: Option<String>,
    pub workflow_execution_timeout: Option<u64>,
    pub workflow_run_timeout: Option<u64>,
    pub workflow_task_timeout: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartWorkflowResponse {
    pub workflow_id: String,
    pub run_id: String,
    pub status_url: String,
    pub stream_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowListRequest {
    pub workflow_type: Option<String>,
    pub status: Option<WorkflowStatus>,
    pub start_time_from: Option<DateTime<Utc>>,
    pub start_time_to: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowListResponse {
    pub workflows: Vec<WorkflowExecution>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowHistory {
    pub events: Vec<WorkflowEvent>,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub event_id: u64,
    pub event_time: DateTime<Utc>,
    pub event_type: String,
    pub attributes: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CancelWorkflowRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TerminateWorkflowRequest {
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowQuery {
    pub query_type: String,
    pub query_args: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowQueryResponse {
    pub query_type: String,
    pub query_result: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowSignal {
    pub signal_name: String,
    pub signal_input: Option<serde_json::Value>,
}

// User-specific workflow types
#[derive(Debug, Serialize, Deserialize)]
pub struct UserOnboardingWorkflow {
    pub user_id: String,
    pub tenant_id: String,
    pub onboarding_steps: Vec<String>,
    pub welcome_email: bool,
    pub setup_preferences: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserProfileSyncWorkflow {
    pub user_id: String,
    pub sync_targets: Vec<String>, // Services to sync profile to
    pub profile_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserPreferenceMigrationWorkflow {
    pub user_id: String,
    pub from_version: String,
    pub to_version: String,
    pub preferences: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDataExportWorkflow {
    pub user_id: String,
    pub export_format: String, // "json", "csv", "xml"
    pub include_activity: bool,
    pub include_files: bool,
    pub encryption_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDeactivationWorkflow {
    pub user_id: String,
    pub deactivation_reason: Option<String>,
    pub retain_data: bool,
    pub notify_admin: bool,
}

impl Default for WorkflowListRequest {
    fn default() -> Self {
        Self {
            workflow_type: None,
            status: None,
            start_time_from: None,
            start_time_to: None,
            page: Some(1),
            per_page: Some(20),
        }
    }
}