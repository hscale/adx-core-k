use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

// Core module types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: ModuleAuthor,
    pub category: ModuleCategory,
    pub manifest: ModuleManifest,
    pub status: ModuleStatus,
    pub tenant_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: ModuleAuthor,
    pub license: String,
    pub adx_core: AdxCoreCompatibility,
    pub dependencies: HashMap<String, String>,
    pub permissions: Vec<ModulePermission>,
    pub extension_points: ExtensionPoints,
    pub resources: ResourceRequirements,
    pub configuration: Option<ModuleConfiguration>,
    pub hooks: Option<ModuleHooks>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAuthor {
    pub name: String,
    pub email: String,
    pub website: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleCategory {
    BusinessManagement,
    Analytics,
    Integration,
    Workflow,
    UI,
    Security,
    Storage,
    Communication,
    Development,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleStatus {
    Available,
    Installing,
    Installed,
    Activating,
    Active,
    Deactivating,
    Inactive,
    Updating,
    Uninstalling,
    Failed,
    Suspended,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdxCoreCompatibility {
    pub min_version: String,
    pub max_version: Option<String>,
    pub tested_versions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModulePermission {
    DatabaseRead(String),
    DatabaseWrite(String),
    ApiExternal(String),
    FileRead(String),
    FileWrite(String),
    WorkflowExecute(String),
    TenantDataAccess,
    UserDataAccess,
    SystemConfiguration,
    ModuleManagement,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPoints {
    pub backend: Option<BackendExtensions>,
    pub frontend: Option<FrontendExtensions>,
    pub workflows: Option<WorkflowExtensions>,
    pub database: Option<DatabaseExtensions>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendExtensions {
    pub activities: Vec<String>,
    pub endpoints: Vec<String>,
    pub middleware: Vec<String>,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrontendExtensions {
    pub components: Vec<String>,
    pub routes: Vec<String>,
    pub hooks: Vec<String>,
    pub providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExtensions {
    pub workflows: Vec<String>,
    pub activities: Vec<String>,
    pub signals: Vec<String>,
    pub queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseExtensions {
    pub migrations: Vec<String>,
    pub seeds: Vec<String>,
    pub views: Vec<String>,
    pub functions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub memory_mb: u64,
    pub cpu_cores: f64,
    pub storage_mb: u64,
    pub network_required: bool,
    pub gpu_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfiguration {
    pub schema: serde_json::Value,
    pub defaults: HashMap<String, serde_json::Value>,
    pub required: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleHooks {
    pub pre_install: Option<String>,
    pub post_install: Option<String>,
    pub pre_activate: Option<String>,
    pub post_activate: Option<String>,
    pub pre_deactivate: Option<String>,
    pub post_deactivate: Option<String>,
    pub pre_uninstall: Option<String>,
    pub post_uninstall: Option<String>,
}

// Installation and management types
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InstallModuleRequest {
    #[validate(length(min = 1, max = 100))]
    pub module_id: String,
    #[validate(length(min = 1, max = 50))]
    pub version: String,
    pub tenant_id: String,
    pub configuration: Option<HashMap<String, serde_json::Value>>,
    pub auto_activate: bool,
    pub force_reinstall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallModuleResult {
    pub module_id: String,
    pub version: String,
    pub installation_id: String,
    pub status: ModuleStatus,
    pub dependencies_installed: Vec<String>,
    pub configuration_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct UpdateModuleRequest {
    #[validate(length(min = 1, max = 100))]
    pub module_id: String,
    #[validate(length(min = 1, max = 50))]
    pub target_version: String,
    pub tenant_id: String,
    pub backup_current: bool,
    pub rollback_on_failure: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleResult {
    pub module_id: String,
    pub from_version: String,
    pub to_version: String,
    pub update_id: String,
    pub status: ModuleStatus,
    pub backup_id: Option<String>,
    pub migration_applied: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallModuleRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub cleanup_data: bool,
    pub force_uninstall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallModuleResult {
    pub module_id: String,
    pub uninstallation_id: String,
    pub status: ModuleStatus,
    pub data_cleaned: bool,
    pub dependencies_removed: Vec<String>,
}

// Marketplace types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListing {
    pub id: String,
    pub name: String,
    pub description: String,
    pub long_description: String,
    pub version: String,
    pub author: ModuleAuthor,
    pub category: ModuleCategory,
    pub subcategory: Option<String>,
    pub price: Option<ModulePrice>,
    pub rating: f32,
    pub review_count: u32,
    pub downloads: u64,
    pub active_installations: u64,
    pub screenshots: Vec<String>,
    pub demo_url: Option<String>,
    pub documentation_url: String,
    pub support_url: String,
    pub tags: Vec<String>,
    pub supported_platforms: Vec<Platform>,
    pub compatibility: CompatibilityInfo,
    pub security_scan_results: SecurityScanResults,
    pub performance_metrics: PerformanceMetrics,
    pub last_updated: DateTime<Utc>,
    pub changelog: Vec<ChangelogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePrice {
    pub model: PricingModel,
    pub amount: Option<f64>,
    pub currency: String,
    pub billing_period: Option<BillingPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    Free,
    OneTime,
    Subscription,
    Usage,
    Enterprise,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingPeriod {
    Monthly,
    Yearly,
    PerUse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Platform {
    Web,
    Desktop,
    Mobile,
    Server,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub adx_core_versions: Vec<String>,
    pub node_version: Option<String>,
    pub browser_support: Option<BrowserSupport>,
    pub os_support: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserSupport {
    pub chrome: Option<String>,
    pub firefox: Option<String>,
    pub safari: Option<String>,
    pub edge: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResults {
    pub passed: bool,
    pub score: u8,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub scan_date: DateTime<Utc>,
    pub scanner_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    pub severity: VulnerabilitySeverity,
    pub category: String,
    pub description: String,
    pub file_path: Option<String>,
    pub line_number: Option<u32>,
    pub recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VulnerabilitySeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub bundle_size_kb: u64,
    pub load_time_ms: u64,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogEntry {
    pub version: String,
    pub date: DateTime<Utc>,
    pub changes: Vec<ChangelogItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChangelogItem {
    pub type_: ChangeType,
    pub description: String,
    pub breaking: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChangeType {
    Added,
    Changed,
    Deprecated,
    Removed,
    Fixed,
    Security,
}

// Workflow types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowApiResponse<T> {
    Synchronous {
        data: T,
        execution_time_ms: u64,
        workflow_id: String,
    },
    Asynchronous {
        operation_id: String,
        status_url: String,
        stream_url: Option<String>,
        estimated_duration_seconds: Option<u64>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatusResponse {
    pub operation_id: String,
    pub status: WorkflowStatus,
    pub progress: Option<WorkflowProgress>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub message: Option<String>,
}

// Sandbox types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    pub module_id: String,
    pub tenant_id: String,
    pub resource_limits: ResourceLimits,
    pub network_policy: NetworkPolicy,
    pub file_system_policy: FileSystemPolicy,
    pub security_policy: SecurityPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f64,
    pub max_storage_mb: u64,
    pub max_network_bandwidth_mbps: Option<f64>,
    pub max_execution_time_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPolicy {
    pub allowed: bool,
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub allowed_ports: Vec<u16>,
    pub rate_limit_requests_per_second: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemPolicy {
    pub read_only_paths: Vec<String>,
    pub writable_paths: Vec<String>,
    pub forbidden_paths: Vec<String>,
    pub max_file_size_mb: u64,
    pub max_total_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPolicy {
    pub allow_eval: bool,
    pub allow_dynamic_imports: bool,
    pub allow_worker_threads: bool,
    pub allow_child_processes: bool,
    pub content_security_policy: Option<String>,
}

// Search and filtering types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSearchRequest {
    pub query: Option<String>,
    pub category: Option<ModuleCategory>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub price_range: Option<PriceRange>,
    pub rating_min: Option<f32>,
    pub compatibility: Option<String>,
    pub sort_by: Option<SortBy>,
    pub sort_order: Option<SortOrder>,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceRange {
    pub min: f64,
    pub max: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Name,
    Rating,
    Downloads,
    Updated,
    Price,
    Relevance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSearchResponse {
    pub modules: Vec<MarketplaceListing>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub total_pages: u32,
    pub facets: SearchFacets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub categories: HashMap<String, u64>,
    pub authors: HashMap<String, u64>,
    pub tags: HashMap<String, u64>,
    pub price_ranges: HashMap<String, u64>,
    pub ratings: HashMap<String, u64>,
}