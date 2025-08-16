use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::{
    ModuleResult, ModuleMetadata, ModuleManifest, ModuleInstance, ModulePackage,
    ModuleSearchQuery, ModuleSearchResult, InstallModuleRequest, InstallModuleResult,
    UpdateModuleRequest, UpdateModuleResult, UninstallModuleRequest, UninstallModuleResult,
    ResourceUsage, HealthStatus,
};

/// Core trait that all ADX modules must implement
#[async_trait]
pub trait AdxModule: Send + Sync {
    /// Get module metadata
    fn metadata(&self) -> &ModuleMetadata;
    
    /// Get module manifest with capabilities and requirements
    fn manifest(&self) -> &ModuleManifest;
    
    /// Initialize the module with configuration
    async fn initialize(&mut self, config: Value) -> ModuleResult<()>;
    
    /// Start the module (activate)
    async fn start(&mut self) -> ModuleResult<()>;
    
    /// Stop the module (deactivate)
    async fn stop(&mut self) -> ModuleResult<()>;
    
    /// Shutdown the module (cleanup resources)
    async fn shutdown(&mut self) -> ModuleResult<()>;
    
    /// Update module configuration
    async fn configure(&mut self, config: Value) -> ModuleResult<()>;
    
    /// Get current module status
    async fn status(&self) -> ModuleResult<ModuleStatus>;
    
    /// Get module health information
    async fn health(&self) -> ModuleResult<HealthStatus>;
    
    /// Get current resource usage
    async fn resource_usage(&self) -> ModuleResult<ResourceUsage>;
    
    /// Handle module events
    async fn handle_event(&mut self, event: ModuleEvent) -> ModuleResult<()>;
    
    /// Execute module-specific command
    async fn execute_command(&mut self, command: String, args: Vec<String>) -> ModuleResult<Value>;
    
    /// Validate module configuration
    fn validate_config(&self, config: &Value) -> ModuleResult<()>;
    
    /// Get module's extension points
    fn get_extension_points(&self) -> HashMap<String, Box<dyn ExtensionPoint>>;
}

/// Module status enumeration
#[derive(Debug, Clone, PartialEq)]
pub enum ModuleStatus {
    Uninitialized,
    Initializing,
    Initialized,
    Starting,
    Running,
    Stopping,
    Stopped,
    Error(String),
}

/// Module event types
#[derive(Debug, Clone)]
pub enum ModuleEvent {
    TenantSwitched { old_tenant: String, new_tenant: String },
    UserLoggedIn { user_id: String, tenant_id: String },
    UserLoggedOut { user_id: String, tenant_id: String },
    ConfigurationChanged { key: String, old_value: Value, new_value: Value },
    ResourceLimitWarning { resource: String, usage: f64, limit: f64 },
    HealthCheckFailed { reason: String },
    Custom { event_type: String, data: Value },
}

/// Extension point trait for module extensibility
pub trait ExtensionPoint: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, context: &ExtensionContext, args: Value) -> ModuleResult<Value>;
}

/// Context provided to extension points
#[derive(Debug, Clone)]
pub struct ExtensionContext {
    pub tenant_id: String,
    pub user_id: Option<String>,
    pub module_id: String,
    pub request_id: String,
    pub metadata: HashMap<String, Value>,
}

/// Module loader trait for different module types
#[async_trait]
pub trait ModuleLoader: Send + Sync {
    /// Load a module from a package
    async fn load_module(&self, package: &ModulePackage) -> ModuleResult<Box<dyn AdxModule>>;
    
    /// Unload a module
    async fn unload_module(&self, module_id: &str) -> ModuleResult<()>;
    
    /// Check if loader supports the module type
    fn supports_module(&self, manifest: &ModuleManifest) -> bool;
    
    /// Get loader name
    fn name(&self) -> &str;
}

/// Module repository trait for storage operations
#[async_trait]
pub trait ModuleRepository: Send + Sync {
    /// Save module metadata
    async fn save_metadata(&self, metadata: &ModuleMetadata) -> ModuleResult<()>;
    
    /// Get module metadata by ID
    async fn get_metadata(&self, module_id: &str) -> ModuleResult<Option<ModuleMetadata>>;
    
    /// List all modules
    async fn list_modules(&self) -> ModuleResult<Vec<ModuleMetadata>>;
    
    /// Search modules
    async fn search_modules(&self, query: &ModuleSearchQuery) -> ModuleResult<ModuleSearchResult>;
    
    /// Save module instance
    async fn save_instance(&self, instance: &ModuleInstance) -> ModuleResult<()>;
    
    /// Get module instance
    async fn get_instance(&self, instance_id: Uuid) -> ModuleResult<Option<ModuleInstance>>;
    
    /// List instances for tenant
    async fn list_tenant_instances(&self, tenant_id: &str) -> ModuleResult<Vec<ModuleInstance>>;
    
    /// Update instance status
    async fn update_instance_status(&self, instance_id: Uuid, status: ModuleStatus) -> ModuleResult<()>;
    
    /// Delete instance
    async fn delete_instance(&self, instance_id: Uuid) -> ModuleResult<()>;
}

/// Module marketplace trait
#[async_trait]
pub trait ModuleMarketplace: Send + Sync {
    /// Search marketplace modules
    async fn search(&self, query: &ModuleSearchQuery) -> ModuleResult<ModuleSearchResult>;
    
    /// Get module details
    async fn get_module(&self, module_id: &str) -> ModuleResult<Option<ModuleMetadata>>;
    
    /// Download module package
    async fn download(&self, module_id: &str, version: &str) -> ModuleResult<ModulePackage>;
    
    /// Get module reviews and ratings
    async fn get_reviews(&self, module_id: &str) -> ModuleResult<Vec<ModuleReview>>;
    
    /// Submit module review
    async fn submit_review(&self, review: &ModuleReview) -> ModuleResult<()>;
    
    /// Get featured modules
    async fn get_featured(&self) -> ModuleResult<Vec<ModuleMetadata>>;
    
    /// Get trending modules
    async fn get_trending(&self) -> ModuleResult<Vec<ModuleMetadata>>;
    
    /// Process module purchase
    async fn purchase_module(&self, purchase: &ModulePurchase) -> ModuleResult<PurchaseResult>;
}

/// Module review structure
#[derive(Debug, Clone)]
pub struct ModuleReview {
    pub module_id: String,
    pub user_id: String,
    pub rating: u8, // 1-5 stars
    pub title: String,
    pub content: String,
    pub version: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Module purchase structure
#[derive(Debug, Clone)]
pub struct ModulePurchase {
    pub module_id: String,
    pub version: String,
    pub tenant_id: String,
    pub user_id: String,
    pub payment_method: PaymentMethod,
    pub billing_address: BillingAddress,
}

/// Payment method enumeration
#[derive(Debug, Clone)]
pub enum PaymentMethod {
    CreditCard { token: String },
    PayPal { account_id: String },
    BankTransfer { account_number: String },
    Enterprise { contract_id: String },
}

/// Billing address structure
#[derive(Debug, Clone)]
pub struct BillingAddress {
    pub name: String,
    pub company: Option<String>,
    pub address_line1: String,
    pub address_line2: Option<String>,
    pub city: String,
    pub state: String,
    pub postal_code: String,
    pub country: String,
}

/// Purchase result
#[derive(Debug, Clone)]
pub struct PurchaseResult {
    pub transaction_id: String,
    pub status: PurchaseStatus,
    pub license_key: Option<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Purchase status enumeration
#[derive(Debug, Clone)]
pub enum PurchaseStatus {
    Pending,
    Completed,
    Failed,
    Refunded,
}

/// Module sandbox trait
#[async_trait]
pub trait ModuleSandbox: Send + Sync {
    /// Create sandbox for module
    async fn create_sandbox(&self, instance_id: Uuid) -> ModuleResult<SandboxHandle>;
    
    /// Execute code in sandbox
    async fn execute_in_sandbox(
        &self,
        handle: &SandboxHandle,
        code: &str,
        args: Vec<String>,
    ) -> ModuleResult<SandboxResult>;
    
    /// Monitor sandbox resource usage
    async fn monitor_resources(&self, handle: &SandboxHandle) -> ModuleResult<ResourceUsage>;
    
    /// Destroy sandbox
    async fn destroy_sandbox(&self, handle: SandboxHandle) -> ModuleResult<()>;
    
    /// Check sandbox health
    async fn check_health(&self, handle: &SandboxHandle) -> ModuleResult<bool>;
}

/// Sandbox handle
#[derive(Debug, Clone)]
pub struct SandboxHandle {
    pub id: String,
    pub instance_id: Uuid,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// Sandbox execution result
#[derive(Debug, Clone)]
pub struct SandboxResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
    pub execution_time_ms: u64,
    pub memory_used_mb: u64,
}

/// Module security scanner trait
#[async_trait]
pub trait ModuleSecurityScanner: Send + Sync {
    /// Scan module package for security issues
    async fn scan_package(&self, package: &ModulePackage) -> ModuleResult<SecurityScanResult>;
    
    /// Scan module runtime for vulnerabilities
    async fn scan_runtime(&self, instance_id: Uuid) -> ModuleResult<SecurityScanResult>;
    
    /// Get security policy for module
    async fn get_security_policy(&self, module_id: &str) -> ModuleResult<SecurityPolicy>;
    
    /// Update security policy
    async fn update_security_policy(&self, policy: &SecurityPolicy) -> ModuleResult<()>;
}

/// Security scan result
#[derive(Debug, Clone)]
pub struct SecurityScanResult {
    pub scan_id: String,
    pub module_id: String,
    pub scan_type: ScanType,
    pub status: ScanStatus,
    pub issues: Vec<SecurityIssue>,
    pub score: u8, // 0-100
    pub scanned_at: chrono::DateTime<chrono::Utc>,
}

/// Security scan type
#[derive(Debug, Clone)]
pub enum ScanType {
    Static,
    Dynamic,
    Dependency,
    Configuration,
    Runtime,
}

/// Security scan status
#[derive(Debug, Clone)]
pub enum ScanStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

/// Security issue
#[derive(Debug, Clone)]
pub struct SecurityIssue {
    pub id: String,
    pub severity: Severity,
    pub category: IssueCategory,
    pub title: String,
    pub description: String,
    pub recommendation: String,
    pub cve_id: Option<String>,
    pub affected_files: Vec<String>,
}

/// Issue severity
#[derive(Debug, Clone)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

/// Issue category
#[derive(Debug, Clone)]
pub enum IssueCategory {
    Vulnerability,
    MaliciousCode,
    DataExfiltration,
    PrivilegeEscalation,
    ResourceAbuse,
    ConfigurationIssue,
    DependencyIssue,
}

/// Security policy
#[derive(Debug, Clone)]
pub struct SecurityPolicy {
    pub module_id: String,
    pub allowed_permissions: Vec<String>,
    pub blocked_permissions: Vec<String>,
    pub resource_limits: ResourceLimits,
    pub network_policy: NetworkPolicy,
    pub file_system_policy: FileSystemPolicy,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

/// Resource limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_mb: u64,
    pub max_cpu_percent: f32,
    pub max_disk_io_mbps: u64,
    pub max_network_io_mbps: u64,
    pub max_execution_time_seconds: u64,
}

/// Network policy
#[derive(Debug, Clone)]
pub struct NetworkPolicy {
    pub allowed_hosts: Vec<String>,
    pub blocked_hosts: Vec<String>,
    pub allowed_ports: Vec<u16>,
    pub blocked_ports: Vec<u16>,
    pub max_connections: u32,
}

/// File system policy
#[derive(Debug, Clone)]
pub struct FileSystemPolicy {
    pub allowed_paths: Vec<String>,
    pub blocked_paths: Vec<String>,
    pub read_only_paths: Vec<String>,
    pub max_file_size_mb: u64,
    pub max_total_size_mb: u64,
}