use clap::{Parser, Subcommand};
use adx_shared::{config::AppConfig, logging::init_logging, database::create_connection_pool};
use tenant_service::{server, worker};

#[derive(Parser)]
#[command(name = "tenant-service")]
#[command(about = "ADX Core Tenant Management Service - Dual-Mode Architecture")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start HTTP server mode for direct endpoints (port 8085)
    Server,
    /// Start Temporal worker mode for workflow execution
    Worker,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    let config = AppConfig::load()?;
    
    init_logging(&config.logging)?;
    
    // Create database connection pool
    let pool = create_connection_pool(&config.database).await?;
    
    match cli.command.unwrap_or(Commands::Server) {
        Commands::Server => {
            let port = 8085; // Fixed port for tenant service
            tracing::info!("🚀 Starting Tenant Service HTTP server (dual-mode) on port {}", port);
            tracing::info!("📋 Available endpoints:");
            tracing::info!("   • GET  /health - Health check");
            tracing::info!("   • POST /api/v1/tenants - Create tenant (direct)");
            tracing::info!("   • GET  /api/v1/tenants - List tenants");
            tracing::info!("   • GET  /api/v1/tenants/:id - Get tenant");
            tracing::info!("   • PUT  /api/v1/tenants/:id - Update tenant");
            tracing::info!("   • DELETE /api/v1/tenants/:id - Delete tenant");
            tracing::info!("   • POST /api/v1/tenant/switch - Switch tenant context");
            tracing::info!("   • Membership management endpoints");
            server::start_server(config, pool).await?;
        }
        Commands::Worker => {
            tracing::info!("🔄 Starting Tenant Service Temporal worker");
            tracing::info!("📋 Registered workflows:");
            tracing::info!("   • create_tenant_workflow - Complex tenant creation");
            tracing::info!("   • switch_tenant_workflow - Complex tenant switching");
            tracing::info!("   • migrate_tenant_workflow - Tenant migration");
            tracing::info!("   • suspend_tenant_workflow - Tenant suspension");
            tracing::info!("   • terminate_tenant_workflow - Tenant termination");
            worker::start_worker(config, pool).await?;
        }
    }
    
    Ok(())
}