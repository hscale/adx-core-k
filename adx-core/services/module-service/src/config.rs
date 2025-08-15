use serde::{Deserialize, Serialize};
use std::env;
use crate::error::ModuleServiceError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleServiceConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub temporal: TemporalConfig,
    pub redis: RedisConfig,
    pub marketplace: MarketplaceConfig,
    pub sandbox: SandboxConfig,
    pub security: SecurityConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub cors_origins: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub server_url: String,
    pub namespace: String,
    pub task_queue: String,
    pub worker_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub max_connections: u32,
    pub cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    pub enabled: bool,
    pub api_url: String,
    pub api_key: String,
    pub webhook_secret: String,
    pub payment_provider: PaymentProvider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub enabled: bool,
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub max_storage_mb: u64,
    pub network_isolation: bool,
    pub allowed_domains: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub scan_enabled: bool,
    pub scan_timeout_seconds: u64,
    pub allowed_file_types: Vec<String>,
    pub max_package_size_mb: u64,
    pub signature_verification: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub provider: StorageProvider,
    pub bucket_name: String,
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub local_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentProvider {
    Stripe,
    PayPal,
    Mock,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StorageProvider {
    S3,
    GCS,
    Azure,
    Local,
}

impl ModuleServiceConfig {
    pub fn from_env() -> Result<Self, ModuleServiceError> {
        Ok(Self {
            server: ServerConfig {
                host: env::var("MODULE_SERVICE_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("MODULE_SERVICE_PORT")
                    .unwrap_or_else(|_| "8086".to_string())
                    .parse()
                    .map_err(|_| ModuleServiceError::ConfigError("Invalid port".to_string()))?,
                cors_origins: env::var("CORS_ORIGINS")
                    .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3006".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .map_err(|_| ModuleServiceError::ConfigError("DATABASE_URL required".to_string()))?,
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                min_connections: env::var("DB_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()
                    .unwrap_or(1),
            },
            temporal: TemporalConfig {
                server_url: env::var("TEMPORAL_SERVER_URL")
                    .unwrap_or_else(|_| "localhost:7233".to_string()),
                namespace: env::var("TEMPORAL_NAMESPACE")
                    .unwrap_or_else(|_| "default".to_string()),
                task_queue: env::var("MODULE_TASK_QUEUE")
                    .unwrap_or_else(|_| "module-task-queue".to_string()),
                worker_identity: env::var("MODULE_WORKER_IDENTITY")
                    .unwrap_or_else(|_| "module-worker".to_string()),
            },
            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
                max_connections: env::var("REDIS_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10),
                cache_ttl_seconds: env::var("CACHE_TTL_SECONDS")
                    .unwrap_or_else(|_| "3600".to_string())
                    .parse()
                    .unwrap_or(3600),
            },
            marketplace: MarketplaceConfig {
                enabled: env::var("MARKETPLACE_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                api_url: env::var("MARKETPLACE_API_URL")
                    .unwrap_or_else(|_| "https://marketplace.adxcore.com".to_string()),
                api_key: env::var("MARKETPLACE_API_KEY")
                    .unwrap_or_else(|_| "dev-key".to_string()),
                webhook_secret: env::var("MARKETPLACE_WEBHOOK_SECRET")
                    .unwrap_or_else(|_| "dev-secret".to_string()),
                payment_provider: match env::var("PAYMENT_PROVIDER").as_deref() {
                    Ok("stripe") => PaymentProvider::Stripe,
                    Ok("paypal") => PaymentProvider::PayPal,
                    _ => PaymentProvider::Mock,
                },
            },
            sandbox: SandboxConfig {
                enabled: env::var("SANDBOX_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                max_memory_mb: env::var("SANDBOX_MAX_MEMORY_MB")
                    .unwrap_or_else(|_| "512".to_string())
                    .parse()
                    .unwrap_or(512),
                max_cpu_percent: env::var("SANDBOX_MAX_CPU_PERCENT")
                    .unwrap_or_else(|_| "50.0".to_string())
                    .parse()
                    .unwrap_or(50.0),
                max_storage_mb: env::var("SANDBOX_MAX_STORAGE_MB")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()
                    .unwrap_or(100),
                network_isolation: env::var("SANDBOX_NETWORK_ISOLATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                allowed_domains: env::var("SANDBOX_ALLOWED_DOMAINS")
                    .unwrap_or_else(|_| "api.adxcore.com,marketplace.adxcore.com".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            security: SecurityConfig {
                scan_enabled: env::var("SECURITY_SCAN_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
                scan_timeout_seconds: env::var("SECURITY_SCAN_TIMEOUT")
                    .unwrap_or_else(|_| "300".to_string())
                    .parse()
                    .unwrap_or(300),
                allowed_file_types: env::var("ALLOWED_FILE_TYPES")
                    .unwrap_or_else(|_| "js,ts,json,md,txt,yml,yaml".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
                max_package_size_mb: env::var("MAX_PACKAGE_SIZE_MB")
                    .unwrap_or_else(|_| "50".to_string())
                    .parse()
                    .unwrap_or(50),
                signature_verification: env::var("SIGNATURE_VERIFICATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()
                    .unwrap_or(true),
            },
            storage: StorageConfig {
                provider: match env::var("STORAGE_PROVIDER").as_deref() {
                    Ok("s3") => StorageProvider::S3,
                    Ok("gcs") => StorageProvider::GCS,
                    Ok("azure") => StorageProvider::Azure,
                    _ => StorageProvider::Local,
                },
                bucket_name: env::var("STORAGE_BUCKET")
                    .unwrap_or_else(|_| "adx-modules".to_string()),
                region: env::var("STORAGE_REGION")
                    .unwrap_or_else(|_| "us-east-1".to_string()),
                access_key: env::var("STORAGE_ACCESS_KEY")
                    .unwrap_or_else(|_| "dev-access-key".to_string()),
                secret_key: env::var("STORAGE_SECRET_KEY")
                    .unwrap_or_else(|_| "dev-secret-key".to_string()),
                local_path: env::var("STORAGE_LOCAL_PATH").ok(),
            },
        })
    }
}