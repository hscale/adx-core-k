use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseConfig {
    pub database_url: String,
    pub redis_url: String,
    pub server_port: u16,
    pub temporal: TemporalConfig,
    pub stripe: StripeConfig,
    pub paypal: PayPalConfig,
    pub billing: BillingConfig,
    pub quotas: QuotaConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemporalConfig {
    pub server_url: String,
    pub namespace: String,
    pub task_queue: String,
    pub workflow_timeout_seconds: u64,
    pub activity_timeout_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StripeConfig {
    pub secret_key: String,
    pub publishable_key: String,
    pub webhook_secret: String,
    pub api_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayPalConfig {
    pub client_id: String,
    pub client_secret: String,
    pub environment: String, // "sandbox" or "live"
    pub webhook_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingConfig {
    pub invoice_prefix: String,
    pub default_currency: String,
    pub tax_rate: f64,
    pub grace_period_days: i32,
    pub retry_failed_payments: bool,
    pub max_payment_retries: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaConfig {
    pub enforcement_enabled: bool,
    pub real_time_monitoring: bool,
    pub usage_aggregation_interval_seconds: u64,
    pub warning_notification_enabled: bool,
    pub auto_suspend_on_violation: bool,
}

impl Default for LicenseConfig {
    fn default() -> Self {
        Self {
            database_url: "postgresql://localhost:5432/adx_core".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            server_port: 8087,
            temporal: TemporalConfig::default(),
            stripe: StripeConfig::default(),
            paypal: PayPalConfig::default(),
            billing: BillingConfig::default(),
            quotas: QuotaConfig::default(),
        }
    }
}

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:7233".to_string(),
            namespace: "default".to_string(),
            task_queue: "license-service-queue".to_string(),
            workflow_timeout_seconds: 3600, // 1 hour
            activity_timeout_seconds: 300,  // 5 minutes
        }
    }
}

impl Default for StripeConfig {
    fn default() -> Self {
        Self {
            secret_key: "sk_test_...".to_string(),
            publishable_key: "pk_test_...".to_string(),
            webhook_secret: "whsec_...".to_string(),
            api_version: "2023-10-16".to_string(),
        }
    }
}

impl Default for PayPalConfig {
    fn default() -> Self {
        Self {
            client_id: "".to_string(),
            client_secret: "".to_string(),
            environment: "sandbox".to_string(),
            webhook_id: "".to_string(),
        }
    }
}

impl Default for BillingConfig {
    fn default() -> Self {
        Self {
            invoice_prefix: "ADX".to_string(),
            default_currency: "USD".to_string(),
            tax_rate: 0.0,
            grace_period_days: 7,
            retry_failed_payments: true,
            max_payment_retries: 3,
        }
    }
}

impl Default for QuotaConfig {
    fn default() -> Self {
        Self {
            enforcement_enabled: true,
            real_time_monitoring: true,
            usage_aggregation_interval_seconds: 300, // 5 minutes
            warning_notification_enabled: true,
            auto_suspend_on_violation: false,
        }
    }
}

impl LicenseConfig {
    pub fn from_env() -> Result<Self, config::ConfigError> {
        let mut cfg = config::Config::builder()
            .add_source(config::Environment::with_prefix("LICENSE_SERVICE"))
            .build()?;
        
        // Set defaults
        cfg.set_default("server_port", 8087)?;
        cfg.set_default("temporal.server_url", "http://localhost:7233")?;
        cfg.set_default("temporal.namespace", "default")?;
        cfg.set_default("temporal.task_queue", "license-service-queue")?;
        cfg.set_default("temporal.workflow_timeout_seconds", 3600)?;
        cfg.set_default("temporal.activity_timeout_seconds", 300)?;
        cfg.set_default("billing.invoice_prefix", "ADX")?;
        cfg.set_default("billing.default_currency", "USD")?;
        cfg.set_default("billing.tax_rate", 0.0)?;
        cfg.set_default("billing.grace_period_days", 7)?;
        cfg.set_default("billing.retry_failed_payments", true)?;
        cfg.set_default("billing.max_payment_retries", 3)?;
        cfg.set_default("quotas.enforcement_enabled", true)?;
        cfg.set_default("quotas.real_time_monitoring", true)?;
        cfg.set_default("quotas.usage_aggregation_interval_seconds", 300)?;
        cfg.set_default("quotas.warning_notification_enabled", true)?;
        cfg.set_default("quotas.auto_suspend_on_violation", false)?;
        
        cfg.try_deserialize()
    }
}