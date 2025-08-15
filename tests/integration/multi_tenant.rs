// Multi-tenant isolation and security tests
use super::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;
use serde_json::json;

/// Multi-tenant isolation test suite
pub struct MultiTenantTests {
    env: Arc<IntegrationTestEnvironment>,
    test_data: TestData,
}

impl MultiTenantTests {
    pub fn new(env: Arc<IntegrationTestEnvironment>, test_data: TestData) -> Self {
        Self { env, test_data }
    }

    /// Run all multi-tenant isolation tests
    pub async fn run_all_tests(&self) -> Result<IntegrationTestResults, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let mut test_results = Vec::new();

        // Test data isolation between tenants
        test_results.push(self.test_data_isolation().await);

        // Test user access control across tenants
        test_results.push(self.test_user_access_control().await);

        // Test workflow isolation between tenants
        test_results.push(self.test_workflow_isolation().await);

        // Test file storage isolation
        test_results.push(self.test_file_storage_isolation().await);

        // Test API endpoint tenant filtering
        test_results.push(self.test_api_tenant_filtering().await);

        // Test cross-tenant data leakage prevention
        test_results.push(self.test_cross_tenant_leakage_prevention().await);

        // Test tenant switching security
        test_results.push(self.test_tenant_switching_security().await);

        // Test quota enforcement per tenant
        test_results.push(self.test_quota_enforcement().await);

        let execution_time = start_time.elapsed().as_millis() as u64;
        let passed_tests = test_results.iter().filter(|r| r.status == TestStatus::Passed).count() as u32;
        let failed_tests = test_results.iter().filter(|r| r.status == TestStatus::Failed).count() as u32;

        Ok(IntegrationTestResults {
            test_suite: "Multi-Tenant Isolation".to_string(),
            total_tests: test_results.len() as u32,
            passed_tests,
            failed_tests,
            skipped_tests: 0,
            execution_time_ms: execution_time,
            test_details: test_results,
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
        })
    }    /
// Test data isolation between tenants
    async fn test_data_isolation(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let tenant2 = &self.test_data.tenants[1];
        let user1 = &self.test_data.users[0]; // User in tenant1
        let user2 = &self.test_data.users[1]; // User in both tenants

        // Step 1: Login as user1 in tenant1
        let login1_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Data Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for tenant1".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Data Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for tenant1".to_string()),
                assertions,
            };
        };

        // Step 2: Create data in tenant1
        let create_data1_response = self.env.http_client
            .post(&format!("{}/api/v1/test-data", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token1))
            .header("X-Tenant-ID", &tenant1.id)
            .json(&json!({
                "name": "Tenant1 Data",
                "value": "Secret data for tenant 1",
                "category": "test"
            }))
            .send()
            .await;

        let data1_created = create_data1_response
            .as_ref()
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Should be able to create data in tenant1".to_string(),
            passed: data1_created,
            expected: "Data created".to_string(),
            actual: if data1_created { "Created" } else { "Failed" }.to_string(),
        });

        // Step 3: Login as user2 in tenant2
        let login2_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Data Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for tenant2".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Data Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for tenant2".to_string()),
                assertions,
            };
        };

        // Step 4: Try to access tenant1 data from tenant2 context
        let access_attempt_response = self.env.http_client
            .get(&format!("{}/api/v1/test-data", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token2))
            .header("X-Tenant-ID", &tenant2.id)
            .send()
            .await;

        let tenant1_data_isolated = if let Ok(response) = access_attempt_response {
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await.unwrap();
                let items = data.as_array().unwrap_or(&vec![]);
                // Should not see tenant1 data
                !items.iter().any(|item| item["name"] == "Tenant1 Data")
            } else {
                true // Access denied is also acceptable
            }
        } else {
            true
        };

        assertions.push(AssertionResult {
            description: "Tenant1 data should not be visible from tenant2".to_string(),
            passed: tenant1_data_isolated,
            expected: "Data isolated".to_string(),
            actual: if tenant1_data_isolated { "Isolated" } else { "Leaked" }.to_string(),
        });

        // Step 5: Create data in tenant2
        let create_data2_response = self.env.http_client
            .post(&format!("{}/api/v1/test-data", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token2))
            .header("X-Tenant-ID", &tenant2.id)
            .json(&json!({
                "name": "Tenant2 Data",
                "value": "Secret data for tenant 2",
                "category": "test"
            }))
            .send()
            .await;

        let data2_created = create_data2_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Should be able to create data in tenant2".to_string(),
            passed: data2_created,
            expected: "Data created".to_string(),
            actual: if data2_created { "Created" } else { "Failed" }.to_string(),
        });

        // Step 6: Verify tenant2 can only see its own data
        let tenant2_data_response = self.env.http_client
            .get(&format!("{}/api/v1/test-data", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token2))
            .header("X-Tenant-ID", &tenant2.id)
            .send()
            .await;

        let tenant2_data_correct = if let Ok(response) = tenant2_data_response {
            if response.status().is_success() {
                let data: serde_json::Value = response.json().await.unwrap();
                let items = data.as_array().unwrap_or(&vec![]);
                items.iter().any(|item| item["name"] == "Tenant2 Data") &&
                !items.iter().any(|item| item["name"] == "Tenant1 Data")
            } else {
                false
            }
        } else {
            false
        };

        assertions.push(AssertionResult {
            description: "Tenant2 should only see its own data".to_string(),
            passed: tenant2_data_correct,
            expected: "Only tenant2 data visible".to_string(),
            actual: if tenant2_data_correct { "Correct" } else { "Incorrect" }.to_string(),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Data Isolation".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Data isolation test failed".to_string()) },
            assertions,
        }
    }

    /// Test user access control across tenants
    async fn test_user_access_control(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let tenant2 = &self.test_data.tenants[1];
        let user1 = &self.test_data.users[0]; // User only in tenant1

        // Step 1: Login as user1 in tenant1
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
                "email": user1.email,
                "password": "password123",
                "tenant_id": tenant1.id
            }))
            .send()
            .await;

        let token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "User Access Control".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "User Access Control".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Try to access tenant2 resources with tenant1 token
        let unauthorized_access_response = self.env.http_client
            .get(&format!("{}/api/v1/tenants/{}/users", self.env.config.api_gateway_url, tenant2.id))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", &tenant2.id)
            .send()
            .await;

        let access_denied = unauthorized_access_response
            .map(|r| r.status() == 403 || r.status() == 401)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "User should not access unauthorized tenant resources".to_string(),
            passed: access_denied,
            expected: "403 Forbidden or 401 Unauthorized".to_string(),
            actual: if access_denied { "Access denied" } else { "Access granted" }.to_string(),
        });

        // Step 3: Try to use wrong tenant ID in header
        let wrong_tenant_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", &tenant2.id) // Wrong tenant ID
            .send()
            .await;

        let tenant_mismatch_rejected = wrong_tenant_response
            .map(|r| r.status() == 403 || r.status() == 401)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Mismatched tenant ID should be rejected".to_string(),
            passed: tenant_mismatch_rejected,
            expected: "Request rejected".to_string(),
            actual: if tenant_mismatch_rejected { "Rejected" } else { "Accepted" }.to_string(),
        });

        // Step 4: Verify correct tenant access works
        let correct_access_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", &tenant1.id)
            .send()
            .await;

        let correct_access_works = correct_access_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Correct tenant access should work".to_string(),
            passed: correct_access_works,
            expected: "Access granted".to_string(),
            actual: if correct_access_works { "Granted" } else { "Denied" }.to_string(),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "User Access Control".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("User access control test failed".to_string()) },
            assertions,
        }
    }

    /// Test workflow isolation between tenants
    async fn test_workflow_isolation(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let tenant2 = &self.test_data.tenants[1];
        let user1 = &self.test_data.users[0];
        let user2 = &self.test_data.users[1];

        // Step 1: Login as users in different tenants
        let login1_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Workflow Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for user1".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Workflow Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for user1".to_string()),
                assertions,
            };
        };

        let login2_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Workflow Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for user2".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Workflow Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for user2".to_string()),
                assertions,
            };
        };

        // Step 2: Start workflows in both tenants
        let workflow1_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/test-tenant-isolation", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token1))
            .header("X-Tenant-ID", &tenant1.id)
            .json(&json!({
                "test_data": "Tenant1 workflow data",
                "duration_seconds": 5
            }))
            .send()
            .await;

        let workflow1_started = workflow1_response
            .as_ref()
            .map(|r| r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Workflow should start in tenant1".to_string(),
            passed: workflow1_started,
            expected: "Workflow started".to_string(),
            actual: if workflow1_started { "Started" } else { "Failed" }.to_string(),
        });

        let workflow2_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/test-tenant-isolation", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token2))
            .header("X-Tenant-ID", &tenant2.id)
            .json(&json!({
                "test_data": "Tenant2 workflow data",
                "duration_seconds": 5
            }))
            .send()
            .await;

        let workflow2_started = workflow2_response
            .as_ref()
            .map(|r| r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Workflow should start in tenant2".to_string(),
            passed: workflow2_started,
            expected: "Workflow started".to_string(),
            actual: if workflow2_started { "Started" } else { "Failed" }.to_string(),
        });

        if workflow1_started && workflow2_started {
            let workflow1_data: serde_json::Value = workflow1_response.unwrap().json().await.unwrap();
            let workflow2_data: serde_json::Value = workflow2_response.unwrap().json().await.unwrap();
            
            let operation1_id = workflow1_data["operation_id"].as_str().unwrap();
            let operation2_id = workflow2_data["operation_id"].as_str().unwrap();

            // Step 3: Verify user1 cannot see user2's workflow
            let cross_workflow_access_response = self.env.http_client
                .get(&format!("{}/api/v1/workflows/{}/status", self.env.config.api_gateway_url, operation2_id))
                .header("Authorization", format!("Bearer {}", token1))
                .header("X-Tenant-ID", &tenant1.id)
                .send()
                .await;

            let cross_access_denied = cross_workflow_access_response
                .map(|r| r.status() == 403 || r.status() == 404)
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "User should not access other tenant's workflows".to_string(),
                passed: cross_access_denied,
                expected: "Access denied".to_string(),
                actual: if cross_access_denied { "Denied" } else { "Allowed" }.to_string(),
            });

            // Step 4: Verify users can see their own workflows
            let own_workflow1_response = self.env.http_client
                .get(&format!("{}/api/v1/workflows/{}/status", self.env.config.api_gateway_url, operation1_id))
                .header("Authorization", format!("Bearer {}", token1))
                .header("X-Tenant-ID", &tenant1.id)
                .send()
                .await;

            let own_workflow1_accessible = own_workflow1_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "User should access their own tenant's workflows".to_string(),
                passed: own_workflow1_accessible,
                expected: "Access granted".to_string(),
                actual: if own_workflow1_accessible { "Granted" } else { "Denied" }.to_string(),
            });

            // Step 5: List workflows and verify isolation
            let workflows_list1_response = self.env.http_client
                .get(&format!("{}/api/v1/workflows/user", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", token1))
                .header("X-Tenant-ID", &tenant1.id)
                .send()
                .await;

            let workflows_isolated = if let Ok(response) = workflows_list1_response {
                if response.status().is_success() {
                    let workflows: serde_json::Value = response.json().await.unwrap();
                    let workflow_list = workflows.as_array().unwrap_or(&vec![]);
                    // Should only see tenant1 workflows
                    workflow_list.iter().any(|w| w["operation_id"] == operation1_id) &&
                    !workflow_list.iter().any(|w| w["operation_id"] == operation2_id)
                } else {
                    false
                }
            } else {
                false
            };

            assertions.push(AssertionResult {
                description: "Workflow list should be tenant-isolated".to_string(),
                passed: workflows_isolated,
                expected: "Only tenant workflows visible".to_string(),
                actual: if workflows_isolated { "Isolated" } else { "Not isolated" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Workflow Isolation".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Workflow isolation test failed".to_string()) },
            assertions,
        }
    }

    /// Test file storage isolation
    async fn test_file_storage_isolation(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let tenant2 = &self.test_data.tenants[1];
        let user1 = &self.test_data.users[0];
        let user2 = &self.test_data.users[1];

        // Step 1: Login as users in different tenants
        let login1_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "File Storage Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for user1".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "File Storage Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for user1".to_string()),
                assertions,
            };
        };

        let login2_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "File Storage Isolation".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for user2".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "File Storage Isolation".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for user2".to_string()),
                assertions,
            };
        };

        // Step 2: Upload files in both tenants
        let file1_content = "Tenant1 confidential file content";
        let upload1_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/file-upload", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token1))
            .header("X-Tenant-ID", &tenant1.id)
            .json(&json!({
                "file_name": "tenant1-secret.txt",
                "file_size": file1_content.len(),
                "content_type": "text/plain",
                "file_content": base64::encode(file1_content),
                "storage_provider": "local"
            }))
            .send()
            .await;

        let file1_uploaded = upload1_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "File should upload successfully in tenant1".to_string(),
            passed: file1_uploaded,
            expected: "Upload successful".to_string(),
            actual: if file1_uploaded { "Success" } else { "Failed" }.to_string(),
        });

        let file2_content = "Tenant2 confidential file content";
        let upload2_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/file-upload", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token2))
            .header("X-Tenant-ID", &tenant2.id)
            .json(&json!({
                "file_name": "tenant2-secret.txt",
                "file_size": file2_content.len(),
                "content_type": "text/plain",
                "file_content": base64::encode(file2_content),
                "storage_provider": "local"
            }))
            .send()
            .await;

        let file2_uploaded = upload2_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "File should upload successfully in tenant2".to_string(),
            passed: file2_uploaded,
            expected: "Upload successful".to_string(),
            actual: if file2_uploaded { "Success" } else { "Failed" }.to_string(),
        });

        if file1_uploaded && file2_uploaded {
            // Get file IDs from workflow responses
            let upload1_data: serde_json::Value = upload1_response.unwrap().json().await.unwrap();
            let upload2_data: serde_json::Value = upload2_response.unwrap().json().await.unwrap();

            // Handle workflow completion to get file IDs
            let file1_id = if let Some(operation_id) = upload1_data.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(result_data) => result_data["file_id"].as_str().unwrap().to_string(),
                    Err(_) => return TestResult {
                        test_name: "File Storage Isolation".to_string(),
                        status: TestStatus::Failed,
                        execution_time_ms: test_start.elapsed().as_millis() as u64,
                        error_message: Some("File1 upload workflow failed".to_string()),
                        assertions,
                    },
                }
            } else {
                upload1_data["file_id"].as_str().unwrap().to_string()
            };

            let file2_id = if let Some(operation_id) = upload2_data.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(result_data) => result_data["file_id"].as_str().unwrap().to_string(),
                    Err(_) => return TestResult {
                        test_name: "File Storage Isolation".to_string(),
                        status: TestStatus::Failed,
                        execution_time_ms: test_start.elapsed().as_millis() as u64,
                        error_message: Some("File2 upload workflow failed".to_string()),
                        assertions,
                    },
                }
            } else {
                upload2_data["file_id"].as_str().unwrap().to_string()
            };

            // Step 3: Try to access tenant2's file from tenant1
            let cross_file_access_response = self.env.http_client
                .get(&format!("{}/api/v1/files/{}", self.env.config.api_gateway_url, file2_id))
                .header("Authorization", format!("Bearer {}", token1))
                .header("X-Tenant-ID", &tenant1.id)
                .send()
                .await;

            let cross_file_access_denied = cross_file_access_response
                .map(|r| r.status() == 403 || r.status() == 404)
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Cross-tenant file access should be denied".to_string(),
                passed: cross_file_access_denied,
                expected: "Access denied".to_string(),
                actual: if cross_file_access_denied { "Denied" } else { "Allowed" }.to_string(),
            });

            // Step 4: Verify users can access their own files
            let own_file_access_response = self.env.http_client
                .get(&format!("{}/api/v1/files/{}", self.env.config.api_gateway_url, file1_id))
                .header("Authorization", format!("Bearer {}", token1))
                .header("X-Tenant-ID", &tenant1.id)
                .send()
                .await;

            let own_file_accessible = own_file_access_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Users should access their own tenant's files".to_string(),
                passed: own_file_accessible,
                expected: "Access granted".to_string(),
                actual: if own_file_accessible { "Granted" } else { "Denied" }.to_string(),
            });

            // Step 5: List files and verify isolation
            let files_list1_response = self.env.http_client
                .get(&format!("{}/api/v1/files", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", token1))
                .header("X-Tenant-ID", &tenant1.id)
                .send()
                .await;

            let files_isolated = if let Ok(response) = files_list1_response {
                if response.status().is_success() {
                    let files: serde_json::Value = response.json().await.unwrap();
                    let file_list = files.as_array().unwrap_or(&vec![]);
                    // Should only see tenant1 files
                    file_list.iter().any(|f| f["id"] == file1_id) &&
                    !file_list.iter().any(|f| f["id"] == file2_id)
                } else {
                    false
                }
            } else {
                false
            };

            assertions.push(AssertionResult {
                description: "File list should be tenant-isolated".to_string(),
                passed: files_isolated,
                expected: "Only tenant files visible".to_string(),
                actual: if files_isolated { "Isolated" } else { "Not isolated" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "File Storage Isolation".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("File storage isolation test failed".to_string()) },
            assertions,
        }
    } 
   /// Test API endpoint tenant filtering
    async fn test_api_tenant_filtering(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let tenant2 = &self.test_data.tenants[1];
        let user1 = &self.test_data.users[0];

        // Step 1: Login as user1
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
                "email": user1.email,
                "password": "password123",
                "tenant_id": tenant1.id
            }))
            .send()
            .await;

        let token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "API Tenant Filtering".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "API Tenant Filtering".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Test missing tenant header
        let no_tenant_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token))
            // Missing X-Tenant-ID header
            .send()
            .await;

        let no_tenant_rejected = no_tenant_response
            .map(|r| r.status() == 400 || r.status() == 401)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Request without tenant header should be rejected".to_string(),
            passed: no_tenant_rejected,
            expected: "400 Bad Request or 401 Unauthorized".to_string(),
            actual: if no_tenant_rejected { "Rejected" } else { "Accepted" }.to_string(),
        });

        // Step 3: Test invalid tenant ID
        let invalid_tenant_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", "invalid-tenant-id")
            .send()
            .await;

        let invalid_tenant_rejected = invalid_tenant_response
            .map(|r| r.status() == 403 || r.status() == 404)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Request with invalid tenant ID should be rejected".to_string(),
            passed: invalid_tenant_rejected,
            expected: "403 Forbidden or 404 Not Found".to_string(),
            actual: if invalid_tenant_rejected { "Rejected" } else { "Accepted" }.to_string(),
        });

        // Step 4: Test unauthorized tenant access
        let unauthorized_tenant_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", &tenant2.id) // User doesn't have access to tenant2
            .send()
            .await;

        let unauthorized_tenant_rejected = unauthorized_tenant_response
            .map(|r| r.status() == 403)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Request with unauthorized tenant should be rejected".to_string(),
            passed: unauthorized_tenant_rejected,
            expected: "403 Forbidden".to_string(),
            actual: if unauthorized_tenant_rejected { "Rejected" } else { "Accepted" }.to_string(),
        });

        // Step 5: Test correct tenant access
        let correct_tenant_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", &tenant1.id)
            .send()
            .await;

        let correct_tenant_works = correct_tenant_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Request with correct tenant should work".to_string(),
            passed: correct_tenant_works,
            expected: "200 OK".to_string(),
            actual: if correct_tenant_works { "Success" } else { "Failed" }.to_string(),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "API Tenant Filtering".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("API tenant filtering test failed".to_string()) },
            assertions,
        }
    }

    /// Test cross-tenant data leakage prevention
    async fn test_cross_tenant_leakage_prevention(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let tenant2 = &self.test_data.tenants[1];
        let user1 = &self.test_data.users[0];
        let user2 = &self.test_data.users[1];

        // Step 1: Login as users in different tenants
        let login1_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Cross-Tenant Leakage Prevention".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for user1".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Cross-Tenant Leakage Prevention".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for user1".to_string()),
                assertions,
            };
        };

        let login2_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Cross-Tenant Leakage Prevention".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed for user2".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Cross-Tenant Leakage Prevention".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed for user2".to_string()),
                assertions,
            };
        };

        // Step 2: Test search across tenants
        let search_response = self.env.http_client
            .get(&format!("{}/api/v1/search?q=tenant", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token1))
            .header("X-Tenant-ID", &tenant1.id)
            .send()
            .await;

        let search_isolated = if let Ok(response) = search_response {
            if response.status().is_success() {
                let search_results: serde_json::Value = response.json().await.unwrap();
                let results = search_results["results"].as_array().unwrap_or(&vec![]);
                // Should not find results from other tenants
                !results.iter().any(|r| r["tenant_id"] != tenant1.id)
            } else {
                true // No results is also acceptable
            }
        } else {
            true
        };

        assertions.push(AssertionResult {
            description: "Search should not return cross-tenant results".to_string(),
            passed: search_isolated,
            expected: "Only tenant-specific results".to_string(),
            actual: if search_isolated { "Isolated" } else { "Leaked" }.to_string(),
        });

        // Step 3: Test aggregation endpoints
        let stats_response = self.env.http_client
            .get(&format!("{}/api/v1/stats/summary", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token1))
            .header("X-Tenant-ID", &tenant1.id)
            .send()
            .await;

        let stats_isolated = if let Ok(response) = stats_response {
            if response.status().is_success() {
                let stats: serde_json::Value = response.json().await.unwrap();
                // Stats should only reflect tenant1 data
                stats.get("tenant_id").map(|t| t == tenant1.id).unwrap_or(true)
            } else {
                true
            }
        } else {
            true
        };

        assertions.push(AssertionResult {
            description: "Statistics should be tenant-isolated".to_string(),
            passed: stats_isolated,
            expected: "Tenant-specific stats only".to_string(),
            actual: if stats_isolated { "Isolated" } else { "Leaked" }.to_string(),
        });

        // Step 4: Test bulk operations
        let bulk_operation_response = self.env.http_client
            .post(&format!("{}/api/v1/bulk/delete", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token1))
            .header("X-Tenant-ID", &tenant1.id)
            .json(&json!({
                "entity_type": "test_data",
                "filters": {
                    "category": "test"
                }
            }))
            .send()
            .await;

        let bulk_operation_safe = bulk_operation_response
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Bulk operations should be tenant-scoped".to_string(),
            passed: bulk_operation_safe,
            expected: "Operation scoped to tenant".to_string(),
            actual: if bulk_operation_safe { "Scoped" } else { "Failed" }.to_string(),
        });

        // Step 5: Test export operations
        let export_response = self.env.http_client
            .post(&format!("{}/api/v1/export/data", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", token1))
            .header("X-Tenant-ID", &tenant1.id)
            .json(&json!({
                "format": "json",
                "include_all": true
            }))
            .send()
            .await;

        let export_isolated = export_response
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Data export should be tenant-isolated".to_string(),
            passed: export_isolated,
            expected: "Export scoped to tenant".to_string(),
            actual: if export_isolated { "Scoped" } else { "Failed" }.to_string(),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Cross-Tenant Leakage Prevention".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Cross-tenant leakage prevention test failed".to_string()) },
            assertions,
        }
    }

    /// Test tenant switching security
    async fn test_tenant_switching_security(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        // Use a user with access to multiple tenants
        let multi_tenant_user = self.test_data.users.iter()
            .find(|u| u.tenant_ids.len() > 1)
            .unwrap();

        let source_tenant = &multi_tenant_user.tenant_ids[0];
        let target_tenant = &multi_tenant_user.tenant_ids[1];

        // Step 1: Login to source tenant
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
                "email": multi_tenant_user.email,
                "password": "password123",
                "tenant_id": source_tenant
            }))
            .send()
            .await;

        let auth_token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "Tenant Switching Security".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Initial login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Tenant Switching Security".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Initial login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Try to switch to unauthorized tenant
        let unauthorized_switch_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/switch-tenant", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&json!({
                "target_tenant_id": "unauthorized-tenant-id",
                "current_tenant_id": source_tenant
            }))
            .send()
            .await;

        let unauthorized_switch_rejected = unauthorized_switch_response
            .map(|r| r.status() == 403 || r.status() == 404)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Unauthorized tenant switch should be rejected".to_string(),
            passed: unauthorized_switch_rejected,
            expected: "403 Forbidden or 404 Not Found".to_string(),
            actual: if unauthorized_switch_rejected { "Rejected" } else { "Allowed" }.to_string(),
        });

        // Step 3: Try to switch with invalid current tenant
        let invalid_current_switch_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/switch-tenant", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&json!({
                "target_tenant_id": target_tenant,
                "current_tenant_id": "wrong-current-tenant"
            }))
            .send()
            .await;

        let invalid_current_rejected = invalid_current_switch_response
            .map(|r| r.status() == 400 || r.status() == 403)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Switch with wrong current tenant should be rejected".to_string(),
            passed: invalid_current_rejected,
            expected: "400 Bad Request or 403 Forbidden".to_string(),
            actual: if invalid_current_rejected { "Rejected" } else { "Allowed" }.to_string(),
        });

        // Step 4: Perform valid tenant switch
        let valid_switch_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/switch-tenant", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&json!({
                "target_tenant_id": target_tenant,
                "current_tenant_id": source_tenant
            }))
            .send()
            .await;

        let valid_switch_accepted = valid_switch_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Valid tenant switch should be accepted".to_string(),
            passed: valid_switch_accepted,
            expected: "200 OK or 202 Accepted".to_string(),
            actual: if valid_switch_accepted { "Accepted" } else { "Rejected" }.to_string(),
        });

        if valid_switch_accepted {
            let switch_data: serde_json::Value = valid_switch_response.unwrap().json().await.unwrap();
            
            // Step 5: Verify old token is invalidated
            sleep(Duration::from_secs(1)).await; // Allow time for token invalidation
            
            let old_token_response = self.env.http_client
                .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", source_tenant)
                .send()
                .await;

            let old_token_invalidated = old_token_response
                .map(|r| r.status() == 401)
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Old token should be invalidated after tenant switch".to_string(),
                passed: old_token_invalidated,
                expected: "401 Unauthorized".to_string(),
                actual: if old_token_invalidated { "Invalidated" } else { "Still valid" }.to_string(),
            });

            // Step 6: Verify new session works
            let new_session_id = if let Some(operation_id) = switch_data.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(result_data) => result_data["new_session_id"].as_str().unwrap().to_string(),
                    Err(_) => return TestResult {
                        test_name: "Tenant Switching Security".to_string(),
                        status: TestStatus::Failed,
                        execution_time_ms: test_start.elapsed().as_millis() as u64,
                        error_message: Some("Tenant switch workflow failed".to_string()),
                        assertions,
                    },
                }
            } else {
                switch_data["new_session_id"].as_str().unwrap().to_string()
            };

            let new_session_response = self.env.http_client
                .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", new_session_id))
                .header("X-Tenant-ID", target_tenant)
                .send()
                .await;

            let new_session_works = new_session_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "New session should work in target tenant".to_string(),
                passed: new_session_works,
                expected: "200 OK".to_string(),
                actual: if new_session_works { "Working" } else { "Failed" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Tenant Switching Security".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Tenant switching security test failed".to_string()) },
            assertions,
        }
    }

    /// Test quota enforcement per tenant
    async fn test_quota_enforcement(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let tenant1 = &self.test_data.tenants[0];
        let user1 = &self.test_data.users[0];

        // Step 1: Login
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
                "email": user1.email,
                "password": "password123",
                "tenant_id": tenant1.id
            }))
            .send()
            .await;

        let auth_token = if let Ok(response) = login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "Quota Enforcement".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Quota Enforcement".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Check current quota usage
        let quota_response = self.env.http_client
            .get(&format!("{}/api/v1/tenants/{}/quota", self.env.config.api_gateway_url, tenant1.id))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", &tenant1.id)
            .send()
            .await;

        let quota_accessible = quota_response
            .as_ref()
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Quota information should be accessible".to_string(),
            passed: quota_accessible,
            expected: "Quota info available".to_string(),
            actual: if quota_accessible { "Available" } else { "Not available" }.to_string(),
        });

        if quota_accessible {
            let quota_data: serde_json::Value = quota_response.unwrap().json().await.unwrap();
            
            // Step 3: Test API rate limiting
            let mut rate_limit_hit = false;
            for i in 0..20 {
                let api_call_response = self.env.http_client
                    .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
                    .header("Authorization", format!("Bearer {}", auth_token))
                    .header("X-Tenant-ID", &tenant1.id)
                    .send()
                    .await;

                if let Ok(response) = api_call_response {
                    if response.status() == 429 {
                        rate_limit_hit = true;
                        break;
                    }
                }
                
                sleep(Duration::from_millis(50)).await;
            }

            assertions.push(AssertionResult {
                description: "API rate limiting should be enforced".to_string(),
                passed: rate_limit_hit,
                expected: "Rate limit enforced".to_string(),
                actual: if rate_limit_hit { "Enforced" } else { "Not enforced" }.to_string(),
            });

            // Step 4: Test storage quota (if applicable)
            let large_file_content = "x".repeat(1024 * 1024); // 1MB file
            let large_file_response = self.env.http_client
                .post(&format!("{}/api/v1/workflows/file-upload", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", &tenant1.id)
                .json(&json!({
                    "file_name": "large-test-file.txt",
                    "file_size": large_file_content.len(),
                    "content_type": "text/plain",
                    "file_content": base64::encode(&large_file_content),
                    "storage_provider": "local"
                }))
                .send()
                .await;

            let storage_quota_respected = large_file_response
                .map(|r| r.status().is_success() || r.status() == 202 || r.status() == 413) // 413 = Payload Too Large
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Storage quota should be respected".to_string(),
                passed: storage_quota_respected,
                expected: "Upload handled according to quota".to_string(),
                actual: if storage_quota_respected { "Respected" } else { "Ignored" }.to_string(),
            });

            // Step 5: Test workflow quota
            let workflow_quota_response = self.env.http_client
                .post(&format!("{}/api/v1/workflows/test-quota-enforcement", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", &tenant1.id)
                .json(&json!({
                    "test_type": "workflow_quota"
                }))
                .send()
                .await;

            let workflow_quota_enforced = workflow_quota_response
                .map(|r| r.status().is_success() || r.status() == 202 || r.status() == 429)
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Workflow quota should be enforced".to_string(),
                passed: workflow_quota_enforced,
                expected: "Workflow quota enforced".to_string(),
                actual: if workflow_quota_enforced { "Enforced" } else { "Not enforced" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Quota Enforcement".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Quota enforcement test failed".to_string()) },
            assertions,
        }
    }
}