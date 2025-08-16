// Load testing for ADX CORE services and workflows
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::{Semaphore, RwLock};
use tokio::time::sleep;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use reqwest::Client as HttpClient;
use futures::future::join_all;

/// Load test configuration
#[derive(Debug, Clone)]
pub struct LoadTestConfig {
    pub base_url: String,
    pub concurrent_users: usize,
    pub test_duration_seconds: u64,
    pub ramp_up_seconds: u64,
    pub requests_per_second: f64,
    pub timeout_seconds: u64,
}

impl Default for LoadTestConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8080".to_string(),
            concurrent_users: 100,
            test_duration_seconds: 300, // 5 minutes
            ramp_up_seconds: 60,        // 1 minute ramp-up
            requests_per_second: 10.0,
            timeout_seconds: 30,
        }
    }
}

/// Load test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadTestResults {
    pub test_name: String,
    pub config: LoadTestConfig,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub response_times: ResponseTimeStats,
    pub throughput_rps: f64,
    pub error_rate: f64,
    pub errors: HashMap<String, u64>,
    pub workflow_stats: Option<WorkflowStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseTimeStats {
    pub min_ms: f64,
    pub max_ms: f64,
    pub mean_ms: f64,
    pub median_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStats {
    pub total_workflows: u64,
    pub completed_workflows: u64,
    pub failed_workflows: u64,
    pub average_execution_time_ms: f64,
    pub workflow_throughput_per_minute: f64,
}

/// Load test runner
pub struct LoadTestRunner {
    config: LoadTestConfig,
    http_client: HttpClient,
    results: Arc<RwLock<Vec<RequestResult>>>,
    workflow_results: Arc<RwLock<Vec<WorkflowResult>>>,
}

#[derive(Debug, Clone)]
struct RequestResult {
    timestamp: Instant,
    duration: Duration,
    status_code: u16,
    success: bool,
    error: Option<String>,
    endpoint: String,
}

#[derive(Debug, Clone)]
struct WorkflowResult {
    workflow_id: String,
    workflow_type: String,
    start_time: Instant,
    end_time: Option<Instant>,
    status: String,
    error: Option<String>,
}

impl LoadTestRunner {
    pub fn new(config: LoadTestConfig) -> Self {
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(config.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
            results: Arc::new(RwLock::new(Vec::new())),
            workflow_results: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Run API endpoint load test
    pub async fn run_api_load_test(&self, test_name: &str) -> Result<LoadTestResults, Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting API load test: {}", test_name);
        let start_time = Utc::now();
        let test_start = Instant::now();

        // Create semaphore to limit concurrent requests
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_users));
        
        // Calculate request interval
        let request_interval = Duration::from_secs_f64(1.0 / self.config.requests_per_second);
        
        let mut handles = Vec::new();
        let test_duration = Duration::from_secs(self.config.test_duration_seconds);
        
        // Spawn request generators
        for user_id in 0..self.config.concurrent_users {
            let semaphore = semaphore.clone();
            let http_client = self.http_client.clone();
            let base_url = self.config.base_url.clone();
            let results = self.results.clone();
            
            let handle = tokio::spawn(async move {
                let mut request_count = 0;
                let user_start = Instant::now();
                
                while user_start.elapsed() < test_duration {
                    let _permit = semaphore.acquire().await.unwrap();
                    
                    // Make API request
                    let request_start = Instant::now();
                    let endpoint = format!("{}/api/v1/health", base_url);
                    
                    let result = match http_client.get(&endpoint).send().await {
                        Ok(response) => {
                            let status_code = response.status().as_u16();
                            RequestResult {
                                timestamp: request_start,
                                duration: request_start.elapsed(),
                                status_code,
                                success: response.status().is_success(),
                                error: None,
                                endpoint: endpoint.clone(),
                            }
                        }
                        Err(e) => RequestResult {
                            timestamp: request_start,
                            duration: request_start.elapsed(),
                            status_code: 0,
                            success: false,
                            error: Some(e.to_string()),
                            endpoint: endpoint.clone(),
                        }
                    };
                    
                    results.write().await.push(result);
                    request_count += 1;
                    
                    // Wait before next request
                    sleep(request_interval).await;
                }
                
                request_count
            });
            
            handles.push(handle);
        }

        // Wait for all request generators to complete
        let request_counts = join_all(handles).await;
        let total_requests: u64 = request_counts.into_iter()
            .map(|r| r.unwrap_or(0) as u64)
            .sum();

        let end_time = Utc::now();
        let test_duration_actual = test_start.elapsed();

        // Analyze results
        let results = self.results.read().await;
        let analysis = self.analyze_results(&results, test_duration_actual);

        Ok(LoadTestResults {
            test_name: test_name.to_string(),
            config: self.config.clone(),
            start_time,
            end_time,
            total_requests,
            successful_requests: analysis.successful_requests,
            failed_requests: analysis.failed_requests,
            response_times: analysis.response_times,
            throughput_rps: analysis.throughput_rps,
            error_rate: analysis.error_rate,
            errors: analysis.errors,
            workflow_stats: None,
        })
    }

    /// Run workflow load test
    pub async fn run_workflow_load_test(&self, test_name: &str) -> Result<LoadTestResults, Box<dyn std::error::Error + Send + Sync>> {
        println!("🚀 Starting workflow load test: {}", test_name);
        let start_time = Utc::now();
        let test_start = Instant::now();

        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_users));
        let mut handles = Vec::new();
        let test_duration = Duration::from_secs(self.config.test_duration_seconds);

        // Spawn workflow generators
        for user_id in 0..self.config.concurrent_users {
            let semaphore = semaphore.clone();
            let http_client = self.http_client.clone();
            let base_url = self.config.base_url.clone();
            let results = self.results.clone();
            let workflow_results = self.workflow_results.clone();

            let handle = tokio::spawn(async move {
                let mut workflow_count = 0;
                let user_start = Instant::now();

                while user_start.elapsed() < test_duration {
                    let _permit = semaphore.acquire().await.unwrap();

                    // Start a workflow
                    let workflow_id = Uuid::new_v4().to_string();
                    let workflow_start = Instant::now();
                    
                    let workflow_request = serde_json::json!({
                        "tenant_name": format!("Load Test Tenant {}", user_id),
                        "admin_email": format!("admin{}@loadtest.com", user_id),
                        "subscription_tier": "professional"
                    });

                    let endpoint = format!("{}/api/v1/workflows/create-tenant", base_url);
                    
                    match http_client.post(&endpoint).json(&workflow_request).send().await {
                        Ok(response) => {
                            let status_code = response.status().as_u16();
                            let success = response.status().is_success();
                            
                            // Record API request result
                            results.write().await.push(RequestResult {
                                timestamp: workflow_start,
                                duration: workflow_start.elapsed(),
                                status_code,
                                success,
                                error: None,
                                endpoint: endpoint.clone(),
                            });

                            if success {
                                // Parse workflow response
                                if let Ok(workflow_response) = response.json::<serde_json::Value>().await {
                                    if let Some(operation_id) = workflow_response["operation_id"].as_str() {
                                        // Poll workflow status
                                        let workflow_result = Self::poll_workflow_completion(
                                            &http_client,
                                            &base_url,
                                            operation_id,
                                            workflow_start,
                                        ).await;
                                        
                                        workflow_results.write().await.push(workflow_result);
                                    }
                                }
                            } else {
                                // Record failed workflow
                                workflow_results.write().await.push(WorkflowResult {
                                    workflow_id: workflow_id.clone(),
                                    workflow_type: "create_tenant".to_string(),
                                    start_time: workflow_start,
                                    end_time: Some(workflow_start),
                                    status: "failed".to_string(),
                                    error: Some(format!("HTTP {}", status_code)),
                                });
                            }
                        }
                        Err(e) => {
                            // Record request error
                            results.write().await.push(RequestResult {
                                timestamp: workflow_start,
                                duration: workflow_start.elapsed(),
                                status_code: 0,
                                success: false,
                                error: Some(e.to_string()),
                                endpoint: endpoint.clone(),
                            });

                            // Record failed workflow
                            workflow_results.write().await.push(WorkflowResult {
                                workflow_id: workflow_id.clone(),
                                workflow_type: "create_tenant".to_string(),
                                start_time: workflow_start,
                                end_time: Some(workflow_start),
                                status: "failed".to_string(),
                                error: Some(e.to_string()),
                            });
                        }
                    }

                    workflow_count += 1;
                    
                    // Wait before next workflow
                    sleep(Duration::from_secs_f64(1.0 / self.config.requests_per_second)).await;
                }

                workflow_count
            });

            handles.push(handle);
        }

        // Wait for all workflow generators to complete
        let workflow_counts = join_all(handles).await;
        let total_workflows: u64 = workflow_counts.into_iter()
            .map(|r| r.unwrap_or(0) as u64)
            .sum();

        let end_time = Utc::now();
        let test_duration_actual = test_start.elapsed();

        // Analyze results
        let results = self.results.read().await;
        let workflow_results = self.workflow_results.read().await;
        let analysis = self.analyze_results(&results, test_duration_actual);
        let workflow_analysis = self.analyze_workflow_results(&workflow_results, test_duration_actual);

        Ok(LoadTestResults {
            test_name: test_name.to_string(),
            config: self.config.clone(),
            start_time,
            end_time,
            total_requests: analysis.total_requests,
            successful_requests: analysis.successful_requests,
            failed_requests: analysis.failed_requests,
            response_times: analysis.response_times,
            throughput_rps: analysis.throughput_rps,
            error_rate: analysis.error_rate,
            errors: analysis.errors,
            workflow_stats: Some(workflow_analysis),
        })
    }

    async fn poll_workflow_completion(
        http_client: &HttpClient,
        base_url: &str,
        operation_id: &str,
        start_time: Instant,
    ) -> WorkflowResult {
        let max_polls = 60; // 5 minutes max
        let poll_interval = Duration::from_secs(5);
        
        for _ in 0..max_polls {
            sleep(poll_interval).await;
            
            let status_url = format!("{}/api/v1/workflows/{}/status", base_url, operation_id);
            
            match http_client.get(&status_url).send().await {
                Ok(response) => {
                    if let Ok(status) = response.json::<serde_json::Value>().await {
                        match status["status"].as_str() {
                            Some("completed") => {
                                return WorkflowResult {
                                    workflow_id: operation_id.to_string(),
                                    workflow_type: "create_tenant".to_string(),
                                    start_time,
                                    end_time: Some(Instant::now()),
                                    status: "completed".to_string(),
                                    error: None,
                                };
                            }
                            Some("failed") => {
                                return WorkflowResult {
                                    workflow_id: operation_id.to_string(),
                                    workflow_type: "create_tenant".to_string(),
                                    start_time,
                                    end_time: Some(Instant::now()),
                                    status: "failed".to_string(),
                                    error: status["error"].as_str().map(|s| s.to_string()),
                                };
                            }
                            _ => continue, // Still running
                        }
                    }
                }
                Err(_) => continue, // Retry on error
            }
        }
        
        // Timeout
        WorkflowResult {
            workflow_id: operation_id.to_string(),
            workflow_type: "create_tenant".to_string(),
            start_time,
            end_time: Some(Instant::now()),
            status: "timeout".to_string(),
            error: Some("Workflow polling timeout".to_string()),
        }
    }

    fn analyze_results(&self, results: &[RequestResult], test_duration: Duration) -> ResultAnalysis {
        let total_requests = results.len() as u64;
        let successful_requests = results.iter().filter(|r| r.success).count() as u64;
        let failed_requests = total_requests - successful_requests;
        
        // Calculate response time statistics
        let mut response_times: Vec<f64> = results.iter()
            .map(|r| r.duration.as_secs_f64() * 1000.0) // Convert to milliseconds
            .collect();
        response_times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let response_time_stats = if !response_times.is_empty() {
            ResponseTimeStats {
                min_ms: response_times[0],
                max_ms: response_times[response_times.len() - 1],
                mean_ms: response_times.iter().sum::<f64>() / response_times.len() as f64,
                median_ms: response_times[response_times.len() / 2],
                p95_ms: response_times[(response_times.len() as f64 * 0.95) as usize],
                p99_ms: response_times[(response_times.len() as f64 * 0.99) as usize],
            }
        } else {
            ResponseTimeStats {
                min_ms: 0.0,
                max_ms: 0.0,
                mean_ms: 0.0,
                median_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
            }
        };
        
        // Calculate throughput
        let throughput_rps = total_requests as f64 / test_duration.as_secs_f64();
        
        // Calculate error rate
        let error_rate = if total_requests > 0 {
            (failed_requests as f64 / total_requests as f64) * 100.0
        } else {
            0.0
        };
        
        // Collect error types
        let mut errors = HashMap::new();
        for result in results {
            if !result.success {
                if let Some(error) = &result.error {
                    *errors.entry(error.clone()).or_insert(0) += 1;
                } else {
                    *errors.entry(format!("HTTP {}", result.status_code)).or_insert(0) += 1;
                }
            }
        }
        
        ResultAnalysis {
            total_requests,
            successful_requests,
            failed_requests,
            response_times: response_time_stats,
            throughput_rps,
            error_rate,
            errors,
        }
    }

    fn analyze_workflow_results(&self, workflow_results: &[WorkflowResult], test_duration: Duration) -> WorkflowStats {
        let total_workflows = workflow_results.len() as u64;
        let completed_workflows = workflow_results.iter()
            .filter(|w| w.status == "completed")
            .count() as u64;
        let failed_workflows = total_workflows - completed_workflows;
        
        // Calculate average execution time
        let execution_times: Vec<f64> = workflow_results.iter()
            .filter_map(|w| {
                if let Some(end_time) = w.end_time {
                    Some(end_time.duration_since(w.start_time).as_secs_f64() * 1000.0)
                } else {
                    None
                }
            })
            .collect();
        
        let average_execution_time_ms = if !execution_times.is_empty() {
            execution_times.iter().sum::<f64>() / execution_times.len() as f64
        } else {
            0.0
        };
        
        // Calculate workflow throughput
        let workflow_throughput_per_minute = (total_workflows as f64 / test_duration.as_secs_f64()) * 60.0;
        
        WorkflowStats {
            total_workflows,
            completed_workflows,
            failed_workflows,
            average_execution_time_ms,
            workflow_throughput_per_minute,
        }
    }
}

struct ResultAnalysis {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    response_times: ResponseTimeStats,
    throughput_rps: f64,
    error_rate: f64,
    errors: HashMap<String, u64>,
}

/// Load test suite
pub struct LoadTestSuite;

impl LoadTestSuite {
    /// Run comprehensive load tests
    pub async fn run_all_tests() -> Result<Vec<LoadTestResults>, Box<dyn std::error::Error + Send + Sync>> {
        let mut results = Vec::new();
        
        // Test 1: API Gateway health endpoint
        let config = LoadTestConfig {
            concurrent_users: 50,
            test_duration_seconds: 60,
            requests_per_second: 100.0,
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        let api_results = runner.run_api_load_test("API Gateway Health Check").await?;
        results.push(api_results);
        
        // Test 2: Tenant creation workflow
        let config = LoadTestConfig {
            concurrent_users: 10,
            test_duration_seconds: 300,
            requests_per_second: 2.0,
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        let workflow_results = runner.run_workflow_load_test("Tenant Creation Workflow").await?;
        results.push(workflow_results);
        
        // Test 3: High concurrency API test
        let config = LoadTestConfig {
            concurrent_users: 200,
            test_duration_seconds: 120,
            requests_per_second: 500.0,
            ..Default::default()
        };
        
        let runner = LoadTestRunner::new(config);
        let high_load_results = runner.run_api_load_test("High Concurrency API Test").await?;
        results.push(high_load_results);
        
        Ok(results)
    }
    
    /// Generate load test report
    pub fn generate_report(results: &[LoadTestResults]) -> String {
        let mut report = String::new();
        
        report.push_str("# ADX CORE Load Test Report\n\n");
        report.push_str(&format!("Generated at: {}\n\n", Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        
        for result in results {
            report.push_str(&format!("## {}\n\n", result.test_name));
            report.push_str(&format!("- **Duration**: {:.2}s\n", 
                result.end_time.signed_duration_since(result.start_time).num_seconds()));
            report.push_str(&format!("- **Total Requests**: {}\n", result.total_requests));
            report.push_str(&format!("- **Successful Requests**: {}\n", result.successful_requests));
            report.push_str(&format!("- **Failed Requests**: {}\n", result.failed_requests));
            report.push_str(&format!("- **Success Rate**: {:.2}%\n", 
                (result.successful_requests as f64 / result.total_requests as f64) * 100.0));
            report.push_str(&format!("- **Throughput**: {:.2} RPS\n", result.throughput_rps));
            report.push_str(&format!("- **Error Rate**: {:.2}%\n", result.error_rate));
            
            report.push_str("\n### Response Times\n\n");
            report.push_str(&format!("- **Min**: {:.2}ms\n", result.response_times.min_ms));
            report.push_str(&format!("- **Max**: {:.2}ms\n", result.response_times.max_ms));
            report.push_str(&format!("- **Mean**: {:.2}ms\n", result.response_times.mean_ms));
            report.push_str(&format!("- **Median**: {:.2}ms\n", result.response_times.median_ms));
            report.push_str(&format!("- **95th Percentile**: {:.2}ms\n", result.response_times.p95_ms));
            report.push_str(&format!("- **99th Percentile**: {:.2}ms\n", result.response_times.p99_ms));
            
            if let Some(workflow_stats) = &result.workflow_stats {
                report.push_str("\n### Workflow Statistics\n\n");
                report.push_str(&format!("- **Total Workflows**: {}\n", workflow_stats.total_workflows));
                report.push_str(&format!("- **Completed Workflows**: {}\n", workflow_stats.completed_workflows));
                report.push_str(&format!("- **Failed Workflows**: {}\n", workflow_stats.failed_workflows));
                report.push_str(&format!("- **Average Execution Time**: {:.2}ms\n", workflow_stats.average_execution_time_ms));
                report.push_str(&format!("- **Workflow Throughput**: {:.2} workflows/minute\n", workflow_stats.workflow_throughput_per_minute));
            }
            
            if !result.errors.is_empty() {
                report.push_str("\n### Errors\n\n");
                for (error, count) in &result.errors {
                    report.push_str(&format!("- **{}**: {} occurrences\n", error, count));
                }
            }
            
            report.push_str("\n---\n\n");
        }
        
        report
    }
}

// Integration test for load testing
#[tokio::test]
async fn test_load_testing_suite() {
    // This test would run in a CI environment with services running
    if std::env::var("RUN_LOAD_TESTS").is_ok() {
        let results = LoadTestSuite::run_all_tests().await
            .expect("Load tests failed");
        
        // Generate and save report
        let report = LoadTestSuite::generate_report(&results);
        std::fs::write("load_test_report.md", report)
            .expect("Failed to write load test report");
        
        // Assert performance criteria
        for result in &results {
            // API response time should be under 200ms for 95th percentile
            if !result.test_name.contains("Workflow") {
                assert!(result.response_times.p95_ms < 200.0, 
                    "95th percentile response time too high: {:.2}ms", result.response_times.p95_ms);
            }
            
            // Error rate should be under 1%
            assert!(result.error_rate < 1.0, 
                "Error rate too high: {:.2}%", result.error_rate);
            
            // Throughput should meet minimum requirements
            if result.test_name.contains("High Concurrency") {
                assert!(result.throughput_rps > 400.0, 
                    "Throughput too low: {:.2} RPS", result.throughput_rps);
            }
        }
        
        println!("✅ All load tests passed performance criteria");
    } else {
        println!("⏭️ Skipping load tests (set RUN_LOAD_TESTS=1 to enable)");
    }
}