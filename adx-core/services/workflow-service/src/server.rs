use crate::{
    config::WorkflowServiceConfig,
    error::{WorkflowServiceError, WorkflowServiceResult},
    handlers::*,
};
use axum::{
    extract::Extension,
    http::{header, Method, StatusCode},
    middleware,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc, time::Duration};
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::info;

pub struct WorkflowServer {
    config: WorkflowServiceConfig,
    app: Router,
}

impl WorkflowServer {
    pub fn new(config: WorkflowServiceConfig) -> Self {
        let app = create_app(config.clone());
        
        Self { config, app }
    }

    pub async fn run(self) -> WorkflowServiceResult<()> {
        let addr = SocketAddr::from(([0, 0, 0, 0], self.config.server.port));
        
        info!("Starting Workflow Service HTTP server on {}", addr);
        
        let listener = tokio::net::TcpListener::bind(&addr).await
            .map_err(|e| WorkflowServiceError::Internal(format!("Failed to bind to address: {}", e)))?;
        
        axum::serve(listener, self.app)
            .await
            .map_err(|e| WorkflowServiceError::Internal(format!("Server error: {}", e)))?;

        Ok(())
    }
}

fn create_app(config: WorkflowServiceConfig) -> Router {
    let config = Arc::new(config);

    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // Workflow endpoints
        .route("/api/v1/workflows/user-onboarding", post(start_user_onboarding_workflow))
        .route("/api/v1/workflows/tenant-switching", post(start_tenant_switching_workflow))
        .route("/api/v1/workflows/data-migration", post(start_data_migration_workflow))
        .route("/api/v1/workflows/bulk-operation", post(start_bulk_operation_workflow))
        .route("/api/v1/workflows/compliance", post(start_compliance_workflow))
        
        // Workflow status endpoints
        .route("/api/v1/workflows/:workflow_id/status", get(get_workflow_status))
        .route("/api/v1/workflows/:workflow_id/status/detailed", get(get_workflow_status_detailed))
        .route("/api/v1/workflows/:workflow_id/debug", get(get_workflow_debug_info))
        .route("/api/v1/workflows/:workflow_id/cancel", post(cancel_workflow))
        .route("/api/v1/workflows/:workflow_id/retry", post(retry_workflow))
        
        // Enhanced workflow management
        .route("/api/v1/workflows/:workflow_id/cancel-enhanced", post(cancel_workflow_enhanced))
        .route("/api/v1/workflows/:workflow_id/retry-enhanced", post(retry_workflow_enhanced))
        .route("/api/v1/workflows/:workflow_id/pause", post(pause_workflow))
        .route("/api/v1/workflows/:workflow_id/resume", post(resume_workflow))
        .route("/api/v1/workflows/:workflow_id/terminate", post(terminate_workflow))
        .route("/api/v1/workflows/:workflow_id/management-options", get(get_workflow_management_options))
        .route("/api/v1/workflows/bulk-operation", post(bulk_workflow_operation))
        
        // Workflow listing and management
        .route("/api/v1/workflows", get(list_workflows))
        .route("/api/v1/workflows/history", get(get_workflow_history))
        .route("/api/v1/workflows/analytics", get(get_workflow_analytics))
        .route("/api/v1/workflows/health", get(get_workflow_health_report))
        
        // Workflow versioning endpoints
        .route("/api/v1/workflow-versions/register", post(register_workflow_version))
        .route("/api/v1/workflow-versions/:workflow_type", get(get_workflow_versions))
        .route("/api/v1/workflow-versions/:workflow_type/compatibility", get(get_compatibility_matrix))
        .route("/api/v1/workflow-versions/migrate", post(migrate_workflows))
        .route("/api/v1/workflow-versions/migrations/:migration_id/status", get(get_migration_status))
        .route("/api/v1/workflow-versions/migrations/rollback", post(rollback_migration))
        .route("/api/v1/workflow-versions/deprecate", post(deprecate_version))
        
        // Workflow template endpoints
        .route("/api/v1/workflow-templates", get(get_workflow_templates))
        .route("/api/v1/workflow-templates", post(create_workflow_template))
        .route("/api/v1/workflow-templates/:template_id", get(get_workflow_template))
        .route("/api/v1/workflow-templates/:template_id", put(update_workflow_template))
        .route("/api/v1/workflow-templates/:template_id", delete(delete_workflow_template))
        .route("/api/v1/workflow-templates/:template_id/usage", get(get_template_usage))
        .route("/api/v1/workflow-templates/create-from", post(create_workflow_from_template))
        .route("/api/v1/workflow-templates/generate", post(generate_template_from_workflows))
        .route("/api/v1/workflow-templates/analyze-patterns", get(analyze_workflow_patterns))
        
        // Service coordination endpoints
        .route("/api/v1/coordination/health-check", post(coordinate_health_check))
        .route("/api/v1/coordination/backup", post(create_cross_service_backup))
        .route("/api/v1/coordination/restore", post(restore_from_backup))
        
        // Add middleware
        .layer(Extension(config))
        .layer(middleware::from_fn(tenant_context_middleware))
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({
        "status": "healthy",
        "service": "workflow-service",
        "timestamp": chrono::Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}

async fn readiness_check(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check Temporal connectivity
    let temporal_healthy = check_temporal_connectivity(&config.temporal.server_url).await;
    
    // Check service dependencies
    let services_healthy = check_service_dependencies(&config.services).await;
    
    let ready = temporal_healthy && services_healthy;
    
    if ready {
        Ok(Json(json!({
            "status": "ready",
            "service": "workflow-service",
            "timestamp": chrono::Utc::now(),
            "checks": {
                "temporal": temporal_healthy,
                "services": services_healthy
            }
        })))
    } else {
        Err(StatusCode::SERVICE_UNAVAILABLE)
    }
}

async fn check_temporal_connectivity(temporal_url: &str) -> bool {
    // Mock implementation - would check actual Temporal connectivity
    let client = reqwest::Client::new();
    client
        .get(&format!("{}/api/v1/namespaces", temporal_url))
        .timeout(Duration::from_secs(5))
        .send()
        .await
        .map(|r| r.status().is_success())
        .unwrap_or(false)
}

async fn check_service_dependencies(services: &crate::config::ServiceEndpoints) -> bool {
    let client = reqwest::Client::new();
    let timeout = Duration::from_secs(2);
    
    let checks = vec![
        client.get(&format!("{}/health", services.auth_service)).timeout(timeout).send(),
        client.get(&format!("{}/health", services.user_service)).timeout(timeout).send(),
        client.get(&format!("{}/health", services.tenant_service)).timeout(timeout).send(),
        client.get(&format!("{}/health", services.file_service)).timeout(timeout).send(),
    ];
    
    let results = futures::future::join_all(checks).await;
    results.iter().all(|r| r.as_ref().map(|resp| resp.status().is_success()).unwrap_or(false))
}

async fn tenant_context_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Result<axum::response::Response, StatusCode> {
    // Extract tenant ID from headers
    let tenant_id = req
        .headers()
        .get("X-Tenant-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // Extract user ID from headers
    let user_id = req
        .headers()
        .get("X-User-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string());
    
    // For workflow endpoints, tenant_id is required
    if req.uri().path().starts_with("/api/v1/workflows") && tenant_id.is_none() {
        return Err(StatusCode::BAD_REQUEST);
    }
    
    // Add context to request extensions
    let mut req = req;
    if let Some(tenant_id) = tenant_id {
        req.extensions_mut().insert(TenantContext { tenant_id, user_id });
    }
    
    Ok(next.run(req).await)
}

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub user_id: Option<String>,
}