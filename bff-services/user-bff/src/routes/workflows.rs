use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use tracing::{debug, error, info};

use crate::{
    middleware::{
        auth::{has_permission, Claims},
        error_handler::{BffError, BffResult},
        tenant::{get_tenant_context, get_tenant_id},
    },
    types::{
        ApiResponse, PaginationParams, ResponseMeta, TenantContext,
        StartWorkflowRequest, StartWorkflowResponse, WorkflowExecution, WorkflowListRequest,
        CancelWorkflowRequest, TerminateWorkflowRequest, WorkflowQuery, WorkflowSignal,
    },
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_workflows))
        .route("/:workflow_id", get(get_workflow_status))
}

#[derive(Debug, Deserialize)]
struct WorkflowListQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    workflow_type: Option<String>,
    status: Option<String>,
    user_id: Option<String>,
    start_time_from: Option<String>,
    start_time_to: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserOnboardingRequest {
    pub user_id: String,
    pub tenant_id: String,
    pub onboarding_data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct UserProfileSyncRequest {
    pub user_id: String,
    pub sync_targets: Vec<String>,
    pub profile_data: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct UserDataExportRequest {
    pub user_id: String,
    pub export_format: String,
    pub export_options: serde_json::Value,
}

#[derive(Debug, Deserialize)]
struct UserDeactivationRequest {
    pub user_id: String,
    pub deactivation_reason: Option<String>,
    pub retain_data: bool,
}

// List workflows with filtering and caching
async fn list_workflows(
    State(state): State<AppState>,
    Query(query): Query<WorkflowListQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<Vec<WorkflowExecution>>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to list workflows"));
    }

    // Parse status filter
    let status_filter = query.status.as_ref().and_then(|s| {
        match s.as_str() {
            "running" => Some(crate::types::WorkflowStatus::Running),
            "completed" => Some(crate::types::WorkflowStatus::Completed),
            "failed" => Some(crate::types::WorkflowStatus::Failed),
            "cancelled" => Some(crate::types::WorkflowStatus::Cancelled),
            "terminated" => Some(crate::types::WorkflowStatus::Terminated),
            "timed_out" => Some(crate::types::WorkflowStatus::TimedOut),
            _ => None,
        }
    });

    // List workflows via Temporal client
    let workflows = state.temporal_client.list_workflows(
        query.workflow_type.as_deref(),
        status_filter,
        query.pagination.per_page,
        None, // next_page_token
    ).await?;

    // Filter by user_id if specified
    let filtered_workflows = if let Some(user_id) = &query.user_id {
        workflows.into_iter()
            .filter(|w| {
                w.input.get("user_id")
                    .and_then(|v| v.as_str())
                    .map(|id| id == user_id)
                    .unwrap_or(false)
            })
            .collect()
    } else {
        workflows
    };

    info!("Listed {} workflows for tenant: {}", filtered_workflows.len(), tenant_context.tenant_id);

    Ok(Json(ApiResponse {
        data: filtered_workflows,
        meta: Some(ResponseMeta {
            total: None,
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Start a generic workflow
async fn start_workflow(
    State(state): State<AppState>,
    Json(workflow_request): Json<StartWorkflowRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<StartWorkflowResponse>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:execute") {
        return Err(BffError::authorization("Insufficient permissions to start workflow"));
    }

    // Add tenant and user context to workflow input
    let mut enriched_input = workflow_request.input.clone();
    if let Some(input_obj) = enriched_input.as_object_mut() {
        input_obj.insert("tenant_id".to_string(), serde_json::Value::String(tenant_context.tenant_id.clone()));
        input_obj.insert("user_id".to_string(), serde_json::Value::String(claims.sub.clone()));
        input_obj.insert("initiated_by".to_string(), serde_json::Value::String(claims.user_email.clone()));
    }

    // Start workflow via Temporal client
    let response = state.temporal_client.start_workflow(
        &workflow_request.workflow_type,
        &enriched_input,
        workflow_request.task_queue.as_deref(),
        workflow_request.workflow_id.as_deref(),
    ).await?;

    info!("Started workflow: {} with ID: {}", workflow_request.workflow_type, response.workflow_id);

    Ok(Json(ApiResponse {
        data: response,
        meta: None,
    }))
}

// Get workflow status with caching
async fn get_workflow_status(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<WorkflowExecution>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to read workflow status"));
    }

    // Try cache first for completed workflows
    if let Ok(Some(cached_status)) = state.redis.get_cached_workflow_status(&workflow_id).await {
        debug!("Returning cached workflow status: {}", workflow_id);
        return Ok(Json(ApiResponse {
            data: serde_json::from_value(cached_status)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Get workflow status via Temporal client
    let workflow_execution = state.temporal_client.get_workflow_status(&workflow_id).await?;

    // Cache completed workflows for longer
    let cache_ttl = match workflow_execution.status {
        crate::types::WorkflowStatus::Completed |
        crate::types::WorkflowStatus::Failed |
        crate::types::WorkflowStatus::Cancelled |
        crate::types::WorkflowStatus::Terminated |
        crate::types::WorkflowStatus::TimedOut => Some(3600), // 1 hour
        _ => Some(30), // 30 seconds for running workflows
    };

    if let Some(ttl) = cache_ttl {
        let workflow_data = serde_json::to_value(&workflow_execution)?;
        if let Err(e) = state.redis.cache_workflow_status(&workflow_id, &workflow_data, Some(ttl)).await {
            error!("Failed to cache workflow status: {}", e);
        }
    }

    debug!("Retrieved workflow status: {} (status: {:?})", workflow_id, workflow_execution.status);

    Ok(Json(ApiResponse {
        data: workflow_execution,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Cancel workflow
async fn cancel_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(cancel_request): Json<CancelWorkflowRequest>,
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

    // Cancel workflow via Temporal client
    state.temporal_client.cancel_workflow(&workflow_id, cancel_request.reason.as_deref()).await?;

    // Invalidate cache
    if let Err(e) = state.redis.delete(&format!("workflow_status:{}", workflow_id)).await {
        error!("Failed to invalidate workflow status cache: {}", e);
    }

    info!("Cancelled workflow: {} (reason: {:?})", workflow_id, cancel_request.reason);

    Ok(StatusCode::NO_CONTENT)
}

// Terminate workflow
async fn terminate_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(terminate_request): Json<TerminateWorkflowRequest>,
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

    // Terminate workflow via Temporal client
    state.temporal_client.terminate_workflow(&workflow_id, terminate_request.reason.as_deref()).await?;

    // Invalidate cache
    if let Err(e) = state.redis.delete(&format!("workflow_status:{}", workflow_id)).await {
        error!("Failed to invalidate workflow status cache: {}", e);
    }

    info!("Terminated workflow: {} (reason: {:?})", workflow_id, terminate_request.reason);

    Ok(StatusCode::NO_CONTENT)
}

// Query workflow
async fn query_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(query_request): Json<WorkflowQuery>,
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

    // Query workflow via Temporal client
    let query_result = state.temporal_client.query_workflow(
        &workflow_id,
        &query_request.query_type,
        query_request.query_args.as_ref(),
    ).await?;

    debug!("Queried workflow: {} (query: {})", workflow_id, query_request.query_type);

    Ok(Json(ApiResponse {
        data: query_result,
        meta: None,
    }))
}

// Signal workflow
async fn signal_workflow(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Json(signal_request): Json<WorkflowSignal>,
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

    // Signal workflow via Temporal client
    state.temporal_client.signal_workflow(
        &workflow_id,
        &signal_request.signal_name,
        signal_request.signal_input.as_ref(),
    ).await?;

    info!("Signaled workflow: {} (signal: {})", workflow_id, signal_request.signal_name);

    Ok(StatusCode::NO_CONTENT)
}

// Get workflow history
async fn get_workflow_history(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    Query(pagination): Query<PaginationParams>,
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

    // Get workflow history via Temporal client
    let history = state.temporal_client.get_workflow_history(
        &workflow_id,
        pagination.per_page,
        None, // next_page_token
    ).await?;

    debug!("Retrieved workflow history: {}", workflow_id);

    Ok(Json(ApiResponse {
        data: history,
        meta: Some(ResponseMeta {
            total: None,
            page: pagination.page,
            per_page: pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// User-specific workflow endpoints

// Start user onboarding workflow
async fn start_user_onboarding_workflow(
    State(state): State<AppState>,
    Json(onboarding_request): Json<UserOnboardingRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<StartWorkflowResponse>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "workflow:execute") {
        return Err(BffError::authorization("Insufficient permissions to start user onboarding workflow"));
    }

    // Start user onboarding workflow via Temporal client
    let response = state.temporal_client.start_user_onboarding_workflow(
        &onboarding_request.user_id,
        &onboarding_request.tenant_id,
        &onboarding_request.onboarding_data,
    ).await?;

    info!("Started user onboarding workflow for user: {} (workflow: {})", 
          onboarding_request.user_id, response.workflow_id);

    Ok(Json(ApiResponse {
        data: response,
        meta: None,
    }))
}

// Start user profile sync workflow
async fn start_user_profile_sync_workflow(
    State(state): State<AppState>,
    Json(sync_request): Json<UserProfileSyncRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<StartWorkflowResponse>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions (users can sync their own profile, or need workflow:execute permission)
    if claims.sub != sync_request.user_id && !has_permission(claims, "workflow:execute") {
        return Err(BffError::authorization("Insufficient permissions to start profile sync workflow"));
    }

    // Start user profile sync workflow via Temporal client
    let response = state.temporal_client.start_user_profile_sync_workflow(
        &sync_request.user_id,
        &sync_request.sync_targets,
        &sync_request.profile_data,
    ).await?;

    info!("Started user profile sync workflow for user: {} (workflow: {})", 
          sync_request.user_id, response.workflow_id);

    Ok(Json(ApiResponse {
        data: response,
        meta: None,
    }))
}

// Start user data export workflow
async fn start_user_data_export_workflow(
    State(state): State<AppState>,
    Json(export_request): Json<UserDataExportRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<StartWorkflowResponse>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions (users can export their own data, or need workflow:execute permission)
    if claims.sub != export_request.user_id && !has_permission(claims, "workflow:execute") {
        return Err(BffError::authorization("Insufficient permissions to start data export workflow"));
    }

    // Start user data export workflow via Temporal client
    let response = state.temporal_client.start_user_data_export_workflow(
        &export_request.user_id,
        &export_request.export_format,
        &export_request.export_options,
    ).await?;

    info!("Started user data export workflow for user: {} (workflow: {})", 
          export_request.user_id, response.workflow_id);

    Ok(Json(ApiResponse {
        data: response,
        meta: None,
    }))
}

// Start user deactivation workflow
async fn start_user_deactivation_workflow(
    State(state): State<AppState>,
    Json(deactivation_request): Json<UserDeactivationRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<StartWorkflowResponse>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions (only admins can deactivate users)
    if !has_permission(claims, "user:delete") {
        return Err(BffError::authorization("Insufficient permissions to start user deactivation workflow"));
    }

    // Start user deactivation workflow via Temporal client
    let response = state.temporal_client.start_user_deactivation_workflow(
        &deactivation_request.user_id,
        deactivation_request.deactivation_reason.as_deref(),
        deactivation_request.retain_data,
    ).await?;

    info!("Started user deactivation workflow for user: {} (workflow: {})", 
          deactivation_request.user_id, response.workflow_id);

    Ok(Json(ApiResponse {
        data: response,
        meta: None,
    }))
}