// Test environment setup and management
use super::*;
use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;
use reqwest::Client;
use sqlx::{PgPool, Pool, Postgres};
use redis::aio::ConnectionManager;
use testcontainers::{clients::Cli, images::postgres::Postgres as PostgresImage, Container};

/// Comprehensive test environment that manages all services
pub struct IntegrationTestEnvironment {
    pub config: TestEnvironmentConfig,
    pub database_pool: PgPool,
    pub redis_client: ConnectionManager,
    pub http_client: Client,
    pub service_processes: HashMap<String, std::process::Child>,
    pub containers: Vec<Box<dyn std::any::Any>>,
    pub temporal_client: Option<temporal_client::Client>,
}

impl IntegrationTestEnvironment {
    /// Initialize complete test environment with all services
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = TestEnvironmentConfig::default();
        
        // Start infrastructure containers
        let docker = Cli::default();
        let postgres_container = docker.run(PostgresImage::default());
        let database_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres_container.get_host_port_ipv4(5432)
        );

        // Setup database
        let database_pool = PgPool::connect(&database_url).await?;
        sqlx::migrate!("../adx-core/services/shared/migrations").run(&database_pool).await?;

        // Setup Redis
        let redis_client = redis::Client::open(config.redis_url.as_str())?;
        let redis_connection = ConnectionManager::new(redis_client).await?;

        // Setup HTTP client
        let http_client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        let mut env = Self {
            config,
            database_pool,
            redis_client: redis_connection,
            http_client,
            service_processes: HashMap::new(),
            containers: vec![Box::new(postgres_container)],
            temporal_client: None,
        };

        // Start all services
        env.start_all_services().await?;
        
        // Wait for services to be ready
        env.wait_for_services_ready().await?;

        Ok(env)
    }

    /// Start all microservices and infrastructure
    async fn start_all_services(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting all ADX CORE services...");

        // Start Temporal server
        self.start_temporal_server().await?;

        // Start backend services
        self.start_backend_services().await?;

        // Start BFF services
        self.start_bff_services().await?;

        // Start frontend services
        self.start_frontend_services().await?;

        Ok(())
    }

    async fn start_temporal_server(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Starting Temporal server...");
        
        let temporal_process = Command::new("docker")
            .args(&[
                "run", "-d", "--rm",
                "-p", "7233:7233",
                "-p", "8088:8088",
                "--name", "temporal-test",
                "temporalio/auto-setup:latest"
            ])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        self.service_processes.insert("temporal".to_string(), temporal_process);
        
        // Wait for Temporal to be ready
        sleep(Duration::from_secs(10)).await;
        
        Ok(())
    }

    async fn start_backend_services(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let services = vec![
            ("api-gateway", "8080"),
            ("auth-service", "8081"),
            ("user-service", "8082"),
            ("file-service", "8083"),
            ("workflow-service", "8084"),
            ("tenant-service", "8085"),
            ("module-service", "8086"),
            ("license-service", "8087"),
        ];

        for (service_name, port) in services {
            println!("Starting {}...", service_name);
            
            let process = Command::new("cargo")
                .args(&["run", "--bin", service_name])
                .current_dir("../adx-core")
                .env("DATABASE_URL", &self.config.database_url)
                .env("REDIS_URL", &self.config.redis_url)
                .env("TEMPORAL_URL", &self.config.temporal_url)
                .env("PORT", port)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            self.service_processes.insert(service_name.to_string(), process);
        }

        Ok(())
    }

    async fn start_bff_services(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let bff_services = vec![
            ("auth-bff", "4001"),
            ("tenant-bff", "4002"),
            ("file-bff", "4003"),
            ("user-bff", "4004"),
            ("workflow-bff", "4005"),
        ];

        for (service_name, port) in bff_services {
            println!("Starting {}...", service_name);
            
            let process = if service_name.contains("auth") || service_name.contains("tenant") {
                // Node.js BFF services
                Command::new("npm")
                    .args(&["run", "dev"])
                    .current_dir(&format!("../bff-services/{}", service_name))
                    .env("PORT", port)
                    .env("API_GATEWAY_URL", &self.config.api_gateway_url)
                    .env("REDIS_URL", &self.config.redis_url)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?
            } else {
                // Rust BFF services
                Command::new("cargo")
                    .args(&["run"])
                    .current_dir(&format!("../bff-services/{}", service_name))
                    .env("PORT", port)
                    .env("API_GATEWAY_URL", &self.config.api_gateway_url)
                    .env("REDIS_URL", &self.config.redis_url)
                    .stdout(Stdio::piped())
                    .stderr(Stdio::piped())
                    .spawn()?
            };

            self.service_processes.insert(service_name.to_string(), process);
        }

        Ok(())
    }

    async fn start_frontend_services(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let frontend_services = vec![
            ("shell", "3000"),
            ("auth", "3001"),
            ("tenant", "3002"),
            ("file", "3003"),
            ("user", "3004"),
            ("workflow", "3005"),
            ("module", "3006"),
        ];

        for (service_name, port) in frontend_services {
            println!("Starting {} micro-frontend...", service_name);
            
            let process = Command::new("npm")
                .args(&["run", "dev"])
                .current_dir(&format!("../apps/{}", service_name))
                .env("PORT", port)
                .env("VITE_API_URL", &self.config.api_gateway_url)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?;

            self.service_processes.insert(format!("frontend-{}", service_name), process);
        }

        Ok(())
    }

    /// Wait for all services to be ready
    async fn wait_for_services_ready(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Waiting for services to be ready...");

        let health_checks = vec![
            ("API Gateway", format!("{}/health", self.config.api_gateway_url)),
            ("Auth Service", "http://localhost:8081/health".to_string()),
            ("User Service", "http://localhost:8082/health".to_string()),
            ("File Service", "http://localhost:8083/health".to_string()),
            ("Workflow Service", "http://localhost:8084/health".to_string()),
            ("Tenant Service", "http://localhost:8085/health".to_string()),
            ("Frontend Shell", format!("{}/", self.config.frontend_shell_url)),
        ];

        for (service_name, health_url) in health_checks {
            self.wait_for_service_health(&service_name, &health_url).await?;
        }

        println!("All services are ready!");
        Ok(())
    }

    async fn wait_for_service_health(&self, service_name: &str, health_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let max_attempts = 60; // 60 seconds timeout
        let mut attempts = 0;

        while attempts < max_attempts {
            match self.http_client.get(health_url).send().await {
                Ok(response) if response.status().is_success() => {
                    println!("{} is ready", service_name);
                    return Ok(());
                }
                _ => {
                    attempts += 1;
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Err(format!("{} failed to become ready within timeout", service_name).into())
    }

    /// Create test tenant and users
    pub async fn setup_test_data(&self) -> Result<TestData, Box<dyn std::error::Error>> {
        println!("Setting up test data...");

        // Create test tenants
        let tenant1 = self.create_test_tenant("Test Tenant 1", "admin1@test.com").await?;
        let tenant2 = self.create_test_tenant("Test Tenant 2", "admin2@test.com").await?;

        // Create test users
        let user1 = self.create_test_user("user1@test.com", "password123", vec![tenant1.id.clone()]).await?;
        let user2 = self.create_test_user("user2@test.com", "password123", vec![tenant1.id.clone(), tenant2.id.clone()]).await?;
        let admin_user = self.create_test_user("admin@test.com", "admin123", vec![tenant1.id.clone()]).await?;

        Ok(TestData {
            tenants: vec![tenant1, tenant2],
            users: vec![user1, user2, admin_user],
        })
    }

    async fn create_test_tenant(&self, name: &str, admin_email: &str) -> Result<TestTenant, Box<dyn std::error::Error>> {
        let response = self.http_client
            .post(&format!("{}/api/v1/workflows/create-tenant", self.config.api_gateway_url))
            .json(&serde_json::json!({
                "tenant_name": name,
                "admin_email": admin_email,
                "subscription_tier": "Professional",
                "isolation_level": "Schema",
                "quotas": {
                    "max_users": 100,
                    "max_storage_gb": 10,
                    "max_api_calls_per_hour": 1000,
                    "max_workflows_per_hour": 100
                },
                "features": ["basic_features"],
                "default_modules": ["client_management"]
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let workflow_response: serde_json::Value = response.json().await?;
            
            // Poll for workflow completion
            if let Some(operation_id) = workflow_response.get("operation_id") {
                let result = self.poll_workflow_completion(operation_id.as_str().unwrap()).await?;
                
                Ok(TestTenant {
                    id: result["tenant_id"].as_str().unwrap().to_string(),
                    name: name.to_string(),
                    admin_email: admin_email.to_string(),
                })
            } else {
                // Synchronous response
                Ok(TestTenant {
                    id: workflow_response["tenant_id"].as_str().unwrap().to_string(),
                    name: name.to_string(),
                    admin_email: admin_email.to_string(),
                })
            }
        } else {
            Err(format!("Failed to create tenant: {}", response.status()).into())
        }
    }

    async fn create_test_user(&self, email: &str, password: &str, tenant_ids: Vec<String>) -> Result<TestUser, Box<dyn std::error::Error>> {
        let response = self.http_client
            .post(&format!("{}/api/v1/workflows/user-onboarding", self.config.api_gateway_url))
            .json(&serde_json::json!({
                "email": email,
                "password": password,
                "tenant_ids": tenant_ids,
                "roles": ["user"]
            }))
            .send()
            .await?;

        if response.status().is_success() {
            let workflow_response: serde_json::Value = response.json().await?;
            
            if let Some(operation_id) = workflow_response.get("operation_id") {
                let result = self.poll_workflow_completion(operation_id.as_str().unwrap()).await?;
                
                Ok(TestUser {
                    id: result["user_id"].as_str().unwrap().to_string(),
                    email: email.to_string(),
                    tenant_ids,
                })
            } else {
                Ok(TestUser {
                    id: workflow_response["user_id"].as_str().unwrap().to_string(),
                    email: email.to_string(),
                    tenant_ids,
                })
            }
        } else {
            Err(format!("Failed to create user: {}", response.status()).into())
        }
    }

    async fn poll_workflow_completion(&self, operation_id: &str) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
        let max_attempts = 60;
        let mut attempts = 0;

        while attempts < max_attempts {
            let response = self.http_client
                .get(&format!("{}/api/v1/workflows/{}/status", self.config.api_gateway_url, operation_id))
                .send()
                .await?;

            if response.status().is_success() {
                let status: serde_json::Value = response.json().await?;
                
                match status["status"].as_str() {
                    Some("completed") => {
                        return Ok(status["result"].clone());
                    }
                    Some("failed") => {
                        return Err(format!("Workflow failed: {}", status["error"]).into());
                    }
                    _ => {
                        attempts += 1;
                        sleep(Duration::from_secs(1)).await;
                    }
                }
            } else {
                return Err(format!("Failed to get workflow status: {}", response.status()).into());
            }
        }

        Err("Workflow completion timeout".into())
    }

    /// Cleanup test environment
    pub async fn cleanup(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Cleaning up test environment...");

        // Stop all service processes
        for (service_name, mut process) in self.service_processes.drain() {
            println!("Stopping {}...", service_name);
            let _ = process.kill();
            let _ = process.wait();
        }

        // Stop Docker containers
        let _ = Command::new("docker")
            .args(&["stop", "temporal-test"])
            .output();

        Ok(())
    }
}

impl Drop for IntegrationTestEnvironment {
    fn drop(&mut self) {
        // Ensure cleanup on drop
        let _ = futures::executor::block_on(self.cleanup());
    }
}

#[derive(Debug, Clone)]
pub struct TestData {
    pub tenants: Vec<TestTenant>,
    pub users: Vec<TestUser>,
}

#[derive(Debug, Clone)]
pub struct TestTenant {
    pub id: String,
    pub name: String,
    pub admin_email: String,
}

#[derive(Debug, Clone)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub tenant_ids: Vec<String>,
}