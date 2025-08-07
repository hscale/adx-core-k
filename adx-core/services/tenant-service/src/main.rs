use adx_shared::{init_tracing, ApiResponse, DatabaseManager, ResponseMetadata, TenantId};
use axum::{
    extract::{Json, Path, State},
    http::StatusCode,
    response::Json as ResponseJson,
    routing::{delete, get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

mod activities;
mod service;
mod types;
mod workflows;

use activities::*;
use service::*;
use types::*;
use workflows::*;

#[derive(Clone)]
pub struct AppState {
    tenant_service: Arc<TenantService>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    // Initialize database
    let db = DatabaseManager::new().await?;

    // Initialize tenant service
    let tenant_service = Arc::new(TenantService::new(Arc::new(db)).await?);

    let app_state = AppState { tenant_service };

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/tenants", post(create_tenant))
        .route("/api/v1/tenants", get(list_tenants))
        .route("/api/v1/tenants/:tenant_id", get(get_tenant))
        .route("/api/v1/tenants/:tenant_id", put(update_tenant))
        .route("/api/v1/tenants/:tenant_id", delete(delete_tenant))
        .route(
            "/api/v1/tenants/:tenant_id/status",
            put(update_tenant_status),
        )
        .route(
            "/api/v1/tenants/:tenant_id/provision",
            post(provision_tenant),
        )
        .route("/api/v1/tenants/:tenant_id/upgrade", post(upgrade_tenant))
        .route("/api/v1/tenants/:tenant_id/monitor", post(monitor_tenant))
        .route(
            "/api/v1/tenants/:tenant_id/billing",
            get(get_tenant_billing),
        )
        .route("/api/v1/tenants/:tenant_id/usage", get(get_tenant_usage))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8085").await?;
    tracing::info!("Tenant Service listening on 0.0.0.0:8085");

    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "Tenant Service OK"
}

/// Create a new tenant - triggers provisioning workflow
async fn create_tenant(
    State(app_state): State<AppState>,
    Json(request): Json<TenantCreationRequest>,
) -> Result<ResponseJson<ApiResponse<TenantCreationResponse>>, StatusCode> {
    match app_state
        .tenant_service
        .create_tenant_workflow(request)
        .await
    {
        Ok(response) => Ok(ResponseJson(ApiResponse {
            data: response,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// List all tenants with pagination
async fn list_tenants(
    State(app_state): State<AppState>,
    Json(query): Json<TenantListQuery>,
) -> Result<ResponseJson<ApiResponse<TenantListResponse>>, StatusCode> {
    match app_state.tenant_service.list_tenants(query).await {
        Ok(response) => Ok(ResponseJson(ApiResponse {
            data: response,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get tenant details
async fn get_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
) -> Result<ResponseJson<ApiResponse<TenantDetails>>, StatusCode> {
    match app_state.tenant_service.get_tenant(tenant_id).await {
        Ok(Some(tenant)) => Ok(ResponseJson(ApiResponse {
            data: tenant,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Update tenant information
async fn update_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
    Json(request): Json<TenantUpdateRequest>,
) -> Result<ResponseJson<ApiResponse<TenantDetails>>, StatusCode> {
    match app_state
        .tenant_service
        .update_tenant(tenant_id, request)
        .await
    {
        Ok(tenant) => Ok(ResponseJson(ApiResponse {
            data: tenant,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Delete tenant - triggers cleanup workflow
async fn delete_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
) -> Result<ResponseJson<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state
        .tenant_service
        .delete_tenant_workflow(tenant_id)
        .await
    {
        Ok(workflow_id) => Ok(ResponseJson(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Tenant deletion workflow started. All data will be permanently removed."
                    .to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Update tenant status (active, suspended, etc.)
async fn update_tenant_status(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
    Json(request): Json<TenantStatusUpdateRequest>,
) -> Result<ResponseJson<ApiResponse<TenantDetails>>, StatusCode> {
    match app_state
        .tenant_service
        .update_tenant_status(tenant_id, request.status)
        .await
    {
        Ok(tenant) => Ok(ResponseJson(ApiResponse {
            data: tenant,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Provision tenant resources - triggers provisioning workflow  
async fn provision_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
    Json(request): Json<TenantProvisioningInput>,
) -> Result<ResponseJson<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state
        .tenant_service
        .provision_tenant_workflow(tenant_id, request)
        .await
    {
        Ok(workflow_id) => Ok(ResponseJson(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Tenant provisioning workflow started. Resources will be allocated."
                    .to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Upgrade tenant plan - triggers upgrade workflow
async fn upgrade_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
    Json(request): Json<TenantUpgradeInput>,
) -> Result<ResponseJson<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state
        .tenant_service
        .upgrade_tenant_workflow(tenant_id, request)
        .await
    {
        Ok(workflow_id) => Ok(ResponseJson(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Tenant upgrade workflow started. Plan changes will be applied."
                    .to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Monitor tenant health - triggers monitoring workflow
async fn monitor_tenant(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
    Json(request): Json<TenantMonitoringInput>,
) -> Result<ResponseJson<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state
        .tenant_service
        .monitor_tenant_workflow(tenant_id, request)
        .await
    {
        Ok(workflow_id) => Ok(ResponseJson(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Tenant monitoring workflow started. Health checks will be performed."
                    .to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get tenant billing information
async fn get_tenant_billing(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
) -> Result<ResponseJson<ApiResponse<TenantBillingInfo>>, StatusCode> {
    match app_state.tenant_service.get_tenant_billing(tenant_id).await {
        Ok(billing) => Ok(ResponseJson(ApiResponse {
            data: billing,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get tenant usage statistics
async fn get_tenant_usage(
    State(app_state): State<AppState>,
    Path(tenant_id): Path<TenantId>,
) -> Result<ResponseJson<ApiResponse<TenantUsageStats>>, StatusCode> {
    match app_state.tenant_service.get_tenant_usage(tenant_id).await {
        Ok(usage) => Ok(ResponseJson(ApiResponse {
            data: usage,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStartResponse {
    pub workflow_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct TenantStatusUpdateRequest {
    pub status: TenantStatus,
}
