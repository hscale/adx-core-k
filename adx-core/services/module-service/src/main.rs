use clap::{Parser, Subcommand};
use std::env;
use tracing::{info, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod error;
mod types;
mod models;
mod handlers;
mod server;
mod worker;
mod activities;
mod workflows;
mod services;
mod repositories;
mod marketplace;
mod sandbox;
mod sdk;

use config::ModuleServiceConfig;
use error::ModuleServiceError;

#[derive(Parser)]
#[command(name = "module-service")]
#[command(about = "ADX Core Module Service - Comprehensive module management with Temporal workflows")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start HTTP server mode
    Server,
    /// Start Temporal worker mode
    Worker,
    /// Start marketplace indexer
    Indexer,
    /// Run module security scanner
    Scanner,
}

#[tokio::main]
async fn main() -> Result<(), ModuleServiceError> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "module_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();
    let config = ModuleServiceConfig::from_env()?;

    match cli.command.unwrap_or(Commands::Server) {
        Commands::Server => {
            info!("Starting Module Service HTTP server on port {}", config.server.port);
            server::start_server(config).await
        }
        Commands::Worker => {
            info!("Starting Module Service Temporal worker");
            worker::start_worker(config).await
        }
        Commands::Indexer => {
            info!("Starting Module Marketplace indexer");
            marketplace::indexer::start_indexer(config).await
        }
        Commands::Scanner => {
            info!("Starting Module Security scanner");
            sandbox::scanner::start_scanner(config).await
        }
    }
}