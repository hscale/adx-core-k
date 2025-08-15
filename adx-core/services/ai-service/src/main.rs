use ai_service::{create_app, start_worker, Config};
use clap::{Parser, Subcommand};
use std::env;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "ai-service")]
#[command(about = "ADX Core AI Service - Temporal-first AI workflow orchestration")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start HTTP server mode
    Server {
        #[arg(short, long, default_value = "8086")]
        port: u16,
    },
    /// Start Temporal worker mode
    Worker {
        #[arg(short, long, default_value = "ai-task-queue")]
        task_queue: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            env::var("RUST_LOG")
                .unwrap_or_else(|_| "ai_service=info,tower_http=debug".to_string()),
        )
        .init();

    // Load configuration
    let config = Config::from_env()?;
    
    let cli = Cli::parse();
    
    match cli.command {
        Some(Commands::Server { port }) => {
            info!("Starting AI Service HTTP server on port {}", port);
            start_server(config, port).await
        }
        Some(Commands::Worker { task_queue }) => {
            info!("Starting AI Service Temporal worker with task queue: {}", task_queue);
            start_worker(config, &task_queue).await
        }
        None => {
            // Default behavior based on environment or arguments
            let mode = env::args().nth(1).unwrap_or_else(|| "server".to_string());
            
            match mode.as_str() {
                "worker" => {
                    info!("Starting AI Service Temporal worker (default task queue)");
                    start_worker(config, "ai-task-queue").await
                }
                _ => {
                    info!("Starting AI Service HTTP server (default port 8086)");
                    start_server(config, 8086).await
                }
            }
        }
    }
}

async fn start_server(config: Config, port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(config).await?;
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("AI Service HTTP server listening on port {}", port);
    
    axum::serve(listener, app).await?;
    Ok(())
}