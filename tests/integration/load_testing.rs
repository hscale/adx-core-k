// Load testing and performance validation
use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use futures::future::join_all;

/// Load testing suite for performance validation
pub struct LoadTestingSuite {
    env: Arc<IntegrationTestEnvironment>,
    test_data: TestData,
    metrics: Arc<LoadTestMetrics>,
}

#[derive(Debug)]
pub struct LoadTestMetrics {
    pub total_requests: AtomicU64,
    pub successful_requests: AtomicU64,
    pub failed_requests: AtomicU64,
    pub total_response_time_ms: AtomicU64,
    pub min_response_time_ms: AtomicU64,
    pub max_response_time_ms: AtomicU64,
}

impl LoadTestMetrics {
    pub fn new() -> Self {
        Self {
            total_requests: AtomicU64::new(0),
            successful_requests: AtomicU64::new(0),
            failed_requests: AtomicU64::new(0),
            total_response_time_ms: AtomicU64::new(0),
            min_response_time_ms: AtomicU64::new(u64::MAX),
            max_response_time_ms: AtomicU64::new(0),
        }
    }

    pub fn record_request(&self, response_time_ms: u64, success: bool) {
        self.total_requests.fetch_add(1, Ordering::Relaxed);
        self.total_response_time_ms.fetch_add(response_time_ms, Ordering::Relaxed);
        
        if success {
            self.successful_requests.fetch_add(1, Ordering::Relaxed);
        } else {
            self.failed_requests.fetch_add(1, Ordering::Relaxed);
        }

        // Update min/max response times
        let current_min = self.min_response_time_ms.load(Ordering::Relaxed);
        if response_time_ms < current_min {
            self.min_response_time_ms.store(response_time_ms, Ordering::Relaxed);
        }

        let current_max = self.max_response_time_ms.load(Ordering::Relaxed);
        if response_time_ms > current_max {
            self.max_response_time_ms.store(response_time_ms, Ordering::Relaxed);
        }
    }

    pub fn get_summary(&self) -> LoadTestSummary {
        let total = self.total_requests.load(Ordering::Relaxed);
        let successful = self.successful_requests.load(Ordering::Relaxed);
        let failed = self.failed_requests.load(Ordering::Relaxed);
        let total_time = self.total_response_time_ms.load(Ordering::Relaxed);

        LoadTestSummary {
            total_requests: total,
            successful_requests: successful,
            failed_requests: failed,
            success_rate: if total > 0 { (successful as f64 / total as f64) * 100.0 } else { 0.0 },
            average_response_time_ms: if total > 0 { total_time as f64 / total as f64 } else { 0.0 },
            min_response_time_ms: if self.min_response_time_ms.load(Ordering::Relaxed) == u64::MAX { 0 } else { self.min_response_time_ms.load(Ordering::Relaxed) },
            max_response_time_ms: self.max_response_time_ms.load(Ordering::Relaxed),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadTestSummary {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub success_rate: f64,
    pub average_response_time_ms: f64,
    pub min_response_time_ms: u64,
    pub max_response_time_ms: u64,
}

impl LoadTestingSuite {
    pub fn new(env: Arc<IntegrationTestEnvironment>, test_data: TestData) -> Self {
        Self {
            env,
            test_data,
            metrics: Arc::new(LoadTestMetrics::new()),
        }
    }

    /// Run all load tests
    pub async fn run_all_tests(&self) -> Result<IntegrationTestResults, Box<dyn std::error::Error>> {
        if !self.env.config.enable_load_testing {
            return Ok(IntegrationTestResults {
                test_suite: "Load Testing".to_string(),
                total_tests: 0,
                passed_tests: 0,
                failed_tests: 0,
                skipped_tests: 1,
                execution_time_ms: 0,
                test_details: vec![TestResult {
                    test_name: "Load Testing".to_string(),
                    status: TestStatus::Skipped,
                    execution_time_ms: 0,
                    error_message: Some("Load testing disabled in configuration".to_string()),
                    assertions: vec![],
                }],
                performance_metrics: PerformanceMetrics {
                    average_response_time_ms: 0.0,
                    p95_response_time_ms: 0.0,
                    p99_response_time_ms: 0.0,
                    throughput_requests_per_second: 0.0,
                    error_rate_percentage: 0.0,
                    memory_usage_mb: 0.0,
                    cpu_usage_percentage: 0.0,
                },
                errors: Vec::new(),
            });
        }

        let start_time = Instant::now();
        let mut test_results = Vec::new();

        // Test API endpoint load
        test_results.push(self.test_api_endpoint_load().await);

        // Test workflow execution load
        test_results.push(self.test_workflow_execution_load().await);

        // Test concurrent user simulation
        test_results.push(self.test_concurrent_user_simulation().await);

        // Test database performance under load
        test_results.push(self.test_database_performance_load().await);

        // Test file upload load
        test_results.push(self.test_file_upload_load().await);

        // Test multi-tenant load isolation
        test_results.push(self.test_multi_tenant_load_isolation().await);

        let execution_time = start_time.elapsed().as_millis() as u64;
        let passed_tests = test_results.iter().filter(|r| r.status == TestStatus::Passed).count() as u32;
        let failed_tests = test_results.iter().filter(|r| r.status == TestStatus::Failed).count() as u32;

        let load_summary = self.metrics.get_summary();

        Ok(IntegrationTestResults {
            test_suite: "Load Testing".to_string(),
            total_tests: test_results.len() as u32,
            passed_tests,
            failed_tests,
            skipped_tests: 0,
            execution_time_ms: execution_time,
            test_details: test_results,
            performance_metrics: PerformanceMetrics {
                average_response_time_ms: load_summary.average_response_time_ms,
                p95_response_time_ms: 0.0, // Would need percentile calculation
                p99_response_time_ms: 0.0, // Would need percentile calculation
                throughput_requests_per_second: if execution_time > 0 { (load_summary.total_requests as f64 / (execution_time as f64 / 1000.0)) } else { 0.0 },
                error_rate_percentage: 100.0 - load_summary.success_rate,
                memory_usage_mb: 0.0, // Would need system monitoring
                cpu_usage_percentage: 0.0, // Would need system monitoring
            },
            errors: Vec::new(),
        })
    }

    /// Test API endpoint load
    async fn test_api_endpoint_load(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Login to get auth token
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&serde_json::json!({
                "email": test_user.email,
                "password": "password123",
                "tenant_id": tenant_id
            }))
            .send()
            .await;

        let auth_token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "API Endpoint Load".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "API Endpoint Load".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Concurrent API calls
        let concurrent_requests = 50;
        let mut handles = Vec::new();

        for i in 0..concurrent_requests {
            let client = self.env.http_client.clone();
            let token = auth_token.clone();
            let tenant = tenant_id.clone();
            let api_url = self.env.config.api_gateway_url.clone();
            let metrics = self.metrics.clone();

            let handle = tokio::spawn(async move {
                let request_start = Instant::now();
                
                let response = client
                    .get(&format!("{}/api/v1/users/profile", api_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-Tenant-ID", &tenant)
                    .send()
                    .await;

                let response_time = request_start.elapsed().as_millis() as u64;
                let success = response.map(|r| r.status().is_success()).unwrap_or(false);
                
                metrics.record_request(response_time, success);
                (i, success, response_time)
            });

            handles.push(handle);
        }

        // Wait for all requests to complete
        let results = join_all(handles).await;
        let successful_requests = results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success, _)| *success)
            .count();

        let success_rate = (successful_requests as f64 / concurrent_requests as f64) * 100.0;

        assertions.push(AssertionResult {
            description: "API endpoints should handle concurrent load".to_string(),
            passed: success_rate >= 95.0,
            expected: ">=95% success rate".to_string(),
            actual: format!("{:.1}% success rate", success_rate),
        });

        // Check average response time
        let load_summary = self.metrics.get_summary();
        let avg_response_time_acceptable = load_summary.average_response_time_ms < 200.0;

        assertions.push(AssertionResult {
            description: "Average response time should be acceptable".to_string(),
            passed: avg_response_time_acceptable,
            expected: "<200ms average response time".to_string(),
            actual: format!("{:.1}ms average response time", load_summary.average_response_time_ms),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "API Endpoint Load".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("API endpoint load test failed".to_string()) },
            assertions,
        }
    }

    /// Test workflow execution load
    async fn test_workflow_execution_load(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Login to get auth token
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&serde_json::json!({
                "email": test_user.email,
                "password": "password123",
                "tenant_id": tenant_id
            }))
            .send()
            .await;

        let auth_token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "Workflow Execution Load".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Workflow Execution Load".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Start multiple workflows concurrently
        let concurrent_workflows = 20;
        let mut workflow_handles = Vec::new();

        for i in 0..concurrent_workflows {
            let client = self.env.http_client.clone();
            let token = auth_token.clone();
            let tenant = tenant_id.clone();
            let api_url = self.env.config.api_gateway_url.clone();

            let handle = tokio::spawn(async move {
                let workflow_start = Instant::now();
                
                let response = client
                    .post(&format!("{}/api/v1/workflows/test-load-workflow", api_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-Tenant-ID", &tenant)
                    .json(&serde_json::json!({
                        "test_id": i,
                        "duration_seconds": 2,
                        "steps": 3
                    }))
                    .send()
                    .await;

                let workflow_initiated = response
                    .map(|r| r.status().is_success() || r.status() == 202)
                    .unwrap_or(false);

                let initiation_time = workflow_start.elapsed().as_millis() as u64;
                (i, workflow_initiated, initiation_time)
            });

            workflow_handles.push(handle);
        }

        // Wait for all workflow initiations
        let workflow_results = join_all(workflow_handles).await;
        let successful_initiations = workflow_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success, _)| *success)
            .count();

        let initiation_success_rate = (successful_initiations as f64 / concurrent_workflows as f64) * 100.0;

        assertions.push(AssertionResult {
            description: "Workflows should initiate successfully under load".to_string(),
            passed: initiation_success_rate >= 90.0,
            expected: ">=90% workflow initiation success rate".to_string(),
            actual: format!("{:.1}% initiation success rate", initiation_success_rate),
        });

        // Check workflow initiation time
        let avg_initiation_time: f64 = workflow_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|(_, _, time)| *time as f64)
            .sum::<f64>() / workflow_results.len() as f64;

        let initiation_time_acceptable = avg_initiation_time < 1000.0; // 1 second

        assertions.push(AssertionResult {
            description: "Workflow initiation time should be acceptable".to_string(),
            passed: initiation_time_acceptable,
            expected: "<1000ms average initiation time".to_string(),
            actual: format!("{:.1}ms average initiation time", avg_initiation_time),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Workflow Execution Load".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Workflow execution load test failed".to_string()) },
            assertions,
        }
    }

    /// Test concurrent user simulation
    async fn test_concurrent_user_simulation(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let concurrent_users = std::cmp::min(self.env.config.max_concurrent_users, 25); // Limit for testing
        let mut user_handles = Vec::new();

        for i in 0..concurrent_users {
            let client = self.env.http_client.clone();
            let api_url = self.env.config.api_gateway_url.clone();
            let test_user = self.test_data.users[i as usize % self.test_data.users.len()].clone();
            let tenant_id = test_user.tenant_ids[0].clone();

            let handle = tokio::spawn(async move {
                let user_start = Instant::now();
                let mut user_success = true;
                let mut operations_completed = 0;

                // Simulate user session
                // 1. Login
                let login_response = client
                    .post(&format!("{}/api/v1/auth/login", api_url))
                    .json(&serde_json::json!({
                        "email": test_user.email,
                        "password": "password123",
                        "tenant_id": tenant_id
                    }))
                    .send()
                    .await;

                let auth_token = if let Ok(response) = login_response {
                    if response.status().is_success() {
                        operations_completed += 1;
                        let login_data: serde_json::Value = response.json().await.unwrap();
                        login_data["token"].as_str().unwrap().to_string()
                    } else {
                        user_success = false;
                        return (i, user_success, operations_completed, user_start.elapsed().as_millis() as u64);
                    }
                } else {
                    user_success = false;
                    return (i, user_success, operations_completed, user_start.elapsed().as_millis() as u64);
                };

                // 2. Get profile
                let profile_response = client
                    .get(&format!("{}/api/v1/users/profile", api_url))
                    .header("Authorization", format!("Bearer {}", auth_token))
                    .header("X-Tenant-ID", &tenant_id)
                    .send()
                    .await;

                if profile_response.map(|r| r.status().is_success()).unwrap_or(false) {
                    operations_completed += 1;
                } else {
                    user_success = false;
                }

                // 3. List files
                let files_response = client
                    .get(&format!("{}/api/v1/files", api_url))
                    .header("Authorization", format!("Bearer {}", auth_token))
                    .header("X-Tenant-ID", &tenant_id)
                    .send()
                    .await;

                if files_response.map(|r| r.status().is_success()).unwrap_or(false) {
                    operations_completed += 1;
                } else {
                    user_success = false;
                }

                // 4. Start a workflow
                let workflow_response = client
                    .post(&format!("{}/api/v1/workflows/test-user-simulation", api_url))
                    .header("Authorization", format!("Bearer {}", auth_token))
                    .header("X-Tenant-ID", &tenant_id)
                    .json(&serde_json::json!({
                        "user_id": i,
                        "simulation_data": "test"
                    }))
                    .send()
                    .await;

                if workflow_response.map(|r| r.status().is_success() || r.status() == 202).unwrap_or(false) {
                    operations_completed += 1;
                }

                let session_time = user_start.elapsed().as_millis() as u64;
                (i, user_success, operations_completed, session_time)
            });

            user_handles.push(handle);
        }

        // Wait for all user simulations
        let user_results = join_all(user_handles).await;
        let successful_users = user_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success, _, _)| *success)
            .count();

        let user_success_rate = (successful_users as f64 / concurrent_users as f64) * 100.0;

        assertions.push(AssertionResult {
            description: "Concurrent users should complete sessions successfully".to_string(),
            passed: user_success_rate >= 85.0,
            expected: ">=85% user session success rate".to_string(),
            actual: format!("{:.1}% user session success rate", user_success_rate),
        });

        // Check average operations per user
        let avg_operations: f64 = user_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|(_, _, ops, _)| *ops as f64)
            .sum::<f64>() / user_results.len() as f64;

        let operations_acceptable = avg_operations >= 3.0; // At least 3 operations per user

        assertions.push(AssertionResult {
            description: "Users should complete multiple operations".to_string(),
            passed: operations_acceptable,
            expected: ">=3 operations per user".to_string(),
            actual: format!("{:.1} average operations per user", avg_operations),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Concurrent User Simulation".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Concurrent user simulation test failed".to_string()) },
            assertions,
        }
    }

    /// Test database performance under load
    async fn test_database_performance_load(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Login to get auth token
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&serde_json::json!({
                "email": test_user.email,
                "password": "password123",
                "tenant_id": tenant_id
            }))
            .send()
            .await;

        let auth_token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "Database Performance Load".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Database Performance Load".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Test database-intensive operations
        let db_operations = 30;
        let mut db_handles = Vec::new();

        for i in 0..db_operations {
            let client = self.env.http_client.clone();
            let token = auth_token.clone();
            let tenant = tenant_id.clone();
            let api_url = self.env.config.api_gateway_url.clone();

            let handle = tokio::spawn(async move {
                let op_start = Instant::now();
                
                // Create test data
                let create_response = client
                    .post(&format!("{}/api/v1/test-data", api_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-Tenant-ID", &tenant)
                    .json(&serde_json::json!({
                        "name": format!("Load Test Data {}", i),
                        "value": format!("Test value for load test iteration {}", i),
                        "category": "load_test"
                    }))
                    .send()
                    .await;

                let create_success = create_response
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);

                if !create_success {
                    return (i, false, op_start.elapsed().as_millis() as u64);
                }

                // Query test data
                let query_response = client
                    .get(&format!("{}/api/v1/test-data?category=load_test", api_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-Tenant-ID", &tenant)
                    .send()
                    .await;

                let query_success = query_response
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);

                let operation_time = op_start.elapsed().as_millis() as u64;
                (i, create_success && query_success, operation_time)
            });

            db_handles.push(handle);
        }

        // Wait for all database operations
        let db_results = join_all(db_handles).await;
        let successful_db_ops = db_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success, _)| *success)
            .count();

        let db_success_rate = (successful_db_ops as f64 / db_operations as f64) * 100.0;

        assertions.push(AssertionResult {
            description: "Database operations should succeed under load".to_string(),
            passed: db_success_rate >= 95.0,
            expected: ">=95% database operation success rate".to_string(),
            actual: format!("{:.1}% database operation success rate", db_success_rate),
        });

        // Check average database operation time
        let avg_db_time: f64 = db_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|(_, _, time)| *time as f64)
            .sum::<f64>() / db_results.len() as f64;

        let db_time_acceptable = avg_db_time < 500.0; // 500ms

        assertions.push(AssertionResult {
            description: "Database operation time should be acceptable".to_string(),
            passed: db_time_acceptable,
            expected: "<500ms average database operation time".to_string(),
            actual: format!("{:.1}ms average database operation time", avg_db_time),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Database Performance Load".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Database performance load test failed".to_string()) },
            assertions,
        }
    }

    /// Test file upload load
    async fn test_file_upload_load(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Login to get auth token
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&serde_json::json!({
                "email": test_user.email,
                "password": "password123",
                "tenant_id": tenant_id
            }))
            .send()
            .await;

        let auth_token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "File Upload Load".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "File Upload Load".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Test concurrent file uploads
        let concurrent_uploads = 10;
        let mut upload_handles = Vec::new();

        for i in 0..concurrent_uploads {
            let client = self.env.http_client.clone();
            let token = auth_token.clone();
            let tenant = tenant_id.clone();
            let api_url = self.env.config.api_gateway_url.clone();

            let handle = tokio::spawn(async move {
                let upload_start = Instant::now();
                
                let file_content = format!("Load test file content for upload {}", i);
                let response = client
                    .post(&format!("{}/api/v1/workflows/file-upload", api_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-Tenant-ID", &tenant)
                    .json(&serde_json::json!({
                        "file_name": format!("load-test-{}.txt", i),
                        "file_size": file_content.len(),
                        "content_type": "text/plain",
                        "file_content": base64::encode(&file_content),
                        "storage_provider": "local"
                    }))
                    .send()
                    .await;

                let upload_initiated = response
                    .map(|r| r.status().is_success() || r.status() == 202)
                    .unwrap_or(false);

                let upload_time = upload_start.elapsed().as_millis() as u64;
                (i, upload_initiated, upload_time)
            });

            upload_handles.push(handle);
        }

        // Wait for all uploads
        let upload_results = join_all(upload_handles).await;
        let successful_uploads = upload_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success, _)| *success)
            .count();

        let upload_success_rate = (successful_uploads as f64 / concurrent_uploads as f64) * 100.0;

        assertions.push(AssertionResult {
            description: "File uploads should succeed under load".to_string(),
            passed: upload_success_rate >= 90.0,
            expected: ">=90% file upload success rate".to_string(),
            actual: format!("{:.1}% file upload success rate", upload_success_rate),
        });

        // Check average upload initiation time
        let avg_upload_time: f64 = upload_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|(_, _, time)| *time as f64)
            .sum::<f64>() / upload_results.len() as f64;

        let upload_time_acceptable = avg_upload_time < 2000.0; // 2 seconds

        assertions.push(AssertionResult {
            description: "File upload initiation time should be acceptable".to_string(),
            passed: upload_time_acceptable,
            expected: "<2000ms average upload initiation time".to_string(),
            actual: format!("{:.1}ms average upload initiation time", avg_upload_time),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "File Upload Load".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("File upload load test failed".to_string()) },
            assertions,
        }
    }

    /// Test multi-tenant load isolation
    async fn test_multi_tenant_load_isolation(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let tenant2 = &self.test_data.tenants[1];
        let user1 = &self.test_data.users[0];
        let user2 = &self.test_data.users[1];

        // Login to both tenants
        let login1_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&serde_json::json!({
                "email": user1.email,
                "password": "password123",
                "tenant_id": tenant1.id
            }))
            .send()
            .await;

        let token1 = if let Ok(response) = login1_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "Multi-Tenant Load Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for tenant1".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Multi-Tenant Load Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for tenant1".to_string()),
                assertions,
            };
        };

        let login2_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&serde_json::json!({
                "email": user2.email,
                "password": "password123",
                "tenant_id": tenant2.id
            }))
            .send()
            .await;

        let token2 = if let Ok(response) = login2_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "Multi-Tenant Load Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for tenant2".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Multi-Tenant Load Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for tenant2".to_string()),
                assertions,
            };
        };

        // Generate load on tenant1
        let tenant1_load = 15;
        let mut tenant1_handles = Vec::new();

        for i in 0..tenant1_load {
            let client = self.env.http_client.clone();
            let token = token1.clone();
            let tenant = tenant1.id.clone();
            let api_url = self.env.config.api_gateway_url.clone();

            let handle = tokio::spawn(async move {
                let op_start = Instant::now();
                
                let response = client
                    .post(&format!("{}/api/v1/workflows/test-tenant-load", api_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-Tenant-ID", &tenant)
                    .json(&serde_json::json!({
                        "tenant_load_test": true,
                        "iteration": i,
                        "duration_seconds": 3
                    }))
                    .send()
                    .await;

                let success = response
                    .map(|r| r.status().is_success() || r.status() == 202)
                    .unwrap_or(false);

                let operation_time = op_start.elapsed().as_millis() as u64;
                (i, success, operation_time)
            });

            tenant1_handles.push(handle);
        }

        // Generate lighter load on tenant2 simultaneously
        let tenant2_load = 5;
        let mut tenant2_handles = Vec::new();

        for i in 0..tenant2_load {
            let client = self.env.http_client.clone();
            let token = token2.clone();
            let tenant = tenant2.id.clone();
            let api_url = self.env.config.api_gateway_url.clone();

            let handle = tokio::spawn(async move {
                let op_start = Instant::now();
                
                let response = client
                    .get(&format!("{}/api/v1/users/profile", api_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .header("X-Tenant-ID", &tenant)
                    .send()
                    .await;

                let success = response
                    .map(|r| r.status().is_success())
                    .unwrap_or(false);

                let operation_time = op_start.elapsed().as_millis() as u64;
                (i, success, operation_time)
            });

            tenant2_handles.push(handle);
        }

        // Wait for all operations
        let (tenant1_results, tenant2_results) = tokio::join!(
            join_all(tenant1_handles),
            join_all(tenant2_handles)
        );

        // Analyze tenant1 results
        let tenant1_success_count = tenant1_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success, _)| *success)
            .count();

        let tenant1_success_rate = (tenant1_success_count as f64 / tenant1_load as f64) * 100.0;

        // Analyze tenant2 results
        let tenant2_success_count = tenant2_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success, _)| *success)
            .count();

        let tenant2_success_rate = (tenant2_success_count as f64 / tenant2_load as f64) * 100.0;

        assertions.push(AssertionResult {
            description: "Tenant1 should handle its load successfully".to_string(),
            passed: tenant1_success_rate >= 80.0,
            expected: ">=80% success rate for tenant1".to_string(),
            actual: format!("{:.1}% success rate for tenant1", tenant1_success_rate),
        });

        assertions.push(AssertionResult {
            description: "Tenant2 should not be affected by tenant1 load".to_string(),
            passed: tenant2_success_rate >= 95.0,
            expected: ">=95% success rate for tenant2".to_string(),
            actual: format!("{:.1}% success rate for tenant2", tenant2_success_rate),
        });

        // Check response time isolation
        let tenant1_avg_time: f64 = tenant1_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|(_, _, time)| *time as f64)
            .sum::<f64>() / tenant1_results.len() as f64;

        let tenant2_avg_time: f64 = tenant2_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .map(|(_, _, time)| *time as f64)
            .sum::<f64>() / tenant2_results.len() as f64;

        let response_time_isolation = tenant2_avg_time < tenant1_avg_time * 1.5; // Tenant2 should not be significantly slower

        assertions.push(AssertionResult {
            description: "Response time isolation should be maintained".to_string(),
            passed: response_time_isolation,
            expected: "Tenant2 response time not significantly affected".to_string(),
            actual: format!("Tenant1: {:.1}ms, Tenant2: {:.1}ms", tenant1_avg_time, tenant2_avg_time),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Multi-Tenant Load Isolation".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Multi-tenant load isolation test failed".to_string()) },
            assertions,
        }
    }
}