use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub temporal_server_url: String,
    pub ai_providers: AIProvidersConfig,
    pub monitoring: MonitoringConfig,
    pub security: SecurityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIProvidersConfig {
    pub openai: OpenAIConfig,
    pub anthropic: AnthropicConfig,
    pub local: LocalAIConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub default_model: String,
    pub max_tokens: u32,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnthropicConfig {
    pub api_key: String,
    pub base_url: Option<String>,
    pub default_model: String,
    pub max_tokens: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalAIConfig {
    pub enabled: bool,
    pub base_url: String,
    pub models: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub prometheus_port: u16,
    pub usage_tracking_enabled: bool,
    pub cost_tracking_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub rate_limit_per_minute: u32,
    pub max_request_size: usize,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();

        // Load from environment variables
        cfg = cfg
            .set_default("database_url", "postgresql://postgres:postgres@localhost:5432/adx_core")?
            .set_default("redis_url", "redis://localhost:6379")?
            .set_default("temporal_server_url", "http://localhost:7233")?
            
            // AI Providers
            .set_default("ai_providers.openai.api_key", "")?
            .set_default("ai_providers.openai.default_model", "gpt-3.5-turbo")?
            .set_default("ai_providers.openai.max_tokens", 4096)?
            .set_default("ai_providers.openai.temperature", 0.7)?
            
            .set_default("ai_providers.anthropic.api_key", "")?
            .set_default("ai_providers.anthropic.default_model", "claude-3-sonnet-20240229")?
            .set_default("ai_providers.anthropic.max_tokens", 4096)?
            
            .set_default("ai_providers.local.enabled", false)?
            .set_default("ai_providers.local.base_url", "http://localhost:11434")?
            
            // Monitoring
            .set_default("monitoring.metrics_enabled", true)?
            .set_default("monitoring.prometheus_port", 9090)?
            .set_default("monitoring.usage_tracking_enabled", true)?
            .set_default("monitoring.cost_tracking_enabled", true)?
            
            // Security
            .set_default("security.jwt_secret", "your-secret-key")?
            .set_default("security.rate_limit_per_minute", 60)?
            .set_default("security.max_request_size", 1048576)?; // 1MB

        // Override with environment variables
        cfg = cfg.add_source(config::Environment::with_prefix("AI_SERVICE"));

        // Override specific environment variables
        if let Ok(db_url) = env::var("DATABASE_URL") {
            cfg = cfg.set_override("database_url", db_url)?;
        }
        
        if let Ok(redis_url) = env::var("REDIS_URL") {
            cfg = cfg.set_override("redis_url", redis_url)?;
        }
        
        if let Ok(temporal_url) = env::var("TEMPORAL_SERVER_URL") {
            cfg = cfg.set_override("temporal_server_url", temporal_url)?;
        }
        
        if let Ok(openai_key) = env::var("OPENAI_API_KEY") {
            cfg = cfg.set_override("ai_providers.openai.api_key", openai_key)?;
        }
        
        if let Ok(anthropic_key) = env::var("ANTHROPIC_API_KEY") {
            cfg = cfg.set_override("ai_providers.anthropic.api_key", anthropic_key)?;
        }

        cfg.build()?.try_deserialize()
    }
}