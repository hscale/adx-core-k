use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowMetrics {
    pub tenant_id: String,
    pub time_range: String,
    pub granularity: String,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub average_duration_ms: u64,
    pub median_duration_ms: u64,
    pub p95_duration_ms: u64,
    pub p99_duration_ms: u64,
    pub throughput_per_hour: f64,
    pub error_rate: f64,
    pub retry_rate: f64,
    pub workflow_types: HashMap<String, serde_json::Value>,
    pub time_series: Vec<serde_json::Value>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowTypeMetrics {
    pub workflow_type: String,
    pub metrics: WorkflowMetrics,
    pub recent_executions: Vec<RecentExecution>,
    pub common_errors: Vec<ErrorSummary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RecentExecution {
    pub workflow_id: String,
    pub run_id: String,
    pub status: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorSummary {
    pub error_type: String,
    pub error_message: String,
    pub count: u64,
    pub first_occurrence: DateTime<Utc>,
    pub last_occurrence: DateTime<Utc>,
    pub sample_workflow_ids: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: String,
    pub timestamp: DateTime<Utc>,
    pub components: HashMap<String, serde_json::Value>,
    pub metrics: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemporalClusterHealth {
    pub status: HealthStatus,
    pub frontend_nodes: u32,
    pub history_nodes: u32,
    pub matching_nodes: u32,
    pub worker_nodes: u32,
    pub cluster_version: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkerHealth {
    pub worker_id: String,
    pub task_queue: String,
    pub status: HealthStatus,
    pub active_pollers: u32,
    pub tasks_per_second: f64,
    pub last_heartbeat: DateTime<Utc>,
    pub uptime_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskQueueHealth {
    pub name: String,
    pub status: HealthStatus,
    pub backlog_size: u64,
    pub rate_per_second: f64,
    pub active_pollers: u32,
    pub partition_count: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseHealth {
    pub status: HealthStatus,
    pub connection_pool_size: u32,
    pub active_connections: u32,
    pub avg_query_time_ms: f64,
    pub slow_queries: u64,
    pub last_backup: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheHealth {
    pub status: HealthStatus,
    pub hit_rate: f64,
    pub memory_usage_mb: u64,
    pub max_memory_mb: u64,
    pub connected_clients: u32,
    pub operations_per_second: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertRule {
    pub id: String,
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub is_active: bool,
    pub notification_channels: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlertCondition {
    pub metric: String,
    pub operator: String, // "gt", "lt", "eq", "gte", "lte"
    pub threshold: f64,
    pub duration_minutes: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub rule_id: String,
    pub rule_name: String,
    pub severity: AlertSeverity,
    pub status: AlertStatus,
    pub message: String,
    pub details: HashMap<String, serde_json::Value>,
    pub triggered_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub acknowledged_at: Option<DateTime<Utc>>,
    pub acknowledged_by: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum AlertStatus {
    Triggered,
    Acknowledged,
    Resolved,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsQuery {
    pub metric_name: String,
    pub time_range: TimeRange,
    pub granularity: String, // "1m", "5m", "1h", "1d"
    pub filters: Option<HashMap<String, String>>,
    pub aggregation: Option<String>, // "avg", "sum", "max", "min", "count"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub metric_name: String,
    pub data_points: Vec<DataPoint>,
    pub aggregation: Option<String>,
    pub time_range: TimeRange,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub widgets: Vec<DashboardWidget>,
    pub layout: DashboardLayout,
    pub refresh_interval_seconds: u32,
    pub is_public: bool,
    pub created_by: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardWidget {
    pub id: String,
    pub widget_type: String, // "chart", "metric", "table", "alert_list"
    pub title: String,
    pub config: serde_json::Value,
    pub position: WidgetPosition,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub columns: u32,
    pub row_height: u32,
    pub margin: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateAlertRuleRequest {
    pub name: String,
    pub description: String,
    pub condition: AlertCondition,
    pub severity: AlertSeverity,
    pub notification_channels: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAlertRuleRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub condition: Option<AlertCondition>,
    pub severity: Option<AlertSeverity>,
    pub is_active: Option<bool>,
    pub notification_channels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AcknowledgeAlertRequest {
    pub acknowledged_by: String,
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowQueryRequest {
    pub query_type: String,
    pub query_args: Option<serde_json::Value>,
}

impl Default for TimeRange {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            start: now - chrono::Duration::hours(24),
            end: now,
        }
    }
}