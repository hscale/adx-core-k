use serde::{Deserialize, Serialize};
use std::time::Duration;
use anyhow::{Result, Context};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiGatewayConfig {
    pub server: ServerConfig,
    pub temporal: TemporalConfig,
    pub services: ServicesConfig,
    pub auth: AuthConfig,
    pub rate_limiting: RateLimitingConfig,
    pub redis: RedisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub request_timeout_seconds: u64,
    pub max_request_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub server_address: String,
    pub namespace: String,
    pub client_identity: String,
    pub connection_timeout_seconds: u64,
    pub request_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServicesConfig {
    pub auth_service: ServiceEndpoint,
    pub user_service: ServiceEndpoint,
    pub tenant_service: ServiceEndpoint,
    pub file_service: ServiceEndpoint,
    pub workflow_service: ServiceEndpoint,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoint {
    pub base_url: String,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub require_auth: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    pub enabled: bool,
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: u32,
    pub connection_timeout_seconds: u64,
}

impl ApiGatewayConfig {
    pub fn from_env() -> Result<Self> {
        let config = config::Config::builder()
            .add_source(config::Environment::with_prefix("API_GATEWAY"))
            .build()
            .context("Failed to build configuration")?;

        let mut gateway_config: ApiGatewayConfig = config
            .try_deserialize()
            .context("Failed to deserialize configuration")?;

        // Apply defaults if not set
        gateway_config.apply_defaults();

        Ok(gateway_config)
    }

    pub fn development() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8080,
                request_timeout_seconds: 30,
                max_request_size: 16 * 1024 * 1024, // 16MB
            },
            temporal: TemporalConfig {
                server_address: "localhost:7233".to_string(),
                namespace: "adx-core-development".to_string(),
                client_identity: "api-gateway".to_string(),
                connection_timeout_seconds: 10,
                request_timeout_seconds: 30,
            },
            services: ServicesConfig {
                auth_service: ServiceEndpoint {
                    base_url: "http://localhost:8081".to_string(),
                    timeout_seconds: 10,
                },
                user_service: ServiceEndpoint {
                    base_url: "http://localhost:8082".to_string(),
                    timeout_seconds: 10,
                },
                tenant_service: ServiceEndpoint {
                    base_url: "http://localhost:8085".to_string(),
                    timeout_seconds: 10,
                },
                file_service: ServiceEndpoint {
                    base_url: "http://localhost:8083".to_string(),
                    timeout_seconds: 30, // Longer timeout for file operations
                },
                workflow_service: ServiceEndpoint {
                    base_url: "http://localhost:8084".to_string(),
                    timeout_seconds: 60, // Longer timeout for workflow operations
                },
            },
            auth: AuthConfig {
                jwt_secret: "development-secret-key-change-in-production".to_string(),
                jwt_expiration_hours: 24,
                require_auth: true,
            },
            rate_limiting: RateLimitingConfig {
                enabled: true,
                requests_per_minute: 100,
                requests_per_hour: 1000,
                burst_limit: 20,
            },
            redis: RedisConfig {
                url: "redis://localhost:6379".to_string(),
                pool_size: 10,
                connection_timeout_seconds: 5,
            },
        }
    }

    fn apply_defaults(&mut self) {
        // Apply default values for missing configuration
        if self.server.host.is_empty() {
            self.server.host = "0.0.0.0".to_string();
        }
        if self.server.port == 0 {
            self.server.port = 8080;
        }
        if self.temporal.server_address.is_empty() {
            self.temporal.server_address = "localhost:7233".to_string();
        }
        if self.temporal.namespace.is_empty() {
            self.temporal.namespace = "adx-core-development".to_string();
        }
        if self.auth.jwt_secret.is_empty() {
            self.auth.jwt_secret = "development-secret-key-change-in-production".to_string();
        }
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_secs(self.server.request_timeout_seconds)
    }

    pub fn temporal_connection_timeout(&self) -> Duration {
        Duration::from_secs(self.temporal.connection_timeout_seconds)
    }

    pub fn temporal_request_timeout(&self) -> Duration {
        Duration::from_secs(self.temporal.request_timeout_seconds)
    }

    pub fn service_timeout(&self, service: &str) -> Duration {
        let timeout_seconds = match service {
            "auth" => self.services.auth_service.timeout_seconds,
            "user" => self.services.user_service.timeout_seconds,
            "tenant" => self.services.tenant_service.timeout_seconds,
            "file" => self.services.file_service.timeout_seconds,
            "workflow" => self.services.workflow_service.timeout_seconds,
            _ => 10, // Default timeout
        };
        Duration::from_secs(timeout_seconds)
    }
}

impl Default for ApiGatewayConfig {
    fn default() -> Self {
        Self::development()
    }
}