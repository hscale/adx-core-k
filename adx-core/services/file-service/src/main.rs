use adx_shared::{init_tracing, RequestContext, StandardWorkflowInput, WorkflowContext};
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

mod workflows;
use workflows::*;

#[derive(Clone)]
pub struct AppState {
    // Add your state here (database pool, storage client, etc.)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileMetadata {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub filename: String,
    pub content_type: String,
    pub size: i64,
    pub storage_path: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileUploadResponse {
    pub file_id: Uuid,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileShareRequest {
    pub share_type: String,
    pub recipients: Vec<String>,
    pub permissions: FileSharePermissions,
    pub expiration_hours: Option<i32>,
    pub notify_recipients: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileSharePermissions {
    pub can_view: bool,
    pub can_download: bool,
    pub can_comment: bool,
    pub can_share: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileShareResponse {
    pub share_id: Uuid,
    pub share_url: String,
    pub notifications_sent: i32,
    pub message: String,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let app_state = AppState {};

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/files", post(upload_file))
        .route("/api/v1/files/:file_id", get(get_file))
        .route("/api/v1/files/:file_id", delete(delete_file))
        .route("/api/v1/files/:file_id/metadata", get(get_file_metadata))
        .route("/api/v1/files/:file_id/share", post(share_file))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8083").await.unwrap();
    tracing::info!(
        "File Service listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "File Service OK"
}

// File upload endpoint - Uses Temporal workflow for complex processing
pub async fn upload_file(
    State(_state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<FileUploadResponse>, StatusCode> {
    // Extract multipart data
    let mut filename = String::new();
    let mut content_type = String::new();
    let mut file_data = Vec::new();
    let mut tenant_id = None;
    let mut user_id = None;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let name = field.name().unwrap_or("unknown");

        match name {
            "file" => {
                filename = field.file_name().unwrap_or("unknown").to_string();
                content_type = field
                    .content_type()
                    .unwrap_or("application/octet-stream")
                    .to_string();
                file_data = field
                    .bytes()
                    .await
                    .map_err(|_| StatusCode::BAD_REQUEST)?
                    .to_vec();
            }
            "tenant_id" => {
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                tenant_id = Some(
                    std::str::from_utf8(&data)
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .parse::<Uuid>()
                        .map_err(|_| StatusCode::BAD_REQUEST)?,
                );
            }
            "user_id" => {
                let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
                user_id = Some(
                    std::str::from_utf8(&data)
                        .map_err(|_| StatusCode::BAD_REQUEST)?
                        .parse::<Uuid>()
                        .map_err(|_| StatusCode::BAD_REQUEST)?,
                );
            }
            _ => continue,
        }
    }

    if filename.is_empty() || file_data.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Create workflow context
    let context = WorkflowContext::new(
        tenant_id.unwrap_or_else(|| Uuid::new_v4()), // Default tenant for demo
        user_id,
    );

    // Create workflow input following Temporal-First Principle
    let upload_data = FileUploadData {
        filename: filename.clone(),
        content_type: content_type.clone(),
        size: file_data.len() as i64,
        content: file_data,
        storage_provider: "s3".to_string(),
        enable_ai_processing: true, // Enable AI processing for complex workflow
        enable_virus_scan: true,    // Enable virus scanning for security
    };

    let workflow_input = StandardWorkflowInput {
        context,
        data: upload_data,
    };

    // Execute Temporal workflow for file upload
    match file_upload_workflow(workflow_input).await {
        Ok(result) => {
            tracing::info!(
                "File upload workflow completed: {:?}",
                result.result.file_id
            );
            Ok(Json(FileUploadResponse {
                file_id: result.result.file_id,
                message: format!("File '{}' uploaded and processed successfully", filename),
            }))
        }
        Err(error) => {
            tracing::error!("File upload workflow failed: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Get file endpoint
pub async fn get_file(
    State(_state): State<AppState>,
    Path(file_id): Path<Uuid>,
) -> Result<String, StatusCode> {
    // TODO: Implement file retrieval logic
    // - Validate file exists and user has access
    // - Stream file from storage

    tracing::info!("Retrieving file: {}", file_id);
    Ok(format!("File content for {}", file_id))
}

// Delete file endpoint
pub async fn delete_file(
    State(_state): State<AppState>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<String>, StatusCode> {
    // TODO: Implement file deletion logic
    // - Validate user permissions
    // - Delete from storage and database

    tracing::info!("Deleting file: {}", file_id);
    Ok(Json(format!("File {} deleted successfully", file_id)))
}

// Get file metadata endpoint
pub async fn get_file_metadata(
    State(_state): State<AppState>,
    Path(file_id): Path<Uuid>,
) -> Result<Json<FileMetadata>, StatusCode> {
    // TODO: Implement metadata retrieval
    // - Query database for file metadata
    // - Return structured file information

    tracing::info!("Getting metadata for file: {}", file_id);

    // Mock response for now
    let metadata = FileMetadata {
        id: file_id,
        tenant_id: Uuid::new_v4(),
        filename: "example.txt".to_string(),
        content_type: "text/plain".to_string(),
        size: 1024,
        storage_path: format!("/files/{}", file_id),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    Ok(Json(metadata))
}

// File sharing endpoint - Uses Temporal workflow for complex sharing logic
pub async fn share_file(
    State(_state): State<AppState>,
    Path(file_id): Path<Uuid>,
    Json(request): Json<FileShareRequest>,
) -> Result<Json<FileShareResponse>, StatusCode> {
    tracing::info!(
        "Sharing file: {} with {} recipients",
        file_id,
        request.recipients.len()
    );

    // Create workflow context (in real implementation, extract from auth headers)
    let context = WorkflowContext::new(
        Uuid::new_v4(),       // Default tenant for demo
        Some(Uuid::new_v4()), // Default user for demo
    );

    // Convert request to workflow data
    let share_type = match request.share_type.as_str() {
        "link" => ShareType::Link,
        "email" => ShareType::Email,
        "internal" => ShareType::Internal,
        _ => ShareType::Link,
    };

    let permissions = SharePermissions {
        can_view: request.permissions.can_view,
        can_download: request.permissions.can_download,
        can_comment: request.permissions.can_comment,
        can_share: request.permissions.can_share,
    };

    let expiration = request
        .expiration_hours
        .map(|hours| Utc::now() + chrono::Duration::hours(hours as i64));

    let share_data = FileShareData {
        file_id,
        share_type,
        recipients: request.recipients,
        permissions,
        expiration,
        notify_recipients: request.notify_recipients,
    };

    let workflow_input = StandardWorkflowInput {
        context,
        data: share_data,
    };

    // Execute Temporal workflow for file sharing
    match file_sharing_workflow(workflow_input).await {
        Ok(result) => {
            tracing::info!(
                "File sharing workflow completed: {:?}",
                result.result.share_id
            );
            Ok(Json(FileShareResponse {
                share_id: result.result.share_id,
                share_url: result.result.share_url,
                notifications_sent: result.result.notifications_sent,
                message: "File shared successfully".to_string(),
            }))
        }
        Err(error) => {
            tracing::error!("File sharing workflow failed: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
