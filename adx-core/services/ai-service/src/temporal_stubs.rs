// Temporary stubs for Temporal SDK types until the actual SDK is available
// This allows the AI service to compile and demonstrates the intended architecture

use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

// Temporal context stubs
#[derive(Debug, Clone)]
pub struct ActContext {
    pub activity_id: String,
    pub workflow_id: String,
}

#[derive(Debug, Clone)]
pub struct WfContext {
    pub workflow_id: String,
    pub run_id: String,
}

impl WfContext {
    pub fn workflow_info(&self) -> WorkflowInfo {
        WorkflowInfo {
            workflow_id: self.workflow_id.clone(),
            run_id: self.run_id.clone(),
        }
    }
    
    pub fn activity<T>(&self, _activities: T) -> ActivityStub {
        ActivityStub {}
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowInfo {
    pub workflow_id: String,
    pub run_id: String,
}

#[derive(Debug, Clone)]
pub struct ActivityStub {}

// Workflow result type
pub type WorkflowResult<T> = Result<T, WorkflowError>;

#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Activity failed: {0}")]
    ActivityFailed(String),
    #[error("Workflow timeout")]
    Timeout,
    #[error("Workflow cancelled")]
    Cancelled,
}

// Worker stubs
pub struct Worker {
    task_queue: String,
    workflows: Vec<String>,
    activities: Vec<String>,
}

pub struct WorkerBuilder {
    task_queue: Option<String>,
    worker_url: Option<String>,
}

impl Default for WorkerBuilder {
    fn default() -> Self {
        Self {
            task_queue: None,
            worker_url: None,
        }
    }
}

impl WorkerBuilder {
    pub fn task_queue(mut self, task_queue: &str) -> Self {
        self.task_queue = Some(task_queue.to_string());
        self
    }
    
    pub fn worker_url(mut self, url: &str) -> Self {
        self.worker_url = Some(url.to_string());
        self
    }
    
    pub async fn build(self) -> Result<Worker, String> {
        Ok(Worker {
            task_queue: self.task_queue.unwrap_or_else(|| "default".to_string()),
            workflows: Vec::new(),
            activities: Vec::new(),
        })
    }
}

impl Worker {
    pub fn register_wf<F, Req, Res>(&mut self, _workflow_fn: F)
    where
        F: Fn(WfContext, Req) -> Pin<Box<dyn Future<Output = WorkflowResult<Res>> + Send>> + Send + Sync + 'static,
        Req: for<'de> Deserialize<'de> + Send + 'static,
        Res: Serialize + Send + 'static,
    {
        // Stub implementation
    }
    
    pub fn register_activity<F, Req, Res>(&mut self, _name: &str, _activity_fn: F)
    where
        F: Fn(ActContext, Req) -> Pin<Box<dyn Future<Output = Result<Res, crate::error::ActivityError>> + Send>> + Send + Sync + 'static,
        Req: for<'de> Deserialize<'de> + Send + 'static,
        Res: Serialize + Send + 'static,
    {
        // Stub implementation
    }
    
    pub async fn run(self) -> Result<(), String> {
        // Stub implementation - would normally start the worker
        tracing::info!("Temporal worker started (stub implementation)");
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    }
}

// Macro stubs
pub use temporal_stubs_macros::*;

mod temporal_stubs_macros {
    // Stub workflow macro
    pub use temporal_workflow_derive::workflow;
    
    // Since we can't easily create a proc macro here, we'll use a simple attribute
    // In the real implementation, this would be a proper proc macro
}

// Temporary workflow attribute - in real implementation this would be a proc macro
pub use workflow_attribute::workflow;

mod workflow_attribute {
    pub fn workflow<F>(_f: F) -> F {
        _f
    }
}

// Activity implementations for the stubs
impl ActivityStub {
    pub async fn generate_text(&self, request: crate::types::TextGenerationRequest) -> Result<crate::types::TextGenerationResult, crate::error::ActivityError> {
        // Stub implementation
        Err(crate::error::ActivityError::ExternalServiceError("Temporal SDK not available".to_string()))
    }
    
    pub async fn classify_text(&self, request: crate::types::TextClassificationRequest) -> Result<crate::types::TextClassificationResult, crate::error::ActivityError> {
        // Stub implementation
        Err(crate::error::ActivityError::ExternalServiceError("Temporal SDK not available".to_string()))
    }
    
    pub async fn summarize_text(&self, request: crate::types::TextSummarizationRequest) -> Result<crate::types::TextSummarizationResult, crate::error::ActivityError> {
        // Stub implementation
        Err(crate::error::ActivityError::ExternalServiceError("Temporal SDK not available".to_string()))
    }
    
    pub async fn extract_entities(&self, request: crate::types::EntityExtractionRequest) -> Result<crate::types::EntityExtractionResult, crate::error::ActivityError> {
        // Stub implementation
        Err(crate::error::ActivityError::ExternalServiceError("Temporal SDK not available".to_string()))
    }
    
    pub async fn validate_ai_request(&self, request: crate::types::AIRequest) -> Result<crate::activities::ValidationResult, crate::error::ActivityError> {
        // Stub implementation
        Err(crate::error::ActivityError::ExternalServiceError("Temporal SDK not available".to_string()))
    }
    
    pub async fn track_ai_usage(&self, usage_record: crate::types::AIUsageRecord) -> Result<(), crate::error::ActivityError> {
        // Stub implementation
        Err(crate::error::ActivityError::ExternalServiceError("Temporal SDK not available".to_string()))
    }
    
    pub async fn check_ai_quotas(&self, context: crate::types::RequestContext, capability: crate::types::AICapability) -> Result<crate::activities::QuotaCheckResult, crate::error::ActivityError> {
        // Stub implementation
        Err(crate::error::ActivityError::ExternalServiceError("Temporal SDK not available".to_string()))
    }
}