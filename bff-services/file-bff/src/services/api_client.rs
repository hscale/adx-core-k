use anyhow::{Context, Result};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{debug, error, warn};

#[derive(Clone)]
pub struct ApiClient {
    client: Client,
    api_gateway_url: String,
    file_service_url: String,
}

impl ApiClient {
    pub async fn new() -> Result<Self> {
        let api_gateway_url = std::env::var("API_GATEWAY_URL")
            .unwrap_or_else(|_| "http://localhost:8080".to_string());
        
        let file_service_url = std::env::var("FILE_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8083".to_string());

        let client = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to create HTTP client")?;

        Ok(Self {
            client,
            api_gateway_url,
            file_service_url,
        })
    }

    // File Service direct calls
    pub async fn get_file_metadata(
        &self,
        file_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/files/{}", self.file_service_url, file_id);
        
        debug!("Fetching file metadata from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to fetch file metadata")?;

        self.handle_response(response).await
    }

    pub async fn list_files(
        &self,
        tenant_id: &str,
        auth_token: &str,
        params: &[(&str, &str)],
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/files", self.file_service_url);
        
        debug!("Listing files from: {} with params: {:?}", url, params);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .query(params)
            .send()
            .await
            .context("Failed to list files")?;

        self.handle_response(response).await
    }

    pub async fn get_file_permissions(
        &self,
        file_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/files/{}/permissions", self.file_service_url, file_id);
        
        debug!("Fetching file permissions from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to fetch file permissions")?;

        self.handle_response(response).await
    }

    pub async fn get_storage_info(
        &self,
        file_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/files/{}/storage", self.file_service_url, file_id);
        
        debug!("Fetching storage info from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to fetch storage info")?;

        self.handle_response(response).await
    }

    // Workflow operations through API Gateway
    pub async fn initiate_workflow<T: Serialize>(
        &self,
        workflow_type: &str,
        input: &T,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/workflows/{}", self.api_gateway_url, workflow_type);
        
        debug!("Initiating workflow: {} at {}", workflow_type, url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(input)
            .send()
            .await
            .context("Failed to initiate workflow")?;

        self.handle_response(response).await
    }

    pub async fn get_workflow_status(
        &self,
        operation_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/workflows/{}/status", self.api_gateway_url, operation_id);
        
        debug!("Getting workflow status from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to get workflow status")?;

        self.handle_response(response).await
    }

    pub async fn cancel_workflow(
        &self,
        operation_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/workflows/{}/cancel", self.api_gateway_url, operation_id);
        
        debug!("Cancelling workflow at: {}", url);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to cancel workflow")?;

        self.handle_response(response).await
    }

    // Search files with advanced filtering
    pub async fn search_files(
        &self,
        search_params: &serde_json::Value,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/files/search", self.file_service_url);
        
        debug!("Searching files at: {} with params: {}", url, search_params);
        
        let response = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .header("Content-Type", "application/json")
            .json(search_params)
            .send()
            .await
            .context("Failed to search files")?;

        self.handle_response(response).await
    }

    // Get upload progress
    pub async fn get_upload_progress(
        &self,
        upload_id: &str,
        tenant_id: &str,
        auth_token: &str,
    ) -> Result<serde_json::Value> {
        let url = format!("{}/api/v1/uploads/{}/progress", self.file_service_url, upload_id);
        
        debug!("Getting upload progress from: {}", url);
        
        let response = self
            .client
            .get(&url)
            .header("Authorization", format!("Bearer {}", auth_token))
            .header("X-Tenant-ID", tenant_id)
            .send()
            .await
            .context("Failed to get upload progress")?;

        self.handle_response(response).await
    }

    // Helper method to handle HTTP responses
    async fn handle_response(&self, response: Response) -> Result<serde_json::Value> {
        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read response body")?;

        if status.is_success() {
            serde_json::from_str(&response_text)
                .context("Failed to parse JSON response")
        } else {
            error!("API request failed with status {}: {}", status, response_text);
            
            // Try to parse error response
            if let Ok(error_json) = serde_json::from_str::<serde_json::Value>(&response_text) {
                return Err(anyhow::anyhow!("API Error: {}", error_json));
            }
            
            Err(anyhow::anyhow!("API request failed with status {}: {}", status, response_text))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::{matchers::{method, path}, Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn test_get_file_metadata() {
        let mock_server = MockServer::start().await;
        
        Mock::given(method("GET"))
            .and(path("/api/v1/files/test-file-id"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "test-file-id",
                "name": "test.txt",
                "size": 1024
            })))
            .mount(&mock_server)
            .await;

        std::env::set_var("FILE_SERVICE_URL", mock_server.uri());
        
        let client = ApiClient::new().await.unwrap();
        let result = client
            .get_file_metadata("test-file-id", "tenant-1", "test-token")
            .await;

        assert!(result.is_ok());
        let data = result.unwrap();
        assert_eq!(data["id"], "test-file-id");
    }
}