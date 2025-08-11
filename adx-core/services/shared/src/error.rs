use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Redis error: {0}")]
    Redis(#[from] redis::RedisError),
    
    #[error("Temporal error: {0}")]
    Temporal(String),
    
    #[error("Authentication error: {0}")]
    Authentication(String),
    
    #[error("Authorization error: {0}")]
    Authorization(String),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Configuration error: {0}")]
    Configuration(String),
    
    #[error("Config error: {0}")]
    Config(#[from] config::ConfigError),
    
    #[error("Network error: {0}")]
    Network(String),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
    
    #[error("Not found: {0}")]
    NotFound(String),
    
    #[error("Conflict: {0}")]
    Conflict(String),
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("Quota exceeded: {0}")]
    QuotaExceeded(String),
    
    #[error("Tenant error: {0}")]
    Tenant(String),
    
    #[error("Workflow error: {0}")]
    Workflow(String),
    
    #[error("Activity error: {0}")]
    Activity(String),
}

impl Error {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Error::Database(_) | Error::Redis(_) | Error::Http(_) | Error::Temporal(_)
        )
    }
    
    pub fn status_code(&self) -> u16 {
        match self {
            Error::Authentication(_) => 401,
            Error::Authorization(_) => 403,
            Error::NotFound(_) => 404,
            Error::Validation(_) | Error::Conflict(_) => 400,
            Error::RateLimitExceeded => 429,
            Error::QuotaExceeded(_) => 429,
            _ => 500,
        }
    }
}