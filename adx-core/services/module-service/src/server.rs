use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

use crate::config::ModuleServiceConfig;
use crate::error::ModuleServiceError;
use crate::handlers::{
    module_handlers, installation_handlers, marketplace_handlers, 
    workflow_handlers, security_handlers, sdk_handlers
};
use crate::services::ModuleManager;
use crate::repositories::{ModuleRepository, InstallationRepository, MarketplaceRepository};

pub struct AppState {
    pub config: ModuleServiceConfig,
    pub module_manager: Arc<ModuleManager>,
    pub module_repo: Arc<ModuleRepository>,
    pub installation_repo: Arc<InstallationRepository>,
    pub marketplace_repo: Arc<MarketplaceRepository>,
}

pub async fn start_server(config: ModuleServiceConfig) -> Result<(), ModuleServiceError> {
    info!("Starting Module Service HTTP server on {}:{}", config.server.host, config.server.port);

    // Initialize database connection
    let database_url = &config.database.url;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .min_connections(config.database.min_connections)
        .connect(database_url)
        .await
        .map_err(|e| ModuleServiceError::DatabaseError(e))?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .map_err(|e| ModuleServiceError::DatabaseError(e))?;

    // Initialize repositories
    let module_repo = Arc::new(ModuleRepository::new(pool.clone()));
    let installation_repo = Arc::new(InstallationRepository::new(pool.clone()));
    let marketplace_repo = Arc::new(MarketplaceRepository::new(pool.clone()));

    // Initialize services
    let module_manager = Arc::new(ModuleManager::new(
        module_repo.clone(),
        installation_repo.clone(),
        // Other service dependencies would be injected here
    ));

    // Create application state
    let state = Arc::new(AppState {
        config: config.clone(),
        module_manager,
        module_repo,
        installation_repo,
        marketplace_repo,
    });

    // Build router
    let app = create_router(state);

    // Start server
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.server.host, config.server.port))
        .await
        .map_err(|e| ModuleServiceError::IoError(e))?;

    info!("Module Service listening on {}:{}", config.server.host, config.server.port);

    axum::serve(listener, app)
        .await
        .map_err(|e| ModuleServiceError::InternalError(e.to_string()))?;

    Ok(())
}

fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        // Module management endpoints
        .route("/api/v1/modules", get(module_handlers::list_modules))
        .route("/api/v1/modules", post(module_handlers::create_module))
        .route("/api/v1/modules/:id", get(module_handlers::get_module))
        .route("/api/v1/modules/:id", put(module_handlers::update_module))
        .route("/api/v1/modules/:id", delete(module_handlers::delete_module))
        .route("/api/v1/modules/search", post(module_handlers::search_modules))
        
        // Installation endpoints
        .route("/api/v1/installations", get(installation_handlers::list_installations))
        .route("/api/v1/installations", post(installation_handlers::install_module))
        .route("/api/v1/installations/:id", get(installation_handlers::get_installation))
        .route("/api/v1/installations/:id", delete(installation_handlers::uninstall_module))
        .route("/api/v1/installations/:id/activate", post(installation_handlers::activate_module))
        .route("/api/v1/installations/:id/deactivate", post(installation_handlers::deactivate_module))
        
        // Marketplace endpoints
        .route("/api/v1/marketplace/modules", get(marketplace_handlers::list_marketplace_modules))
        .route("/api/v1/marketplace/modules/:id", get(marketplace_handlers::get_marketplace_module))
        .route("/api/v1/marketplace/search", post(marketplace_handlers::search_marketplace))
        .route("/api/v1/marketplace/featured", get(marketplace_handlers::get_featured_modules))
        .route("/api/v1/marketplace/categories", get(marketplace_handlers::get_categories))
        
        // Workflow endpoints
        .route("/api/v1/workflows/install", post(workflow_handlers::install_module_workflow))
        .route("/api/v1/workflows/update", post(workflow_handlers::update_module_workflow))
        .route("/api/v1/workflows/uninstall", post(workflow_handlers::uninstall_module_workflow))
        .route("/api/v1/workflows/:id/status", get(workflow_handlers::get_workflow_status))
        .route("/api/v1/workflows/:id/cancel", post(workflow_handlers::cancel_workflow))
        
        // Security endpoints
        .route("/api/v1/security/scan", post(security_handlers::scan_module))
        .route("/api/v1/security/scans/:id", get(security_handlers::get_scan_results))
        .route("/api/v1/security/vulnerabilities", get(security_handlers::list_vulnerabilities))
        
        // SDK endpoints
        .route("/api/v1/sdk/templates", get(sdk_handlers::list_templates))
        .route("/api/v1/sdk/templates/:name", post(sdk_handlers::create_from_template))
        .route("/api/v1/sdk/validate", post(sdk_handlers::validate_module))
        .route("/api/v1/sdk/build", post(sdk_handlers::build_module))
        .route("/api/v1/sdk/test", post(sdk_handlers::test_module))
        .route("/api/v1/sdk/package", post(sdk_handlers::package_module))
        .route("/api/v1/sdk/publish", post(sdk_handlers::publish_module))
        
        // Health check
        .route("/health", get(health_check))
        .route("/ready", get(readiness_check))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

async fn health_check() -> Result<Json<serde_json::Value>, StatusCode> {
    Ok(Json(serde_json::json!({
        "status": "healthy",
        "service": "module-service",
        "version": "1.0.0",
        "timestamp": chrono::Utc::now()
    })))
}

async fn readiness_check(State(state): State<Arc<AppState>>) -> Result<Json<serde_json::Value>, StatusCode> {
    // Check database connectivity
    match sqlx::query("SELECT 1").fetch_one(&state.module_repo.pool).await {
        Ok(_) => Ok(Json(serde_json::json!({
            "status": "ready",
            "service": "module-service",
            "checks": {
                "database": "healthy"
            },
            "timestamp": chrono::Utc::now()
        }))),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}