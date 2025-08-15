use crate::config::WhiteLabelConfig;
use crate::error::{WhiteLabelError, WhiteLabelResult};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::io::AsyncWriteExt;

pub struct StorageService {
    config: Arc<WhiteLabelConfig>,
}

impl StorageService {
    pub fn new(config: Arc<WhiteLabelConfig>) -> Self {
        Self { config }
    }

    pub async fn store_file(&self, file_path: &str, data: &[u8]) -> WhiteLabelResult<()> {
        match self.config.storage_config.provider.as_str() {
            "local" => self.store_file_local(file_path, data).await,
            "s3" => self.store_file_s3(file_path, data).await,
            "gcs" => self.store_file_gcs(file_path, data).await,
            "azure" => self.store_file_azure(file_path, data).await,
            _ => Err(WhiteLabelError::Configuration(format!(
                "Unsupported storage provider: {}",
                self.config.storage_config.provider
            ))),
        }
    }

    pub async fn retrieve_file(&self, file_path: &str) -> WhiteLabelResult<Vec<u8>> {
        match self.config.storage_config.provider.as_str() {
            "local" => self.retrieve_file_local(file_path).await,
            "s3" => self.retrieve_file_s3(file_path).await,
            "gcs" => self.retrieve_file_gcs(file_path).await,
            "azure" => self.retrieve_file_azure(file_path).await,
            _ => Err(WhiteLabelError::Configuration(format!(
                "Unsupported storage provider: {}",
                self.config.storage_config.provider
            ))),
        }
    }

    pub async fn delete_file(&self, file_path: &str) -> WhiteLabelResult<()> {
        match self.config.storage_config.provider.as_str() {
            "local" => self.delete_file_local(file_path).await,
            "s3" => self.delete_file_s3(file_path).await,
            "gcs" => self.delete_file_gcs(file_path).await,
            "azure" => self.delete_file_azure(file_path).await,
            _ => Err(WhiteLabelError::Configuration(format!(
                "Unsupported storage provider: {}",
                self.config.storage_config.provider
            ))),
        }
    }

    pub async fn file_exists(&self, file_path: &str) -> WhiteLabelResult<bool> {
        match self.config.storage_config.provider.as_str() {
            "local" => self.file_exists_local(file_path).await,
            "s3" => self.file_exists_s3(file_path).await,
            "gcs" => self.file_exists_gcs(file_path).await,
            "azure" => self.file_exists_azure(file_path).await,
            _ => Err(WhiteLabelError::Configuration(format!(
                "Unsupported storage provider: {}",
                self.config.storage_config.provider
            ))),
        }
    }

    pub fn get_file_url(&self, file_path: &str) -> String {
        match self.config.storage_config.provider.as_str() {
            "local" => {
                if let Some(ref cdn_base_url) = self.config.asset_config.cdn_base_url {
                    format!("{}/{}", cdn_base_url, file_path)
                } else {
                    format!("/storage/{}", file_path)
                }
            }
            "s3" => {
                let bucket = self.config.storage_config.bucket_name.as_deref().unwrap_or("default");
                let region = self.config.storage_config.region.as_deref().unwrap_or("us-east-1");
                format!("https://{}.s3.{}.amazonaws.com/{}", bucket, region, file_path)
            }
            "gcs" => {
                let bucket = self.config.storage_config.bucket_name.as_deref().unwrap_or("default");
                format!("https://storage.googleapis.com/{}/{}", bucket, file_path)
            }
            "azure" => {
                let account = self.config.storage_config.access_key.as_deref().unwrap_or("default");
                format!("https://{}.blob.core.windows.net/{}", account, file_path)
            }
            _ => format!("/storage/{}", file_path),
        }
    }

    // Local storage implementation
    async fn store_file_local(&self, file_path: &str, data: &[u8]) -> WhiteLabelResult<()> {
        let full_path = Path::new(&self.config.asset_config.storage_path).join(file_path);
        
        // Create parent directories if they don't exist
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                WhiteLabelError::ExternalService(format!("Failed to create directories: {}", e))
            })?;
        }

        // Write file
        let mut file = fs::File::create(&full_path).await.map_err(|e| {
            WhiteLabelError::ExternalService(format!("Failed to create file: {}", e))
        })?;

        file.write_all(data).await.map_err(|e| {
            WhiteLabelError::ExternalService(format!("Failed to write file: {}", e))
        })?;

        file.flush().await.map_err(|e| {
            WhiteLabelError::ExternalService(format!("Failed to flush file: {}", e))
        })?;

        tracing::info!("Stored file locally: {}", full_path.display());
        Ok(())
    }

    async fn retrieve_file_local(&self, file_path: &str) -> WhiteLabelResult<Vec<u8>> {
        let full_path = Path::new(&self.config.asset_config.storage_path).join(file_path);
        
        fs::read(&full_path).await.map_err(|e| {
            WhiteLabelError::ExternalService(format!("Failed to read file: {}", e))
        })
    }

    async fn delete_file_local(&self, file_path: &str) -> WhiteLabelResult<()> {
        let full_path = Path::new(&self.config.asset_config.storage_path).join(file_path);
        
        fs::remove_file(&full_path).await.map_err(|e| {
            WhiteLabelError::ExternalService(format!("Failed to delete file: {}", e))
        })?;

        tracing::info!("Deleted file locally: {}", full_path.display());
        Ok(())
    }

    async fn file_exists_local(&self, file_path: &str) -> WhiteLabelResult<bool> {
        let full_path = Path::new(&self.config.asset_config.storage_path).join(file_path);
        Ok(full_path.exists())
    }

    // S3 storage implementation (placeholder)
    async fn store_file_s3(&self, file_path: &str, _data: &[u8]) -> WhiteLabelResult<()> {
        tracing::info!("Storing file to S3: {}", file_path);
        // In a real implementation, this would use the AWS SDK
        Ok(())
    }

    async fn retrieve_file_s3(&self, file_path: &str) -> WhiteLabelResult<Vec<u8>> {
        tracing::info!("Retrieving file from S3: {}", file_path);
        // In a real implementation, this would use the AWS SDK
        Ok(Vec::new())
    }

    async fn delete_file_s3(&self, file_path: &str) -> WhiteLabelResult<()> {
        tracing::info!("Deleting file from S3: {}", file_path);
        // In a real implementation, this would use the AWS SDK
        Ok(())
    }

    async fn file_exists_s3(&self, file_path: &str) -> WhiteLabelResult<bool> {
        tracing::info!("Checking if file exists in S3: {}", file_path);
        // In a real implementation, this would use the AWS SDK
        Ok(false)
    }

    // GCS storage implementation (placeholder)
    async fn store_file_gcs(&self, file_path: &str, _data: &[u8]) -> WhiteLabelResult<()> {
        tracing::info!("Storing file to GCS: {}", file_path);
        // In a real implementation, this would use the Google Cloud SDK
        Ok(())
    }

    async fn retrieve_file_gcs(&self, file_path: &str) -> WhiteLabelResult<Vec<u8>> {
        tracing::info!("Retrieving file from GCS: {}", file_path);
        // In a real implementation, this would use the Google Cloud SDK
        Ok(Vec::new())
    }

    async fn delete_file_gcs(&self, file_path: &str) -> WhiteLabelResult<()> {
        tracing::info!("Deleting file from GCS: {}", file_path);
        // In a real implementation, this would use the Google Cloud SDK
        Ok(())
    }

    async fn file_exists_gcs(&self, file_path: &str) -> WhiteLabelResult<bool> {
        tracing::info!("Checking if file exists in GCS: {}", file_path);
        // In a real implementation, this would use the Google Cloud SDK
        Ok(false)
    }

    // Azure storage implementation (placeholder)
    async fn store_file_azure(&self, file_path: &str, _data: &[u8]) -> WhiteLabelResult<()> {
        tracing::info!("Storing file to Azure: {}", file_path);
        // In a real implementation, this would use the Azure SDK
        Ok(())
    }

    async fn retrieve_file_azure(&self, file_path: &str) -> WhiteLabelResult<Vec<u8>> {
        tracing::info!("Retrieving file from Azure: {}", file_path);
        // In a real implementation, this would use the Azure SDK
        Ok(Vec::new())
    }

    async fn delete_file_azure(&self, file_path: &str) -> WhiteLabelResult<()> {
        tracing::info!("Deleting file from Azure: {}", file_path);
        // In a real implementation, this would use the Azure SDK
        Ok(())
    }

    async fn file_exists_azure(&self, file_path: &str) -> WhiteLabelResult<bool> {
        tracing::info!("Checking if file exists in Azure: {}", file_path);
        // In a real implementation, this would use the Azure SDK
        Ok(false)
    }
}