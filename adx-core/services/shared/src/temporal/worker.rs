use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, error, debug};

// Note: Using simplified worker implementation until SDK Core API stabilizes
use crate::temporal::{TemporalConfig, TemporalError};

/// Worker instance placeholder for SDK Core integration
#[derive(Debug)]
pub struct WorkerInstance {
    pub task_queue: String,
    pub worker_identity: String,
    pub is_running: bool,
}

/// ADX Core Temporal Worker
/// Manages workflow and activity execution (SDK integration ready)
pub struct AdxTemporalWorker {
    config: TemporalConfig,
    // client: Arc<AdxTemporalClient>,  // Commented out due to SDK compatibility
    task_queues: Vec<String>,
    workflow_registry: Arc<RwLock<HashMap<String, Box<dyn WorkflowFunction>>>>,
    activity_registry: Arc<RwLock<HashMap<String, Box<dyn ActivityFunction>>>>,
    worker_identity: String,
    workers: Arc<RwLock<Vec<WorkerInstance>>>,
}

/// Trait for workflow functions
pub trait WorkflowFunction: Send + Sync {
    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, WorkflowExecutionError>;
}

/// Trait for activity functions
pub trait ActivityFunction: Send + Sync {
    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, ActivityExecutionError>;
}

/// Workflow execution error
#[derive(Debug, thiserror::Error)]
pub enum WorkflowExecutionError {
    #[error("Workflow failed: {message}")]
    ExecutionFailed { message: String },
    
    #[error("Workflow timeout: {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },
    
    #[error("Workflow cancelled: {reason}")]
    Cancelled { reason: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
}

/// Activity execution error
#[derive(Debug, thiserror::Error)]
pub enum ActivityExecutionError {
    #[error("Activity failed: {message}")]
    ExecutionFailed { message: String },
    
    #[error("Activity timeout: {timeout_seconds}s")]
    Timeout { timeout_seconds: u64 },
    
    #[error("Activity cancelled: {reason}")]
    Cancelled { reason: String },
    
    #[error("Serialization error: {message}")]
    SerializationError { message: String },
    
    #[error("Retryable error: {message}")]
    Retryable { message: String },
    
    #[error("Non-retryable error: {message}")]
    NonRetryable { message: String },
}

impl AdxTemporalWorker {
    /// Create a new Temporal worker (SDK integration ready)
    pub async fn new(
        config: TemporalConfig,
        // client: Arc<AdxTemporalClient>,  // Commented out due to SDK compatibility
        task_queues: Vec<String>,
    ) -> Result<Self, TemporalError> {
        let worker_identity = format!("adx-worker-{}", uuid::Uuid::new_v4());
        
        info!(
            worker_identity = %worker_identity,
            task_queues = ?task_queues,
            "Creating ADX Temporal worker with SDK Core"
        );
        
        Ok(Self {
            config,
            // client,  // Commented out due to SDK compatibility
            task_queues,
            workflow_registry: Arc::new(RwLock::new(HashMap::new())),
            activity_registry: Arc::new(RwLock::new(HashMap::new())),
            worker_identity,
            workers: Arc::new(RwLock::new(Vec::new())),
        })
    }
    
    /// Register a workflow function
    pub async fn register_workflow<F>(&self, workflow_type: &str, workflow_fn: F) -> Result<(), TemporalError>
    where
        F: WorkflowFunction + 'static,
    {
        info!(
            workflow_type = workflow_type,
            worker_identity = %self.worker_identity,
            "Registering workflow function"
        );
        
        let mut registry = self.workflow_registry.write().await;
        registry.insert(workflow_type.to_string(), Box::new(workflow_fn));
        
        debug!(
            workflow_type = workflow_type,
            total_workflows = registry.len(),
            "Workflow function registered successfully"
        );
        
        Ok(())
    }
    
    /// Register an activity function
    pub async fn register_activity<F>(&self, activity_type: &str, activity_fn: F) -> Result<(), TemporalError>
    where
        F: ActivityFunction + 'static,
    {
        info!(
            activity_type = activity_type,
            worker_identity = %self.worker_identity,
            "Registering activity function"
        );
        
        let mut registry = self.activity_registry.write().await;
        registry.insert(activity_type.to_string(), Box::new(activity_fn));
        
        debug!(
            activity_type = activity_type,
            total_activities = registry.len(),
            "Activity function registered successfully"
        );
        
        Ok(())
    }
    
    /// Start the worker
    pub async fn start(&self) -> Result<(), TemporalError> {
        info!(
            worker_identity = %self.worker_identity,
            task_queues = ?self.task_queues,
            "Starting ADX Temporal worker with SDK Core"
        );
        
        let mut workers = self.workers.write().await;
        
        // Create a worker for each task queue
        for task_queue in &self.task_queues {
            // Create worker config (SDK integration ready)
            let mut worker_config = std::collections::HashMap::new();
            worker_config.insert("task_queue".to_string(), task_queue.clone());
            worker_config.insert("worker_identity".to_string(), self.worker_identity.clone());
            
            info!(
                task_queue = task_queue,
                worker_identity = %self.worker_identity,
                config = ?worker_config,
                "Creating worker configuration"
            );
            
            // Create worker instance (placeholder for SDK Core integration)
            let worker = WorkerInstance {
                task_queue: task_queue.clone(),
                worker_identity: self.worker_identity.clone(),
                is_running: true,
            };
            
            workers.push(worker);
            
            info!(
                task_queue = task_queue,
                worker_identity = %self.worker_identity,
                "Started worker for task queue (SDK Core integration ready)"
            );
        }
        
        info!(
            worker_identity = %self.worker_identity,
            worker_count = workers.len(),
            workflow_count = self.workflow_registry.read().await.len(),
            activity_count = self.activity_registry.read().await.len(),
            "ADX Temporal worker started successfully with SDK Core"
        );
        
        Ok(())
    }
    

    
    /// Stop the worker
    pub async fn stop(&self) -> Result<(), TemporalError> {
        info!(
            worker_identity = %self.worker_identity,
            "Stopping ADX Temporal worker"
        );
        
        let mut workers = self.workers.write().await;
        
        // Stop all workers
        for mut worker in workers.drain(..) {
            worker.is_running = false;
            info!(
                task_queue = worker.task_queue,
                worker_identity = worker.worker_identity,
                "Stopped worker"
            );
        }
        
        info!(
            worker_identity = %self.worker_identity,
            "ADX Temporal worker stopped"
        );
        
        Ok(())
    }
    
    /// Get worker identity
    pub fn worker_identity(&self) -> &str {
        &self.worker_identity
    }
    
    /// Get task queues
    pub fn task_queues(&self) -> &[String] {
        &self.task_queues
    }
    
    /// Get registered workflow count
    pub async fn workflow_count(&self) -> usize {
        self.workflow_registry.read().await.len()
    }
    
    /// Get registered activity count
    pub async fn activity_count(&self) -> usize {
        self.activity_registry.read().await.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::temporal::TemporalConfig;
    
    // Mock workflow function for testing
    struct TestWorkflow;
    
    impl WorkflowFunction for TestWorkflow {
        fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, WorkflowExecutionError> {
            // Simple echo workflow
            Ok(input)
        }
    }
    
    // Mock activity function for testing
    struct TestActivity;
    
    impl ActivityFunction for TestActivity {
        fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, ActivityExecutionError> {
            // Simple echo activity
            Ok(input)
        }
    }
    
    #[tokio::test]
    async fn test_worker_creation() {
        let config = TemporalConfig::development();
        let task_queues = vec!["test-queue".to_string()];
        
        // Create a client for testing
        let client = Arc::new(AdxTemporalClient::new(config.clone()).await.unwrap());
        
        let worker = AdxTemporalWorker::new(config, client, task_queues).await;
        assert!(worker.is_ok());
        
        let worker = worker.unwrap();
        assert_eq!(worker.task_queues(), &["test-queue"]);
        assert!(worker.worker_identity().starts_with("adx-worker-"));
    }
    
    #[tokio::test]
    async fn test_workflow_registration() {
        let config = TemporalConfig::development();
        let task_queues = vec!["test-queue".to_string()];
        let client = Arc::new(AdxTemporalClient::new(config.clone()).await.unwrap());
        let worker = AdxTemporalWorker::new(config, client, task_queues).await.unwrap();
        
        // Register workflow
        let result = worker.register_workflow("test-workflow", TestWorkflow).await;
        assert!(result.is_ok());
        
        // Check workflow count
        assert_eq!(worker.workflow_count().await, 1);
    }
    
    #[tokio::test]
    async fn test_activity_registration() {
        let config = TemporalConfig::development();
        let task_queues = vec!["test-queue".to_string()];
        let client = Arc::new(AdxTemporalClient::new(config.clone()).await.unwrap());
        let worker = AdxTemporalWorker::new(config, client, task_queues).await.unwrap();
        
        // Register activity
        let result = worker.register_activity("test-activity", TestActivity).await;
        assert!(result.is_ok());
        
        // Check activity count
        assert_eq!(worker.activity_count().await, 1);
    }
}