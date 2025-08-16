use thiserror::Error;
use serde::{Deserialize, Serialize};

pub type ModuleResult<T> = Result<T, ModuleError>;

#[derive(Error, Debug, Clone, Serialize, Deserialize)]
pub enum ModuleError {
    #[error("Module not found: {0}")]
    NotFound(String),
    
    #[error("Module already exists: {0}")]
    AlreadyExists(String),
    
    #[error("Module validation failed: {0}")]
    ValidationFailed(String),
    
    #[error("Module dependency error: {0}")]
    DependencyError(String),
    
    #[error("Module installation failed: {0}")]
    InstallationFailed(String),
    
    #[error("Module activation failed: {0}")]
    ActivationFailed(String),
    
    #[error("Module deactivation failed: {0}")]
    DeactivationFailed(String),
    
    #[error("Module uninstallation failed: {0}")]
    UninstallationFailed(String),
    
    #[error("Module security scan failed: {0}")]
    SecurityScanFailed(String),
    
    #[error("Module sandbox violation: {0}")]
    SandboxViolation(String),
    
    #[error("Module resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("Module marketplace error: {0}")]
    MarketplaceError(String),
    
    #[error("Module payment error: {0}")]
    PaymentError(String),
    
    #[error("Module version incompatible: {0}")]
    VersionIncompatible(String),
    
    #[error("Module permission denied: {0}")]
    PermissionDenied(String),
    
    #[error("Module runtime error: {0}")]
    RuntimeError(String),
    
    #[error("Module workflow error: {0}")]
    WorkflowError(String),
    
    #[error("Database error: {0}")]
    DatabaseError(String),
    
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("IO error: {0}")]
    IoError(String),
    
    #[error("Serialization error: {0}")]
    SerializationError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl From<sqlx::Error> for ModuleError {
    fn from(err: sqlx::Error) -> Self {
        ModuleError::DatabaseError(err.to_string())
    }
}

impl From<reqwest::Error> for ModuleError {
    fn from(err: reqwest::Error) -> Self {
        ModuleError::NetworkError(err.to_string())
    }
}

impl From<std::io::Error> for ModuleError {
    fn from(err: std::io::Error) -> Self {
        ModuleError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for ModuleError {
    fn from(err: serde_json::Error) -> Self {
        ModuleError::SerializationError(err.to_string())
    }
}

impl From<anyhow::Error> for ModuleError {
    fn from(err: anyhow::Error) -> Self {
        ModuleError::InternalError(err.to_string())
    }
}