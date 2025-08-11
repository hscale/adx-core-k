use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::temporal::{ActivityError, TenantContext, UserContext};

/// Activity execution context for ADX Core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityContext {
    /// Activity ID
    pub activity_id: String,
    
    /// Activity type
    pub activity_type: String,
    
    /// Workflow ID that started this activity
    pub workflow_id: String,
    
    /// Workflow run ID
    pub workflow_run_id: String,
    
    /// Activity attempt number
    pub attempt: u32,
    
    /// User context
    pub user_context: UserContext,
    
    /// Tenant context
    pub tenant_context: TenantContext,
    
    /// Activity metadata
    pub metadata: ActivityMetadata,
    
    /// Heartbeat details
    pub heartbeat_details: Option<serde_json::Value>,
}

/// Activity metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityMetadata {
    /// Activity start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// Activity timeout
    pub timeout: Duration,
    
    /// Heartbeat timeout
    pub heartbeat_timeout: Option<Duration>,
    
    /// Retry policy
    pub retry_policy: Option<ActivityRetryPolicy>,
    
    /// Activity tags
    pub tags: Vec<String>,
    
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

/// Activity retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityRetryPolicy {
    /// Initial retry interval
    pub initial_interval: Duration,
    
    /// Maximum retry interval
    pub max_interval: Duration,
    
    /// Backoff coefficient
    pub backoff_coefficient: f64,
    
    /// Maximum attempts
    pub max_attempts: u32,
    
    /// Non-retryable error types
    pub non_retryable_errors: Vec<String>,
}

/// Activity execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityExecutionRequest<T> {
    /// Activity type
    pub activity_type: String,
    
    /// Activity input
    pub input: T,
    
    /// Activity options
    pub options: ActivityExecutionOptions,
}

/// Activity execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityExecutionOptions {
    /// Start to close timeout
    pub start_to_close_timeout: Option<Duration>,
    
    /// Schedule to start timeout
    pub schedule_to_start_timeout: Option<Duration>,
    
    /// Schedule to close timeout
    pub schedule_to_close_timeout: Option<Duration>,
    
    /// Heartbeat timeout
    pub heartbeat_timeout: Option<Duration>,
    
    /// Retry policy
    pub retry_policy: Option<ActivityRetryPolicy>,
    
    /// Activity tags
    pub tags: Vec<String>,
    
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ActivityExecutionOptions {
    fn default() -> Self {
        Self {
            start_to_close_timeout: Some(Duration::from_secs(300)), // 5 minutes
            schedule_to_start_timeout: Some(Duration::from_secs(60)), // 1 minute
            schedule_to_close_timeout: Some(Duration::from_secs(360)), // 6 minutes
            heartbeat_timeout: Some(Duration::from_secs(30)),
            retry_policy: None,
            tags: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}

/// Activity execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityExecutionResult<T> {
    /// Activity ID
    pub activity_id: String,
    
    /// Execution status
    pub status: ActivityExecutionStatus,
    
    /// Result data (if completed successfully)
    pub result: Option<T>,
    
    /// Error (if failed)
    pub error: Option<ActivityError>,
    
    /// Execution time
    pub execution_time: Duration,
    
    /// Attempt number
    pub attempt: u32,
    
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// End time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Activity execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ActivityExecutionStatus {
    Scheduled,
    Started,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

/// Base trait for ADX Core activities
pub trait AdxActivity<Input, Output>: Send + Sync
where
    Input: for<'de> Deserialize<'de> + Send + Sync,
    Output: Serialize + Send + Sync,
{
    /// Execute the activity
    async fn execute(
        &self,
        context: ActivityContext,
        input: Input,
    ) -> Result<Output, ActivityError>;
    
    /// Get activity type name
    fn activity_type(&self) -> &'static str;
    
    /// Get default execution options
    fn default_options(&self) -> ActivityExecutionOptions {
        ActivityExecutionOptions::default()
    }
    
    /// Validate input before execution
    fn validate_input(&self, input: &Input) -> Result<(), ActivityError> {
        // Default implementation does no validation
        Ok(())
    }
    
    /// Handle activity heartbeat
    async fn heartbeat(&self, context: &ActivityContext, details: Option<serde_json::Value>) -> Result<(), ActivityError> {
        // Default implementation does nothing
        Ok(())
    }
    
    /// Handle activity cancellation
    async fn cancel(&self, context: &ActivityContext) -> Result<(), ActivityError> {
        // Default implementation does nothing
        Ok(())
    }
}

/// Tenant-aware activity trait
pub trait TenantAwareActivity<Input, Output>: AdxActivity<Input, Output>
where
    Input: for<'de> Deserialize<'de> + Send + Sync,
    Output: Serialize + Send + Sync,
{
    /// Validate tenant access before execution
    async fn validate_tenant_access(
        &self,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<(), ActivityError> {
        // Default implementation allows all access
        Ok(())
    }
    
    /// Check tenant quotas before execution
    async fn check_tenant_quotas(
        &self,
        tenant_context: &TenantContext,
        resource_type: &str,
        requested_amount: u64,
    ) -> Result<(), ActivityError> {
        // Default implementation does no quota checking
        Ok(())
    }
}

/// Database activity trait for activities that interact with databases
pub trait DatabaseActivity<Input, Output>: TenantAwareActivity<Input, Output>
where
    Input: for<'de> Deserialize<'de> + Send + Sync,
    Output: Serialize + Send + Sync,
{
    /// Get database connection for tenant
    async fn get_tenant_connection(
        &self,
        tenant_context: &TenantContext,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, ActivityError>;
    
    /// Execute database transaction
    async fn execute_transaction<F, R>(
        &self,
        tenant_context: &TenantContext,
        transaction: F,
    ) -> Result<R, ActivityError>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, ActivityError>> + Send>> + Send,
        R: Send + Sync;
}

/// External service activity trait for activities that call external services
pub trait ExternalServiceActivity<Input, Output>: AdxActivity<Input, Output>
where
    Input: for<'de> Deserialize<'de> + Send + Sync,
    Output: Serialize + Send + Sync,
{
    /// Get service endpoint URL
    fn get_service_endpoint(&self) -> &str;
    
    /// Get authentication headers
    async fn get_auth_headers(&self) -> Result<HashMap<String, String>, ActivityError>;
    
    /// Handle service rate limiting
    async fn handle_rate_limit(&self, retry_after: Duration) -> Result<(), ActivityError> {
        tokio::time::sleep(retry_after).await;
        Ok(())
    }
    
    /// Validate service response
    fn validate_response(&self, response: &serde_json::Value) -> Result<(), ActivityError> {
        // Default implementation does no validation
        Ok(())
    }
}

/// File operation activity trait
pub trait FileOperationActivity<Input, Output>: TenantAwareActivity<Input, Output>
where
    Input: for<'de> Deserialize<'de> + Send + Sync,
    Output: Serialize + Send + Sync,
{
    /// Get file storage provider
    async fn get_storage_provider(
        &self,
        tenant_context: &TenantContext,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, ActivityError>;
    
    /// Validate file permissions
    async fn validate_file_permissions(
        &self,
        user_context: &UserContext,
        tenant_context: &TenantContext,
        file_path: &str,
        operation: FileOperation,
    ) -> Result<(), ActivityError>;
    
    /// Scan file for viruses
    async fn scan_file(&self, file_path: &str) -> Result<ScanResult, ActivityError>;
}

/// File operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FileOperation {
    Read,
    Write,
    Delete,
    Share,
    Move,
    Copy,
}

/// Virus scan result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    pub is_clean: bool,
    pub threats: Vec<String>,
    pub scan_time: chrono::DateTime<chrono::Utc>,
}

/// Activity registry for managing activity implementations
pub struct ActivityRegistry {
    activities: HashMap<String, Box<dyn std::any::Any + Send + Sync>>,
}

impl ActivityRegistry {
    /// Create a new activity registry
    pub fn new() -> Self {
        Self {
            activities: HashMap::new(),
        }
    }
    
    /// Register an activity
    pub fn register<A, Input, Output>(&mut self, activity: A)
    where
        A: AdxActivity<Input, Output> + 'static,
        Input: for<'de> Deserialize<'de> + Send + Sync + 'static,
        Output: Serialize + Send + Sync + 'static,
    {
        let activity_type = activity.activity_type().to_string();
        self.activities.insert(activity_type, Box::new(activity));
    }
    
    /// Get activity by type
    pub fn get<A>(&self, activity_type: &str) -> Option<&A>
    where
        A: 'static,
    {
        self.activities.get(activity_type)
            .and_then(|activity| activity.downcast_ref::<A>())
    }
    
    /// Check if activity is registered
    pub fn is_registered(&self, activity_type: &str) -> bool {
        self.activities.contains_key(activity_type)
    }
    
    /// Get all registered activity types
    pub fn get_activity_types(&self) -> Vec<String> {
        self.activities.keys().cloned().collect()
    }
}

impl Default for ActivityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Activity builder for creating activity execution requests
pub struct ActivityBuilder<T> {
    activity_type: String,
    input: Option<T>,
    options: ActivityExecutionOptions,
}

impl<T> ActivityBuilder<T>
where
    T: Serialize,
{
    /// Create a new activity builder
    pub fn new(activity_type: &str) -> Self {
        Self {
            activity_type: activity_type.to_string(),
            input: None,
            options: ActivityExecutionOptions::default(),
        }
    }
    
    /// Set activity input
    pub fn input(mut self, input: T) -> Self {
        self.input = Some(input);
        self
    }
    
    /// Set start to close timeout
    pub fn start_to_close_timeout(mut self, timeout: Duration) -> Self {
        self.options.start_to_close_timeout = Some(timeout);
        self
    }
    
    /// Set schedule to start timeout
    pub fn schedule_to_start_timeout(mut self, timeout: Duration) -> Self {
        self.options.schedule_to_start_timeout = Some(timeout);
        self
    }
    
    /// Set heartbeat timeout
    pub fn heartbeat_timeout(mut self, timeout: Duration) -> Self {
        self.options.heartbeat_timeout = Some(timeout);
        self
    }
    
    /// Set retry policy
    pub fn retry_policy(mut self, retry_policy: ActivityRetryPolicy) -> Self {
        self.options.retry_policy = Some(retry_policy);
        self
    }
    
    /// Add tag
    pub fn tag(mut self, tag: String) -> Self {
        self.options.tags.push(tag);
        self
    }
    
    /// Add metadata
    pub fn metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.options.metadata.insert(key, value);
        self
    }
    
    /// Build the activity execution request
    pub fn build(self) -> Result<ActivityExecutionRequest<T>, ActivityError> {
        let input = self.input.ok_or_else(|| ActivityError::ConfigurationError {
            message: "Activity input is required".to_string(),
        })?;
        
        Ok(ActivityExecutionRequest {
            activity_type: self.activity_type,
            input,
            options: self.options,
        })
    }
}

/// Utility functions for activity management
pub mod utils {
    use super::*;
    use uuid::Uuid;
    
    /// Generate a unique activity ID
    pub fn generate_activity_id(activity_type: &str) -> String {
        format!("{}-{}", activity_type, Uuid::new_v4())
    }
    
    /// Create default retry policy for database activities
    pub fn database_retry_policy() -> ActivityRetryPolicy {
        ActivityRetryPolicy {
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(30),
            backoff_coefficient: 1.5,
            max_attempts: 5,
            non_retryable_errors: vec![
                "ValidationError".to_string(),
                "AuthorizationError".to_string(),
                "ConstraintViolationError".to_string(),
            ],
        }
    }
    
    /// Create default retry policy for external service activities
    pub fn external_service_retry_policy() -> ActivityRetryPolicy {
        ActivityRetryPolicy {
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
        }
    }
    
    /// Create default retry policy for file operations
    pub fn file_operation_retry_policy() -> ActivityRetryPolicy {
        ActivityRetryPolicy {
            initial_interval: Duration::from_millis(100),
            max_interval: Duration::from_secs(10),
            backoff_coefficient: 1.5,
            max_attempts: 3,
            non_retryable_errors: vec![
                "PermissionDeniedError".to_string(),
                "FileNotFoundError".to_string(),
                "InvalidPathError".to_string(),
            ],
        }
    }
    
    /// Validate activity context
    pub fn validate_activity_context(context: &ActivityContext) -> Result<(), ActivityError> {
        if context.activity_id.is_empty() {
            return Err(ActivityError::ConfigurationError {
                message: "Activity ID cannot be empty".to_string(),
            });
        }
        
        if context.activity_type.is_empty() {
            return Err(ActivityError::ConfigurationError {
                message: "Activity type cannot be empty".to_string(),
            });
        }
        
        if context.workflow_id.is_empty() {
            return Err(ActivityError::ConfigurationError {
                message: "Workflow ID cannot be empty".to_string(),
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Mock activity for testing
    struct MockActivity;
    
    impl AdxActivity<String, String> for MockActivity {
        async fn execute(
            &self,
            _context: ActivityContext,
            input: String,
        ) -> Result<String, ActivityError> {
            Ok(format!("Processed: {}", input))
        }
        
        fn activity_type(&self) -> &'static str {
            "mock_activity"
        }
    }
    
    #[test]
    fn test_activity_builder() {
        let request = ActivityBuilder::new("test_activity")
            .input("test input".to_string())
            .start_to_close_timeout(Duration::from_secs(60))
            .tag("test".to_string())
            .build()
            .unwrap();
        
        assert_eq!(request.activity_type, "test_activity");
        assert_eq!(request.input, "test input");
        assert_eq!(request.options.start_to_close_timeout, Some(Duration::from_secs(60)));
        assert_eq!(request.options.tags, vec!["test".to_string()]);
    }
    
    #[test]
    fn test_activity_registry() {
        let mut registry = ActivityRegistry::new();
        
        let activity = MockActivity;
        registry.register(activity);
        
        assert!(registry.is_registered("mock_activity"));
        assert!(!registry.is_registered("non_existent_activity"));
        
        let activity_types = registry.get_activity_types();
        assert!(activity_types.contains(&"mock_activity".to_string()));
    }
    
    #[test]
    fn test_retry_policy_creation() {
        let db_policy = utils::database_retry_policy();
        assert_eq!(db_policy.max_attempts, 5);
        assert_eq!(db_policy.backoff_coefficient, 1.5);
        
        let external_policy = utils::external_service_retry_policy();
        assert_eq!(external_policy.max_attempts, 4);
        assert_eq!(external_policy.backoff_coefficient, 2.0);
        
        let file_policy = utils::file_operation_retry_policy();
        assert_eq!(file_policy.max_attempts, 3);
        assert_eq!(file_policy.backoff_coefficient, 1.5);
    }
}