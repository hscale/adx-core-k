use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::{json, Value};

use crate::{AppState, middleware::{auth::Claims, tenant::TenantContext}};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/user/:user_id", get(get_user_workflows))
        .route("/user/:user_id/sync", post(start_user_sync))
        .route("/:workflow_id/status", get(get_workflow_status))
}

async fn get_user_workflows(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Extension(claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Check permissions
    if user_id != claims.sub && !claims.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.temporal_client.get_user_workflows(&user_id).await {
        Ok(workflows) => Ok(Json(json!({
            "workflows": workflows,
            "user_id": user_id,
            "total": workflows.len()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn start_user_sync(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Extension(claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Check permissions
    if user_id != claims.sub && !claims.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    match state.temporal_client.start_user_sync_workflow(&user_id).await {
        Ok(workflow_id) => Ok(Json(json!({
            "workflow_id": workflow_id,
            "status": "STARTED",
            "user_id": user_id,
            "started_at": chrono::Utc::now().to_rfc3339()
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_workflow_status(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Extension(_claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.temporal_client.get_workflow_status(&workflow_id).await {
        Ok(status) => Ok(Json(json!(status))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}