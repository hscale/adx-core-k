// Error handling for ADX Core services

use thiserror::Error;

pub type Result<T> = std::result::Result<T, ServiceError>;

#[derive(Error, Debug)]
pub enum ServiceError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Tenant error: {0}")]
    Tenant(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Temporal workflow error: {0}")]
    Workflow(String),
    
    #[error("External service error: {0}")]
    ExternalService(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Internal server error: {0}")]
    Internal(String),
}

impl ServiceError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            ServiceError::Database(_) | ServiceError::Redis(_) | ServiceError::ExternalService(_)
        )
    }
    
    pub fn status_code(&self) -> u16 {
        match self {
            ServiceError::Authentication(_) => 401,
            ServiceError::Authorization(_) => 403,
            ServiceError::Validation(_) => 400,
            ServiceError::Tenant(_) => 404,
            _ => 500,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_status_codes() {
        assert_eq!(ServiceError::Authentication("test".to_string()).status_code(), 401);
        assert_eq!(ServiceError::Authorization("test".to_string()).status_code(), 403);
        assert_eq!(ServiceError::Validation("test".to_string()).status_code(), 400);
        assert_eq!(ServiceError::Internal("test".to_string()).status_code(), 500);
    }

    #[test]
    fn test_error_retryable() {
        assert!(ServiceError::ExternalService("test".to_string()).is_retryable());
        assert!(!ServiceError::Authentication("test".to_string()).is_retryable());
        assert!(!ServiceError::Validation("test".to_string()).is_retryable());
    }
}