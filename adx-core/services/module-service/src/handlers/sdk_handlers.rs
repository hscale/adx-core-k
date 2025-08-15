use axum::{extract::State, response::Json};
use std::sync::Arc;

use crate::server::AppState;
use crate::error::ModuleServiceError;

pub async fn list_templates(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"templates": []})))
}

pub async fn create_from_template(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"created": true})))
}

pub async fn validate_module(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"valid": true})))
}

pub async fn build_module(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"built": true})))
}

pub async fn test_module(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"tested": true})))
}

pub async fn package_module(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"packaged": true})))
}

pub async fn publish_module(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"published": true})))
}