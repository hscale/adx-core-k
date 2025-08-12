use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum ApiGatewayError {
    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Invalid authentication token: {message}")]
    InvalidToken { message: String },

    #[error("Insufficient permissions: {required_permission}")]
    InsufficientPermissions { required_permission: String },

    #[error("Rate limit exceeded: {limit_type}")]
    RateLimitExceeded { limit_type: String, retry_after: u64 },

    #[error("Tenant not found: {tenant_id}")]
    TenantNotFound { tenant_id: String },

    #[error("Tenant access denied: {reason}")]
    TenantAccessDenied { reason: String },

    #[error("Service unavailable: {service}")]
    ServiceUnavailable { service: String },

    #[error("Service timeout: {service}")]
    ServiceTimeout { service: String },

    #[error("Workflow execution failed: {workflow_id}")]
    WorkflowExecutionFailed { workflow_id: String, error: String },

    #[error("Workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: String },

    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },

    #[error("Validation failed")]
    ValidationFailed { errors: Vec<ValidationError> },

    #[error("Internal server error: {message}")]
    InternalError { message: String },

    #[error("Temporal client error: {message}")]
    TemporalError { message: String },

    #[error("Redis error: {message}")]
    RedisError { message: String },

    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub rejected_value: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ErrorDetails,
    pub request_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub validation_errors: Option<Vec<ValidationError>>,
    pub retry_after: Option<u64>,
    pub documentation_url: Option<String>,
}

impl ApiGatewayError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiGatewayError::AuthenticationRequired => StatusCode::UNAUTHORIZED,
            ApiGatewayError::InvalidToken { .. } => StatusCode::UNAUTHORIZED,
            ApiGatewayError::InsufficientPermissions { .. } => StatusCode::FORBIDDEN,
            ApiGatewayError::RateLimitExceeded { .. } => StatusCode::TOO_MANY_REQUESTS,
            ApiGatewayError::TenantNotFound { .. } => StatusCode::NOT_FOUND,
            ApiGatewayError::TenantAccessDenied { .. } => StatusCode::FORBIDDEN,
            ApiGatewayError::ServiceUnavailable { .. } => StatusCode::SERVICE_UNAVAILABLE,
            ApiGatewayError::ServiceTimeout { .. } => StatusCode::GATEWAY_TIMEOUT,
            ApiGatewayError::WorkflowNotFound { .. } => StatusCode::NOT_FOUND,
            ApiGatewayError::WorkflowExecutionFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiGatewayError::InvalidRequest { .. } => StatusCode::BAD_REQUEST,
            ApiGatewayError::ValidationFailed { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            ApiGatewayError::InternalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiGatewayError::TemporalError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiGatewayError::RedisError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
            ApiGatewayError::ConfigurationError { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn error_code(&self) -> &'static str {
        match self {
            ApiGatewayError::AuthenticationRequired => "AUTHENTICATION_REQUIRED",
            ApiGatewayError::InvalidToken { .. } => "INVALID_TOKEN",
            ApiGatewayError::InsufficientPermissions { .. } => "INSUFFICIENT_PERMISSIONS",
            ApiGatewayError::RateLimitExceeded { .. } => "RATE_LIMIT_EXCEEDED",
            ApiGatewayError::TenantNotFound { .. } => "TENANT_NOT_FOUND",
            ApiGatewayError::TenantAccessDenied { .. } => "TENANT_ACCESS_DENIED",
            ApiGatewayError::ServiceUnavailable { .. } => "SERVICE_UNAVAILABLE",
            ApiGatewayError::ServiceTimeout { .. } => "SERVICE_TIMEOUT",
            ApiGatewayError::WorkflowNotFound { .. } => "WORKFLOW_NOT_FOUND",
            ApiGatewayError::WorkflowExecutionFailed { .. } => "WORKFLOW_EXECUTION_FAILED",
            ApiGatewayError::InvalidRequest { .. } => "INVALID_REQUEST",
            ApiGatewayError::ValidationFailed { .. } => "VALIDATION_FAILED",
            ApiGatewayError::InternalError { .. } => "INTERNAL_ERROR",
            ApiGatewayError::TemporalError { .. } => "TEMPORAL_ERROR",
            ApiGatewayError::RedisError { .. } => "REDIS_ERROR",
            ApiGatewayError::ConfigurationError { .. } => "CONFIGURATION_ERROR",
        }
    }

    pub fn to_response(&self, request_id: Option<String>) -> ApiErrorResponse {
        let request_id = request_id.unwrap_or_else(|| Uuid::new_v4().to_string());
        
        let mut details = ErrorDetails {
            code: self.error_code().to_string(),
            message: self.to_string(),
            details: None,
            validation_errors: None,
            retry_after: None,
            documentation_url: Some("https://docs.adxcore.com/api/errors".to_string()),
        };

        // Add specific details based on error type
        match self {
            ApiGatewayError::RateLimitExceeded { limit_type, retry_after } => {
                details.retry_after = Some(*retry_after);
                details.details = Some(serde_json::json!({
                    "limit_type": limit_type
                }));
            }
            ApiGatewayError::ValidationFailed { errors } => {
                details.validation_errors = Some(errors.clone());
            }
            ApiGatewayError::InsufficientPermissions { required_permission } => {
                details.details = Some(serde_json::json!({
                    "required_permission": required_permission
                }));
            }
            ApiGatewayError::WorkflowExecutionFailed { workflow_id, error } => {
                details.details = Some(serde_json::json!({
                    "workflow_id": workflow_id,
                    "error": error
                }));
            }
            _ => {}
        }

        ApiErrorResponse {
            error: details,
            request_id,
            timestamp: chrono::Utc::now(),
        }
    }
}

impl IntoResponse for ApiGatewayError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let error_response = self.to_response(None);
        
        let mut response = Json(error_response).into_response();
        *response.status_mut() = status;
        
        // Add retry-after header for rate limiting
        if let ApiGatewayError::RateLimitExceeded { retry_after, .. } = self {
            response.headers_mut().insert(
                "Retry-After",
                retry_after.to_string().parse().unwrap(),
            );
        }
        
        response
    }
}

// Conversion from shared library errors
impl From<adx_shared::Error> for ApiGatewayError {
    fn from(error: adx_shared::Error) -> Self {
        match error {
            adx_shared::Error::Temporal(msg) => ApiGatewayError::TemporalError { message: msg },
            adx_shared::Error::Database(e) => ApiGatewayError::InternalError { message: e.to_string() },
            adx_shared::Error::Validation(msg) => ApiGatewayError::InvalidRequest { message: msg },
            adx_shared::Error::Authentication(msg) => ApiGatewayError::InvalidToken { message: msg },
            adx_shared::Error::Authorization(msg) => ApiGatewayError::InsufficientPermissions { 
                required_permission: msg 
            },
            adx_shared::Error::NotFound(msg) => ApiGatewayError::InvalidRequest { message: msg },
            adx_shared::Error::Configuration(msg) => ApiGatewayError::ConfigurationError { message: msg },
            adx_shared::Error::Http(e) => ApiGatewayError::ServiceUnavailable { 
                service: e.to_string() 
            },
            adx_shared::Error::Redis(e) => ApiGatewayError::RedisError { 
                message: e.to_string() 
            },
            _ => ApiGatewayError::InternalError { 
                message: error.to_string() 
            },
        }
    }
}

impl From<anyhow::Error> for ApiGatewayError {
    fn from(error: anyhow::Error) -> Self {
        ApiGatewayError::InternalError {
            message: error.to_string(),
        }
    }
}

impl From<reqwest::Error> for ApiGatewayError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            ApiGatewayError::ServiceTimeout {
                service: "unknown".to_string(),
            }
        } else if error.is_connect() {
            ApiGatewayError::ServiceUnavailable {
                service: "unknown".to_string(),
            }
        } else {
            ApiGatewayError::InternalError {
                message: error.to_string(),
            }
        }
    }
}

impl From<redis::RedisError> for ApiGatewayError {
    fn from(error: redis::RedisError) -> Self {
        ApiGatewayError::RedisError {
            message: error.to_string(),
        }
    }
}

pub type ApiResult<T> = Result<T, ApiGatewayError>;