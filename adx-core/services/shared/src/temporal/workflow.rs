use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::temporal::{WorkflowVersion, TemporalError, WorkflowError};

/// Workflow execution context for ADX Core
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Workflow execution ID
    pub workflow_id: String,
    
    /// Workflow run ID
    pub run_id: String,
    
    /// Workflow type
    pub workflow_type: String,
    
    /// Workflow version
    pub version: WorkflowVersion,
    
    /// Task queue name
    pub task_queue: String,
    
    /// Namespace
    pub namespace: String,
    
    /// User context
    pub user_context: UserContext,
    
    /// Tenant context
    pub tenant_context: TenantContext,
    
    /// Workflow metadata
    pub metadata: WorkflowMetadata,
    
    /// Search attributes
    pub search_attributes: HashMap<String, serde_json::Value>,
}

/// User context for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    /// User ID
    pub user_id: String,
    
    /// User email
    pub email: String,
    
    /// User roles in current tenant
    pub roles: Vec<String>,
    
    /// User permissions
    pub permissions: Vec<String>,
    
    /// Session ID
    pub session_id: Option<String>,
    
    /// Device information
    pub device_info: Option<DeviceInfo>,
}

/// Tenant context for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    /// Tenant ID
    pub tenant_id: String,
    
    /// Tenant name
    pub tenant_name: String,
    
    /// Subscription tier
    pub subscription_tier: SubscriptionTier,
    
    /// Available features
    pub features: Vec<String>,
    
    /// Tenant quotas
    pub quotas: TenantQuotas,
    
    /// Tenant settings
    pub settings: TenantSettings,
    
    /// Isolation level
    pub isolation_level: TenantIsolationLevel,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Device ID
    pub device_id: String,
    
    /// Device type (web, mobile, desktop)
    pub device_type: String,
    
    /// User agent
    pub user_agent: Option<String>,
    
    /// IP address
    pub ip_address: String,
}

/// Subscription tier
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SubscriptionTier {
    Free,
    Professional,
    Enterprise,
    Custom,
}

/// Tenant quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    /// Maximum users
    pub max_users: u32,
    
    /// Maximum storage in GB
    pub max_storage_gb: u32,
    
    /// Maximum API calls per hour
    pub max_api_calls_per_hour: u32,
    
    /// Maximum concurrent workflows
    pub max_concurrent_workflows: u32,
    
    /// Maximum file upload size in MB
    pub max_file_upload_size_mb: u32,
}

/// Tenant settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    /// Default language
    pub default_language: String,
    
    /// Timezone
    pub timezone: String,
    
    /// Date format
    pub date_format: String,
    
    /// Currency
    pub currency: String,
    
    /// Custom branding
    pub branding: Option<TenantBranding>,
}

/// Tenant branding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    /// Logo URL
    pub logo_url: Option<String>,
    
    /// Primary color
    pub primary_color: Option<String>,
    
    /// Secondary color
    pub secondary_color: Option<String>,
    
    /// Custom domain
    pub custom_domain: Option<String>,
}

/// Tenant isolation level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TenantIsolationLevel {
    /// Row-level security
    Row,
    
    /// Schema-level isolation
    Schema,
    
    /// Database-level isolation
    Database,
}

/// Workflow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    /// Workflow start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// Workflow timeout
    pub timeout: Duration,
    
    /// Retry policy
    pub retry_policy: Option<WorkflowRetryPolicy>,
    
    /// Parent workflow ID (if child workflow)
    pub parent_workflow_id: Option<String>,
    
    /// Correlation ID for tracking
    pub correlation_id: Option<String>,
    
    /// Business process identifier
    pub business_process: Option<String>,
    
    /// Priority level
    pub priority: WorkflowPriority,
    
    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Workflow retry policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowRetryPolicy {
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

/// Workflow priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum WorkflowPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Workflow execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionRequest<T> {
    /// Workflow type
    pub workflow_type: String,
    
    /// Workflow ID (optional, will be generated if not provided)
    pub workflow_id: Option<String>,
    
    /// Workflow version (optional, will use default if not provided)
    pub version: Option<WorkflowVersion>,
    
    /// Task queue
    pub task_queue: String,
    
    /// Workflow input
    pub input: T,
    
    /// User context
    pub user_context: UserContext,
    
    /// Tenant context
    pub tenant_context: TenantContext,
    
    /// Workflow options
    pub options: WorkflowExecutionOptions,
}

/// Workflow execution options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionOptions {
    /// Workflow timeout
    pub timeout: Option<Duration>,
    
    /// Retry policy
    pub retry_policy: Option<WorkflowRetryPolicy>,
    
    /// Priority
    pub priority: WorkflowPriority,
    
    /// Correlation ID
    pub correlation_id: Option<String>,
    
    /// Business process
    pub business_process: Option<String>,
    
    /// Tags
    pub tags: Vec<String>,
    
    /// Search attributes
    pub search_attributes: HashMap<String, serde_json::Value>,
    
    /// Memo
    pub memo: HashMap<String, String>,
    
    /// Cron schedule (for scheduled workflows)
    pub cron_schedule: Option<String>,
}

impl Default for WorkflowExecutionOptions {
    fn default() -> Self {
        Self {
            timeout: Some(Duration::from_secs(3600)), // 1 hour default
            retry_policy: None,
            priority: WorkflowPriority::Normal,
            correlation_id: None,
            business_process: None,
            tags: Vec::new(),
            search_attributes: HashMap::new(),
            memo: HashMap::new(),
            cron_schedule: None,
        }
    }
}

/// Workflow execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionResult<T> {
    /// Workflow ID
    pub workflow_id: String,
    
    /// Run ID
    pub run_id: String,
    
    /// Execution status
    pub status: WorkflowExecutionStatus,
    
    /// Result data (if completed successfully)
    pub result: Option<T>,
    
    /// Error (if failed)
    pub error: Option<WorkflowError>,
    
    /// Execution time
    pub execution_time: Duration,
    
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    
    /// End time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum WorkflowExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Terminated,
    ContinuedAsNew,
    TimedOut,
}

/// Workflow builder for creating workflow execution requests
pub struct WorkflowBuilder<T> {
    workflow_type: String,
    workflow_id: Option<String>,
    version: Option<WorkflowVersion>,
    task_queue: String,
    input: Option<T>,
    user_context: Option<UserContext>,
    tenant_context: Option<TenantContext>,
    options: WorkflowExecutionOptions,
}

impl<T> WorkflowBuilder<T>
where
    T: Serialize,
{
    /// Create a new workflow builder
    pub fn new(workflow_type: &str) -> Self {
        Self {
            workflow_type: workflow_type.to_string(),
            workflow_id: None,
            version: None,
            task_queue: "default".to_string(),
            input: None,
            user_context: None,
            tenant_context: None,
            options: WorkflowExecutionOptions::default(),
        }
    }
    
    /// Set workflow ID
    pub fn workflow_id(mut self, workflow_id: String) -> Self {
        self.workflow_id = Some(workflow_id);
        self
    }
    
    /// Generate a unique workflow ID
    pub fn generate_workflow_id(mut self) -> Self {
        self.workflow_id = Some(format!("{}-{}", self.workflow_type, Uuid::new_v4()));
        self
    }
    
    /// Set workflow version
    pub fn version(mut self, version: WorkflowVersion) -> Self {
        self.version = Some(version);
        self
    }
    
    /// Set task queue
    pub fn task_queue(mut self, task_queue: &str) -> Self {
        self.task_queue = task_queue.to_string();
        self
    }
    
    /// Set workflow input
    pub fn input(mut self, input: T) -> Self {
        self.input = Some(input);
        self
    }
    
    /// Set user context
    pub fn user_context(mut self, user_context: UserContext) -> Self {
        self.user_context = Some(user_context);
        self
    }
    
    /// Set tenant context
    pub fn tenant_context(mut self, tenant_context: TenantContext) -> Self {
        self.tenant_context = Some(tenant_context);
        self
    }
    
    /// Set workflow timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.options.timeout = Some(timeout);
        self
    }
    
    /// Set workflow priority
    pub fn priority(mut self, priority: WorkflowPriority) -> Self {
        self.options.priority = priority;
        self
    }
    
    /// Set correlation ID
    pub fn correlation_id(mut self, correlation_id: String) -> Self {
        self.options.correlation_id = Some(correlation_id);
        self
    }
    
    /// Set business process
    pub fn business_process(mut self, business_process: String) -> Self {
        self.options.business_process = Some(business_process);
        self
    }
    
    /// Add tag
    pub fn tag(mut self, tag: String) -> Self {
        self.options.tags.push(tag);
        self
    }
    
    /// Add search attribute
    pub fn search_attribute(mut self, key: String, value: serde_json::Value) -> Self {
        self.options.search_attributes.insert(key, value);
        self
    }
    
    /// Add memo
    pub fn memo(mut self, key: String, value: String) -> Self {
        self.options.memo.insert(key, value);
        self
    }
    
    /// Set cron schedule
    pub fn cron_schedule(mut self, cron_schedule: String) -> Self {
        self.options.cron_schedule = Some(cron_schedule);
        self
    }
    
    /// Build the workflow execution request
    pub fn build(self) -> Result<WorkflowExecutionRequest<T>, TemporalError> {
        let input = self.input.ok_or_else(|| TemporalError::ConfigurationError {
            message: "Workflow input is required".to_string(),
        })?;
        
        let user_context = self.user_context.ok_or_else(|| TemporalError::ConfigurationError {
            message: "User context is required".to_string(),
        })?;
        
        let tenant_context = self.tenant_context.ok_or_else(|| TemporalError::ConfigurationError {
            message: "Tenant context is required".to_string(),
        })?;
        
        Ok(WorkflowExecutionRequest {
            workflow_type: self.workflow_type,
            workflow_id: self.workflow_id,
            version: self.version,
            task_queue: self.task_queue,
            input,
            user_context,
            tenant_context,
            options: self.options,
        })
    }
}

/// Utility functions for workflow management
pub mod utils {
    use super::*;
    
    /// Generate a unique workflow ID with prefix
    pub fn generate_workflow_id(prefix: &str) -> String {
        format!("{}-{}", prefix, Uuid::new_v4())
    }
    
    /// Generate a correlation ID
    pub fn generate_correlation_id() -> String {
        Uuid::new_v4().to_string()
    }
    
    /// Create search attributes for tenant-aware workflows
    pub fn create_tenant_search_attributes(
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> HashMap<String, serde_json::Value> {
        let mut attributes = HashMap::new();
        
        attributes.insert("TenantId".to_string(), serde_json::Value::String(tenant_context.tenant_id.clone()));
        attributes.insert("UserId".to_string(), serde_json::Value::String(user_context.user_id.clone()));
        attributes.insert("SubscriptionTier".to_string(), serde_json::Value::String(
            match tenant_context.subscription_tier {
                SubscriptionTier::Free => "Free".to_string(),
                SubscriptionTier::Professional => "Professional".to_string(),
                SubscriptionTier::Enterprise => "Enterprise".to_string(),
                SubscriptionTier::Custom => "Custom".to_string(),
            }
        ));
        
        attributes
    }
    
    /// Validate workflow context
    pub fn validate_workflow_context(
        user_context: &UserContext,
        tenant_context: &TenantContext,
    ) -> Result<(), TemporalError> {
        // Validate user context
        if user_context.user_id.is_empty() {
            return Err(TemporalError::ConfigurationError {
                message: "User ID cannot be empty".to_string(),
            });
        }
        
        if user_context.email.is_empty() {
            return Err(TemporalError::ConfigurationError {
                message: "User email cannot be empty".to_string(),
            });
        }
        
        // Validate tenant context
        if tenant_context.tenant_id.is_empty() {
            return Err(TemporalError::ConfigurationError {
                message: "Tenant ID cannot be empty".to_string(),
            });
        }
        
        if tenant_context.tenant_name.is_empty() {
            return Err(TemporalError::ConfigurationError {
                message: "Tenant name cannot be empty".to_string(),
            });
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_builder() {
        let user_context = UserContext {
            user_id: "user123".to_string(),
            email: "user@example.com".to_string(),
            roles: vec!["user".to_string()],
            permissions: vec!["read".to_string()],
            session_id: None,
            device_info: None,
        };
        
        let tenant_context = TenantContext {
            tenant_id: "tenant123".to_string(),
            tenant_name: "Test Tenant".to_string(),
            subscription_tier: SubscriptionTier::Professional,
            features: vec!["feature1".to_string()],
            quotas: TenantQuotas {
                max_users: 100,
                max_storage_gb: 1000,
                max_api_calls_per_hour: 10000,
                max_concurrent_workflows: 50,
                max_file_upload_size_mb: 100,
            },
            settings: TenantSettings {
                default_language: "en".to_string(),
                timezone: "UTC".to_string(),
                date_format: "YYYY-MM-DD".to_string(),
                currency: "USD".to_string(),
                branding: None,
            },
            isolation_level: TenantIsolationLevel::Schema,
        };
        
        let request = WorkflowBuilder::new("test_workflow")
            .generate_workflow_id()
            .task_queue("test-queue")
            .input("test input")
            .user_context(user_context)
            .tenant_context(tenant_context)
            .priority(WorkflowPriority::High)
            .tag("test".to_string())
            .build()
            .unwrap();
        
        assert_eq!(request.workflow_type, "test_workflow");
        assert!(request.workflow_id.is_some());
        assert_eq!(request.task_queue, "test-queue");
        assert_eq!(request.input, "test input");
        assert_eq!(request.options.priority, WorkflowPriority::High);
        assert_eq!(request.options.tags, vec!["test".to_string()]);
    }
    
    #[test]
    fn test_search_attributes_creation() {
        let user_context = UserContext {
            user_id: "user123".to_string(),
            email: "user@example.com".to_string(),
            roles: vec![],
            permissions: vec![],
            session_id: None,
            device_info: None,
        };
        
        let tenant_context = TenantContext {
            tenant_id: "tenant123".to_string(),
            tenant_name: "Test Tenant".to_string(),
            subscription_tier: SubscriptionTier::Enterprise,
            features: vec![],
            quotas: TenantQuotas {
                max_users: 100,
                max_storage_gb: 1000,
                max_api_calls_per_hour: 10000,
                max_concurrent_workflows: 50,
                max_file_upload_size_mb: 100,
            },
            settings: TenantSettings {
                default_language: "en".to_string(),
                timezone: "UTC".to_string(),
                date_format: "YYYY-MM-DD".to_string(),
                currency: "USD".to_string(),
                branding: None,
            },
            isolation_level: TenantIsolationLevel::Schema,
        };
        
        let attributes = utils::create_tenant_search_attributes(&tenant_context, &user_context);
        
        assert_eq!(attributes.get("TenantId"), Some(&serde_json::Value::String("tenant123".to_string())));
        assert_eq!(attributes.get("UserId"), Some(&serde_json::Value::String("user123".to_string())));
        assert_eq!(attributes.get("SubscriptionTier"), Some(&serde_json::Value::String("Enterprise".to_string())));
    }
}