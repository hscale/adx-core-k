use async_trait::async_trait;
use std::path::Path;
use sha2::{Sha256, Digest};

use crate::error::ModuleServiceError;
use crate::config::StorageConfig;

#[async_trait]
pub trait PackageServiceTrait {
    async fn download_package(&self, module_id: &str, version: &str, checksum: &str) -> Result<String, ModuleServiceError>;
    async fn verify_checksum(&self, package_path: &str, expected_checksum: &str) -> Result<bool, ModuleServiceError>;
    async fn extract_package(&self, package_path: &str, extraction_id: &str) -> Result<String, ModuleServiceError>;
    async fn deploy_to_tenant(&self, source_path: &str, tenant_id: &str, module_id: &str) -> Result<String, ModuleServiceError>;
    async fn download_package_for_scan(&self, module_id: &str, version: &str) -> Result<String, ModuleServiceError>;
}

pub struct PackageService {
    storage_config: StorageConfig,
}

impl PackageService {
    pub fn new(storage_config: StorageConfig) -> Self {
        Self { storage_config }
    }
}

#[async_trait]
impl PackageServiceTrait for PackageService {
    async fn download_package(&self, module_id: &str, version: &str, checksum: &str) -> Result<String, ModuleServiceError> {
        let package_url = format!("https://packages.adxcore.com/{}/{}.tar.gz", module_id, version);
        let local_path = format!("/tmp/packages/{}_{}.tar.gz", module_id, version);
        
        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&local_path).parent() {
            std::fs::create_dir_all(parent)?;
        }
        
        // Download package (simplified - would use proper HTTP client)
        let response = reqwest::get(&package_url).await?;
        let bytes = response.bytes().await?;
        
        std::fs::write(&local_path, bytes)?;
        
        Ok(local_path)
    }

    async fn verify_checksum(&self, package_path: &str, expected_checksum: &str) -> Result<bool, ModuleServiceError> {
        let contents = std::fs::read(package_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&contents);
        let result = hasher.finalize();
        let actual_checksum = format!("{:x}", result);
        
        Ok(actual_checksum == expected_checksum)
    }

    async fn extract_package(&self, package_path: &str, extraction_id: &str) -> Result<String, ModuleServiceError> {
        let extraction_path = format!("/tmp/extractions/{}", extraction_id);
        std::fs::create_dir_all(&extraction_path)?;
        
        // Extract using tar command (simplified)
        let output = std::process::Command::new("tar")
            .args(&["-xzf", package_path, "-C", &extraction_path])
            .output()?;
        
        if !output.status.success() {
            return Err(ModuleServiceError::InternalError(
                format!("Failed to extract package: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(extraction_path)
    }

    async fn deploy_to_tenant(&self, source_path: &str, tenant_id: &str, module_id: &str) -> Result<String, ModuleServiceError> {
        let deployment_path = format!("/modules/{}/{}", tenant_id, module_id);
        std::fs::create_dir_all(&deployment_path)?;
        
        // Copy files from source to deployment path
        let output = std::process::Command::new("cp")
            .args(&["-r", &format!("{}/*", source_path), &deployment_path])
            .output()?;
        
        if !output.status.success() {
            return Err(ModuleServiceError::InternalError(
                format!("Failed to deploy module: {}", String::from_utf8_lossy(&output.stderr))
            ));
        }
        
        Ok(deployment_path)
    }

    async fn download_package_for_scan(&self, module_id: &str, version: &str) -> Result<String, ModuleServiceError> {
        // Similar to download_package but for security scanning
        self.download_package(module_id, version, "").await
    }
}