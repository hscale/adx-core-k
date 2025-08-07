//! # ADX CORE Integration Tests
//!
//! Comprehensive integration tests following enterprise testing patterns
//! with proper error handling, tenant isolation, and workflow testing.

use serde_json::{json, Value};
use std::time::Duration;
use tokio::time::timeout;
use uuid::Uuid;

// ============================================================================
// TEST CONFIGURATION
// ============================================================================

const API_GATEWAY_URL: &str = "http://localhost:8080";
const AUTH_SERVICE_URL: &str = "http://localhost:8081";
const USER_SERVICE_URL: &str = "http://localhost:8082";
const FILE_SERVICE_URL: &str = "http://localhost:8083";
const WORKFLOW_SERVICE_URL: &str = "http://localhost:8084";
const TENANT_SERVICE_URL: &str = "http://localhost:8085";

const TEST_TIMEOUT: Duration = Duration::from_secs(30);
const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// Test tenant ID for consistent testing
const TEST_TENANT_ID: &str = "550e8400-e29b-41d4-a716-446655440000";

// ============================================================================
// HEALTH CHECK TESTS - Enterprise Monitoring
// ============================================================================

#[tokio::test]
async fn test_health_endpoints_comprehensive() {
    let client = reqwest::Client::new();

    // Test all service health endpoints with proper error handling
    let services = vec![
        ("API Gateway", format!("{}/health", API_GATEWAY_URL)),
        ("Auth Service", format!("{}/health", AUTH_SERVICE_URL)),
        ("User Service", format!("{}/health", USER_SERVICE_URL)),
        ("File Service", format!("{}/health", FILE_SERVICE_URL)),
        (
            "Workflow Service",
            format!("{}/health", WORKFLOW_SERVICE_URL),
        ),
        ("Tenant Service", format!("{}/health", TENANT_SERVICE_URL)),
    ];

    for (service_name, health_url) in services {
        let response = timeout(HEALTH_CHECK_TIMEOUT, client.get(&health_url).send())
            .await
            .expect(&format!("{} health check timed out", service_name))
            .expect(&format!("{} health check failed", service_name));

        assert_eq!(
            response.status(),
            200,
            "{} health check returned non-200 status: {}",
            service_name,
            response.status()
        );

        // Verify response format for structured health checks
        if service_name != "API Gateway" {
            let health_data: Value = response.json().await.expect(&format!(
                "{} health response is not valid JSON",
                service_name
            ));

            // Verify ADX CORE health response structure
            assert!(
                health_data["data"]["status"].is_string(),
                "{} health response missing status field",
                service_name
            );

            assert!(
                health_data["metadata"]["correlation_id"].is_string(),
                "{} health response missing correlation_id",
                service_name
            );
        }
    }
}

#[tokio::test]
async fn test_metrics_endpoints() {
    let client = reqwest::Client::new();

    // Test metrics endpoints for monitoring integration
    let services = vec![
        ("Auth Service", format!("{}/metrics", AUTH_SERVICE_URL)),
        ("User Service", format!("{}/metrics", USER_SERVICE_URL)),
    ];

    for (service_name, metrics_url) in services {
        let response = timeout(TEST_TIMEOUT, client.get(&metrics_url).send())
            .await
            .expect(&format!("{} metrics check timed out", service_name))
            .expect(&format!("{} metrics check failed", service_name));

        assert_eq!(
            response.status(),
            200,
            "{} metrics endpoint returned non-200 status: {}",
            service_name,
            response.status()
        );

        let metrics_data: Value = response.json().await.expect(&format!(
            "{} metrics response is not valid JSON",
            service_name
        ));

        // Verify metrics structure
        assert!(
            metrics_data["data"]["requests_per_minute"].is_number(),
            "{} metrics missing requests_per_minute",
            service_name
        );
    }
}

// ============================================================================
// AUTHENTICATION FLOW TESTS - Security & JWT
// ============================================================================

#[tokio::test]
async fn test_authentication_flow_comprehensive() {
    let client = reqwest::Client::new();

    // Test complete authentication flow with tenant isolation
    let login_request = json!({
        "email": "admin@example.com",
        "password": "password",
        "tenant_id": TEST_TENANT_ID
    });

    // Test login with proper error handling
    let login_response = timeout(
        TEST_TIMEOUT,
        client
            .post(&format!("{}/api/v1/auth/login", AUTH_SERVICE_URL))
            .json(&login_request)
            .send(),
    )
    .await
    .expect("Login request timed out")
    .expect("Login request failed");

    assert_eq!(login_response.status(), 200, "Login failed");

    let login_data: Value = login_response
        .json()
        .await
        .expect("Login response is not valid JSON");

    // Verify ADX CORE response structure
    assert!(
        login_data["data"]["access_token"].is_string(),
        "Missing access_token"
    );
    assert!(
        login_data["data"]["refresh_token"].is_string(),
        "Missing refresh_token"
    );
    assert!(login_data["data"]["user_id"].is_string(), "Missing user_id");
    assert!(
        login_data["metadata"]["correlation_id"].is_string(),
        "Missing correlation_id"
    );

    let access_token = login_data["data"]["access_token"].as_str().unwrap();
    let user_id = login_data["data"]["user_id"].as_str().unwrap();

    // Test token validation
    let validation_request = json!({
        "token": access_token
    });

    let validation_response = timeout(
        TEST_TIMEOUT,
        client
            .post(&format!("{}/api/v1/auth/validate", AUTH_SERVICE_URL))
            .json(&validation_request)
            .send(),
    )
    .await
    .expect("Token validation timed out")
    .expect("Token validation failed");

    assert_eq!(validation_response.status(), 200, "Token validation failed");

    let validation_data: Value = validation_response
        .json()
        .await
        .expect("Validation response is not valid JSON");

    assert_eq!(validation_data["data"]["valid"].as_bool().unwrap(), true);
    assert_eq!(
        validation_data["data"]["user_id"].as_str().unwrap(),
        user_id
    );
    assert_eq!(
        validation_data["data"]["tenant_id"].as_str().unwrap(),
        TEST_TENANT_ID
    );

    // Test invalid token
    let invalid_validation = json!({
        "token": "invalid_token_12345"
    });

    let invalid_response = client
        .post(&format!("{}/api/v1/auth/validate", AUTH_SERVICE_URL))
        .json(&invalid_validation)
        .send()
        .await
        .expect("Invalid token test failed");

    let invalid_data: Value = invalid_response
        .json()
        .await
        .expect("Invalid token response is not valid JSON");

    assert_eq!(invalid_data["data"]["valid"].as_bool().unwrap(), false);
}

#[tokio::test]
async fn test_rbac_permission_checks() {
    let client = reqwest::Client::new();

    // Test RBAC permission checking (simple operation - should be fast)
    let permission_request = json!({
        "user_id": Uuid::new_v4(),
        "tenant_id": TEST_TENANT_ID,
        "resource": "users",
        "action": "read",
        "context": {}
    });

    let permission_response = timeout(
        Duration::from_millis(100), // RBAC checks should be < 100ms
        client
            .post(&format!(
                "{}/api/v1/auth/permissions/check",
                AUTH_SERVICE_URL
            ))
            .json(&permission_request)
            .send(),
    )
    .await
    .expect("Permission check timed out (should be < 100ms)")
    .expect("Permission check failed");

    assert_eq!(permission_response.status(), 200, "Permission check failed");

    let permission_data: Value = permission_response
        .json()
        .await
        .expect("Permission response is not valid JSON");

    assert!(
        permission_data["data"]["allowed"].is_boolean(),
        "Permission response missing allowed field"
    );
}

// ============================================================================
// USER MANAGEMENT TESTS - Temporal Workflows
// ============================================================================

#[tokio::test]
async fn test_user_management_workflow() {
    let client = reqwest::Client::new();

    // Test user creation through API Gateway (complex operation - triggers workflow)
    let create_request = json!({
        "email": format!("test-{}@example.com", Uuid::new_v4()),
        "password": "securepassword123",
        "full_name": "Test User",
        "tenant_id": TEST_TENANT_ID,
        "roles": ["user"],
        "create_workspace": true
    });

    let create_response = timeout(
        TEST_TIMEOUT,
        client
            .post(&format!("{}/api/v1/users", API_GATEWAY_URL))
            .json(&create_request)
            .send(),
    )
    .await
    .expect("User creation timed out")
    .expect("User creation failed");

    // User creation should trigger workflow, expect workflow response
    if create_response.status() == 202 {
        // Workflow started
        let workflow_data: Value = create_response
            .json()
            .await
            .expect("Workflow response is not valid JSON");

        assert!(
            workflow_data["data"]["workflow_id"].is_string(),
            "Missing workflow_id in workflow response"
        );

        let workflow_id = workflow_data["data"]["workflow_id"].as_str().unwrap();

        // Poll workflow status until completion
        let mut attempts = 0;
        let max_attempts = 30; // 30 seconds max

        loop {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let status_response = client
                .get(&format!(
                    "{}/api/v1/auth/workflows/{}/status",
                    AUTH_SERVICE_URL, workflow_id
                ))
                .send()
                .await
                .expect("Workflow status check failed");

            if status_response.status() == 200 {
                let status_data: Value = status_response
                    .json()
                    .await
                    .expect("Workflow status response is not valid JSON");

                let status = status_data["data"]["status"].as_str().unwrap();

                if status == "completed" {
                    // Workflow completed successfully
                    break;
                } else if status == "failed" {
                    panic!("User creation workflow failed: {:?}", status_data);
                }
            }

            attempts += 1;
            if attempts >= max_attempts {
                panic!(
                    "User creation workflow did not complete within {} seconds",
                    max_attempts
                );
            }
        }
    } else if create_response.status() == 200 {
        // Direct creation (simple operation)
        let created_user: Value = create_response
            .json()
            .await
            .expect("User creation response is not valid JSON");

        assert_eq!(
            created_user["data"]["email"].as_str().unwrap(),
            create_request["email"].as_str().unwrap()
        );

        let user_id = created_user["data"]["id"].as_str().unwrap();

        // Test getting the created user with tenant isolation
        let get_response = timeout(
            TEST_TIMEOUT,
            client
                .get(&format!("{}/api/v1/users/{}", API_GATEWAY_URL, user_id))
                .header("X-Tenant-ID", TEST_TENANT_ID)
                .send(),
        )
        .await
        .expect("Get user timed out")
        .expect("Get user failed");

        assert_eq!(get_response.status(), 200, "Get user failed");

        let user_data: Value = get_response
            .json()
            .await
            .expect("Get user response is not valid JSON");

        assert_eq!(
            user_data["data"]["email"].as_str().unwrap(),
            create_request["email"].as_str().unwrap()
        );

        // Verify tenant isolation
        assert_eq!(
            user_data["data"]["tenant_id"].as_str().unwrap(),
            TEST_TENANT_ID
        );
    } else {
        panic!(
            "Unexpected user creation response status: {}",
            create_response.status()
        );
    }
}

// ============================================================================
// API GATEWAY TESTS - Routing & Load Balancing
// ============================================================================

#[tokio::test]
async fn test_api_gateway_routing_comprehensive() {
    let client = reqwest::Client::new();

    // Test routing to different services through API Gateway
    let routes = vec![
        ("users", format!("{}/api/v1/users", API_GATEWAY_URL)),
        ("auth", format!("{}/api/v1/auth/validate", API_GATEWAY_URL)),
        ("files", format!("{}/api/v1/files", API_GATEWAY_URL)),
        ("tenants", format!("{}/api/v1/tenants", API_GATEWAY_URL)),
    ];

    for (service, url) in routes {
        let response = timeout(TEST_TIMEOUT, client.get(&url).send())
            .await
            .expect(&format!("{} routing timed out", service))
            .expect(&format!("{} routing failed", service));

        // Should get a response (even if 401/403 due to auth)
        assert!(
            response.status().as_u16() < 500,
            "{} routing returned server error: {}",
            service,
            response.status()
        );
    }
}

#[tokio::test]
async fn test_api_gateway_auth_middleware() {
    let client = reqwest::Client::new();

    // Test that protected routes require authentication
    let protected_routes = vec![
        format!("{}/api/v1/users", API_GATEWAY_URL),
        format!("{}/api/v1/files", API_GATEWAY_URL),
    ];

    for route in protected_routes {
        let response = client
            .get(&route)
            .send()
            .await
            .expect("Protected route test failed");

        // Should require authentication
        assert!(
            response.status() == 401 || response.status() == 403,
            "Protected route {} should require authentication, got: {}",
            route,
            response.status()
        );
    }
}

// ============================================================================
// TEMPORAL WORKFLOW TESTS - Complex Operations
// ============================================================================

#[tokio::test]
async fn test_workflow_execution() {
    let client = reqwest::Client::new();

    // Test file upload workflow (complex operation)
    let file_upload_request = json!({
        "filename": "test-document.pdf",
        "content_type": "application/pdf",
        "size": 1024000,
        "tenant_id": TEST_TENANT_ID,
        "user_id": Uuid::new_v4(),
        "metadata": {
            "department": "engineering",
            "classification": "internal"
        }
    });

    let upload_response = client
        .post(&format!("{}/api/v1/files/upload", API_GATEWAY_URL))
        .json(&file_upload_request)
        .send()
        .await
        .expect("File upload workflow failed");

    // Should return workflow ID for complex operation
    if upload_response.status() == 202 {
        let workflow_data: Value = upload_response
            .json()
            .await
            .expect("Upload workflow response is not valid JSON");

        assert!(
            workflow_data["data"]["workflow_id"].is_string(),
            "File upload should return workflow_id"
        );

        assert_eq!(workflow_data["data"]["status"].as_str().unwrap(), "started");
    }
}

// ============================================================================
// TENANT ISOLATION TESTS - Multi-Tenancy
// ============================================================================

#[tokio::test]
async fn test_tenant_isolation() {
    let client = reqwest::Client::new();

    let tenant_a = "550e8400-e29b-41d4-a716-446655440000";
    let tenant_b = "550e8400-e29b-41d4-a716-446655440001";

    // Create user in tenant A
    let user_a_request = json!({
        "email": format!("user-a-{}@example.com", Uuid::new_v4()),
        "password": "password123",
        "tenant_id": tenant_a
    });

    let user_a_response = client
        .post(&format!("{}/api/v1/users", API_GATEWAY_URL))
        .json(&user_a_request)
        .send()
        .await
        .expect("User A creation failed");

    if user_a_response.status() == 200 {
        let user_a_data: Value = user_a_response
            .json()
            .await
            .expect("User A response is not valid JSON");

        let user_a_id = user_a_data["data"]["id"].as_str().unwrap();

        // Try to access user A from tenant B (should fail)
        let unauthorized_response = client
            .get(&format!("{}/api/v1/users/{}", API_GATEWAY_URL, user_a_id))
            .header("X-Tenant-ID", tenant_b)
            .send()
            .await
            .expect("Tenant isolation test failed");

        // Should not be able to access user from different tenant
        assert!(
            unauthorized_response.status() == 404 || unauthorized_response.status() == 403,
            "Tenant isolation failed: user from tenant A accessible from tenant B"
        );
    }
}

// ============================================================================
// PERFORMANCE TESTS - Enterprise SLA
// ============================================================================

#[tokio::test]
async fn test_performance_requirements() {
    let client = reqwest::Client::new();

    // Test that simple operations meet performance requirements
    let start = std::time::Instant::now();

    let health_response = client
        .get(&format!("{}/health", API_GATEWAY_URL))
        .send()
        .await
        .expect("Health check failed");

    let health_duration = start.elapsed();

    assert_eq!(health_response.status(), 200);
    assert!(
        health_duration < Duration::from_millis(100),
        "Health check took {}ms, should be < 100ms",
        health_duration.as_millis()
    );

    // Test permission check performance (should be < 10ms)
    let permission_start = std::time::Instant::now();

    let permission_request = json!({
        "user_id": Uuid::new_v4(),
        "tenant_id": TEST_TENANT_ID,
        "resource": "files",
        "action": "read"
    });

    let permission_response = client
        .post(&format!(
            "{}/api/v1/auth/permissions/check",
            AUTH_SERVICE_URL
        ))
        .json(&permission_request)
        .send()
        .await
        .expect("Permission check failed");

    let permission_duration = permission_start.elapsed();

    assert_eq!(permission_response.status(), 200);
    assert!(
        permission_duration < Duration::from_millis(50),
        "Permission check took {}ms, should be < 50ms for enterprise SLA",
        permission_duration.as_millis()
    );
}

// ============================================================================
// ERROR HANDLING TESTS - Resilience
// ============================================================================

#[tokio::test]
async fn test_error_handling() {
    let client = reqwest::Client::new();

    // Test invalid JSON handling
    let invalid_response = client
        .post(&format!("{}/api/v1/auth/login", AUTH_SERVICE_URL))
        .body("invalid json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .expect("Invalid JSON test failed");

    assert_eq!(
        invalid_response.status(),
        400,
        "Should return 400 for invalid JSON"
    );

    // Test missing required fields
    let incomplete_request = json!({
        "email": "test@example.com"
        // Missing password and tenant_id
    });

    let incomplete_response = client
        .post(&format!("{}/api/v1/auth/login", AUTH_SERVICE_URL))
        .json(&incomplete_request)
        .send()
        .await
        .expect("Incomplete request test failed");

    assert_eq!(
        incomplete_response.status(),
        400,
        "Should return 400 for incomplete request"
    );

    // Test proper error response structure
    let error_data: Value = incomplete_response
        .json()
        .await
        .expect("Error response is not valid JSON");

    assert!(
        error_data["metadata"]["correlation_id"].is_string(),
        "Error response should include correlation_id for tracking"
    );
}
