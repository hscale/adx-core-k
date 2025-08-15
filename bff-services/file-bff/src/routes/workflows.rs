use axum::{
    extract::{Path, Query, Request, State},
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info};

use crate::{
    middleware::{
        auth::Claims,
        error_handler::{BffError, BffResult},
        tenant::get_tenant_context,
    },
    types::{
        BulkFileOperationWorkflowInput, FileCleanupWorkflowInput, FileMigrationWorkflowInput,
        FileProcessingWorkflowInput, FileUploadWorkflowInput, WorkflowOptions, WorkflowRequest,
    },
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/file-upload", post(initiate_file_upload_workflow))
        .route("/file-processing", post(initiate_file_processing_workflow))
        .route("/file-migration", post(initiate_file_migration_workflow))
        .route("/bulk-file-operation", post(initiate_bulk_file_operation_workflow))
        .route("/file-cleanup", post(initiate_file_cleanup_workflow))
        .route("/:operation_id/status", get(get_workflow_status))
        .route("/:operation_id/cancel", post(cancel_workflow))
        .route("/:operation_id/stream", get(stream_workflow_progress))
}

#[derive(Debug, Deserialize)]
struct WorkflowStatusQuery {
    include_history: Option<bool>,
    include_progress: Option<bool>,
}

async fn initiate_file_upload_workflow(
    State(state): State<AppState>,
    Json(workflow_request): Json<WorkflowRequest<FileUploadWorkflowInput>>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Initiating file upload workflow for tenant: {}", tenant_context.tenant_id);

    // Validate file upload request
    validate_file_upload_input(&workflow_request.input)?;

    // Check tenant quotas and permissions
    check_file_upload_permissions(claims, tenant_context)?;

    // Prepare workflow input with additional context
    let enhanced_input = json!({
        "file_upload_input": workflow_request.input,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id,
        "user_email": claims.user_email,
        "tenant_features": tenant_context.features,
        "options": workflow_request.options
    });

    // Initiate workflow through API Gateway
    let workflow_result = state
        .api_client
        .initiate_workflow(
            "file_upload_workflow",
            &enhanced_input,
            &tenant_context.tenant_id,
            &get_auth_token(&request)?,
        )
        .await
        .map_err(BffError::from)?;

    info!("Initiated file upload workflow for tenant: {}", tenant_context.tenant_id);
    Ok(Json(workflow_result))
}

async fn initiate_file_processing_workflow(
    State(state): State<AppState>,
    Json(workflow_request): Json<WorkflowRequest<FileProcessingWorkflowInput>>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Initiating file processing workflow for tenant: {}", tenant_context.tenant_id);

    // Validate processing options based on tenant features
    validate_processing_options(&workflow_request.input, tenant_context)?;

    let enhanced_input = json!({
        "processing_input": workflow_request.input,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id,
        "tenant_features": tenant_context.features,
        "options": workflow_request.options
    });

    let workflow_result = state
        .api_client
        .initiate_workflow(
            "file_processing_workflow",
            &enhanced_input,
            &tenant_context.tenant_id,
            &get_auth_token(&request)?,
        )
        .await
        .map_err(BffError::from)?;

    info!("Initiated file processing workflow for tenant: {}", tenant_context.tenant_id);
    Ok(Json(workflow_result))
}

async fn initiate_file_migration_workflow(
    State(state): State<AppState>,
    Json(workflow_request): Json<WorkflowRequest<FileMigrationWorkflowInput>>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Initiating file migration workflow for tenant: {}", tenant_context.tenant_id);

    // Check if user has admin permissions for migration
    if !claims.user_roles.contains(&"admin".to_string()) {
        return Err(BffError::authorization("File migration requires admin permissions"));
    }

    // Validate migration request
    validate_migration_input(&workflow_request.input)?;

    let enhanced_input = json!({
        "migration_input": workflow_request.input,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id,
        "options": workflow_request.options
    });

    let workflow_result = state
        .api_client
        .initiate_workflow(
            "file_migration_workflow",
            &enhanced_input,
            &tenant_context.tenant_id,
            &get_auth_token(&request)?,
        )
        .await
        .map_err(BffError::from)?;

    info!("Initiated file migration workflow for tenant: {}", tenant_context.tenant_id);
    Ok(Json(workflow_result))
}

async fn initiate_bulk_file_operation_workflow(
    State(state): State<AppState>,
    Json(workflow_request): Json<WorkflowRequest<BulkFileOperationWorkflowInput>>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Initiating bulk file operation workflow for tenant: {}", tenant_context.tenant_id);

    // Validate bulk operation limits
    if workflow_request.input.file_ids.len() > 1000 {
        return Err(BffError::validation("Bulk operations limited to 1000 files"));
    }

    let enhanced_input = json!({
        "bulk_operation_input": workflow_request.input,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id,
        "options": workflow_request.options
    });

    let workflow_result = state
        .api_client
        .initiate_workflow(
            "bulk_file_operation_workflow",
            &enhanced_input,
            &tenant_context.tenant_id,
            &get_auth_token(&request)?,
        )
        .await
        .map_err(BffError::from)?;

    info!("Initiated bulk file operation workflow for tenant: {}", tenant_context.tenant_id);
    Ok(Json(workflow_result))
}

async fn initiate_file_cleanup_workflow(
    State(state): State<AppState>,
    Json(workflow_request): Json<WorkflowRequest<FileCleanupWorkflowInput>>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Initiating file cleanup workflow for tenant: {}", tenant_context.tenant_id);

    // Check if user has admin permissions for cleanup
    if !claims.user_roles.contains(&"admin".to_string()) {
        return Err(BffError::authorization("File cleanup requires admin permissions"));
    }

    let enhanced_input = json!({
        "cleanup_input": workflow_request.input,
        "user_id": claims.sub,
        "tenant_id": tenant_context.tenant_id,
        "options": workflow_request.options
    });

    let workflow_result = state
        .api_client
        .initiate_workflow(
            "file_cleanup_workflow",
            &enhanced_input,
            &tenant_context.tenant_id,
            &get_auth_token(&request)?,
        )
        .await
        .map_err(BffError::from)?;

    info!("Initiated file cleanup workflow for tenant: {}", tenant_context.tenant_id);
    Ok(Json(workflow_result))
}

async fn get_workflow_status(
    State(state): State<AppState>,
    Path(operation_id): Path<String>,
    Query(query): Query<WorkflowStatusQuery>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Getting workflow status for: {} for tenant: {}", operation_id, tenant_context.tenant_id);

    // Try to get from cache first
    if let Ok(Some(cached_status)) = state.redis.get_cached_workflow_status(&operation_id, &tenant_context.tenant_id).await {
        debug!("Returning cached workflow status");
        return Ok(Json(cached_status));
    }

    // Fetch workflow status from API Gateway
    let workflow_status = state
        .api_client
        .get_workflow_status(&operation_id, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Cache the status (short TTL for active workflows)
    let ttl = if is_workflow_active(&workflow_status) { 30 } else { 300 };
    if let Err(e) = state.redis.cache_workflow_status(&operation_id, &tenant_context.tenant_id, &workflow_status, Some(ttl)).await {
        debug!("Failed to cache workflow status: {}", e);
    }

    info!("Retrieved workflow status for: {}", operation_id);
    Ok(Json(workflow_status))
}

async fn cancel_workflow(
    State(state): State<AppState>,
    Path(operation_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;
    
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication"))?;

    debug!("Cancelling workflow: {} for tenant: {}", operation_id, tenant_context.tenant_id);

    // Cancel workflow through API Gateway
    let cancel_result = state
        .api_client
        .cancel_workflow(&operation_id, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    // Invalidate cached workflow status
    if let Err(e) = state.redis.delete(&format!("workflow:status:{}:{}", tenant_context.tenant_id, operation_id)).await {
        debug!("Failed to invalidate workflow status cache: {}", e);
    }

    info!("Cancelled workflow: {} for user: {}", operation_id, claims.sub);
    Ok(Json(cancel_result))
}

async fn stream_workflow_progress(
    State(state): State<AppState>,
    Path(operation_id): Path<String>,
    request: Request,
) -> BffResult<Json<serde_json::Value>> {
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    debug!("Streaming workflow progress for: {} for tenant: {}", operation_id, tenant_context.tenant_id);

    // For now, return the current status. In a full implementation, this would
    // establish a WebSocket connection or Server-Sent Events stream
    let workflow_status = state
        .api_client
        .get_workflow_status(&operation_id, &tenant_context.tenant_id, &get_auth_token(&request)?)
        .await
        .map_err(BffError::from)?;

    Ok(Json(json!({
        "operation_id": operation_id,
        "stream_type": "status",
        "data": workflow_status
    })))
}

// Validation functions
fn validate_file_upload_input(input: &FileUploadWorkflowInput) -> BffResult<()> {
    if input.file_name.is_empty() {
        return Err(BffError::validation("File name cannot be empty"));
    }

    if input.file_size == 0 {
        return Err(BffError::validation("File size must be greater than 0"));
    }

    if input.file_size > 5 * 1024 * 1024 * 1024 { // 5GB limit
        return Err(BffError::validation("File size exceeds maximum limit of 5GB"));
    }

    if input.mime_type.is_empty() {
        return Err(BffError::validation("MIME type cannot be empty"));
    }

    Ok(())
}

fn validate_processing_options(
    input: &FileProcessingWorkflowInput,
    tenant_context: &crate::types::TenantContext,
) -> BffResult<()> {
    // Check if tenant has required features for processing options
    if input.processing_options.generate_thumbnails && !tenant_context.features.contains(&"thumbnail_generation".to_string()) {
        return Err(BffError::authorization("Thumbnail generation not available for this tenant"));
    }

    if input.processing_options.ocr_processing && !tenant_context.features.contains(&"ocr_processing".to_string()) {
        return Err(BffError::authorization("OCR processing not available for this tenant"));
    }

    if input.processing_options.virus_scan && !tenant_context.features.contains(&"virus_scanning".to_string()) {
        return Err(BffError::authorization("Virus scanning not available for this tenant"));
    }

    Ok(())
}

fn validate_migration_input(input: &FileMigrationWorkflowInput) -> BffResult<()> {
    if input.file_ids.is_empty() {
        return Err(BffError::validation("File IDs list cannot be empty"));
    }

    if input.file_ids.len() > 10000 {
        return Err(BffError::validation("Migration limited to 10,000 files per operation"));
    }

    if input.source_provider == input.target_provider {
        return Err(BffError::validation("Source and target providers must be different"));
    }

    Ok(())
}

fn check_file_upload_permissions(
    claims: &Claims,
    tenant_context: &crate::types::TenantContext,
) -> BffResult<()> {
    // Check if user has file upload permission
    if !claims.permissions.contains(&"file:write".to_string()) && 
       !claims.user_roles.contains(&"admin".to_string()) {
        return Err(BffError::authorization("User lacks file upload permissions"));
    }

    // Check if tenant has file upload feature
    if !tenant_context.features.contains(&"file_upload".to_string()) {
        return Err(BffError::authorization("File upload not available for this tenant"));
    }

    // Check storage quota (simplified check)
    if let Some(storage_quota) = tenant_context.quotas.get("storage_gb") {
        if *storage_quota == 0 {
            return Err(BffError::authorization("Storage quota exceeded"));
        }
    }

    Ok(())
}

fn is_workflow_active(workflow_status: &serde_json::Value) -> bool {
    if let Some(status) = workflow_status.get("status").and_then(|s| s.as_str()) {
        matches!(status, "pending" | "running")
    } else {
        false
    }
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
    use uuid::Uuid;

    #[test]
    fn test_validate_file_upload_input() {
        let valid_input = FileUploadWorkflowInput {
            file_name: "test.txt".to_string(),
            file_size: 1024,
            mime_type: "text/plain".to_string(),
            storage_provider: None,
            virus_scan: true,
            generate_thumbnails: false,
            extract_metadata: true,
        };

        assert!(validate_file_upload_input(&valid_input).is_ok());

        let invalid_input = FileUploadWorkflowInput {
            file_name: "".to_string(),
            file_size: 0,
            mime_type: "".to_string(),
            storage_provider: None,
            virus_scan: true,
            generate_thumbnails: false,
            extract_metadata: true,
        };

        assert!(validate_file_upload_input(&invalid_input).is_err());
    }

    #[test]
    fn test_validate_migration_input() {
        let valid_input = FileMigrationWorkflowInput {
            file_ids: vec![Uuid::new_v4()],
            source_provider: "s3".to_string(),
            target_provider: "gcs".to_string(),
            preserve_urls: true,
            verify_integrity: true,
        };

        assert!(validate_migration_input(&valid_input).is_ok());

        let invalid_input = FileMigrationWorkflowInput {
            file_ids: vec![],
            source_provider: "s3".to_string(),
            target_provider: "s3".to_string(),
            preserve_urls: true,
            verify_integrity: true,
        };

        assert!(validate_migration_input(&invalid_input).is_err());
    }

    #[test]
    fn test_is_workflow_active() {
        let active_status = serde_json::json!({
            "status": "running",
            "progress": 50
        });

        assert!(is_workflow_active(&active_status));

        let completed_status = serde_json::json!({
            "status": "completed",
            "result": {}
        });

        assert!(!is_workflow_active(&completed_status));
    }
}