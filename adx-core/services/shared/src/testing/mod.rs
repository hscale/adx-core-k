// Shared testing utilities for ADX CORE services
pub mod mocks;
pub mod fixtures;
pub mod temporal;
pub mod database;
pub mod assertions;

use std::sync::Arc;
use sqlx::PgPool;
use redis::Client as RedisClient;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

/// Test environment configuration
#[derive(Debug, Clone)]
pub struct TestConfig {
    pub database_url: String,
    pub redis_url: String,
    pub temporal_url: String,
    pub test_timeout_seconds: u64,
    pub cleanup_on_drop: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("TEST_DATABASE_URL")
                .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/adx_core_test".to_string()),
            redis_url: std::env::var("TEST_REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            temporal_url: std::env::var("TEST_TEMPORAL_URL")
                .unwrap_or_else(|_| "localhost:7233".to_string()),
            test_timeout_seconds: 30,
            cleanup_on_drop: true,
        }
    }
}

/// Test context for service unit tests
pub struct TestContext {
    pub config: TestConfig,
    pub database: Arc<PgPool>,
    pub redis: Arc<RedisClient>,
    pub test_id: String,
    pub cleanup_tasks: Arc<RwLock<Vec<Box<dyn Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync>>>>,
}

impl TestContext {
    /// Create a new test context with isolated database schema
    pub async fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let config = TestConfig::default();
        let test_id = Uuid::new_v4().to_string();
        
        // Create isolated database connection
        let database = Arc::new(
            PgPool::connect(&config.database_url).await?
        );
        
        // Create test schema
        let schema_name = format!("test_{}", test_id.replace('-', "_"));
        sqlx::query(&format!("CREATE SCHEMA IF NOT EXISTS {}", schema_name))
            .execute(&*database)
            .await?;
        
        // Set search path to test schema
        sqlx::query(&format!("SET search_path = {}, public", schema_name))
            .execute(&*database)
            .await?;
        
        // Run migrations in test schema
        sqlx::migrate!("../migrations")
            .run(&*database)
            .await?;
        
        // Create Redis client
        let redis = Arc::new(RedisClient::open(config.redis_url.as_str())?);
        
        let cleanup_tasks = Arc::new(RwLock::new(Vec::new()));
        
        Ok(Self {
            config,
            database,
            redis,
            test_id: test_id.clone(),
            cleanup_tasks,
        })
    }
    
    /// Add a cleanup task to be executed when the test context is dropped
    pub async fn add_cleanup_task<F>(&self, task: F)
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error + Send + Sync>> + Send + Sync + 'static,
    {
        self.cleanup_tasks.write().await.push(Box::new(task));
    }
    
    /// Create test data with automatic cleanup
    pub async fn create_test_tenant(&self) -> Result<TestTenant, Box<dyn std::error::Error + Send + Sync>> {
        let tenant = TestTenant {
            id: Uuid::new_v4().to_string(),
            name: format!("Test Tenant {}", &self.test_id[..8]),
            admin_email: format!("admin-{}@test.com", &self.test_id[..8]),
            created_at: Utc::now(),
        };
        
        // Insert into database
        sqlx::query!(
            "INSERT INTO tenants (id, name, admin_email, created_at) VALUES ($1, $2, $3, $4)",
            tenant.id,
            tenant.name,
            tenant.admin_email,
            tenant.created_at
        )
        .execute(&*self.database)
        .await?;
        
        // Add cleanup task
        let tenant_id = tenant.id.clone();
        let database = self.database.clone();
        self.add_cleanup_task(move || {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                sqlx::query!("DELETE FROM tenants WHERE id = $1", tenant_id)
                    .execute(&*database)
                    .await?;
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })?;
            Ok(())
        }).await;
        
        Ok(tenant)
    }
    
    /// Create test user with automatic cleanup
    pub async fn create_test_user(&self, tenant_id: &str) -> Result<TestUser, Box<dyn std::error::Error + Send + Sync>> {
        let user = TestUser {
            id: Uuid::new_v4().to_string(),
            email: format!("user-{}@test.com", &self.test_id[..8]),
            tenant_id: tenant_id.to_string(),
            created_at: Utc::now(),
        };
        
        // Insert into database
        sqlx::query!(
            "INSERT INTO users (id, email, tenant_id, created_at) VALUES ($1, $2, $3, $4)",
            user.id,
            user.email,
            user.tenant_id,
            user.created_at
        )
        .execute(&*self.database)
        .await?;
        
        // Add cleanup task
        let user_id = user.id.clone();
        let database = self.database.clone();
        self.add_cleanup_task(move || {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                sqlx::query!("DELETE FROM users WHERE id = $1", user_id)
                    .execute(&*database)
                    .await?;
                Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
            })?;
            Ok(())
        }).await;
        
        Ok(user)
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        if self.config.cleanup_on_drop {
            // Execute cleanup tasks
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                let tasks = self.cleanup_tasks.read().await;
                for task in tasks.iter() {
                    if let Err(e) = task() {
                        eprintln!("Cleanup task failed: {}", e);
                    }
                }
            });
            
            // Drop test schema
            rt.block_on(async {
                let schema_name = format!("test_{}", self.test_id.replace('-', "_"));
                if let Err(e) = sqlx::query(&format!("DROP SCHEMA IF EXISTS {} CASCADE", schema_name))
                    .execute(&*self.database)
                    .await
                {
                    eprintln!("Failed to drop test schema: {}", e);
                }
            });
        }
    }
}

/// Test tenant data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestTenant {
    pub id: String,
    pub name: String,
    pub admin_email: String,
    pub created_at: DateTime<Utc>,
}

/// Test user data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestUser {
    pub id: String,
    pub email: String,
    pub tenant_id: String,
    pub created_at: DateTime<Utc>,
}

/// Test assertion utilities
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that a result is Ok and return the value
    pub fn assert_ok<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(error) => panic!("Expected Ok, got Err: {:?}", error),
        }
    }
    
    /// Assert that a result is Err
    pub fn assert_err<T, E>(result: Result<T, E>)
    where
        T: std::fmt::Debug,
    {
        match result {
            Ok(value) => panic!("Expected Err, got Ok: {:?}", value),
            Err(_) => {},
        }
    }
    
    /// Assert that two values are equal with custom message
    pub fn assert_eq_with_msg<T>(left: T, right: T, message: &str)
    where
        T: std::fmt::Debug + PartialEq,
    {
        if left != right {
            panic!("{}: expected {:?}, got {:?}", message, right, left);
        }
    }
    
    /// Assert that a condition is true with custom message
    pub fn assert_with_msg(condition: bool, message: &str) {
        if !condition {
            panic!("Assertion failed: {}", message);
        }
    }
}

/// Macro for creating test cases with automatic cleanup
#[macro_export]
macro_rules! test_case {
    ($name:ident, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() {
            let ctx = TestContext::new().await.expect("Failed to create test context");
            $test_fn(ctx).await;
        }
    };
}

/// Macro for creating parameterized test cases
#[macro_export]
macro_rules! parameterized_test {
    ($name:ident, $params:expr, $test_fn:expr) => {
        #[tokio::test]
        async fn $name() {
            for (i, param) in $params.iter().enumerate() {
                let ctx = TestContext::new().await.expect("Failed to create test context");
                println!("Running test case {} with parameter: {:?}", i + 1, param);
                $test_fn(ctx, param.clone()).await;
            }
        }
    };
}