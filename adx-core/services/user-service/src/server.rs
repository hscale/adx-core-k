use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use std::time::Duration;
use sqlx::PgPool;
use adx_shared::{
    config::AppConfig,
    middleware::{tenant_context_middleware, user_context_middleware},
    health::health_check as shared_health_check,
};
use crate::{
    handlers::*,
    repositories::*,
    validation::UserValidator,
};

pub async fn create_app(config: &AppConfig, pool: PgPool) -> Router {
    // Create repositories
    let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
    let profile_repo = Arc::new(PostgresUserProfileRepository::new(pool.clone()));
    let preference_repo = Arc::new(PostgresUserPreferenceRepository::new(pool.clone()));
    let activity_repo = Arc::new(PostgresUserActivityRepository::new(pool.clone()));
    let validator = Arc::new(UserValidator::new());
    
    // Create application state
    let state = UserServiceState {
        user_repo,
        profile_repo,
        preference_repo,
        activity_repo,
        validator,
    };
    
    // Create router with routes
    Router::new()
        // Health check (no auth required)
        .route("/health", get(shared_health_check))
        .route("/api/v1/health", get(health_check))
        
        // User CRUD routes
        .route("/api/v1/users", post(create_user))
        .route("/api/v1/users", get(list_users))
        .route("/api/v1/users/:user_id", get(get_user))
        .route("/api/v1/users/:user_id", put(update_user))
        .route("/api/v1/users/:user_id", delete(delete_user))
        
        // User profile routes
        .route("/api/v1/users/:user_id/profile", get(get_user_profile))
        .route("/api/v1/users/:user_id/profile", post(create_user_profile))
        .route("/api/v1/users/:user_id/profile", put(update_user_profile))
        
        // User preferences routes
        .route("/api/v1/users/:user_id/preferences", get(get_user_preferences))
        .route("/api/v1/users/:user_id/preferences", post(set_user_preferences))
        
        // User search and directory routes
        .route("/api/v1/users/search", get(search_users))
        .route("/api/v1/users/directory", get(get_user_directory))
        
        // User activity routes
        .route("/api/v1/users/:user_id/activity", get(get_user_activity))
        
        // Workflow routes
        .route("/api/v1/workflows/user-profile-sync", post(start_user_profile_sync_workflow))
        .route("/api/v1/workflows/user-preference-migration", post(start_user_preference_migration_workflow))
        .route("/api/v1/workflows/user-data-export", post(start_user_data_export_workflow))
        .route("/api/v1/workflows/user-deactivation", post(start_user_deactivation_workflow))
        .route("/api/v1/workflows/user-reactivation", post(start_user_reactivation_workflow))
        .route("/api/v1/workflows/bulk-user-operation", post(start_bulk_user_operation_workflow))
        
        // Add middleware
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(tenant_context_middleware))
                .layer(middleware::from_fn(user_context_middleware))
        )
        .with_state(state)
}

pub async fn start_server(config: AppConfig, pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(&config, pool).await;
    
    let port = config.server.port + 1; // User service runs on port 8082 (base + 1)
    let addr = format!("0.0.0.0:{}", port);
    
    tracing::info!("User Service HTTP server starting on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}