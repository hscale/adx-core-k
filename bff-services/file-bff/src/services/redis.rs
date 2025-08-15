use anyhow::{Context, Result};
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

#[derive(Clone)]
pub struct RedisService {
    connection: ConnectionManager,
}

impl RedisService {
    pub async fn new() -> Result<Self> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let client = Client::open(redis_url)
            .context("Failed to create Redis client")?;

        let connection = ConnectionManager::new(client)
            .await
            .context("Failed to create Redis connection manager")?;

        Ok(Self { connection })
    }

    // Generic cache operations
    pub async fn get<T>(&self, key: &str) -> Result<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let mut conn = self.connection.clone();
        
        debug!("Getting cache key: {}", key);
        
        let result: Option<String> = conn
            .get(key)
            .await
            .context("Failed to get value from Redis")?;

        match result {
            Some(json_str) => {
                let value = serde_json::from_str(&json_str)
                    .context("Failed to deserialize cached value")?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl_seconds: Option<u64>) -> Result<()>
    where
        T: Serialize,
    {
        let mut conn = self.connection.clone();
        
        debug!("Setting cache key: {} with TTL: {:?}", key, ttl_seconds);
        
        let json_str = serde_json::to_string(value)
            .context("Failed to serialize value")?;

        if let Some(ttl) = ttl_seconds {
            conn.set_ex(key, json_str, ttl)
                .await
                .context("Failed to set value in Redis with TTL")?;
        } else {
            conn.set(key, json_str)
                .await
                .context("Failed to set value in Redis")?;
        }

        Ok(())
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        
        debug!("Deleting cache key: {}", key);
        
        conn.del(key)
            .await
            .context("Failed to delete key from Redis")?;

        Ok(())
    }

    pub async fn exists(&self, key: &str) -> Result<bool> {
        let mut conn = self.connection.clone();
        
        let exists: bool = conn
            .exists(key)
            .await
            .context("Failed to check key existence in Redis")?;

        Ok(exists)
    }

    // File-specific cache operations
    pub async fn cache_file_metadata(
        &self,
        file_id: &str,
        tenant_id: &str,
        metadata: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let key = format!("file:metadata:{}:{}", tenant_id, file_id);
        self.set(&key, metadata, ttl_seconds).await
    }

    pub async fn get_cached_file_metadata(
        &self,
        file_id: &str,
        tenant_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let key = format!("file:metadata:{}:{}", tenant_id, file_id);
        self.get(&key).await
    }

    pub async fn cache_file_permissions(
        &self,
        file_id: &str,
        tenant_id: &str,
        permissions: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let key = format!("file:permissions:{}:{}", tenant_id, file_id);
        self.set(&key, permissions, ttl_seconds).await
    }

    pub async fn get_cached_file_permissions(
        &self,
        file_id: &str,
        tenant_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let key = format!("file:permissions:{}:{}", tenant_id, file_id);
        self.get(&key).await
    }

    pub async fn cache_storage_info(
        &self,
        file_id: &str,
        tenant_id: &str,
        storage_info: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let key = format!("file:storage:{}:{}", tenant_id, file_id);
        self.set(&key, storage_info, ttl_seconds).await
    }

    pub async fn get_cached_storage_info(
        &self,
        file_id: &str,
        tenant_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let key = format!("file:storage:{}:{}", tenant_id, file_id);
        self.get(&key).await
    }

    // Search result caching
    pub async fn cache_search_results(
        &self,
        search_hash: &str,
        tenant_id: &str,
        results: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let key = format!("file:search:{}:{}", tenant_id, search_hash);
        self.set(&key, results, ttl_seconds).await
    }

    pub async fn get_cached_search_results(
        &self,
        search_hash: &str,
        tenant_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let key = format!("file:search:{}:{}", tenant_id, search_hash);
        self.get(&key).await
    }

    // Upload progress tracking
    pub async fn set_upload_progress(
        &self,
        upload_id: &str,
        tenant_id: &str,
        progress: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let key = format!("upload:progress:{}:{}", tenant_id, upload_id);
        self.set(&key, progress, ttl_seconds.or(Some(3600))).await // Default 1 hour TTL
    }

    pub async fn get_upload_progress(
        &self,
        upload_id: &str,
        tenant_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let key = format!("upload:progress:{}:{}", tenant_id, upload_id);
        self.get(&key).await
    }

    // Workflow status caching
    pub async fn cache_workflow_status(
        &self,
        operation_id: &str,
        tenant_id: &str,
        status: &serde_json::Value,
        ttl_seconds: Option<u64>,
    ) -> Result<()> {
        let key = format!("workflow:status:{}:{}", tenant_id, operation_id);
        self.set(&key, status, ttl_seconds.or(Some(300))).await // Default 5 minutes TTL
    }

    pub async fn get_cached_workflow_status(
        &self,
        operation_id: &str,
        tenant_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let key = format!("workflow:status:{}:{}", tenant_id, operation_id);
        self.get(&key).await
    }

    // Batch operations
    pub async fn invalidate_file_cache(&self, file_id: &str, tenant_id: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        
        let pattern = format!("file:*:{}:{}", tenant_id, file_id);
        debug!("Invalidating file cache with pattern: {}", pattern);
        
        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .context("Failed to get keys for cache invalidation")?;

        if !keys.is_empty() {
            conn.del(&keys)
                .await
                .context("Failed to delete cache keys")?;
        }

        Ok(())
    }

    pub async fn invalidate_tenant_cache(&self, tenant_id: &str) -> Result<()> {
        let mut conn = self.connection.clone();
        
        let pattern = format!("*:{}:*", tenant_id);
        debug!("Invalidating tenant cache with pattern: {}", pattern);
        
        let keys: Vec<String> = conn
            .keys(&pattern)
            .await
            .context("Failed to get keys for tenant cache invalidation")?;

        if !keys.is_empty() {
            conn.del(&keys)
                .await
                .context("Failed to delete tenant cache keys")?;
        }

        Ok(())
    }

    // Health check
    pub async fn health_check(&self) -> Result<()> {
        let mut conn = self.connection.clone();
        
        // Use a simple get operation to test connectivity
        let _: Option<String> = conn
            .get("__health_check__")
            .await
            .context("Redis health check failed")?;

        Ok(())
    }
}

// Utility function to generate cache keys
pub fn generate_search_hash(search_params: &serde_json::Value) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let search_str = serde_json::to_string(search_params).unwrap_or_default();
    let mut hasher = DefaultHasher::new();
    search_str.hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_cache_operations() {
        // This test requires a running Redis instance
        if std::env::var("REDIS_URL").is_err() {
            return; // Skip test if Redis is not available
        }

        let redis = RedisService::new().await.unwrap();
        
        let test_data = json!({
            "id": "test-file-id",
            "name": "test.txt",
            "size": 1024
        });

        // Test set and get
        redis.set("test:key", &test_data, Some(60)).await.unwrap();
        let retrieved: Option<serde_json::Value> = redis.get("test:key").await.unwrap();
        
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap(), test_data);

        // Test delete
        redis.delete("test:key").await.unwrap();
        let deleted: Option<serde_json::Value> = redis.get("test:key").await.unwrap();
        assert!(deleted.is_none());
    }

    #[test]
    fn test_generate_search_hash() {
        let search_params = json!({
            "query": "test",
            "file_types": ["pdf", "txt"]
        });

        let hash1 = generate_search_hash(&search_params);
        let hash2 = generate_search_hash(&search_params);
        
        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }
}