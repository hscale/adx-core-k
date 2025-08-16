use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde::{Deserialize, Serialize};
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, error};
use uuid::Uuid;

use module_service::{
    ModuleServiceConfig, ModuleResult, ModuleError,
    runtime::ModuleServiceRuntime,
    InstallModuleRequest, UpdateModuleRequest, UninstallModuleRequest,
    ModuleSearchQuery, ModulePurchase, ModuleReview,
};

#[derive(Clone)]
struct AppState {
    runtime: Arc<ModuleServiceRuntime>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Load configuration
    let config = load_config().await?;
    
    // Initialize runtime
    let runtime = Arc::new(ModuleServiceRuntime::new(config.clone()).await?);
    
    // Start runtime
    runtime.start().await?;

    let app_state = AppState { runtime };

    // Build router
    let app = Router::new()
        // Module management endpoints
        .route("/api/v1/modules/install", post(install_module))
        .route("/api/v1/modules/:instance_id/update", put(update_module))
        .route("/api/v1/modules/:instance_id/uninstall", delete(uninstall_module))
        .route("/api/v1/modules/:instance_id/activate", post(activate_module))
        .route("/api/v1/modules/:instance_id/deactivate", post(deactivate_module))
        .route("/api/v1/modules/:instance_id/reload", post(hot_reload_module))
        
        // Module status and monitoring
        .route("/api/v1/modules/:instance_id/status", get(get_module_status))
        .route("/api/v1/modules/:instance_id/health", get(get_module_health))
        .route("/api/v1/modules/:instance_id/resources", get(get_module_resources))
        
        // Tenant module management
        .route("/api/v1/tenants/:tenant_id/modules", get(list_tenant_modules))
        
        // Marketplace endpoints
        .route("/api/v1/marketplace/search", post(search_marketplace))
        .route("/api/v1/marketplace/modules/:module_id", get(get_marketplace_module))
        .route("/api/v1/marketplace/featured", get(get_featured_modules))
        .route("/api/v1/marketplace/trending", get(get_trending_modules))
        .route("/api/v1/marketplace/purchase", post(purchase_module))
        
        // Review endpoints
        .route("/api/v1/marketplace/modules/:module_id/reviews", get(get_module_reviews))
        .route("/api/v1/marketplace/reviews", post(submit_module_review))
        
        // Workflow endpoints
        .route("/api/v1/workflows/install-module", post(install_module_workflow))
        .route("/api/v1/workflows/update-module", post(update_module_workflow))
        .route("/api/v1/workflows/uninstall-module", post(uninstall_module_workflow))
        .route("/api/v1/workflows/:operation_id/status", get(get_workflow_status))
        
        // Health check
        .route("/health", get(health_check))
        
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(app_state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await?;
    
    info!("Module Service listening on {}:{}", config.server.host, config.server.port);
    
    axum::serve(listener, app).await?;

    Ok(())
}

async fn load_config() -> Result<ModuleServiceConfig, Box<dyn std::error::Error>> {
    // Load configuration from environment variables or config file
    Ok(ModuleServiceConfig::default())
}

// Module management handlers

async fn install_module(
    State(state): State<AppState>,
    Json(request): Json<InstallModuleRequest>,
) -> Result<Json<ApiResponse<module_service::InstallModuleResult>>, ApiError> {
    match state.runtime.install_module(request).await {
        Ok(result) => Ok(Json(ApiResponse::success(result))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn update_module(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
    Json(mut request): Json<UpdateModuleRequest>,
) -> Result<Json<ApiResponse<module_service::UpdateModuleResult>>, ApiError> {
    request.instance_id = instance_id;
    match state.runtime.update_module(request).await {
        Ok(result) => Ok(Json(ApiResponse::success(result))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn uninstall_module(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
    Json(mut request): Json<UninstallModuleRequest>,
) -> Result<Json<ApiResponse<module_service::UninstallModuleResult>>, ApiError> {
    request.instance_id = instance_id;
    match state.runtime.uninstall_module(request).await {
        Ok(result) => Ok(Json(ApiResponse::success(result))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn activate_module(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    match state.runtime.activate_module(instance_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn deactivate_module(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    match state.runtime.deactivate_module(instance_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn hot_reload_module(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    match state.runtime.hot_reload_module(instance_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err(ApiError::from(e)),
    }
}

// Status and monitoring handlers

async fn get_module_status(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
) -> Result<Json<ApiResponse<module_service::ModuleStatus>>, ApiError> {
    match state.runtime.get_module_status(instance_id).await {
        Ok(status) => Ok(Json(ApiResponse::success(status))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn get_module_health(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
) -> Result<Json<ApiResponse<module_service::HealthStatus>>, ApiError> {
    match state.runtime.get_module_health(instance_id).await {
        Ok(health) => Ok(Json(ApiResponse::success(health))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn get_module_resources(
    State(state): State<AppState>,
    Path(instance_id): Path<Uuid>,
) -> Result<Json<ApiResponse<module_service::ResourceUsage>>, ApiError> {
    match state.runtime.get_module_resource_usage(instance_id).await {
        Ok(resources) => Ok(Json(ApiResponse::success(resources))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn list_tenant_modules(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<module_service::ModuleInstance>>>, ApiError> {
    match state.runtime.list_tenant_modules(&tenant_id).await {
        Ok(modules) => Ok(Json(ApiResponse::success(modules))),
        Err(e) => Err(ApiError::from(e)),
    }
}

// Marketplace handlers

async fn search_marketplace(
    State(state): State<AppState>,
    Json(query): Json<ModuleSearchQuery>,
) -> Result<Json<ApiResponse<module_service::ModuleSearchResult>>, ApiError> {
    match state.runtime.search_marketplace(&query).await {
        Ok(result) => Ok(Json(ApiResponse::success(result))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn get_marketplace_module(
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<ApiResponse<Option<module_service::ModuleMetadata>>>, ApiError> {
    match state.runtime.get_marketplace_module(&module_id).await {
        Ok(module) => Ok(Json(ApiResponse::success(module))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn get_featured_modules(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<module_service::ModuleMetadata>>>, ApiError> {
    match state.runtime.get_featured_modules().await {
        Ok(modules) => Ok(Json(ApiResponse::success(modules))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn get_trending_modules(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<module_service::ModuleMetadata>>>, ApiError> {
    match state.runtime.get_trending_modules().await {
        Ok(modules) => Ok(Json(ApiResponse::success(modules))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn purchase_module(
    State(state): State<AppState>,
    Json(purchase): Json<ModulePurchase>,
) -> Result<Json<ApiResponse<module_service::PurchaseResult>>, ApiError> {
    match state.runtime.purchase_module(&purchase).await {
        Ok(result) => Ok(Json(ApiResponse::success(result))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn get_module_reviews(
    State(state): State<AppState>,
    Path(module_id): Path<String>,
) -> Result<Json<ApiResponse<Vec<ModuleReview>>>, ApiError> {
    match state.runtime.get_module_reviews(&module_id).await {
        Ok(reviews) => Ok(Json(ApiResponse::success(reviews))),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn submit_module_review(
    State(state): State<AppState>,
    Json(review): Json<ModuleReview>,
) -> Result<Json<ApiResponse<()>>, ApiError> {
    match state.runtime.submit_module_review(&review).await {
        Ok(_) => Ok(Json(ApiResponse::success(()))),
        Err(e) => Err(ApiError::from(e)),
    }
}

// Workflow handlers

async fn install_module_workflow(
    State(state): State<AppState>,
    Json(request): Json<InstallModuleRequest>,
) -> Result<Json<WorkflowResponse<module_service::InstallModuleResult>>, ApiError> {
    // In a real implementation, this would initiate a Temporal workflow
    match state.runtime.install_module(request).await {
        Ok(result) => Ok(Json(WorkflowResponse::Synchronous {
            data: result,
            execution_time_ms: 1000,
            workflow_id: Uuid::new_v4().to_string(),
        })),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn update_module_workflow(
    State(state): State<AppState>,
    Json(request): Json<UpdateModuleRequest>,
) -> Result<Json<WorkflowResponse<module_service::UpdateModuleResult>>, ApiError> {
    match state.runtime.update_module(request).await {
        Ok(result) => Ok(Json(WorkflowResponse::Synchronous {
            data: result,
            execution_time_ms: 1000,
            workflow_id: Uuid::new_v4().to_string(),
        })),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn uninstall_module_workflow(
    State(state): State<AppState>,
    Json(request): Json<UninstallModuleRequest>,
) -> Result<Json<WorkflowResponse<module_service::UninstallModuleResult>>, ApiError> {
    match state.runtime.uninstall_module(request).await {
        Ok(result) => Ok(Json(WorkflowResponse::Synchronous {
            data: result,
            execution_time_ms: 1000,
            workflow_id: Uuid::new_v4().to_string(),
        })),
        Err(e) => Err(ApiError::from(e)),
    }
}

async fn get_workflow_status(
    Path(operation_id): Path<String>,
) -> Result<Json<WorkflowStatusResponse>, ApiError> {
    // In a real implementation, this would query Temporal for workflow status
    Ok(Json(WorkflowStatusResponse {
        operation_id,
        status: WorkflowStatus::Completed,
        progress: None,
        result: Some(serde_json::json!({"status": "completed"})),
        error: None,
        started_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
        estimated_completion: None,
    }))
}

async fn health_check() -> Json<HealthCheckResponse> {
    Json(HealthCheckResponse {
        status: "healthy".to_string(),
        timestamp: chrono::Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

// Response types

#[derive(Debug, Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
    timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
enum WorkflowResponse<T> {
    Synchronous {
        data: T,
        execution_time_ms: u64,
        workflow_id: String,
    },
    Asynchronous {
        operation_id: String,
        status_url: String,
        stream_url: Option<String>,
        estimated_duration_seconds: Option<u64>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowStatusResponse {
    operation_id: String,
    status: WorkflowStatus,
    progress: Option<WorkflowProgress>,
    result: Option<serde_json::Value>,
    error: Option<String>,
    started_at: chrono::DateTime<chrono::Utc>,
    updated_at: chrono::DateTime<chrono::Utc>,
    estimated_completion: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Serialize, Deserialize)]
struct WorkflowProgress {
    current_step: String,
    total_steps: u32,
    completed_steps: u32,
    percentage: f32,
    message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct HealthCheckResponse {
    status: String,
    timestamp: chrono::DateTime<chrono::Utc>,
    version: String,
}

// Error handling

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    message: String,
}

impl From<ModuleError> for ApiError {
    fn from(err: ModuleError) -> Self {
        let (status, message) = match err {
            ModuleError::NotFound(msg) => (StatusCode::NOT_FOUND, msg),
            ModuleError::AlreadyExists(msg) => (StatusCode::CONFLICT, msg),
            ModuleError::ValidationFailed(msg) => (StatusCode::BAD_REQUEST, msg),
            ModuleError::PermissionDenied(msg) => (StatusCode::FORBIDDEN, msg),
            ModuleError::SecurityScanFailed(msg) => (StatusCode::BAD_REQUEST, msg),
            ModuleError::PaymentError(msg) => (StatusCode::PAYMENT_REQUIRED, msg),
            ModuleError::NetworkError(msg) => (StatusCode::BAD_GATEWAY, msg),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()),
        };

        Self { status, message }
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let body = Json(ApiResponse::<()>::error(self.message));
        (self.status, body).into_response()
    }
}