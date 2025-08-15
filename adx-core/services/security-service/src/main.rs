use anyhow::Result;
use clap::{Parser, Subcommand};
use security_service::{
    config::SecurityConfig,
    server::SecurityServer,
    worker::SecurityWorker,
};
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser)]
#[command(name = "security-service")]
#[command(about = "ADX Core Security and Compliance Service")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start HTTP server for direct endpoints
    Server,
    /// Start Temporal workflow worker
    Worker,
    /// Run both server and worker
    Both,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "security_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let config = SecurityConfig::from_env()?;

    match cli.command {
        Commands::Server => {
            info!("Starting Security Service HTTP server on port {}", config.server.port);
            let server = SecurityServer::new(config).await?;
            server.run().await?;
        }
        Commands::Worker => {
            info!("Starting Security Service Temporal worker");
            let worker = SecurityWorker::new(config).await?;
            worker.run().await?;
        }
        Commands::Both => {
            info!("Starting Security Service in dual mode (server + worker)");
            let server_config = config.clone();
            let worker_config = config;

            let server_handle = tokio::spawn(async move {
                let server = SecurityServer::new(server_config).await?;
                server.run().await
            });

            let worker_handle = tokio::spawn(async move {
                let worker = SecurityWorker::new(worker_config).await?;
                worker.run().await
            });

            tokio::select! {
                result = server_handle => {
                    if let Err(e) = result? {
                        error!("Server error: {}", e);
                    }
                }
                result = worker_handle => {
                    if let Err(e) = result? {
                        error!("Worker error: {}", e);
                    }
                }
            }
        }
    }

    Ok(())
}