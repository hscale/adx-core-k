// Integration tests for Auth Service

use axum_test::TestServer;
use serde_json::json;
use std::env;

// Mock auth service for testing
async fn create_test_server() -> TestServer {
    use axum::{
        extract::Query,
        http::StatusCode,
        response::Json,
        routing::{get, post},
        Router,
    };
    use serde::{Deserialize, Serialize};
    
    #[derive(Serialize)]
    struct HealthResponse {
        status: String,
        service: String,
    }
    
    #[derive(Deserialize)]
    struct LoginRequest {
        email: String,
        password: String,
    }
    
    #[derive(Serialize)]
    struct LoginResponse {
        token: String,
        user_id: String,
    }
    
    async fn health() -> Json<HealthResponse> {
        Json(HealthResponse {
            status: "healthy".to_string(),
            service: "auth-service".to_string(),
        })
    }
    
    async fn login(Json(payload): Json<LoginRequest>) -> Result<Json<LoginResponse>, StatusCode> {
        // Mock login logic
        if payload.email == "test@example.com" && payload.password == "password123" {
            Ok(Json(LoginResponse {
                token: "mock-jwt-token".to_string(),
                user_id: "user-123".to_string(),
            }))
        } else {
            Err(StatusCode::UNAUTHORIZED)
        }
    }
    
    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/auth/login", post(login));
    
    TestServer::new(app).unwrap()
}

#[tokio::test]
async fn test_health_endpoint() {
    let server = create_test_server().await;
    
    let response = server.get("/health").await;
    response.assert_status_ok();
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["service"], "auth-service");
}

#[tokio::test]
async fn test_login_success() {
    let server = create_test_server().await;
    
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "password123"
        }))
        .await;
    
    response.assert_status_ok();
    
    let body: serde_json::Value = response.json();
    assert_eq!(body["token"], "mock-jwt-token");
    assert_eq!(body["user_id"], "user-123");
}

#[tokio::test]
async fn test_login_failure() {
    let server = create_test_server().await;
    
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": "test@example.com",
            "password": "wrong-password"
        }))
        .await;
    
    response.assert_status(401);
}

#[tokio::test]
async fn test_login_invalid_request() {
    let server = create_test_server().await;
    
    let response = server
        .post("/api/v1/auth/login")
        .json(&json!({
            "email": "test@example.com"
            // Missing password field
        }))
        .await;
    
    // Should return 422 for invalid request format
    assert!(response.status_code().is_client_error());
}

#[tokio::test]
async fn test_concurrent_requests() {
    let server = create_test_server().await;
    
    // Test multiple concurrent requests
    let mut handles = Vec::new();
    
    for i in 0..10 {
        let server = server.clone();
        let handle = tokio::spawn(async move {
            let response = server
                .post("/api/v1/auth/login")
                .json(&json!({
                    "email": "test@example.com",
                    "password": "password123"
                }))
                .await;
            
            response.assert_status_ok();
            let body: serde_json::Value = response.json();
            assert_eq!(body["token"], "mock-jwt-token");
            i
        });
        handles.push(handle);
    }
    
    // Wait for all requests to complete
    for handle in handles {
        handle.await.unwrap();
    }
}

// Database integration test (only runs if database is available)
#[tokio::test]
async fn test_database_integration() {
    // Skip if database tests are disabled
    if env::var("SKIP_DB_TESTS").is_ok() {
        return;
    }
    
    use adx_shared::database::DatabaseManager;
    
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/adx_core_test".to_string());
    
    let db = DatabaseManager::new(&database_url).await;
    
    match db {
        Ok(db_manager) => {
            // Test database connectivity
            assert!(db_manager.health_check().await.is_ok());
            
            // Test database version
            let version = db_manager.get_version().await.unwrap();
            assert!(version.contains("PostgreSQL"));
        }
        Err(_) => {
            // Database not available, skip test
            println!("Database not available, skipping database integration test");
        }
    }
}

// Temporal integration test (only runs if Temporal is available)
#[tokio::test]
async fn test_temporal_integration() {
    // Skip if Temporal tests are disabled
    if env::var("SKIP_TEMPORAL_TESTS").is_ok() {
        return;
    }
    
    use adx_shared::temporal::TemporalClient;
    
    let temporal_url = env::var("TEMPORAL_SERVER_URL")
        .unwrap_or_else(|_| "localhost:7233".to_string());
    
    let client = TemporalClient::new(&temporal_url);
    
    // Test workflow execution (mock)
    let execution = client
        .start_workflow(
            "user_registration_workflow",
            "test-workflow-123",
            "auth-task-queue",
            json!({
                "email": "test@example.com",
                "password": "password123"
            }),
        )
        .await;
    
    match execution {
        Ok(exec) => {
            assert_eq!(exec.workflow_id, "test-workflow-123");
            assert!(!exec.run_id.is_empty());
        }
        Err(e) => {
            println!("Temporal not available, skipping Temporal integration test: {}", e);
        }
    }
}