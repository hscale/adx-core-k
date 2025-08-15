use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Temporal error: {0}")]
    Temporal(String),

    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Audit log error: {0}")]
    AuditLog(String),

    #[error("Compliance error: {0}")]
    Compliance(String),

    #[error("GDPR request error: {0}")]
    GdprRequest(String),

    #[error("Data retention error: {0}")]
    DataRetention(String),

    #[error("Security scan error: {0}")]
    SecurityScan(String),

    #[error("Zero trust policy error: {0}")]
    ZeroTrust(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
    pub request_id: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

impl IntoResponse for SecurityError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details) = match &self {
            SecurityError::Database(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Database operation failed",
                Some(serde_json::json!({ "error": e.to_string() })),
            ),
            SecurityError::Temporal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "TEMPORAL_ERROR",
                "Workflow operation failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::Config(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CONFIG_ERROR",
                "Configuration error",
                Some(serde_json::json!({ "error": e.to_string() })),
            ),
            SecurityError::Serialization(e) => (
                StatusCode::BAD_REQUEST,
                "SERIALIZATION_ERROR",
                "Data serialization failed",
                Some(serde_json::json!({ "error": e.to_string() })),
            ),
            SecurityError::HttpClient(e) => (
                StatusCode::BAD_GATEWAY,
                "HTTP_CLIENT_ERROR",
                "External service request failed",
                Some(serde_json::json!({ "error": e.to_string() })),
            ),
            SecurityError::Io(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "IO_ERROR",
                "File system operation failed",
                Some(serde_json::json!({ "error": e.to_string() })),
            ),
            SecurityError::Encryption(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ENCRYPTION_ERROR",
                "Encryption operation failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::AuditLog(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "AUDIT_LOG_ERROR",
                "Audit logging failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::Compliance(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "COMPLIANCE_ERROR",
                "Compliance operation failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::GdprRequest(e) => (
                StatusCode::BAD_REQUEST,
                "GDPR_REQUEST_ERROR",
                "GDPR request processing failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::DataRetention(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATA_RETENTION_ERROR",
                "Data retention operation failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::SecurityScan(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "SECURITY_SCAN_ERROR",
                "Security scan failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::ZeroTrust(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "ZERO_TRUST_ERROR",
                "Zero trust policy operation failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::Validation(e) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                "Request validation failed",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::Authorization(e) => (
                StatusCode::FORBIDDEN,
                "AUTHORIZATION_ERROR",
                "Access denied",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::NotFound(e) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                "Resource not found",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::Conflict(e) => (
                StatusCode::CONFLICT,
                "CONFLICT",
                "Resource conflict",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::RateLimitExceeded => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                "Rate limit exceeded",
                None,
            ),
            SecurityError::ServiceUnavailable(e) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "SERVICE_UNAVAILABLE",
                "Service temporarily unavailable",
                Some(serde_json::json!({ "error": e })),
            ),
            SecurityError::Internal(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "Internal server error",
                Some(serde_json::json!({ "error": e })),
            ),
        };

        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: error_code.to_string(),
                message: message.to_string(),
                details,
            },
            request_id: None, // This would be populated by middleware
            timestamp: chrono::Utc::now(),
        };

        (status, Json(error_response)).into_response()
    }
}

pub type SecurityResult<T> = Result<T, SecurityError>;