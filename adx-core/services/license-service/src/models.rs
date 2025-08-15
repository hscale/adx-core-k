use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "subscription_tier", rename_all = "lowercase")]
pub enum SubscriptionTier {
    Free,
    Professional,
    Enterprise,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "license_status", rename_all = "lowercase")]
pub enum LicenseStatus {
    Active,
    Expired,
    Suspended,
    Cancelled,
    Pending,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "billing_cycle", rename_all = "lowercase")]
pub enum BillingCycle {
    Monthly,
    Yearly,
    OneTime,
    UsageBased,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "payment_status", rename_all = "lowercase")]
pub enum PaymentStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct License {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub license_key: String,
    pub subscription_tier: SubscriptionTier,
    pub status: LicenseStatus,
    pub billing_cycle: BillingCycle,
    
    // Pricing information
    pub base_price: Decimal,
    pub currency: String,
    
    // License validity
    pub starts_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: bool,
    
    // Features and limits
    pub features: serde_json::Value,
    pub custom_quotas: Option<serde_json::Value>,
    
    // Billing information
    pub stripe_subscription_id: Option<String>,
    pub stripe_customer_id: Option<String>,
    pub paypal_subscription_id: Option<String>,
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct QuotaDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub unit: String,
    pub category: String,
    
    // Default limits per tier
    pub free_limit: i64,
    pub professional_limit: i64,
    pub enterprise_limit: i64,
    
    // Enforcement settings
    pub enforce_hard_limit: bool,
    pub warning_threshold_percent: i32,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TenantQuota {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub quota_definition_id: Uuid,
    
    // Current quota settings
    pub quota_limit: i64,
    pub current_usage: i64,
    
    // Usage tracking
    pub last_reset_at: DateTime<Utc>,
    pub reset_period_days: i32,
    
    // Overrides
    pub custom_limit: Option<i64>,
    pub notes: Option<String>,
    
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UsageLog {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub quota_definition_id: Uuid,
    
    // Usage details
    pub amount: i64,
    pub operation_type: Option<String>,
    pub resource_id: Option<Uuid>,
    
    // Context
    pub user_id: Option<Uuid>,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    
    // Metadata
    pub metadata: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct BillingHistory {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub license_id: Uuid,
    
    // Invoice information
    pub invoice_number: String,
    pub amount: Decimal,
    pub currency: String,
    pub tax_amount: Decimal,
    
    // Billing period
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    
    // Payment information
    pub payment_status: PaymentStatus,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    
    // Usage-based billing details
    pub usage_details: Option<serde_json::Value>,
    
    // Metadata
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceLog {
    pub id: Uuid,
    pub tenant_id: Uuid,
    
    // Event information
    pub event_type: String,
    pub event_category: String,
    pub severity: String,
    
    // Event details
    pub description: String,
    pub details: Option<serde_json::Value>,
    
    // Context
    pub user_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
    pub ip_address: Option<std::net::IpAddr>,
    
    // Resolution
    pub resolved: bool,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolved_by: Option<Uuid>,
    pub resolution_notes: Option<String>,
    
    pub created_at: DateTime<Utc>,
}

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateLicenseRequest {
    pub tenant_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub billing_cycle: BillingCycle,
    pub base_price: Decimal,
    pub currency: String,
    pub features: Vec<String>,
    pub custom_quotas: Option<serde_json::Value>,
    pub auto_renew: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateLicenseRequest {
    pub subscription_tier: Option<SubscriptionTier>,
    pub status: Option<LicenseStatus>,
    pub base_price: Option<Decimal>,
    pub expires_at: Option<DateTime<Utc>>,
    pub auto_renew: Option<bool>,
    pub features: Option<Vec<String>>,
    pub custom_quotas: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaUsageRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
    pub amount: i64,
    pub operation_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub remaining: i64,
    pub warning_threshold_reached: bool,
    pub quota_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingInvoice {
    pub invoice_number: String,
    pub tenant_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub tax_amount: Decimal,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub line_items: Vec<BillingLineItem>,
    pub usage_summary: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingLineItem {
    pub description: String,
    pub quantity: i64,
    pub unit_price: Decimal,
    pub total_price: Decimal,
    pub item_type: String, // 'subscription', 'usage', 'overage', 'tax'
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ComplianceReport {
    pub tenant_id: Uuid,
    pub report_period_start: DateTime<Utc>,
    pub report_period_end: DateTime<Utc>,
    pub license_status: LicenseStatus,
    pub quota_violations: Vec<QuotaViolation>,
    pub billing_issues: Vec<BillingIssue>,
    pub compliance_score: f64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaViolation {
    pub quota_name: String,
    pub violation_count: i64,
    pub last_violation: DateTime<Utc>,
    pub severity: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BillingIssue {
    pub issue_type: String,
    pub description: String,
    pub amount: Option<Decimal>,
    pub occurred_at: DateTime<Utc>,
    pub resolved: bool,
}

impl License {
    pub fn is_active(&self) -> bool {
        matches!(self.status, LicenseStatus::Active) &&
        self.expires_at.map_or(true, |exp| exp > Utc::now())
    }
    
    pub fn is_expired(&self) -> bool {
        self.expires_at.map_or(false, |exp| exp <= Utc::now())
    }
    
    pub fn days_until_expiry(&self) -> Option<i64> {
        self.expires_at.map(|exp| (exp - Utc::now()).num_days())
    }
    
    pub fn has_feature(&self, feature: &str) -> bool {
        if let Ok(features) = serde_json::from_value::<Vec<String>>(self.features.clone()) {
            features.contains(&feature.to_string())
        } else {
            false
        }
    }
}

impl TenantQuota {
    pub fn is_exceeded(&self) -> bool {
        self.quota_limit >= 0 && self.current_usage >= self.quota_limit
    }
    
    pub fn usage_percentage(&self) -> f64 {
        if self.quota_limit <= 0 {
            0.0
        } else {
            (self.current_usage as f64 / self.quota_limit as f64) * 100.0
        }
    }
    
    pub fn remaining(&self) -> i64 {
        if self.quota_limit < 0 {
            -1 // Unlimited
        } else {
            (self.quota_limit - self.current_usage).max(0)
        }
    }
    
    pub fn is_warning_threshold_reached(&self, warning_threshold: i32) -> bool {
        self.usage_percentage() >= warning_threshold as f64
    }
}