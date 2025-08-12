use axum::{
    extract::{Path, Query, State, Request},
    http::{Method, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn, error};

use crate::config::ApiGatewayConfig;
use crate::error::{ApiGatewayError, ApiResult};
use crate::middleware::{MiddlewareState, RequestContext};
use crate::routing::{IntelligentRouter, OperationType, DirectOperation, WorkflowOperation};
use crate::temporal_client::{ApiGatewayTemporalClient, WorkflowExecutionResponse};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<ApiGatewayConfig>,
    pub router: Arc<IntelligentRouter>,
    pub temporal_client: Arc<ApiGatewayTemporalClient>,
    pub http_client: reqwest::Client,
    pub middleware_state: MiddlewareState,
}

/// Health check response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub services: HashMap<String, ServiceHealth>,
}

#[derive(Serialize)]
pub struct ServiceHealth {
    pub status: String,
    pub response_time_ms: Option<u64>,
}

/// Workflow request payload
#[derive(Deserialize)]
pub struct WorkflowRequest {
    #[serde(flatten)]
    pub payload: Value,
}

/// Query parameters for workflow status
#[derive(Deserialize)]
pub struct WorkflowStatusQuery {
    pub include_history: Option<bool>,
    pub include_progress: Option<bool>,
}

/// Health check handler
pub async fn health_handler(State(state): State<AppState>) -> ApiResult<Json<HealthResponse>> {
    let start_time = std::time::Instant::now();
    
    // Check service health
    let mut services = HashMap::new();
    
    // Check Temporal health
    let temporal_health = check_temporal_health(&state.temporal_client).await;
    services.insert("temporal".to_string(), temporal_health);
    
    // Check downstream services health
    for (service_name, service_config) in [
        ("auth", &state.config.services.auth_service),
        ("user", &state.config.services.user_service),
        ("tenant", &state.config.services.tenant_service),
        ("file", &state.config.services.file_service),
        ("workflow", &state.config.services.workflow_service),
    ] {
        let service_health = check_service_health(&state.http_client, service_config).await;
        services.insert(service_name.to_string(), service_health);
    }
    
    let response = HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        timestamp: chrono::Utc::now(),
        services,
    };
    
    debug!(
        duration_ms = start_time.elapsed().as_millis(),
        "Health check completed"
    );
    
    Ok(Json(response))
}

/// Main request handler - intelligent routing between direct calls and workflows
pub async fn handle_request(
    State(state): State<AppState>,
    request: Request,
) -> Result<Response, ApiGatewayError> {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    
    // Get request context
    let context = request.extensions().get::<RequestContext>().cloned()
        .unwrap_or_else(RequestContext::new);
    
    debug!(
        method = %method,
        path = %path,
        request_id = %context.request_id,
        "Processing request through intelligent router"
    );
    
    // Classify the operation
    let operation = state.router.classify_operation(&method, &path)?;
    
    match operation {
        OperationType::Direct(direct_op) => {
            handle_direct_operation(state, request, direct_op, &path, &context).await
        }
        OperationType::Workflow(workflow_op) => {
            handle_workflow_operation(state, request, workflow_op, &context).await
        }
    }
}

/// Handle direct operations by proxying to backend services
async fn handle_direct_operation(
    state: AppState,
    request: Request,
    operation: DirectOperation,
    path: &str,
    context: &RequestContext,
) -> Result<Response, ApiGatewayError> {
    debug!(
        operation = ?operation,
        path = path,
        request_id = %context.request_id,
        "Handling direct operation"
    );
    
    // Get service route
    let service_route = state.router.get_service_route(&operation, path)?;
    let target_url = state.router.build_service_url(&service_route, path);
    
    // Extract all needed information before consuming request
    let method_str = request.method().as_str().to_string();
    let headers = request.headers().clone();
    
    // Extract request body
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|e| ApiGatewayError::InvalidRequest {
            message: format!("Failed to read request body: {}", e),
        })?;
    
    // Build request to downstream service
    let reqwest_method = reqwest::Method::from_bytes(method_str.as_bytes())
        .map_err(|e| ApiGatewayError::InternalError {
            message: format!("Invalid HTTP method: {}", e),
        })?;
    
    let mut downstream_request = state.http_client
        .request(reqwest_method, &target_url)
        .timeout(state.config.service_timeout(&service_route.service_name));
    
    // Forward headers (excluding hop-by-hop headers)
    for (name, value) in &headers {
        if !is_hop_by_hop_header(name.as_str()) {
            if let Ok(value_str) = value.to_str() {
                downstream_request = downstream_request.header(name.as_str(), value_str);
            }
        }
    }
    
    // Add request ID for tracing
    downstream_request = downstream_request.header("X-Request-ID", &context.request_id);
    
    // Add tenant context if available
    if let Some(tenant_context) = &context.tenant_context {
        downstream_request = downstream_request.header("X-Tenant-ID", &tenant_context.tenant_id);
    }
    
    // Add body if present
    if !body_bytes.is_empty() {
        downstream_request = downstream_request.body(body_bytes);
    }
    
    // Execute request
    let start_time = std::time::Instant::now();
    let response = downstream_request.send().await
        .map_err(|e| {
            if e.is_timeout() {
                ApiGatewayError::ServiceTimeout {
                    service: service_route.service_name.clone(),
                }
            } else {
                ApiGatewayError::ServiceUnavailable {
                    service: service_route.service_name.clone(),
                }
            }
        })?;
    
    let duration = start_time.elapsed();
    
    info!(
        service = %service_route.service_name,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        request_id = %context.request_id,
        "Direct operation completed"
    );
    
    // Convert response
    let status_code = response.status().as_u16();
    let headers = response.headers().clone();
    let body = response.bytes().await
        .map_err(|e| ApiGatewayError::InternalError {
            message: format!("Failed to read response body: {}", e),
        })?;
    
    let axum_status = axum::http::StatusCode::from_u16(status_code)
        .map_err(|e| ApiGatewayError::InternalError {
            message: format!("Invalid status code: {}", e),
        })?;
    
    let mut axum_response = Response::builder().status(axum_status);
    
    // Forward response headers (excluding hop-by-hop headers)
    for (name, value) in headers {
        if let Some(name) = name {
            if !is_hop_by_hop_header(name.as_str()) {
                if let Ok(value_str) = value.to_str() {
                    axum_response = axum_response.header(name.as_str(), value_str);
                }
            }
        }
    }
    
    axum_response.body(axum::body::Body::from(body))
        .map_err(|e| ApiGatewayError::InternalError {
            message: format!("Failed to build response: {}", e),
        })
}

/// Handle workflow operations by initiating Temporal workflows
async fn handle_workflow_operation(
    state: AppState,
    request: Request,
    operation: WorkflowOperation,
    context: &RequestContext,
) -> Result<Response, ApiGatewayError> {
    debug!(
        operation = ?operation,
        request_id = %context.request_id,
        "Handling workflow operation"
    );
    
    // Get workflow route
    let workflow_route = state.router.get_workflow_route(&operation)?;
    
    // Extract request body as JSON
    let body_bytes = axum::body::to_bytes(request.into_body(), usize::MAX).await
        .map_err(|e| ApiGatewayError::InvalidRequest {
            message: format!("Failed to read request body: {}", e),
        })?;
    
    let workflow_input: Value = if body_bytes.is_empty() {
        serde_json::json!({})
    } else {
        serde_json::from_slice(&body_bytes)
            .map_err(|e| ApiGatewayError::InvalidRequest {
                message: format!("Invalid JSON in request body: {}", e),
            })?
    };
    
    // Get user and tenant context
    let tenant_id = context.tenant_context
        .as_ref()
        .map(|t| t.tenant_id.as_str())
        .unwrap_or("anonymous");
    let user_id = context.user_context
        .as_ref()
        .map(|u| u.user_id.as_str())
        .unwrap_or("anonymous");
    
    // Start workflow execution
    let start_time = std::time::Instant::now();
    let workflow_response = state.temporal_client
        .start_workflow(
            &workflow_route.workflow_type,
            None, // Let the client generate workflow ID
            &workflow_route.task_queue,
            workflow_input,
            tenant_id,
            user_id,
        )
        .await?;
    
    let duration = start_time.elapsed();
    
    info!(
        workflow_type = %workflow_route.workflow_type,
        task_queue = %workflow_route.task_queue,
        duration_ms = duration.as_millis(),
        request_id = %context.request_id,
        "Workflow operation initiated"
    );
    
    // Return appropriate response based on workflow type
    match workflow_response {
        WorkflowExecutionResponse::Synchronous { data, execution_time_ms, workflow_id } => {
            debug!(
                workflow_id = %workflow_id,
                execution_time_ms = execution_time_ms,
                "Synchronous workflow completed"
            );
            
            Ok(Json(data).into_response())
        }
        WorkflowExecutionResponse::Asynchronous { 
            operation_id, 
            status_url, 
            stream_url, 
            estimated_duration_seconds 
        } => {
            debug!(
                operation_id = %operation_id,
                estimated_duration_seconds = ?estimated_duration_seconds,
                "Asynchronous workflow started"
            );
            
            let response_body = serde_json::json!({
                "operation_id": operation_id,
                "status_url": status_url,
                "stream_url": stream_url,
                "estimated_duration_seconds": estimated_duration_seconds
            });
            
            let mut response = Json(response_body).into_response();
            *response.status_mut() = StatusCode::ACCEPTED;
            
            Ok(response)
        }
    }
}

/// Get workflow status handler
pub async fn get_workflow_status(
    State(state): State<AppState>,
    Path(operation_id): Path<String>,
    Query(query): Query<WorkflowStatusQuery>,
) -> ApiResult<Json<Value>> {
    debug!(
        operation_id = %operation_id,
        include_history = ?query.include_history,
        include_progress = ?query.include_progress,
        "Getting workflow status"
    );
    
    let status_response = state.temporal_client
        .get_workflow_status(&operation_id)
        .await?;
    
    let mut response = serde_json::to_value(&status_response)
        .map_err(|e| ApiGatewayError::InternalError {
            message: format!("Failed to serialize workflow status: {}", e),
        })?;
    
    // Add additional information based on query parameters
    if query.include_history.unwrap_or(false) {
        // TODO: Add workflow history when Temporal SDK supports it
        response["history"] = serde_json::json!([]);
    }
    
    if query.include_progress.unwrap_or(true) {
        // Progress is already included in the response
    }
    
    Ok(Json(response))
}

/// Cancel workflow handler
pub async fn cancel_workflow(
    State(state): State<AppState>,
    Path(operation_id): Path<String>,
    Json(payload): Json<Value>,
) -> ApiResult<Json<Value>> {
    let reason = payload.get("reason")
        .and_then(|r| r.as_str())
        .unwrap_or("Cancelled by user");
    
    debug!(
        operation_id = %operation_id,
        reason = reason,
        "Cancelling workflow"
    );
    
    state.temporal_client
        .cancel_workflow(&operation_id, reason)
        .await?;
    
    let response = serde_json::json!({
        "operation_id": operation_id,
        "status": "cancelled",
        "reason": reason,
        "cancelled_at": chrono::Utc::now()
    });
    
    Ok(Json(response))
}

/// Signal workflow handler
pub async fn signal_workflow(
    State(state): State<AppState>,
    Path((operation_id, signal_name)): Path<(String, String)>,
    Json(payload): Json<Value>,
) -> ApiResult<Json<Value>> {
    debug!(
        operation_id = %operation_id,
        signal_name = %signal_name,
        "Sending signal to workflow"
    );
    
    state.temporal_client
        .signal_workflow(&operation_id, &signal_name, payload.clone())
        .await?;
    
    let response = serde_json::json!({
        "operation_id": operation_id,
        "signal_name": signal_name,
        "signal_sent_at": chrono::Utc::now(),
        "payload": payload
    });
    
    Ok(Json(response))
}

/// Helper functions

async fn check_temporal_health(_temporal_client: &ApiGatewayTemporalClient) -> ServiceHealth {
    // For now, assume Temporal is healthy if client exists
    // This should be replaced with actual health check when SDK is stable
    ServiceHealth {
        status: "healthy".to_string(),
        response_time_ms: Some(1),
    }
}

async fn check_service_health(
    http_client: &reqwest::Client,
    service_config: &crate::config::ServiceEndpoint,
) -> ServiceHealth {
    let health_url = format!("{}/health", service_config.base_url);
    let start_time = std::time::Instant::now();
    
    match http_client
        .get(&health_url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            let duration = start_time.elapsed();
            if response.status().is_success() {
                ServiceHealth {
                    status: "healthy".to_string(),
                    response_time_ms: Some(duration.as_millis() as u64),
                }
            } else {
                ServiceHealth {
                    status: "unhealthy".to_string(),
                    response_time_ms: Some(duration.as_millis() as u64),
                }
            }
        }
        Err(_) => ServiceHealth {
            status: "unreachable".to_string(),
            response_time_ms: None,
        },
    }
}

fn is_hop_by_hop_header(name: &str) -> bool {
    matches!(
        name.to_lowercase().as_str(),
        "connection" | "keep-alive" | "proxy-authenticate" | "proxy-authorization" |
        "te" | "trailers" | "transfer-encoding" | "upgrade"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hop_by_hop_header_detection() {
        assert!(is_hop_by_hop_header("Connection"));
        assert!(is_hop_by_hop_header("connection"));
        assert!(is_hop_by_hop_header("Transfer-Encoding"));
        assert!(!is_hop_by_hop_header("Content-Type"));
        assert!(!is_hop_by_hop_header("Authorization"));
    }
}