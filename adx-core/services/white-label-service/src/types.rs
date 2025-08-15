use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDomain {
    pub id: Uuid,
    pub tenant_id: String,
    pub domain: String,
    pub status: DomainStatus,
    pub verification_token: String,
    pub ssl_certificate_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainStatus {
    Pending,
    Verifying,
    Verified,
    Failed,
    Expired,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelBranding {
    pub id: Uuid,
    pub tenant_id: String,
    pub brand_name: String,
    pub logo_url: Option<String>,
    pub favicon_url: Option<String>,
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub font_family: String,
    pub custom_css: Option<String>,
    pub email_templates: HashMap<String, EmailTemplate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplate {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
    pub variables: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResellerHierarchy {
    pub id: Uuid,
    pub parent_reseller_id: Option<Uuid>,
    pub tenant_id: String,
    pub reseller_name: String,
    pub reseller_type: ResellerType,
    pub commission_rate: f64,
    pub revenue_share_model: RevenueShareModel,
    pub support_contact: SupportContact,
    pub branding_overrides: Option<WhiteLabelBranding>,
    pub allowed_features: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResellerType {
    DirectReseller,
    SubReseller,
    Partner,
    Distributor,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevenueShareModel {
    pub model_type: RevenueShareType,
    pub percentage: f64,
    pub minimum_amount: Option<f64>,
    pub maximum_amount: Option<f64>,
    pub tier_rates: Option<Vec<TierRate>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RevenueShareType {
    Flat,
    Tiered,
    Progressive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierRate {
    pub min_revenue: f64,
    pub max_revenue: Option<f64>,
    pub rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupportContact {
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub support_url: Option<String>,
    pub escalation_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandingAsset {
    pub id: Uuid,
    pub tenant_id: String,
    pub asset_type: AssetType,
    pub original_filename: String,
    pub file_path: String,
    pub file_size: u64,
    pub mime_type: String,
    pub dimensions: Option<AssetDimensions>,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetType {
    Logo,
    Favicon,
    BackgroundImage,
    EmailHeader,
    EmailFooter,
    CustomIcon,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetDimensions {
    pub width: u32,
    pub height: u32,
}

// Workflow request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDomainSetupRequest {
    pub tenant_id: String,
    pub domain: String,
    pub ssl_enabled: bool,
    pub auto_redirect: bool,
    pub dns_provider: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomDomainSetupResult {
    pub domain_id: Uuid,
    pub verification_token: String,
    pub dns_records: Vec<DnsRecord>,
    pub ssl_certificate_id: Option<String>,
    pub status: DomainStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsRecord {
    pub record_type: String,
    pub name: String,
    pub value: String,
    pub ttl: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelBrandingRequest {
    pub tenant_id: String,
    pub brand_name: String,
    pub logo_file: Option<Vec<u8>>,
    pub favicon_file: Option<Vec<u8>>,
    pub color_scheme: ColorScheme,
    pub typography: Typography,
    pub email_templates: HashMap<String, EmailTemplateRequest>,
    pub custom_css: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub primary_color: String,
    pub secondary_color: String,
    pub accent_color: String,
    pub background_color: String,
    pub text_color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Typography {
    pub font_family: String,
    pub heading_font: Option<String>,
    pub font_sizes: FontSizes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontSizes {
    pub small: String,
    pub medium: String,
    pub large: String,
    pub extra_large: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailTemplateRequest {
    pub subject: String,
    pub html_body: String,
    pub text_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WhiteLabelBrandingResult {
    pub branding_id: Uuid,
    pub asset_urls: HashMap<String, String>,
    pub css_url: String,
    pub preview_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResellerSetupRequest {
    pub parent_reseller_id: Option<Uuid>,
    pub tenant_id: String,
    pub reseller_name: String,
    pub reseller_type: ResellerType,
    pub commission_rate: f64,
    pub revenue_share_model: RevenueShareModel,
    pub support_contact: SupportContact,
    pub allowed_features: Vec<String>,
    pub branding_overrides: Option<WhiteLabelBrandingRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResellerSetupResult {
    pub reseller_id: Uuid,
    pub hierarchy_level: u32,
    pub effective_commission_rate: f64,
    pub branding_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainVerificationResult {
    pub verified: bool,
    pub verification_method: String,
    pub dns_records_found: Vec<DnsRecord>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SslCertificateResult {
    pub certificate_id: String,
    pub certificate_arn: Option<String>,
    pub status: SslStatus,
    pub expires_at: DateTime<Utc>,
    pub auto_renewal: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SslStatus {
    Pending,
    Issued,
    Failed,
    Expired,
    Revoked,
}