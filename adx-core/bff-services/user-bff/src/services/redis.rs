use anyhow::Result;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

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

    pub async fn get_cached_user(&self, user_id: &str) -> Result<Option<Value>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("user:{}", user_id);
        
        let cached: Option<String> = conn.get(&key).await?;
        
        match cached {
            Some(data) => {
                let user: Value = serde_json::from_str(&data)?;
                Ok(Some(user))
            }
            None => Ok(None),
        }
    }

    pub async fn cache_user(&self, user_id: &str, user_data: &Value, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("user:{}", user_id);
        let data = serde_json::to_string(user_data)?;
        
        conn.set_ex(&key, data, ttl_seconds).await?;
        Ok(())
    }

    pub async fn get_cached_user_profile(&self, user_id: &str) -> Result<Option<Value>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("user:{}:profile", user_id);
        
        let cached: Option<String> = conn.get(&key).await?;
        
        match cached {
            Some(data) => {
                let profile: Value = serde_json::from_str(&data)?;
                Ok(Some(profile))
            }
            None => Ok(None),
        }
    }

    pub async fn cache_user_profile(&self, user_id: &str, profile_data: &Value, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("user:{}:profile", user_id);
        let data = serde_json::to_string(profile_data)?;
        
        conn.set_ex(&key, data, ttl_seconds).await?;
        Ok(())
    }

    pub async fn invalidate_user_cache(&self, user_id: &str) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        
        let keys = vec![
            format!("user:{}", user_id),
            format!("user:{}:profile", user_id),
            format!("user:{}:tenants", user_id),
            format!("user:{}:activity", user_id),
        ];
        
        for key in keys {
            let _: () = conn.del(&key).await?;
        }
        
        Ok(())
    }

    pub async fn get_aggregated_dashboard(&self, user_id: &str) -> Result<Option<Value>> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("dashboard:{}", user_id);
        
        let cached: Option<String> = conn.get(&key).await?;
        
        match cached {
            Some(data) => {
                let dashboard: Value = serde_json::from_str(&data)?;
                Ok(Some(dashboard))
            }
            None => Ok(None),
        }
    }

    pub async fn cache_aggregated_dashboard(&self, user_id: &str, dashboard_data: &Value, ttl_seconds: u64) -> Result<()> {
        let mut conn = self.client.get_async_connection().await?;
        let key = format!("dashboard:{}", user_id);
        let data = serde_json::to_string(dashboard_data)?;
        
        conn.set_ex(&key, data, ttl_seconds).await?;
        Ok(())
    }
}