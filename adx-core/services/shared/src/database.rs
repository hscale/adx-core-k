use crate::types::TenantId;
use async_trait::async_trait;
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct DatabaseConfig {
    pub primary_url: String,
    pub replica_urls: Vec<String>,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout: u64,
}

#[derive(Debug)]
pub struct DatabaseManager {
    primary_pool: Arc<sqlx::PgPool>,
    replica_pools: Vec<Arc<sqlx::PgPool>>,
    #[allow(dead_code)]
    config: DatabaseConfig,
}

impl DatabaseManager {
    pub async fn new() -> Result<Self, sqlx::Error> {
        let config = DatabaseConfig {
            primary_url: "postgresql://adx_user:dev_password@localhost:5432/adx_core".to_string(),
            replica_urls: vec![],
            max_connections: 20,
            min_connections: 2,
            acquire_timeout: 30,
        };
        Self::with_config(config).await
    }

    pub async fn with_config(config: DatabaseConfig) -> Result<Self, sqlx::Error> {
        let primary_pool = Arc::new(PgPool::connect(&config.primary_url).await?);

        let mut replica_pools = Vec::new();
        for replica_url in &config.replica_urls {
            let pool = Arc::new(PgPool::connect(replica_url).await?);
            replica_pools.push(pool);
        }

        Ok(Self {
            primary_pool,
            replica_pools,
            config,
        })
    }

    pub fn primary(&self) -> &PgPool {
        &self.primary_pool
    }

    pub fn replica(&self) -> &PgPool {
        if self.replica_pools.is_empty() {
            &self.primary_pool
        } else {
            &self.replica_pools[0]
        }
    }

    pub async fn health_check(&self) -> Result<(), sqlx::Error> {
        sqlx::query("SELECT 1").execute(&*self.primary_pool).await?;
        Ok(())
    }
}

#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn create(&self, tenant_id: TenantId, entity: &T) -> Result<T, sqlx::Error>;
    async fn get_by_id(&self, tenant_id: TenantId, id: Uuid) -> Result<Option<T>, sqlx::Error>;
    async fn list(
        &self,
        tenant_id: TenantId,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<T>, sqlx::Error>;
    async fn update(&self, tenant_id: TenantId, id: Uuid, entity: &T) -> Result<T, sqlx::Error>;
    async fn delete(&self, tenant_id: TenantId, id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct TenantQuery {
    tenant_id: TenantId,
    base_query: String,
}

impl TenantQuery {
    pub fn new(tenant_id: TenantId, base_query: &str) -> Self {
        Self {
            tenant_id,
            base_query: base_query.to_string(),
        }
    }

    pub fn build(&self) -> String {
        if self.base_query.contains("WHERE") {
            format!("{} AND tenant_id = '{}'", self.base_query, self.tenant_id)
        } else {
            format!("{} WHERE tenant_id = '{}'", self.base_query, self.tenant_id)
        }
    }
}
