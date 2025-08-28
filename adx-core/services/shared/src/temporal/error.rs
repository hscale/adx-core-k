use std::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Temporal-related errors for ADX Core
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum TemporalError {
    /// Connection errors
    #[error("Failed to connect to Temporal server: {message}")]
    ConnectionError { message: String },
    
    /// Client initialization errors
    #[error("Failed to initialize Temporal client: {message}")]
    ClientInitializationError { message: String },
    
    /// Workflow execution errors
    #[error("Workflow execution failed: {workflow_id} - {message}")]
    WorkflowExecutionError { workflow_id: String, message: String },
    
    /// SDK client not initialized
    #[error("Temporal SDK client not initialized")]
    ClientNotInitialized,
    
    /// Worker already running
    #[error("Temporal worker already running")]
    WorkerAlreadyRunning,
    
    /// Workflow not found with run ID
    #[error("Workflow not found: {workflow_id}:{run_id}")]
    WorkflowNotFoundWithRun { workflow_id: String, run_id: String },
    
    /// Activity execution errors
    #[error("Activity execution failed: {activity_id} - {message}")]
    ActivityExecutionError { activity_id: String, message: String },
    
    /// Worker initialization errors
    #[error("Worker initialization failed: {message}")]
    WorkerInitializationError { message: String },
    
    /// Workflow not found errors
    #[error("Workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: String },
    
    /// Namespace errors
    #[error("Namespace error: {namespace} - {message}")]
    NamespaceError { namespace: String, message: String },
    
    /// Serialization/Deserialization errors
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    /// Timeout errors
    #[error("Operation timed out: {operation} after {timeout_seconds}s")]
    TimeoutError { operation: String, timeout_seconds: u64 },
    
    /// Retry exhausted errors
    #[error("Retry attempts exhausted for {operation}: {attempts} attempts failed")]
    RetryExhaustedError { operation: String, attempts: u32 },
    
    /// Workflow versioning errors
    #[error("Workflow version error: {workflow_type} - {message}")]
    VersioningError { workflow_type: String, message: String },
    
    /// Multi-tenant errors
    #[error("Multi-tenant error: tenant {tenant_id} - {message}")]
    MultiTenantError { tenant_id: String, message: String },
    
    /// Configuration errors
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    /// Worker errors
    #[error("Worker error: {worker_identity} - {message}")]
    WorkerError { worker_identity: String, message: String },
    
    /// Task queue errors
    #[error("Task queue error: {task_queue} - {message}")]
    TaskQueueError { task_queue: String, message: String },
    
    /// Generic temporal errors
    #[error("Temporal error: {message}")]
    Generic { message: String },
}

impl TemporalError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            TemporalError::ConnectionError { .. } => true,
            TemporalError::TimeoutError { .. } => true,
            TemporalError::TaskQueueError { .. } => true,
            TemporalError::WorkerError { .. } => true,
            TemporalError::Generic { .. } => true,
            
            // Non-retryable errors
            TemporalError::ClientInitializationError { .. } => false,
            TemporalError::WorkflowNotFound { .. } => false,
            TemporalError::SerializationError { .. } => false,
            TemporalError::VersioningError { .. } => false,
            TemporalError::ConfigurationError { .. } => false,
            TemporalError::NamespaceError { .. } => false,
            TemporalError::MultiTenantError { .. } => false,
            TemporalError::RetryExhaustedError { .. } => false,
            
            // Context-dependent
            TemporalError::WorkflowExecutionError { .. } => true,
            TemporalError::ActivityExecutionError { .. } => true,
            TemporalError::WorkerInitializationError { .. } => false,
            TemporalError::ClientNotInitialized => false,
            TemporalError::WorkerAlreadyRunning => false,
            TemporalError::WorkflowNotFoundWithRun { .. } => false,
        }
    }
    
    /// Get error category for monitoring and alerting
    pub fn category(&self) -> ErrorCategory {
        match self {
            TemporalError::ConnectionError { .. } => ErrorCategory::Infrastructure,
            TemporalError::ClientInitializationError { .. } => ErrorCategory::Configuration,
            TemporalError::WorkflowExecutionError { .. } => ErrorCategory::Business,
            TemporalError::ActivityExecutionError { .. } => ErrorCategory::Business,
            TemporalError::WorkerInitializationError { .. } => ErrorCategory::Configuration,
            TemporalError::WorkflowNotFound { .. } => ErrorCategory::NotFound,
            TemporalError::NamespaceError { .. } => ErrorCategory::Configuration,
            TemporalError::SerializationError { .. } => ErrorCategory::Data,
            TemporalError::TimeoutError { .. } => ErrorCategory::Performance,
            TemporalError::RetryExhaustedError { .. } => ErrorCategory::Reliability,
            TemporalError::VersioningError { .. } => ErrorCategory::Compatibility,
            TemporalError::MultiTenantError { .. } => ErrorCategory::Security,
            TemporalError::ConfigurationError { .. } => ErrorCategory::Configuration,
            TemporalError::WorkerError { .. } => ErrorCategory::Infrastructure,
            TemporalError::TaskQueueError { .. } => ErrorCategory::Infrastructure,
            TemporalError::Generic { .. } => ErrorCategory::Unknown,
            TemporalError::ClientNotInitialized => ErrorCategory::Configuration,
            TemporalError::WorkerAlreadyRunning => ErrorCategory::Infrastructure,
            TemporalError::WorkflowNotFoundWithRun { .. } => ErrorCategory::NotFound,
        }
    }
    
    /// Get severity level for monitoring
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            TemporalError::ConnectionError { .. } => ErrorSeverity::High,
            TemporalError::ClientInitializationError { .. } => ErrorSeverity::Critical,
            TemporalError::WorkflowExecutionError { .. } => ErrorSeverity::Medium,
            TemporalError::ActivityExecutionError { .. } => ErrorSeverity::Medium,
            TemporalError::WorkerInitializationError { .. } => ErrorSeverity::Critical,
            TemporalError::WorkflowNotFound { .. } => ErrorSeverity::Low,
            TemporalError::NamespaceError { .. } => ErrorSeverity::High,
            TemporalError::SerializationError { .. } => ErrorSeverity::Medium,
            TemporalError::TimeoutError { .. } => ErrorSeverity::Medium,
            TemporalError::RetryExhaustedError { .. } => ErrorSeverity::High,
            TemporalError::VersioningError { .. } => ErrorSeverity::High,
            TemporalError::MultiTenantError { .. } => ErrorSeverity::High,
            TemporalError::ConfigurationError { .. } => ErrorSeverity::Critical,
            TemporalError::WorkerError { .. } => ErrorSeverity::High,
            TemporalError::TaskQueueError { .. } => ErrorSeverity::Medium,
            TemporalError::Generic { .. } => ErrorSeverity::Medium,
            TemporalError::ClientNotInitialized => ErrorSeverity::Critical,
            TemporalError::WorkerAlreadyRunning => ErrorSeverity::Low,
            TemporalError::WorkflowNotFoundWithRun { .. } => ErrorSeverity::Low,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorCategory {
    Infrastructure,
    Configuration,
    Business,
    NotFound,
    Data,
    Performance,
    Reliability,
    Compatibility,
    Security,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Workflow-specific error types
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum WorkflowError {
    #[error("Workflow validation failed: {errors:?}")]
    ValidationFailed { errors: Vec<String> },
    
    #[error("Workflow cancelled by user: {reason}")]
    CancelledByUser { reason: String },
    
    #[error("Workflow timed out: {workflow_type} after {timeout_seconds}s")]
    TimedOut { workflow_type: String, timeout_seconds: u64 },
    
    #[error("Activity failed: {activity_name} - {error}")]
    ActivityFailed { activity_name: String, error: String },
    
    #[error("Child workflow failed: {child_workflow_id} - {error}")]
    ChildWorkflowFailed { child_workflow_id: String, error: String },
    
    #[error("Tenant access denied: {tenant_id} - {reason}")]
    TenantAccessDenied { tenant_id: String, reason: String },
    
    #[error("Quota exceeded: {quota_type} - current: {current}, limit: {limit}")]
    QuotaExceeded { quota_type: String, current: u64, limit: u64 },
    
    #[error("Cross-tenant sharing denied: {reason}")]
    CrossTenantSharingDenied { reason: String },
    
    #[error("Security scan failed: {issues:?}")]
    SecurityScanFailed { issues: Vec<String> },
    
    #[error("Payment failed: {error}")]
    PaymentFailed { error: String },
    
    #[error("External service error: {service} - {error}")]
    ExternalServiceError { service: String, error: String },
    
    #[error("Workflow execution failed: {workflow_id} - {error}")]
    ExecutionFailed { workflow_id: String, error: String },
    
    #[error("Rate limit exceeded: {message}, retry after {retry_after:?}")]
    RateLimitExceeded { message: String, retry_after: chrono::Duration },
    
    #[error("Security violation: {message}")]
    SecurityViolation { message: String },
    
    #[error("Workflow cancelled: {workflow_id}")]
    Cancelled { workflow_id: String },
    
    #[error("Serialization error in workflow {workflow_id}: {error}")]
    SerializationError { workflow_id: String, error: String },
}

/// Activity-specific error types
#[derive(Debug, Error, Clone, Serialize, Deserialize)]
pub enum ActivityError {
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    #[error("Validation error: {field} - {message}")]
    ValidationError { field: String, message: String },
    
    #[error("Authorization error: {message}")]
    AuthorizationError { message: String },
    
    #[error("External service error: {service} - {message}")]
    ExternalServiceError { service: String, message: String },
    
    #[error("File system error: {operation} - {message}")]
    FileSystemError { operation: String, message: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Configuration error: {message}")]
    ConfigurationError { message: String },
    
    #[error("Resource not found: {resource_type} - {resource_id}")]
    ResourceNotFound { resource_type: String, resource_id: String },
    
    #[error("Resource conflict: {resource_type} - {message}")]
    ResourceConflict { resource_type: String, message: String },
    
    #[error("Rate limit exceeded: {limit_type} - {current}/{limit}")]
    RateLimitExceeded { limit_type: String, current: u64, limit: u64 },
    
    #[error("Temporary failure: {message}")]
    TemporaryFailure { message: String },
    
    #[error("Internal error: {message}")]
    InternalError { message: String },
    
    #[error("Quota exceeded: {message}, current: {current_usage}, limit: {limit}")]
    QuotaExceeded { message: String, current_usage: u64, limit: u64 },
}

impl ActivityError {
    /// Check if the activity error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            ActivityError::DatabaseError { .. } => true,
            ActivityError::NetworkError { .. } => true,
            ActivityError::ExternalServiceError { .. } => true,
            ActivityError::FileSystemError { .. } => true,
            ActivityError::TemporaryFailure { .. } => true,
            ActivityError::RateLimitExceeded { .. } => true,
            ActivityError::InternalError { .. } => false,
            ActivityError::QuotaExceeded { .. } => false,
            
            // Non-retryable errors
            ActivityError::ValidationError { .. } => false,
            ActivityError::AuthorizationError { .. } => false,
            ActivityError::SerializationError { .. } => false,
            ActivityError::ConfigurationError { .. } => false,
            ActivityError::ResourceNotFound { .. } => false,
            ActivityError::ResourceConflict { .. } => false,
        }
    }
    
    /// Get retry delay for retryable errors
    pub fn retry_delay(&self) -> Option<std::time::Duration> {
        match self {
            ActivityError::RateLimitExceeded { .. } => Some(std::time::Duration::from_secs(60)),
            ActivityError::NetworkError { .. } => Some(std::time::Duration::from_secs(5)),
            ActivityError::ExternalServiceError { .. } => Some(std::time::Duration::from_secs(10)),
            ActivityError::TemporaryFailure { .. } => Some(std::time::Duration::from_secs(30)),
            _ => None,
        }
    }
}

/// Convert from standard errors to Temporal errors
impl From<std::io::Error> for TemporalError {
    fn from(err: std::io::Error) -> Self {
        TemporalError::Generic {
            message: format!("IO error: {}", err),
        }
    }
}

impl From<serde_json::Error> for TemporalError {
    fn from(err: serde_json::Error) -> Self {
        TemporalError::SerializationError {
            message: format!("JSON serialization error: {}", err),
        }
    }
}

impl From<tokio::time::error::Elapsed> for TemporalError {
    fn from(_err: tokio::time::error::Elapsed) -> Self {
        TemporalError::TimeoutError {
            operation: "unknown".to_string(),
            timeout_seconds: 0,
        }
    }
}

/// Convert from ActivityError to WorkflowError
impl From<ActivityError> for WorkflowError {
    fn from(err: ActivityError) -> Self {
        WorkflowError::ActivityFailed {
            activity_name: "unknown".to_string(),
            error: format!("{}", err),
        }
    }
}