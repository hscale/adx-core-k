use std::sync::Arc;
use uuid::Uuid;
use adx_shared::{Result, TenantContext, UserContext};
use crate::models::*;
use crate::repositories::*;
use crate::storage::StorageManager;

pub struct FileService {
    file_repo: Arc<dyn FileRepository>,
    permission_repo: Arc<dyn FilePermissionRepository>,
    share_repo: Arc<dyn FileShareRepository>,
    storage_manager: Arc<StorageManager>,
}

impl FileService {
    pub fn new(
        file_repo: Arc<dyn FileRepository>,
        permission_repo: Arc<dyn FilePermissionRepository>,
        share_repo: Arc<dyn FileShareRepository>,
        storage_manager: Arc<StorageManager>,
    ) -> Self {
        Self {
            file_repo,
            permission_repo,
            share_repo,
            storage_manager,
        }
    }

    pub async fn create_file(
        &self,
        request: &CreateFileRequest,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<FileUploadResponse> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        // Create file record
        let file = self.file_repo.create(request, tenant_context, user_uuid).await?;
        
        // Generate upload URL for direct upload
        let upload_url = if request.file_size > 1024 * 1024 * 10 { // 10MB threshold
            // For large files, provide presigned upload URL
            Some(self.storage_manager.get_upload_url(None, &file.storage_path, 3600).await?)
        } else {
            // For small files, allow direct upload through API
            None
        };

        Ok(FileUploadResponse {
            file_id: file.id,
            upload_url,
            status: file.status,
        })
    }

    pub async fn get_file(
        &self,
        file_id: Uuid,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<Option<File>> {
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?;
        
        if let Some(ref file) = file {
            let user_uuid = Uuid::parse_str(&user_context.user_id)
                .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
            
            // Check if user has permission to access this file
            if file.user_id != user_uuid && !file.is_public {
                let has_permission = self.permission_repo
                    .check_permission(file_id, user_uuid, PermissionType::Read, tenant_context)
                    .await?;
                
                if !has_permission {
                    return Ok(None); // Return None instead of error for security
                }
            }
        }

        Ok(file)
    }

    pub async fn update_file(
        &self,
        file_id: Uuid,
        updates: &UpdateFileRequest,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<File> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        // Check if user owns the file or has write permission
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        if file.user_id != user_uuid {
            let has_permission = self.permission_repo
                .check_permission(file_id, user_uuid, PermissionType::Write, tenant_context)
                .await?;
            
            if !has_permission {
                return Err(anyhow::anyhow!("Permission denied"));
            }
        }

        self.file_repo.update(file_id, updates, tenant_context).await
    }

    pub async fn delete_file(
        &self,
        file_id: Uuid,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<()> {
        // Check if user owns the file or has admin permission
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        if file.user_id != user_context.user_id {
            let has_permission = self.permission_repo
                .check_permission(file_id, user_context.user_id, PermissionType::Admin, tenant_context)
                .await?;
            
            if !has_permission {
                return Err(anyhow::anyhow!("Permission denied"));
            }
        }

        // Mark file as deleted in database
        self.file_repo.delete(file_id, tenant_context).await?;

        // TODO: Schedule actual file deletion from storage (should be done via workflow)
        
        Ok(())
    }

    pub async fn list_files(
        &self,
        tenant_context: &TenantContext,
        user_context: &UserContext,
        page: i32,
        per_page: i32,
    ) -> Result<FileListResponse> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        // For now, only show user's own files
        // TODO: Add support for shared files and admin view
        self.file_repo.list(tenant_context, Some(user_uuid), page, per_page).await
    }

    pub async fn upload_file_data(
        &self,
        file_id: Uuid,
        data: &[u8],
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<()> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        // Check if user owns the file
        if file.user_id != user_uuid {
            return Err(anyhow::anyhow!("Permission denied"));
        }

        // Upload to storage
        let storage_url = self.storage_manager.upload(None, &file.storage_path, data).await?;
        
        // Calculate checksum
        let checksum = format!("{:x}", md5::compute(data));

        // Update file status and storage info
        self.file_repo.update_storage_info(file_id, &storage_url, Some(&checksum), tenant_context).await?;
        self.file_repo.update_status(file_id, FileStatus::Ready, tenant_context).await?;

        Ok(())
    }

    pub async fn download_file(
        &self,
        file_id: Uuid,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<FileDownloadResponse> {
        let file = self.get_file(file_id, tenant_context, user_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found or access denied"))?;

        if file.status != FileStatus::Ready {
            return Err(anyhow::anyhow!("File not ready for download"));
        }

        // Generate download URL
        let download_url = self.storage_manager.get_download_url(None, &file.storage_path, 3600).await?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(3600);

        Ok(FileDownloadResponse {
            download_url,
            expires_at,
        })
    }

    pub async fn create_file_share(
        &self,
        file_id: Uuid,
        request: &CreateFileShareRequest,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<FileShare> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        // Check if user owns the file or has admin permission
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        if file.user_id != user_uuid {
            let has_permission = self.permission_repo
                .check_permission(file_id, user_uuid, PermissionType::Admin, tenant_context)
                .await?;
            
            if !has_permission {
                return Err(anyhow::anyhow!("Permission denied"));
            }
        }

        self.share_repo.create(file_id, request, tenant_context, user_uuid).await
    }

    pub async fn get_file_shares(
        &self,
        file_id: Uuid,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<Vec<FileShare>> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        // Check if user owns the file or has admin permission
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        if file.user_id != user_uuid {
            let has_permission = self.permission_repo
                .check_permission(file_id, user_uuid, PermissionType::Admin, tenant_context)
                .await?;
            
            if !has_permission {
                return Err(anyhow::anyhow!("Permission denied"));
            }
        }

        self.share_repo.get_by_file_id(file_id, tenant_context).await
    }

    pub async fn access_shared_file(
        &self,
        share_token: &str,
        password: Option<&str>,
    ) -> Result<FileDownloadResponse> {
        let share = self.share_repo.get_by_token(share_token).await?
            .ok_or_else(|| anyhow::anyhow!("Invalid or expired share link"))?;

        // Check download limit
        if let Some(limit) = share.download_limit {
            if share.download_count >= limit {
                return Err(anyhow::anyhow!("Download limit exceeded"));
            }
        }

        // Check password if required
        if let Some(hash) = &share.password_hash {
            let provided_password = password.ok_or_else(|| anyhow::anyhow!("Password required"))?;
            if !bcrypt::verify(provided_password, hash).map_err(|e| anyhow::anyhow!("Password verification failed: {}", e))? {
                return Err(anyhow::anyhow!("Invalid password"));
            }
        }

        // Get file info (we need tenant context, but for shared files we can bypass some checks)
        let tenant_context = TenantContext {
            tenant_id: share.tenant_id.to_string(),
            tenant_name: "".to_string(), // We don't have this info in share context
            subscription_tier: adx_shared::SubscriptionTier::Free, // Default
            features: vec![],
            quotas: adx_shared::TenantQuotas::default(),
            settings: adx_shared::TenantSettings::default(),
            is_active: true,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let file = self.file_repo.get_by_id(share.file_id, &tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        if file.status != FileStatus::Ready {
            return Err(anyhow::anyhow!("File not ready for download"));
        }

        // Update download count
        self.share_repo.update_download_count(share.id).await?;

        // Generate download URL
        let download_url = self.storage_manager.get_download_url(None, &file.storage_path, 3600).await?;
        let expires_at = chrono::Utc::now() + chrono::Duration::seconds(3600);

        Ok(FileDownloadResponse {
            download_url,
            expires_at,
        })
    }

    pub async fn grant_file_permission(
        &self,
        file_id: Uuid,
        request: &CreateFilePermissionRequest,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<FilePermission> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        // Check if user owns the file or has admin permission
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        if file.user_id != user_uuid {
            let has_permission = self.permission_repo
                .check_permission(file_id, user_uuid, PermissionType::Admin, tenant_context)
                .await?;
            
            if !has_permission {
                return Err(anyhow::anyhow!("Permission denied"));
            }
        }

        self.permission_repo.create(file_id, request, tenant_context, user_uuid).await
    }

    pub async fn get_file_permissions(
        &self,
        file_id: Uuid,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<Vec<FilePermission>> {
        let user_uuid = Uuid::parse_str(&user_context.user_id)
            .map_err(|e| anyhow::anyhow!("Invalid user ID format: {}", e))?;
        
        // Check if user owns the file or has admin permission
        let file = self.file_repo.get_by_id(file_id, tenant_context).await?
            .ok_or_else(|| anyhow::anyhow!("File not found"))?;

        if file.user_id != user_uuid {
            let has_permission = self.permission_repo
                .check_permission(file_id, user_uuid, PermissionType::Admin, tenant_context)
                .await?;
            
            if !has_permission {
                return Err(anyhow::anyhow!("Permission denied"));
            }
        }

        self.permission_repo.get_by_file_id(file_id, tenant_context).await
    }
}