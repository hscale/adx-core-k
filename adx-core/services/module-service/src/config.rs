use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleServiceConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub temporal: TemporalConfig,
    pub marketplace: MarketplaceConfig,
    pub sandbox: SandboxConfig,
    pub security: SecurityConfig,
    pub monitoring: MonitoringConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub workers: Option<usize>,
    pub max_connections: u32,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout_seconds: u64,
    pub idle_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub server_url: String,
    pub namespace: String,
    pub task_queue: String,
    pub worker_identity: String,
    pub max_concurrent_activities: u32,
    pub max_concurrent_workflows: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub base_url: String,
    pub api_key: String,
    pub timeout_seconds: u64,
    pub cache_ttl_seconds: u64,
    pub enable_analytics: bool,
    pub enable_recommendations: bool,
    pub payment_providers: Vec<PaymentProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentProviderConfig {
    pub provider_type: String,
    pub config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub default_isolation_level: String,
    pub max_sandboxes: u32,
    pub sandbox_timeout_seconds: u64,
    pub enable_wasm: bool,
    pub enable_containers: bool,
    pub enable_process_isolation: bool,
    pub resource_check_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub enable_security_scanning: bool,
    pub scan_timeout_seconds: u64,
    pub min_security_score: u8,
    pub allowed_permissions: Vec<String>,
    pub blocked_permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enable_metrics: bool,
    pub metrics_port: u16,
    pub health_check_interval_seconds: u64,
    pub resource_check_interval_seconds: u64,
    pub log_level: String,
}

impl Default for ModuleServiceConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8086,
                workers: None,
                max_connections: 1000,
                timeout_seconds: 30,
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/adx_core".to_string(),
                max_connections: 20,
                min_connections: 5,
                connection_timeout_seconds: 30,
                idle_timeout_seconds: 600,
            },
            temporal: TemporalConfig {
                server_url: "localhost:7233".to_string(),
                namespace: "adx-core".to_string(),
                task_queue: "module-service".to_string(),
                worker_identity: "module-service-worker".to_string(),
                max_concurrent_activities: 100,
                max_concurrent_workflows: 50,
            },
            marketplace: MarketplaceConfig {
                base_url: "https://marketplace.adxcore.com".to_string(),
                api_key: "".to_string(),
                timeout_seconds: 30,
                cache_ttl_seconds: 300,
                enable_analytics: true,
                enable_recommendations: true,
                payment_providers: vec![],
            },
            sandbox: SandboxConfig {
                default_isolation_level: "process".to_string(),
                max_sandboxes: 100,
                sandbox_timeout_seconds: 300,
                enable_wasm: true,
                enable_containers: true,
                enable_process_isolation: true,
                resource_check_interval_seconds: 5,
            },
            security: SecurityConfig {
                enable_security_scanning: true,
                scan_timeout_seconds: 120,
                min_security_score: 70,
                allowed_permissions: vec![],
                blocked_permissions: vec![],
            },
            monitoring: MonitoringConfig {
                enable_metrics: true,
                metrics_port: 9090,
                health_check_interval_seconds: 30,
                resource_check_interval_seconds: 10,
                log_level: "info".to_string(),
            },
        }
    }
}