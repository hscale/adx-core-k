use axum::Router;
use std::net::SocketAddr;
use tokio::net::TcpListener;

use adx_shared::{
    config::AppConfig,
    auth::JwtManager,
    database::{create_database_pool_with_config, DatabaseConfig},
    Error, Result,
};
use crate::{
    routes::create_versioned_routes,
    middleware::rate_limit::RateLimiter,
    AppState,
};

pub struct AuthServer {
    app: Router,
    addr: SocketAddr,
}

impl AuthServer {
    pub async fn new(config: &AppConfig) -> Result<Self> {
        // Initialize JWT manager
        let jwt_secret = config.auth.jwt_secret.clone()
            .unwrap_or_else(|| "default-secret-change-in-production".to_string());
        let jwt_manager = JwtManager::new(&jwt_secret);

        // Initialize rate limiter
        let rate_limiter = RateLimiter::new();

        // Initialize database connection pool
        let db_config = DatabaseConfig::from_env();
        let database_pool = create_database_pool_with_config(db_config).await?;

        // TODO: Initialize Redis connection

        // Create application state
        let state = AppState {
            config: config.clone(),
            jwt_manager,
            rate_limiter,
            database_pool,
            // redis_client: redis,
        };

        // Create routes
        let app = create_versioned_routes(state);

        // Parse server address
        let addr = format!("{}:{}", config.server.host, config.server.port)
            .parse()
            .map_err(|e| Error::Configuration(format!("Invalid server address: {}", e)))?;

        Ok(Self { app, addr })
    }

    pub async fn run(self) -> Result<()> {
        tracing::info!("Starting Auth Service HTTP server on {}", self.addr);

        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| Error::Network(format!("Failed to bind to address: {}", e)))?;

        tracing::info!("Auth Service HTTP server listening on {}", self.addr);

        axum::serve(listener, self.app)
            .await
            .map_err(|e| Error::Network(format!("Server error: {}", e)))?;

        Ok(())
    }
}

// Tests disabled for now due to SQLx macro compilation issues
// TODO: Re-enable tests once DATABASE_URL is properly configured
// #[cfg(test)]
// mod tests { ... }