use axum::{
    extract::{Path, Query, Request, State},
    response::Json,
    routing::get,
    Router,
};
use futures::future::try_join_all;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use crate::{
    middleware::{
        auth::Claims,
        error_handler::{BffError, BffResult},
        tenant::get_tenant_context,
    },
    types::{AggregatedFileData, FileMetadata, FilePermissions, StorageInfo},
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/file/:file_id", get(get_aggregated_file_data))
        .route("/files", get(get_aggregated_files_list))
        .route("/dashboard", get(get_file_dashboard_data))
        .route("/storage-summary", get(get_storage_summary))
        .route("/recent-activity", get(get_recent_file_activity))
        .route("/upload-status", get(get_upload_status_summary))
}

#[derive(Debug, Deserialize)]
struct AggregatedFilesQuery {
    page: Option<u32>,
    limit: Option<u32>,
    include_permissions: Option<bool>,
    include_storage: Option<bool>,
    include_progress: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileDashboardData {
    total_files: u64,
    total_storage_used: u64,
    storage_quota: u64,
    recent_uploads: Vec<serde_json::Value>,
    active_uploads: Vec<serde_json::Value>,
    storage_breakdown: HashMap<String, u64>,
    quota_warnings: Vec<QuotaWarning>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuotaWarning {
    quota_type: String,
    current_usage: u64,
    limit: u64,
    percentage_used: f32,
    warning_level: String, // "info", "warning", "critical"
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageSummary {
    total_files: u64,
    total_size: u64,
    by_file_type: HashMap<String, FileTypeStats>,
    by_storage_provider: HashMap<String, StorageProviderStats>,
    recent_growth: Vec<StorageGrowthPoint>,
}

#[derive(Debug, Serialize, Deserialize)]
struct FileTypeStats {
    count: u64,
    total_size: u64,
    average_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageProviderStats {
    count: u64,
    total_size: u64,
    provider_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct StorageGrowthPoint {
    date: String,
    total_size: u64,
    file_count: u64,
}

async fn get_aggregated_file_data(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    request: Request,
) -> BffResult<Json<AggregatedFileData>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Getting aggregated file data for: {} for tenant: {}", file_id, tenant_context.tenant_id);

    let auth_token = get_auth_token(&request)?;

    // Try to get aggregated data from cache first
    let cache_key = format!("aggregated:file:{}:{}", tenant_context.tenant_id, file_id);
    if let Ok(Some(cached_data)) = state.redis.get::<AggregatedFileData>(&cache_key).await {
        debug!("Returning cached aggregated file data");
        return Ok(Json(cached_data));
    }

    // Fetch data from multiple sources in parallel
    let metadata_future = state.api_client.get_file_metadata(&file_id, &tenant_context.tenant_id, &auth_token);
    let permissions_future = state.api_client.get_file_permissions(&file_id, &tenant_context.tenant_id, &auth_token);
    let storage_future = state.api_client.get_storage_info(&file_id, &tenant_context.tenant_id, &auth_token);

    let (metadata_result, permissions_result, storage_result) = 
        futures::future::try_join3(metadata_future, permissions_future, storage_future).await
        .map_err(BffError::from)?;

    // Parse the results
    let metadata: FileMetadata = serde_json::from_value(metadata_result)
        .map_err(|e| BffError::validation(format!("Invalid metadata format: {}", e)))?;
    
    let permissions: FilePermissions = serde_json::from_value(permissions_result)
        .map_err(|e| BffError::validation(format!("Invalid permissions format: {}", e)))?;
    
    let storage_info: StorageInfo = serde_json::from_value(storage_result)
        .map_err(|e| BffError::validation(format!("Invalid storage info format: {}", e)))?;

    // Check for upload progress if file is being uploaded
    let upload_progress = if metadata.path.contains("uploading") {
        // Try to find upload progress
        match find_upload_progress(&state, &file_id, &tenant_context.tenant_id).await {
            Ok(progress) => progress,
            Err(e) => {
                warn!("Failed to get upload progress: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Generate additional URLs
    let thumbnail_url = generate_thumbnail_url(&metadata, &storage_info);
    let preview_url = generate_preview_url(&metadata, &storage_info);
    let download_url = storage_info.url.clone().unwrap_or_else(|| {
        format!("/api/files/{}/download", file_id)
    });

    let aggregated_data = AggregatedFileData {
        metadata,
        permissions,
        storage_info,
        upload_progress: upload_progress.and_then(|p| serde_json::from_value(p).ok()),
        thumbnail_url,
        preview_url,
        download_url,
    };

    // Cache the aggregated data
    if let Err(e) = state.redis.set(&cache_key, &aggregated_data, Some(300)).await {
        warn!("Failed to cache aggregated file data: {}", e);
    }

    info!("Retrieved aggregated file data for: {}", file_id);
    Ok(Json(aggregated_data))
}

async fn get_aggregated_files_list(
    State(state): State<AppState>,
    Query(query): Query<AggregatedFilesQuery>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting aggregated files list for tenant: {}", tenant_context.tenant_id);

    let auth_token = get_auth_token(&request)?;
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);

    // Build cache key based on query parameters
    let cache_key = format!("aggregated:files:{}:{}:{}:{:?}", 
        tenant_context.tenant_id, page, limit, 
        (query.include_permissions, query.include_storage, query.include_progress)
    );

    if let Ok(Some(cached_data)) = state.redis.get::<serde_json::Value>(&cache_key).await {
        debug!("Returning cached aggregated files list");
        return Ok(Json(cached_data));
    }

    // Get basic file list
    let params = vec![
        ("page", page.to_string()),
        ("limit", limit.to_string()),
    ];
    let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();

    let files_response = state
        .api_client
        .list_files(&tenant_context.tenant_id, &auth_token, &params_ref)
        .await
        .map_err(BffError::from)?;

    let mut files_list = files_response.clone();

    // Enhance with additional data if requested
    if let Some(files_array) = files_list.get_mut("files").and_then(|f| f.as_array_mut()) {
        for file in files_array {
            if let Some(file_id_value) = file.get("id") {
                if let Some(file_id) = file_id_value.as_str() {
                    let file_id_owned = file_id.to_string();
                    
                    // Add permissions if requested
                    if query.include_permissions.unwrap_or(false) {
                        if let Ok(permissions) = state.api_client.get_file_permissions(&file_id_owned, &tenant_context.tenant_id, &auth_token).await {
                            file.as_object_mut().unwrap().insert("permissions".to_string(), permissions);
                        }
                    }

                    // Add storage info if requested
                    if query.include_storage.unwrap_or(false) {
                        if let Ok(storage_info) = state.api_client.get_storage_info(&file_id_owned, &tenant_context.tenant_id, &auth_token).await {
                            file.as_object_mut().unwrap().insert("storage_info".to_string(), storage_info);
                        }
                    }

                    // Add upload progress if requested and applicable
                    if query.include_progress.unwrap_or(false) {
                        if let Ok(Some(progress)) = find_upload_progress(&state, &file_id_owned, &tenant_context.tenant_id).await {
                            file.as_object_mut().unwrap().insert("upload_progress".to_string(), serde_json::to_value(progress)?);
                        }
                    }
                }
            }
        }
    }

    // Cache the result
    if let Err(e) = state.redis.set(&cache_key, &files_list, Some(180)).await {
        warn!("Failed to cache aggregated files list: {}", e);
    }

    info!("Retrieved aggregated files list for tenant: {}", tenant_context.tenant_id);
    Ok(Json(files_list))
}

async fn get_file_dashboard_data(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<FileDashboardData>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting file dashboard data for tenant: {}", tenant_context.tenant_id);

    let cache_key = format!("dashboard:files:{}", tenant_context.tenant_id);
    if let Ok(Some(cached_data)) = state.redis.get::<FileDashboardData>(&cache_key).await {
        debug!("Returning cached dashboard data");
        return Ok(Json(cached_data));
    }

    let auth_token = get_auth_token(&request)?;

    // Fetch dashboard data from multiple sources in parallel
    let files_future = get_files_summary(&state, &tenant_context.tenant_id, &auth_token);
    let uploads_future = get_recent_uploads(&state, &tenant_context.tenant_id, &auth_token);
    let active_uploads_future = get_active_uploads(&state, &tenant_context.tenant_id, &auth_token);

    let (files_summary, recent_uploads, active_uploads) = 
        futures::future::try_join3(files_future, uploads_future, active_uploads_future).await
        .map_err(BffError::from)?;

    // Calculate storage breakdown and quota warnings
    let storage_breakdown = calculate_storage_breakdown(&files_summary);
    let quota_warnings = calculate_quota_warnings(tenant_context, &files_summary);

    let dashboard_data = FileDashboardData {
        total_files: files_summary.get("total_count").and_then(|v| v.as_u64()).unwrap_or(0),
        total_storage_used: files_summary.get("total_size").and_then(|v| v.as_u64()).unwrap_or(0),
        storage_quota: tenant_context.quotas.get("storage_gb").copied().unwrap_or(0) * 1024 * 1024 * 1024,
        recent_uploads,
        active_uploads,
        storage_breakdown,
        quota_warnings,
    };

    // Cache dashboard data
    if let Err(e) = state.redis.set(&cache_key, &dashboard_data, Some(300)).await {
        warn!("Failed to cache dashboard data: {}", e);
    }

    info!("Retrieved file dashboard data for tenant: {}", tenant_context.tenant_id);
    Ok(Json(dashboard_data))
}

async fn get_storage_summary(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<StorageSummary>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting storage summary for tenant: {}", tenant_context.tenant_id);

    let cache_key = format!("storage:summary:{}", tenant_context.tenant_id);
    if let Ok(Some(cached_data)) = state.redis.get::<StorageSummary>(&cache_key).await {
        debug!("Returning cached storage summary");
        return Ok(Json(cached_data));
    }

    // This would typically call analytics endpoints or aggregate data
    // For now, we'll create a mock summary
    let storage_summary = create_mock_storage_summary(&tenant_context.tenant_id);

    // Cache storage summary
    if let Err(e) = state.redis.set(&cache_key, &storage_summary, Some(600)).await {
        warn!("Failed to cache storage summary: {}", e);
    }

    info!("Retrieved storage summary for tenant: {}", tenant_context.tenant_id);
    Ok(Json(storage_summary))
}

async fn get_recent_file_activity(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting recent file activity for tenant: {}", tenant_context.tenant_id);

    let cache_key = format!("activity:recent:{}", tenant_context.tenant_id);
    if let Ok(Some(cached_data)) = state.redis.get::<serde_json::Value>(&cache_key).await {
        debug!("Returning cached recent activity");
        return Ok(Json(cached_data));
    }

    // This would typically call an activity service or audit log
    let recent_activity = json!({
        "activities": [
            {
                "id": "activity-1",
                "type": "file_upload",
                "file_name": "document.pdf",
                "user_email": "user@example.com",
                "timestamp": "2024-01-15T10:30:00Z",
                "status": "completed"
            },
            {
                "id": "activity-2",
                "type": "file_share",
                "file_name": "presentation.pptx",
                "user_email": "user@example.com",
                "timestamp": "2024-01-15T09:15:00Z",
                "status": "completed"
            }
        ],
        "total_count": 2
    });

    // Cache recent activity
    if let Err(e) = state.redis.set(&cache_key, &recent_activity, Some(180)).await {
        warn!("Failed to cache recent activity: {}", e);
    }

    info!("Retrieved recent file activity for tenant: {}", tenant_context.tenant_id);
    Ok(Json(recent_activity))
}

async fn get_upload_status_summary(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting upload status summary for tenant: {}", tenant_context.tenant_id);

    // Get all active uploads for the tenant
    let active_uploads = get_active_uploads(&state, &tenant_context.tenant_id, &get_auth_token(&request)?).await?;

    let summary = json!({
        "active_uploads": active_uploads.len(),
        "uploads": active_uploads,
        "total_progress": calculate_total_progress(&active_uploads)
    });

    info!("Retrieved upload status summary for tenant: {}", tenant_context.tenant_id);
    Ok(Json(summary))
}

// Helper functions

async fn find_upload_progress(
    state: &AppState,
    file_id: &str,
    tenant_id: &str,
) -> BffResult<Option<serde_json::Value>> {
    // Try to find upload progress by file ID
    // This is a simplified implementation
    if let Ok(Some(progress)) = state.redis.get_upload_progress(file_id, tenant_id).await {
        Ok(Some(progress))
    } else {
        Ok(None)
    }
}

fn generate_thumbnail_url(metadata: &FileMetadata, storage_info: &StorageInfo) -> Option<String> {
    // Generate thumbnail URL based on file type and storage provider
    if metadata.mime_type.starts_with("image/") {
        Some(format!("{}/thumbnails/{}", storage_info.cdn_url.as_ref().unwrap_or(&storage_info.path), metadata.id))
    } else {
        None
    }
}

fn generate_preview_url(metadata: &FileMetadata, storage_info: &StorageInfo) -> Option<String> {
    // Generate preview URL for supported file types
    if metadata.mime_type.starts_with("image/") || 
       metadata.mime_type == "application/pdf" ||
       metadata.mime_type.starts_with("text/") {
        Some(format!("{}/previews/{}", storage_info.cdn_url.as_ref().unwrap_or(&storage_info.path), metadata.id))
    } else {
        None
    }
}

async fn get_files_summary(
    state: &AppState,
    tenant_id: &str,
    auth_token: &str,
) -> Result<serde_json::Value, anyhow::Error> {
    // This would typically call a summary endpoint
    Ok(json!({
        "total_count": 150,
        "total_size": 5368709120, // 5GB in bytes
        "by_type": {
            "pdf": 45,
            "image": 67,
            "document": 23,
            "other": 15
        }
    }))
}

async fn get_recent_uploads(
    state: &AppState,
    tenant_id: &str,
    auth_token: &str,
) -> Result<Vec<serde_json::Value>, anyhow::Error> {
    // This would typically query recent uploads
    Ok(vec![
        json!({
            "id": "upload-1",
            "file_name": "document.pdf",
            "size": 1048576,
            "completed_at": "2024-01-15T10:30:00Z"
        })
    ])
}

async fn get_active_uploads(
    state: &AppState,
    tenant_id: &str,
    auth_token: &str,
) -> Result<Vec<serde_json::Value>, anyhow::Error> {
    // This would typically query active uploads
    Ok(vec![
        json!({
            "id": "upload-2",
            "file_name": "large-file.zip",
            "progress": 65.5,
            "status": "uploading"
        })
    ])
}

fn calculate_storage_breakdown(files_summary: &serde_json::Value) -> HashMap<String, u64> {
    let mut breakdown = HashMap::new();
    
    if let Some(by_type) = files_summary.get("by_type").and_then(|v| v.as_object()) {
        for (file_type, count) in by_type {
            // Simplified calculation - in reality this would be based on actual sizes
            let estimated_size = count.as_u64().unwrap_or(0) * 10 * 1024 * 1024; // 10MB average
            breakdown.insert(file_type.clone(), estimated_size);
        }
    }
    
    breakdown
}

fn calculate_quota_warnings(
    tenant_context: &crate::types::TenantContext,
    files_summary: &serde_json::Value,
) -> Vec<QuotaWarning> {
    let mut warnings = Vec::new();
    
    if let Some(storage_quota_gb) = tenant_context.quotas.get("storage_gb") {
        let storage_quota_bytes = storage_quota_gb * 1024 * 1024 * 1024;
        let used_bytes = files_summary.get("total_size").and_then(|v| v.as_u64()).unwrap_or(0);
        let percentage = (used_bytes as f32 / storage_quota_bytes as f32) * 100.0;
        
        if percentage > 90.0 {
            warnings.push(QuotaWarning {
                quota_type: "storage".to_string(),
                current_usage: used_bytes,
                limit: storage_quota_bytes,
                percentage_used: percentage,
                warning_level: "critical".to_string(),
            });
        } else if percentage > 80.0 {
            warnings.push(QuotaWarning {
                quota_type: "storage".to_string(),
                current_usage: used_bytes,
                limit: storage_quota_bytes,
                percentage_used: percentage,
                warning_level: "warning".to_string(),
            });
        }
    }
    
    warnings
}

fn create_mock_storage_summary(tenant_id: &str) -> StorageSummary {
    let mut by_file_type = HashMap::new();
    by_file_type.insert("pdf".to_string(), FileTypeStats {
        count: 45,
        total_size: 450 * 1024 * 1024,
        average_size: 10 * 1024 * 1024,
    });
    by_file_type.insert("image".to_string(), FileTypeStats {
        count: 67,
        total_size: 335 * 1024 * 1024,
        average_size: 5 * 1024 * 1024,
    });

    let mut by_storage_provider = HashMap::new();
    by_storage_provider.insert("s3".to_string(), StorageProviderStats {
        count: 100,
        total_size: 700 * 1024 * 1024,
        provider_name: "Amazon S3".to_string(),
    });

    StorageSummary {
        total_files: 150,
        total_size: 5 * 1024 * 1024 * 1024, // 5GB
        by_file_type,
        by_storage_provider,
        recent_growth: vec![
            StorageGrowthPoint {
                date: "2024-01-01".to_string(),
                total_size: 4 * 1024 * 1024 * 1024,
                file_count: 120,
            },
            StorageGrowthPoint {
                date: "2024-01-15".to_string(),
                total_size: 5 * 1024 * 1024 * 1024,
                file_count: 150,
            },
        ],
    }
}

fn calculate_total_progress(active_uploads: &[serde_json::Value]) -> f32 {
    if active_uploads.is_empty() {
        return 0.0;
    }

    let total_progress: f32 = active_uploads
        .iter()
        .filter_map(|upload| upload.get("progress").and_then(|p| p.as_f64()))
        .map(|p| p as f32)
        .sum();

    total_progress / active_uploads.len() as f32
}

fn get_auth_token(request: &Request) -> BffResult<String> {
    let auth_header = request
        .headers()
        .get("authorization")
        .ok_or_else(|| BffError::authentication("Missing authorization header"))?;

    let auth_str = auth_header
        .to_str()
        .map_err(|_| BffError::authentication("Invalid authorization header"))?;

    if auth_str.starts_with("Bearer ") {
        Ok(auth_str[7..].to_string())
    } else {
        Err(BffError::authentication("Invalid authorization format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_total_progress() {
        let uploads = vec![
            json!({"progress": 50.0}),
            json!({"progress": 75.0}),
            json!({"progress": 25.0}),
        ];

        let total = calculate_total_progress(&uploads);
        assert_eq!(total, 50.0);
    }

    #[test]
    fn test_calculate_total_progress_empty() {
        let uploads = vec![];
        let total = calculate_total_progress(&uploads);
        assert_eq!(total, 0.0);
    }

    #[test]
    fn test_generate_thumbnail_url() {
        use uuid::Uuid;
        use chrono::Utc;

        let metadata = FileMetadata {
            id: Uuid::new_v4(),
            name: "test.jpg".to_string(),
            original_name: "test.jpg".to_string(),
            mime_type: "image/jpeg".to_string(),
            size: 1024,
            path: "/files/test.jpg".to_string(),
            storage_provider: "s3".to_string(),
            checksum: "abc123".to_string(),
            tenant_id: "tenant1".to_string(),
            owner_id: "user1".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            tags: vec![],
            metadata: std::collections::HashMap::new(),
        };

        let storage_info = StorageInfo {
            provider: "s3".to_string(),
            region: Some("us-east-1".to_string()),
            bucket: Some("test-bucket".to_string()),
            path: "/files/test.jpg".to_string(),
            url: Some("https://example.com/test.jpg".to_string()),
            cdn_url: Some("https://cdn.example.com".to_string()),
            backup_locations: vec![],
        };

        let thumbnail_url = generate_thumbnail_url(&metadata, &storage_info);
        assert!(thumbnail_url.is_some());
        assert!(thumbnail_url.unwrap().contains("thumbnails"));
    }
}