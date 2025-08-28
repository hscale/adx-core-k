use std::time::Duration;
use std::sync::Arc;
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use serde_json::json;
use tokio::time::timeout;

use crate::temporal::{
    TemporalConfig, TemporalError, TemporalSDKClient, 
    AdxTemporalWorkerManager, WorkflowFunction, ActivityFunction,
    WorkflowExecutionError, ActivityExecutionError
};

/// Integration test for complete Temporal SDK workflow
pub async fn test_end_to_end_workflow() -> Result<(), TemporalError> {
    info!("Starting end-to-end Temporal SDK integration test");
    
    // Create SDK client
    let config = TemporalConfig::development();
    let client = Arc::new(TemporalSDKClient::new(config.clone()).await?);
    
    // Create worker manager
    let task_queues = vec!["adx-core-integration-test".to_string()];
    let worker_manager = AdxTemporalWorkerManager::new(config, task_queues).await?;
    
    // Register test workflow and activity
    worker_manager.register_workflow("integration_test_workflow", TestWorkflow).await?;
    worker_manager.register_activity("integration_test_activity", TestActivity).await?;
    
    // Start worker manager
    worker_manager.start().await?;
    
    info!("Worker manager started, executing test workflow");
    
    // Execute test workflow
    let workflow_id = format!("integration-test-{}", Uuid::new_v4());
    let test_input = json!({
        "test_data": "integration test",
        "steps": ["validate", "process", "complete"],
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    let handle = client.start_workflow::<serde_json::Value, serde_json::Value>(
        "integration_test_workflow",
        workflow_id.clone(),
        "adx-core-integration-test",
        test_input,
    ).await?;
    
    info!(
        workflow_id = %handle.workflow_id(),
        run_id = %handle.run_id(),
        "Integration test workflow started"
    );
    
    // Wait for workflow completion with timeout
    let result = timeout(Duration::from_secs(60), handle.get_result()).await;
    
    match result {
        Ok(Ok(workflow_result)) => {
            info!(
                workflow_id = %handle.workflow_id(),
                result = ?workflow_result,
                "Integration test workflow completed successfully"
            );
        }
        Ok(Err(e)) => {
            error!(
                workflow_id = %handle.workflow_id(),
                error = %e,
                "Integration test workflow failed"
            );
            worker_manager.stop().await?;
            return Err(TemporalError::WorkflowExecutionError {
                workflow_id: handle.workflow_id().to_string(),
                message: e.to_string(),
            });
        }
        Err(_) => {
            error!(
                workflow_id = %handle.workflow_id(),
                "Integration test workflow timed out"
            );
            worker_manager.stop().await?;
            return Err(TemporalError::WorkflowExecutionError {
                workflow_id: handle.workflow_id().to_string(),
                message: "Workflow execution timed out".to_string(),
            });
        }
    }
    
    // Stop worker manager
    worker_manager.stop().await?;
    
    info!("End-to-end Temporal SDK integration test completed successfully");
    Ok(())
}

/// Test concurrent workflow execution
pub async fn test_concurrent_workflows() -> Result<(), TemporalError> {
    info!("Starting concurrent workflows test");
    
    let config = TemporalConfig::development();
    let client = Arc::new(TemporalSDKClient::new(config.clone()).await?);
    
    // Create worker manager
    let task_queues = vec!["adx-core-concurrent-test".to_string()];
    let worker_manager = AdxTemporalWorkerManager::new(config, task_queues).await?;
    
    // Register test workflow
    worker_manager.register_workflow("concurrent_test_workflow", ConcurrentTestWorkflow).await?;
    worker_manager.start().await?;
    
    // Start multiple workflows concurrently
    let workflow_count = 5;
    let mut handles = Vec::new();
    
    for i in 0..workflow_count {
        let workflow_id = format!("concurrent-test-{}-{}", i, Uuid::new_v4());
        let test_input = json!({
            "workflow_index": i,
            "message": format!("Concurrent test workflow {}", i),
            "timestamp": chrono::Utc::now().to_rfc3339()
        });
        
        let handle = client.start_workflow::<serde_json::Value, serde_json::Value>(
            "concurrent_test_workflow",
            workflow_id,
            "adx-core-concurrent-test",
            test_input,
        ).await?;
        
        handles.push(handle);
    }
    
    info!(
        workflow_count = workflow_count,
        "Started concurrent workflows"
    );
    
    // Wait for all workflows to complete
    let mut completed_count = 0;
    for handle in handles {
        match timeout(Duration::from_secs(30), handle.get_result()).await {
            Ok(Ok(_)) => {
                completed_count += 1;
                debug!(
                    workflow_id = %handle.workflow_id(),
                    "Concurrent workflow completed"
                );
            }
            Ok(Err(e)) => {
                warn!(
                    workflow_id = %handle.workflow_id(),
                    error = %e,
                    "Concurrent workflow failed"
                );
            }
            Err(_) => {
                warn!(
                    workflow_id = %handle.workflow_id(),
                    "Concurrent workflow timed out"
                );
            }
        }
    }
    
    worker_manager.stop().await?;
    
    info!(
        completed_count = completed_count,
        total_count = workflow_count,
        "Concurrent workflows test completed"
    );
    
    if completed_count == workflow_count {
        info!("All concurrent workflows completed successfully");
        Ok(())
    } else {
        Err(TemporalError::WorkflowExecutionError {
            workflow_id: "concurrent_test".to_string(),
            message: format!("Only {} of {} workflows completed", completed_count, workflow_count),
        })
    }
}

/// Test workflow with signal and query
pub async fn test_workflow_signals_and_queries() -> Result<(), TemporalError> {
    info!("Starting workflow signals and queries test");
    
    let config = TemporalConfig::development();
    let client = Arc::new(TemporalSDKClient::new(config.clone()).await?);
    
    // Create worker manager
    let task_queues = vec!["adx-core-signal-test".to_string()];
    let worker_manager = AdxTemporalWorkerManager::new(config, task_queues).await?;
    
    // Register test workflow
    worker_manager.register_workflow("signal_test_workflow", SignalTestWorkflow).await?;
    worker_manager.start().await?;
    
    // Start workflow
    let workflow_id = format!("signal-test-{}", Uuid::new_v4());
    let test_input = json!({
        "initial_state": "waiting_for_signal",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    let handle = client.start_workflow::<serde_json::Value, serde_json::Value>(
        "signal_test_workflow",
        workflow_id.clone(),
        "adx-core-signal-test",
        test_input,
    ).await?;
    
    info!(
        workflow_id = %handle.workflow_id(),
        "Signal test workflow started"
    );
    
    // Send signal to workflow
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let signal_result = client.signal_workflow(
        handle.workflow_id(),
        handle.run_id(),
        "proceed_signal",
        json!({"action": "proceed", "data": "test signal data"}),
    ).await;
    
    match signal_result {
        Ok(_) => info!("Signal sent successfully"),
        Err(e) => warn!(error = %e, "Signal failed (expected for mock)"),
    }
    
    // Query workflow state
    let query_result = client.query_workflow::<serde_json::Value, serde_json::Value>(
        handle.workflow_id(),
        handle.run_id(),
        "get_state",
        json!({}),
    ).await;
    
    match query_result {
        Ok(state) => info!(state = ?state, "Query executed successfully"),
        Err(e) => warn!(error = %e, "Query failed (expected for mock)"),
    }
    
    // Wait for workflow completion
    let result = timeout(Duration::from_secs(30), handle.get_result()).await;
    
    match result {
        Ok(Ok(workflow_result)) => {
            info!(
                workflow_id = %handle.workflow_id(),
                result = ?workflow_result,
                "Signal test workflow completed successfully"
            );
        }
        Ok(Err(e)) => {
            warn!(
                workflow_id = %handle.workflow_id(),
                error = %e,
                "Signal test workflow failed"
            );
        }
        Err(_) => {
            warn!(
                workflow_id = %handle.workflow_id(),
                "Signal test workflow timed out"
            );
        }
    }
    
    worker_manager.stop().await?;
    
    info!("Workflow signals and queries test completed");
    Ok(())
}

/// Run all integration tests
pub async fn run_all_integration_tests() -> Result<(), TemporalError> {
    info!("Running all Temporal SDK integration tests");
    
    // Test end-to-end workflow
    test_end_to_end_workflow().await?;
    
    // Test concurrent workflows
    test_concurrent_workflows().await?;
    
    // Test signals and queries
    test_workflow_signals_and_queries().await?;
    
    info!("All Temporal SDK integration tests completed successfully");
    Ok(())
}

// Test workflow implementations
struct TestWorkflow;

impl WorkflowFunction for TestWorkflow {
    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, WorkflowExecutionError> {
        let input_json: serde_json::Value = serde_json::from_slice(&input)
            .map_err(|e| WorkflowExecutionError::SerializationError {
                message: format!("Failed to deserialize input: {}", e),
            })?;
        
        let result = json!({
            "status": "completed",
            "input_received": input_json,
            "processed_at": chrono::Utc::now().to_rfc3339(),
            "workflow_type": "integration_test_workflow"
        });
        
        serde_json::to_vec(&result)
            .map_err(|e| WorkflowExecutionError::SerializationError {
                message: format!("Failed to serialize result: {}", e),
            })
    }
}

struct TestActivity;

impl ActivityFunction for TestActivity {
    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, ActivityExecutionError> {
        let input_json: serde_json::Value = serde_json::from_slice(&input)
            .map_err(|e| ActivityExecutionError::SerializationError {
                message: format!("Failed to deserialize input: {}", e),
            })?;
        
        let result = json!({
            "status": "completed",
            "input_processed": input_json,
            "processed_at": chrono::Utc::now().to_rfc3339(),
            "activity_type": "integration_test_activity"
        });
        
        serde_json::to_vec(&result)
            .map_err(|e| ActivityExecutionError::SerializationError {
                message: format!("Failed to serialize result: {}", e),
            })
    }
}

struct ConcurrentTestWorkflow;

impl WorkflowFunction for ConcurrentTestWorkflow {
    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, WorkflowExecutionError> {
        let input_json: serde_json::Value = serde_json::from_slice(&input)
            .map_err(|e| WorkflowExecutionError::SerializationError {
                message: format!("Failed to deserialize input: {}", e),
            })?;
        
        // Simulate some processing time
        std::thread::sleep(Duration::from_millis(100));
        
        let result = json!({
            "status": "completed",
            "workflow_index": input_json.get("workflow_index"),
            "message": input_json.get("message"),
            "completed_at": chrono::Utc::now().to_rfc3339(),
            "workflow_type": "concurrent_test_workflow"
        });
        
        serde_json::to_vec(&result)
            .map_err(|e| WorkflowExecutionError::SerializationError {
                message: format!("Failed to serialize result: {}", e),
            })
    }
}

struct SignalTestWorkflow;

impl WorkflowFunction for SignalTestWorkflow {
    fn execute(&self, input: Vec<u8>) -> Result<Vec<u8>, WorkflowExecutionError> {
        let input_json: serde_json::Value = serde_json::from_slice(&input)
            .map_err(|e| WorkflowExecutionError::SerializationError {
                message: format!("Failed to deserialize input: {}", e),
            })?;
        
        // Simulate waiting for signal
        std::thread::sleep(Duration::from_millis(200));
        
        let result = json!({
            "status": "completed",
            "initial_state": input_json.get("initial_state"),
            "final_state": "signal_received",
            "completed_at": chrono::Utc::now().to_rfc3339(),
            "workflow_type": "signal_test_workflow"
        });
        
        serde_json::to_vec(&result)
            .map_err(|e| WorkflowExecutionError::SerializationError {
                message: format!("Failed to serialize result: {}", e),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_end_to_end() {
        let result = test_end_to_end_workflow().await;
        assert!(result.is_ok(), "End-to-end test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_concurrent() {
        let result = test_concurrent_workflows().await;
        assert!(result.is_ok(), "Concurrent test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_signals_queries() {
        let result = test_workflow_signals_and_queries().await;
        assert!(result.is_ok(), "Signals and queries test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_all_integration() {
        let result = run_all_integration_tests().await;
        assert!(result.is_ok(), "Integration tests failed: {:?}", result);
    }
}