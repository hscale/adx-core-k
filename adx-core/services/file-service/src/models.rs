use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct File {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub filename: String,
    pub original_filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub storage_path: String,
    pub storage_provider: String,
    pub status: FileStatus,
    pub metadata: serde_json::Value,
    pub checksum: Option<String>,
    pub is_public: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "file_status", rename_all = "lowercase")]
pub enum FileStatus {
    Uploading,
    Processing,
    Ready,
    Failed,
    Deleted,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FilePermission {
    pub id: Uuid,
    pub file_id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub permission_type: PermissionType,
    pub granted_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum PermissionType {
    #[sqlx(rename = "read")]
    Read,
    #[sqlx(rename = "write")]
    Write,
    #[sqlx(rename = "admin")]
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FileShare {
    pub id: Uuid,
    pub file_id: Uuid,
    pub tenant_id: Uuid,
    pub share_token: String,
    pub share_type: ShareType,
    pub password_hash: Option<String>,
    pub allowed_emails: Option<Vec<String>>,
    pub download_limit: Option<i32>,
    pub download_count: i32,
    pub expires_at: Option<DateTime<Utc>>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum ShareType {
    #[sqlx(rename = "public")]
    Public,
    #[sqlx(rename = "password")]
    Password,
    #[sqlx(rename = "email")]
    Email,
    #[sqlx(rename = "time_limited")]
    TimeLimited,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct StorageProvider {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub provider_name: String,
    pub provider_type: StorageProviderType,
    pub configuration: serde_json::Value,
    pub is_default: bool,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar")]
pub enum StorageProviderType {
    #[sqlx(rename = "local")]
    Local,
    #[sqlx(rename = "s3")]
    S3,
    #[sqlx(rename = "gcs")]
    Gcs,
    #[sqlx(rename = "azure")]
    Azure,
    #[sqlx(rename = "ftp")]
    Ftp,
}

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFileRequest {
    pub filename: String,
    pub mime_type: String,
    pub file_size: i64,
    pub metadata: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateFileRequest {
    pub filename: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub is_public: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFileShareRequest {
    pub share_type: ShareType,
    pub password: Option<String>,
    pub allowed_emails: Option<Vec<String>>,
    pub download_limit: Option<i32>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFilePermissionRequest {
    pub user_id: Option<Uuid>,
    pub permission_type: PermissionType,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub file_id: Uuid,
    pub upload_url: Option<String>,
    pub status: FileStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub files: Vec<File>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileDownloadResponse {
    pub download_url: String,
    pub expires_at: DateTime<Utc>,
}

// Storage configuration types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S3Config {
    pub bucket: String,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcsConfig {
    pub bucket: String,
    pub project_id: String,
    pub credentials_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureConfig {
    pub account_name: String,
    pub account_key: String,
    pub container_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalConfig {
    pub base_path: String,
    pub url_prefix: String,
}