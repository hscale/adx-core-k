use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub temporal: TemporalConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub observability: ObservabilityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: Option<usize>,
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
    pub max_lifetime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub command_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub server_url: String,
    pub namespace: String,
    pub task_queue: String,
    pub worker_max_concurrent_activities: usize,
    pub worker_max_concurrent_workflows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: Option<String>,
    pub jwt_expiration_hours: u64,
    pub refresh_token_expiration_days: u64,
    pub bcrypt_cost: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String, // json or pretty
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObservabilityConfig {
    pub tracing_enabled: bool,
    pub metrics_enabled: bool,
    pub jaeger_endpoint: Option<String>,
    pub prometheus_endpoint: Option<String>,
}

impl AppConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
        
        let config = Config::builder()
            // Start with default configuration
            .add_source(File::with_name("config/default").required(false))
            // Add environment-specific configuration
            .add_source(File::with_name(&format!("config/{}", environment)).required(false))
            // Add local configuration (for development overrides)
            .add_source(File::with_name("config/local").required(false))
            // Add environment variables with ADX_ prefix
            .add_source(Environment::with_prefix("ADX").separator("_"))
            .build()?;
        
        config.try_deserialize()
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                workers: None,
                max_connections: Some(1000),
                timeout_seconds: Some(30),
            },
            database: DatabaseConfig {
                url: "postgresql://postgres:postgres@localhost:5432/adx_core".to_string(),
                max_connections: 20,
                min_connections: 5,
                acquire_timeout_seconds: 30,
                idle_timeout_seconds: 600,
                max_lifetime_seconds: 1800,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                max_connections: 20,
                connection_timeout_seconds: 5,
                command_timeout_seconds: 5,
            },
            temporal: TemporalConfig {
                server_url: "http://localhost:7233".to_string(),
                namespace: "default".to_string(),
                task_queue: "adx-core-task-queue".to_string(),
                worker_max_concurrent_activities: 100,
                worker_max_concurrent_workflows: 50,
            },
            auth: AuthConfig {
                jwt_secret: Some("your-secret-key-change-in-production".to_string()),
                jwt_expiration_hours: 24,
                refresh_token_expiration_days: 30,
                bcrypt_cost: 12,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "json".to_string(),
                file_path: None,
            },
            observability: ObservabilityConfig {
                tracing_enabled: true,
                metrics_enabled: true,
                jaeger_endpoint: Some("http://localhost:14268/api/traces".to_string()),
                prometheus_endpoint: Some("http://localhost:9090".to_string()),
            },
        }
    }
}