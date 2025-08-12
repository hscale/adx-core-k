use std::sync::Arc;
use std::time::Duration;
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

use crate::config::TemporalConfig;
use crate::error::{ApiGatewayError, ApiResult};

/// Temporal client wrapper for API Gateway
/// Provides workflow orchestration capabilities with intelligent routing
#[derive(Clone)]
pub struct ApiGatewayTemporalClient {
    config: TemporalConfig,
    http_client: reqwest::Client,
    namespace: String,
    server_address: String,
}

impl ApiGatewayTemporalClient {
    pub async fn new(config: TemporalConfig) -> ApiResult<Self> {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(config.connection_timeout_seconds))
            .build()
            .map_err(|e| ApiGatewayError::TemporalError {
                message: format!("Failed to create HTTP client: {}", e),
            })?;

        info!(
            namespace = %config.namespace,
            server_address = %config.server_address,
            "Initializing API Gateway Temporal client"
        );

        // Test connection to Temporal server
        let health_url = format!("http://{}/api/v1/namespaces/{}", config.server_address, config.namespace);
        match http_client.get(&health_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Successfully connected to Temporal server");
                } else {
                    warn!(
                        status = %response.status(),
                        "Temporal server responded with non-success status"
                    );
                }
            }
            Err(e) => {
                warn!(
                    error = %e,
                    "Failed to connect to Temporal server, will retry on first operation"
                );
            }
        }

        Ok(Self {
            namespace: config.namespace.clone(),
            server_address: config.server_address.clone(),
            config,
            http_client,
        })
    }

    /// Start a workflow execution
    pub async fn start_workflow<T>(
        &self,
        workflow_type: &str,
        workflow_id: Option<String>,
        task_queue: &str,
        input: T,
        tenant_id: &str,
        user_id: &str,
    ) -> ApiResult<WorkflowExecutionResponse>
    where
        T: Serialize + Send + Sync + 'static,
    {
        let workflow_id = workflow_id.unwrap_or_else(|| {
            format!("{}-{}-{}", workflow_type, tenant_id, Uuid::new_v4())
        });

        info!(
            workflow_type = workflow_type,
            workflow_id = %workflow_id,
            task_queue = task_queue,
            tenant_id = tenant_id,
            user_id = user_id,
            "Starting workflow execution"
        );

        // Serialize input for workflow execution
        let _input_json = serde_json::to_value(&input)
            .map_err(|e| ApiGatewayError::InvalidRequest {
                message: format!("Failed to serialize workflow input: {}", e),
            })?;

        // For now, simulate workflow execution since we don't have the full SDK
        // This will be replaced with actual Temporal communication when SDK is stable
        let run_id = Uuid::new_v4().to_string();
        let _started_at = Utc::now();

        debug!(
            workflow_id = %workflow_id,
            workflow_type = workflow_type,
            run_id = %run_id,
            "Workflow execution started (simulated)"
        );

        // Determine if this should be synchronous or asynchronous based on workflow type
        let is_synchronous = self.is_synchronous_workflow(workflow_type);

        if is_synchronous {
            // For quick workflows, simulate immediate completion
            tokio::time::sleep(Duration::from_millis(100)).await;
            
            let result = serde_json::json!({
                "success": true,
                "workflow_id": workflow_id,
                "completed_at": Utc::now()
            });

            Ok(WorkflowExecutionResponse::Synchronous {
                data: result,
                execution_time_ms: 100,
                workflow_id: workflow_id.clone(),
            })
        } else {
            // For long-running workflows, return operation tracking info
            Ok(WorkflowExecutionResponse::Asynchronous {
                operation_id: workflow_id.clone(),
                status_url: format!("/api/v1/workflows/{}/status", workflow_id),
                stream_url: Some(format!("/api/v1/workflows/{}/stream", workflow_id)),
                estimated_duration_seconds: self.estimate_workflow_duration(workflow_type),
            })
        }
    }

    /// Get workflow execution status
    pub async fn get_workflow_status(
        &self,
        workflow_id: &str,
    ) -> ApiResult<WorkflowStatusResponse> {
        debug!(
            workflow_id = workflow_id,
            "Getting workflow execution status"
        );

        // For now, simulate workflow status
        // This will be replaced with actual Temporal API calls when SDK is stable
        Ok(WorkflowStatusResponse {
            operation_id: workflow_id.to_string(),
            status: WorkflowStatus::Completed, // Simulate completed for now
            progress: Some(WorkflowProgress {
                current_step: "completed".to_string(),
                total_steps: 3,
                completed_steps: 3,
                percentage: 100.0,
                message: Some("Workflow completed successfully".to_string()),
            }),
            result: Some(serde_json::json!({
                "success": true,
                "workflow_id": workflow_id,
                "completed_at": Utc::now()
            })),
            error: None,
            started_at: Utc::now() - chrono::Duration::seconds(10),
            updated_at: Utc::now(),
            estimated_completion: None,
        })
    }

    /// Cancel workflow execution
    pub async fn cancel_workflow(
        &self,
        workflow_id: &str,
        reason: &str,
    ) -> ApiResult<()> {
        info!(
            workflow_id = workflow_id,
            reason = reason,
            "Cancelling workflow execution"
        );

        // For now, simulate workflow cancellation
        // This will be replaced with actual Temporal API calls when SDK is stable
        debug!(
            workflow_id = workflow_id,
            "Workflow cancellation simulated successfully"
        );

        Ok(())
    }

    /// Signal workflow execution
    pub async fn signal_workflow<T>(
        &self,
        workflow_id: &str,
        signal_name: &str,
        signal_input: T,
    ) -> ApiResult<()>
    where
        T: Serialize + Send + Sync + 'static,
    {
        debug!(
            workflow_id = workflow_id,
            signal_name = signal_name,
            "Sending signal to workflow"
        );

        // Serialize signal input for logging
        let _input_json = serde_json::to_value(&signal_input)
            .map_err(|e| ApiGatewayError::InvalidRequest {
                message: format!("Failed to serialize signal input: {}", e),
            })?;

        // For now, simulate signal sending
        // This will be replaced with actual Temporal API calls when SDK is stable
        debug!(
            workflow_id = workflow_id,
            signal_name = signal_name,
            "Signal sent successfully (simulated)"
        );

        Ok(())
    }

    /// Determine if a workflow should be executed synchronously
    fn is_synchronous_workflow(&self, workflow_type: &str) -> bool {
        match workflow_type {
            // Quick operations that should complete immediately
            "validate_user" | "check_permissions" | "get_user_profile" => true,
            
            // Complex operations that should be asynchronous
            "create_tenant" | "migrate_tenant" | "file_upload" | "bulk_operation" => false,
            
            // Default to asynchronous for safety
            _ => false,
        }
    }

    /// Estimate workflow duration in seconds
    fn estimate_workflow_duration(&self, workflow_type: &str) -> Option<u64> {
        match workflow_type {
            "create_tenant" => Some(60),      // 1 minute
            "migrate_tenant" => Some(300),    // 5 minutes
            "file_upload" => Some(120),       // 2 minutes
            "bulk_operation" => Some(600),    // 10 minutes
            "user_onboarding" => Some(30),    // 30 seconds
            _ => Some(60),                    // Default 1 minute
        }
    }

    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    pub fn server_address(&self) -> &str {
        &self.server_address
    }
}

/// Workflow execution response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WorkflowExecutionResponse {
    #[serde(rename = "synchronous")]
    Synchronous {
        data: serde_json::Value,
        execution_time_ms: u64,
        workflow_id: String,
    },
    #[serde(rename = "asynchronous")]
    Asynchronous {
        operation_id: String,
        status_url: String,
        stream_url: Option<String>,
        estimated_duration_seconds: Option<u64>,
    },
}

/// Workflow status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatusResponse {
    pub operation_id: String,
    pub status: WorkflowStatus,
    pub progress: Option<WorkflowProgress>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Workflow execution status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

/// Workflow progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub message: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_temporal_client_creation() {
        let config = TemporalConfig {
            server_address: "localhost:7233".to_string(),
            namespace: "test".to_string(),
            client_identity: "api-gateway-test".to_string(),
            connection_timeout_seconds: 5,
            request_timeout_seconds: 10,
        };

        // Skip actual connection in tests
        std::env::set_var("SIMULATE_CONNECTION_FAILURE", "false");

        let client = ApiGatewayTemporalClient::new(config).await;
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.namespace(), "test");
    }

    #[test]
    fn test_workflow_type_classification() {
        let config = TemporalConfig {
            server_address: "localhost:7233".to_string(),
            namespace: "test".to_string(),
            client_identity: "api-gateway-test".to_string(),
            connection_timeout_seconds: 5,
            request_timeout_seconds: 10,
        };

        let client = ApiGatewayTemporalClient {
            config: config.clone(),
            http_client: reqwest::Client::new(),
            namespace: config.namespace.clone(),
            server_address: config.server_address.clone(),
        };

        // Test synchronous workflows
        assert!(client.is_synchronous_workflow("validate_user"));
        assert!(client.is_synchronous_workflow("check_permissions"));

        // Test asynchronous workflows
        assert!(!client.is_synchronous_workflow("create_tenant"));
        assert!(!client.is_synchronous_workflow("file_upload"));

        // Test default behavior
        assert!(!client.is_synchronous_workflow("unknown_workflow"));
    }
}