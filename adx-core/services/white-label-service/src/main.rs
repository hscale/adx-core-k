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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_loading() {
        let config = WhiteLabelConfig::default();
        assert_eq!(config.server_port, 8087);
        assert!(config.domain_config.max_domains_per_tenant > 0);
        assert!(!config.ssl_config.provider.is_empty());
    }
}