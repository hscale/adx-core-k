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

    pub async fn get_workflow(&self, workflow_id: &str, token: &str) -> Result<Value> {
        let url = format!("{}/api/workflows/{}", self.base_url, workflow_id);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        Ok(json)
    }

    pub async fn start_workflow(&self, workflow_type: &str, input: &Value, token: &str) -> Result<Value> {
        let url = format!("{}/api/workflows/start", self.base_url);
        
        let payload = serde_json::json!({
            "workflow_type": workflow_type,
            "input": input
        });
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&payload)
            .send()
            .await?;

        let json = response.json::<Value>().await?;
        Ok(json)
    }
}