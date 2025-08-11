use async_trait::async_trait;
use std::io::Read;
use uuid::Uuid;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use crate::models::{StorageProviderType, S3Config, GcsConfig, AzureConfig, LocalConfig};

#[async_trait]
pub trait StorageProvider: Send + Sync {
    async fn upload(&self, path: &str, data: &[u8]) -> Result<String>;
    async fn download(&self, path: &str) -> Result<Vec<u8>>;
    async fn delete(&self, path: &str) -> Result<()>;
    async fn exists(&self, path: &str) -> Result<bool>;
    async fn get_download_url(&self, path: &str, expires_in_seconds: u64) -> Result<String>;
    async fn get_upload_url(&self, path: &str, expires_in_seconds: u64) -> Result<String>;
    fn provider_type(&self) -> StorageProviderType;
}

pub struct LocalStorageProvider {
    config: LocalConfig,
}

impl LocalStorageProvider {
    pub fn new(config: LocalConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl StorageProvider for LocalStorageProvider {
    async fn upload(&self, path: &str, data: &[u8]) -> Result<String> {
        let full_path = format!("{}/{}", self.config.base_path, path);
        
        // Create directory if it doesn't exist
        if let Some(parent) = std::path::Path::new(&full_path).parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        
        tokio::fs::write(&full_path, data).await?;
        Ok(format!("{}/{}", self.config.url_prefix, path))
    }

    async fn download(&self, path: &str) -> Result<Vec<u8>> {
        let full_path = format!("{}/{}", self.config.base_path, path);
        let data = tokio::fs::read(&full_path).await?;
        Ok(data)
    }

    async fn delete(&self, path: &str) -> Result<()> {
        let full_path = format!("{}/{}", self.config.base_path, path);
        tokio::fs::remove_file(&full_path).await?;
        Ok(())
    }

    async fn exists(&self, path: &str) -> Result<bool> {
        let full_path = format!("{}/{}", self.config.base_path, path);
        Ok(tokio::fs::metadata(&full_path).await.is_ok())
    }

    async fn get_download_url(&self, path: &str, _expires_in_seconds: u64) -> Result<String> {
        // For local storage, return direct URL (in production, this should be secured)
        Ok(format!("{}/{}", self.config.url_prefix, path))
    }

    async fn get_upload_url(&self, path: &str, _expires_in_seconds: u64) -> Result<String> {
        // For local storage, return direct URL (in production, this should be secured)
        Ok(format!("{}/{}", self.config.url_prefix, path))
    }

    fn provider_type(&self) -> StorageProviderType {
        StorageProviderType::Local
    }
}

// S3 Storage Provider (placeholder - would need AWS SDK)
pub struct S3StorageProvider {
    config: S3Config,
}

impl S3StorageProvider {
    pub fn new(config: S3Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl StorageProvider for S3StorageProvider {
    async fn upload(&self, path: &str, _data: &[u8]) -> Result<String> {
        // TODO: Implement S3 upload using AWS SDK
        // This is a placeholder implementation
        tracing::warn!("S3 storage provider not fully implemented");
        Ok(format!("s3://{}/{}", self.config.bucket, path))
    }

    async fn download(&self, _path: &str) -> Result<Vec<u8>> {
        // TODO: Implement S3 download using AWS SDK
        tracing::warn!("S3 storage provider not fully implemented");
        Err(anyhow::anyhow!("S3 download not implemented"))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        // TODO: Implement S3 delete using AWS SDK
        tracing::warn!("S3 storage provider not fully implemented");
        Ok(())
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        // TODO: Implement S3 exists check using AWS SDK
        tracing::warn!("S3 storage provider not fully implemented");
        Ok(false)
    }

    async fn get_download_url(&self, path: &str, expires_in_seconds: u64) -> Result<String> {
        // TODO: Generate presigned URL using AWS SDK
        tracing::warn!("S3 storage provider not fully implemented");
        Ok(format!("https://{}.s3.amazonaws.com/{}", self.config.bucket, path))
    }

    async fn get_upload_url(&self, path: &str, expires_in_seconds: u64) -> Result<String> {
        // TODO: Generate presigned URL using AWS SDK
        tracing::warn!("S3 storage provider not fully implemented");
        Ok(format!("https://{}.s3.amazonaws.com/{}", self.config.bucket, path))
    }

    fn provider_type(&self) -> StorageProviderType {
        StorageProviderType::S3
    }
}

// GCS Storage Provider (placeholder)
pub struct GcsStorageProvider {
    config: GcsConfig,
}

impl GcsStorageProvider {
    pub fn new(config: GcsConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl StorageProvider for GcsStorageProvider {
    async fn upload(&self, path: &str, _data: &[u8]) -> Result<String> {
        tracing::warn!("GCS storage provider not fully implemented");
        Ok(format!("gs://{}/{}", self.config.bucket, path))
    }

    async fn download(&self, _path: &str) -> Result<Vec<u8>> {
        tracing::warn!("GCS storage provider not fully implemented");
        Err(anyhow::anyhow!("GCS download not implemented"))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        tracing::warn!("GCS storage provider not fully implemented");
        Ok(())
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        tracing::warn!("GCS storage provider not fully implemented");
        Ok(false)
    }

    async fn get_download_url(&self, path: &str, _expires_in_seconds: u64) -> Result<String> {
        tracing::warn!("GCS storage provider not fully implemented");
        Ok(format!("https://storage.googleapis.com/{}/{}", self.config.bucket, path))
    }

    async fn get_upload_url(&self, path: &str, _expires_in_seconds: u64) -> Result<String> {
        tracing::warn!("GCS storage provider not fully implemented");
        Ok(format!("https://storage.googleapis.com/{}/{}", self.config.bucket, path))
    }

    fn provider_type(&self) -> StorageProviderType {
        StorageProviderType::Gcs
    }
}

// Azure Storage Provider (placeholder)
pub struct AzureStorageProvider {
    config: AzureConfig,
}

impl AzureStorageProvider {
    pub fn new(config: AzureConfig) -> Self {
        Self { config }
    }
}

#[async_trait]
impl StorageProvider for AzureStorageProvider {
    async fn upload(&self, path: &str, _data: &[u8]) -> Result<String> {
        tracing::warn!("Azure storage provider not fully implemented");
        Ok(format!("https://{}.blob.core.windows.net/{}/{}", 
                  self.config.account_name, self.config.container_name, path))
    }

    async fn download(&self, _path: &str) -> Result<Vec<u8>> {
        tracing::warn!("Azure storage provider not fully implemented");
        Err(anyhow::anyhow!("Azure download not implemented"))
    }

    async fn delete(&self, _path: &str) -> Result<()> {
        tracing::warn!("Azure storage provider not fully implemented");
        Ok(())
    }

    async fn exists(&self, _path: &str) -> Result<bool> {
        tracing::warn!("Azure storage provider not fully implemented");
        Ok(false)
    }

    async fn get_download_url(&self, path: &str, _expires_in_seconds: u64) -> Result<String> {
        tracing::warn!("Azure storage provider not fully implemented");
        Ok(format!("https://{}.blob.core.windows.net/{}/{}", 
                  self.config.account_name, self.config.container_name, path))
    }

    async fn get_upload_url(&self, path: &str, _expires_in_seconds: u64) -> Result<String> {
        tracing::warn!("Azure storage provider not fully implemented");
        Ok(format!("https://{}.blob.core.windows.net/{}/{}", 
                  self.config.account_name, self.config.container_name, path))
    }

    fn provider_type(&self) -> StorageProviderType {
        StorageProviderType::Azure
    }
}

// Storage Manager to handle multiple providers
pub struct StorageManager {
    providers: std::collections::HashMap<String, Box<dyn StorageProvider>>,
    default_provider: String,
}

impl StorageManager {
    pub fn new() -> Self {
        Self {
            providers: std::collections::HashMap::new(),
            default_provider: "local".to_string(),
        }
    }

    pub fn add_provider(&mut self, name: String, provider: Box<dyn StorageProvider>) {
        self.providers.insert(name, provider);
    }

    pub fn set_default_provider(&mut self, name: String) {
        self.default_provider = name;
    }

    pub fn get_provider(&self, name: Option<&str>) -> Option<&dyn StorageProvider> {
        let provider_name = name.unwrap_or(&self.default_provider);
        self.providers.get(provider_name).map(|p| p.as_ref())
    }

    pub async fn upload(&self, provider_name: Option<&str>, path: &str, data: &[u8]) -> Result<String> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Storage provider not found"))?;
        provider.upload(path, data).await
    }

    pub async fn download(&self, provider_name: Option<&str>, path: &str) -> Result<Vec<u8>> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Storage provider not found"))?;
        provider.download(path).await
    }

    pub async fn delete(&self, provider_name: Option<&str>, path: &str) -> Result<()> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Storage provider not found"))?;
        provider.delete(path).await
    }

    pub async fn get_download_url(&self, provider_name: Option<&str>, path: &str, expires_in_seconds: u64) -> Result<String> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Storage provider not found"))?;
        provider.get_download_url(path, expires_in_seconds).await
    }

    pub async fn get_upload_url(&self, provider_name: Option<&str>, path: &str, expires_in_seconds: u64) -> Result<String> {
        let provider = self.get_provider(provider_name)
            .ok_or_else(|| anyhow::anyhow!("Storage provider not found"))?;
        provider.get_upload_url(path, expires_in_seconds).await
    }
}

impl Default for StorageManager {
    fn default() -> Self {
        Self::new()
    }
}