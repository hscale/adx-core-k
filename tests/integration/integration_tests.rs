// Main integration test runner for ADX CORE
use crate::circuit_breakers::CircuitBreakerTests;
use crate::user_workflows::UserWorkflowTests;
use crate::multi_tenant::MultiTenantTests;
use crate::load_testing::LoadTestingSuite;
use crate::micro_frontend::MicroFrontendTests;
use crate::test_environment::{IntegrationTestEnvironment, TestData};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize test environment
    let test_env = Arc::new(IntegrationTestEnvironment::new().await?);
    let test_data = test_env.setup_test_data().await?;

    // Run Circuit Breaker Tests
    let circuit_breaker_tests = CircuitBreakerTests::new(test_env.clone());
    let cb_results = circuit_breaker_tests.run_all_tests().await?;

    // Run User Workflow Tests
    let user_workflow_tests = UserWorkflowTests::new(test_env.clone(), test_data.clone());
    let uw_results = user_workflow_tests.run_all_tests().await?;

    // Run Multi-Tenant Tests
    let multi_tenant_tests = MultiTenantTests::new(test_env.clone(), test_data.clone());
    let mt_results = multi_tenant_tests.run_all_tests().await?;

    // Run Load Testing Suite
    let load_tests = LoadTestingSuite::new(test_env.clone(), test_data.clone());
    let lt_results = load_tests.run_all_tests().await?;

    // Run Micro-Frontend Tests
    let micro_frontend_tests = MicroFrontendTests::new(test_env.clone(), test_data.clone());
    let mf_results = micro_frontend_tests.run_all_tests().await?;

    // Print results (can be extended to aggregate and report)
    println!("Circuit Breaker Results: {:?}", cb_results);
    println!("User Workflow Results: {:?}", uw_results);
    println!("Multi-Tenant Results: {:?}", mt_results);
    println!("Load Testing Results: {:?}", lt_results);
    println!("Micro-Frontend Results: {:?}", mf_results);

    Ok(())
}
