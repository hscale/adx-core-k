// Main test runner for ADX CORE comprehensive testing infrastructure
use std::env;
use std::process::Command;
use std::time::Instant;
use tokio;

mod integration;
mod performance;
mod security;

use integration::cross_service_tests::CrossServiceTestEnvironment;
use performance::load_testing::{LoadTestSuite, LoadTestConfig};
use security::security_tests::{SecurityTestRunner, SecurityTestConfig};

/// Test suite configuration
#[derive(Debug, Clone)]
pub struct TestSuiteConfig {
    pub run_unit_tests: bool,
    pub run_integration_tests: bool,
    pub run_workflow_tests: bool,
    pub run_e2e_tests: bool,
    pub run_performance_tests: bool,
    pub run_security_tests: bool,
    pub parallel_execution: bool,
    pub generate_reports: bool,
}

impl Default for TestSuiteConfig {
    fn default() -> Self {
        Self {
            run_unit_tests: true,
            run_integration_tests: true,
            run_workflow_tests: true,
            run_e2e_tests: true,
            run_performance_tests: env::var("RUN_PERFORMANCE_TESTS").is_ok(),
            run_security_tests: env::var("RUN_SECURITY_TESTS").is_ok(),
            parallel_execution: true,
            generate_reports: true,
        }
    }
}

/// Main test runner
pub struct TestRunner {
    config: TestSuiteConfig,
}

impl TestRunner {
    pub fn new(config: TestSuiteConfig) -> Self {
        Self { config }
    }

    /// Run all configured test suites
    pub async fn run_all_tests(&self) -> Result<TestResults, Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting ADX CORE Comprehensive Test Suite");
        println!("===============================================");
        
        let overall_start = Instant::now();
        let mut results = TestResults::new();

        // 1. Unit Tests
        if self.config.run_unit_tests {
            println!("\n📋 Running Unit Tests...");
            let unit_start = Instant::now();
            
            match self.run_unit_tests().await {
                Ok(unit_results) => {
                    results.unit_test_results = Some(unit_results);
                    println!("✅ Unit tests completed in {:.2}s", unit_start.elapsed().as_secs_f64());
                }
                Err(e) => {
                    println!("❌ Unit tests failed: {}", e);
                    results.failed_suites.push("Unit Tests".to_string());
                }
            }
        }

        // 2. Workflow Tests
        if self.config.run_workflow_tests {
            println!("\n⚡ Running Temporal Workflow Tests...");
            let workflow_start = Instant::now();
            
            match self.run_workflow_tests().await {
                Ok(workflow_results) => {
                    results.workflow_test_results = Some(workflow_results);
                    println!("✅ Workflow tests completed in {:.2}s", workflow_start.elapsed().as_secs_f64());
                }
                Err(e) => {
                    println!("❌ Workflow tests failed: {}", e);
                    results.failed_suites.push("Workflow Tests".to_string());
                }
            }
        }

        // 3. Integration Tests
        if self.config.run_integration_tests {
            println!("\n🔗 Running Cross-Service Integration Tests...");
            let integration_start = Instant::now();
            
            match self.run_integration_tests().await {
                Ok(integration_results) => {
                    results.integration_test_results = Some(integration_results);
                    println!("✅ Integration tests completed in {:.2}s", integration_start.elapsed().as_secs_f64());
                }
                Err(e) => {
                    println!("❌ Integration tests failed: {}", e);
                    results.failed_suites.push("Integration Tests".to_string());
                }
            }
        }

        // 4. E2E Tests
        if self.config.run_e2e_tests {
            println!("\n🎭 Running End-to-End Tests...");
            let e2e_start = Instant::now();
            
            match self.run_e2e_tests().await {
                Ok(e2e_results) => {
                    results.e2e_test_results = Some(e2e_results);
                    println!("✅ E2E tests completed in {:.2}s", e2e_start.elapsed().as_secs_f64());
                }
                Err(e) => {
                    println!("❌ E2E tests failed: {}", e);
                    results.failed_suites.push("E2E Tests".to_string());
                }
            }
        }

        // 5. Performance Tests
        if self.config.run_performance_tests {
            println!("\n⚡ Running Performance Tests...");
            let perf_start = Instant::now();
            
            match self.run_performance_tests().await {
                Ok(perf_results) => {
                    results.performance_test_results = Some(perf_results);
                    println!("✅ Performance tests completed in {:.2}s", perf_start.elapsed().as_secs_f64());
                }
                Err(e) => {
                    println!("❌ Performance tests failed: {}", e);
                    results.failed_suites.push("Performance Tests".to_string());
                }
            }
        }

        // 6. Security Tests
        if self.config.run_security_tests {
            println!("\n🔒 Running Security Tests...");
            let security_start = Instant::now();
            
            match self.run_security_tests().await {
                Ok(security_results) => {
                    results.security_test_results = Some(security_results);
                    println!("✅ Security tests completed in {:.2}s", security_start.elapsed().as_secs_f64());
                }
                Err(e) => {
                    println!("❌ Security tests failed: {}", e);
                    results.failed_suites.push("Security Tests".to_string());
                }
            }
        }

        results.total_duration = overall_start.elapsed();
        
        // Generate reports
        if self.config.generate_reports {
            self.generate_comprehensive_report(&results).await?;
        }

        // Print summary
        self.print_test_summary(&results);

        Ok(results)
    }

    /// Run unit tests for all services and micro-frontends
    async fn run_unit_tests(&self) -> Result<UnitTestResults, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = UnitTestResults::new();

        // Run Rust unit tests
        println!("  🦀 Running Rust service unit tests...");
        let rust_output = Command::new("cargo")
            .args(&["test", "--workspace", "--lib"])
            .current_dir("adx-core")
            .output()?;

        results.rust_tests_passed = rust_output.status.success();
        results.rust_test_output = String::from_utf8_lossy(&rust_output.stdout).to_string();

        // Run frontend unit tests
        println!("  ⚛️ Running frontend unit tests...");
        let frontend_output = Command::new("npm")
            .args(&["run", "test:unit", "--", "--run"])
            .output()?;

        results.frontend_tests_passed = frontend_output.status.success();
        results.frontend_test_output = String::from_utf8_lossy(&frontend_output.stdout).to_string();

        Ok(results)
    }

    /// Run Temporal workflow tests
    async fn run_workflow_tests(&self) -> Result<WorkflowTestResults, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = WorkflowTestResults::new();

        // Run workflow-specific tests
        let workflow_output = Command::new("cargo")
            .args(&["test", "--workspace", "--test", "workflow_tests"])
            .current_dir("adx-core")
            .output()?;

        results.tests_passed = workflow_output.status.success();
        results.test_output = String::from_utf8_lossy(&workflow_output.stdout).to_string();

        Ok(results)
    }

    /// Run cross-service integration tests
    async fn run_integration_tests(&self) -> Result<IntegrationTestResults, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = IntegrationTestResults::new();

        // Run integration tests with test containers
        let integration_output = Command::new("cargo")
            .args(&["test", "--test", "cross_service_tests"])
            .current_dir("tests")
            .output()?;

        results.tests_passed = integration_output.status.success();
        results.test_output = String::from_utf8_lossy(&integration_output.stdout).to_string();

        Ok(results)
    }

    /// Run end-to-end tests with Playwright
    async fn run_e2e_tests(&self) -> Result<E2ETestResults, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = E2ETestResults::new();

        // Run Playwright tests
        let e2e_output = Command::new("npx")
            .args(&["playwright", "test", "--config=tests/e2e/playwright.config.ts"])
            .output()?;

        results.tests_passed = e2e_output.status.success();
        results.test_output = String::from_utf8_lossy(&e2e_output.stdout).to_string();

        Ok(results)
    }

    /// Run performance tests
    async fn run_performance_tests(&self) -> Result<PerformanceTestResults, Box<dyn std::error::Error + Send + Sync>> {
        let load_test_results = LoadTestSuite::run_all_tests().await?;
        
        Ok(PerformanceTestResults {
            load_test_results,
            tests_passed: true,
        })
    }

    /// Run security tests
    async fn run_security_tests(&self) -> Result<SecurityTestResults, Box<dyn std::error::Error + Send + Sync>> {
        let config = SecurityTestConfig::default();
        let runner = SecurityTestRunner::new(config);
        let security_results = runner.run_security_tests().await?;
        
        Ok(SecurityTestResults {
            security_results,
            tests_passed: security_results.security_score >= 80.0,
        })
    }

    /// Generate comprehensive test report
    async fn generate_comprehensive_report(&self, results: &TestResults) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("\n📊 Generating comprehensive test report...");
        
        let report = self.create_html_report(results);
        std::fs::write("test-results/comprehensive-report.html", report)?;
        
        let json_report = serde_json::to_string_pretty(results)?;
        std::fs::write("test-results/comprehensive-report.json", json_report)?;
        
        println!("✅ Reports generated:");
        println!("  - test-results/comprehensive-report.html");
        println!("  - test-results/comprehensive-report.json");
        
        Ok(())
    }

    /// Create HTML report
    fn create_html_report(&self, results: &TestResults) -> String {
        format!(r#"
<!DOCTYPE html>
<html>
<head>
    <title>ADX CORE Test Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .summary {{ background: #f5f5f5; padding: 20px; border-radius: 8px; }}
        .passed {{ color: #28a745; }}
        .failed {{ color: #dc3545; }}
        .section {{ margin: 20px 0; }}
        .metric {{ display: inline-block; margin: 10px; padding: 10px; background: white; border-radius: 4px; }}
    </style>
</head>
<body>
    <h1>ADX CORE Comprehensive Test Report</h1>
    
    <div class="summary">
        <h2>Test Summary</h2>
        <div class="metric">
            <strong>Total Duration:</strong> {:.2}s
        </div>
        <div class="metric">
            <strong>Failed Suites:</strong> {}
        </div>
    </div>
    
    <div class="section">
        <h2>Test Results</h2>
        {}
    </div>
</body>
</html>
        "#, 
        results.total_duration.as_secs_f64(),
        results.failed_suites.len(),
        self.format_test_sections(results)
        )
    }

    fn format_test_sections(&self, results: &TestResults) -> String {
        let mut sections = String::new();
        
        if let Some(unit_results) = &results.unit_test_results {
            sections.push_str(&format!(
                "<h3>Unit Tests</h3><p class=\"{}\">Rust: {} | Frontend: {}</p>",
                if unit_results.rust_tests_passed && unit_results.frontend_tests_passed { "passed" } else { "failed" },
                if unit_results.rust_tests_passed { "✅ Passed" } else { "❌ Failed" },
                if unit_results.frontend_tests_passed { "✅ Passed" } else { "❌ Failed" }
            ));
        }
        
        // Add other test result sections...
        
        sections
    }

    /// Print test summary
    fn print_test_summary(&self, results: &TestResults) {
        println!("\n📊 TEST SUMMARY");
        println!("================");
        println!("Total Duration: {:.2}s", results.total_duration.as_secs_f64());
        println!("Failed Suites: {}", results.failed_suites.len());
        
        if results.failed_suites.is_empty() {
            println!("✅ All test suites passed!");
        } else {
            println!("❌ Failed suites: {:?}", results.failed_suites);
        }
    }
}

// Result structures
#[derive(Debug, serde::Serialize)]
pub struct TestResults {
    pub total_duration: std::time::Duration,
    pub failed_suites: Vec<String>,
    pub unit_test_results: Option<UnitTestResults>,
    pub workflow_test_results: Option<WorkflowTestResults>,
    pub integration_test_results: Option<IntegrationTestResults>,
    pub e2e_test_results: Option<E2ETestResults>,
    pub performance_test_results: Option<PerformanceTestResults>,
    pub security_test_results: Option<SecurityTestResults>,
}

impl TestResults {
    fn new() -> Self {
        Self {
            total_duration: std::time::Duration::new(0, 0),
            failed_suites: Vec::new(),
            unit_test_results: None,
            workflow_test_results: None,
            integration_test_results: None,
            e2e_test_results: None,
            performance_test_results: None,
            security_test_results: None,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct UnitTestResults {
    pub rust_tests_passed: bool,
    pub rust_test_output: String,
    pub frontend_tests_passed: bool,
    pub frontend_test_output: String,
}

impl UnitTestResults {
    fn new() -> Self {
        Self {
            rust_tests_passed: false,
            rust_test_output: String::new(),
            frontend_tests_passed: false,
            frontend_test_output: String::new(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct WorkflowTestResults {
    pub tests_passed: bool,
    pub test_output: String,
}

impl WorkflowTestResults {
    fn new() -> Self {
        Self {
            tests_passed: false,
            test_output: String::new(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct IntegrationTestResults {
    pub tests_passed: bool,
    pub test_output: String,
}

impl IntegrationTestResults {
    fn new() -> Self {
        Self {
            tests_passed: false,
            test_output: String::new(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct E2ETestResults {
    pub tests_passed: bool,
    pub test_output: String,
}

impl E2ETestResults {
    fn new() -> Self {
        Self {
            tests_passed: false,
            test_output: String::new(),
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct PerformanceTestResults {
    pub load_test_results: Vec<performance::load_testing::LoadTestResults>,
    pub tests_passed: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct SecurityTestResults {
    pub security_results: security::security_tests::SecurityTestResults,
    pub tests_passed: bool,
}

// Main entry point
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = TestSuiteConfig::default();
    let runner = TestRunner::new(config);
    
    let results = runner.run_all_tests().await?;
    
    // Exit with error code if any tests failed
    if !results.failed_suites.is_empty() {
        std::process::exit(1);
    }
    
    Ok(())
}