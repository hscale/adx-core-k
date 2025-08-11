use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use adx_shared::{Result, Error, TenantContext};
use crate::models::*;

#[async_trait]
pub trait FileRepository: Send + Sync {
    async fn create(&self, file: &CreateFileRequest, tenant_context: &TenantContext, user_id: Uuid) -> Result<File>;
    async fn get_by_id(&self, id: Uuid, tenant_context: &TenantContext) -> Result<Option<File>>;
    async fn update(&self, id: Uuid, updates: &UpdateFileRequest, tenant_context: &TenantContext) -> Result<File>;
    async fn delete(&self, id: Uuid, tenant_context: &TenantContext) -> Result<()>;
    async fn list(&self, tenant_context: &TenantContext, user_id: Option<Uuid>, page: i32, per_page: i32) -> Result<FileListResponse>;
    async fn update_status(&self, id: Uuid, status: FileStatus, tenant_context: &TenantContext) -> Result<()>;
    async fn update_storage_info(&self, id: Uuid, storage_path: &str, checksum: Option<&str>, tenant_context: &TenantContext) -> Result<()>;
}

#[async_trait]
pub trait FilePermissionRepository: Send + Sync {
    async fn create(&self, file_id: Uuid, permission: &CreateFilePermissionRequest, tenant_context: &TenantContext, granted_by: Uuid) -> Result<FilePermission>;
    async fn get_by_file_id(&self, file_id: Uuid, tenant_context: &TenantContext) -> Result<Vec<FilePermission>>;
    async fn delete(&self, id: Uuid, tenant_context: &TenantContext) -> Result<()>;
    async fn check_permission(&self, file_id: Uuid, user_id: Uuid, permission_type: PermissionType, tenant_context: &TenantContext) -> Result<bool>;
}

#[async_trait]
pub trait FileShareRepository: Send + Sync {
    async fn create(&self, file_id: Uuid, share: &CreateFileShareRequest, tenant_context: &TenantContext, created_by: Uuid) -> Result<FileShare>;
    async fn get_by_token(&self, token: &str) -> Result<Option<FileShare>>;
    async fn get_by_file_id(&self, file_id: Uuid, tenant_context: &TenantContext) -> Result<Vec<FileShare>>;
    async fn update_download_count(&self, id: Uuid) -> Result<()>;
    async fn deactivate(&self, id: Uuid, tenant_context: &TenantContext) -> Result<()>;
}

#[async_trait]
pub trait StorageProviderRepository: Send + Sync {
    async fn create(&self, provider: &StorageProvider, tenant_context: &TenantContext) -> Result<StorageProvider>;
    async fn get_by_tenant(&self, tenant_context: &TenantContext) -> Result<Vec<StorageProvider>>;
    async fn get_default(&self, tenant_context: &TenantContext) -> Result<Option<StorageProvider>>;
    async fn update(&self, id: Uuid, updates: serde_json::Value, tenant_context: &TenantContext) -> Result<StorageProvider>;
    async fn set_default(&self, id: Uuid, tenant_context: &TenantContext) -> Result<()>;
}

pub struct PostgresFileRepository {
    pool: PgPool,
}

impl PostgresFileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileRepository for PostgresFileRepository {
    async fn create(&self, file: &CreateFileRequest, tenant_context: &TenantContext, user_id: Uuid) -> Result<File> {
        let id = Uuid::new_v4();
        let storage_path = format!("{}/{}/{}", tenant_context.tenant_id, user_id, id);
        
        let result = sqlx::query_as!(
            File,
            r#"
            INSERT INTO files (
                id, tenant_id, user_id, filename, original_filename, 
                mime_type, file_size, storage_path, storage_provider, 
                status, metadata, is_public
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, tenant_id, user_id, filename, original_filename,
                mime_type, file_size, storage_path, storage_provider,
                status as "status: FileStatus", metadata, checksum, is_public,
                created_at, updated_at
            "#,
            id,
            tenant_context.tenant_id,
            user_id,
            file.filename,
            file.filename, // original_filename same as filename for now
            file.mime_type,
            file.file_size,
            storage_path,
            "local", // default storage provider
            FileStatus::Uploading as FileStatus,
            file.metadata.as_ref().unwrap_or(&serde_json::json!({})),
            file.is_public.unwrap_or(false)
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn get_by_id(&self, id: Uuid, tenant_context: &TenantContext) -> Result<Option<File>> {
        let result = sqlx::query_as!(
            File,
            r#"
            SELECT 
                id, tenant_id, user_id, filename, original_filename,
                mime_type, file_size, storage_path, storage_provider,
                status as "status: FileStatus", metadata, checksum, is_public,
                created_at, updated_at
            FROM files 
            WHERE id = $1 AND tenant_id = $2
            "#,
            id,
            tenant_context.tenant_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn update(&self, id: Uuid, updates: &UpdateFileRequest, tenant_context: &TenantContext) -> Result<File> {
        let result = sqlx::query_as!(
            File,
            r#"
            UPDATE files 
            SET 
                filename = COALESCE($3, filename),
                metadata = COALESCE($4, metadata),
                is_public = COALESCE($5, is_public),
                updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            RETURNING 
                id, tenant_id, user_id, filename, original_filename,
                mime_type, file_size, storage_path, storage_provider,
                status as "status: FileStatus", metadata, checksum, is_public,
                created_at, updated_at
            "#,
            id,
            tenant_context.tenant_id,
            updates.filename,
            updates.metadata,
            updates.is_public
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid, tenant_context: &TenantContext) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE files SET status = $3, updated_at = NOW() WHERE id = $1 AND tenant_id = $2",
            id,
            tenant_context.tenant_id,
            FileStatus::Deleted as FileStatus
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("File not found".to_string()));
        }

        Ok(())
    }

    async fn list(&self, tenant_context: &TenantContext, user_id: Option<Uuid>, page: i32, per_page: i32) -> Result<FileListResponse> {
        let offset = (page - 1) * per_page;
        
        let files = if let Some(user_id) = user_id {
            sqlx::query_as!(
                File,
                r#"
                SELECT 
                    id, tenant_id, user_id, filename, original_filename,
                    mime_type, file_size, storage_path, storage_provider,
                    status as "status: FileStatus", metadata, checksum, is_public,
                    created_at, updated_at
                FROM files 
                WHERE tenant_id = $1 AND user_id = $2 AND status != $3
                ORDER BY created_at DESC
                LIMIT $4 OFFSET $5
                "#,
                tenant_context.tenant_id,
                user_id,
                FileStatus::Deleted as FileStatus,
                per_page as i64,
                offset as i64
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?
        } else {
            sqlx::query_as!(
                File,
                r#"
                SELECT 
                    id, tenant_id, user_id, filename, original_filename,
                    mime_type, file_size, storage_path, storage_provider,
                    status as "status: FileStatus", metadata, checksum, is_public,
                    created_at, updated_at
                FROM files 
                WHERE tenant_id = $1 AND status != $2
                ORDER BY created_at DESC
                LIMIT $3 OFFSET $4
                "#,
                tenant_context.tenant_id,
                FileStatus::Deleted as FileStatus,
                per_page as i64,
                offset as i64
            )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?
        };

        let total_query = if user_id.is_some() {
            sqlx::query!(
                "SELECT COUNT(*) as count FROM files WHERE tenant_id = $1 AND user_id = $2 AND status != $3",
                tenant_context.tenant_id,
                user_id,
                FileStatus::Deleted as FileStatus
            )
        } else {
            sqlx::query!(
                "SELECT COUNT(*) as count FROM files WHERE tenant_id = $1 AND status != $2",
                tenant_context.tenant_id,
                FileStatus::Deleted as FileStatus
            )
        };

        let total = total_query
            .fetch_one(&self.pool)
            .await
            .map_err(|e| Error::Database(e.to_string()))?
            .count
            .unwrap_or(0);

        Ok(FileListResponse {
            files,
            total,
            page,
            per_page,
        })
    }

    async fn update_status(&self, id: Uuid, status: FileStatus, tenant_context: &TenantContext) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE files SET status = $3, updated_at = NOW() WHERE id = $1 AND tenant_id = $2",
            id,
            tenant_context.tenant_id,
            status as FileStatus
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("File not found".to_string()));
        }

        Ok(())
    }

    async fn update_storage_info(&self, id: Uuid, storage_path: &str, checksum: Option<&str>, tenant_context: &TenantContext) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE files SET storage_path = $3, checksum = $4, updated_at = NOW() WHERE id = $1 AND tenant_id = $2",
            id,
            tenant_context.tenant_id,
            storage_path,
            checksum
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("File not found".to_string()));
        }

        Ok(())
    }
}

pub struct PostgresFilePermissionRepository {
    pool: PgPool,
}

impl PostgresFilePermissionRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FilePermissionRepository for PostgresFilePermissionRepository {
    async fn create(&self, file_id: Uuid, permission: &CreateFilePermissionRequest, tenant_context: &TenantContext, granted_by: Uuid) -> Result<FilePermission> {
        let id = Uuid::new_v4();
        
        let result = sqlx::query_as!(
            FilePermission,
            r#"
            INSERT INTO file_permissions (
                id, file_id, tenant_id, user_id, permission_type, granted_by, expires_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING 
                id, file_id, tenant_id, user_id,
                permission_type as "permission_type: PermissionType",
                granted_by, expires_at, created_at
            "#,
            id,
            file_id,
            tenant_context.tenant_id,
            permission.user_id,
            permission.permission_type as PermissionType,
            granted_by,
            permission.expires_at
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn get_by_file_id(&self, file_id: Uuid, tenant_context: &TenantContext) -> Result<Vec<FilePermission>> {
        let result = sqlx::query_as!(
            FilePermission,
            r#"
            SELECT 
                id, file_id, tenant_id, user_id,
                permission_type as "permission_type: PermissionType",
                granted_by, expires_at, created_at
            FROM file_permissions 
            WHERE file_id = $1 AND tenant_id = $2
            AND (expires_at IS NULL OR expires_at > NOW())
            ORDER BY created_at DESC
            "#,
            file_id,
            tenant_context.tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn delete(&self, id: Uuid, tenant_context: &TenantContext) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM file_permissions WHERE id = $1 AND tenant_id = $2",
            id,
            tenant_context.tenant_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Permission not found".to_string()));
        }

        Ok(())
    }

    async fn check_permission(&self, file_id: Uuid, user_id: Uuid, permission_type: PermissionType, tenant_context: &TenantContext) -> Result<bool> {
        let result = sqlx::query!(
            r#"
            SELECT COUNT(*) as count
            FROM file_permissions 
            WHERE file_id = $1 AND tenant_id = $2 AND user_id = $3 
            AND permission_type = $4
            AND (expires_at IS NULL OR expires_at > NOW())
            "#,
            file_id,
            tenant_context.tenant_id,
            user_id,
            permission_type as PermissionType
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result.count.unwrap_or(0) > 0)
    }
}

pub struct PostgresFileShareRepository {
    pool: PgPool,
}

impl PostgresFileShareRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl FileShareRepository for PostgresFileShareRepository {
    async fn create(&self, file_id: Uuid, share: &CreateFileShareRequest, tenant_context: &TenantContext, created_by: Uuid) -> Result<FileShare> {
        let id = Uuid::new_v4();
        let share_token = format!("share_{}", Uuid::new_v4().to_string().replace('-', ""));
        
        let password_hash = if let Some(password) = &share.password {
            Some(bcrypt::hash(password, bcrypt::DEFAULT_COST).map_err(|e| Error::Internal(e.to_string()))?)
        } else {
            None
        };

        let result = sqlx::query_as!(
            FileShare,
            r#"
            INSERT INTO file_shares (
                id, file_id, tenant_id, share_token, share_type, 
                password_hash, allowed_emails, download_limit, expires_at, created_by
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING 
                id, file_id, tenant_id, share_token,
                share_type as "share_type: ShareType",
                password_hash, allowed_emails, download_limit, download_count,
                expires_at, is_active, created_by, created_at, updated_at
            "#,
            id,
            file_id,
            tenant_context.tenant_id,
            share_token,
            share.share_type as ShareType,
            password_hash,
            share.allowed_emails.as_deref(),
            share.download_limit,
            share.expires_at,
            created_by
        )
        .fetch_one(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn get_by_token(&self, token: &str) -> Result<Option<FileShare>> {
        let result = sqlx::query_as!(
            FileShare,
            r#"
            SELECT 
                id, file_id, tenant_id, share_token,
                share_type as "share_type: ShareType",
                password_hash, allowed_emails, download_limit, download_count,
                expires_at, is_active, created_by, created_at, updated_at
            FROM file_shares 
            WHERE share_token = $1 AND is_active = true
            AND (expires_at IS NULL OR expires_at > NOW())
            "#,
            token
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn get_by_file_id(&self, file_id: Uuid, tenant_context: &TenantContext) -> Result<Vec<FileShare>> {
        let result = sqlx::query_as!(
            FileShare,
            r#"
            SELECT 
                id, file_id, tenant_id, share_token,
                share_type as "share_type: ShareType",
                password_hash, allowed_emails, download_limit, download_count,
                expires_at, is_active, created_by, created_at, updated_at
            FROM file_shares 
            WHERE file_id = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            "#,
            file_id,
            tenant_context.tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result)
    }

    async fn update_download_count(&self, id: Uuid) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE file_shares SET download_count = download_count + 1, updated_at = NOW() WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Share not found".to_string()));
        }

        Ok(())
    }

    async fn deactivate(&self, id: Uuid, tenant_context: &TenantContext) -> Result<()> {
        let result = sqlx::query!(
            "UPDATE file_shares SET is_active = false, updated_at = NOW() WHERE id = $1 AND tenant_id = $2",
            id,
            tenant_context.tenant_id
        )
        .execute(&self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Share not found".to_string()));
        }

        Ok(())
    }
}