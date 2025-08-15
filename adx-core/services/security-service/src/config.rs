use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub temporal: TemporalConfig,
    pub audit: AuditConfig,
    pub compliance: ComplianceConfig,
    pub encryption: EncryptionConfig,
    pub scanning: ScanningConfig,
    pub zero_trust: ZeroTrustConfig,
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
    pub acquire_timeout: u64,
    pub idle_timeout: u64,
    pub max_lifetime: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub server_url: String,
    pub namespace: String,
    pub task_queue: String,
    pub client_name: String,
    pub client_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditConfig {
    pub enabled: bool,
    pub log_level: String,
    pub retention_days: u32,
    pub batch_size: u32,
    pub flush_interval_seconds: u32,
    pub storage_backend: String,
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    pub gdpr_enabled: bool,
    pub data_retention_days: u32,
    pub automatic_deletion: bool,
    pub export_format: String,
    pub notification_email: String,
    pub compliance_officer_email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionConfig {
    pub algorithm: String,
    pub key_rotation_days: u32,
    pub master_key_id: String,
    pub kms_provider: String,
    pub at_rest_enabled: bool,
    pub in_transit_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanningConfig {
    pub enabled: bool,
    pub scan_interval_hours: u32,
    pub vulnerability_db_url: String,
    pub severity_threshold: String,
    pub auto_remediation: bool,
    pub notification_webhook: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZeroTrustConfig {
    pub enabled: bool,
    pub verify_all_requests: bool,
    pub certificate_validation: bool,
    pub mutual_tls: bool,
    pub network_segmentation: bool,
    pub device_verification: bool,
}

impl SecurityConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            server: ServerConfig {
                host: env::var("SECURITY_SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
                port: env::var("SECURITY_SERVER_PORT")
                    .unwrap_or_else(|_| "8087".to_string())
                    .parse()?,
                cors_origins: env::var("SECURITY_CORS_ORIGINS")
                    .unwrap_or_else(|_| "*".to_string())
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect(),
            },
            database: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/adx_core".to_string()),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()?,
                min_connections: env::var("DATABASE_MIN_CONNECTIONS")
                    .unwrap_or_else(|_| "1".to_string())
                    .parse()?,
                acquire_timeout: env::var("DATABASE_ACQUIRE_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                idle_timeout: env::var("DATABASE_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()?,
                max_lifetime: env::var("DATABASE_MAX_LIFETIME")
                    .unwrap_or_else(|_| "1800".to_string())
                    .parse()?,
            },
            temporal: TemporalConfig {
                server_url: env::var("TEMPORAL_SERVER_URL")
                    .unwrap_or_else(|_| "http://localhost:7233".to_string()),
                namespace: env::var("TEMPORAL_NAMESPACE")
                    .unwrap_or_else(|_| "default".to_string()),
                task_queue: env::var("TEMPORAL_TASK_QUEUE")
                    .unwrap_or_else(|_| "security-task-queue".to_string()),
                client_name: env::var("TEMPORAL_CLIENT_NAME")
                    .unwrap_or_else(|_| "security-service".to_string()),
                client_version: env::var("TEMPORAL_CLIENT_VERSION")
                    .unwrap_or_else(|_| "1.0.0".to_string()),
            },
            audit: AuditConfig {
                enabled: env::var("AUDIT_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                log_level: env::var("AUDIT_LOG_LEVEL")
                    .unwrap_or_else(|_| "INFO".to_string()),
                retention_days: env::var("AUDIT_RETENTION_DAYS")
                    .unwrap_or_else(|_| "2555".to_string()) // 7 years
                    .parse()?,
                batch_size: env::var("AUDIT_BATCH_SIZE")
                    .unwrap_or_else(|_| "100".to_string())
                    .parse()?,
                flush_interval_seconds: env::var("AUDIT_FLUSH_INTERVAL")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()?,
                storage_backend: env::var("AUDIT_STORAGE_BACKEND")
                    .unwrap_or_else(|_| "database".to_string()),
                encryption_enabled: env::var("AUDIT_ENCRYPTION_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
            },
            compliance: ComplianceConfig {
                gdpr_enabled: env::var("GDPR_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                data_retention_days: env::var("DATA_RETENTION_DAYS")
                    .unwrap_or_else(|_| "2555".to_string()) // 7 years default
                    .parse()?,
                automatic_deletion: env::var("AUTOMATIC_DELETION")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()?,
                export_format: env::var("EXPORT_FORMAT")
                    .unwrap_or_else(|_| "json".to_string()),
                notification_email: env::var("COMPLIANCE_NOTIFICATION_EMAIL")
                    .unwrap_or_else(|_| "compliance@adxcore.com".to_string()),
                compliance_officer_email: env::var("COMPLIANCE_OFFICER_EMAIL")
                    .unwrap_or_else(|_| "dpo@adxcore.com".to_string()),
            },
            encryption: EncryptionConfig {
                algorithm: env::var("ENCRYPTION_ALGORITHM")
                    .unwrap_or_else(|_| "AES-256-GCM".to_string()),
                key_rotation_days: env::var("KEY_ROTATION_DAYS")
                    .unwrap_or_else(|_| "90".to_string())
                    .parse()?,
                master_key_id: env::var("MASTER_KEY_ID")
                    .unwrap_or_else(|_| "adx-core-master-key".to_string()),
                kms_provider: env::var("KMS_PROVIDER")
                    .unwrap_or_else(|_| "local".to_string()),
                at_rest_enabled: env::var("ENCRYPTION_AT_REST")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                in_transit_enabled: env::var("ENCRYPTION_IN_TRANSIT")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
            },
            scanning: ScanningConfig {
                enabled: env::var("SECURITY_SCANNING_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                scan_interval_hours: env::var("SCAN_INTERVAL_HOURS")
                    .unwrap_or_else(|_| "24".to_string())
                    .parse()?,
                vulnerability_db_url: env::var("VULNERABILITY_DB_URL")
                    .unwrap_or_else(|_| "https://nvd.nist.gov/feeds/json/cve/1.1/".to_string()),
                severity_threshold: env::var("SEVERITY_THRESHOLD")
                    .unwrap_or_else(|_| "MEDIUM".to_string()),
                auto_remediation: env::var("AUTO_REMEDIATION")
                    .unwrap_or_else(|_| "false".to_string())
                    .parse()?,
                notification_webhook: env::var("SECURITY_NOTIFICATION_WEBHOOK").ok(),
            },
            zero_trust: ZeroTrustConfig {
                enabled: env::var("ZERO_TRUST_ENABLED")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                verify_all_requests: env::var("VERIFY_ALL_REQUESTS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                certificate_validation: env::var("CERTIFICATE_VALIDATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                mutual_tls: env::var("MUTUAL_TLS")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                network_segmentation: env::var("NETWORK_SEGMENTATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
                device_verification: env::var("DEVICE_VERIFICATION")
                    .unwrap_or_else(|_| "true".to_string())
                    .parse()?,
            },
        })
    }
}