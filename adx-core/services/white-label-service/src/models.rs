use crate::types::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CustomDomainModel {
    pub id: Uuid,
    pub tenant_id: String,
    pub domain: String,
    pub status: String,
    pub verification_token: String,
    pub ssl_certificate_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl From<CustomDomainModel> for CustomDomain {
    fn from(model: CustomDomainModel) -> Self {
        Self {
            id: model.id,
            tenant_id: model.tenant_id,
            domain: model.domain,
            status: match model.status.as_str() {
                "pending" => DomainStatus::Pending,
                "verifying" => DomainStatus::Verifying,
                "verified" => DomainStatus::Verified,
                "failed" => DomainStatus::Failed,
                "expired" => DomainStatus::Expired,
                "suspended" => DomainStatus::Suspended,
                _ => DomainStatus::Pending,
            },
            verification_token: model.verification_token,
            ssl_certificate_id: model.ssl_certificate_id,
            created_at: model.created_at,
            verified_at: model.verified_at,
            expires_at: model.expires_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct WhiteLabelBrandingModel {
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
    pub email_templates: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<WhiteLabelBrandingModel> for WhiteLabelBranding {
    fn from(model: WhiteLabelBrandingModel) -> Self {
        let email_templates = serde_json::from_value(model.email_templates)
            .unwrap_or_default();

        Self {
            id: model.id,
            tenant_id: model.tenant_id,
            brand_name: model.brand_name,
            logo_url: model.logo_url,
            favicon_url: model.favicon_url,
            primary_color: model.primary_color,
            secondary_color: model.secondary_color,
            accent_color: model.accent_color,
            font_family: model.font_family,
            custom_css: model.custom_css,
            email_templates,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ResellerHierarchyModel {
    pub id: Uuid,
    pub parent_reseller_id: Option<Uuid>,
    pub tenant_id: String,
    pub reseller_name: String,
    pub reseller_type: String,
    pub commission_rate: f64,
    pub revenue_share_model: serde_json::Value,
    pub support_contact: serde_json::Value,
    pub branding_overrides: Option<Uuid>,
    pub allowed_features: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ResellerHierarchyModel> for ResellerHierarchy {
    fn from(model: ResellerHierarchyModel) -> Self {
        let reseller_type = match model.reseller_type.as_str() {
            "direct_reseller" => ResellerType::DirectReseller,
            "sub_reseller" => ResellerType::SubReseller,
            "partner" => ResellerType::Partner,
            "distributor" => ResellerType::Distributor,
            _ => ResellerType::DirectReseller,
        };

        let revenue_share_model = serde_json::from_value(model.revenue_share_model)
            .unwrap_or(RevenueShareModel {
                model_type: RevenueShareType::Flat,
                percentage: 0.0,
                minimum_amount: None,
                maximum_amount: None,
                tier_rates: None,
            });

        let support_contact = serde_json::from_value(model.support_contact)
            .unwrap_or(SupportContact {
                name: "Support".to_string(),
                email: "support@example.com".to_string(),
                phone: None,
                support_url: None,
                escalation_email: None,
            });

        Self {
            id: model.id,
            parent_reseller_id: model.parent_reseller_id,
            tenant_id: model.tenant_id,
            reseller_name: model.reseller_name,
            reseller_type,
            commission_rate: model.commission_rate,
            revenue_share_model,
            support_contact,
            branding_overrides: None, // This would be loaded separately if needed
            allowed_features: model.allowed_features,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BrandingAssetModel {
    pub id: Uuid,
    pub tenant_id: String,
    pub asset_type: String,
    pub original_filename: String,
    pub file_path: String,
    pub file_size: i64,
    pub mime_type: String,
    pub dimensions_width: Option<i32>,
    pub dimensions_height: Option<i32>,
    pub checksum: String,
    pub created_at: DateTime<Utc>,
}

impl From<BrandingAssetModel> for BrandingAsset {
    fn from(model: BrandingAssetModel) -> Self {
        let asset_type = match model.asset_type.as_str() {
            "logo" => AssetType::Logo,
            "favicon" => AssetType::Favicon,
            "background_image" => AssetType::BackgroundImage,
            "email_header" => AssetType::EmailHeader,
            "email_footer" => AssetType::EmailFooter,
            "custom_icon" => AssetType::CustomIcon,
            _ => AssetType::Logo,
        };

        let dimensions = if let (Some(width), Some(height)) = (model.dimensions_width, model.dimensions_height) {
            Some(AssetDimensions {
                width: width as u32,
                height: height as u32,
            })
        } else {
            None
        };

        Self {
            id: model.id,
            tenant_id: model.tenant_id,
            asset_type,
            original_filename: model.original_filename,
            file_path: model.file_path,
            file_size: model.file_size as u64,
            mime_type: model.mime_type,
            dimensions,
            checksum: model.checksum,
            created_at: model.created_at,
        }
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct BrandingBackupModel {
    pub id: Uuid,
    pub tenant_id: String,
    pub original_branding_id: Option<Uuid>,
    pub backup_data: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct RevenueSharingConfigModel {
    pub id: Uuid,
    pub reseller_id: Uuid,
    pub parent_reseller_id: Option<Uuid>,
    pub revenue_share_model: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct SupportRoutingConfigModel {
    pub id: Uuid,
    pub reseller_id: Uuid,
    pub support_contact: serde_json::Value,
    pub hierarchy_level: i32,
    pub created_at: DateTime<Utc>,
}