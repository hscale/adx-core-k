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

    // Workflow management endpoints
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

    pub async fn list_workflows(
        &self,
        tenant_id: &str,
        auth_token: &str,
        params: Option<HashMap<String, String>>,
    ) -> BffResult<serde_json::Value> {
        let mut url = format!("{}/api/v1/workflows", self.base_url);
        
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

    pub async fn cancel_workflow(
        &self,
        workflow_id: &str,
        tenant_id: &str,
        auth_token: &str,
        reason: Option<&str>,
    ) -> BffResult<()> {
        let url = format!("{}/api/v1/workflows/{}/cancel", self.base_url, workflow_id);
        
        let body = serde_json::json!({
            "reason": reason
        });
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Cancel workflow failed: {} - {}", status, error_text);
            Err(BffError::api_client(anyhow::anyhow!("Cancel workflow failed: {}", status)))
        }
    }

    pub async fn terminate_workflow(
        &self,
        workflow_id: &str,
        tenant_id: &str,
        auth_token: &str,
        reason: Option<&str>,
    ) -> BffResult<()> {
        let url = format!("{}/api/v1/workflows/{}/terminate", self.base_url, workflow_id);
        
        let body = serde_json::json!({
            "reason": reason
        });
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Terminate workflow failed: {} - {}", status, error_text);
            Err(BffError::api_client(anyhow::anyhow!("Terminate workflow failed: {}", status)))
        }
    }

    pub async fn query_workflow(
        &self,
        workflow_id: &str,
        query_type: &str,
        tenant_id: &str,
        auth_token: &str,
        query_args: Option<&serde_json::Value>,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/workflows/{}/query", self.base_url, workflow_id);
        
        let body = serde_json::json!({
            "query_type": query_type,
            "query_args": query_args
        });
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn signal_workflow(
        &self,
        workflow_id: &str,
        signal_name: &str,
        tenant_id: &str,
        auth_token: &str,
        signal_input: Option<&serde_json::Value>,
    ) -> BffResult<()> {
        let url = format!("{}/api/v1/workflows/{}/signal", self.base_url, workflow_id);
        
        let body = serde_json::json!({
            "signal_name": signal_name,
            "signal_input": signal_input
        });
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let status = response.status();
            let error_text = response.text().await.unwrap_or_default();
            error!("Signal workflow failed: {} - {}", status, error_text);
            Err(BffError::api_client(anyhow::anyhow!("Signal workflow failed: {}", status)))
        }
    }

    pub async fn get_workflow_history(
        &self,
        workflow_id: &str,
        tenant_id: &str,
        auth_token: &str,
        params: Option<HashMap<String, String>>,
    ) -> BffResult<serde_json::Value> {
        let mut url = format!("{}/api/v1/workflows/{}/history", self.base_url, workflow_id);
        
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

    // Workflow template endpoints
    pub async fn list_workflow_templates(
        &self,
        tenant_id: &str,
        auth_token: &str,
        params: Option<HashMap<String, String>>,
    ) -> BffResult<serde_json::Value> {
        let mut url = format!("{}/api/v1/workflow-templates", self.base_url);
        
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

    pub async fn get_workflow_template(
        &self,
        template_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/workflow-templates/{}", self.base_url, template_id);
        
        let response = self.client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await?;

        self.handle_response(response).await
    }

    // Workflow schedule endpoints
    pub async fn list_workflow_schedules(
        &self,
        tenant_id: &str,
        auth_token: &str,
        params: Option<HashMap<String, String>>,
    ) -> BffResult<serde_json::Value> {
        let mut url = format!("{}/api/v1/workflow-schedules", self.base_url);
        
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

    pub async fn create_workflow_schedule(
        &self,
        tenant_id: &str,
        auth_token: &str,
        schedule_data: &serde_json::Value,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/workflow-schedules", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(schedule_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn update_workflow_schedule(
        &self,
        schedule_id: &str,
        tenant_id: &str,
        auth_token: &str,
        schedule_data: &serde_json::Value,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/workflow-schedules/{}", self.base_url, schedule_id);
        
        let response = self.client
            .put(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(schedule_data)
            .send()
            .await?;

        self.handle_response(response).await
    }

    pub async fn delete_workflow_schedule(
        &self,
        schedule_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<()> {
        let url = format!("{}/api/v1/workflow-schedules/{}", self.base_url, schedule_id);
        
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
            error!("Delete workflow schedule failed: {} - {}", status, error_text);
            Err(BffError::api_client(anyhow::anyhow!("Delete workflow schedule failed: {}", status)))
        }
    }

    // Monitoring endpoints
    pub async fn get_workflow_metrics(
        &self,
        tenant_id: &str,
        auth_token: &str,
        params: Option<HashMap<String, String>>,
    ) -> BffResult<serde_json::Value> {
        let mut url = format!("{}/api/v1/monitoring/workflows/metrics", self.base_url);
        
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

    pub async fn get_system_health(
        &self,
        tenant_id: &str,
        auth_token: &str,
    ) -> BffResult<serde_json::Value> {
        let url = format!("{}/api/v1/monitoring/health", self.base_url);
        
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
                _ => Err(BffError::api_client(anyhow::anyhow!("API request failed: {}", status))),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_workflow_status_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/api/v1/workflows/workflow123/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "workflow_id": "workflow123",
                "status": "running"
            })))
            .mount(&mock_server)
            .await;

        std::env::set_var("API_GATEWAY_URL", mock_server.uri());
        
        let client = ApiClient::new().await.unwrap();
        let result = client.get_workflow_status("workflow123", "tenant123", "token123").await;
        
        assert!(result.is_ok());
        let status_data = result.unwrap();
        assert_eq!(status_data["workflow_id"], "workflow123");
        assert_eq!(status_data["status"], "running");
    }

    #[tokio::test]
    async fn test_start_workflow_success() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("POST"))
            .and(path("/api/v1/workflows/test_workflow"))
            .respond_with(ResponseTemplate::new(202).set_body_json(serde_json::json!({
                "workflow_id": "workflow123",
                "run_id": "run456",
                "status_url": "/api/workflows/workflow123/status"
            })))
            .mount(&mock_server)
            .await;

        std::env::set_var("API_GATEWAY_URL", mock_server.uri());
        
        let client = ApiClient::new().await.unwrap();
        let workflow_data = serde_json::json!({"input": "test"});
        let result = client.start_workflow("test_workflow", "tenant123", "token123", &workflow_data).await;
        
        assert!(result.is_ok());
        let response_data = result.unwrap();
        assert_eq!(response_data["workflow_id"], "workflow123");
    }
}