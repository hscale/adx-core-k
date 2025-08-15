use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tracing::error;

use crate::types::ApiError;

pub async fn handle_error() -> Response {
    let error = ApiError {
        error: "NOT_FOUND".to_string(),
        message: "The requested resource was not found".to_string(),
        details: None,
    };

    (StatusCode::NOT_FOUND, Json(error)).into_response()
}

// Custom error type for the BFF service
#[derive(Debug, thiserror::Error)]
pub enum BffError {
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    #[error("Tenant validation failed: {0}")]
    TenantValidation(String),
    
    #[error("API client error: {0}")]
    ApiClient(#[from] anyhow::Error),
    
    #[error("Redis error: {0}")]
    Redis(String),
    
    #[error("Temporal client error: {0}")]
    Temporal(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for BffError {
    fn into_response(self) -> Response {
        let (status, error_code, message) = match &self {
            BffError::Authentication(_) => (StatusCode::UNAUTHORIZED, "AUTHENTICATION_FAILED", self.to_string()),
            BffError::Authorization(_) => (StatusCode::FORBIDDEN, "AUTHORIZATION_FAILED", self.to_string()),
            BffError::TenantValidation(_) => (StatusCode::FORBIDDEN, "TENANT_VALIDATION_FAILED", self.to_string()),
            BffError::Validation(_) => (StatusCode::BAD_REQUEST, "VALIDATION_ERROR", self.to_string()),
            BffError::NotFound(_) => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            BffError::Conflict(_) => (StatusCode::CONFLICT, "CONFLICT", self.to_string()),
            BffError::RateLimit(_) => (StatusCode::TOO_MANY_REQUESTS, "RATE_LIMIT_EXCEEDED", self.to_string()),
            BffError::ApiClient(_) => (StatusCode::BAD_GATEWAY, "UPSTREAM_ERROR", "Upstream service error".to_string()),
            BffError::Redis(_) => (StatusCode::INTERNAL_SERVER_ERROR, "CACHE_ERROR", "Cache service error".to_string()),
            BffError::Temporal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "WORKFLOW_ERROR", "Workflow service error".to_string()),
            BffError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_ERROR", "Internal server error".to_string()),
        };

        error!("BFF Error: {} - {}", error_code, message);

        let error_response = ApiError {
            error: error_code.to_string(),
            message,
            details: None,
        };

        (status, Json(error_response)).into_response()
    }
}

// Result type alias for convenience
pub type BffResult<T> = Result<T, BffError>;

// Helper functions for creating specific errors
impl BffError {
    pub fn authentication<S: Into<String>>(msg: S) -> Self {
        BffError::Authentication(msg.into())
    }

    pub fn authorization<S: Into<String>>(msg: S) -> Self {
        BffError::Authorization(msg.into())
    }

    pub fn tenant_validation<S: Into<String>>(msg: S) -> Self {
        BffError::TenantValidation(msg.into())
    }

    pub fn validation<S: Into<String>>(msg: S) -> Self {
        BffError::Validation(msg.into())
    }

    pub fn not_found<S: Into<String>>(msg: S) -> Self {
        BffError::NotFound(msg.into())
    }

    pub fn conflict<S: Into<String>>(msg: S) -> Self {
        BffError::Conflict(msg.into())
    }

    pub fn rate_limit<S: Into<String>>(msg: S) -> Self {
        BffError::RateLimit(msg.into())
    }

    pub fn redis<S: Into<String>>(msg: S) -> Self {
        BffError::Redis(msg.into())
    }

    pub fn temporal<S: Into<String>>(msg: S) -> Self {
        BffError::Temporal(msg.into())
    }

    pub fn internal<S: Into<String>>(msg: S) -> Self {
        BffError::Internal(msg.into())
    }
}

// Convert common error types to BffError
impl From<redis::RedisError> for BffError {
    fn from(err: redis::RedisError) -> Self {
        BffError::Redis(err.to_string())
    }
}

impl From<serde_json::Error> for BffError {
    fn from(err: serde_json::Error) -> Self {
        BffError::Validation(format!("JSON serialization error: {}", err))
    }
}

impl From<reqwest::Error> for BffError {
    fn from(err: reqwest::Error) -> Self {
        BffError::ApiClient(err.into())
    }
}

// Validation error details
#[derive(Debug, serde::Serialize)]
pub struct ValidationErrorDetail {
    pub field: String,
    pub code: String,
    pub message: String,
    pub rejected_value: Option<serde_json::Value>,
}

impl BffError {
    pub fn validation_with_details<S: Into<String>>(
        msg: S,
        details: Vec<ValidationErrorDetail>,
    ) -> Self {
        let error_details = json!({
            "validation_errors": details
        });
        
        // For now, we'll just include the message. In a more sophisticated implementation,
        // we could extend the ApiError type to include validation details
        BffError::Validation(msg.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::StatusCode;

    #[test]
    fn test_bff_error_status_codes() {
        assert_eq!(
            BffError::authentication("test").into_response().status(),
            StatusCode::UNAUTHORIZED
        );
        
        assert_eq!(
            BffError::authorization("test").into_response().status(),
            StatusCode::FORBIDDEN
        );
        
        assert_eq!(
            BffError::validation("test").into_response().status(),
            StatusCode::BAD_REQUEST
        );
        
        assert_eq!(
            BffError::not_found("test").into_response().status(),
            StatusCode::NOT_FOUND
        );
        
        assert_eq!(
            BffError::internal("test").into_response().status(),
            StatusCode::INTERNAL_SERVER_ERROR
        );
    }

    #[test]
    fn test_error_conversion() {
        let redis_error = redis::RedisError::from((redis::ErrorKind::TypeError, "test error"));
        let bff_error = BffError::from(redis_error);
        
        match bff_error {
            BffError::Redis(_) => (),
            _ => panic!("Expected Redis error"),
        }
    }

    #[test]
    fn test_validation_error_with_details() {
        let details = vec![
            ValidationErrorDetail {
                field: "email".to_string(),
                code: "INVALID_FORMAT".to_string(),
                message: "Email format is invalid".to_string(),
                rejected_value: Some(json!("invalid-email")),
            }
        ];

        let error = BffError::validation_with_details("Validation failed", details);
        
        match error {
            BffError::Validation(_) => (),
            _ => panic!("Expected Validation error"),
        }
    }
}