use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::time::Duration;

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
}

impl ApiClient {
    pub async fn new() -> Result<Self> {
        let base_url = std::env::var("API_GATEWAY_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(Self { client, base_url })
    }

    pub async fn get_user(&self, user_id: &str, token: &str) -> Result<Value> {
        let url = format!("{}/api/users/{}", self.base_url, user_id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn get_user_profile(&self, user_id: &str, token: &str) -> Result<Value> {
        let url = format!("{}/api/users/{}/profile", self.base_url, user_id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn get_user_tenants(&self, user_id: &str, token: &str) -> Result<Value> {
        let url = format!("{}/api/users/{}/tenants", self.base_url, user_id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn get_user_activity(&self, user_id: &str, token: &str) -> Result<Value> {
        let url = format!("{}/api/users/{}/activity", self.base_url, user_id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        Ok(json)
    }
}