use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use tracing::{debug, info};
use uuid::Uuid;

use crate::{
    middleware::{
        auth::{require_permission, Claims},
        error_handler::{BffError, BffResult},
        tenant::{get_tenant_context, get_tenant_id},
    },
    services::redis::generate_search_hash,
    types::{
        FileSearchRequest, FileSearchResponse, FileUploadRequest, FileUploadResponse,
        FileShareRequest, FileShareResponse, PaginationParams,
    },
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_files))
        .route("/search", post(search_files))
        .route("/upload", post(initiate_upload))
        .route("/:file_id", get(get_file))
        .route("/:file_id", put(update_file))
        .route("/:file_id", delete(delete_file))
        .route("/:file_id/permissions", get(get_file_permissions))
        .route("/:file_id/permissions", put(update_file_permissions))
        .route("/:file_id/share", post(share_file))
        .route("/:file_id/download", get(get_download_url))
        .route("/uploads/:upload_id/progress", get(get_upload_progress))
        .route("/uploads/:upload_id/cancel", post(cancel_upload))
}

#[derive(Debug, Deserialize)]
struct ListFilesQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    owner_id: Option<String>,
    file_type: Option<String>,
    tags: Option<String>, // Comma-separated tags
}

async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<ListFilesQuery>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Listing files for tenant: {}", tenant_context.tenant_id);

    // Build query parameters for the file service
    let mut params = vec![
        ("page", query.pagination.page.unwrap_or(1).to_string()),
        ("limit", query.pagination.limit.unwrap_or(20).to_string()),
    ];

    if let Some(sort_by) = &query.pagination.sort_by {
        params.push(("sort_by", sort_by.clone()));
    }
    if let Some(sort_order) = &query.pagination.sort_order {
        params.push(("sort_order", sort_order.clone()));
    }
    if let Some(owner_id) = &query.owner_id {
        params.push(("owner_id", owner_id.clone()));
    }
    if let Some(file_type) = &query.file_type {
        params.push(("file_type", file_type.clone()));
    }
    if let Some(tags) = &query.tags {
        params.push(("tags", tags.clone()));
    }

    // Convert params to the format expected by the API client
    let params_ref: Vec<(&str, &str)> = params.iter().map(|(k, v)| (k.as_ref(), v.as_ref())).collect();

    // Try to get from cache first
    let cache_key = format!("list_files_{}_{:?}", tenant_context.tenant_id, params);
    if let Ok(Some(cached_result)) = state.redis.get::<serde_json::Value>(&cache_key).await {
        debug!("Returning cached file list");
        return Ok(Json(cached_result));
    }

    // Fetch from file service
    let files = state
        .api_client
        .list_files(&tenant_context.tenant_id, &get_auth_token(&request)?, &params_ref)
        .await
        .map_err(BffError::from)?;

    // Cache the result
    if let Err(e) = state.redis.set(&cache_key, &files, Some(300)).await {
        debug!("Failed to cache file list: {}", e);
    }

    info!("Listed files for tenant: {}", tenant_context.tenant_id);
    Ok(Json(files))
}

async fn search_files(
    State(state): State<AppState>,
    Json(search_request): Json<FileSearchRequest>,
    request: Request,
) -> BffResult<Json<FileSearchResponse>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Searching files for tenant: {}", tenant_context.tenant_id);

    // Generate cache key from search parameters
    let search_json = serde_json::to_value(&search_request)?;
    let search_hash = generate_search_hash(&search_json);

    // Try to get from cache first
    if let Ok(Some(cached_result)) = state.redis.get_cached_search_results(&search_hash, &tenant_context.tenant_id).await {
        debug!("Returning cached search results");
        let cached_response: FileSearchResponse = serde_json::from_value(cached_result)?;
        return Ok(Json(cached_response));
    }

    // Search files through file service
    let search_results = state
        .api_client
        .search_files(&search_json, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Parse the response into our typed structure
    let response: FileSearchResponse = serde_json::from_value(search_results)
        .map_err(|e| BffError::validation(format!("Invalid search response format: {}", e)))?;

    // Cache the search results
    if let Err(e) = state.redis.cache_search_results(&search_hash, &tenant_context.tenant_id, &serde_json::to_value(&response)?, Some(300)).await {
        debug!("Failed to cache search results: {}", e);
    }

    info!("Searched files for tenant: {} with {} results", tenant_context.tenant_id, response.files.len());
    Ok(Json(response))
}

async fn get_file(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Getting file: {} for tenant: {}", file_id, tenant_context.tenant_id);

    // Try to get from cache first
    if let Ok(Some(cached_metadata)) = state.redis.get_cached_file_metadata(&file_id, &tenant_context.tenant_id).await {
        debug!("Returning cached file metadata");
        return Ok(Json(cached_metadata));
    }

    // Fetch file metadata from file service
    let file_metadata = state
        .api_client
        .get_file_metadata(&file_id, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Cache the metadata
    if let Err(e) = state.redis.cache_file_metadata(&file_id, &tenant_context.tenant_id, &file_metadata, Some(600)).await {
        debug!("Failed to cache file metadata: {}", e);
    }

    info!("Retrieved file metadata for: {}", file_id);
    Ok(Json(file_metadata))
}

async fn update_file(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Json(update_data): Json<serde_json::Value>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Updating file: {} for tenant: {}", file_id, tenant_context.tenant_id);

    // This would typically initiate a workflow for complex file updates
    let workflow_input = json!({
        "file_id": file_id,
        "updates": update_data,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id
    });

    let workflow_result = state
        .api_client
        .initiate_workflow("update_file", &workflow_input, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Invalidate cache
    if let Err(e) = state.redis.invalidate_file_cache(&file_id, &tenant_context.tenant_id).await {
        debug!("Failed to invalidate file cache: {}", e);
    }

    info!("Initiated file update workflow for: {}", file_id);
    Ok(Json(workflow_result))
}

async fn delete_file(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Deleting file: {} for tenant: {}", file_id, tenant_context.tenant_id);

    // Initiate file deletion workflow
    let workflow_input = json!({
        "file_id": file_id,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id,
        "soft_delete": true
    });

    let workflow_result = state
        .api_client
        .initiate_workflow("delete_file", &workflow_input, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Invalidate cache
    if let Err(e) = state.redis.invalidate_file_cache(&file_id, &tenant_context.tenant_id).await {
        debug!("Failed to invalidate file cache: {}", e);
    }

    info!("Initiated file deletion workflow for: {}", file_id);
    Ok(Json(workflow_result))
}

async fn get_file_permissions(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting file permissions: {} for tenant: {}", file_id, tenant_context.tenant_id);

    // Try to get from cache first
    if let Ok(Some(cached_permissions)) = state.redis.get_cached_file_permissions(&file_id, &tenant_context.tenant_id).await {
        debug!("Returning cached file permissions");
        return Ok(Json(cached_permissions));
    }

    // Fetch file permissions from file service
    let permissions = state
        .api_client
        .get_file_permissions(&file_id, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Cache the permissions
    if let Err(e) = state.redis.cache_file_permissions(&file_id, &tenant_context.tenant_id, &permissions, Some(300)).await {
        debug!("Failed to cache file permissions: {}", e);
    }

    info!("Retrieved file permissions for: {}", file_id);
    Ok(Json(permissions))
}

async fn update_file_permissions(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Json(permissions_data): Json<serde_json::Value>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Updating file permissions: {} for tenant: {}", file_id, tenant_context.tenant_id);

    // Initiate permissions update workflow
    let workflow_input = json!({
        "file_id": file_id,
        "permissions": permissions_data,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id
    });

    let workflow_result = state
        .api_client
        .initiate_workflow("update_file_permissions", &workflow_input, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Invalidate permissions cache
    let cache_key = format!("file:permissions:{}:{}", tenant_context.tenant_id, file_id);
    if let Err(e) = state.redis.delete(&cache_key).await {
        debug!("Failed to invalidate permissions cache: {}", e);
    }

    info!("Initiated permissions update workflow for: {}", file_id);
    Ok(Json(workflow_result))
}

async fn share_file(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    Json(share_request): Json<FileShareRequest>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Sharing file: {} for tenant: {}", file_id, tenant_context.tenant_id);

    // Initiate file sharing workflow
    let workflow_input = json!({
        "file_id": file_id,
        "share_request": share_request,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id
    });

    let workflow_result = state
        .api_client
        .initiate_workflow("share_file", &workflow_input, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    info!("Initiated file sharing workflow for: {}", file_id);
    Ok(Json(workflow_result))
}

async fn get_download_url(
    State(state): State<AppState>,
    Path(file_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting download URL for file: {} for tenant: {}", file_id, tenant_context.tenant_id);

    // Get storage info (which includes download URL)
    let storage_info = state
        .api_client
        .get_storage_info(&file_id, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    info!("Retrieved download URL for file: {}", file_id);
    Ok(Json(storage_info))
}

async fn initiate_upload(
    State(state): State<AppState>,
    Json(upload_request): Json<FileUploadRequest>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Initiating file upload for tenant: {}", tenant_context.tenant_id);

    // Initiate file upload workflow
    let workflow_input = json!({
        "upload_request": upload_request,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id
    });

    let workflow_result = state
        .api_client
        .initiate_workflow("file_upload", &workflow_input, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    info!("Initiated file upload workflow for tenant: {}", tenant_context.tenant_id);
    Ok(Json(workflow_result))
}

async fn get_upload_progress(
    State(state): State<AppState>,
    Path(upload_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting upload progress for: {} for tenant: {}", upload_id, tenant_context.tenant_id);

    // Try to get from cache first
    if let Ok(Some(cached_progress)) = state.redis.get_upload_progress(&upload_id, &tenant_context.tenant_id).await {
        debug!("Returning cached upload progress");
        return Ok(Json(cached_progress));
    }

    // Fetch upload progress from file service
    let progress = state
        .api_client
        .get_upload_progress(&upload_id, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Cache the progress (short TTL since it changes frequently)
    if let Err(e) = state.redis.set_upload_progress(&upload_id, &tenant_context.tenant_id, &progress, Some(30)).await {
        debug!("Failed to cache upload progress: {}", e);
    }

    info!("Retrieved upload progress for: {}", upload_id);
    Ok(Json(progress))
}

async fn cancel_upload(
    State(state): State<AppState>,
    Path(upload_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Cancelling upload: {} for tenant: {}", upload_id, tenant_context.tenant_id);

    // Initiate upload cancellation workflow
    let workflow_input = json!({
        "upload_id": upload_id,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id
    });

    let workflow_result = state
        .api_client
        .initiate_workflow("cancel_upload", &workflow_input, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Remove upload progress from cache
    if let Err(e) = state.redis.delete(&format!("upload:progress:{}:{}", tenant_context.tenant_id, upload_id)).await {
        debug!("Failed to remove upload progress from cache: {}", e);
    }

    info!("Initiated upload cancellation workflow for: {}", upload_id);
    Ok(Json(workflow_result))
}

// Helper function to extract auth token from request
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
    use axum::http::{HeaderMap, HeaderValue};

    #[test]
    fn test_get_auth_token() {
        let mut headers = HeaderMap::new();
        headers.insert("authorization", HeaderValue::from_static("Bearer test-token"));
        
        // Create a minimal request for testing
        let request = Request::builder()
            .header("authorization", "Bearer test-token")
            .body(())
            .unwrap();

        let token = get_auth_token(&request).unwrap();
        assert_eq!(token, "test-token");
    }

    #[test]
    fn test_get_auth_token_invalid_format() {
        let request = Request::builder()
            .header("authorization", "Invalid test-token")
            .body(())
            .unwrap();

        let result = get_auth_token(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_auth_token_missing_header() {
        let request = Request::builder()
            .body(())
            .unwrap();

        let result = get_auth_token(&request);
        assert!(result.is_err());
    }
}