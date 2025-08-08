use clap::{Parser, Subcommand};
use adx_shared::{config::AppConfig, logging::init_logging};

#[derive(Parser)]
#[command(name = "user-service")]
#[command(about = "ADX Core User Management Service")]
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
            tracing::info!("Starting User Service HTTP server on port {}", config.server.port + 1);
            // HTTP server implementation would go here
            tokio::signal::ctrl_c().await?;
        }
        Commands::Worker => {
            tracing::info!("Starting User Service Temporal worker");
            // Temporal worker implementation would go here
            tokio::signal::ctrl_c().await?;
        }
    }
    
    Ok(())
}