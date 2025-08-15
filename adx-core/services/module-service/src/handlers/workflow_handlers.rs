use axum::{extract::State, response::Json};
use std::sync::Arc;

use crate::server::AppState;
use crate::error::ModuleServiceError;

pub async fn install_module_workflow(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"workflow_id": "placeholder"})))
}

pub async fn update_module_workflow(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"workflow_id": "placeholder"})))
}

pub async fn uninstall_module_workflow(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"workflow_id": "placeholder"})))
}

pub async fn get_workflow_status(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"status": "running"})))
}

pub async fn cancel_workflow(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"cancelled": true})))
}