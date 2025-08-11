use std::sync::Arc;
use axum::{
    extract::{Path, Query, State, Multipart},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use adx_shared::{TenantContext, UserContext, Result, Error};
use crate::models::*;
use crate::services::FileService;

#[derive(Debug, Deserialize)]
pub struct ListFilesQuery {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct ShareAccessRequest {
    pub password: Option<String>,
}

pub struct FileHandlers {
    file_service: Arc<FileService>,
}

impl FileHandlers {
    pub fn new(file_service: Arc<FileService>) -> Self {
        Self { file_service }
    }

    pub async fn create_file(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Json(request): Json<CreateFileRequest>,
    ) -> Result<Json<FileUploadResponse>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.create_file(&request, &tenant_context, &user_context).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                tracing::error!("Failed to create file: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to create file",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn get_file(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
    ) -> Result<Json<File>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.get_file(file_id, &tenant_context, &user_context).await {
            Ok(Some(file)) => Ok(Json(file)),
            Ok(None) => Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({
                    "error": "File not found"
                }))
            )),
            Err(e) => {
                tracing::error!("Failed to get file: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to get file",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn update_file(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
        Json(request): Json<UpdateFileRequest>,
    ) -> Result<Json<File>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.update_file(file_id, &request, &tenant_context, &user_context).await {
            Ok(file) => Ok(Json(file)),
            Err(e) => {
                tracing::error!("Failed to update file: {}", e);
                let status = if e.to_string().contains("Permission denied") {
                    StatusCode::FORBIDDEN
                } else if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to update file",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn delete_file(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
    ) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.delete_file(file_id, &tenant_context, &user_context).await {
            Ok(()) => Ok(StatusCode::NO_CONTENT),
            Err(e) => {
                tracing::error!("Failed to delete file: {}", e);
                let status = if e.to_string().contains("Permission denied") {
                    StatusCode::FORBIDDEN
                } else if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to delete file",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn list_files(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Query(query): Query<ListFilesQuery>,
    ) -> Result<Json<FileListResponse>, (StatusCode, Json<serde_json::Value>)> {
        let page = query.page.unwrap_or(1);
        let per_page = query.per_page.unwrap_or(20).min(100); // Cap at 100 items per page

        match handlers.file_service.list_files(&tenant_context, &user_context, page, per_page).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                tracing::error!("Failed to list files: {}", e);
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "Failed to list files",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn upload_file_data(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
        mut multipart: Multipart,
    ) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
        // Extract file data from multipart form
        let mut file_data = Vec::new();
        
        while let Some(field) = multipart.next_field().await.map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "Invalid multipart data",
                    "details": e.to_string()
                }))
            )
        })? {
            if field.name() == Some("file") {
                let data = field.bytes().await.map_err(|e| {
                    (
                        StatusCode::BAD_REQUEST,
                        Json(serde_json::json!({
                            "error": "Failed to read file data",
                            "details": e.to_string()
                        }))
                    )
                })?;
                file_data = data.to_vec();
                break;
            }
        }

        if file_data.is_empty() {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": "No file data provided"
                }))
            ));
        }

        match handlers.file_service.upload_file_data(file_id, &file_data, &tenant_context, &user_context).await {
            Ok(()) => Ok(StatusCode::OK),
            Err(e) => {
                tracing::error!("Failed to upload file data: {}", e);
                let status = if e.to_string().contains("Permission denied") {
                    StatusCode::FORBIDDEN
                } else if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to upload file data",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn download_file(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
    ) -> Result<Json<FileDownloadResponse>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.download_file(file_id, &tenant_context, &user_context).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                tracing::error!("Failed to get download URL: {}", e);
                let status = if e.to_string().contains("access denied") || e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else if e.to_string().contains("not ready") {
                    StatusCode::CONFLICT
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to get download URL",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn create_file_share(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
        Json(request): Json<CreateFileShareRequest>,
    ) -> Result<Json<FileShare>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.create_file_share(file_id, &request, &tenant_context, &user_context).await {
            Ok(share) => Ok(Json(share)),
            Err(e) => {
                tracing::error!("Failed to create file share: {}", e);
                let status = if e.to_string().contains("Permission denied") {
                    StatusCode::FORBIDDEN
                } else if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to create file share",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn get_file_shares(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
    ) -> Result<Json<Vec<FileShare>>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.get_file_shares(file_id, &tenant_context, &user_context).await {
            Ok(shares) => Ok(Json(shares)),
            Err(e) => {
                tracing::error!("Failed to get file shares: {}", e);
                let status = if e.to_string().contains("Permission denied") {
                    StatusCode::FORBIDDEN
                } else if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to get file shares",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn access_shared_file(
        State(handlers): State<Arc<FileHandlers>>,
        Path(share_token): Path<String>,
        Json(request): Json<ShareAccessRequest>,
    ) -> Result<Json<FileDownloadResponse>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.access_shared_file(&share_token, request.password.as_deref()).await {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                tracing::error!("Failed to access shared file: {}", e);
                let status = if e.to_string().contains("Invalid") || e.to_string().contains("expired") {
                    StatusCode::NOT_FOUND
                } else if e.to_string().contains("Password") {
                    StatusCode::UNAUTHORIZED
                } else if e.to_string().contains("limit exceeded") {
                    StatusCode::TOO_MANY_REQUESTS
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to access shared file",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn grant_file_permission(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
        Json(request): Json<CreateFilePermissionRequest>,
    ) -> Result<Json<FilePermission>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.grant_file_permission(file_id, &request, &tenant_context, &user_context).await {
            Ok(permission) => Ok(Json(permission)),
            Err(e) => {
                tracing::error!("Failed to grant file permission: {}", e);
                let status = if e.to_string().contains("Permission denied") {
                    StatusCode::FORBIDDEN
                } else if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to grant file permission",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn get_file_permissions(
        State(handlers): State<Arc<FileHandlers>>,
        Extension(tenant_context): Extension<TenantContext>,
        Extension(user_context): Extension<UserContext>,
        Path(file_id): Path<Uuid>,
    ) -> Result<Json<Vec<FilePermission>>, (StatusCode, Json<serde_json::Value>)> {
        match handlers.file_service.get_file_permissions(file_id, &tenant_context, &user_context).await {
            Ok(permissions) => Ok(Json(permissions)),
            Err(e) => {
                tracing::error!("Failed to get file permissions: {}", e);
                let status = if e.to_string().contains("Permission denied") {
                    StatusCode::FORBIDDEN
                } else if e.to_string().contains("not found") {
                    StatusCode::NOT_FOUND
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": "Failed to get file permissions",
                        "details": e.to_string()
                    }))
                ))
            }
        }
    }

    pub async fn health_check() -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
        Ok(Json(serde_json::json!({
            "status": "healthy",
            "service": "file-service",
            "timestamp": chrono::Utc::now()
        })))
    }
}