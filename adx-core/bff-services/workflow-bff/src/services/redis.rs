use anyhow::Result;
use redis::{AsyncCommands, Client};
use serde_json::Value;

#[derive(Clone)]
pub struct RedisService {
    client: Client,
}

impl RedisService {
    pub async fn new() -> Result<Self> {
        let redis_url = std::env::var("REDIS_URL")
            .unwrap_or_else(|_| "redis://localhost:6379".to_string());

        let client = Client::open(redis_url)?;
        
        Ok(Self { client })
    }

    pub async fn cache_workflow_status(&self, workflow_id: &str, status: &Value, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("workflow:{}:status", workflow_id);
        let data = serde_json::to_string(status)?;
        
        conn.set_ex(&key, data, ttl_seconds).await?;
        Ok(())
    }

    pub async fn get_cached_workflow_status(&self, workflow_id: &str) -> Result<Option<Value>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("workflow:{}:status", workflow_id);
        
        let cached: Option<String> = conn.get(&key).await?;
        
        match cached {
            Some(data) => {
                let status: Value = serde_json::from_str(&data)?;
                Ok(Some(status))
            }
            None => Ok(None),
        }
    }
}