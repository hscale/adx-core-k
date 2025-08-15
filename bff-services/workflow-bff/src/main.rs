use anyhow::Result;
use axum::{
    http::{header, Method},
    middleware::{self, from_fn_with_state},
    routing::get,
    Router,
};
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    cors::{Any, CorsLayer},
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod middleware;
mod routes;
mod services;
mod types;

use middleware::{auth::auth_middleware, error_handler::handle_error, tenant::tenant_middleware};
use routes::{aggregated, monitoring, workflows};
use services::{api_client::ApiClient, redis::RedisService, temporal_client::TemporalClient, websocket::WebSocketService};

#[derive(Clone)]
pub struct AppState {
    pub api_client: ApiClient,
    pub redis: RedisService,
    pub temporal_client: TemporalClient,
    pub websocket: WebSocketService,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "workflow_bff=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize services
    let api_client = ApiClient::new().await?;
    let redis = RedisService::new().await?;
    let temporal_client = TemporalClient::new().await?;
    let websocket = WebSocketService::new();

    let state = AppState { 
        api_client, 
        redis, 
        temporal_client,
        websocket,
    };

    // Build the application router
    let app = create_app(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 4005));
    info!("Workflow BFF server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app(state: AppState) -> Router {
    Router::new()
        // Health check endpoint
        .route("/health", get(health_check))
        
        // Workflow routes
        .nest("/api/workflows", workflows::create_routes())
        
        // Monitoring routes
        .nest("/api/monitoring", monitoring::create_routes())
        
        // Aggregated data routes
        .nest("/api/aggregated", aggregated::create_routes())
        
        // Add middleware layers
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CompressionLayer::new())
                .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
                .layer(
                    CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any)
                        .allow_credentials(true),
                )
                .layer(from_fn_with_state(
                    state.clone(),
                    auth_middleware,
                ))
                .layer(from_fn_with_state(
                    state.clone(),
                    tenant_middleware,
                )),
        )
        .with_state(state)
        .fallback(handle_error)
}

async fn health_check() -> &'static str {
    "Workflow BFF Service is healthy"
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;

    #[tokio::test]
    async fn test_health_check() {
        let api_client = ApiClient::new().await.unwrap();
        let redis = RedisService::new().await.unwrap();
        let temporal_client = TemporalClient::new().await.unwrap();
        let websocket = WebSocketService::new();
        let state = AppState { api_client, redis, temporal_client, websocket };
        
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/health").await;
        response.assert_status_ok();
        response.assert_text("Workflow BFF Service is healthy");
    }
}