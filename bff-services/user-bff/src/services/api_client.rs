use anyhow::Result;
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, warn};

use crate::middleware::error_handler::{BffError, BffResult};

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    base_url: String,
    timeout_seconds: u64,
}

impl ApiClient {
    pub async fn new() -> Result<Self> {
        let base_url = std::env::var("API_GATEWAY_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        
        let timeout_seconds = std::env::var("API_TIMEOUT_SECONDS")
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30);

        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(timeout_seconds))
            .build()?;

        Ok(Self {
            client,
            base_url,
            timeout_seconds,
        })
    }

    // User management endpoints
    pub async fn get_user(&self, user_id: &str, tenant_id: &str, auth_token: &str) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users/{}", self.base_url, user_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn list_users(
        &self, 
        tenant_id: &str, 
        auth_token: &str,
        params: Option<HashMap<String, String>>
    ) -> BffResult<serde_json::Value> {
        let mut url = format!("{}/api/v1/users", self.base_url);
        
        if let Some(params) = params {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            
            if !query_string.is_empty() {
                url.push_str(&format!("?{}", query_string));
            }
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn create_user(
        &self,
        tenant_id: &str,
        auth_token: &str,
        user_data: &serde_json::Value,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(user_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn update_user(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
        user_data: &serde_json::Value,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users/{}", self.base_url, user_id);
        
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(user_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn delete_user(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<()> {
        let url = format!("{}/api/v1/users/{}", self.base_url, user_id);
        
        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Delete user failed: {} - {}", status, error_text);
            Err(BffError::ApiClient(anyhow::anyhow!("Delete user failed: {}", status)))
        }
    }

    // User profile endpoints
    pub async fn get_user_profile(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users/{}/profile", self.base_url, user_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn update_user_profile(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
        profile_data: &serde_json::Value,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users/{}/profile", self.base_url, user_id);
        
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(profile_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // User preferences endpoints
    pub async fn get_user_preferences(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users/{}/preferences", self.base_url, user_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn update_user_preferences(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
        preferences_data: &serde_json::Value,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users/{}/preferences", self.base_url, user_id);
        
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(preferences_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // User activity endpoints
    pub async fn get_user_activity(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
        params: Option<HashMap<String, String>>,
    ) -> BffResult<serde_json::Value> {
        let mut url = format!("{}/api/v1/users/{}/activity", self.base_url, user_id);
        
        if let Some(params) = params {
            let query_string = params
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("&");
            
            if !query_string.is_empty() {
                url.push_str(&format!("?{}", query_string));
            }
        }
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // User sessions endpoints
    pub async fn get_user_sessions(
        &self,
        user_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/users/{}/sessions", self.base_url, user_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn revoke_user_session(
        &self,
        user_id: &str,
        session_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<()> {
        let url = format!("{}/api/v1/users/{}/sessions/{}", self.base_url, user_id, session_id);
        
        let response = self.client
            .delete(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Revoke session failed: {} - {}", status, error_text);
            Err(BffError::ApiClient(anyhow::anyhow!("Revoke session failed: {}", status)))
        }
    }

    // Workflow endpoints
    pub async fn start_workflow(
        &self,
        workflow_type: &str,
        tenant_id: &str,
        auth_token: &str,
        workflow_data: &serde_json::Value,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/workflows/{}", self.base_url, workflow_type);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(workflow_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn get_workflow_status(
        &self,
        workflow_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/workflows/{}/status", self.base_url, workflow_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    async fn handle_response(&self, response: Response) -> BffResult<serde_json::Value> {
        let status = response.status();
        
        if status.is_success() {
            let json = response.json::<serde_json::Value>().await?;
            debug!("API request successful: {}", status);
            Ok(json)
        } else {
            let error_text = response.text().await.unwrap_or_default();
            error!("API request failed: {} - {}", status, error_text);
            
            match status.as_u16() {
                401 => Err(BffError::authentication("API authentication failed")),
                403 => Err(BffError::authorization("API authorization failed")),
                404 => Err(BffError::not_found("Resource not found")),
                409 => Err(BffError::conflict("Resource conflict")),
                429 => Err(BffError::rate_limit("API rate limit exceeded")),
                _ => Err(BffError::ApiClient(anyhow::anyhow!("API request failed: {}", status))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_user_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/api/v1/users/user123"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "user123",
                "email": "test@example.com"
            })))
            .mount(&mock_server)
            .await;

        std::env::set_var("API_GATEWAY_URL", mock_server.uri());
        
        let client = ApiClient::new().await.unwrap();
        let result = client.get_user("user123", "tenant123", "token123").await;
        
        assert!(result.is_ok());
        let user_data = result.unwrap();
        assert_eq!(user_data["id"], "user123");
        assert_eq!(user_data["email"], "test@example.com");
    }

    #[tokio::test]
    async fn test_get_user_not_found() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/api/v1/users/nonexistent"))
            .respond_with(ResponseTemplate::new(404))
            .mount(&mock_server)
            .await;

        std::env::set_var("API_GATEWAY_URL", mock_server.uri());
        
        let client = ApiClient::new().await.unwrap();
        let result = client.get_user("nonexistent", "tenant123", "token123").await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), BffError::NotFound(_)));
    }
}