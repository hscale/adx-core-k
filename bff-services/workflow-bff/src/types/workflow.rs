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
    pub parent_workflow_id: Option<String>,
    pub child_workflows: Vec<String>,
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
    pub retry_count: u32,
    pub last_retry_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub message: Option<String>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub activity_progress: Vec<ActivityProgress>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityProgress {
    pub activity_id: String,
    pub activity_type: String,
    pub status: ActivityStatus,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub retry_count: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityStatus {
    Scheduled,
    Started,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
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
    pub cron_schedule: Option<String>,
    pub memo: Option<HashMap<String, String>>,
    pub search_attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartWorkflowResponse {
    pub workflow_id: String,
    pub run_id: String,
    pub status_url: String,
    pub stream_url: Option<String>,
    pub estimated_duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowListRequest {
    pub workflow_type: Option<String>,
    pub status: Option<WorkflowStatus>,
    pub start_time_from: Option<DateTime<Utc>>,
    pub start_time_to: Option<DateTime<Utc>>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub search_attributes: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowListResponse {
    pub workflows: Vec<WorkflowExecution>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
    pub aggregations: Option<WorkflowAggregations>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowAggregations {
    pub status_counts: HashMap<String, u64>,
    pub type_counts: HashMap<String, u64>,
    pub avg_execution_time: Option<f64>,
    pub success_rate: f64,
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
    pub task_id: Option<u64>,
    pub activity_id: Option<String>,
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

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub workflow_type: String,
    pub default_input: serde_json::Value,
    pub parameters: Vec<WorkflowParameter>,
    pub tags: Vec<String>,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowParameter {
    pub name: String,
    pub parameter_type: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation_rules: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowSchedule {
    pub id: String,
    pub workflow_type: String,
    pub cron_expression: String,
    pub input: serde_json::Value,
    pub is_active: bool,
    pub next_run: Option<DateTime<Utc>>,
    pub last_run: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWorkflowScheduleRequest {
    pub workflow_type: String,
    pub cron_expression: String,
    pub input: serde_json::Value,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateWorkflowScheduleRequest {
    pub cron_expression: Option<String>,
    pub input: Option<serde_json::Value>,
    pub is_active: Option<bool>,
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
            sort_by: Some("start_time".to_string()),
            sort_order: Some("desc".to_string()),
            search_attributes: None,
        }
    }
}