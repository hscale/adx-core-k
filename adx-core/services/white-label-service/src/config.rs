use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelConfig {
    pub database_url: String,
    pub temporal_server_url: String,
    pub server_port: u16,
    pub domain_config: DomainConfig,
    pub ssl_config: SslConfig,
    pub asset_config: AssetConfig,
    pub dns_providers: HashMap<String, DnsProviderConfig>,
    pub email_config: EmailConfig,
    pub storage_config: StorageConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub verification_timeout_seconds: u64,
    pub max_domains_per_tenant: u32,
    pub allowed_tlds: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub dns_propagation_wait_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslConfig {
    pub provider: String, // "letsencrypt", "aws_acm", "cloudflare"
    pub auto_renewal: bool,
    pub renewal_days_before_expiry: u32,
    pub certificate_authority: String,
    pub key_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    pub max_file_size_mb: u32,
    pub allowed_mime_types: Vec<String>,
    pub image_optimization: ImageOptimizationConfig,
    pub storage_path: String,
    pub cdn_base_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageOptimizationConfig {
    pub enabled: bool,
    pub max_width: u32,
    pub max_height: u32,
    pub quality: u8,
    pub formats: Vec<String>, // "webp", "png", "jpg"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsProviderConfig {
    pub provider_type: String, // "cloudflare", "route53", "godaddy"
    pub api_key: String,
    pub api_secret: Option<String>,
    pub zone_id: Option<String>,
    pub base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub template_cache_ttl_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub provider: String, // "local", "s3", "gcs", "azure"
    pub bucket_name: Option<String>,
    pub region: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub endpoint: Option<String>,
}

impl Default for WhiteLabelConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/adx_core".to_string(),
            temporal_server_url: "http://localhost:7233".to_string(),
            server_port: 8087,
            domain_config: DomainConfig {
                verification_timeout_seconds: 300,
                max_domains_per_tenant: 5,
                allowed_tlds: vec![
                    "com".to_string(),
                    "org".to_string(),
                    "net".to_string(),
                    "io".to_string(),
                    "co".to_string(),
                ],
                blocked_domains: vec![
                    "localhost".to_string(),
                    "127.0.0.1".to_string(),
                    "0.0.0.0".to_string(),
                ],
                dns_propagation_wait_seconds: 60,
            },
            ssl_config: SslConfig {
                provider: "letsencrypt".to_string(),
                auto_renewal: true,
                renewal_days_before_expiry: 30,
                certificate_authority: "https://acme-v02.api.letsencrypt.org/directory".to_string(),
                key_size: 2048,
            },
            asset_config: AssetConfig {
                max_file_size_mb: 10,
                allowed_mime_types: vec![
                    "image/png".to_string(),
                    "image/jpeg".to_string(),
                    "image/gif".to_string(),
                    "image/webp".to_string(),
                    "image/svg+xml".to_string(),
                ],
                image_optimization: ImageOptimizationConfig {
                    enabled: true,
                    max_width: 2048,
                    max_height: 2048,
                    quality: 85,
                    formats: vec!["webp".to_string(), "png".to_string()],
                },
                storage_path: "./storage/assets".to_string(),
                cdn_base_url: None,
            },
            dns_providers: HashMap::new(),
            email_config: EmailConfig {
                smtp_host: "localhost".to_string(),
                smtp_port: 587,
                smtp_username: "".to_string(),
                smtp_password: "".to_string(),
                from_email: "noreply@adxcore.com".to_string(),
                from_name: "ADX Core".to_string(),
                template_cache_ttl_seconds: 3600,
            },
            storage_config: StorageConfig {
                provider: "local".to_string(),
                bucket_name: None,
                region: None,
                access_key: None,
                secret_key: None,
                endpoint: None,
            },
        }
    }
}

impl WhiteLabelConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder()
            .add_source(config::Environment::with_prefix("WHITE_LABEL"))
            .build()?;

        // Set defaults
        let default_config = Self::default();
        cfg.set_default("database_url", default_config.database_url)?;
        cfg.set_default("temporal_server_url", default_config.temporal_server_url)?;
        cfg.set_default("server_port", default_config.server_port)?;

        cfg.try_deserialize()
    }
}