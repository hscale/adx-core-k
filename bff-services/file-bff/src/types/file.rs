use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Uuid,
    pub name: String,
    pub original_name: String,
    pub mime_type: String,
    pub size: u64,
    pub path: String,
    pub storage_provider: String,
    pub checksum: String,
    pub tenant_id: String,
    pub owner_id: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermissions {
    pub file_id: Uuid,
    pub owner_id: String,
    pub permissions: Vec<FilePermission>,
    pub public_access: bool,
    pub shared_links: Vec<SharedLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilePermission {
    pub user_id: Option<String>,
    pub role: Option<String>,
    pub permission_type: PermissionType,
    pub granted_at: DateTime<Utc>,
    pub granted_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionType {
    Read,
    Write,
    Delete,
    Share,
    Admin,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedLink {
    pub id: Uuid,
    pub file_id: Uuid,
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub access_count: u32,
    pub max_access_count: Option<u32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub provider: String,
    pub region: Option<String>,
    pub bucket: Option<String>,
    pub path: String,
    pub url: Option<String>,
    pub cdn_url: Option<String>,
    pub backup_locations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileUploadProgress {
    pub upload_id: Uuid,
    pub file_name: String,
    pub total_size: u64,
    pub uploaded_size: u64,
    pub progress_percentage: f32,
    pub status: UploadStatus,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UploadStatus {
    Pending,
    Uploading,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedFileData {
    pub metadata: FileMetadata,
    pub permissions: FilePermissions,
    pub storage_info: StorageInfo,
    pub upload_progress: Option<FileUploadProgress>,
    pub thumbnail_url: Option<String>,
    pub preview_url: Option<String>,
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchRequest {
    pub query: Option<String>,
    pub file_types: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub owner_id: Option<String>,
    pub created_after: Option<DateTime<Utc>>,
    pub created_before: Option<DateTime<Utc>>,
    pub size_min: Option<u64>,
    pub size_max: Option<u64>,
    pub page: Option<u32>,
    pub limit: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSearchResponse {
    pub files: Vec<AggregatedFileData>,
    pub total_count: u64,
    pub page: u32,
    pub limit: u32,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadRequest {
    pub file_name: String,
    pub file_size: u64,
    pub mime_type: String,
    pub tags: Option<Vec<String>>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
    pub storage_provider: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub upload_id: Uuid,
    pub upload_url: String,
    pub file_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileShareRequest {
    pub file_id: Uuid,
    pub share_with: Vec<String>, // user IDs or email addresses
    pub permission_type: PermissionType,
    pub expires_at: Option<DateTime<Utc>>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileShareResponse {
    pub shared_link: Option<SharedLink>,
    pub notifications_sent: u32,
    pub success: bool,
}