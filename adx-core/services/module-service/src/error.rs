use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModuleServiceError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Database error: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("Redis error: {0}")]
    RedisError(#[from] redis::RedisError),

    #[error("HTTP client error: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Module already exists: {0}")]
    ModuleAlreadyExists(String),

    #[error("Module validation failed: {0}")]
    ModuleValidationError(String),

    #[error("Dependency resolution failed: {0}")]
    DependencyResolutionError(String),

    #[error("Security scan failed: {0}")]
    SecurityScanError(String),

    #[error("Sandbox violation: {0}")]
    SandboxViolation(String),

    #[error("Payment processing failed: {0}")]
    PaymentError(String),

    #[error("Marketplace error: {0}")]
    MarketplaceError(String),

    #[error("Workflow error: {0}")]
    WorkflowError(String),

    #[error("Activity error: {0}")]
    ActivityError(String),

    #[error("Temporal error: {0}")]
    TemporalError(String),

    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    #[error("Authorization error: {0}")]
    AuthorizationError(String),

    #[error("Tenant error: {0}")]
    TenantError(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),

    #[error("Internal server error: {0}")]
    InternalError(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub retry_after: Option<u64>,
    pub documentation_url: Option<String>,
}

impl IntoResponse for ModuleServiceError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details) = match &self {
            ModuleServiceError::ConfigError(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
                msg.clone(),
                None,
            ),
            ModuleServiceError::DatabaseError(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Database operation failed".to_string(),
                Some(serde_json::json!({ "database_error": err.to_string() })),
            ),
            ModuleServiceError::ModuleNotFound(id) => (
                StatusCode::NOT_FOUND,
                "MODULE_NOT_FOUND",
                format!("Module '{}' not found", id),
                None,
            ),
            ModuleServiceError::ModuleAlreadyExists(id) => (
                StatusCode::CONFLICT,
                "MODULE_ALREADY_EXISTS",
                format!("Module '{}' already exists", id),
                None,
            ),
            ModuleServiceError::ModuleValidationError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "MODULE_VALIDATION_ERROR",
                msg.clone(),
                None,
            ),
            ModuleServiceError::DependencyResolutionError(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "DEPENDENCY_RESOLUTION_ERROR",
                msg.clone(),
                None,
            ),
            ModuleServiceError::SecurityScanError(msg) => (
                StatusCode::FORBIDDEN,
                "SECURITY_SCAN_FAILED",
                msg.clone(),
                Some(serde_json::json!({
                    "documentation_url": "https://docs.adxcore.com/modules/security"
                })),
            ),
            ModuleServiceError::SandboxViolation(msg) => (
                StatusCode::FORBIDDEN,
                "SANDBOX_VIOLATION",
                msg.clone(),
                None,
            ),
            ModuleServiceError::PaymentError(msg) => (
                StatusCode::PAYMENT_REQUIRED,
                "PAYMENT_ERROR",
                msg.clone(),
                None,
            ),
            ModuleServiceError::AuthenticationError(msg) => (
                StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_ERROR",
                msg.clone(),
                None,
            ),
            ModuleServiceError::AuthorizationError(msg) => (
                StatusCode::FORBIDDEN,
                "AUTHORIZATION_ERROR",
                msg.clone(),
                None,
            ),
            ModuleServiceError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Rate limit exceeded".to_string(),
                Some(serde_json::json!({ "retry_after": 60 })),
            ),
            ModuleServiceError::QuotaExceeded(msg) => (
                StatusCode::FORBIDDEN,
                "QUOTA_EXCEEDED",
                msg.clone(),
                None,
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An internal error occurred".to_string(),
                None,
            ),
        };

        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: error_code.to_string(),
                message,
                details,
                retry_after: if matches!(self, ModuleServiceError::RateLimitExceeded) {
                    Some(60)
                } else {
                    None
                },
                documentation_url: Some("https://docs.adxcore.com/api/modules".to_string()),
            },
            request_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

// Activity-specific errors for Temporal workflows
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum ActivityError {
    #[error("Module validation failed: {0}")]
    ValidationFailed(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Installation failed: {0}")]
    InstallationFailed(String),

    #[error("Activation failed: {0}")]
    ActivationFailed(String),

    #[error("Deactivation failed: {0}")]
    DeactivationFailed(String),

    #[error("Uninstallation failed: {0}")]
    UninstallationFailed(String),

    #[error("Security scan failed: {0}")]
    SecurityScanFailed(String),

    #[error("Payment processing failed: {0}")]
    PaymentFailed(String),

    #[error("Database operation failed: {0}")]
    DatabaseFailed(String),

    #[error("External service error: {0}")]
    ExternalServiceError(String),

    #[error("Timeout error: {0}")]
    TimeoutError(String),

    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
}

impl ActivityError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ActivityError::DownloadFailed(_)
                | ActivityError::DatabaseFailed(_)
                | ActivityError::ExternalServiceError(_)
                | ActivityError::TimeoutError(_)
        )
    }
}

// Workflow-specific errors
#[derive(Error, Debug, Serialize, Deserialize)]
pub enum WorkflowError {
    #[error("Module installation workflow failed: {0}")]
    InstallationWorkflowFailed(String),

    #[error("Module update workflow failed: {0}")]
    UpdateWorkflowFailed(String),

    #[error("Module uninstallation workflow failed: {0}")]
    UninstallationWorkflowFailed(String),

    #[error("Marketplace sync workflow failed: {0}")]
    MarketplaceSyncFailed(String),

    #[error("Security scan workflow failed: {0}")]
    SecurityScanWorkflowFailed(String),

    #[error("Payment workflow failed: {0}")]
    PaymentWorkflowFailed(String),

    #[error("Rollback failed: {0}")]
    RollbackFailed(String),

    #[error("Compensation failed: {0}")]
    CompensationFailed(String),

    #[error("Workflow timeout: {0}")]
    WorkflowTimeout(String),

    #[error("Workflow cancelled: {0}")]
    WorkflowCancelled(String),
}