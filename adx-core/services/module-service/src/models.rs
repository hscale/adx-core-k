use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;

use crate::types::{ModuleStatus, ModuleCategory, ModuleManifest, ModuleAuthor};

// Database models
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleRecord {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author_name: String,
    pub author_email: String,
    pub author_website: Option<String>,
    pub author_organization: Option<String>,
    pub category: String,
    pub manifest_json: serde_json::Value,
    pub status: String,
    pub tenant_id: Option<String>,
    pub package_url: Option<String>,
    pub package_hash: Option<String>,
    pub installation_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ModuleRecord {
    pub fn to_module(&self) -> Result<crate::types::Module, serde_json::Error> {
        Ok(crate::types::Module {
            id: self.id.clone(),
            name: self.name.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            author: ModuleAuthor {
                name: self.author_name.clone(),
                email: self.author_email.clone(),
                website: self.author_website.clone(),
                organization: self.author_organization.clone(),
            },
            category: serde_json::from_str(&format!("\"{}\"", self.category))
                .unwrap_or(ModuleCategory::Other(self.category.clone())),
            manifest: serde_json::from_value(self.manifest_json.clone())?,
            status: serde_json::from_str(&format!("\"{}\"", self.status))
                .unwrap_or(ModuleStatus::Available),
            tenant_id: self.tenant_id.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleDependencyRecord {
    pub id: Uuid,
    pub module_id: String,
    pub dependency_id: String,
    pub version_requirement: String,
    pub optional: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleInstallationRecord {
    pub id: String,
    pub module_id: String,
    pub tenant_id: String,
    pub version: String,
    pub status: String,
    pub configuration_json: Option<serde_json::Value>,
    pub installation_path: Option<String>,
    pub sandbox_config_json: Option<serde_json::Value>,
    pub installed_by: String,
    pub installed_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModulePermissionRecord {
    pub id: Uuid,
    pub module_id: String,
    pub permission_type: String,
    pub resource: Option<String>,
    pub granted: bool,
    pub granted_by: Option<String>,
    pub granted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleUsageRecord {
    pub id: Uuid,
    pub module_id: String,
    pub tenant_id: String,
    pub usage_type: String,
    pub usage_count: i64,
    pub resource_usage_json: Option<serde_json::Value>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleReviewRecord {
    pub id: Uuid,
    pub module_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub rating: i16,
    pub title: Option<String>,
    pub comment: Option<String>,
    pub helpful_count: i32,
    pub verified_purchase: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleVersionRecord {
    pub id: Uuid,
    pub module_id: String,
    pub version: String,
    pub changelog: Option<String>,
    pub package_url: String,
    pub package_hash: String,
    pub package_size_bytes: i64,
    pub manifest_json: serde_json::Value,
    pub security_scan_json: Option<serde_json::Value>,
    pub performance_metrics_json: Option<serde_json::Value>,
    pub compatibility_json: Option<serde_json::Value>,
    pub is_stable: bool,
    pub is_deprecated: bool,
    pub deprecation_reason: Option<String>,
    pub published_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleMarketplaceRecord {
    pub id: String,
    pub name: String,
    pub description: String,
    pub long_description: Option<String>,
    pub current_version: String,
    pub author_name: String,
    pub author_email: String,
    pub category: String,
    pub subcategory: Option<String>,
    pub price_model: Option<String>,
    pub price_amount: Option<f64>,
    pub price_currency: Option<String>,
    pub billing_period: Option<String>,
    pub rating_average: f32,
    pub rating_count: i32,
    pub download_count: i64,
    pub active_installation_count: i64,
    pub screenshots_json: Option<serde_json::Value>,
    pub demo_url: Option<String>,
    pub documentation_url: Option<String>,
    pub support_url: Option<String>,
    pub tags_json: Option<serde_json::Value>,
    pub featured: bool,
    pub verified: bool,
    pub published: bool,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleWorkflowRecord {
    pub id: String,
    pub workflow_type: String,
    pub module_id: String,
    pub tenant_id: String,
    pub status: String,
    pub input_json: serde_json::Value,
    pub output_json: Option<serde_json::Value>,
    pub error_message: Option<String>,
    pub progress_json: Option<serde_json::Value>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleSandboxRecord {
    pub id: Uuid,
    pub module_id: String,
    pub tenant_id: String,
    pub sandbox_config_json: serde_json::Value,
    pub resource_usage_json: Option<serde_json::Value>,
    pub violations_json: Option<serde_json::Value>,
    pub last_violation_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ModuleSecurityScanRecord {
    pub id: Uuid,
    pub module_id: String,
    pub version: String,
    pub scan_type: String,
    pub scanner_version: String,
    pub passed: bool,
    pub score: i16,
    pub vulnerabilities_json: serde_json::Value,
    pub scan_duration_seconds: i32,
    pub scanned_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// Request/Response models for API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateModuleRequest {
    pub name: String,
    pub description: String,
    pub author: ModuleAuthor,
    pub category: ModuleCategory,
    pub manifest: ModuleManifest,
    pub package_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleRequest {
    pub description: Option<String>,
    pub category: Option<ModuleCategory>,
    pub manifest: Option<ModuleManifest>,
    pub package_data: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleListResponse {
    pub modules: Vec<crate::types::Module>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInstallationResponse {
    pub installation_id: String,
    pub module_id: String,
    pub version: String,
    pub status: ModuleStatus,
    pub configuration: Option<HashMap<String, serde_json::Value>>,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleUsageResponse {
    pub module_id: String,
    pub tenant_id: String,
    pub total_usage: i64,
    pub usage_by_type: HashMap<String, i64>,
    pub resource_usage: Option<serde_json::Value>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealthResponse {
    pub module_id: String,
    pub status: ModuleStatus,
    pub health_score: f32,
    pub resource_usage: ResourceUsageSnapshot,
    pub last_check_at: DateTime<Utc>,
    pub issues: Vec<ModuleHealthIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    pub memory_mb: f64,
    pub cpu_percent: f64,
    pub storage_mb: f64,
    pub network_bytes_per_second: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHealthIssue {
    pub severity: String,
    pub category: String,
    pub message: String,
    pub recommendation: Option<String>,
    pub detected_at: DateTime<Utc>,
}

// Workflow-specific models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallModuleWorkflowInput {
    pub module_id: String,
    pub version: String,
    pub tenant_id: String,
    pub user_id: String,
    pub configuration: Option<HashMap<String, serde_json::Value>>,
    pub auto_activate: bool,
    pub force_reinstall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallModuleWorkflowOutput {
    pub installation_id: String,
    pub module_id: String,
    pub version: String,
    pub status: ModuleStatus,
    pub dependencies_installed: Vec<String>,
    pub configuration_applied: bool,
    pub sandbox_configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleWorkflowInput {
    pub module_id: String,
    pub target_version: String,
    pub tenant_id: String,
    pub user_id: String,
    pub backup_current: bool,
    pub rollback_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleWorkflowOutput {
    pub update_id: String,
    pub module_id: String,
    pub from_version: String,
    pub to_version: String,
    pub status: ModuleStatus,
    pub backup_id: Option<String>,
    pub migration_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallModuleWorkflowInput {
    pub module_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub cleanup_data: bool,
    pub force_uninstall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallModuleWorkflowOutput {
    pub uninstallation_id: String,
    pub module_id: String,
    pub status: ModuleStatus,
    pub data_cleaned: bool,
    pub dependencies_removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSyncWorkflowInput {
    pub sync_type: String,
    pub module_ids: Option<Vec<String>>,
    pub force_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSyncWorkflowOutput {
    pub sync_id: String,
    pub modules_synced: u32,
    pub modules_updated: u32,
    pub modules_added: u32,
    pub modules_removed: u32,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanWorkflowInput {
    pub module_id: String,
    pub version: String,
    pub scan_type: String,
    pub deep_scan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanWorkflowOutput {
    pub scan_id: String,
    pub module_id: String,
    pub version: String,
    pub passed: bool,
    pub score: u8,
    pub vulnerabilities_count: u32,
    pub scan_duration_seconds: u32,
}