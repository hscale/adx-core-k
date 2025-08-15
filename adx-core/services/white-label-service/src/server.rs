use crate::config::WhiteLabelConfig;
use crate::handlers::{create_routes, AppState};
use axum::{
    http::{
        header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
        HeaderValue, Method,
    },
    Router,
};
use sqlx::PgPool;
use std::sync::Arc;
use crate::temporal_mock::TemporalClient;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

pub struct WhiteLabelServer {
    config: Arc<WhiteLabelConfig>,
    db_pool: Arc<PgPool>,
    temporal_client: Arc<TemporalClient>,
}

impl WhiteLabelServer {
    pub fn new(
        config: Arc<WhiteLabelConfig>,
        db_pool: Arc<PgPool>,
        temporal_client: Arc<TemporalClient>,
    ) -> Self {
        Self {
            config,
            db_pool,
            temporal_client,
        }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        let app_state = AppState {
            db_pool: self.db_pool.clone(),
            temporal_client: self.temporal_client.clone(),
        };

        let app = self.create_app(app_state);

        let addr = format!("0.0.0.0:{}", self.config.server_port);
        tracing::info!("White Label Service starting on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }

    fn create_app(&self, state: AppState) -> Router {
        let cors = CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE])
            .allow_credentials(true);

        let trace_layer = TraceLayer::new_for_http()
            .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
            .on_request(DefaultOnRequest::new().level(Level::INFO))
            .on_response(DefaultOnResponse::new().level(Level::INFO));

        Router::new()
            .nest("/api/v1/white-label", create_routes())
            .layer(
                ServiceBuilder::new()
                    .layer(cors)
                    .layer(trace_layer)
                    .into_inner(),
            )
            .with_state(state)
    }
}

pub async fn create_database_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

pub async fn create_temporal_client(server_url: &str) -> Result<TemporalClient, Box<dyn std::error::Error>> {
    // In a real implementation, this would create a proper Temporal client
    // For now, we'll create a mock client
    tracing::info!("Connecting to Temporal server at {}", server_url);
    
    // This is a placeholder - in a real implementation, you would use:
    // let client = TemporalClient::new(server_url).await?;
    
    // For now, we'll create a mock client that satisfies the type system
    TemporalClient::new(server_url).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum_test::TestServer;
    use serde_json::json;

    #[tokio::test]
    async fn test_health_check() {
        let config = Arc::new(WhiteLabelConfig::default());
        let db_pool = Arc::new(create_test_database_pool().await);
        let temporal_client = Arc::new(create_mock_temporal_client());
        
        let server = WhiteLabelServer::new(config, db_pool, temporal_client);
        let app_state = AppState {
            db_pool: server.db_pool.clone(),
            temporal_client: server.temporal_client.clone(),
        };
        
        let app = server.create_app(app_state);
        let test_server = TestServer::new(app).unwrap();

        let response = test_server.get("/api/v1/white-label/health").await;
        response.assert_status_ok();
        
        let body: serde_json::Value = response.json();
        assert_eq!(body["status"], "healthy");
        assert_eq!(body["service"], "white-label-service");
    }

    async fn create_test_database_pool() -> PgPool {
        // This would create a test database pool
        // For now, we'll use an in-memory SQLite database for testing
        sqlx::SqlitePool::connect(":memory:").await.unwrap()
    }

    fn create_mock_temporal_client() -> TemporalClient {
        // This would create a mock Temporal client for testing
        todo!("Implement mock Temporal client for tests")
    }
}