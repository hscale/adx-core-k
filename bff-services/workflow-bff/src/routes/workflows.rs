use axum::{
    extract::{Path, Query, Request, State, WebSocketUpgrade, WebSocket},
    http::StatusCode,
    response::{Json, Response},
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info, warn};
use futures::{sink::SinkExt, stream::StreamExt};

use crate::{
    middleware::{
        auth::{has_permission, Claims},
        error_handler::{BffError, BffResult},
        tenant::{get_tenant_context, get_tenant_id},
    },
    types::{
        ApiResponse, PaginationParams, ResponseMeta, TenantContext,
        WorkflowListRequest, StartWorkflowRequest, WorkflowQueryRequest,
        CreateWorkflowScheduleRequest, UpdateWorkflowScheduleRequest,
    },
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_workflows).post(start_workflow))
        .route("/:workflow_id", get(get_workflow_status).delete(cancel_workflow))
        .route("/:workflow_id/terminate", post(terminate_workflow))
        .route("/:workflow_id/history", get(get_workflow_history))
        .route("/:workflow_id/query", post(query_workflow))
        .route("/:workflow_id/signal", post(signal_workflow))
        .route("/:workflow_id/stream", get(stream_workflow_progress))
        .route("/templates", get(list_workflow_templates))
        .route("/templates/:template_id", get(get_workflow_template))
        .route("/schedules", get(list_workflow_schedules).post(create_workflow_schedule))
        .route("/schedules/:schedule_id", get(get_workflow_schedule).put(update_workflow_schedule).delete(delete_workflow_schedule))
        .route("/stats", get(get_workflow_stats))
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkflowListQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    workflow_type: Option<String>,
    status: Option<String>,
    start_time_from: Option<String>,
    start_time_to: Option<String>,
    search: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkflowHistoryQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    event_type_filter: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkflowQueryBody {
    query_type: String,
    query_args: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct WorkflowSignalBody {
    signal_name: String,
    signal_input: Option<serde_json::Value>,
}

// List workflows with advanced filtering and caching
async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<WorkflowListQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to list workflows"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Create cache key based on query parameters
    let params_hash = create_params_hash(&query)?;
    
    // Try to get from cache first
    if let Ok(Some(cached_workflows)) = state.redis.get_cached_workflow_list(tenant_id, &params_hash).await {
        debug!("Returning cached workflow list for tenant: {}", tenant_id);
        return Ok(Json(ApiResponse {
            data: cached_workflows,
            meta: Some(ResponseMeta {
                total: None,
                page: query.pagination.page,
                per_page: query.pagination.per_page,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Build request for Temporal client
    let list_request = WorkflowListRequest {
        workflow_type: query.workflow_type.clone(),
        status: query.status.as_ref().and_then(|s| s.parse().ok()),
        start_time_from: query.start_time_from.as_ref().and_then(|s| s.parse().ok()),
        start_time_to: query.start_time_to.as_ref().and_then(|s| s.parse().ok()),
        page: query.pagination.page,
        per_page: query.pagination.per_page,
        search: query.search.clone(),
    };

    // Fetch from Temporal client
    let workflows = state.temporal_client.list_workflows(&list_request).await?;
    let workflows_json = serde_json::to_value(&workflows)?;

    // Cache the result
    if let Err(e) = state.redis.cache_workflow_list(tenant_id, &params_hash, &workflows_json, Some(300)).await {
        error!("Failed to cache workflow list: {}", e);
    }

    info!("Listed workflows for tenant: {} (query: {:?})", tenant_id, query);

    Ok(Json(ApiResponse {
        data: workflows_json,
        meta: Some(ResponseMeta {
            total: Some(workflows.len() as u64),
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Start workflow
async fn start_workflow(
    State(state): State<AppState>,
    Json(start_request): Json<StartWorkflowRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:execute") {
        return Err(BffError::authorization("Insufficient permissions to start workflow"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Add tenant context to workflow input
    let mut workflow_input = start_request.input.clone();
    if let Some(input_obj) = workflow_input.as_object_mut() {
        input_obj.insert("tenant_id".to_string(), serde_json::Value::String(tenant_id.clone()));
        input_obj.insert("user_id".to_string(), serde_json::Value::String(claims.sub.clone()));
    }

    // Start workflow via Temporal client
    let response = state.temporal_client.start_workflow(
        &start_request.workflow_type,
        &workflow_input,
        start_request.task_queue.as_deref(),
        start_request.workflow_id.as_deref(),
        start_request.cron_schedule.as_deref(),
        start_request.memo.as_ref(),
        start_request.search_attributes.as_ref(),
    ).await?;

    // Invalidate workflow list cache
    if let Err(e) = state.redis.invalidate_tenant_workflow_caches(tenant_id).await {
        error!("Failed to invalidate workflow caches: {}", e);
    }

    info!("Started workflow: {} for tenant: {}", response.workflow_id, tenant_id);

    Ok(Json(ApiResponse {
        data: serde_json::to_value(&response)?,
        meta: None,
    }))
}

// Get workflow status with caching
async fn get_workflow_status(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to read workflow status"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Try cache first for completed workflows
    if let Ok(Some(cached_status)) = state.redis.get_cached_workflow_status(&workflow_id).await {
        debug!("Returning cached workflow status: {}", workflow_id);
        return Ok(Json(ApiResponse {
            data: serde_json::to_value(cached_status)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(600),
            }),
        }));
    }

    // Fetch from Temporal client
    let workflow_execution = state.temporal_client.get_workflow_status(&workflow_id).await?;

    // Cache completed workflows for longer
    let cache_ttl = match workflow_execution.status {
        crate::types::WorkflowStatus::Completed | 
        crate::types::WorkflowStatus::Failed | 
        crate::types::WorkflowStatus::Cancelled | 
        crate::types::WorkflowStatus::Terminated | 
        crate::types::WorkflowStatus::TimedOut => Some(3600), // 1 hour
        _ => Some(60), // 1 minute for running workflows
    };

    if let Err(e) = state.redis.cache_workflow_status(&workflow_id, &workflow_execution, cache_ttl).await {
        error!("Failed to cache workflow status: {}", e);
    }

    info!("Retrieved workflow status: {} for tenant: {}", workflow_id, tenant_id);

    Ok(Json(ApiResponse {
        data: serde_json::to_value(&workflow_execution)?,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl,
        }),
    }))
}

// Cancel workflow
async fn cancel_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    request: Request,
) -> BffResult<StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:cancel") {
        return Err(BffError::authorization("Insufficient permissions to cancel workflow"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Cancel workflow via Temporal client
    state.temporal_client.cancel_workflow(&workflow_id, Some("Cancelled by user")).await?;

    // Invalidate caches
    if let Err(e) = state.redis.invalidate_workflow_caches(&workflow_id).await {
        error!("Failed to invalidate workflow caches: {}", e);
    }

    info!("Cancelled workflow: {} for tenant: {}", workflow_id, tenant_id);

    Ok(StatusCode::NO_CONTENT)
}

// Terminate workflow
async fn terminate_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    request: Request,
) -> BffResult<StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:terminate") {
        return Err(BffError::authorization("Insufficient permissions to terminate workflow"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Terminate workflow via Temporal client
    state.temporal_client.terminate_workflow(&workflow_id, Some("Terminated by user")).await?;

    // Invalidate caches
    if let Err(e) = state.redis.invalidate_workflow_caches(&workflow_id).await {
        error!("Failed to invalidate workflow caches: {}", e);
    }

    info!("Terminated workflow: {} for tenant: {}", workflow_id, tenant_id);

    Ok(StatusCode::NO_CONTENT)
}

// Get workflow history with caching
async fn get_workflow_history(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Query(query): Query<WorkflowHistoryQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to read workflow history"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Create cache key based on query parameters
    let params_hash = create_params_hash(&query)?;
    
    // Try cache first
    if let Ok(Some(cached_history)) = state.redis.get_cached_workflow_history(&workflow_id, &params_hash).await {
        debug!("Returning cached workflow history: {}", workflow_id);
        return Ok(Json(ApiResponse {
            data: cached_history,
            meta: Some(ResponseMeta {
                total: None,
                page: query.pagination.page,
                per_page: query.pagination.per_page,
                cached: Some(true),
                cache_ttl: Some(600),
            }),
        }));
    }

    // Fetch from Temporal client
    let history = state.temporal_client.get_workflow_history(
        &workflow_id,
        query.pagination.per_page,
        None, // next_page_token would come from query
        query.event_type_filter.as_deref(),
    ).await?;

    // Cache the result
    if let Err(e) = state.redis.cache_workflow_history(&workflow_id, &params_hash, &history, Some(600)).await {
        error!("Failed to cache workflow history: {}", e);
    }

    Ok(Json(ApiResponse {
        data: history,
        meta: Some(ResponseMeta {
            total: None,
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Query workflow
async fn query_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(query_body): Json<WorkflowQueryBody>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to query workflow"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Query workflow via Temporal client
    let result = state.temporal_client.query_workflow(
        &workflow_id,
        &query_body.query_type,
        query_body.query_args.as_ref(),
    ).await?;

    info!("Queried workflow: {} (query: {}) for tenant: {}", workflow_id, query_body.query_type, tenant_id);

    Ok(Json(ApiResponse {
        data: result,
        meta: None,
    }))
}

// Signal workflow
async fn signal_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(signal_body): Json<WorkflowSignalBody>,
    request: Request,
) -> BffResult<StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:signal") {
        return Err(BffError::authorization("Insufficient permissions to signal workflow"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Signal workflow via Temporal client
    state.temporal_client.signal_workflow(
        &workflow_id,
        &signal_body.signal_name,
        signal_body.signal_input.as_ref(),
    ).await?;

    // Invalidate workflow status cache since it may have changed
    if let Err(e) = state.redis.invalidate_workflow_caches(&workflow_id).await {
        error!("Failed to invalidate workflow caches: {}", e);
    }

    info!("Signaled workflow: {} (signal: {}) for tenant: {}", workflow_id, signal_body.signal_name, tenant_id);

    Ok(StatusCode::NO_CONTENT)
}

// Stream workflow progress via WebSocket
async fn stream_workflow_progress(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    request: Request,
) -> Response {
    let claims = match request.extensions().get::<Claims>() {
        Some(claims) => claims.clone(),
        None => return (StatusCode::UNAUTHORIZED, "Missing authentication").into_response(),
    };

    let tenant_context = match get_tenant_context(&request) {
        Some(context) => context.clone(),
        None => return (StatusCode::BAD_REQUEST, "Missing tenant context").into_response(),
    };

    // Check permissions
    if !has_permission(&claims, "workflow:read") {
        return (StatusCode::FORBIDDEN, "Insufficient permissions").into_response();
    }

    ws.on_upgrade(move |socket| handle_workflow_stream(socket, state, workflow_id, claims, tenant_context))
}

async fn handle_workflow_stream(
    mut socket: WebSocket,
    state: AppState,
    workflow_id: String,
    claims: Claims,
    tenant_context: TenantContext,
) {
    let session_id = uuid::Uuid::new_v4().to_string();
    
    // Subscribe to workflow updates
    if let Err(e) = state.redis.subscribe_to_workflow(&workflow_id, &session_id, Some(3600)).await {
        error!("Failed to subscribe to workflow updates: {}", e);
        let _ = socket.close().await;
        return;
    }

    // Store WebSocket session
    if let Err(e) = state.redis.store_websocket_session(&session_id, &claims.sub, &tenant_context.tenant_id, Some(3600)).await {
        error!("Failed to store WebSocket session: {}", e);
    }

    info!("Started workflow stream for workflow: {} (session: {})", workflow_id, session_id);

    // Send initial workflow status
    match state.temporal_client.get_workflow_status(&workflow_id).await {
        Ok(status) => {
            let message = serde_json::json!({
                "type": "status",
                "workflow_id": workflow_id,
                "data": status,
                "timestamp": chrono::Utc::now()
            });
            
            if let Err(e) = socket.send(axum::extract::ws::Message::Text(message.to_string())).await {
                error!("Failed to send initial status: {}", e);
                return;
            }
        }
        Err(e) => {
            error!("Failed to get initial workflow status: {}", e);
            let error_message = serde_json::json!({
                "type": "error",
                "message": "Failed to get workflow status",
                "timestamp": chrono::Utc::now()
            });
            let _ = socket.send(axum::extract::ws::Message::Text(error_message.to_string())).await;
            return;
        }
    }

    // Handle WebSocket messages and send periodic updates
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
    
    loop {
        tokio::select! {
            msg = socket.recv() => {
                match msg {
                    Some(Ok(axum::extract::ws::Message::Text(text))) => {
                        debug!("Received WebSocket message: {}", text);
                        // Handle client messages if needed
                    }
                    Some(Ok(axum::extract::ws::Message::Close(_))) => {
                        debug!("WebSocket connection closed");
                        break;
                    }
                    Some(Err(e)) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                    None => break,
                }
            }
            _ = interval.tick() => {
                // Send periodic workflow status updates
                match state.temporal_client.get_workflow_status(&workflow_id).await {
                    Ok(status) => {
                        let message = serde_json::json!({
                            "type": "status_update",
                            "workflow_id": workflow_id,
                            "data": status,
                            "timestamp": chrono::Utc::now()
                        });
                        
                        if let Err(e) = socket.send(axum::extract::ws::Message::Text(message.to_string())).await {
                            error!("Failed to send status update: {}", e);
                            break;
                        }

                        // Stop streaming if workflow is completed
                        match status.status {
                            crate::types::WorkflowStatus::Completed | 
                            crate::types::WorkflowStatus::Failed | 
                            crate::types::WorkflowStatus::Cancelled | 
                            crate::types::WorkflowStatus::Terminated | 
                            crate::types::WorkflowStatus::TimedOut => {
                                let final_message = serde_json::json!({
                                    "type": "workflow_completed",
                                    "workflow_id": workflow_id,
                                    "final_status": status.status,
                                    "timestamp": chrono::Utc::now()
                                });
                                let _ = socket.send(axum::extract::ws::Message::Text(final_message.to_string())).await;
                                break;
                            }
                            _ => {}
                        }
                    }
                    Err(e) => {
                        error!("Failed to get workflow status for streaming: {}", e);
                        let error_message = serde_json::json!({
                            "type": "error",
                            "message": "Failed to get workflow status",
                            "timestamp": chrono::Utc::now()
                        });
                        let _ = socket.send(axum::extract::ws::Message::Text(error_message.to_string())).await;
                        break;
                    }
                }
            }
        }
    }

    // Cleanup
    if let Err(e) = state.redis.unsubscribe_from_workflow(&workflow_id, &session_id).await {
        error!("Failed to unsubscribe from workflow: {}", e);
    }
    
    if let Err(e) = state.redis.remove_websocket_session(&session_id).await {
        error!("Failed to remove WebSocket session: {}", e);
    }

    info!("Ended workflow stream for workflow: {} (session: {})", workflow_id, session_id);
}

// List workflow templates with caching
async fn list_workflow_templates(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to list workflow templates"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Try cache first
    if let Ok(Some(cached_templates)) = state.redis.get_cached_workflow_templates(tenant_id).await {
        debug!("Returning cached workflow templates for tenant: {}", tenant_id);
        return Ok(Json(ApiResponse {
            data: cached_templates,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(3600),
            }),
        }));
    }

    // Fetch from Temporal client
    let templates = state.temporal_client.list_workflow_templates().await?;
    let templates_json = serde_json::to_value(&templates)?;

    // Cache the result
    if let Err(e) = state.redis.cache_workflow_templates(tenant_id, &templates_json, Some(3600)).await {
        error!("Failed to cache workflow templates: {}", e);
    }

    Ok(Json(ApiResponse {
        data: templates_json,
        meta: Some(ResponseMeta {
            total: Some(templates.len() as u64),
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get workflow template
async fn get_workflow_template(
    State(state): State<AppState>,
    Path(template_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to read workflow template"));
    }

    // Try cache first
    if let Ok(Some(cached_template)) = state.redis.get_cached_workflow_template(&template_id).await {
        debug!("Returning cached workflow template: {}", template_id);
        return Ok(Json(ApiResponse {
            data: cached_template,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(3600),
            }),
        }));
    }

    // In a real implementation, this would fetch from a template service
    // For now, we'll return a mock template
    let template = serde_json::json!({
        "id": template_id,
        "name": "Sample Template",
        "description": "A sample workflow template",
        "workflow_type": "sample_workflow",
        "parameters": [],
        "default_input": {}
    });

    // Cache the result
    if let Err(e) = state.redis.cache_workflow_template(&template_id, &template, Some(3600)).await {
        error!("Failed to cache workflow template: {}", e);
    }

    Ok(Json(ApiResponse {
        data: template,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// List workflow schedules
async fn list_workflow_schedules(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to list workflow schedules"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Try cache first
    if let Ok(Some(cached_schedules)) = state.redis.get_cached_workflow_schedules(tenant_id).await {
        debug!("Returning cached workflow schedules for tenant: {}", tenant_id);
        return Ok(Json(ApiResponse {
            data: cached_schedules,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(600),
            }),
        }));
    }

    // Fetch from Temporal client
    let schedules = state.temporal_client.list_workflow_schedules().await?;
    let schedules_json = serde_json::to_value(&schedules)?;

    // Cache the result
    if let Err(e) = state.redis.cache_workflow_schedules(tenant_id, &schedules_json, Some(600)).await {
        error!("Failed to cache workflow schedules: {}", e);
    }

    Ok(Json(ApiResponse {
        data: schedules_json,
        meta: Some(ResponseMeta {
            total: Some(schedules.len() as u64),
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Create workflow schedule
async fn create_workflow_schedule(
    State(state): State<AppState>,
    Json(create_request): Json<CreateWorkflowScheduleRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:schedule") {
        return Err(BffError::authorization("Insufficient permissions to create workflow schedule"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Create schedule via Temporal client
    let schedule = state.temporal_client.create_workflow_schedule(&create_request).await?;

    // Invalidate schedules cache
    let schedules_key = format!("workflow_schedules:{}", tenant_id);
    if let Err(e) = state.redis.delete(&schedules_key).await {
        error!("Failed to invalidate workflow schedules cache: {}", e);
    }

    info!("Created workflow schedule: {} for tenant: {}", schedule.id, tenant_id);

    Ok(Json(ApiResponse {
        data: serde_json::to_value(&schedule)?,
        meta: None,
    }))
}

// Get workflow schedule
async fn get_workflow_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to read workflow schedule"));
    }

    // In a real implementation, this would fetch from Temporal
    // For now, we'll return a mock schedule
    let schedule = serde_json::json!({
        "id": schedule_id,
        "workflow_type": "sample_workflow",
        "cron_expression": "0 9 * * *",
        "input": {},
        "is_active": true,
        "next_run": chrono::Utc::now() + chrono::Duration::hours(24),
        "last_run": chrono::Utc::now() - chrono::Duration::hours(24),
        "created_at": chrono::Utc::now() - chrono::Duration::days(30),
        "updated_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse {
        data: schedule,
        meta: None,
    }))
}

// Update workflow schedule
async fn update_workflow_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<String>,
    Json(update_request): Json<UpdateWorkflowScheduleRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:schedule") {
        return Err(BffError::authorization("Insufficient permissions to update workflow schedule"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Update schedule via Temporal client
    let schedule = state.temporal_client.update_workflow_schedule(&schedule_id, &update_request).await?;

    // Invalidate schedules cache
    let schedules_key = format!("workflow_schedules:{}", tenant_id);
    if let Err(e) = state.redis.delete(&schedules_key).await {
        error!("Failed to invalidate workflow schedules cache: {}", e);
    }

    info!("Updated workflow schedule: {} for tenant: {}", schedule_id, tenant_id);

    Ok(Json(ApiResponse {
        data: serde_json::to_value(&schedule)?,
        meta: None,
    }))
}

// Delete workflow schedule
async fn delete_workflow_schedule(
    State(state): State<AppState>,
    Path(schedule_id): Path<String>,
    request: Request,
) -> BffResult<StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:schedule") {
        return Err(BffError::authorization("Insufficient permissions to delete workflow schedule"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Delete schedule via Temporal client
    state.temporal_client.delete_workflow_schedule(&schedule_id).await?;

    // Invalidate schedules cache
    let schedules_key = format!("workflow_schedules:{}", tenant_id);
    if let Err(e) = state.redis.delete(&schedules_key).await {
        error!("Failed to invalidate workflow schedules cache: {}", e);
    }

    info!("Deleted workflow schedule: {} for tenant: {}", schedule_id, tenant_id);

    Ok(StatusCode::NO_CONTENT)
}

// Get workflow stats (admin only)
async fn get_workflow_stats(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:admin") {
        return Err(BffError::authorization("Insufficient permissions to view workflow stats"));
    }

    // Mock workflow stats (in real implementation, this would aggregate from Temporal)
    let stats = serde_json::json!({
        "total_workflows": 5420,
        "running_workflows": 234,
        "completed_workflows": 4890,
        "failed_workflows": 296,
        "workflows_today": 156,
        "workflows_this_week": 892,
        "workflows_this_month": 3456,
        "average_execution_time_ms": 45000,
        "workflow_types": {
            "user_onboarding": 1234,
            "data_processing": 2156,
            "file_processing": 1890,
            "report_generation": 140
        },
        "success_rate": 94.5,
        "retry_rate": 12.3
    });

    Ok(Json(ApiResponse {
        data: stats,
        meta: None,
    }))
}

// Helper functions
fn extract_auth_token(request: &Request) -> BffResult<String> {
    let auth_header = request.headers()
        .get("authorization")
        .ok_or_else(|| BffError::authentication("Missing authorization header"))?;
    
    let auth_str = auth_header.to_str()
        .map_err(|_| BffError::authentication("Invalid authorization header"))?;
    
    if auth_str.starts_with("Bearer ") {
        Ok(auth_str[7..].to_string())
    } else {
        Err(BffError::authentication("Invalid authorization header format"))
    }
}

fn create_params_hash<T: serde::Serialize>(params: &T) -> BffResult<String> {
    let params_json = serde_json::to_string(params)?;
    let hash = format!("{:x}", md5::compute(params_json.as_bytes()));
    Ok(hash)
}