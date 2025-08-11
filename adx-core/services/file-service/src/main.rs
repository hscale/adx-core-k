use clap::{Parser, Subcommand};
use adx_shared::{config::AppConfig, logging::init_logging};

mod models;
mod repositories;
mod handlers;
mod server;
mod worker;
mod activities;
mod workflows;
mod storage;
mod services;

use server::start_server;
use worker::start_worker;

#[derive(Parser)]
#[command(name = "file-service")]
#[command(about = "ADX Core File Management Service")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start HTTP server mode
    Server,
    /// Start Temporal worker mode
    Worker,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = AppConfig::load()?;
    
    init_logging(&config.logging)?;
    
    match cli.command {
        Commands::Server => {
            tracing::info!("Starting File Service HTTP server on port {}", config.server.port + 2);
            start_server(config).await?;
        }
        Commands::Worker => {
            tracing::info!("Starting File Service Temporal worker");
            start_worker(config).await?;
        }
    }
    
    Ok(())
}