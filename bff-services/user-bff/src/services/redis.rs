use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

use crate::{
    middleware::error_handler::{BffError, BffResult},
    types::{TenantContext, User, UserPreferences, UserProfile},
};

#[derive(Clone)]
pub struct RedisService {
    connection: ConnectionManager,
    key_prefix: String,
    default_ttl: u64,
}

impl RedisService {
    pub async fn new() -> Result<Self> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());
        
        let key_prefix = std::env::var("REDIS_KEY_PREFIX")
            .unwrap_or_else(|_| "user_bff".to_string());
        
        let default_ttl = std::env::var("REDIS_DEFAULT_TTL")
            .unwrap_or_else(|_| "300".to_string())
            .parse()
            .unwrap_or(300);

        let client = Client::open(redis_url)?;
        let connection = ConnectionManager::new(client).await?;

        Ok(Self {
            connection,
            key_prefix,
            default_ttl,
        })
    }

    // Generic cache operations
    pub async fn get<T>(&self, key: &str) -> BffResult<Option<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let full_key = self.build_key(key);
        
        match self.connection.clone().get::<_, Option<String>>(&full_key).await {
            Ok(Some(value)) => {
                match serde_json::from_str::<T>(&value) {
                    Ok(data) => {
                        debug!("Cache hit for key: {}", full_key);
                        Ok(Some(data))
                    }
                    Err(e) => {
                        warn!("Failed to deserialize cached value for key {}: {}", full_key, e);
                        // Remove corrupted cache entry
                        let _ = self.delete(key).await;
                        Ok(None)
                    }
                }
            }
            Ok(None) => {
                debug!("Cache miss for key: {}", full_key);
                Ok(None)
            }
            Err(e) => {
                error!("Redis get error for key {}: {}", full_key, e);
                Err(BffError::redis(format!("Failed to get from cache: {}", e)))
            }
        }
    }

    pub async fn set<T>(&self, key: &str, value: &T, ttl: Option<u64>) -> BffResult<()>
    where
        T: Serialize,
    {
        let full_key = self.build_key(key);
        let serialized = serde_json::to_string(value)
            .map_err(|e| BffError::redis(format!("Failed to serialize value: {}", e)))?;
        
        let ttl_seconds = ttl.unwrap_or(self.default_ttl);
        
        match self.connection.clone().set_ex::<_, _, ()>(&full_key, serialized, ttl_seconds).await {
            Ok(_) => {
                debug!("Cached value for key: {} (TTL: {}s)", full_key, ttl_seconds);
                Ok(())
            }
            Err(e) => {
                error!("Redis set error for key {}: {}", full_key, e);
                Err(BffError::redis(format!("Failed to set cache: {}", e)))
            }
        }
    }

    pub async fn delete(&self, key: &str) -> BffResult<()> {
        let full_key = self.build_key(key);
        
        match self.connection.clone().del::<_, ()>(&full_key).await {
            Ok(_) => {
                debug!("Deleted cache key: {}", full_key);
                Ok(())
            }
            Err(e) => {
                error!("Redis delete error for key {}: {}", full_key, e);
                Err(BffError::redis(format!("Failed to delete from cache: {}", e)))
            }
        }
    }

    pub async fn exists(&self, key: &str) -> BffResult<bool> {
        let full_key = self.build_key(key);
        
        match self.connection.clone().exists::<_, bool>(&full_key).await {
            Ok(exists) => Ok(exists),
            Err(e) => {
                error!("Redis exists error for key {}: {}", full_key, e);
                Err(BffError::redis(format!("Failed to check cache existence: {}", e)))
            }
        }
    }

    pub async fn expire(&self, key: &str, ttl: u64) -> BffResult<()> {
        let full_key = self.build_key(key);
        
        match self.connection.clone().expire::<_, ()>(&full_key, ttl as i64).await {
            Ok(_) => {
                debug!("Set expiration for key: {} (TTL: {}s)", full_key, ttl);
                Ok(())
            }
            Err(e) => {
                error!("Redis expire error for key {}: {}", full_key, e);
                Err(BffError::redis(format!("Failed to set expiration: {}", e)))
            }
        }
    }

    // User-specific cache operations
    pub async fn cache_user(&self, user_id: &str, user: &User, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("user:{}", user_id);
        self.set(&key, user, ttl).await
    }

    pub async fn get_cached_user(&self, user_id: &str) -> BffResult<Option<User>> {
        let key = format!("user:{}", user_id);
        self.get(&key).await
    }

    pub async fn invalidate_user_cache(&self, user_id: &str) -> BffResult<()> {
        let keys = vec![
            format!("user:{}", user_id),
            format!("user_profile:{}", user_id),
            format!("user_preferences:{}", user_id),
            format!("user_activity:{}", user_id),
            format!("user_sessions:{}", user_id),
        ];

        for key in keys {
            let _ = self.delete(&key).await; // Don't fail if key doesn't exist
        }

        debug!("Invalidated user cache for user: {}", user_id);
        Ok(())
    }

    // User profile cache operations
    pub async fn cache_user_profile(&self, user_id: &str, profile: &UserProfile, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("user_profile:{}", user_id);
        self.set(&key, profile, ttl).await
    }

    pub async fn get_cached_user_profile(&self, user_id: &str) -> BffResult<Option<UserProfile>> {
        let key = format!("user_profile:{}", user_id);
        self.get(&key).await
    }

    // User preferences cache operations
    pub async fn cache_user_preferences(&self, user_id: &str, preferences: &UserPreferences, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("user_preferences:{}", user_id);
        self.set(&key, preferences, ttl).await
    }

    pub async fn get_cached_user_preferences(&self, user_id: &str) -> BffResult<Option<UserPreferences>> {
        let key = format!("user_preferences:{}", user_id);
        self.get(&key).await
    }

    // Tenant context cache operations
    pub async fn cache_tenant_context(&self, tenant_id: &str, context: &TenantContext, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("tenant_context:{}", tenant_id);
        self.set(&key, context, ttl).await
    }

    pub async fn get_cached_tenant_context(&self, tenant_id: &str) -> BffResult<Option<TenantContext>> {
        let key = format!("tenant_context:{}", tenant_id);
        self.get(&key).await
    }

    // User list cache operations
    pub async fn cache_user_list(&self, tenant_id: &str, params_hash: &str, users: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("user_list:{}:{}", tenant_id, params_hash);
        self.set(&key, users, ttl).await
    }

    pub async fn get_cached_user_list(&self, tenant_id: &str, params_hash: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("user_list:{}:{}", tenant_id, params_hash);
        self.get(&key).await
    }

    // User activity cache operations
    pub async fn cache_user_activity(&self, user_id: &str, params_hash: &str, activity: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("user_activity:{}:{}", user_id, params_hash);
        self.set(&key, activity, ttl).await
    }

    pub async fn get_cached_user_activity(&self, user_id: &str, params_hash: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("user_activity:{}:{}", user_id, params_hash);
        self.get(&key).await
    }

    // User sessions cache operations
    pub async fn cache_user_sessions(&self, user_id: &str, sessions: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("user_sessions:{}", user_id);
        self.set(&key, sessions, ttl).await
    }

    pub async fn get_cached_user_sessions(&self, user_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("user_sessions:{}", user_id);
        self.get(&key).await
    }

    // Workflow status cache operations
    pub async fn cache_workflow_status(&self, workflow_id: &str, status: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_status:{}", workflow_id);
        self.set(&key, status, ttl).await
    }

    pub async fn get_cached_workflow_status(&self, workflow_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_status:{}", workflow_id);
        self.get(&key).await
    }

    // Aggregated data cache operations
    pub async fn cache_user_dashboard(&self, user_id: &str, dashboard_data: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("user_dashboard:{}", user_id);
        self.set(&key, dashboard_data, ttl).await
    }

    pub async fn get_cached_user_dashboard(&self, user_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("user_dashboard:{}", user_id);
        self.get(&key).await
    }

    // Utility methods
    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    pub async fn health_check(&self) -> BffResult<()> {
        match redis::cmd("PING").query_async::<_, String>(&mut self.connection.clone()).await {
            Ok(_) => Ok(()),
            Err(e) => Err(BffError::redis(format!("Redis health check failed: {}", e))),
        }
    }

    // Batch operations
    pub async fn delete_pattern(&self, pattern: &str) -> BffResult<u64> {
        let full_pattern = self.build_key(pattern);
        
        // Get all keys matching the pattern
        let keys: Vec<String> = match self.connection.clone().keys(&full_pattern).await {
            Ok(keys) => keys,
            Err(e) => {
                error!("Redis keys error for pattern {}: {}", full_pattern, e);
                return Err(BffError::redis(format!("Failed to get keys: {}", e)));
            }
        };

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all matching keys
        match self.connection.clone().del::<_, u64>(&keys).await {
            Ok(count) => {
                debug!("Deleted {} keys matching pattern: {}", count, full_pattern);
                Ok(count)
            }
            Err(e) => {
                error!("Redis delete pattern error for {}: {}", full_pattern, e);
                Err(BffError::redis(format!("Failed to delete pattern: {}", e)))
            }
        }
    }
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
            "id": "test123",
            "name": "Test User"
        });

        // Test set and get
        redis.set("test_key", &test_data, Some(60)).await.unwrap();
        let cached_data: Option<serde_json::Value> = redis.get("test_key").await.unwrap();
        
        assert!(cached_data.is_some());
        assert_eq!(cached_data.unwrap(), test_data);

        // Test exists
        let exists = redis.exists("test_key").await.unwrap();
        assert!(exists);

        // Test delete
        redis.delete("test_key").await.unwrap();
        let deleted_data: Option<serde_json::Value> = redis.get("test_key").await.unwrap();
        assert!(deleted_data.is_none());
    }

    #[tokio::test]
    async fn test_health_check() {
        if std::env::var("REDIS_URL").is_err() {
            return; // Skip test if Redis is not available
        }

        let redis = RedisService::new().await.unwrap();
        let result = redis.health_check().await;
        assert!(result.is_ok());
    }
}