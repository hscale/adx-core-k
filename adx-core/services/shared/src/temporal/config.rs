use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::temporal::TemporalError;

/// Temporal configuration for ADX Core services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    /// Temporal server address (e.g., "localhost:7233")
    pub server_address: String,
    
    /// Namespace for this environment
    pub namespace: String,
    
    /// Client identity for this service
    pub client_identity: String,
    
    /// Connection configuration
    pub connection: ConnectionConfig,
    
    /// Retry configuration
    pub retry: RetryConfig,
    
    /// Worker configuration
    pub worker: WorkerConfig,
    
    /// Workflow configuration
    pub workflow: WorkflowConfig,
    
    /// Activity configuration
    pub activity: ActivityConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Connection timeout
    pub connect_timeout: Duration,
    
    /// Keep-alive timeout
    pub keep_alive_timeout: Duration,
    
    /// Keep-alive interval
    pub keep_alive_interval: Duration,
    
    /// Maximum number of concurrent connections
    pub max_concurrent_connections: usize,
    
    /// Enable TLS
    pub enable_tls: bool,
    
    /// TLS configuration
    pub tls: Option<TlsConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsConfig {
    /// Server name for TLS verification
    pub server_name: String,
    
    /// Client certificate path
    pub client_cert_path: Option<String>,
    
    /// Client key path
    pub client_key_path: Option<String>,
    
    /// CA certificate path
    pub ca_cert_path: Option<String>,
    
    /// Skip certificate verification (development only)
    pub insecure_skip_verify: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Initial retry interval
    pub initial_interval: Duration,
    
    /// Maximum retry interval
    pub max_interval: Duration,
    
    /// Backoff coefficient
    pub backoff_coefficient: f64,
    
    /// Maximum retry attempts
    pub max_attempts: u32,
    
    /// Non-retryable error types
    pub non_retryable_errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerConfig {
    /// Maximum number of concurrent workflow tasks
    pub max_concurrent_workflow_tasks: usize,
    
    /// Maximum number of concurrent activity tasks
    pub max_concurrent_activity_tasks: usize,
    
    /// Worker identity
    pub identity: String,
    
    /// Task queue names this worker will poll
    pub task_queues: Vec<String>,
    
    /// Enable sticky execution
    pub enable_sticky_execution: bool,
    
    /// Sticky schedule to start timeout
    pub sticky_schedule_to_start_timeout: Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Default workflow execution timeout
    pub default_execution_timeout: Duration,
    
    /// Default workflow run timeout
    pub default_run_timeout: Duration,
    
    /// Default workflow task timeout
    pub default_task_timeout: Duration,
    
    /// Enable workflow versioning
    pub enable_versioning: bool,
    
    /// Default workflow retry policy
    pub default_retry_policy: RetryConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityConfig {
    /// Default activity start to close timeout
    pub default_start_to_close_timeout: Duration,
    
    /// Default activity schedule to start timeout
    pub default_schedule_to_start_timeout: Duration,
    
    /// Default activity schedule to close timeout
    pub default_schedule_to_close_timeout: Duration,
    
    /// Default activity heartbeat timeout
    pub default_heartbeat_timeout: Duration,
    
    /// Default activity retry policy
    pub default_retry_policy: RetryConfig,
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            server_address: "localhost:7233".to_string(),
            namespace: "adx-core-development".to_string(),
            client_identity: "adx-core-client".to_string(),
            connection: ConnectionConfig::default(),
            retry: RetryConfig::default(),
            worker: WorkerConfig::default(),
            workflow: WorkflowConfig::default(),
            activity: ActivityConfig::default(),
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            keep_alive_timeout: Duration::from_secs(30),
            keep_alive_interval: Duration::from_secs(30),
            max_concurrent_connections: 100,
            enable_tls: false,
            tls: None,
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            initial_interval: Duration::from_secs(1),
            max_interval: Duration::from_secs(60),
            backoff_coefficient: 2.0,
            max_attempts: 3,
            non_retryable_errors: vec![
                "ValidationError".to_string(),
                "AuthorizationError".to_string(),
                "TenantNotFoundError".to_string(),
            ],
        }
    }
}

impl Default for WorkerConfig {
    fn default() -> Self {
        Self {
            max_concurrent_workflow_tasks: 100,
            max_concurrent_activity_tasks: 200,
            identity: format!("adx-core-worker-{}", uuid::Uuid::new_v4()),
            task_queues: vec!["adx-core-default".to_string()],
            enable_sticky_execution: true,
            sticky_schedule_to_start_timeout: Duration::from_secs(5),
        }
    }
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            default_execution_timeout: Duration::from_secs(3600), // 1 hour
            default_run_timeout: Duration::from_secs(1800), // 30 minutes
            default_task_timeout: Duration::from_secs(10),
            enable_versioning: true,
            default_retry_policy: RetryConfig::default(),
        }
    }
}

impl Default for ActivityConfig {
    fn default() -> Self {
        Self {
            default_start_to_close_timeout: Duration::from_secs(300), // 5 minutes
            default_schedule_to_start_timeout: Duration::from_secs(60), // 1 minute
            default_schedule_to_close_timeout: Duration::from_secs(360), // 6 minutes
            default_heartbeat_timeout: Duration::from_secs(30),
            default_retry_policy: RetryConfig::default(),
        }
    }
}

/// Environment-specific configuration factory
impl TemporalConfig {
    /// Create configuration for development environment
    pub fn development() -> Self {
        Self {
            namespace: "adx-core-development".to_string(),
            server_address: "localhost:7233".to_string(),
            client_identity: "adx-core-dev-client".to_string(),
            connection: ConnectionConfig {
                connect_timeout: Duration::from_secs(5),
                ..Default::default()
            },
            workflow: WorkflowConfig {
                default_execution_timeout: Duration::from_secs(1800), // 30 minutes for dev
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// Create configuration for staging environment
    pub fn staging() -> Self {
        Self {
            namespace: "adx-core-staging".to_string(),
            server_address: std::env::var("TEMPORAL_SERVER_ADDRESS")
                .unwrap_or_else(|_| "temporal-staging:7233".to_string()),
            client_identity: "adx-core-staging-client".to_string(),
            connection: ConnectionConfig {
                enable_tls: true,
                tls: Some(TlsConfig {
                    server_name: "temporal-staging".to_string(),
                    client_cert_path: Some("/etc/temporal/certs/client.crt".to_string()),
                    client_key_path: Some("/etc/temporal/certs/client.key".to_string()),
                    ca_cert_path: Some("/etc/temporal/certs/ca.crt".to_string()),
                    insecure_skip_verify: false,
                }),
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// Create configuration for production environment
    pub fn production() -> Self {
        Self {
            namespace: "adx-core-production".to_string(),
            server_address: std::env::var("TEMPORAL_SERVER_ADDRESS")
                .unwrap_or_else(|_| "temporal-prod:7233".to_string()),
            client_identity: "adx-core-prod-client".to_string(),
            connection: ConnectionConfig {
                enable_tls: true,
                max_concurrent_connections: 500,
                tls: Some(TlsConfig {
                    server_name: "temporal-prod".to_string(),
                    client_cert_path: Some("/etc/temporal/certs/client.crt".to_string()),
                    client_key_path: Some("/etc/temporal/certs/client.key".to_string()),
                    ca_cert_path: Some("/etc/temporal/certs/ca.crt".to_string()),
                    insecure_skip_verify: false,
                }),
                ..Default::default()
            },
            worker: WorkerConfig {
                max_concurrent_workflow_tasks: 500,
                max_concurrent_activity_tasks: 1000,
                ..Default::default()
            },
            workflow: WorkflowConfig {
                default_execution_timeout: Duration::from_secs(86400), // 24 hours for prod
                default_run_timeout: Duration::from_secs(7200), // 2 hours for prod
                ..Default::default()
            },
            ..Default::default()
        }
    }
    
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, TemporalError> {
        let environment = std::env::var("ADX_ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());
        
        match environment.as_str() {
            "development" => Ok(Self::development()),
            "staging" => Ok(Self::staging()),
            "production" => Ok(Self::production()),
            _ => Err(TemporalError::ConfigurationError {
                message: format!("Invalid environment: {}", environment),
            }),
        }
    }
}