use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use white_label_service::{config::WhiteLabelConfig, server};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "white_label_service=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration
    let config = Arc::new(WhiteLabelConfig::default());

    tracing::info!("Starting White Label Service");

    // Start HTTP server
    server::start_server(config.server_port).await?;

    Ok(())
}