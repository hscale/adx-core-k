use clap::{Parser, Subcommand};
use adx_shared::{config::AppConfig, logging::init_logging};
use auth_service::{AuthServer, AuthWorker};

#[derive(Parser)]
#[command(name = "auth-service")]
#[command(about = "ADX Core Authentication Service")]
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
            tracing::info!("Starting Auth Service HTTP server on port {}", config.server.port);
            
            // Create and run HTTP server
            let server = AuthServer::new(&config).await?;
            server.run().await?;
        }
        Commands::Worker => {
            tracing::info!("Starting Auth Service Temporal worker");
            
            // Create and configure the worker
            let worker = AuthWorker::new(config).await?;
            
            // Register all workflows and activities
            worker.register_workflows_and_activities().await?;
            
            // Set up graceful shutdown handling
            let worker_arc = std::sync::Arc::new(worker);
            let shutdown_worker = worker_arc.clone();
            
            // Spawn shutdown handler
            tokio::spawn(async move {
                auth_service::worker::handle_shutdown_signal(shutdown_worker).await;
            });
            
            // Start the worker (this will run indefinitely)
            worker_arc.start().await?;
        }
    }
    
    Ok(())
}