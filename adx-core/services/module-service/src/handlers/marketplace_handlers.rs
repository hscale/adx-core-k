use axum::{extract::State, response::Json};
use std::sync::Arc;

use crate::server::AppState;
use crate::error::ModuleServiceError;

pub async fn list_marketplace_modules(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"modules": []})))
}

pub async fn get_marketplace_module(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"module": null})))
}

pub async fn search_marketplace(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"results": []})))
}

pub async fn get_featured_modules(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"featured": []})))
}

pub async fn get_categories(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ModuleServiceError> {
    Ok(Json(serde_json::json!({"categories": []})))
}