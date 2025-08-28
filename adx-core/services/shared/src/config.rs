// Configuration management for ADX Core services

use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database_url: String,
    pub redis_url: String,
    pub temporal_server_url: String,
    pub jwt_secret: String,
    pub service_port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder();
        
        // Set defaults
        cfg = cfg
            .set_default("database_url", "postgres://postgres:postgres@localhost:5432/adx_core")?
            .set_default("redis_url", "redis://localhost:6379")?
            .set_default("temporal_server_url", "localhost:7233")?
            .set_default("jwt_secret", "development-secret-key")?
            .set_default("service_port", 8080)?
            .set_default("log_level", "info")?;
        
        // Override with environment variables
        cfg = cfg.add_source(config::Environment::with_prefix("ADX"));
        
        // Override with test values in test mode
        if env::var("TEST_MODE").is_ok() {
            cfg = cfg
                .set_override("database_url", "postgres://postgres:postgres@localhost:5432/adx_core_test")?
                .set_override("log_level", "debug")?;
        }
        
        cfg.build()?.try_deserialize()
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            database_url: "postgres://postgres:postgres@localhost:5432/adx_core".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            temporal_server_url: "localhost:7233".to_string(),
            jwt_secret: "development-secret-key".to_string(),
            service_port: 8080,
            log_level: "info".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.service_port, 8080);
        assert_eq!(config.log_level, "info");
        assert!(config.database_url.contains("adx_core"));
    }

    #[test]
    fn test_config_from_env() {
        // Set test environment variable
        env::set_var("ADX_SERVICE_PORT", "9999");
        env::set_var("TEST_MODE", "true");
        
        let config = Config::from_env().unwrap();
        assert_eq!(config.service_port, 9999);
        assert!(config.database_url.contains("adx_core_test"));
        
        // Clean up
        env::remove_var("ADX_SERVICE_PORT");
        env::remove_var("TEST_MODE");
    }
}