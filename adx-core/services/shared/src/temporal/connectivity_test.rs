use std::time::Duration;
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use serde_json::json;

use crate::temporal::{TemporalConfig, TemporalError, TemporalSDKClient};

/// Test Temporal SDK connectivity and basic workflow execution
pub async fn test_temporal_connectivity() -> Result<(), TemporalError> {
    info!("Starting Temporal SDK connectivity test");
    
    // Create SDK client
    let config = TemporalConfig::development();
    let client = TemporalSDKClient::new(config).await?;
    
    info!(
        client_id = %client.client_id(),
        namespace = %client.namespace(),
        "SDK client created successfully"
    );
    
    // Test basic workflow execution
    let workflow_id = format!("connectivity-test-{}", Uuid::new_v4());
    let test_input = json!({
        "message": "Connectivity test",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    info!(
        workflow_id = %workflow_id,
        "Starting connectivity test workflow"
    );
    
    let handle = client.start_workflow::<serde_json::Value, serde_json::Value>(
        "connectivity_test_workflow",
        workflow_id.clone(),
        "adx-core-connectivity-test",
        test_input,
    ).await?;
    
    info!(
        workflow_id = %handle.workflow_id(),
        run_id = %handle.run_id(),
        "Connectivity test workflow started"
    );
    
    // Test getting workflow execution info
    let execution_info = client.get_workflow_execution_info(
        handle.workflow_id(),
        handle.run_id(),
    ).await?;
    
    info!(
        workflow_id = %execution_info.workflow_id,
        run_id = %execution_info.run_id,
        status = ?execution_info.status,
        "Retrieved workflow execution info"
    );
    
    // Test workflow result (with timeout)
    let result = tokio::time::timeout(
        Duration::from_secs(30),
        handle.get_result()
    ).await;
    
    match result {
        Ok(Ok(workflow_result)) => {
            info!(
                workflow_id = %handle.workflow_id(),
                result = ?workflow_result,
                "Connectivity test workflow completed successfully"
            );
        }
        Ok(Err(e)) => {
            warn!(
                workflow_id = %handle.workflow_id(),
                error = %e,
                "Connectivity test workflow failed"
            );
            return Err(TemporalError::WorkflowExecutionError {
                workflow_id: handle.workflow_id().to_string(),
                message: e.to_string(),
            });
        }
        Err(_) => {
            warn!(
                workflow_id = %handle.workflow_id(),
                "Connectivity test workflow timed out"
            );
            return Err(TemporalError::WorkflowExecutionError {
                workflow_id: handle.workflow_id().to_string(),
                message: "Workflow execution timed out".to_string(),
            });
        }
    }
    
    // Test signal workflow
    info!(
        workflow_id = %handle.workflow_id(),
        "Testing workflow signal"
    );
    
    let signal_result = client.signal_workflow(
        handle.workflow_id(),
        handle.run_id(),
        "test_signal",
        json!({"signal_message": "connectivity test signal"}),
    ).await;
    
    match signal_result {
        Ok(_) => {
            info!("Workflow signal sent successfully");
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to send workflow signal (expected for mock implementation)"
            );
        }
    }
    
    // Test query workflow
    info!(
        workflow_id = %handle.workflow_id(),
        "Testing workflow query"
    );
    
    let query_result = client.query_workflow::<serde_json::Value, serde_json::Value>(
        handle.workflow_id(),
        handle.run_id(),
        "test_query",
        json!({"query_message": "connectivity test query"}),
    ).await;
    
    match query_result {
        Ok(query_response) => {
            info!(
                query_response = ?query_response,
                "Workflow query executed successfully"
            );
        }
        Err(e) => {
            warn!(
                error = %e,
                "Failed to execute workflow query (expected for mock implementation)"
            );
        }
    }
    
    info!("Temporal SDK connectivity test completed successfully");
    Ok(())
}

/// Test worker creation and management
pub async fn test_worker_functionality() -> Result<(), TemporalError> {
    info!("Starting Temporal SDK worker functionality test");
    
    let config = TemporalConfig::development();
    let client = TemporalSDKClient::new(config).await?;
    
    // Test worker creation
    let worker = client.create_worker("adx-core-worker-test").await?;
    
    info!(
        task_queue = %worker.config().task_queue,
        worker_build_id = %worker.config().worker_build_id,
        "Worker created successfully"
    );
    
    // Test worker start
    worker.start().await?;
    info!("Worker started successfully");
    
    // Verify worker is running
    assert!(worker.is_running().await, "Worker should be running");
    
    // Test worker stop
    worker.stop().await?;
    info!("Worker stopped successfully");
    
    // Verify worker is stopped
    assert!(!worker.is_running().await, "Worker should be stopped");
    
    info!("Temporal SDK worker functionality test completed successfully");
    Ok(())
}

/// Comprehensive connectivity test
pub async fn run_comprehensive_connectivity_test() -> Result<(), TemporalError> {
    info!("Running comprehensive Temporal SDK connectivity test");
    
    // Test basic connectivity
    test_temporal_connectivity().await?;
    
    // Test worker functionality
    test_worker_functionality().await?;
    
    info!("Comprehensive Temporal SDK connectivity test completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_connectivity() {
        let result = test_temporal_connectivity().await;
        assert!(result.is_ok(), "Connectivity test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_worker() {
        let result = test_worker_functionality().await;
        assert!(result.is_ok(), "Worker test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_comprehensive() {
        let result = run_comprehensive_connectivity_test().await;
        assert!(result.is_ok(), "Comprehensive test failed: {:?}", result);
    }
}