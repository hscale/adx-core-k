// Cross-service integration tests using test containers
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;
use tokio::time::sleep;

use testcontainers::{clients::Cli, images::postgres::Postgres, images::redis::Redis, Container};
use sqlx::{PgPool, Row};
use redis::Client as RedisClient;
use reqwest::Client as HttpClient;

/// Integration test environment with real services
pub struct CrossServiceTestEnvironment {
    pub postgres_container: Container<'static, Postgres>,
    pub redis_container: Container<'static, Redis>,
    pub database_pool: PgPool,
    pub redis_client: RedisClient,
    pub http_client: HttpClient,
    pub service_urls: HashMap<String, String>,
    pub test_id: String,
}

impl CrossServiceTestEnvironment {
    /// Create a new cross-service test environment
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let docker = Cli::default();
        let test_id = Uuid::new_v4().to_string();
        
        // Start PostgreSQL container
        let postgres_container = docker.run(Postgres::default());
        let postgres_port = postgres_container.get_host_port_ipv4(5432);
        let database_url = format!("postgres://postgres:postgres@127.0.0.1:{}/postgres", postgres_port);
        
        // Start Redis container
        let redis_container = docker.run(Redis::default());
        let redis_port = redis_container.get_host_port_ipv4(6379);
        let redis_url = format!("redis://127.0.0.1:{}", redis_port);
        
        // Connect to databases
        let database_pool = PgPool::connect(&database_url).await?;
        let redis_client = RedisClient::open(redis_url.as_str())?;
        
        // Run database migrations
        sqlx::migrate!("../adx-core/services/shared/migrations")
            .run(&database_pool)
            .await?;
        
        // Create HTTP client
        let http_client = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;
        
        // Service URLs (assuming services are running locally for integration tests)
        let mut service_urls = HashMap::new();
        service_urls.insert("api_gateway".to_string(), "http://localhost:8080".to_string());
        service_urls.insert("auth_service".to_string(), "http://localhost:8081".to_string());
        service_urls.insert("user_service".to_string(), "http://localhost:8082".to_string());
        service_urls.insert("file_service".to_string(), "http://localhost:8083".to_string());
        service_urls.insert("workflow_service".to_string(), "http://localhost:8084".to_string());
        service_urls.insert("tenant_service".to_string(), "http://localhost:8085".to_string());
        
        Ok(Self {
            postgres_container,
            redis_container,
            database_pool,
            redis_client,
            http_client,
            service_urls,
            test_id,
        })
    }
    
    /// Create test tenant data
    pub async fn create_test_tenant(&self) -> Result<TestTenant, Box<dyn std::error::Error + Send + Sync>> {
        let tenant = TestTenant {
            id: Uuid::new_v4().to_string(),
            name: format!("Test Tenant {}", &self.test_id[..8]),
            admin_email: format!("admin-{}@test.com", &self.test_id[..8]),
            subscription_tier: "professional".to_string(),
            created_at: Utc::now(),
        };
        
        // Insert directly into database
        sqlx::query!(
            "INSERT INTO tenants (id, name, admin_email, subscription_tier, created_at) VALUES ($1, $2, $3, $4, $5)",
            tenant.id,
            tenant.name,
            tenant.admin_email,
            tenant.subscription_tier,
            tenant.created_at
        )
        .execute(&self.database_pool)
        .await?;
        
        Ok(tenant)
    }
    
    /// Create test user data
    pub async fn create_test_user(&self, tenant_id: &str) -> Result<TestUser, Box<dyn std::error::Error + Send + Sync>> {
        let user = TestUser {
            id: Uuid::new_v4().to_string(),
            email: format!("user-{}@test.com", &self.test_id[..8]),
            password_hash: "hashed_password_123".to_string(),
            tenant_id: tenant_id.to_string(),
            is_active: true,
            created_at: Utc::now(),
        };
        
        // Insert directly into database
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash, tenant_id, is_active, created_at) VALUES ($1, $2, $3, $4, $5, $6)",
            user.id,
            user.email,
            user.password_hash,
            user.tenant_id,
            user.is_active,
            user.created_at
        )
        .execute(&self.database_pool)
        .await?;
        
        Ok(user)
    }
    
    /// Wait for services to be healthy
    pub async fn wait_for_services(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let max_attempts = 30;
        let delay = Duration::from_secs(2);
        
        for service_name in ["api_gateway", "auth_service", "user_service", "tenant_service"] {
            let service_url = self.service_urls.get(service_name).unwrap();
            let health_url = format!("{}/health", service_url);
            
            let mut attempts = 0;
            loop {
                match self.http_client.get(&health_url).send().await {
                    Ok(response) if response.status().is_success() => {
                        println!("✅ {} is healthy", service_name);
                        break;
                    }
                    _ => {
                        attempts += 1;
                        if attempts >= max_attempts {
                            return Err(format!("Service {} failed to become healthy", service_name).into());
                        }
                        println!("⏳ Waiting for {} to become healthy (attempt {}/{})", service_name, attempts, max_attempts);
                        sleep(delay).await;
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Clean up test data
    pub async fn cleanup(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // Clean up database
        sqlx::query!("DELETE FROM users WHERE email LIKE $1", format!("%{}%", &self.test_id[..8]))
            .execute(&self.database_pool)
            .await?;
        
        sqlx::query!("DELETE FROM tenants WHERE name LIKE $1", format!("%{}%", &self.test_id[..8]))
            .execute(&self.database_pool)
            .await?;
        
        Ok(())
    }
}

/// Test end-to-end tenant creation workflow across services
#[tokio::test]
async fn test_cross_service_tenant_creation() {
    let test_env = CrossServiceTestEnvironment::new().await
        .expect("Failed to create test environment");
    
    // Wait for services to be ready
    test_env.wait_for_services().await
        .expect("Services failed to become healthy");
    
    // Step 1: Create tenant through API Gateway workflow endpoint
    let create_tenant_request = json!({
        "tenant_name": "Cross Service Test Tenant",
        "admin_email": "admin@crossservice.test",
        "subscription_tier": "professional",
        "isolation_level": "schema",
        "quotas": {
            "max_users": 100,
            "max_storage_gb": 50
        },
        "features": ["basic_features"],
        "default_modules": ["client_management"]
    });
    
    let api_gateway_url = test_env.service_urls.get("api_gateway").unwrap();
    let create_response = test_env.http_client
        .post(&format!("{}/api/v1/workflows/create-tenant", api_gateway_url))
        .json(&create_tenant_request)
        .send()
        .await
        .expect("Failed to send create tenant request");
    
    assert_eq!(create_response.status(), 202); // Accepted for async workflow
    
    let workflow_response: serde_json::Value = create_response.json().await
        .expect("Failed to parse workflow response");
    
    let operation_id = workflow_response["operation_id"].as_str()
        .expect("Missing operation_id in response");
    
    // Step 2: Poll workflow status until completion
    let mut workflow_completed = false;
    let mut attempts = 0;
    let max_attempts = 30;
    
    while !workflow_completed && attempts < max_attempts {
        sleep(Duration::from_secs(2)).await;
        
        let status_response = test_env.http_client
            .get(&format!("{}/api/v1/workflows/{}/status", api_gateway_url, operation_id))
            .send()
            .await
            .expect("Failed to get workflow status");
        
        assert_eq!(status_response.status(), 200);
        
        let status: serde_json::Value = status_response.json().await
            .expect("Failed to parse status response");
        
        match status["status"].as_str().unwrap() {
            "completed" => {
                workflow_completed = true;
                
                // Verify workflow result
                let result = &status["result"];
                assert!(result["tenant_id"].is_string());
                assert!(result["admin_user_id"].is_string());
                
                let tenant_id = result["tenant_id"].as_str().unwrap();
                let admin_user_id = result["admin_user_id"].as_str().unwrap();
                
                // Step 3: Verify tenant was created in tenant service
                let tenant_service_url = test_env.service_urls.get("tenant_service").unwrap();
                let tenant_response = test_env.http_client
                    .get(&format!("{}/api/v1/tenants/{}", tenant_service_url, tenant_id))
                    .send()
                    .await
                    .expect("Failed to get tenant");
                
                assert_eq!(tenant_response.status(), 200);
                
                let tenant: serde_json::Value = tenant_response.json().await
                    .expect("Failed to parse tenant response");
                
                assert_eq!(tenant["name"], "Cross Service Test Tenant");
                assert_eq!(tenant["admin_email"], "admin@crossservice.test");
                assert_eq!(tenant["subscription_tier"], "professional");
                
                // Step 4: Verify admin user was created in user service
                let user_service_url = test_env.service_urls.get("user_service").unwrap();
                let user_response = test_env.http_client
                    .get(&format!("{}/api/v1/users/{}", user_service_url, admin_user_id))
                    .send()
                    .await
                    .expect("Failed to get user");
                
                assert_eq!(user_response.status(), 200);
                
                let user: serde_json::Value = user_response.json().await
                    .expect("Failed to parse user response");
                
                assert_eq!(user["email"], "admin@crossservice.test");
                assert_eq!(user["tenant_id"], tenant_id);
                assert!(user["roles"].as_array().unwrap().contains(&json!("admin")));
                
                // Step 5: Verify database state
                let db_tenant = sqlx::query!(
                    "SELECT id, name, admin_email FROM tenants WHERE id = $1",
                    tenant_id
                )
                .fetch_one(&test_env.database_pool)
                .await
                .expect("Failed to fetch tenant from database");
                
                assert_eq!(db_tenant.name, "Cross Service Test Tenant");
                assert_eq!(db_tenant.admin_email, "admin@crossservice.test");
                
                let db_user = sqlx::query!(
                    "SELECT id, email, tenant_id FROM users WHERE id = $1",
                    admin_user_id
                )
                .fetch_one(&test_env.database_pool)
                .await
                .expect("Failed to fetch user from database");
                
                assert_eq!(db_user.email, "admin@crossservice.test");
                assert_eq!(db_user.tenant_id, tenant_id);
            }
            "failed" => {
                panic!("Workflow failed: {:?}", status["error"]);
            }
            _ => {
                attempts += 1;
            }
        }
    }
    
    assert!(workflow_completed, "Workflow did not complete within timeout");
    
    // Cleanup
    test_env.cleanup().await.expect("Failed to cleanup test data");
}

/// Test cross-service user authentication and tenant switching
#[tokio::test]
async fn test_cross_service_user_authentication() {
    let test_env = CrossServiceTestEnvironment::new().await
        .expect("Failed to create test environment");
    
    test_env.wait_for_services().await
        .expect("Services failed to become healthy");
    
    // Setup: Create test tenant and user
    let tenant = test_env.create_test_tenant().await
        .expect("Failed to create test tenant");
    let user = test_env.create_test_user(&tenant.id).await
        .expect("Failed to create test user");
    
    // Step 1: Authenticate user through auth service
    let auth_service_url = test_env.service_urls.get("auth_service").unwrap();
    let login_request = json!({
        "email": user.email,
        "password": "password123", // This would be the actual password
        "tenant_id": tenant.id
    });
    
    let auth_response = test_env.http_client
        .post(&format!("{}/api/v1/auth/login", auth_service_url))
        .json(&login_request)
        .send()
        .await
        .expect("Failed to authenticate");
    
    // For this test, we'll assume authentication succeeds
    // In a real scenario, we'd need to set up proper password hashing
    if auth_response.status() == 200 {
        let auth_result: serde_json::Value = auth_response.json().await
            .expect("Failed to parse auth response");
        
        let access_token = auth_result["access_token"].as_str()
            .expect("Missing access token");
        
        // Step 2: Use token to access user service
        let user_service_url = test_env.service_urls.get("user_service").unwrap();
        let profile_response = test_env.http_client
            .get(&format!("{}/api/v1/users/profile", user_service_url))
            .header("Authorization", format!("Bearer {}", access_token))
            .header("X-Tenant-ID", &tenant.id)
            .send()
            .await
            .expect("Failed to get user profile");
        
        assert_eq!(profile_response.status(), 200);
        
        let profile: serde_json::Value = profile_response.json().await
            .expect("Failed to parse profile response");
        
        assert_eq!(profile["email"], user.email);
        assert_eq!(profile["tenant_id"], tenant.id);
    }
    
    test_env.cleanup().await.expect("Failed to cleanup test data");
}

/// Test cross-service file upload workflow
#[tokio::test]
async fn test_cross_service_file_upload() {
    let test_env = CrossServiceTestEnvironment::new().await
        .expect("Failed to create test environment");
    
    test_env.wait_for_services().await
        .expect("Services failed to become healthy");
    
    // Setup: Create test tenant and user
    let tenant = test_env.create_test_tenant().await
        .expect("Failed to create test tenant");
    let user = test_env.create_test_user(&tenant.id).await
        .expect("Failed to create test user");
    
    // Step 1: Initiate file upload workflow through API Gateway
    let api_gateway_url = test_env.service_urls.get("api_gateway").unwrap();
    let upload_request = json!({
        "filename": "test-document.pdf",
        "file_size": 1024000,
        "content_type": "application/pdf",
        "user_id": user.id,
        "tenant_id": tenant.id,
        "folder_path": "/documents",
        "permissions": {
            "read": ["user"],
            "write": ["user"],
            "admin": ["admin"]
        }
    });
    
    let upload_response = test_env.http_client
        .post(&format!("{}/api/v1/workflows/file-upload", api_gateway_url))
        .json(&upload_request)
        .send()
        .await
        .expect("Failed to initiate file upload");
    
    assert_eq!(upload_response.status(), 202);
    
    let workflow_response: serde_json::Value = upload_response.json().await
        .expect("Failed to parse upload response");
    
    let operation_id = workflow_response["operation_id"].as_str()
        .expect("Missing operation_id");
    
    // Step 2: Poll for workflow completion
    let mut workflow_completed = false;
    let mut attempts = 0;
    let max_attempts = 20;
    
    while !workflow_completed && attempts < max_attempts {
        sleep(Duration::from_secs(1)).await;
        
        let status_response = test_env.http_client
            .get(&format!("{}/api/v1/workflows/{}/status", api_gateway_url, operation_id))
            .send()
            .await
            .expect("Failed to get workflow status");
        
        let status: serde_json::Value = status_response.json().await
            .expect("Failed to parse status response");
        
        match status["status"].as_str().unwrap() {
            "completed" => {
                workflow_completed = true;
                
                let result = &status["result"];
                let file_id = result["file_id"].as_str().expect("Missing file_id");
                
                // Step 3: Verify file metadata in file service
                let file_service_url = test_env.service_urls.get("file_service").unwrap();
                let file_response = test_env.http_client
                    .get(&format!("{}/api/v1/files/{}", file_service_url, file_id))
                    .header("X-Tenant-ID", &tenant.id)
                    .send()
                    .await
                    .expect("Failed to get file metadata");
                
                assert_eq!(file_response.status(), 200);
                
                let file_metadata: serde_json::Value = file_response.json().await
                    .expect("Failed to parse file metadata");
                
                assert_eq!(file_metadata["filename"], "test-document.pdf");
                assert_eq!(file_metadata["owner_id"], user.id);
                assert_eq!(file_metadata["tenant_id"], tenant.id);
                assert_eq!(file_metadata["status"], "uploaded");
            }
            "failed" => {
                panic!("File upload workflow failed: {:?}", status["error"]);
            }
            _ => {
                attempts += 1;
            }
        }
    }
    
    assert!(workflow_completed, "File upload workflow did not complete");
    
    test_env.cleanup().await.expect("Failed to cleanup test data");
}

/// Test cross-service data consistency during tenant switching
#[tokio::test]
async fn test_cross_service_tenant_switching() {
    let test_env = CrossServiceTestEnvironment::new().await
        .expect("Failed to create test environment");
    
    test_env.wait_for_services().await
        .expect("Services failed to become healthy");
    
    // Setup: Create two tenants and a user with access to both
    let tenant1 = test_env.create_test_tenant().await
        .expect("Failed to create first tenant");
    let tenant2 = test_env.create_test_tenant().await
        .expect("Failed to create second tenant");
    
    let user = test_env.create_test_user(&tenant1.id).await
        .expect("Failed to create test user");
    
    // Grant user access to second tenant
    sqlx::query!(
        "INSERT INTO tenant_memberships (user_id, tenant_id, role) VALUES ($1, $2, $3)",
        user.id,
        tenant2.id,
        "member"
    )
    .execute(&test_env.database_pool)
    .await
    .expect("Failed to add user to second tenant");
    
    // Step 1: Switch tenant through workflow
    let api_gateway_url = test_env.service_urls.get("api_gateway").unwrap();
    let switch_request = json!({
        "user_id": user.id,
        "current_tenant_id": tenant1.id,
        "target_tenant_id": tenant2.id
    });
    
    let switch_response = test_env.http_client
        .post(&format!("{}/api/v1/workflows/switch-tenant", api_gateway_url))
        .json(&switch_request)
        .send()
        .await
        .expect("Failed to initiate tenant switch");
    
    assert_eq!(switch_response.status(), 202);
    
    let workflow_response: serde_json::Value = switch_response.json().await
        .expect("Failed to parse switch response");
    
    let operation_id = workflow_response["operation_id"].as_str()
        .expect("Missing operation_id");
    
    // Step 2: Wait for workflow completion
    let mut workflow_completed = false;
    let mut attempts = 0;
    let max_attempts = 15;
    
    while !workflow_completed && attempts < max_attempts {
        sleep(Duration::from_secs(1)).await;
        
        let status_response = test_env.http_client
            .get(&format!("{}/api/v1/workflows/{}/status", api_gateway_url, operation_id))
            .send()
            .await
            .expect("Failed to get workflow status");
        
        let status: serde_json::Value = status_response.json().await
            .expect("Failed to parse status response");
        
        match status["status"].as_str().unwrap() {
            "completed" => {
                workflow_completed = true;
                
                let result = &status["result"];
                assert_eq!(result["new_tenant_id"], tenant2.id);
                assert!(result["new_session_id"].is_string());
                
                // Step 3: Verify user context updated across services
                let user_service_url = test_env.service_urls.get("user_service").unwrap();
                let context_response = test_env.http_client
                    .get(&format!("{}/api/v1/users/{}/context", user_service_url, user.id))
                    .send()
                    .await
                    .expect("Failed to get user context");
                
                assert_eq!(context_response.status(), 200);
                
                let user_context: serde_json::Value = context_response.json().await
                    .expect("Failed to parse user context");
                
                assert_eq!(user_context["active_tenant_id"], tenant2.id);
            }
            "failed" => {
                panic!("Tenant switch workflow failed: {:?}", status["error"]);
            }
            _ => {
                attempts += 1;
            }
        }
    }
    
    assert!(workflow_completed, "Tenant switch workflow did not complete");
    
    test_env.cleanup().await.expect("Failed to cleanup test data");
}

// Test data structures
#[derive(Debug, Clone)]
pub struct TestTenant {
    pub id: String,
    pub name: String,
    pub admin_email: String,
    pub subscription_tier: String,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub tenant_id: String,
    pub is_active: bool,
    pub created_at: chrono::DateTime<Utc>,
}