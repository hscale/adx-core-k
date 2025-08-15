use axum::{extract::State, response::Json};
use std::sync::Arc;

use crate::server::AppState;
use crate::error::ModuleServiceError;

pub async fn scan_module(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"scan_id": "placeholder"})))
}

pub async fn get_scan_results(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"results": {}})))
}

pub async fn list_vulnerabilities(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"vulnerabilities": []})))
}