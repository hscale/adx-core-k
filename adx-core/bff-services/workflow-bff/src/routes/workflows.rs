use axum::{
    extract::{Path, State, Extension, Query},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{AppState, middleware::{auth::Claims, tenant::TenantContext}};

#[derive(Debug, Deserialize)]
struct WorkflowQuery {
    status: Option<String>,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_workflows))
        .route("/:workflow_id", get(get_workflow))
        .route("/:workflow_id/status", get(get_workflow_status))
        .route("/:workflow_id/cancel", post(cancel_workflow))
        .route("/:workflow_id/retry", post(retry_workflow))
        .route("/user/:user_id", get(get_user_workflows))
}

async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<WorkflowQuery>,
    Extension(claims): Extension<Claims>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, this would query Temporal for workflows
    // filtered by tenant and user permissions
    
    let mock_workflows = vec![
        json!({
            "workflow_id": "user-onboarding-123",
            "workflow_type": "UserOnboardingWorkflow",
            "status": "COMPLETED",
            "started_at": "2024-01-15T10:00:00Z",
            "completed_at": "2024-01-15T10:05:00Z",
            "user_id": claims.sub,
            "tenant_id": tenant.tenant_id
        }),
        json!({
            "workflow_id": "file-processing-456",
            "workflow_type": "FileProcessingWorkflow",
            "status": "RUNNING",
            "started_at": "2024-01-15T10:30:00Z",
            "completed_at": null,
            "user_id": claims.sub,
            "tenant_id": tenant.tenant_id
        }),
    ];

    let filtered_workflows: Vec<_> = mock_workflows
        .into_iter()
        .filter(|w| {
            if let Some(status) = &query.status {
                w["status"].as_str() == Some(status)
            } else {
                true
            }
        })
        .collect();

    Ok(Json(json!({
        "workflows": filtered_workflows,
        "total": filtered_workflows.len(),
        "tenant_id": tenant.tenant_id
    })))
}

async fn get_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Extension(claims): Extension<Claims>,
    Extension(tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.temporal_client.get_workflow_status(&workflow_id).await {
        Ok(status) => Ok(Json(json!(status))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn get_workflow_status(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Extension(_claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    match state.temporal_client.get_workflow_status(&workflow_id).await {
        Ok(status) => Ok(Json(json!({
            "workflow_id": workflow_id,
            "status": status.status,
            "result": status.result,
            "started_at": status.started_at,
            "completed_at": status.completed_at
        }))),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

async fn cancel_workflow(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
    Extension(_claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, this would cancel the Temporal workflow
    tracing::info!("Cancelling workflow: {}", workflow_id);
    
    Ok(Json(json!({
        "workflow_id": workflow_id,
        "status": "CANCELLED",
        "cancelled_at": chrono::Utc::now().to_rfc3339()
    })))
}

async fn retry_workflow(
    State(_state): State<AppState>,
    Path(workflow_id): Path<String>,
    Extension(_claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // In a real implementation, this would retry the Temporal workflow
    tracing::info!("Retrying workflow: {}", workflow_id);
    
    let new_workflow_id = format!("{}-retry-{}", workflow_id, uuid::Uuid::new_v4());
    
    Ok(Json(json!({
        "original_workflow_id": workflow_id,
        "new_workflow_id": new_workflow_id,
        "status": "STARTED",
        "started_at": chrono::Utc::now().to_rfc3339()
    })))
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