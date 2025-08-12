use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type WorkflowServiceResult<T> = Result<T, WorkflowServiceError>;

#[derive(Error, Debug)]
pub enum WorkflowServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Temporal error: {0}")]
    Temporal(String),

    #[error("Workflow execution error: {0}")]
    WorkflowExecution(String),

    #[error("Activity execution error: {0}")]
    ActivityExecution(String),

    #[error("Service communication error: {service}: {message}")]
    ServiceCommunication { service: String, message: String },

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Tenant context error: {0}")]
    TenantContext(String),

    #[error("Cross-service coordination error: {0}")]
    CrossServiceCoordination(String),

    #[error("Data migration error: {0}")]
    DataMigration(String),

    #[error("Compliance workflow error: {0}")]
    ComplianceWorkflow(String),

    #[error("Bulk operation error: {0}")]
    BulkOperation(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Invalid operation: {0}")]
    InvalidOperation(String),

    #[error("Invalid version format: {0}")]
    InvalidVersion(String),

    #[error("Invalid template: {0}")]
    InvalidTemplate(String),

    #[error("Template in use: {0}")]
    TemplateInUse(String),

    #[error("Missing parameter: {0}")]
    MissingParameter(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Monitoring error: {0}")]
    Monitoring(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for WorkflowServiceError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            WorkflowServiceError::Validation(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            WorkflowServiceError::Authorization(_) => (StatusCode::FORBIDDEN, self.to_string()),
            WorkflowServiceError::TenantContext(_) => (StatusCode::BAD_REQUEST, self.to_string()),
            WorkflowServiceError::ServiceCommunication { .. } => {
                (StatusCode::BAD_GATEWAY, self.to_string())
            }
            WorkflowServiceError::Temporal(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Workflow engine error".to_string())
            }
            _ => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(json!({
            "error": {
                "message": error_message,
                "type": format!("{:?}", self).split('(').next().unwrap_or("Unknown"),
            }
        }));

        (status, body).into_response()
    }
}