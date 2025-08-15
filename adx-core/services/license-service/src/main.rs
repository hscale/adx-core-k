use std::sync::Arc;

use axum::Server;
use clap::{Arg, Command};
use sqlx::PgPool;
use tokio::signal;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use license_service::{
    billing::BillingService,
    config::LicenseConfig,
    handlers::{create_router, AppState},
    repositories::{LicenseRepository, QuotaRepository, BillingRepository, ComplianceRepository},
    services::LicenseService,
    LicenseError, Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "license_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Parse command line arguments
    let matches = Command::new("license-service")
        .version("1.0.0")
        .about("ADX Core License and Quota Management Service")
        .arg(
            Arg::new("mode")
                .long("mode")
                .value_name("MODE")
                .help("Service mode: server or worker")
                .default_value("server")
                .value_parser(["server", "worker"])
        )
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("Configuration file path")
        )
        .get_matches();

    let mode = matches.get_one::<String>("mode").unwrap();

    // Load configuration
    dotenvy::dotenv().ok();
    let config = LicenseConfig::from_env()
        .map_err(|e| LicenseError::ConfigError(format!("Failed to load config: {}", e)))?;

    info!("Starting license service in {} mode", mode);
    info!("Configuration loaded: server_port={}", config.server_port);

    match mode.as_str() {
        "server" => run_server(config).await,
        "worker" => run_worker(config).await,
        _ => {
            warn!("Unknown mode: {}", mode);
            std::process::exit(1);
        }
    }
}

async fn run_server(config: LicenseConfig) -> Result<()> {
    info!("Starting HTTP server on port {}", config.server_port);

    // Initialize database connection
    let database_pool = PgPool::connect(&config.database_url)
        .await
        .map_err(|e| LicenseError::Database(e))?;

    // Run database migrations
    sqlx::migrate!("./migrations")
        .run(&database_pool)
        .await
        .map_err(|e| LicenseError::Database(e))?;

    info!("Database migrations completed");

    // Initialize repositories
    let license_repo = LicenseRepository::new(database_pool.clone());
    let quota_repo = QuotaRepository::new(database_pool.clone());
    let billing_repo = BillingRepository::new(database_pool.clone());
    let compliance_repo = ComplianceRepository::new(database_pool.clone());

    // Initialize billing service
    let billing_service = BillingService::new(
        Some(config.stripe.clone()),
        Some(config.paypal.clone()),
        config.billing.clone(),
    );

    // Initialize license service
    let license_service = LicenseService::new(
        license_repo,
        quota_repo,
        billing_repo,
        compliance_repo,
        billing_service,
    );

    // Create application state
    let app_state = AppState {
        license_service,
    };

    // Create router with middleware
    let app = create_router(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(TimeoutLayer::from_secs(30))
        );

    // Start server
    let addr = format!("0.0.0.0:{}", config.server_port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .map_err(|e| LicenseError::Internal(format!("Failed to bind to {}: {}", addr, e)))?;

    info!("License service HTTP server listening on {}", addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| LicenseError::Internal(format!("Server error: {}", e)))?;

    Ok(())
}

async fn run_worker(config: LicenseConfig) -> Result<()> {
    info!("Starting Temporal worker");

    // Initialize database connection
    let database_pool = PgPool::connect(&config.database_url)
        .await
        .map_err(|e| LicenseError::Database(e))?;

    // Initialize repositories
    let license_repo = LicenseRepository::new(database_pool.clone());
    let quota_repo = QuotaRepository::new(database_pool.clone());
    let billing_repo = BillingRepository::new(database_pool.clone());
    let compliance_repo = ComplianceRepository::new(database_pool.clone());

    // Initialize billing service
    let billing_service = BillingService::new(
        Some(config.stripe.clone()),
        Some(config.paypal.clone()),
        config.billing.clone(),
    );

    // Initialize license service
    let license_service = LicenseService::new(
        license_repo,
        quota_repo,
        billing_repo,
        compliance_repo,
        billing_service,
    );

    info!("License service worker initialized");

    // TODO: Initialize Temporal worker
    // This would typically involve:
    // 1. Creating a Temporal client
    // 2. Registering workflows and activities
    // 3. Starting the worker
    
    info!("Temporal worker configuration:");
    info!("  Server URL: {}", config.temporal.server_url);
    info!("  Namespace: {}", config.temporal.namespace);
    info!("  Task Queue: {}", config.temporal.task_queue);

    // For now, just run indefinitely
    // In a real implementation, this would start the Temporal worker
    tokio::select! {
        _ = shutdown_signal() => {
            info!("Shutdown signal received, stopping worker");
        }
    }

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("Signal received, starting graceful shutdown");
}