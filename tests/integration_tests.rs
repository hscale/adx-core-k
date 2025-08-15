// Main integration test runner for ADX CORE End-to-End Testing
use std::sync::Arc;
use std::time::Instant;
use tokio;

mod integration;

use integration::*;

/// Main integration test runner
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting ADX CORE End-to-End Integration Tests");
    println!("================================================");

    let overall_start = Instant::now();

    // Initialize test environment
    println!("📋 Initializing test environment...");
    let test_env = match IntegrationTestEnvironment::new().await {
        Ok(env) => Arc::new(env),
        Err(e) => {
            eprintln!("❌ Failed to initialize test environment: {}", e);
            std::process::exit(1);
        }
    };

    // Setup test data
    println!("🔧 Setting up test data...");
    let test_data = match test_env.setup_test_data().await {
        Ok(data) => data,
        Err(e) => {
            eprintln!("❌ Failed to setup test data: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize test state manager
    let state_manager = TestStateManager::new(test_env.config.clone());

    println!("✅ Test environment ready!");
    println!();

    // Run all test suites
    let mut all_results = Vec::new();

    // 1. Circuit Breaker Tests
    println!("🔄 Running Circuit Breaker Tests...");
    let circuit_breaker_tests = CircuitBreakerTests::new(test_env.clone());
    match circuit_breaker_tests.run_all_tests().await {
        Ok(results) => {
            print_test_results(&results);
            state_manager.add_result(results.clone()).await;
            all_results.push(results);
        }
        Err(e) => {
            eprintln!("❌ Circuit breaker tests failed: {}", e);
        }
    }

    // 2. User Workflow Tests
    println!("👤 Running User Workflow Tests...");
    let user_workflow_tests = UserWorkflowTests::new(test_env.clone(), test_data.clone());
    match user_workflow_tests.run_all_tests().await {
        Ok(results) => {
            print_test_results(&results);
            state_manager.add_result(results.clone()).await;
            all_results.push(results);
        }
        Err(e) => {
            eprintln!("❌ User workflow tests failed: {}", e);
        }
    }

    // 3. Multi-Tenant Tests
    println!("🏢 Running Multi-Tenant Isolation Tests...");
    let multi_tenant_tests = MultiTenantTests::new(test_env.clone(), test_data.clone());
    match multi_tenant_tests.run_all_tests().await {
        Ok(results) => {
            print_test_results(&results);
            state_manager.add_result(results.clone()).await;
            all_results.push(results);
        }
        Err(e) => {
            eprintln!("❌ Multi-tenant tests failed: {}", e);
        }
    }

    // 4. Load Testing (if enabled)
    if test_env.config.enable_load_testing {
        println!("⚡ Running Load Tests...");
        let load_tests = LoadTestingSuite::new(test_env.clone(), test_data.clone());
        match load_tests.run_all_tests().await {
            Ok(results) => {
                print_test_results(&results);
                state_manager.add_result(results.clone()).await;
                all_results.push(results);
            }
            Err(e) => {
                eprintln!("❌ Load tests failed: {}", e);
            }
        }
    } else {
        println!("⏭️  Skipping Load Tests (disabled in configuration)");
    }

    // 5. Micro-Frontend Tests
    println!("🎨 Running Micro-Frontend Integration Tests...");
    let micro_frontend_tests = MicroFrontendTests::new(test_env.clone(), test_data.clone());
    match micro_frontend_tests.run_all_tests().await {
        Ok(results) => {
            print_test_results(&results);
            state_manager.add_result(results.clone()).await;
            all_results.push(results);
        }
        Err(e) => {
            eprintln!("❌ Micro-frontend tests failed: {}", e);
        }
    }

    // Generate final summary
    let overall_duration = overall_start.elapsed();
    let summary = state_manager.get_summary().await;

    println!();
    println!("📊 FINAL TEST SUMMARY");
    println!("=====================");
    println!("Total Test Suites: {}", summary.total_test_suites);
    println!("Total Tests: {}", summary.total_tests);
    println!("Passed: {} ✅", summary.passed_tests);
    println!("Failed: {} ❌", summary.failed_tests);
    println!("Success Rate: {:.1}%", summary.success_rate);
    println!("Total Execution Time: {:.2}s", overall_duration.as_secs_f64());
    println!();

    // Print detailed results for failed tests
    if summary.failed_tests > 0 {
        println!("❌ FAILED TESTS DETAILS");
        println!("=======================");
        for result in &all_results {
            for test in &result.test_details {
                if test.status == TestStatus::Failed {
                    println!("Suite: {} | Test: {}", result.test_suite, test.test_name);
                    if let Some(error) = &test.error_message {
                        println!("  Error: {}", error);
                    }
                    for assertion in &test.assertions {
                        if !assertion.passed {
                            println!("  ❌ {}: Expected '{}', Got '{}'", 
                                   assertion.description, assertion.expected, assertion.actual);
                        }
                    }
                    println!();
                }
            }
        }
    }

    // Generate test report
    generate_test_report(&all_results, &summary, overall_duration).await?;

    // Cleanup
    println!("🧹 Cleaning up test environment...");
    // The test environment will be cleaned up when it goes out of scope

    // Exit with appropriate code
    if summary.failed_tests > 0 {
        println!("❌ Some tests failed. Exiting with error code 1.");
        std::process::exit(1);
    } else {
        println!("✅ All tests passed successfully!");
        std::process::exit(0);
    }
}

/// Print test results for a test suite
fn print_test_results(results: &IntegrationTestResults) {
    println!("  Suite: {}", results.test_suite);
    println!("  Tests: {} | Passed: {} | Failed: {} | Skipped: {}", 
             results.total_tests, results.passed_tests, results.failed_tests, results.skipped_tests);
    println!("  Execution Time: {:.2}s", results.execution_time_ms as f64 / 1000.0);
    
    if results.failed_tests > 0 {
        println!("  ❌ Failed Tests:");
        for test in &results.test_details {
            if test.status == TestStatus::Failed {
                println!("    - {}", test.test_name);
                if let Some(error) = &test.error_message {
                    println!("      Error: {}", error);
                }
            }
        }
    }
    println!();
}

/// Generate comprehensive test report
async fn generate_test_report(
    results: &[IntegrationTestResults],
    summary: &TestSummary,
    duration: std::time::Duration,
) -> Result<(), Box<dyn std::error::Error>> {
    use std::fs::File;
    use std::io::Write;

    let report_content = serde_json::json!({
        "test_run_summary": {
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "total_duration_seconds": duration.as_secs_f64(),
            "total_test_suites": summary.total_test_suites,
            "total_tests": summary.total_tests,
            "passed_tests": summary.passed_tests,
            "failed_tests": summary.failed_tests,
            "success_rate": summary.success_rate
        },
        "test_suites": results,
        "environment": {
            "api_gateway_url": "http://localhost:8080",
            "frontend_shell_url": "http://localhost:3000",
            "database_url": "postgres://localhost:5432/adx_core_test",
            "temporal_url": "http://localhost:7233"
        }
    });

    // Write JSON report
    let mut json_file = File::create("integration_test_report.json")?;
    json_file.write_all(serde_json::to_string_pretty(&report_content)?.as_bytes())?;

    // Write HTML report
    let html_content = generate_html_report(&report_content)?;
    let mut html_file = File::create("integration_test_report.html")?;
    html_file.write_all(html_content.as_bytes())?;

    println!("📄 Test reports generated:");
    println!("  - integration_test_report.json");
    println!("  - integration_test_report.html");

    Ok(())
}

/// Generate HTML test report
fn generate_html_report(report_data: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let html = format!(r#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>ADX CORE Integration Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; background-color: #f5f5f5; }}
        .container {{ max-width: 1200px; margin: 0 auto; background: white; padding: 20px; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }}
        .header {{ text-align: center; margin-bottom: 30px; }}
        .summary {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin-bottom: 30px; }}
        .summary-card {{ background: #f8f9fa; padding: 20px; border-radius: 8px; text-align: center; }}
        .summary-card h3 {{ margin: 0 0 10px 0; color: #333; }}
        .summary-card .value {{ font-size: 2em; font-weight: bold; }}
        .passed {{ color: #28a745; }}
        .failed {{ color: #dc3545; }}
        .test-suite {{ margin-bottom: 30px; border: 1px solid #ddd; border-radius: 8px; overflow: hidden; }}
        .test-suite-header {{ background: #007bff; color: white; padding: 15px; }}
        .test-suite-content {{ padding: 15px; }}
        .test-item {{ margin-bottom: 15px; padding: 10px; border-left: 4px solid #ddd; }}
        .test-passed {{ border-left-color: #28a745; background: #f8fff9; }}
        .test-failed {{ border-left-color: #dc3545; background: #fff8f8; }}
        .test-skipped {{ border-left-color: #ffc107; background: #fffdf5; }}
        .assertions {{ margin-top: 10px; }}
        .assertion {{ margin: 5px 0; padding: 5px; font-size: 0.9em; }}
        .assertion.passed {{ color: #28a745; }}
        .assertion.failed {{ color: #dc3545; }}
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>ADX CORE Integration Test Report</h1>
            <p>Generated on {}</p>
        </div>
        
        <div class="summary">
            <div class="summary-card">
                <h3>Total Tests</h3>
                <div class="value">{}</div>
            </div>
            <div class="summary-card">
                <h3>Passed</h3>
                <div class="value passed">{}</div>
            </div>
            <div class="summary-card">
                <h3>Failed</h3>
                <div class="value failed">{}</div>
            </div>
            <div class="summary-card">
                <h3>Success Rate</h3>
                <div class="value">{:.1}%</div>
            </div>
            <div class="summary-card">
                <h3>Duration</h3>
                <div class="value">{:.2}s</div>
            </div>
        </div>
        
        <div class="test-suites">
            {}
        </div>
    </div>
</body>
</html>
"#,
        report_data["test_run_summary"]["timestamp"].as_str().unwrap_or("Unknown"),
        report_data["test_run_summary"]["total_tests"].as_u64().unwrap_or(0),
        report_data["test_run_summary"]["passed_tests"].as_u64().unwrap_or(0),
        report_data["test_run_summary"]["failed_tests"].as_u64().unwrap_or(0),
        report_data["test_run_summary"]["success_rate"].as_f64().unwrap_or(0.0),
        report_data["test_run_summary"]["total_duration_seconds"].as_f64().unwrap_or(0.0),
        generate_test_suites_html(&report_data["test_suites"])?
    );

    Ok(html)
}

/// Generate HTML for test suites
fn generate_test_suites_html(test_suites: &serde_json::Value) -> Result<String, Box<dyn std::error::Error>> {
    let mut html = String::new();
    
    if let Some(suites) = test_suites.as_array() {
        for suite in suites {
            let suite_name = suite["test_suite"].as_str().unwrap_or("Unknown Suite");
            let total_tests = suite["total_tests"].as_u64().unwrap_or(0);
            let passed_tests = suite["passed_tests"].as_u64().unwrap_or(0);
            let failed_tests = suite["failed_tests"].as_u64().unwrap_or(0);
            let execution_time = suite["execution_time_ms"].as_u64().unwrap_or(0) as f64 / 1000.0;
            
            html.push_str(&format!(r#"
                <div class="test-suite">
                    <div class="test-suite-header">
                        <h2>{}</h2>
                        <p>Tests: {} | Passed: {} | Failed: {} | Time: {:.2}s</p>
                    </div>
                    <div class="test-suite-content">
            "#, suite_name, total_tests, passed_tests, failed_tests, execution_time));
            
            if let Some(test_details) = suite["test_details"].as_array() {
                for test in test_details {
                    let test_name = test["test_name"].as_str().unwrap_or("Unknown Test");
                    let status = test["status"].as_str().unwrap_or("Unknown");
                    let test_time = test["execution_time_ms"].as_u64().unwrap_or(0) as f64 / 1000.0;
                    
                    let status_class = match status {
                        "Passed" => "test-passed",
                        "Failed" => "test-failed",
                        "Skipped" => "test-skipped",
                        _ => "",
                    };
                    
                    html.push_str(&format!(r#"
                        <div class="test-item {}">
                            <h4>{} - {} ({:.2}s)</h4>
                    "#, status_class, test_name, status, test_time));
                    
                    if let Some(error) = test["error_message"].as_str() {
                        html.push_str(&format!("<p><strong>Error:</strong> {}</p>", error));
                    }
                    
                    if let Some(assertions) = test["assertions"].as_array() {
                        html.push_str("<div class=\"assertions\">");
                        for assertion in assertions {
                            let desc = assertion["description"].as_str().unwrap_or("");
                            let passed = assertion["passed"].as_bool().unwrap_or(false);
                            let expected = assertion["expected"].as_str().unwrap_or("");
                            let actual = assertion["actual"].as_str().unwrap_or("");
                            
                            let assertion_class = if passed { "passed" } else { "failed" };
                            let icon = if passed { "✅" } else { "❌" };
                            
                            html.push_str(&format!(r#"
                                <div class="assertion {}">
                                    {} {} (Expected: {}, Actual: {})
                                </div>
                            "#, assertion_class, icon, desc, expected, actual));
                        }
                        html.push_str("</div>");
                    }
                    
                    html.push_str("</div>");
                }
            }
            
            html.push_str("</div></div>");
        }
    }
    
    Ok(html)
}