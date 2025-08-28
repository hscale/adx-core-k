use std::sync::Arc;
use anyhow::Result;
use tracing::info;

use adx_shared::config::AppConfig;

/// Auth Service HTTP Server
pub struct AuthServer {
    config: AppConfig,
}

impl AuthServer {
    /// Create a new Auth Service HTTP server
    pub async fn new(config: &AppConfig) -> Result<Self> {
        info!("Initializing Auth Service HTTP server");

        Ok(Self {
            config: config.clone(),
        })
    }

    /// Start the HTTP server
    pub async fn run(&self) -> Result<()> {
        info!(
            port = %self.config.server.port,
            "Starting Auth Service HTTP server"
        );

        // TODO: Implement actual HTTP server with Axum
        // For now, just simulate server running
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            info!("Auth Service HTTP server health check");
        }
    }
}