// Database utilities and abstractions

use sqlx::{PgPool, Row};
use crate::{Result, ServiceError};

pub struct DatabaseManager {
    pool: PgPool,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = PgPool::connect(database_url).await?;
        Ok(Self { pool })
    }
    
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }
    
    pub async fn health_check(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await?;
        Ok(())
    }
    
    pub async fn get_version(&self) -> Result<String> {
        let row = sqlx::query("SELECT version()")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(row.get::<String, _>(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    async fn get_test_db_manager() -> DatabaseManager {
        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/adx_core_test".to_string());
        
        DatabaseManager::new(&database_url).await.unwrap()
    }

    #[tokio::test]
    async fn test_database_connection() {
        // Skip if no database available
        if env::var("SKIP_DB_TESTS").is_ok() {
            return;
        }
        
        let db = get_test_db_manager().await;
        assert!(db.health_check().await.is_ok());
    }

    #[tokio::test]
    async fn test_database_version() {
        // Skip if no database available
        if env::var("SKIP_DB_TESTS").is_ok() {
            return;
        }
        
        let db = get_test_db_manager().await;
        let version = db.get_version().await.unwrap();
        assert!(version.contains("PostgreSQL"));
    }
}