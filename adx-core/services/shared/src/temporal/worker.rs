use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{info, error, debug};
use async_trait::async_trait;

use crate::temporal::{TemporalConfig, TemporalError};
use crate::temporal::sdk_client::{TemporalSDKClient, TemporalWorker, WorkerConfig};

/// Worker instance placeholder for SDK Core integration
#[derive(Debug)]
pub struct WorkerInstance {
    pub task_queue: String,
    pub worker_identity: String,
    pub is_running: bool,
}

/// ADX Core Temporal Worker Manager
/// Manages workflow and activity execution with real SDK integration
pub struct AdxTemporalWorkerManager {
    config: TemporalConfig,
    sdk_client: Arc<TemporalSDKClient>,
    task_queues: Vec<String>,
    workflow_registry: Arc<RwLock<HashMap<String, Box<dyn WorkflowFunction>>>>,
    activity_registry: Arc<RwLock<HashMap<String, Box<dyn ActivityFunction>>>>,
    worker_identity: String,
    workers: Arc<RwLock<HashMap<String, Arc<TemporalWorker>>>>,
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

impl AdxTemporalWorkerManager {
    /// Create a new Temporal worker manager with SDK integration
    pub async fn new(
        config: TemporalConfig,
        task_queues: Vec<String>,
    ) -> Result<Self, TemporalError> {
        let worker_identity = format!("adx-worker-{}", uuid::Uuid::new_v4());
        
        info!(
            worker_identity = %worker_identity,
            task_queues = ?task_queues,
            "Creating ADX Temporal worker manager with SDK integration"
        );
        
        // Create SDK client
        let sdk_client = Arc::new(TemporalSDKClient::new(config.clone()).await?);
        
        Ok(Self {
            config,
            sdk_client,
            task_queues,
            workflow_registry: Arc::new(RwLock::new(HashMap::new())),
            activity_registry: Arc::new(RwLock::new(HashMap::new())),
            worker_identity,
            workers: Arc::new(RwLock::new(HashMap::new())),
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
    
    /// Start the worker manager
    pub async fn start(&self) -> Result<(), TemporalError> {
        info!(
            worker_identity = %self.worker_identity,
            task_queues = ?self.task_queues,
            "Starting ADX Temporal worker manager with SDK integration"
        );
        
        let mut workers = self.workers.write().await;
        
        // Create a worker for each task queue using the SDK client
        for task_queue in &self.task_queues {
            info!(
                task_queue = task_queue,
                worker_identity = %self.worker_identity,
                "Creating SDK worker for task queue"
            );
            
            // Create worker using SDK client
            let worker = self.sdk_client.create_worker(task_queue).await?;
            let worker_arc = Arc::new(worker);
            
            // Start the worker
            worker_arc.start().await?;
            
            workers.insert(task_queue.clone(), worker_arc);
            
            info!(
                task_queue = task_queue,
                worker_identity = %self.worker_identity,
                "Started SDK worker for task queue"
            );
        }
        
        info!(
            worker_identity = %self.worker_identity,
            worker_count = workers.len(),
            workflow_count = self.workflow_registry.read().await.len(),
            activity_count = self.activity_registry.read().await.len(),
            "ADX Temporal worker manager started successfully with SDK integration"
        );
        
        Ok(())
    }
    

    
    /// Stop the worker manager
    pub async fn stop(&self) -> Result<(), TemporalError> {
        info!(
            worker_identity = %self.worker_identity,
            "Stopping ADX Temporal worker manager"
        );
        
        let mut workers = self.workers.write().await;
        
        // Stop all workers
        for (task_queue, worker) in workers.drain() {
            worker.stop().await?;
            info!(
                task_queue = task_queue,
                worker_identity = %self.worker_identity,
                "Stopped SDK worker"
            );
        }
        
        info!(
            worker_identity = %self.worker_identity,
            "ADX Temporal worker manager stopped"
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
    async fn test_worker_manager_creation() {
        let config = TemporalConfig::development();
        let task_queues = vec!["test-queue".to_string()];
        
        let worker_manager = AdxTemporalWorkerManager::new(config, task_queues).await;
        assert!(worker_manager.is_ok());
        
        let worker_manager = worker_manager.unwrap();
        assert_eq!(worker_manager.task_queues(), &["test-queue"]);
        assert!(worker_manager.worker_identity().starts_with("adx-worker-"));
    }
    
    #[tokio::test]
    async fn test_workflow_registration() {
        let config = TemporalConfig::development();
        let task_queues = vec!["test-queue".to_string()];
        let worker_manager = AdxTemporalWorkerManager::new(config, task_queues).await.unwrap();
        
        // Register workflow
        let result = worker_manager.register_workflow("test-workflow", TestWorkflow).await;
        assert!(result.is_ok());
        
        // Check workflow count
        assert_eq!(worker_manager.workflow_count().await, 1);
    }
    
    #[tokio::test]
    async fn test_activity_registration() {
        let config = TemporalConfig::development();
        let task_queues = vec!["test-queue".to_string()];
        let worker_manager = AdxTemporalWorkerManager::new(config, task_queues).await.unwrap();
        
        // Register activity
        let result = worker_manager.register_activity("test-activity", TestActivity).await;
        assert!(result.is_ok());
        
        // Check activity count
        assert_eq!(worker_manager.activity_count().await, 1);
    }
}