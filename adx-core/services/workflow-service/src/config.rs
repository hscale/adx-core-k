use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowServiceConfig {
    pub server: ServerConfig,
    pub temporal: TemporalConfig,
    pub services: ServiceEndpoints,
    pub workflows: WorkflowConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub server_url: String,
    pub namespace: String,
    pub task_queue: String,
    pub worker_identity: String,
    pub max_concurrent_activities: usize,
    pub max_concurrent_workflows: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEndpoints {
    pub auth_service: String,
    pub user_service: String,
    pub tenant_service: String,
    pub file_service: String,
    pub api_gateway: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub default_timeout: Duration,
    pub retry_policy: RetryPolicyConfig,
    pub batch_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicyConfig {
    pub initial_interval: Duration,
    pub backoff_coefficient: f64,
    pub maximum_interval: Duration,
    pub maximum_attempts: u32,
}

impl Default for WorkflowServiceConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig {
                host: "0.0.0.0".to_string(),
                port: 8084,
                timeout_seconds: 30,
            },
            temporal: TemporalConfig {
                server_url: "http://localhost:7233".to_string(),
                namespace: "default".to_string(),
                task_queue: "workflow-service-queue".to_string(),
                worker_identity: "workflow-service-worker".to_string(),
                max_concurrent_activities: 100,
                max_concurrent_workflows: 50,
            },
            services: ServiceEndpoints {
                auth_service: "http://localhost:8081".to_string(),
                user_service: "http://localhost:8082".to_string(),
                tenant_service: "http://localhost:8085".to_string(),
                file_service: "http://localhost:8083".to_string(),
                api_gateway: "http://localhost:8080".to_string(),
            },
            workflows: WorkflowConfig {
                default_timeout: Duration::from_secs(300), // 5 minutes
                retry_policy: RetryPolicyConfig {
                    initial_interval: Duration::from_secs(1),
                    backoff_coefficient: 2.0,
                    maximum_interval: Duration::from_secs(60),
                    maximum_attempts: 3,
                },
                batch_size: 100,
            },
        }
    }
}