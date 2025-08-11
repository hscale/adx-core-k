use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use crate::{Error, Result, TenantId, TenantIsolationLevel};

pub mod seeder;

pub type DatabasePool = Arc<PgPool>;

// Database connection pool wrapper
pub struct AdxDatabasePool {
    pool: DatabasePool,
}

impl AdxDatabasePool {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool: Arc::new(pool),
        }
    }

    pub fn pool(&self) -> &DatabasePool {
        &self.pool
    }

    #[cfg(test)]
    pub fn new_mock() -> Self {
        // Create a mock pool for testing
        // This is a placeholder - in real tests you'd use sqlx::testing
        use std::sync::Arc;
        use std::mem;
        
        // Create a mock Arc for testing - this should only be used in unit tests
        // where the database operations are mocked at the repository level
        unsafe {
            mem::transmute(Arc::new(()))
        }
    }
}

#[async_trait]
pub trait Repository<T> {
    async fn create(&self, entity: T) -> Result<T>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>>;
    async fn update(&self, entity: T) -> Result<T>;
    async fn delete(&self, id: &str) -> Result<()>;
    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<T>>;
}

pub struct TenantAwareDatabase {
    pool: DatabasePool,
    tenant_id: TenantId,
    isolation_level: TenantIsolationLevel,
}

impl TenantAwareDatabase {
    pub fn new(pool: DatabasePool, tenant_id: TenantId, isolation_level: TenantIsolationLevel) -> Self {
        Self {
            pool,
            tenant_id,
            isolation_level,
        }
    }
    
    pub async fn execute_query(&self, query: &str) -> Result<Vec<sqlx::postgres::PgRow>> {
        match self.isolation_level {
            TenantIsolationLevel::Schema => {
                let schema_query = format!("SET search_path = tenant_{};", self.tenant_id);
                sqlx::query(&schema_query).execute(&*self.pool).await?;
                let rows = sqlx::query(query).fetch_all(&*self.pool).await?;
                Ok(rows)
            }
            TenantIsolationLevel::Database => {
                // Use tenant-specific connection pool (would be implemented in connection manager)
                let rows = sqlx::query(query).fetch_all(&*self.pool).await?;
                Ok(rows)
            }
            TenantIsolationLevel::Row => {
                // Automatically inject tenant_id filter
                let tenant_query = self.inject_tenant_filter(query)?;
                let rows = sqlx::query(&tenant_query).fetch_all(&*self.pool).await?;
                Ok(rows)
            }
        }
    }
    
    fn inject_tenant_filter(&self, query: &str) -> Result<String> {
        // Simple implementation - in production, use a proper SQL parser
        if query.to_lowercase().contains("where") {
            Ok(format!("{} AND tenant_id = '{}'", query, self.tenant_id))
        } else if query.to_lowercase().contains("from") {
            let parts: Vec<&str> = query.split("FROM").collect();
            if parts.len() == 2 {
                Ok(format!("{}FROM{} WHERE tenant_id = '{}'", parts[0], parts[1], self.tenant_id))
            } else {
                Err(Error::Internal("Failed to inject tenant filter".to_string()))
            }
        } else {
            Err(Error::Internal("Unsupported query format for tenant filtering".to_string()))
        }
    }
}

pub async fn create_connection_pool(config: &crate::config::DatabaseConfig) -> Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_seconds))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_seconds))
        .max_lifetime(std::time::Duration::from_secs(config.max_lifetime_seconds))
        .connect(&config.url)
        .await
        .map_err(|e| Error::Internal(format!("Failed to create database pool: {}", e)))?;
    
    Ok(pool)
}

pub async fn create_database_pool(database_url: &str, max_connections: u32) -> Result<DatabasePool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(1)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .idle_timeout(std::time::Duration::from_secs(600)) // 10 minutes
        .max_lifetime(std::time::Duration::from_secs(1800)) // 30 minutes
        .connect(database_url)
        .await
        .map_err(|e| Error::Internal(format!("Failed to create database pool: {}", e)))?;
    
    Ok(Arc::new(pool))
}

/// Create a database pool with custom configuration
pub async fn create_database_pool_with_config(config: DatabaseConfig) -> Result<DatabasePool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.max_connections)
        .min_connections(config.min_connections)
        .acquire_timeout(std::time::Duration::from_secs(config.acquire_timeout_seconds))
        .idle_timeout(std::time::Duration::from_secs(config.idle_timeout_seconds))
        .max_lifetime(std::time::Duration::from_secs(config.max_lifetime_seconds))
        .test_before_acquire(config.test_before_acquire)
        .connect(&config.database_url)
        .await
        .map_err(|e| Error::Internal(format!("Failed to create database pool: {}", e)))?;
    
    Ok(Arc::new(pool))
}

/// Database configuration structure
#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub database_url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
    pub test_before_acquire: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/adx_core".to_string()),
            max_connections: 20,
            min_connections: 1,
            acquire_timeout_seconds: 30,
            idle_timeout_seconds: 600,  // 10 minutes
            max_lifetime_seconds: 1800, // 30 minutes
            test_before_acquire: true,
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            database_url: std::env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/adx_core".to_string()),
            max_connections: std::env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            min_connections: std::env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "1".to_string())
                .parse()
                .unwrap_or(1),
            acquire_timeout_seconds: std::env::var("DB_ACQUIRE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .unwrap_or(30),
            idle_timeout_seconds: std::env::var("DB_IDLE_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "600".to_string())
                .parse()
                .unwrap_or(600),
            max_lifetime_seconds: std::env::var("DB_MAX_LIFETIME_SECONDS")
                .unwrap_or_else(|_| "1800".to_string())
                .parse()
                .unwrap_or(1800),
            test_before_acquire: std::env::var("DB_TEST_BEFORE_ACQUIRE")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| Error::Internal(format!("Migration failed: {}", e)))?;
    
    Ok(())
}

// Health check for database
pub async fn check_database_health(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await
        .map_err(|e| Error::Internal(format!("Database health check failed: {}", e)))?;
    
    Ok(())
}

/// Comprehensive database health check with detailed information
pub async fn comprehensive_health_check(pool: &PgPool) -> Result<DatabaseHealthStatus> {
    let start_time = std::time::Instant::now();
    
    // Basic connectivity test
    let connectivity_result = sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await;
    
    let response_time = start_time.elapsed().as_millis() as u64;
    
    if connectivity_result.is_err() {
        return Ok(DatabaseHealthStatus {
            is_healthy: false,
            response_time_ms: response_time,
            connection_count: 0,
            active_tenants: 0,
            active_sessions: 0,
            error_message: Some(connectivity_result.unwrap_err().to_string()),
        });
    }
    
    // Get connection count
    let connection_count: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM pg_stat_activity WHERE state = 'active'"
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    
    // Get active tenants count
    let active_tenants: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM tenants WHERE is_active = true"
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    
    // Get active sessions count
    let active_sessions: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM user_sessions WHERE status = 'active' AND expires_at > NOW()"
    )
    .fetch_one(pool)
    .await
    .unwrap_or((0,));
    
    Ok(DatabaseHealthStatus {
        is_healthy: true,
        response_time_ms: response_time,
        connection_count: connection_count.0,
        active_tenants: active_tenants.0,
        active_sessions: active_sessions.0,
        error_message: None,
    })
}

/// Database health status information
#[derive(Debug, Clone)]
pub struct DatabaseHealthStatus {
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub connection_count: i64,
    pub active_tenants: i64,
    pub active_sessions: i64,
    pub error_message: Option<String>,
}

impl std::fmt::Display for DatabaseHealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_healthy {
            write!(
                f,
                "Database Status: HEALTHY\n  Response Time: {}ms\n  Active Connections: {}\n  Active Tenants: {}\n  Active Sessions: {}",
                self.response_time_ms, self.connection_count, self.active_tenants, self.active_sessions
            )
        } else {
            write!(
                f,
                "Database Status: UNHEALTHY\n  Error: {}",
                self.error_message.as_deref().unwrap_or("Unknown error")
            )
        }
    }
}