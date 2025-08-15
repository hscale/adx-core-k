use thiserror::Error;

pub type Result<T> = std::result::Result<T, LicenseError>;

#[derive(Error, Debug)]
pub enum LicenseError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("License not found: {0}")]
    LicenseNotFound(String),
    
    #[error("License expired: {license_id}")]
    LicenseExpired { license_id: String },
    
    #[error("License suspended: {license_id}")]
    LicenseSuspended { license_id: String },
    
    #[error("Quota exceeded: {quota_name} (used: {current_usage}, limit: {quota_limit})")]
    QuotaExceeded {
        quota_name: String,
        current_usage: i64,
        quota_limit: i64,
    },
    
    #[error("Quota not found: {quota_name}")]
    QuotaNotFound { quota_name: String },
    
    #[error("Payment processing error: {0}")]
    PaymentError(String),
    

    
    #[error("Billing error: {0}")]
    BillingError(String),
    
    #[error("Invalid license key: {0}")]
    InvalidLicenseKey(String),
    
    #[error("Subscription not found: {0}")]
    SubscriptionNotFound(String),
    
    #[error("Configuration error: {0}")]
    ConfigError(String),
    
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    #[error("Temporal workflow error: {0}")]
    WorkflowError(#[from] adx_shared::WorkflowError),
    
    #[error("Temporal activity error: {0}")]
    ActivityError(#[from] adx_shared::ActivityError),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("HTTP client error: {0}")]
    HttpError(#[from] reqwest::Error),
    
    #[error("Internal error: {0}")]
    Internal(String),
}

impl LicenseError {
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            LicenseError::Database(_) |
            LicenseError::HttpError(_) |
            LicenseError::PaymentError(_)
        )
    }
    
    pub fn error_code(&self) -> &'static str {
        match self {
            LicenseError::Database(_) => "DATABASE_ERROR",
            LicenseError::LicenseNotFound(_) => "LICENSE_NOT_FOUND",
            LicenseError::LicenseExpired { .. } => "LICENSE_EXPIRED",
            LicenseError::LicenseSuspended { .. } => "LICENSE_SUSPENDED",
            LicenseError::QuotaExceeded { .. } => "QUOTA_EXCEEDED",
            LicenseError::QuotaNotFound { .. } => "QUOTA_NOT_FOUND",
            LicenseError::PaymentError(_) => "PAYMENT_ERROR",

            LicenseError::BillingError(_) => "BILLING_ERROR",
            LicenseError::InvalidLicenseKey(_) => "INVALID_LICENSE_KEY",
            LicenseError::SubscriptionNotFound(_) => "SUBSCRIPTION_NOT_FOUND",
            LicenseError::ConfigError(_) => "CONFIG_ERROR",
            LicenseError::ValidationError(_) => "VALIDATION_ERROR",
            LicenseError::WorkflowError(_) => "WORKFLOW_ERROR",
            LicenseError::ActivityError(_) => "ACTIVITY_ERROR",
            LicenseError::SerializationError(_) => "SERIALIZATION_ERROR",
            LicenseError::HttpError(_) => "HTTP_ERROR",
            LicenseError::Internal(_) => "INTERNAL_ERROR",
        }
    }
}