use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use temporal_sdk::{WfContext, ActContext};
use crate::{Error, Result, TenantContext, UserContext};

// Re-export temporal types for convenience
pub use temporal_sdk::{WfContext as WorkflowContext, ActContext as ActivityContext};

pub struct TemporalClient {
    // This would be the actual Temporal client in a real implementation
    // For now, we'll use a placeholder structure
    server_url: String,
    namespace: String,
}

impl TemporalClient {
    pub fn new(server_url: String, namespace: String) -> Self {
        Self {
            server_url,
            namespace,
        }
    }
    
    pub async fn start_workflow<T: Serialize>(
        &self,
        workflow_type: &str,
        workflow_id: String,
        task_queue: &str,
        input: T,
    ) -> Result<WorkflowHandle> {
        // In a real implementation, this would start a Temporal workflow
        // For now, return a mock handle
        Ok(WorkflowHandle {
            workflow_id,
            run_id: uuid::Uuid::new_v4().to_string(),
        })
    }
    
    pub async fn get_workflow_status(&self, workflow_id: &str) -> Result<WorkflowExecutionInfo> {
        // Mock implementation
        Ok(WorkflowExecutionInfo {
            workflow_id: workflow_id.to_string(),
            run_id: uuid::Uuid::new_v4().to_string(),
            status: crate::types::WorkflowStatus::Running,
            result: None,
            error: None,
        })
    }
}

pub struct WorkflowHandle {
    pub workflow_id: String,
    pub run_id: String,
}

impl WorkflowHandle {
    pub async fn get_result<T: for<'de> Deserialize<'de>>(&self) -> Result<T> {
        // Mock implementation - would wait for workflow completion
        Err(Error::Temporal("Not implemented".to_string()))
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowExecutionInfo {
    pub workflow_id: String,
    pub run_id: String,
    pub status: crate::types::WorkflowStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

// Workflow context with tenant and user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdxWorkflowContext {
    pub tenant_context: TenantContext,
    pub user_context: UserContext,
    pub workflow_id: String,
    pub run_id: String,
    pub metadata: HashMap<String, String>,
}

// Activity trait for ADX Core activities
#[async_trait]
pub trait AdxActivity {
    type Input: for<'de> Deserialize<'de> + Send + Sync;
    type Output: Serialize + Send + Sync;
    
    async fn execute(
        &self,
        ctx: &ActivityContext,
        input: Self::Input,
    ) -> Result<Self::Output>;
    
    fn activity_name(&self) -> &'static str;
    
    fn retry_policy(&self) -> RetryPolicy {
        RetryPolicy::default()
    }
}

// Workflow trait for ADX Core workflows
#[async_trait]
pub trait AdxWorkflow {
    type Input: for<'de> Deserialize<'de> + Send + Sync;
    type Output: Serialize + Send + Sync;
    
    async fn execute(
        &self,
        ctx: &WorkflowContext,
        input: Self::Input,
    ) -> Result<Self::Output>;
    
    fn workflow_name(&self) -> &'static str;
    
    fn task_queue(&self) -> &'static str {
        "adx-core-task-queue"
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_interval_seconds: u64,
    pub backoff_coefficient: f64,
    pub maximum_interval_seconds: u64,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_interval_seconds: 1,
            backoff_coefficient: 2.0,
            maximum_interval_seconds: 60,
        }
    }
}

// Utility functions for workflow execution
pub async fn call_activity<A, I, O>(
    ctx: &WorkflowContext,
    activity: A,
    input: I,
) -> Result<O>
where
    A: AdxActivity<Input = I, Output = O>,
    I: Serialize + for<'de> Deserialize<'de> + Send + Sync,
    O: for<'de> Deserialize<'de> + Send + Sync,
{
    // Mock implementation - would call actual Temporal activity
    Err(Error::Temporal("Activity execution not implemented".to_string()))
}

pub async fn spawn_workflow<W, I, O>(
    workflow: W,
    input: I,
) -> Result<WorkflowHandle>
where
    W: AdxWorkflow<Input = I, Output = O>,
    I: Serialize + Send + Sync,
    O: for<'de> Deserialize<'de> + Send + Sync,
{
    // Mock implementation - would spawn child workflow
    Ok(WorkflowHandle {
        workflow_id: uuid::Uuid::new_v4().to_string(),
        run_id: uuid::Uuid::new_v4().to_string(),
    })
}