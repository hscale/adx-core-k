use anyhow::Result;
use redis::{aio::ConnectionManager, AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

use crate::{
    middleware::error_handler::{BffError, BffResult},
    types::{TenantContext, WorkflowExecution, WorkflowMetrics, SystemHealth},
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
            .unwrap_or_else(|_| "workflow_bff".to_string());
        
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

    // Workflow-specific cache operations
    pub async fn cache_workflow_status(&self, workflow_id: &str, status: &WorkflowExecution, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_status:{}", workflow_id);
        self.set(&key, status, ttl).await
    }

    pub async fn get_cached_workflow_status(&self, workflow_id: &str) -> BffResult<Option<WorkflowExecution>> {
        let key = format!("workflow_status:{}", workflow_id);
        self.get(&key).await
    }

    pub async fn cache_workflow_list(&self, tenant_id: &str, params_hash: &str, workflows: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_list:{}:{}", tenant_id, params_hash);
        self.set(&key, workflows, ttl).await
    }

    pub async fn get_cached_workflow_list(&self, tenant_id: &str, params_hash: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_list:{}:{}", tenant_id, params_hash);
        self.get(&key).await
    }

    pub async fn cache_workflow_history(&self, workflow_id: &str, params_hash: &str, history: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_history:{}:{}", workflow_id, params_hash);
        self.set(&key, history, ttl).await
    }

    pub async fn get_cached_workflow_history(&self, workflow_id: &str, params_hash: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_history:{}:{}", workflow_id, params_hash);
        self.get(&key).await
    }

    // Workflow template cache operations
    pub async fn cache_workflow_templates(&self, tenant_id: &str, templates: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_templates:{}", tenant_id);
        self.set(&key, templates, ttl).await
    }

    pub async fn get_cached_workflow_templates(&self, tenant_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_templates:{}", tenant_id);
        self.get(&key).await
    }

    pub async fn cache_workflow_template(&self, template_id: &str, template: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_template:{}", template_id);
        self.set(&key, template, ttl).await
    }

    pub async fn get_cached_workflow_template(&self, template_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_template:{}", template_id);
        self.get(&key).await
    }

    // Workflow schedule cache operations
    pub async fn cache_workflow_schedules(&self, tenant_id: &str, schedules: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_schedules:{}", tenant_id);
        self.set(&key, schedules, ttl).await
    }

    pub async fn get_cached_workflow_schedules(&self, tenant_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_schedules:{}", tenant_id);
        self.get(&key).await
    }

    // Monitoring and metrics cache operations
    pub async fn cache_workflow_metrics(&self, tenant_id: &str, params_hash: &str, metrics: &WorkflowMetrics, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_metrics:{}:{}", tenant_id, params_hash);
        self.set(&key, metrics, ttl).await
    }

    pub async fn get_cached_workflow_metrics(&self, tenant_id: &str, params_hash: &str) -> BffResult<Option<WorkflowMetrics>> {
        let key = format!("workflow_metrics:{}:{}", tenant_id, params_hash);
        self.get(&key).await
    }

    pub async fn cache_system_health(&self, health: &SystemHealth, ttl: Option<u64>) -> BffResult<()> {
        let key = "system_health".to_string();
        self.set(&key, health, ttl).await
    }

    pub async fn get_cached_system_health(&self) -> BffResult<Option<SystemHealth>> {
        let key = "system_health".to_string();
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

    // Real-time workflow progress cache operations
    pub async fn cache_workflow_progress(&self, workflow_id: &str, progress: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_progress:{}", workflow_id);
        self.set(&key, progress, ttl.or(Some(60))).await // Short TTL for progress updates
    }

    pub async fn get_cached_workflow_progress(&self, workflow_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_progress:{}", workflow_id);
        self.get(&key).await
    }

    // Workflow aggregation cache operations
    pub async fn cache_workflow_dashboard(&self, tenant_id: &str, dashboard_data: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_dashboard:{}", tenant_id);
        self.set(&key, dashboard_data, ttl).await
    }

    pub async fn get_cached_workflow_dashboard(&self, tenant_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_dashboard:{}", tenant_id);
        self.get(&key).await
    }

    pub async fn cache_workflow_analytics(&self, tenant_id: &str, params_hash: &str, analytics: &serde_json::Value, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_analytics:{}:{}", tenant_id, params_hash);
        self.set(&key, analytics, ttl).await
    }

    pub async fn get_cached_workflow_analytics(&self, tenant_id: &str, params_hash: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("workflow_analytics:{}:{}", tenant_id, params_hash);
        self.get(&key).await
    }

    // WebSocket session management
    pub async fn store_websocket_session(&self, session_id: &str, user_id: &str, tenant_id: &str, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("ws_session:{}", session_id);
        let session_data = serde_json::json!({
            "user_id": user_id,
            "tenant_id": tenant_id,
            "connected_at": chrono::Utc::now()
        });
        self.set(&key, &session_data, ttl.or(Some(3600))).await // 1 hour default
    }

    pub async fn get_websocket_session(&self, session_id: &str) -> BffResult<Option<serde_json::Value>> {
        let key = format!("ws_session:{}", session_id);
        self.get(&key).await
    }

    pub async fn remove_websocket_session(&self, session_id: &str) -> BffResult<()> {
        let key = format!("ws_session:{}", session_id);
        self.delete(&key).await
    }

    // Workflow subscription management for real-time updates
    pub async fn subscribe_to_workflow(&self, workflow_id: &str, session_id: &str, ttl: Option<u64>) -> BffResult<()> {
        let key = format!("workflow_subscribers:{}", workflow_id);
        
        // Add session to the set of subscribers
        match self.connection.clone().sadd::<_, _, ()>(&self.build_key(&key), session_id).await {
            Ok(_) => {
                // Set expiration on the key
                if let Some(ttl) = ttl {
                    let _ = self.connection.clone().expire::<_, ()>(&self.build_key(&key), ttl as usize).await;
                }
                debug!("Added subscriber {} to workflow {}", session_id, workflow_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to add workflow subscriber: {}", e);
                Err(BffError::redis(format!("Failed to add subscriber: {}", e)))
            }
        }
    }

    pub async fn unsubscribe_from_workflow(&self, workflow_id: &str, session_id: &str) -> BffResult<()> {
        let key = format!("workflow_subscribers:{}", workflow_id);
        
        match self.connection.clone().srem::<_, _, ()>(&self.build_key(&key), session_id).await {
            Ok(_) => {
                debug!("Removed subscriber {} from workflow {}", session_id, workflow_id);
                Ok(())
            }
            Err(e) => {
                error!("Failed to remove workflow subscriber: {}", e);
                Err(BffError::redis(format!("Failed to remove subscriber: {}", e)))
            }
        }
    }

    pub async fn get_workflow_subscribers(&self, workflow_id: &str) -> BffResult<Vec<String>> {
        let key = format!("workflow_subscribers:{}", workflow_id);
        
        match self.connection.clone().smembers::<_, Vec<String>>(&self.build_key(&key)).await {
            Ok(subscribers) => Ok(subscribers),
            Err(e) => {
                error!("Failed to get workflow subscribers: {}", e);
                Err(BffError::redis(format!("Failed to get subscribers: {}", e)))
            }
        }
    }

    // Utility methods
    fn build_key(&self, key: &str) -> String {
        format!("{}:{}", self.key_prefix, key)
    }

    pub async fn health_check(&self) -> BffResult<()> {
        match self.connection.clone().ping::<()>().await {
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

    // Invalidate workflow-related caches
    pub async fn invalidate_workflow_caches(&self, workflow_id: &str) -> BffResult<()> {
        let patterns = vec![
            format!("workflow_status:{}", workflow_id),
            format!("workflow_progress:{}", workflow_id),
            format!("workflow_history:{}:*", workflow_id),
        ];

        for pattern in patterns {
            let _ = self.delete_pattern(&pattern).await; // Don't fail if pattern doesn't exist
        }

        debug!("Invalidated workflow caches for workflow: {}", workflow_id);
        Ok(())
    }

    // Invalidate tenant workflow caches
    pub async fn invalidate_tenant_workflow_caches(&self, tenant_id: &str) -> BffResult<()> {
        let patterns = vec![
            format!("workflow_list:{}:*", tenant_id),
            format!("workflow_dashboard:{}", tenant_id),
            format!("workflow_analytics:{}:*", tenant_id),
            format!("workflow_metrics:{}:*", tenant_id),
        ];

        for pattern in patterns {
            let _ = self.delete_pattern(&pattern).await; // Don't fail if pattern doesn't exist
        }

        debug!("Invalidated tenant workflow caches for tenant: {}", tenant_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_workflow_cache_operations() {
        // This test requires a running Redis instance
        if std::env::var("REDIS_URL").is_err() {
            return; // Skip test if Redis is not available
        }

        let redis = RedisService::new().await.unwrap();
        
        let workflow_execution = WorkflowExecution {
            workflow_id: "test-workflow-123".to_string(),
            run_id: "run-456".to_string(),
            workflow_type: "test_workflow".to_string(),
            status: crate::types::WorkflowStatus::Running,
            start_time: chrono::Utc::now(),
            end_time: None,
            execution_time_ms: None,
            input: json!({"test": "data"}),
            result: None,
            error: None,
            progress: None,
            metadata: std::collections::HashMap::new(),
            parent_workflow_id: None,
            child_workflows: vec![],
        };

        // Test workflow status caching
        redis.cache_workflow_status("test-workflow-123", &workflow_execution, Some(60)).await.unwrap();
        let cached_status = redis.get_cached_workflow_status("test-workflow-123").await.unwrap();
        
        assert!(cached_status.is_some());
        let cached = cached_status.unwrap();
        assert_eq!(cached.workflow_id, "test-workflow-123");
        assert_eq!(cached.workflow_type, "test_workflow");

        // Test workflow subscription
        redis.subscribe_to_workflow("test-workflow-123", "session-123", Some(300)).await.unwrap();
        let subscribers = redis.get_workflow_subscribers("test-workflow-123").await.unwrap();
        assert!(subscribers.contains(&"session-123".to_string()));

        // Test cleanup
        redis.unsubscribe_from_workflow("test-workflow-123", "session-123").await.unwrap();
        redis.invalidate_workflow_caches("test-workflow-123").await.unwrap();
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