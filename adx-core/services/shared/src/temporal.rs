// Temporal workflow utilities and abstractions

use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::{Result, ServiceError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOptions {
    pub task_queue: String,
    pub workflow_id: String,
    pub workflow_timeout: Duration,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_interval: Duration,
    pub backoff_coefficient: f64,
    pub maximum_interval: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_interval: Duration::from_secs(1),
            backoff_coefficient: 2.0,
            maximum_interval: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    Terminated,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub workflow_id: String,
    pub run_id: String,
    pub status: WorkflowStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

// Mock Temporal client for testing
pub struct TemporalClient {
    server_url: String,
}

impl TemporalClient {
    pub fn new(server_url: &str) -> Self {
        Self {
            server_url: server_url.to_string(),
        }
    }
    
    pub async fn start_workflow<T: Serialize>(
        &self,
        workflow_type: &str,
        workflow_id: &str,
        task_queue: &str,
        input: T,
    ) -> Result<WorkflowExecution> {
        // Mock implementation for testing
        Ok(WorkflowExecution {
            workflow_id: workflow_id.to_string(),
            run_id: uuid::Uuid::new_v4().to_string(),
            status: WorkflowStatus::Running,
            result: None,
            error: None,
        })
    }
    
    pub async fn get_workflow_status(&self, workflow_id: &str) -> Result<WorkflowExecution> {
        // Mock implementation for testing
        Ok(WorkflowExecution {
            workflow_id: workflow_id.to_string(),
            run_id: uuid::Uuid::new_v4().to_string(),
            status: WorkflowStatus::Completed,
            result: Some(serde_json::json!({"success": true})),
            error: None,
        })
    }
    
    pub async fn cancel_workflow(&self, workflow_id: &str) -> Result<()> {
        // Mock implementation for testing
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_policy_default() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_attempts, 3);
        assert_eq!(policy.backoff_coefficient, 2.0);
    }

    #[tokio::test]
    async fn test_temporal_client_start_workflow() {
        let client = TemporalClient::new("localhost:7233");
        let execution = client
            .start_workflow(
                "test_workflow",
                "test-workflow-id",
                "test-queue",
                serde_json::json!({"test": "data"}),
            )
            .await
            .unwrap();
        
        assert_eq!(execution.workflow_id, "test-workflow-id");
        assert!(matches!(execution.status, WorkflowStatus::Running));
    }

    #[tokio::test]
    async fn test_temporal_client_get_status() {
        let client = TemporalClient::new("localhost:7233");
        let execution = client
            .get_workflow_status("test-workflow-id")
            .await
            .unwrap();
        
        assert_eq!(execution.workflow_id, "test-workflow-id");
        assert!(matches!(execution.status, WorkflowStatus::Completed));
        assert!(execution.result.is_some());
    }

    #[tokio::test]
    async fn test_temporal_client_cancel_workflow() {
        let client = TemporalClient::new("localhost:7233");
        let result = client.cancel_workflow("test-workflow-id").await;
        assert!(result.is_ok());
    }
}