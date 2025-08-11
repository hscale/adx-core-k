use std::time::Duration;
use serde::{Deserialize, Serialize};
use tracing::{debug, warn};

use crate::temporal::{TemporalError, ActivityError};

/// Retry policy for Temporal operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Initial retry interval
    pub initial_interval: Duration,
    
    /// Maximum retry interval
    pub max_interval: Duration,
    
    /// Backoff coefficient (multiplier for each retry)
    pub backoff_coefficient: f64,
    
    /// Maximum number of retry attempts
    pub max_attempts: u32,
    
    /// Non-retryable error types
    pub non_retryable_errors: Vec<String>,
    
    /// Maximum total time for all retries
    pub max_elapsed_time: Option<Duration>,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            backoff_coefficient: 2.0,
            max_attempts: 3,
            non_retryable_errors: vec![
                "ValidationError".to_string(),
                "AuthorizationError".to_string(),
                "TenantNotFoundError".to_string(),
                "ConfigurationError".to_string(),
            ],
            max_elapsed_time: Some(Duration::from_secs(300)), // 5 minutes
        }
    }
}

impl RetryPolicy {
    /// Create a retry policy with no retries
    pub fn no_retry() -> Self {
        Self {
            max_attempts: 1,
            ..Default::default()
        }
    }
    
    /// Create a retry policy with exponential backoff
    pub fn exponential_backoff(max_attempts: u32, initial_interval: Duration) -> Self {
        Self {
            max_attempts,
            initial_interval,
            backoff_coefficient: 2.0,
            max_interval: Duration::from_secs(60),
            ..Default::default()
        }
    }
    
    /// Create a retry policy with linear backoff
    pub fn linear_backoff(max_attempts: u32, interval: Duration) -> Self {
        Self {
            max_attempts,
            initial_interval: interval,
            backoff_coefficient: 1.0,
            max_interval: interval,
            ..Default::default()
        }
    }
    
    /// Create a retry policy for database operations
    pub fn database_operations() -> Self {
        Self {
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(30),
            backoff_coefficient: 1.5,
            max_attempts: 5,
            non_retryable_errors: vec![
                "ValidationError".to_string(),
                "AuthorizationError".to_string(),
                "ConstraintViolationError".to_string(),
            ],
            max_elapsed_time: Some(Duration::from_secs(120)),
        }
    }
    
    /// Create a retry policy for external service calls
    pub fn external_service_calls() -> Self {
        Self {
            initial_interval: Duration::from_secs(2),
            max_interval: Duration::from_secs(120),
            backoff_coefficient: 2.0,
            max_attempts: 4,
            non_retryable_errors: vec![
                "AuthenticationError".to_string(),
                "AuthorizationError".to_string(),
                "BadRequestError".to_string(),
                "NotFoundError".to_string(),
            ],
            max_elapsed_time: Some(Duration::from_secs(600)), // 10 minutes
        }
    }
    
    /// Create a retry policy for file operations
    pub fn file_operations() -> Self {
        Self {
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(10),
            backoff_coefficient: 1.5,
            max_attempts: 3,
            non_retryable_errors: vec![
                "PermissionDeniedError".to_string(),
                "FileNotFoundError".to_string(),
                "InvalidPathError".to_string(),
            ],
            max_elapsed_time: Some(Duration::from_secs(30)),
        }
    }
    
    /// Calculate the delay for a specific retry attempt
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        if attempt == 0 {
            return Duration::from_secs(0);
        }
        
        let base_delay = self.initial_interval.as_millis() as f64;
        let backoff_multiplier = self.backoff_coefficient.powi(attempt as i32 - 1);
        let delay_ms = (base_delay * backoff_multiplier) as u64;
        
        let delay = Duration::from_millis(delay_ms);
        std::cmp::min(delay, self.max_interval)
    }
    
    /// Check if an error should be retried
    pub fn should_retry(&self, error: &TemporalError, attempt: u32) -> bool {
        // Check if we've exceeded max attempts
        if attempt >= self.max_attempts {
            debug!(
                attempt = attempt,
                max_attempts = self.max_attempts,
                "Max retry attempts exceeded"
            );
            return false;
        }
        
        // Check if error is retryable
        if !error.is_retryable() {
            debug!(
                error = %error,
                "Error is not retryable"
            );
            return false;
        }
        
        // Check if error type is in non-retryable list
        let error_type = self.get_error_type(error);
        if self.non_retryable_errors.contains(&error_type) {
            debug!(
                error_type = error_type,
                "Error type is in non-retryable list"
            );
            return false;
        }
        
        true
    }
    
    /// Check if an activity error should be retried
    pub fn should_retry_activity(&self, error: &ActivityError, attempt: u32) -> bool {
        // Check if we've exceeded max attempts
        if attempt >= self.max_attempts {
            return false;
        }
        
        // Check if error is retryable
        if !error.is_retryable() {
            return false;
        }
        
        // Check if error type is in non-retryable list
        let error_type = self.get_activity_error_type(error);
        if self.non_retryable_errors.contains(&error_type) {
            return false;
        }
        
        true
    }
    
    /// Get error type string for classification
    fn get_error_type(&self, error: &TemporalError) -> String {
        match error {
            TemporalError::ConnectionError { .. } => "ConnectionError".to_string(),
            TemporalError::ClientInitializationError { .. } => "ClientInitializationError".to_string(),
            TemporalError::WorkflowExecutionError { .. } => "WorkflowExecutionError".to_string(),
            TemporalError::ActivityExecutionError { .. } => "ActivityExecutionError".to_string(),
            TemporalError::WorkflowNotFound { .. } => "WorkflowNotFound".to_string(),
            TemporalError::NamespaceError { .. } => "NamespaceError".to_string(),
            TemporalError::SerializationError { .. } => "SerializationError".to_string(),
            TemporalError::TimeoutError { .. } => "TimeoutError".to_string(),
            TemporalError::RetryExhaustedError { .. } => "RetryExhaustedError".to_string(),
            TemporalError::VersioningError { .. } => "VersioningError".to_string(),
            TemporalError::MultiTenantError { .. } => "MultiTenantError".to_string(),
            TemporalError::ConfigurationError { .. } => "ConfigurationError".to_string(),
            TemporalError::WorkerError { .. } => "WorkerError".to_string(),
            TemporalError::TaskQueueError { .. } => "TaskQueueError".to_string(),
            TemporalError::Generic { .. } => "Generic".to_string(),
            TemporalError::WorkerInitializationError { .. } => "WorkerInitializationError".to_string(),
        }
    }
    
    /// Get activity error type string for classification
    fn get_activity_error_type(&self, error: &ActivityError) -> String {
        match error {
            ActivityError::DatabaseError { .. } => "DatabaseError".to_string(),
            ActivityError::NetworkError { .. } => "NetworkError".to_string(),
            ActivityError::ValidationError { .. } => "ValidationError".to_string(),
            ActivityError::AuthorizationError { .. } => "AuthorizationError".to_string(),
            ActivityError::ExternalServiceError { .. } => "ExternalServiceError".to_string(),
            ActivityError::FileSystemError { .. } => "FileSystemError".to_string(),
            ActivityError::SerializationError { .. } => "SerializationError".to_string(),
            ActivityError::ConfigurationError { .. } => "ConfigurationError".to_string(),
            ActivityError::ResourceNotFound { .. } => "ResourceNotFound".to_string(),
            ActivityError::ResourceConflict { .. } => "ResourceConflict".to_string(),
            ActivityError::RateLimitExceeded { .. } => "RateLimitExceeded".to_string(),
            ActivityError::TemporaryFailure { .. } => "TemporaryFailure".to_string(),
            ActivityError::InternalError { .. } => "InternalError".to_string(),
            ActivityError::QuotaExceeded { .. } => "QuotaExceeded".to_string(),
        }
    }
}

/// Retry executor for operations with retry logic
pub struct RetryExecutor {
    policy: RetryPolicy,
}

impl RetryExecutor {
    /// Create a new retry executor with the given policy
    pub fn new(policy: RetryPolicy) -> Self {
        Self { policy }
    }
    
    /// Execute an operation with retry logic
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>> + Send>>,
        E: std::fmt::Display + Clone,
    {
        let mut attempt = 1;
        let start_time = std::time::Instant::now();
        
        loop {
            debug!(
                attempt = attempt,
                max_attempts = self.policy.max_attempts,
                "Executing operation with retry"
            );
            
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!(
                            attempt = attempt,
                            elapsed_ms = start_time.elapsed().as_millis(),
                            "Operation succeeded after retries"
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    // Check if we should retry
                    if attempt >= self.policy.max_attempts {
                        warn!(
                            attempt = attempt,
                            max_attempts = self.policy.max_attempts,
                            error = %error,
                            "Operation failed after max retry attempts"
                        );
                        return Err(error);
                    }
                    
                    // Check max elapsed time
                    if let Some(max_elapsed) = self.policy.max_elapsed_time {
                        if start_time.elapsed() >= max_elapsed {
                            warn!(
                                elapsed_ms = start_time.elapsed().as_millis(),
                                max_elapsed_ms = max_elapsed.as_millis(),
                                error = %error,
                                "Operation failed due to max elapsed time"
                            );
                            return Err(error);
                        }
                    }
                    
                    // Calculate delay and wait
                    let delay = self.policy.calculate_delay(attempt);
                    warn!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = %error,
                        "Operation failed, retrying after delay"
                    );
                    
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
    
    /// Execute an operation with retry logic for Temporal errors
    pub async fn execute_temporal<F, T>(&self, operation: F) -> Result<T, TemporalError>
    where
        F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, TemporalError>> + Send>>,
    {
        let mut attempt = 1;
        let start_time = std::time::Instant::now();
        
        loop {
            debug!(
                attempt = attempt,
                max_attempts = self.policy.max_attempts,
                "Executing Temporal operation with retry"
            );
            
            match operation().await {
                Ok(result) => {
                    if attempt > 1 {
                        debug!(
                            attempt = attempt,
                            elapsed_ms = start_time.elapsed().as_millis(),
                            "Temporal operation succeeded after retries"
                        );
                    }
                    return Ok(result);
                }
                Err(error) => {
                    // Check if we should retry this error
                    if !self.policy.should_retry(&error, attempt) {
                        warn!(
                            attempt = attempt,
                            error = %error,
                            "Temporal operation failed with non-retryable error"
                        );
                        return Err(error);
                    }
                    
                    // Check max elapsed time
                    if let Some(max_elapsed) = self.policy.max_elapsed_time {
                        if start_time.elapsed() >= max_elapsed {
                            warn!(
                                elapsed_ms = start_time.elapsed().as_millis(),
                                max_elapsed_ms = max_elapsed.as_millis(),
                                error = %error,
                                "Temporal operation failed due to max elapsed time"
                            );
                            return Err(TemporalError::RetryExhaustedError {
                                operation: "temporal_operation".to_string(),
                                attempts: attempt,
                            });
                        }
                    }
                    
                    // Calculate delay and wait
                    let delay = self.policy.calculate_delay(attempt);
                    warn!(
                        attempt = attempt,
                        delay_ms = delay.as_millis(),
                        error = %error,
                        "Temporal operation failed, retrying after delay"
                    );
                    
                    tokio::time::sleep(delay).await;
                    attempt += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_retry_policy_delay_calculation() {
        let policy = RetryPolicy::exponential_backoff(5, Duration::from_secs(1));
        
        assert_eq!(policy.calculate_delay(0), Duration::from_secs(0));
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(2));
        assert_eq!(policy.calculate_delay(3), Duration::from_secs(4));
        assert_eq!(policy.calculate_delay(4), Duration::from_secs(8));
    }
    
    #[test]
    fn test_retry_policy_max_interval() {
        let policy = RetryPolicy {
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(5),
            backoff_coefficient: 2.0,
            max_attempts: 10,
            ..Default::default()
        };
        
        assert_eq!(policy.calculate_delay(1), Duration::from_secs(1));
        assert_eq!(policy.calculate_delay(2), Duration::from_secs(2));
        assert_eq!(policy.calculate_delay(3), Duration::from_secs(4));
        assert_eq!(policy.calculate_delay(4), Duration::from_secs(5)); // Capped at max_interval
        assert_eq!(policy.calculate_delay(5), Duration::from_secs(5)); // Still capped
    }
    
    #[test]
    fn test_should_retry_logic() {
        let policy = RetryPolicy::default();
        
        // Retryable error within max attempts
        let retryable_error = TemporalError::ConnectionError {
            message: "Connection failed".to_string(),
        };
        assert!(policy.should_retry(&retryable_error, 1));
        assert!(policy.should_retry(&retryable_error, 2));
        assert!(!policy.should_retry(&retryable_error, 3)); // Exceeds max_attempts
        
        // Non-retryable error
        let non_retryable_error = TemporalError::ConfigurationError {
            message: "Invalid config".to_string(),
        };
        assert!(!policy.should_retry(&non_retryable_error, 1));
    }
    
    #[tokio::test]
    async fn test_retry_executor_success() {
        let policy = RetryPolicy::exponential_backoff(3, Duration::from_millis(10));
        let executor = RetryExecutor::new(policy);
        
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let result = executor.execute(|| {
            let count = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(async move {
                if count < 1 {
                    Err("Temporary failure")
                } else {
                    Ok("Success")
                }
            })
        }).await;
        
        assert_eq!(result, Ok("Success"));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
    
    #[tokio::test]
    async fn test_retry_executor_max_attempts() {
        let policy = RetryPolicy::exponential_backoff(2, Duration::from_millis(1));
        let executor = RetryExecutor::new(policy);
        
        let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let result: Result<&str, &str> = executor.execute(|| {
            let _count = call_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Box::pin(async move {
                Err("Always fails")
            })
        }).await;
        
        assert_eq!(result, Err("Always fails"));
        assert_eq!(call_count.load(std::sync::atomic::Ordering::SeqCst), 2);
    }
}