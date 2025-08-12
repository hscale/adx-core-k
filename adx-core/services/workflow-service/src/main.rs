use clap::{Parser, Subcommand};
use adx_shared::{config::AppConfig, logging::init_logging};
use workflow_service::{
    config::WorkflowServiceConfig,
    server::WorkflowServer,
    worker::WorkflowWorker,
    error::WorkflowServiceResult,
};
use tracing::{info, error};

#[derive(Parser)]
#[command(name = "workflow-service")]
#[command(about = "ADX Core Cross-Service Workflow Orchestration Service")]
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
    let app_config = AppConfig::load()?;
    
    init_logging(&app_config.logging)?;
    
    // Load workflow service specific configuration
    let workflow_config = load_workflow_config()?;
    
    match cli.command {
        Commands::Server => {
            info!("Starting Workflow Service HTTP server on port {}", workflow_config.server.port);
            
            let server = WorkflowServer::new(workflow_config);
            if let Err(e) = server.run().await {
                error!("Server error: {}", e);
                return Err(e.into());
            }
        }
        Commands::Worker => {
            info!("Starting Workflow Service Temporal worker");
            
            let worker = WorkflowWorker::new(workflow_config);
            if let Err(e) = worker.start().await {
                error!("Worker error: {}", e);
                return Err(e.into());
            }
        }
    }
    
    Ok(())
}

fn load_workflow_config() -> WorkflowServiceResult<WorkflowServiceConfig> {
    // In a real implementation, this would load from configuration files
    // For now, use default configuration
    Ok(WorkflowServiceConfig::default())
}