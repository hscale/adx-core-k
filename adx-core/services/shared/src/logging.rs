use tracing::{info, warn, error};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};
use tracing_subscriber::fmt::Layer as FmtLayer;
use crate::{config::LoggingConfig, Result, Error};

pub fn init_logging(config: &LoggingConfig) -> Result<()> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(&config.level));

    let fmt_layer = match config.format.as_str() {
        "json" => FmtLayer::new()
            .json()
            .with_current_span(true)
            .with_span_list(true),
        "pretty" => FmtLayer::new()
            .pretty()
            .with_current_span(true)
            .with_span_list(true),
        _ => FmtLayer::new()
            .with_current_span(true)
            .with_span_list(true),
    };

    let registry = Registry::default()
        .with(env_filter)
        .with(fmt_layer);

    // Add file output if configured
    if let Some(file_path) = &config.file_path {
        let file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file_path)
            .map_err(|e| Error::Internal(format!("Failed to open log file: {}", e)))?;
        
        let file_layer = FmtLayer::new()
            .json()
            .with_writer(file)
            .with_current_span(true)
            .with_span_list(true);
        
        registry.with(file_layer).init();
    } else {
        registry.init();
    }

    info!("Logging initialized with level: {}", config.level);
    Ok(())
}

// Structured logging macros for common patterns
#[macro_export]
macro_rules! log_request {
    ($method:expr, $path:expr, $status:expr, $duration:expr) => {
        tracing::info!(
            method = %$method,
            path = %$path,
            status = %$status,
            duration_ms = %$duration,
            "HTTP request completed"
        );
    };
}

#[macro_export]
macro_rules! log_workflow_event {
    ($workflow_id:expr, $workflow_type:expr, $event:expr) => {
        tracing::info!(
            workflow_id = %$workflow_id,
            workflow_type = %$workflow_type,
            event = %$event,
            "Workflow event"
        );
    };
}

#[macro_export]
macro_rules! log_tenant_operation {
    ($tenant_id:expr, $operation:expr, $user_id:expr) => {
        tracing::info!(
            tenant_id = %$tenant_id,
            operation = %$operation,
            user_id = %$user_id,
            "Tenant operation"
        );
    };
}

#[macro_export]
macro_rules! log_error {
    ($error:expr, $context:expr) => {
        tracing::error!(
            error = %$error,
            context = %$context,
            "Error occurred"
        );
    };
}

// Correlation ID utilities for request tracing
use std::sync::Arc;
use tokio::task_local;

task_local! {
    pub static CORRELATION_ID: Arc<String>;
}

pub fn generate_correlation_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub async fn with_correlation_id<F, R>(correlation_id: String, f: F) -> R
where
    F: std::future::Future<Output = R>,
{
    CORRELATION_ID.scope(Arc::new(correlation_id), f).await
}

pub fn get_correlation_id() -> Option<String> {
    CORRELATION_ID.try_with(|id| id.as_ref().clone()).ok()
}