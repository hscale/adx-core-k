use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use semver::Version;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleMetadata {
    pub id: String,
    pub name: String,
    pub version: Version,
    pub description: String,
    pub long_description: Option<String>,
    pub author: ModuleAuthor,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
    pub documentation: Option<String>,
    pub keywords: Vec<String>,
    pub categories: Vec<ModuleCategory>,
    pub adx_core_version: VersionRequirement,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAuthor {
    pub name: String,
    pub email: Option<String>,
    pub website: Option<String>,
    pub organization: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleCategory {
    BusinessManagement,
    Analytics,
    Integration,
    Workflow,
    Security,
    Communication,
    FileManagement,
    UserInterface,
    Development,
    Utility,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionRequirement {
    pub min_version: Version,
    pub max_version: Option<Version>,
    pub compatible_versions: Vec<Version>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleManifest {
    pub metadata: ModuleMetadata,
    pub dependencies: Vec<ModuleDependency>,
    pub capabilities: ModuleCapabilities,
    pub permissions: Vec<ModulePermission>,
    pub resources: ResourceRequirements,
    pub configuration: ModuleConfiguration,
    pub extension_points: ExtensionPoints,
    pub sandbox_config: SandboxConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependency {
    pub module_id: String,
    pub version_requirement: String,
    pub optional: bool,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCapabilities {
    pub ui_extensions: Vec<UiExtensionPoint>,
    pub api_extensions: Vec<ApiExtensionPoint>,
    pub workflow_extensions: Vec<WorkflowExtensionPoint>,
    pub database_extensions: Vec<DatabaseExtensionPoint>,
    pub event_handlers: Vec<EventHandler>,
    pub cross_platform_features: CrossPlatformFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UiExtensionPoint {
    Dashboard(String),
    Navigation(String),
    Settings(String),
    Modal(String),
    Widget(String),
    Page(String),
    Component(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ApiExtensionPoint {
    RestEndpoints(Vec<String>),
    GraphQLTypes(Vec<String>),
    WebhookHandlers(Vec<String>),
    Middleware(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowExtensionPoint {
    Activities(Vec<String>),
    Workflows(Vec<String>),
    Signals(Vec<String>),
    Queries(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseExtensionPoint {
    Tables(Vec<String>),
    Views(Vec<String>),
    Functions(Vec<String>),
    Triggers(Vec<String>),
    Migrations(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventHandler {
    pub event_type: String,
    pub handler_name: String,
    pub priority: i32,
    pub async_processing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformFeatures {
    pub web_support: bool,
    pub desktop_support: Vec<DesktopPlatform>,
    pub mobile_support: Vec<MobilePlatform>,
    pub native_integrations: Vec<NativeIntegration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DesktopPlatform {
    Windows,
    MacOS,
    Linux,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MobilePlatform {
    iOS,
    Android,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NativeIntegration {
    FileSystem,
    Notifications,
    Camera,
    GPS,
    Bluetooth,
    Calendar,
    Contacts,
    SystemTray,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModulePermission {
    DatabaseRead(String),
    DatabaseWrite(String),
    FileRead(String),
    FileWrite(String),
    NetworkAccess(String),
    SystemAccess(String),
    UserDataAccess,
    TenantDataAccess,
    WorkflowExecution(String),
    ApiAccess(String),
    ModuleManagement,
    AdminAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub min_memory_mb: u64,
    pub max_memory_mb: u64,
    pub min_cpu_cores: f32,
    pub max_cpu_cores: f32,
    pub storage_mb: u64,
    pub network_bandwidth_mbps: Option<u64>,
    pub concurrent_operations: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleConfiguration {
    pub config_schema: serde_json::Value,
    pub default_config: serde_json::Value,
    pub required_config: Vec<String>,
    pub tenant_configurable: Vec<String>,
    pub user_configurable: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionPoints {
    pub backend_entry: Option<String>,
    pub frontend_entry: Option<String>,
    pub workflow_entry: Option<String>,
    pub migration_entry: Option<String>,
    pub test_entry: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfiguration {
    pub isolation_level: IsolationLevel,
    pub allowed_syscalls: Vec<String>,
    pub blocked_syscalls: Vec<String>,
    pub network_restrictions: NetworkRestrictions,
    pub file_system_restrictions: FileSystemRestrictions,
    pub resource_limits: ResourceLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    None,
    Process,
    Container,
    Wasm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkRestrictions {
    pub allowed_domains: Vec<String>,
    pub blocked_domains: Vec<String>,
    pub allowed_ports: Vec<u16>,
    pub blocked_ports: Vec<u16>,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileSystemRestrictions {
    pub allowed_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
    pub read_only_paths: Vec<String>,
    pub max_file_size: u64,
    pub max_files: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub max_execution_time_seconds: u64,
    pub max_disk_io_mbps: u64,
    pub max_network_io_mbps: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleInstance {
    pub id: Uuid,
    pub module_id: String,
    pub tenant_id: String,
    pub version: Version,
    pub status: ModuleStatus,
    pub configuration: serde_json::Value,
    pub installation_path: String,
    pub installed_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub last_updated: DateTime<Utc>,
    pub resource_usage: ResourceUsage,
    pub health_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleStatus {
    Downloaded,
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
pub struct ResourceUsage {
    pub memory_mb: u64,
    pub cpu_percent: f32,
    pub disk_mb: u64,
    pub network_in_mbps: f32,
    pub network_out_mbps: f32,
    pub active_connections: u32,
    pub last_measured: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    pub is_healthy: bool,
    pub last_health_check: DateTime<Utc>,
    pub error_count: u32,
    pub warning_count: u32,
    pub uptime_seconds: u64,
    pub response_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModulePackage {
    pub metadata: ModuleMetadata,
    pub manifest: ModuleManifest,
    pub content: Vec<u8>,
    pub checksum: String,
    pub signature: Option<String>,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleRegistry {
    pub modules: HashMap<String, Vec<ModuleMetadata>>,
    pub categories: Vec<ModuleCategory>,
    pub featured_modules: Vec<String>,
    pub trending_modules: Vec<String>,
    pub last_updated: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSearchQuery {
    pub query: Option<String>,
    pub categories: Vec<ModuleCategory>,
    pub author: Option<String>,
    pub min_version: Option<Version>,
    pub max_version: Option<Version>,
    pub keywords: Vec<String>,
    pub sort_by: SortBy,
    pub limit: u32,
    pub offset: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortBy {
    Relevance,
    Name,
    Version,
    Downloads,
    Rating,
    UpdatedAt,
    CreatedAt,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSearchResult {
    pub modules: Vec<ModuleMetadata>,
    pub total_count: u64,
    pub has_more: bool,
    pub facets: SearchFacets,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchFacets {
    pub categories: HashMap<ModuleCategory, u64>,
    pub authors: HashMap<String, u64>,
    pub versions: HashMap<String, u64>,
}

// Workflow-related models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallModuleRequest {
    pub module_id: String,
    pub version: Option<Version>,
    pub tenant_id: String,
    pub user_id: String,
    pub configuration: Option<serde_json::Value>,
    pub auto_activate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallModuleResult {
    pub instance_id: Uuid,
    pub module_id: String,
    pub version: Version,
    pub status: ModuleStatus,
    pub installation_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleRequest {
    pub instance_id: Uuid,
    pub target_version: Option<Version>,
    pub preserve_config: bool,
    pub backup_current: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateModuleResult {
    pub instance_id: Uuid,
    pub old_version: Version,
    pub new_version: Version,
    pub backup_id: Option<String>,
    pub status: ModuleStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallModuleRequest {
    pub instance_id: Uuid,
    pub cleanup_data: bool,
    pub backup_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UninstallModuleResult {
    pub instance_id: Uuid,
    pub backup_id: Option<String>,
    pub cleanup_summary: CleanupSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupSummary {
    pub files_removed: u32,
    pub database_objects_removed: u32,
    pub configuration_removed: bool,
    pub data_backed_up: bool,
}