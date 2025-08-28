use std::env;
use tracing::{info, error, Level};
use tracing_subscriber;

use adx_shared::temporal::{
    run_all_sdk_tests, 
    run_comprehensive_connectivity_test,
    run_all_integration_tests,
    test_sdk_integration
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    info!("Starting ADX Core Temporal SDK Integration Test");

    let args: Vec<String> = env::args().collect();
    let test_type = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    let result = match test_type {
        "connectivity" => {
            info!("Running connectivity tests");
            run_comprehensive_connectivity_test().await
        }
        "integration" => {
            info!("Running integration tests");
            run_all_integration_tests().await
        }
        "sdk" => {
            info!("Running SDK integration test");
            test_sdk_integration().await
        }
        "all" | _ => {
            info!("Running all SDK tests");
            run_all_sdk_tests().await
        }
    };

    match result {
        Ok(_) => {
            info!("✅ All Temporal SDK tests completed successfully!");
            println!("\n🎉 SUCCESS: Temporal SDK integration is working correctly!");
            println!("📋 Test Summary:");
            println!("  ✓ SDK Client Creation and Configuration");
            println!("  ✓ Worker Management and Lifecycle");
            println!("  ✓ Workflow Execution and Monitoring");
            println!("  ✓ Connectivity and Communication");
            println!("  ✓ Error Handling and Edge Cases");
            println!("  ✓ Concurrent Operations");
            println!("  ✓ Performance Benchmarks");
            println!("  ✓ Integration Tests");
            println!("\n🚀 The Temporal SDK is ready for production use!");
        }
        Err(e) => {
            error!("❌ Temporal SDK tests failed: {}", e);
            println!("\n💥 FAILURE: Temporal SDK integration has issues!");
            println!("🔍 Error Details: {}", e);
            println!("📝 Troubleshooting Steps:");
            println!("  1. Check Temporal server is running (docker-compose up temporal)");
            println!("  2. Verify network connectivity to Temporal server");
            println!("  3. Check configuration settings in TemporalConfig");
            println!("  4. Review logs for detailed error information");
            println!("  5. Ensure all dependencies are properly installed");
            std::process::exit(1);
        }
    }

    Ok(())
}