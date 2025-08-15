use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

pub type WhiteLabelResult<T> = Result<T, WhiteLabelError>;

#[derive(Error, Debug)]
pub enum WhiteLabelError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Temporal error: {0}")]
    Temporal(String),

    #[error("DNS verification failed: {0}")]
    DnsVerification(String),

    #[error("SSL certificate error: {0}")]
    SslCertificate(String),

    #[error("Asset processing error: {0}")]
    AssetProcessing(String),

    #[error("Domain validation error: {0}")]
    DomainValidation(String),

    #[error("Branding validation error: {0}")]
    BrandingValidation(String),

    #[error("Reseller hierarchy error: {0}")]
    ResellerHierarchy(String),

    #[error("Template processing error: {0}")]
    TemplateProcessing(String),

    #[error("External service error: {0}")]
    ExternalService(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Unauthorized: {0}")]
    Unauthorized(String),

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal server error: {0}")]
    Internal(String),
}

impl IntoResponse for WhiteLabelError {
    fn into_response(self) -> Response {
        let (status, error_message) = match &self {
            WhiteLabelError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error"),
            WhiteLabelError::Temporal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Workflow error"),
            WhiteLabelError::DnsVerification(_) => (StatusCode::BAD_REQUEST, "DNS verification failed"),
            WhiteLabelError::SslCertificate(_) => (StatusCode::BAD_REQUEST, "SSL certificate error"),
            WhiteLabelError::AssetProcessing(_) => (StatusCode::BAD_REQUEST, "Asset processing error"),
            WhiteLabelError::DomainValidation(_) => (StatusCode::BAD_REQUEST, "Domain validation error"),
            WhiteLabelError::BrandingValidation(_) => (StatusCode::BAD_REQUEST, "Branding validation error"),
            WhiteLabelError::ResellerHierarchy(_) => (StatusCode::BAD_REQUEST, "Reseller hierarchy error"),
            WhiteLabelError::TemplateProcessing(_) => (StatusCode::BAD_REQUEST, "Template processing error"),
            WhiteLabelError::ExternalService(_) => (StatusCode::BAD_GATEWAY, "External service error"),
            WhiteLabelError::Configuration(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Configuration error"),
            WhiteLabelError::Validation(_) => (StatusCode::BAD_REQUEST, "Validation error"),
            WhiteLabelError::NotFound(_) => (StatusCode::NOT_FOUND, "Resource not found"),
            WhiteLabelError::Unauthorized(_) => (StatusCode::UNAUTHORIZED, "Unauthorized"),
            WhiteLabelError::Forbidden(_) => (StatusCode::FORBIDDEN, "Forbidden"),
            WhiteLabelError::Conflict(_) => (StatusCode::CONFLICT, "Resource conflict"),
            WhiteLabelError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error"),
        };

        let body = Json(json!({
            "error": {
                "code": format!("{:?}", self).split('(').next().unwrap_or("Unknown"),
                "message": error_message,
                "details": self.to_string(),
            }
        }));

        (status, body).into_response()
    }
}