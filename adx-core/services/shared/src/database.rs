use async_trait::async_trait;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use crate::{Error, Result, TenantId, TenantIsolationLevel};

pub type DatabasePool = Arc<PgPool>;

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
    
    pub async fn execute_query(&self, query: &str, params: &[&(dyn sqlx::Encode<sqlx::Postgres> + Sync)]) -> Result<Vec<sqlx::postgres::PgRow>> {
        match self.isolation_level {
            TenantIsolationLevel::Schema => {
                let schema_query = format!("SET search_path = tenant_{};", self.tenant_id);
                sqlx::query(&schema_query).execute(&*self.pool).await?;
                let rows = sqlx::query(query).bind_all(params).fetch_all(&*self.pool).await?;
                Ok(rows)
            }
            TenantIsolationLevel::Database => {
                // Use tenant-specific connection pool (would be implemented in connection manager)
                let rows = sqlx::query(query).bind_all(params).fetch_all(&*self.pool).await?;
                Ok(rows)
            }
            TenantIsolationLevel::Row => {
                // Automatically inject tenant_id filter
                let tenant_query = self.inject_tenant_filter(query)?;
                let rows = sqlx::query(&tenant_query).bind_all(params).fetch_all(&*self.pool).await?;
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

pub async fn create_database_pool(database_url: &str, max_connections: u32) -> Result<DatabasePool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(max_connections)
        .connect(database_url)
        .await?;
    
    Ok(Arc::new(pool))
}

pub async fn run_migrations(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("./migrations")
        .run(pool)
        .await
        .map_err(|e| Error::Database(e))?;
    
    Ok(())
}

// Health check for database
pub async fn check_database_health(pool: &PgPool) -> Result<()> {
    sqlx::query("SELECT 1")
        .fetch_one(pool)
        .await?;
    
    Ok(())
}