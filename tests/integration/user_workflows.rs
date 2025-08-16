// Complete user workflow integration tests
use super::*;
use crate::test_environment::{IntegrationTestEnvironment, TestData};
use std::time::{Duration, Instant};
use tokio::time::sleep;
use serde_json::json;

/// Complete user workflow test suite
pub struct UserWorkflowTests {
    env: Arc<IntegrationTestEnvironment>,
    test_data: TestData,
}

impl UserWorkflowTests {
    pub fn new(env: Arc<IntegrationTestEnvironment>, test_data: TestData) -> Self {
        Self { env, test_data }
    }

    /// Run all user workflow tests
    pub async fn run_all_tests(&self) -> Result<IntegrationTestResults, Box<dyn std::error::Error>> {
        let start_time = Instant::now();
        let mut test_results = Vec::new();

        // Test complete user registration workflow
        test_results.push(self.test_user_registration_workflow().await);

        // Test user login and authentication workflow
        test_results.push(self.test_user_login_workflow().await);

        // Test tenant switching workflow
        test_results.push(self.test_tenant_switching_workflow().await);

        // Test file upload and management workflow
        test_results.push(self.test_file_management_workflow().await);

        // Test user profile management workflow
        test_results.push(self.test_user_profile_workflow().await);

        // Test module installation and usage workflow
        test_results.push(self.test_module_workflow().await);

        // Test workflow monitoring and management
        test_results.push(self.test_workflow_monitoring().await);

        // Test user deactivation workflow
        test_results.push(self.test_user_deactivation_workflow().await);

        let execution_time = start_time.elapsed().as_millis() as u64;
        let passed_tests = test_results.iter().filter(|r| r.status == TestStatus::Passed).count() as u32;
        let failed_tests = test_results.iter().filter(|r| r.status == TestStatus::Failed).count() as u32;

        Ok(IntegrationTestResults {
            test_suite: "User Workflows".to_string(),
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
    }

    /// Test complete user registration workflow
    async fn test_user_registration_workflow(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let new_user_email = format!("newuser{}@test.com", chrono::Utc::now().timestamp());
        let tenant_id = &self.test_data.tenants[0].id;

        // Step 1: Initiate user registration workflow
        let registration_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/user-registration", self.env.config.api_gateway_url))
            .json(&json!({
                "email": new_user_email,
                "password": "newpassword123",
                "tenant_id": tenant_id,
                "first_name": "New",
                "last_name": "User",
                "roles": ["user"]
            }))
            .send()
            .await;

        let registration_initiated = registration_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "User registration workflow should be initiated".to_string(),
            passed: registration_initiated,
            expected: "200 OK or 202 Accepted".to_string(),
            actual: registration_response.as_ref().map(|r| r.status().to_string()).unwrap_or("Error".to_string()),
        });

        if registration_initiated {
            let workflow_response: serde_json::Value = registration_response.unwrap().json().await.unwrap();
            
            // Step 2: Poll for workflow completion
            let user_id = if let Some(operation_id) = workflow_response.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(result_data) => {
                        assertions.push(AssertionResult {
                            description: "User registration workflow should complete successfully".to_string(),
                            passed: true,
                            expected: "Workflow completed".to_string(),
                            actual: "Completed".to_string(),
                        });
                        result_data["user_id"].as_str().unwrap().to_string()
                    }
                    Err(e) => {
                        assertions.push(AssertionResult {
                            description: "User registration workflow should complete successfully".to_string(),
                            passed: false,
                            expected: "Workflow completed".to_string(),
                            actual: format!("Failed: {}", e),
                        });
                        return TestResult {
                            test_name: "User Registration Workflow".to_string(),
                            status: TestStatus::Failed,
                            execution_time_ms: test_start.elapsed().as_millis() as u64,
                            error_message: Some(format!("Registration workflow failed: {}", e)),
                            assertions,
                        };
                    }
                }
            } else {
                workflow_response["user_id"].as_str().unwrap().to_string()
            };

            // Step 3: Verify user was created in database
            let user_check_response = self.env.http_client
                .get(&format!("{}/api/v1/users/{}", self.env.config.api_gateway_url, user_id))
                .header("X-Tenant-ID", tenant_id)
                .send()
                .await;

            let user_exists = user_check_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "User should exist in database after registration".to_string(),
                passed: user_exists,
                expected: "User found".to_string(),
                actual: if user_exists { "Found" } else { "Not found" }.to_string(),
            });

            // Step 4: Verify user can login
            let login_response = self.env.http_client
                .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
                .json(&json!({
                    "email": new_user_email,
                    "password": "newpassword123",
                    "tenant_id": tenant_id
                }))
                .send()
                .await;

            let login_successful = login_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "User should be able to login after registration".to_string(),
                passed: login_successful,
                expected: "Login successful".to_string(),
                actual: if login_successful { "Success" } else { "Failed" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "User Registration Workflow".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("User registration workflow test failed".to_string()) },
            assertions,
        }
    }

    /// Test user login and authentication workflow
    async fn test_user_login_workflow(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Step 1: Login with valid credentials
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
                "email": test_user.email,
                "password": "password123",
                "tenant_id": tenant_id
            }))
            .send()
            .await;

        let login_successful = login_response
            .as_ref()
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "User should be able to login with valid credentials".to_string(),
            passed: login_successful,
            expected: "200 OK".to_string(),
            actual: login_response.as_ref().map(|r| r.status().to_string()).unwrap_or("Error".to_string()),
        });

        let auth_token = if login_successful {
            let login_data: serde_json::Value = login_response.unwrap().json().await.unwrap();
            
            // Verify token structure
            let token_valid = login_data.get("token").is_some() && 
                             login_data.get("user").is_some() &&
                             login_data.get("tenant").is_some();

            assertions.push(AssertionResult {
                description: "Login response should contain token, user, and tenant data".to_string(),
                passed: token_valid,
                expected: "Token, user, and tenant data present".to_string(),
                actual: if token_valid { "Present" } else { "Missing" }.to_string(),
            });

            login_data["token"].as_str().unwrap().to_string()
        } else {
            return TestResult {
                test_name: "User Login Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login failed".to_string()),
                assertions,
            };
        };

        // Step 2: Use token to access protected endpoint
        let protected_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await;

        let protected_access = protected_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Token should provide access to protected endpoints".to_string(),
            passed: protected_access,
            expected: "Access granted".to_string(),
            actual: if protected_access { "Granted" } else { "Denied" }.to_string(),
        });

        // Step 3: Test token validation
        let token_validation_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/validate", self.env.config.api_gateway_url))
            .json(&json!({
                "token": auth_token
            }))
            .send()
            .await;

        let token_valid = token_validation_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Token should be valid when validated".to_string(),
            passed: token_valid,
            expected: "Token valid".to_string(),
            actual: if token_valid { "Valid" } else { "Invalid" }.to_string(),
        });

        // Step 4: Test invalid login
        let invalid_login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
                "email": test_user.email,
                "password": "wrongpassword",
                "tenant_id": tenant_id
            }))
            .send()
            .await;

        let invalid_login_rejected = invalid_login_response
            .map(|r| r.status().is_client_error())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Invalid credentials should be rejected".to_string(),
            passed: invalid_login_rejected,
            expected: "401 Unauthorized".to_string(),
            actual: if invalid_login_rejected { "Rejected" } else { "Accepted" }.to_string(),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "User Login Workflow".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("User login workflow test failed".to_string()) },
            assertions,
        }
    }

    /// Test tenant switching workflow
    async fn test_tenant_switching_workflow(&self) -> TestResult {
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
                    test_name: "Tenant Switching Workflow".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Initial login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Tenant Switching Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Initial login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Initiate tenant switch workflow
        let switch_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/switch-tenant", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .json(&json!({
                "target_tenant_id": target_tenant,
                "current_tenant_id": source_tenant
            }))
            .send()
            .await;

        let switch_initiated = switch_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Tenant switch workflow should be initiated".to_string(),
            passed: switch_initiated,
            expected: "200 OK or 202 Accepted".to_string(),
            actual: switch_response.as_ref().map(|r| r.status().to_string()).unwrap_or("Error".to_string()),
        });

        if switch_initiated {
            let switch_data: serde_json::Value = switch_response.unwrap().json().await.unwrap();
            
            // Step 3: Handle workflow completion
            let new_session_id = if let Some(operation_id) = switch_data.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(result_data) => {
                        assertions.push(AssertionResult {
                            description: "Tenant switch workflow should complete successfully".to_string(),
                            passed: true,
                            expected: "Workflow completed".to_string(),
                            actual: "Completed".to_string(),
                        });
                        result_data["new_session_id"].as_str().unwrap().to_string()
                    }
                    Err(e) => {
                        assertions.push(AssertionResult {
                            description: "Tenant switch workflow should complete successfully".to_string(),
                            passed: false,
                            expected: "Workflow completed".to_string(),
                            actual: format!("Failed: {}", e),
                        });
                        return TestResult {
                            test_name: "Tenant Switching Workflow".to_string(),
                            status: TestStatus::Failed,
                            execution_time_ms: test_start.elapsed().as_millis() as u64,
                            error_message: Some(format!("Tenant switch workflow failed: {}", e)),
                            assertions,
                        };
                    }
                }
            } else {
                switch_data["new_session_id"].as_str().unwrap().to_string()
            };

            // Step 4: Verify new tenant context
            let context_response = self.env.http_client
                .get(&format!("{}/api/v1/users/context", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", new_session_id))
                .header("X-Tenant-ID", target_tenant)
                .send()
                .await;

            let context_updated = if let Ok(response) = context_response {
                if response.status().is_success() {
                    let context_data: serde_json::Value = response.json().await.unwrap();
                    context_data["active_tenant_id"].as_str() == Some(target_tenant)
                } else {
                    false
                }
            } else {
                false
            };

            assertions.push(AssertionResult {
                description: "User context should reflect new tenant".to_string(),
                passed: context_updated,
                expected: format!("Active tenant: {}", target_tenant),
                actual: if context_updated { "Updated" } else { "Not updated" }.to_string(),
            });

            // Step 5: Verify access to target tenant resources
            let tenant_resources_response = self.env.http_client
                .get(&format!("{}/api/v1/tenants/{}/resources", self.env.config.api_gateway_url, target_tenant))
                .header("Authorization", format!("Bearer {}", new_session_id))
                .header("X-Tenant-ID", target_tenant)
                .send()
                .await;

            let tenant_access = tenant_resources_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "User should have access to target tenant resources".to_string(),
                passed: tenant_access,
                expected: "Access granted".to_string(),
                actual: if tenant_access { "Granted" } else { "Denied" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Tenant Switching Workflow".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Tenant switching workflow test failed".to_string()) },
            assertions,
        }
    }

    /// Test file upload and management workflow
    async fn test_file_management_workflow(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Step 1: Login to get auth token
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "File Management Workflow".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "File Management Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Initiate file upload workflow
        let file_content = "This is a test file content for integration testing.";
        let file_upload_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/file-upload", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .json(&json!({
                "file_name": "test-integration.txt",
                "file_size": file_content.len(),
                "content_type": "text/plain",
                "file_content": base64::encode(file_content),
                "storage_provider": "local"
            }))
            .send()
            .await;

        let upload_initiated = file_upload_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "File upload workflow should be initiated".to_string(),
            passed: upload_initiated,
            expected: "200 OK or 202 Accepted".to_string(),
            actual: file_upload_response.as_ref().map(|r| r.status().to_string()).unwrap_or("Error".to_string()),
        });

        let file_id = if upload_initiated {
            let upload_data: serde_json::Value = file_upload_response.unwrap().json().await.unwrap();
            
            // Handle workflow completion
            if let Some(operation_id) = upload_data.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(result_data) => {
                        assertions.push(AssertionResult {
                            description: "File upload workflow should complete successfully".to_string(),
                            passed: true,
                            expected: "Workflow completed".to_string(),
                            actual: "Completed".to_string(),
                        });
                        result_data["file_id"].as_str().unwrap().to_string()
                    }
                    Err(e) => {
                        assertions.push(AssertionResult {
                            description: "File upload workflow should complete successfully".to_string(),
                            passed: false,
                            expected: "Workflow completed".to_string(),
                            actual: format!("Failed: {}", e),
                        });
                        return TestResult {
                            test_name: "File Management Workflow".to_string(),
                            status: TestStatus::Failed,
                            execution_time_ms: test_start.elapsed().as_millis() as u64,
                            error_message: Some(format!("File upload workflow failed: {}", e)),
                            assertions,
                        };
                    }
                }
            } else {
                upload_data["file_id"].as_str().unwrap().to_string()
            }
        } else {
            return TestResult {
                test_name: "File Management Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("File upload initiation failed".to_string()),
                assertions,
            };
        };

        // Step 3: Verify file exists
        let file_info_response = self.env.http_client
            .get(&format!("{}/api/v1/files/{}", self.env.config.api_gateway_url, file_id))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await;

        let file_exists = file_info_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "File should exist after upload".to_string(),
            passed: file_exists,
            expected: "File found".to_string(),
            actual: if file_exists { "Found" } else { "Not found" }.to_string(),
        });

        // Step 4: Test file download
        let file_download_response = self.env.http_client
            .get(&format!("{}/api/v1/files/{}/download", self.env.config.api_gateway_url, file_id))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await;

        let download_successful = file_download_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "File should be downloadable".to_string(),
            passed: download_successful,
            expected: "Download successful".to_string(),
            actual: if download_successful { "Success" } else { "Failed" }.to_string(),
        });

        // Step 5: Test file sharing workflow
        let share_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/file-share", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .json(&json!({
                "file_id": file_id,
                "share_with": "user2@test.com",
                "permission_level": "read",
                "expiry_hours": 24
            }))
            .send()
            .await;

        let share_initiated = share_response
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "File sharing workflow should be initiated".to_string(),
            passed: share_initiated,
            expected: "Sharing initiated".to_string(),
            actual: if share_initiated { "Initiated" } else { "Failed" }.to_string(),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "File Management Workflow".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("File management workflow test failed".to_string()) },
            assertions,
        }
    }

    /// Test user profile management workflow
    async fn test_user_profile_workflow(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Step 1: Login
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "User Profile Workflow".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "User Profile Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Get current profile
        let profile_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await;

        let profile_retrieved = profile_response
            .as_ref()
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "User profile should be retrievable".to_string(),
            passed: profile_retrieved,
            expected: "Profile retrieved".to_string(),
            actual: if profile_retrieved { "Retrieved" } else { "Failed" }.to_string(),
        });

        // Step 3: Update profile
        let update_response = self.env.http_client
            .put(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .json(&json!({
                "first_name": "Updated",
                "last_name": "Name",
                "preferences": {
                    "theme": "dark",
                    "language": "en",
                    "notifications": true
                }
            }))
            .send()
            .await;

        let profile_updated = update_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "User profile should be updatable".to_string(),
            passed: profile_updated,
            expected: "Profile updated".to_string(),
            actual: if profile_updated { "Updated" } else { "Failed" }.to_string(),
        });

        // Step 4: Verify update
        let verify_response = self.env.http_client
            .get(&format!("{}/api/v1/users/profile", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await;

        let update_verified = if let Ok(response) = verify_response {
            if response.status().is_success() {
                let profile_data: serde_json::Value = response.json().await.unwrap();
                profile_data["first_name"].as_str() == Some("Updated")
            } else {
                false
            }
        } else {
            false
        };

        assertions.push(AssertionResult {
            description: "Profile updates should be persisted".to_string(),
            passed: update_verified,
            expected: "Updates persisted".to_string(),
            actual: if update_verified { "Persisted" } else { "Not persisted" }.to_string(),
        });

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "User Profile Workflow".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("User profile workflow test failed".to_string()) },
            assertions,
        }
    }

    /// Test module installation and usage workflow
    async fn test_module_workflow(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Step 1: Login
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Module Workflow".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Module Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: List available modules
        let modules_response = self.env.http_client
            .get(&format!("{}/api/v1/modules/marketplace", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await;

        let modules_listed = modules_response
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Module marketplace should be accessible".to_string(),
            passed: modules_listed,
            expected: "Modules listed".to_string(),
            actual: if modules_listed { "Listed" } else { "Failed" }.to_string(),
        });

        // Step 3: Install a test module
        let install_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/install-module", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .json(&json!({
                "module_id": "test-analytics-module",
                "version": "1.0.0"
            }))
            .send()
            .await;

        let install_initiated = install_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Module installation workflow should be initiated".to_string(),
            passed: install_initiated,
            expected: "Installation initiated".to_string(),
            actual: if install_initiated { "Initiated" } else { "Failed" }.to_string(),
        });

        if install_initiated {
            let install_data: serde_json::Value = install_response.unwrap().json().await.unwrap();
            
            // Handle workflow completion
            if let Some(operation_id) = install_data.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(_) => {
                        assertions.push(AssertionResult {
                            description: "Module installation should complete successfully".to_string(),
                            passed: true,
                            expected: "Installation completed".to_string(),
                            actual: "Completed".to_string(),
                        });
                    }
                    Err(e) => {
                        assertions.push(AssertionResult {
                            description: "Module installation should complete successfully".to_string(),
                            passed: false,
                            expected: "Installation completed".to_string(),
                            actual: format!("Failed: {}", e),
                        });
                    }
                }
            }

            // Step 4: Verify module is installed
            let installed_modules_response = self.env.http_client
                .get(&format!("{}/api/v1/modules/installed", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", tenant_id)
                .send()
                .await;

            let module_installed = if let Ok(response) = installed_modules_response {
                if response.status().is_success() {
                    let modules_data: serde_json::Value = response.json().await.unwrap();
                    modules_data.as_array()
                        .map(|modules| modules.iter().any(|m| m["module_id"] == "test-analytics-module"))
                        .unwrap_or(false)
                } else {
                    false
                }
            } else {
                false
            };

            assertions.push(AssertionResult {
                description: "Module should appear in installed modules list".to_string(),
                passed: module_installed,
                expected: "Module in installed list".to_string(),
                actual: if module_installed { "Found" } else { "Not found" }.to_string(),
            });

            // Step 5: Test module functionality
            let module_function_response = self.env.http_client
                .post(&format!("{}/api/v1/modules/test-analytics-module/analyze", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", tenant_id)
                .json(&json!({
                    "data": "test data for analysis"
                }))
                .send()
                .await;

            let module_functional = module_function_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Installed module should be functional".to_string(),
                passed: module_functional,
                expected: "Module function works".to_string(),
                actual: if module_functional { "Working" } else { "Not working" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Module Workflow".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Module workflow test failed".to_string()) },
            assertions,
        }
    }

    /// Test workflow monitoring and management
    async fn test_workflow_monitoring(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        let test_user = &self.test_data.users[0];
        let tenant_id = &test_user.tenant_ids[0];

        // Step 1: Login
        let login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
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
                    test_name: "Workflow Monitoring".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "Workflow Monitoring".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Login request failed".to_string()),
                assertions,
            };
        };

        // Step 2: Start a long-running workflow
        let long_workflow_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/long-running-test", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .json(&json!({
                "duration_seconds": 10,
                "steps": 5
            }))
            .send()
            .await;

        let workflow_started = long_workflow_response
            .as_ref()
            .map(|r| r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "Long-running workflow should be started".to_string(),
            passed: workflow_started,
            expected: "202 Accepted".to_string(),
            actual: long_workflow_response.as_ref().map(|r| r.status().to_string()).unwrap_or("Error".to_string()),
        });

        if workflow_started {
            let workflow_data: serde_json::Value = long_workflow_response.unwrap().json().await.unwrap();
            let operation_id = workflow_data["operation_id"].as_str().unwrap();

            // Step 3: Monitor workflow progress
            sleep(Duration::from_secs(2)).await;

            let status_response = self.env.http_client
                .get(&format!("{}/api/v1/workflows/{}/status", self.env.config.api_gateway_url, operation_id))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", tenant_id)
                .send()
                .await;

            let status_available = status_response
                .as_ref()
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Workflow status should be available".to_string(),
                passed: status_available,
                expected: "Status available".to_string(),
                actual: if status_available { "Available" } else { "Not available" }.to_string(),
            });

            if status_available {
                let status_data: serde_json::Value = status_response.unwrap().json().await.unwrap();
                let has_progress = status_data.get("progress").is_some();

                assertions.push(AssertionResult {
                    description: "Workflow status should include progress information".to_string(),
                    passed: has_progress,
                    expected: "Progress information present".to_string(),
                    actual: if has_progress { "Present" } else { "Missing" }.to_string(),
                });
            }

            // Step 4: List user workflows
            let workflows_list_response = self.env.http_client
                .get(&format!("{}/api/v1/workflows/user", self.env.config.api_gateway_url))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", tenant_id)
                .send()
                .await;

            let workflows_listed = workflows_list_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "User workflows should be listable".to_string(),
                passed: workflows_listed,
                expected: "Workflows listed".to_string(),
                actual: if workflows_listed { "Listed" } else { "Failed" }.to_string(),
            });

            // Step 5: Test workflow cancellation
            let cancel_response = self.env.http_client
                .post(&format!("{}/api/v1/workflows/{}/cancel", self.env.config.api_gateway_url, operation_id))
                .header("Authorization", format!("Bearer {}", auth_token))
                .header("X-Tenant-ID", tenant_id)
                .send()
                .await;

            let cancel_accepted = cancel_response
                .map(|r| r.status().is_success())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Workflow cancellation should be accepted".to_string(),
                passed: cancel_accepted,
                expected: "Cancellation accepted".to_string(),
                actual: if cancel_accepted { "Accepted" } else { "Rejected" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "Workflow Monitoring".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("Workflow monitoring test failed".to_string()) },
            assertions,
        }
    }

    /// Test user deactivation workflow
    async fn test_user_deactivation_workflow(&self) -> TestResult {
        let test_start = Instant::now();
        let mut assertions = Vec::new();

        // Create a temporary user for deactivation test
        let temp_user_email = format!("tempuser{}@test.com", chrono::Utc::now().timestamp());
        let tenant_id = &self.test_data.tenants[0].id;

        // Step 1: Create temporary user
        let create_user_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/user-registration", self.env.config.api_gateway_url))
            .json(&json!({
                "email": temp_user_email,
                "password": "temppassword123",
                "tenant_id": tenant_id,
                "first_name": "Temp",
                "last_name": "User",
                "roles": ["user"]
            }))
            .send()
            .await;

        let user_created = create_user_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        if !user_created {
            return TestResult {
                test_name: "User Deactivation Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Failed to create temporary user".to_string()),
                assertions,
            };
        }

        let create_data: serde_json::Value = create_user_response.unwrap().json().await.unwrap();
        let temp_user_id = if let Some(operation_id) = create_data.get("operation_id") {
            let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
            match result {
                Ok(result_data) => result_data["user_id"].as_str().unwrap().to_string(),
                Err(_) => return TestResult {
                    test_name: "User Deactivation Workflow".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("User creation workflow failed".to_string()),
                    assertions,
                },
            }
        } else {
            create_data["user_id"].as_str().unwrap().to_string()
        };

        // Step 2: Login as admin to deactivate user
        let admin_login_response = self.env.http_client
            .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
            .json(&json!({
                "email": self.test_data.tenants[0].admin_email,
                "password": "admin123",
                "tenant_id": tenant_id
            }))
            .send()
            .await;

        let admin_token = if let Ok(response) = admin_login_response {
            if response.status().is_success() {
                let login_data: serde_json::Value = response.json().await.unwrap();
                login_data["token"].as_str().unwrap().to_string()
            } else {
                return TestResult {
                    test_name: "User Deactivation Workflow".to_string(),
                    status: TestStatus::Failed,
                    execution_time_ms: test_start.elapsed().as_millis() as u64,
                    error_message: Some("Admin login failed".to_string()),
                    assertions,
                };
            }
        } else {
            return TestResult {
                test_name: "User Deactivation Workflow".to_string(),
                status: TestStatus::Failed,
                execution_time_ms: test_start.elapsed().as_millis() as u64,
                error_message: Some("Admin login request failed".to_string()),
                assertions,
            };
        };

        // Step 3: Initiate user deactivation workflow
        let deactivation_response = self.env.http_client
            .post(&format!("{}/api/v1/workflows/user-deactivation", self.env.config.api_gateway_url))
            .header("Authorization", format!("Bearer {}", admin_token))
            .header("X-Tenant-ID", tenant_id)
            .json(&json!({
                "user_id": temp_user_id,
                "reason": "Integration test cleanup",
                "data_retention_days": 30
            }))
            .send()
            .await;

        let deactivation_initiated = deactivation_response
            .as_ref()
            .map(|r| r.status().is_success() || r.status() == 202)
            .unwrap_or(false);

        assertions.push(AssertionResult {
            description: "User deactivation workflow should be initiated".to_string(),
            passed: deactivation_initiated,
            expected: "200 OK or 202 Accepted".to_string(),
            actual: deactivation_response.as_ref().map(|r| r.status().to_string()).unwrap_or("Error".to_string()),
        });

        if deactivation_initiated {
            let deactivation_data: serde_json::Value = deactivation_response.unwrap().json().await.unwrap();
            
            // Handle workflow completion
            if let Some(operation_id) = deactivation_data.get("operation_id") {
                let result = self.env.poll_workflow_completion(operation_id.as_str().unwrap()).await;
                match result {
                    Ok(_) => {
                        assertions.push(AssertionResult {
                            description: "User deactivation workflow should complete successfully".to_string(),
                            passed: true,
                            expected: "Workflow completed".to_string(),
                            actual: "Completed".to_string(),
                        });
                    }
                    Err(e) => {
                        assertions.push(AssertionResult {
                            description: "User deactivation workflow should complete successfully".to_string(),
                            passed: false,
                            expected: "Workflow completed".to_string(),
                            actual: format!("Failed: {}", e),
                        });
                    }
                }
            }

            // Step 4: Verify user cannot login
            let login_attempt_response = self.env.http_client
                .post(&format!("{}/api/v1/auth/login", self.env.config.api_gateway_url))
                .json(&json!({
                    "email": temp_user_email,
                    "password": "temppassword123",
                    "tenant_id": tenant_id
                }))
                .send()
                .await;

            let login_blocked = login_attempt_response
                .map(|r| r.status().is_client_error())
                .unwrap_or(false);

            assertions.push(AssertionResult {
                description: "Deactivated user should not be able to login".to_string(),
                passed: login_blocked,
                expected: "Login blocked".to_string(),
                actual: if login_blocked { "Blocked" } else { "Allowed" }.to_string(),
            });

            // Step 5: Verify user status
            let user_status_response = self.env.http_client
                .get(&format!("{}/api/v1/users/{}/status", self.env.config.api_gateway_url, temp_user_id))
                .header("Authorization", format!("Bearer {}", admin_token))
                .header("X-Tenant-ID", tenant_id)
                .send()
                .await;

            let status_correct = if let Ok(response) = user_status_response {
                if response.status().is_success() {
                    let status_data: serde_json::Value = response.json().await.unwrap();
                    status_data["status"].as_str() == Some("deactivated")
                } else {
                    false
                }
            } else {
                false
            };

            assertions.push(AssertionResult {
                description: "User status should be 'deactivated'".to_string(),
                passed: status_correct,
                expected: "Status: deactivated".to_string(),
                actual: if status_correct { "Deactivated" } else { "Other" }.to_string(),
            });
        }

        let all_passed = assertions.iter().all(|a| a.passed);

        TestResult {
            test_name: "User Deactivation Workflow".to_string(),
            status: if all_passed { TestStatus::Passed } else { TestStatus::Failed },
            execution_time_ms: test_start.elapsed().as_millis() as u64,
            error_message: if all_passed { None } else { Some("User deactivation workflow test failed".to_string()) },
            assertions,
        }
    }
}