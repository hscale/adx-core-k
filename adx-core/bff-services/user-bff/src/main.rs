use anyhow::Result;
use axum::{
    middleware::from_fn_with_state,
    routing::get,
    Router,
};
use std::net::SocketAddr;
// use tower_http::{
//     cors::{Any, CorsLayer},
// };
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod middleware;
mod routes;
mod services;
mod types;

use middleware::{auth::auth_middleware, error_handler::handle_error, tenant::tenant_middleware};
use routes::{aggregated, users, workflows};
use services::{api_client::ApiClient, redis::RedisService, temporal_client::TemporalClient};

#[derive(Clone)]
pub struct AppState {
    pub api_client: ApiClient,
    pub redis: RedisService,
    pub temporal_client: TemporalClient,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "user_bff=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize services
    let api_client = ApiClient::new().await?;
    let redis = RedisService::new().await?;
    let temporal_client = TemporalClient::new().await?;

    let state = AppState { 
        api_client, 
        redis, 
        temporal_client 
    };

    // Build the application router
    let app = create_app(state);

    // Start the server
    let addr = SocketAddr::from(([0, 0, 0, 0], 4004));
    info!("User BFF server starting on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app(state: AppState) -> Router {
    // Create API routes with authentication middleware
    let api_routes = Router::new()
        .nest("/users", users::create_routes())
        .nest("/workflows", workflows::create_routes())
        .nest("/aggregated", aggregated::create_routes())
        .layer(from_fn_with_state(
            state.clone(),
            auth_middleware,
        ))
        .layer(from_fn_with_state(
            state.clone(),
            tenant_middleware,
        ));

    Router::new()
        // Health check endpoint (no auth required)
        .route("/health", get(health_check))
        
        // API routes with authentication
        .nest("/api", api_routes)
        
        // Add global middleware layers (CORS can be added later)
        .with_state(state)
        .fallback(handle_error)
}

async fn health_check() -> &'static str {
    "User BFF Service is healthy"
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
        let state = AppState { api_client, redis, temporal_client };
        
        let app = create_app(state);
        let server = TestServer::new(app).unwrap();

        let response = server.get("/health").await;
        response.assert_status_ok();
        response.assert_text("User BFF Service is healthy");
    }
}