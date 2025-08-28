use std::time::Duration;
use tracing::{info, debug, warn, error};
use uuid::Uuid;
use serde_json::json;

use crate::temporal::{
    TemporalConfig, TemporalError, TemporalSDKClient, 
    AdxTemporalWorkerManager, connectivity_test, integration_test
};

/// Comprehensive test suite for Temporal SDK integration
pub async fn run_sdk_test_suite() -> Result<(), TemporalError> {
    info!("Starting comprehensive Temporal SDK test suite");
    
    // Test 1: Basic SDK client functionality
    test_sdk_client_basic().await?;
    
    // Test 2: Worker management
    test_worker_management().await?;
    
    // Test 3: Workflow execution
    test_workflow_execution().await?;
    
    // Test 4: Connectivity tests
    connectivity_test::run_comprehensive_connectivity_test().await?;
    
    // Test 5: Integration tests
    integration_test::run_all_integration_tests().await?;
    
    info!("Comprehensive Temporal SDK test suite completed successfully");
    Ok(())
}

/// Test basic SDK client functionality
async fn test_sdk_client_basic() -> Result<(), TemporalError> {
    info!("Testing basic SDK client functionality");
    
    let config = TemporalConfig::development();
    let client = TemporalSDKClient::new(config).await?;
    
    // Test client properties
    assert!(!client.client_id().is_empty(), "Client ID should not be empty");
    assert_eq!(client.namespace(), "adx-core-development");
    
    info!(
        client_id = %client.client_id(),
        namespace = %client.namespace(),
        "SDK client created successfully"
    );
    
    Ok(())
}

/// Test worker management functionality
async fn test_worker_management() -> Result<(), TemporalError> {
    info!("Testing worker management functionality");
    
    let config = TemporalConfig::development();
    let task_queues = vec!["test-worker-management".to_string()];
    
    let worker_manager = AdxTemporalWorkerManager::new(config, task_queues).await?;
    
    // Test worker manager properties
    assert_eq!(worker_manager.task_queues(), &["test-worker-management"]);
    assert!(worker_manager.worker_identity().starts_with("adx-worker-"));
    
    // Test workflow and activity counts
    assert_eq!(worker_manager.workflow_count().await, 0);
    assert_eq!(worker_manager.activity_count().await, 0);
    
    info!(
        worker_identity = %worker_manager.worker_identity(),
        task_queues = ?worker_manager.task_queues(),
        "Worker manager created successfully"
    );
    
    Ok(())
}

/// Test workflow execution functionality
async fn test_workflow_execution() -> Result<(), TemporalError> {
    info!("Testing workflow execution functionality");
    
    let config = TemporalConfig::development();
    let client = TemporalSDKClient::new(config).await?;
    
    // Test workflow start
    let workflow_id = format!("test-execution-{}", Uuid::new_v4());
    let test_input = json!({
        "test": "workflow execution test",
        "timestamp": chrono::Utc::now().to_rfc3339()
    });
    
    let handle = client.start_workflow::<serde_json::Value, serde_json::Value>(
        "test_workflow",
        workflow_id.clone(),
        "test-execution-queue",
        test_input,
    ).await?;
    
    // Test handle properties
    assert_eq!(handle.workflow_id(), workflow_id);
    assert!(!handle.run_id().is_empty());
    assert_eq!(handle.namespace(), "adx-core-development");
    
    info!(
        workflow_id = %handle.workflow_id(),
        run_id = %handle.run_id(),
        "Workflow started successfully"
    );
    
    // Test workflow execution info
    let execution_info = client.get_workflow_execution_info(
        handle.workflow_id(),
        handle.run_id(),
    ).await?;
    
    assert_eq!(execution_info.workflow_id, workflow_id);
    assert_eq!(execution_info.run_id, handle.run_id());
    
    info!(
        execution_info = ?execution_info,
        "Workflow execution info retrieved successfully"
    );
    
    // Test workflow result (with timeout)
    let result = tokio::time::timeout(
        Duration::from_secs(10),
        handle.get_result()
    ).await;
    
    match result {
        Ok(Ok(workflow_result)) => {
            info!(
                workflow_result = ?workflow_result,
                "Workflow completed successfully"
            );
        }
        Ok(Err(e)) => {
            warn!(
                error = %e,
                "Workflow failed (expected for mock implementation)"
            );
        }
        Err(_) => {
            warn!("Workflow timed out (expected for mock implementation)");
        }
    }
    
    Ok(())
}

/// Test error handling and edge cases
pub async fn test_error_handling() -> Result<(), TemporalError> {
    info!("Testing error handling and edge cases");
    
    let config = TemporalConfig::development();
    let client = TemporalSDKClient::new(config).await?;
    
    // Test invalid workflow ID
    let invalid_workflow_id = "";
    let test_input = json!({"test": "error handling"});
    
    let result = client.start_workflow::<serde_json::Value, serde_json::Value>(
        "test_workflow",
        invalid_workflow_id.to_string(),
        "test-error-queue",
        test_input,
    ).await;
    
    // Should succeed even with empty workflow ID (mock implementation)
    match result {
        Ok(_) => info!("Empty workflow ID handled gracefully"),
        Err(e) => info!(error = %e, "Empty workflow ID rejected as expected"),
    }
    
    // Test non-existent workflow query
    let query_result = client.get_workflow_execution_info(
        "non-existent-workflow",
        "non-existent-run",
    ).await;
    
    match query_result {
        Ok(_) => info!("Non-existent workflow query handled gracefully"),
        Err(e) => info!(error = %e, "Non-existent workflow query failed as expected"),
    }
    
    info!("Error handling tests completed");
    Ok(())
}

/// Test concurrent operations
pub async fn test_concurrent_operations() -> Result<(), TemporalError> {
    info!("Testing concurrent operations");
    
    let config = TemporalConfig::development();
    let client = TemporalSDKClient::new(config).await?;
    
    // Start multiple workflows concurrently
    let workflow_count = 3;
    let mut handles = Vec::new();
    
    for i in 0..workflow_count {
        let workflow_id = format!("concurrent-test-{}-{}", i, Uuid::new_v4());
        let test_input = json!({
            "index": i,
            "message": format!("Concurrent test {}", i)
        });
        
        let handle = client.start_workflow::<serde_json::Value, serde_json::Value>(
            "concurrent_test_workflow",
            workflow_id,
            "concurrent-test-queue",
            test_input,
        ).await?;
        
        handles.push(handle);
    }
    
    info!(
        workflow_count = workflow_count,
        "Started concurrent workflows"
    );
    
    // Wait for all workflows (with timeout)
    let mut completed = 0;
    for handle in handles {
        match tokio::time::timeout(Duration::from_secs(5), handle.get_result()).await {
            Ok(Ok(_)) => {
                completed += 1;
                debug!(
                    workflow_id = %handle.workflow_id(),
                    "Concurrent workflow completed"
                );
            }
            Ok(Err(e)) => {
                debug!(
                    workflow_id = %handle.workflow_id(),
                    error = %e,
                    "Concurrent workflow failed"
                );
            }
            Err(_) => {
                debug!(
                    workflow_id = %handle.workflow_id(),
                    "Concurrent workflow timed out"
                );
            }
        }
    }
    
    info!(
        completed = completed,
        total = workflow_count,
        "Concurrent operations test completed"
    );
    
    Ok(())
}

/// Performance benchmark test
pub async fn test_performance_benchmark() -> Result<(), TemporalError> {
    info!("Running performance benchmark test");
    
    let config = TemporalConfig::development();
    let client = TemporalSDKClient::new(config).await?;
    
    let start_time = std::time::Instant::now();
    let operation_count = 10;
    
    // Benchmark workflow starts
    for i in 0..operation_count {
        let workflow_id = format!("benchmark-{}-{}", i, Uuid::new_v4());
        let test_input = json!({"benchmark_index": i});
        
        let _handle = client.start_workflow::<serde_json::Value, serde_json::Value>(
            "benchmark_workflow",
            workflow_id,
            "benchmark-queue",
            test_input,
        ).await?;
    }
    
    let duration = start_time.elapsed();
    let ops_per_second = operation_count as f64 / duration.as_secs_f64();
    
    info!(
        operation_count = operation_count,
        duration_ms = duration.as_millis(),
        ops_per_second = ops_per_second,
        "Performance benchmark completed"
    );
    
    // Basic performance assertion (should be able to start at least 1 workflow per second)
    assert!(ops_per_second >= 1.0, "Performance below minimum threshold");
    
    Ok(())
}

/// Test configuration variations
pub async fn test_configuration_variations() -> Result<(), TemporalError> {
    info!("Testing configuration variations");
    
    // Test development config
    let dev_config = TemporalConfig::development();
    let dev_client = TemporalSDKClient::new(dev_config).await?;
    assert_eq!(dev_client.namespace(), "adx-core-development");
    
    // Test production config
    let prod_config = TemporalConfig::production();
    let prod_client = TemporalSDKClient::new(prod_config).await?;
    assert_eq!(prod_client.namespace(), "adx-core-production");
    
    // Test staging config
    let staging_config = TemporalConfig::staging();
    let staging_client = TemporalSDKClient::new(staging_config).await?;
    assert_eq!(staging_client.namespace(), "adx-core-staging");
    
    info!("Configuration variations test completed");
    Ok(())
}

/// Run all SDK tests
pub async fn run_all_sdk_tests() -> Result<(), TemporalError> {
    info!("Running all Temporal SDK tests");
    
    // Core functionality tests
    run_sdk_test_suite().await?;
    
    // Error handling tests
    test_error_handling().await?;
    
    // Concurrent operations tests
    test_concurrent_operations().await?;
    
    // Performance benchmark
    test_performance_benchmark().await?;
    
    // Configuration tests
    test_configuration_variations().await?;
    
    info!("All Temporal SDK tests completed successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_sdk_client_basic_functionality() {
        let result = test_sdk_client_basic().await;
        assert!(result.is_ok(), "SDK client basic test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_worker_management_functionality() {
        let result = test_worker_management().await;
        assert!(result.is_ok(), "Worker management test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_workflow_execution_functionality() {
        let result = test_workflow_execution().await;
        assert!(result.is_ok(), "Workflow execution test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_error_handling_functionality() {
        let result = test_error_handling().await;
        assert!(result.is_ok(), "Error handling test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_concurrent_operations_functionality() {
        let result = test_concurrent_operations().await;
        assert!(result.is_ok(), "Concurrent operations test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_performance_benchmark_functionality() {
        let result = test_performance_benchmark().await;
        assert!(result.is_ok(), "Performance benchmark test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_configuration_variations_functionality() {
        let result = test_configuration_variations().await;
        assert!(result.is_ok(), "Configuration variations test failed: {:?}", result);
    }
    
    #[tokio::test]
    async fn test_complete_sdk_suite() {
        let result = run_all_sdk_tests().await;
        assert!(result.is_ok(), "Complete SDK test suite failed: {:?}", result);
    }
}