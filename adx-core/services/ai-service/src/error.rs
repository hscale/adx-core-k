use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

pub type AIResult<T> = Result<T, AIError>;

#[derive(Error, Debug)]
pub enum AIError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),
    
    #[error("Temporal error: {0}")]
    Temporal(String),
    
    #[error("AI provider error: {0}")]
    AIProvider(String),
    
    #[error("Configuration error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),
    
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    
    #[error("Model not available: {0}")]
    ModelNotAvailable(String),
    
    #[error("Token limit exceeded: {0}")]
    TokenLimitExceeded(String),
    
    #[error("Content filtered: {0}")]
    ContentFiltered(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
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

impl IntoResponse for AIError {
    fn into_response(self) -> Response {
        let (status, error_code, message, details, retry_after) = match &self {
            AIError::Database(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "DATABASE_ERROR",
                "Database operation failed",
                None,
                None,
            ),
            AIError::Redis(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "CACHE_ERROR",
                "Cache operation failed",
                None,
                None,
            ),
            AIError::HttpClient(_) => (
                StatusCode::BAD_GATEWAY,
                "EXTERNAL_SERVICE_ERROR",
                "External service request failed",
                None,
                Some(30),
            ),
            AIError::Temporal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "WORKFLOW_ERROR",
                msg,
                None,
                None,
            ),
            AIError::AIProvider(msg) => (
                StatusCode::BAD_GATEWAY,
                "AI_PROVIDER_ERROR",
                msg,
                None,
                Some(60),
            ),
            AIError::Authentication(msg) => (
                StatusCode::UNAUTHORIZED,
                "AUTHENTICATION_FAILED",
                msg,
                None,
                None,
            ),
            AIError::Authorization(msg) => (
                StatusCode::FORBIDDEN,
                "AUTHORIZATION_FAILED",
                msg,
                None,
                None,
            ),
            AIError::Validation(msg) => (
                StatusCode::BAD_REQUEST,
                "VALIDATION_ERROR",
                msg,
                None,
                None,
            ),
            AIError::RateLimit(msg) => (
                StatusCode::TOO_MANY_REQUESTS,
                "RATE_LIMIT_EXCEEDED",
                msg,
                None,
                Some(60),
            ),
            AIError::QuotaExceeded(msg) => (
                StatusCode::PAYMENT_REQUIRED,
                "QUOTA_EXCEEDED",
                msg,
                None,
                None,
            ),
            AIError::ModelNotAvailable(msg) => (
                StatusCode::SERVICE_UNAVAILABLE,
                "MODEL_NOT_AVAILABLE",
                msg,
                None,
                Some(300),
            ),
            AIError::TokenLimitExceeded(msg) => (
                StatusCode::BAD_REQUEST,
                "TOKEN_LIMIT_EXCEEDED",
                msg,
                None,
                None,
            ),
            AIError::ContentFiltered(msg) => (
                StatusCode::BAD_REQUEST,
                "CONTENT_FILTERED",
                msg,
                None,
                None,
            ),
            AIError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                "NOT_FOUND",
                msg,
                None,
                None,
            ),
            AIError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                "BAD_REQUEST",
                msg,
                None,
                None,
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "INTERNAL_ERROR",
                "An unexpected error occurred",
                None,
                None,
            ),
        };

        let error_response = ErrorResponse {
            error: ErrorDetails {
                code: error_code.to_string(),
                message: message.to_string(),
                details,
                retry_after,
                documentation_url: Some("https://docs.adxcore.com/api/ai-service".to_string()),
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
    #[error("AI generation failed: {0}")]
    GenerationFailed(String),
    
    #[error("Model unavailable: {0}")]
    ModelUnavailable(String),
    
    #[error("Token limit exceeded: {0}")]
    TokenLimitExceeded(String),
    
    #[error("Content policy violation: {0}")]
    ContentPolicyViolation(String),
    
    #[error("Rate limit exceeded: {0}")]
    RateLimitExceeded(String),
    
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
}

impl ActivityError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ActivityError::ModelUnavailable(_)
                | ActivityError::RateLimitExceeded(_)
                | ActivityError::ExternalServiceError(_)
        )
    }
    
    pub fn retry_delay_seconds(&self) -> u64 {
        match self {
            ActivityError::RateLimitExceeded(_) => 60,
            ActivityError::ModelUnavailable(_) => 30,
            ActivityError::ExternalServiceError(_) => 10,
            _ => 5,
        }
    }
}