use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use async_trait::async_trait;
use serde_json::Value;

use crate::{
    ModuleResult, ModuleError, ModuleMetadata, ModuleManifest, AdxModule,
    ModuleStatus, HealthStatus, ResourceUsage, ModuleEvent, ExtensionPoint, ExtensionContext,
};

/// ADX Module SDK - Provides utilities and abstractions for module development
pub struct ModuleSDK {
    pub logger: ModuleLogger,
    pub config: ModuleConfigManager,
    pub storage: ModuleStorage,
    pub http: ModuleHttpClient,
    pub events: ModuleEventBus,
    pub ui: ModuleUIBuilder,
    pub workflows: ModuleWorkflowBuilder,
    pub database: ModuleDatabaseBuilder,
}

impl ModuleSDK {
    pub fn new(module_id: String, tenant_id: String) -> Self {
        Self {
            logger: ModuleLogger::new(&module_id),
            config: ModuleConfigManager::new(&module_id, &tenant_id),
            storage: ModuleStorage::new(&module_id, &tenant_id),
            http: ModuleHttpClient::new(&module_id),
            events: ModuleEventBus::new(&module_id),
            ui: ModuleUIBuilder::new(&module_id),
            workflows: ModuleWorkflowBuilder::new(&module_id),
            database: ModuleDatabaseBuilder::new(&module_id, &tenant_id),
        }
    }
}

/// Base module implementation that developers can extend
pub struct BaseModule {
    metadata: ModuleMetadata,
    manifest: ModuleManifest,
    sdk: ModuleSDK,
    status: ModuleStatus,
    config: Value,
    extension_points: HashMap<String, Box<dyn ExtensionPoint>>,
}

impl BaseModule {
    pub fn new(metadata: ModuleMetadata, manifest: ModuleManifest) -> Self {
        let sdk = ModuleSDK::new(metadata.id.clone(), "default".to_string());
        
        Self {
            metadata,
            manifest,
            sdk,
            status: ModuleStatus::Uninitialized,
            config: Value::Null,
            extension_points: HashMap::new(),
        }
    }

    /// Register an extension point
    pub fn register_extension_point(&mut self, name: String, extension: Box<dyn ExtensionPoint>) {
        self.extension_points.insert(name, extension);
    }

    /// Get SDK reference for module development
    pub fn sdk(&self) -> &ModuleSDK {
        &self.sdk
    }

    /// Get mutable SDK reference
    pub fn sdk_mut(&mut self) -> &mut ModuleSDK {
        &mut self.sdk
    }
}

#[async_trait]
impl AdxModule for BaseModule {
    fn metadata(&self) -> &ModuleMetadata {
        &self.metadata
    }

    fn manifest(&self) -> &ModuleManifest {
        &self.manifest
    }

    async fn initialize(&mut self, config: Value) -> ModuleResult<()> {
        self.config = config;
        self.status = crate::traits::ModuleStatus::Initialized;
        self.sdk.logger.info("Module initialized");
        Ok(())
    }

    async fn start(&mut self) -> ModuleResult<()> {
        self.status = crate::traits::ModuleStatus::Starting;
        self.sdk.logger.info("Module starting");
        
        // Override in derived modules for custom start logic
        
        self.status = crate::traits::ModuleStatus::Running;
        self.sdk.logger.info("Module started");
        Ok(())
    }

    async fn stop(&mut self) -> ModuleResult<()> {
        self.status = crate::traits::ModuleStatus::Stopping;
        self.sdk.logger.info("Module stopping");
        
        // Override in derived modules for custom stop logic
        
        self.status = crate::traits::ModuleStatus::Stopped;
        self.sdk.logger.info("Module stopped");
        Ok(())
    }

    async fn shutdown(&mut self) -> ModuleResult<()> {
        self.sdk.logger.info("Module shutting down");
        self.status = crate::traits::ModuleStatus::Stopped;
        Ok(())
    }

    async fn configure(&mut self, config: Value) -> ModuleResult<()> {
        self.validate_config(&config)?;
        self.config = config;
        self.sdk.logger.info("Module reconfigured");
        Ok(())
    }

    async fn status(&self) -> ModuleResult<crate::traits::ModuleStatus> {
        Ok(self.status.clone())
    }

    async fn health(&self) -> ModuleResult<HealthStatus> {
        Ok(HealthStatus {
            is_healthy: matches!(self.status, crate::traits::ModuleStatus::Running),
            last_health_check: chrono::Utc::now(),
            error_count: 0,
            warning_count: 0,
            uptime_seconds: 0,
            response_time_ms: 0,
        })
    }

    async fn resource_usage(&self) -> ModuleResult<ResourceUsage> {
        Ok(ResourceUsage {
            memory_mb: 0,
            cpu_percent: 0.0,
            disk_mb: 0,
            network_in_mbps: 0.0,
            network_out_mbps: 0.0,
            active_connections: 0,
            last_measured: chrono::Utc::now(),
        })
    }

    async fn handle_event(&mut self, event: ModuleEvent) -> ModuleResult<()> {
        self.sdk.logger.debug(&format!("Received event: {:?}", event));
        // Override in derived modules for custom event handling
        Ok(())
    }

    async fn execute_command(&mut self, command: String, args: Vec<String>) -> ModuleResult<Value> {
        self.sdk.logger.info(&format!("Executing command: {} with args: {:?}", command, args));
        // Override in derived modules for custom commands
        Ok(Value::Null)
    }

    fn validate_config(&self, config: &Value) -> ModuleResult<()> {
        // Basic validation - override in derived modules
        if config.is_null() && !self.manifest.configuration.required_config.is_empty() {
            return Err(ModuleError::ValidationFailed("Required configuration missing".to_string()));
        }
        Ok(())
    }

    fn get_extension_points(&self) -> HashMap<String, Box<dyn ExtensionPoint>> {
        // This would need to be implemented differently due to ownership
        HashMap::new()
    }
}

/// Module logging utilities
pub struct ModuleLogger {
    module_id: String,
}

impl ModuleLogger {
    pub fn new(module_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
        }
    }

    pub fn debug(&self, message: &str) {
        tracing::debug!(module_id = %self.module_id, "{}", message);
    }

    pub fn info(&self, message: &str) {
        tracing::info!(module_id = %self.module_id, "{}", message);
    }

    pub fn warn(&self, message: &str) {
        tracing::warn!(module_id = %self.module_id, "{}", message);
    }

    pub fn error(&self, message: &str) {
        tracing::error!(module_id = %self.module_id, "{}", message);
    }
}

/// Module configuration management
pub struct ModuleConfigManager {
    module_id: String,
    tenant_id: String,
    config: HashMap<String, Value>,
}

impl ModuleConfigManager {
    pub fn new(module_id: &str, tenant_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
            tenant_id: tenant_id.to_string(),
            config: HashMap::new(),
        }
    }

    pub async fn get(&self, key: &str) -> ModuleResult<Option<Value>> {
        Ok(self.config.get(key).cloned())
    }

    pub async fn set(&mut self, key: String, value: Value) -> ModuleResult<()> {
        self.config.insert(key, value);
        // In a real implementation, this would persist to storage
        Ok(())
    }

    pub async fn get_typed<T: for<'de> Deserialize<'de>>(&self, key: &str) -> ModuleResult<Option<T>> {
        if let Some(value) = self.config.get(key) {
            let typed_value: T = serde_json::from_value(value.clone())?;
            Ok(Some(typed_value))
        } else {
            Ok(None)
        }
    }

    pub async fn set_typed<T: Serialize>(&mut self, key: String, value: T) -> ModuleResult<()> {
        let json_value = serde_json::to_value(value)?;
        self.set(key, json_value).await
    }
}

/// Module storage utilities
pub struct ModuleStorage {
    module_id: String,
    tenant_id: String,
}

impl ModuleStorage {
    pub fn new(module_id: &str, tenant_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
            tenant_id: tenant_id.to_string(),
        }
    }

    pub async fn store(&self, key: &str, data: &[u8]) -> ModuleResult<()> {
        // Store data in module-specific storage
        let storage_key = format!("modules/{}/{}/{}", self.tenant_id, self.module_id, key);
        // Implementation would use actual storage backend
        Ok(())
    }

    pub async fn retrieve(&self, key: &str) -> ModuleResult<Option<Vec<u8>>> {
        // Retrieve data from module-specific storage
        let storage_key = format!("modules/{}/{}/{}", self.tenant_id, self.module_id, key);
        // Implementation would use actual storage backend
        Ok(None)
    }

    pub async fn delete(&self, key: &str) -> ModuleResult<()> {
        // Delete data from module-specific storage
        let storage_key = format!("modules/{}/{}/{}", self.tenant_id, self.module_id, key);
        // Implementation would use actual storage backend
        Ok(())
    }

    pub async fn list_keys(&self, prefix: Option<&str>) -> ModuleResult<Vec<String>> {
        // List keys in module storage
        Ok(vec![])
    }
}

/// Module HTTP client with built-in security and rate limiting
pub struct ModuleHttpClient {
    module_id: String,
    client: reqwest::Client,
}

impl ModuleHttpClient {
    pub fn new(module_id: &str) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(format!("ADX-Module/{}", module_id))
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            module_id: module_id.to_string(),
            client,
        }
    }

    pub async fn get(&self, url: &str) -> ModuleResult<reqwest::Response> {
        let response = self.client.get(url).send().await?;
        Ok(response)
    }

    pub async fn post(&self, url: &str, body: Value) -> ModuleResult<reqwest::Response> {
        let response = self.client.post(url).json(&body).send().await?;
        Ok(response)
    }

    pub async fn put(&self, url: &str, body: Value) -> ModuleResult<reqwest::Response> {
        let response = self.client.put(url).json(&body).send().await?;
        Ok(response)
    }

    pub async fn delete(&self, url: &str) -> ModuleResult<reqwest::Response> {
        let response = self.client.delete(url).send().await?;
        Ok(response)
    }
}

/// Module event bus for inter-module communication
pub struct ModuleEventBus {
    module_id: String,
}

impl ModuleEventBus {
    pub fn new(module_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
        }
    }

    pub async fn emit(&self, event_type: &str, data: Value) -> ModuleResult<()> {
        // Emit event to the global event bus
        tracing::info!(
            module_id = %self.module_id,
            event_type = %event_type,
            "Emitting event"
        );
        Ok(())
    }

    pub async fn subscribe(&self, event_type: &str, handler: Box<dyn Fn(Value) -> ModuleResult<()>>) -> ModuleResult<()> {
        // Subscribe to events
        tracing::info!(
            module_id = %self.module_id,
            event_type = %event_type,
            "Subscribing to event"
        );
        Ok(())
    }
}

/// Module UI builder for creating frontend components
pub struct ModuleUIBuilder {
    module_id: String,
    components: Vec<UIComponent>,
}

impl ModuleUIBuilder {
    pub fn new(module_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
            components: Vec::new(),
        }
    }

    pub fn add_page(&mut self, route: &str, component: UIComponent) -> &mut Self {
        self.components.push(component);
        self
    }

    pub fn add_widget(&mut self, name: &str, component: UIComponent) -> &mut Self {
        self.components.push(component);
        self
    }

    pub fn add_modal(&mut self, name: &str, component: UIComponent) -> &mut Self {
        self.components.push(component);
        self
    }

    pub fn build(&self) -> ModuleResult<UIManifest> {
        Ok(UIManifest {
            module_id: self.module_id.clone(),
            components: self.components.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIComponent {
    pub name: String,
    pub component_type: UIComponentType,
    pub props: HashMap<String, Value>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UIComponentType {
    Page,
    Widget,
    Modal,
    Form,
    Chart,
    Table,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIManifest {
    pub module_id: String,
    pub components: Vec<UIComponent>,
}

/// Module workflow builder for creating Temporal workflows
pub struct ModuleWorkflowBuilder {
    module_id: String,
    workflows: Vec<WorkflowDefinition>,
    activities: Vec<ActivityDefinition>,
}

impl ModuleWorkflowBuilder {
    pub fn new(module_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
            workflows: Vec::new(),
            activities: Vec::new(),
        }
    }

    pub fn add_workflow(&mut self, name: &str, definition: WorkflowDefinition) -> &mut Self {
        self.workflows.push(definition);
        self
    }

    pub fn add_activity(&mut self, name: &str, definition: ActivityDefinition) -> &mut Self {
        self.activities.push(definition);
        self
    }

    pub fn build(&self) -> ModuleResult<WorkflowManifest> {
        Ok(WorkflowManifest {
            module_id: self.module_id.clone(),
            workflows: self.workflows.clone(),
            activities: self.activities.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub timeout_seconds: u64,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDefinition {
    pub name: String,
    pub description: String,
    pub input_schema: Value,
    pub output_schema: Value,
    pub timeout_seconds: u64,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_interval_seconds: u64,
    pub backoff_coefficient: f64,
    pub max_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowManifest {
    pub module_id: String,
    pub workflows: Vec<WorkflowDefinition>,
    pub activities: Vec<ActivityDefinition>,
}

/// Module database builder for creating database schemas
pub struct ModuleDatabaseBuilder {
    module_id: String,
    tenant_id: String,
    tables: Vec<TableDefinition>,
    views: Vec<ViewDefinition>,
    functions: Vec<FunctionDefinition>,
}

impl ModuleDatabaseBuilder {
    pub fn new(module_id: &str, tenant_id: &str) -> Self {
        Self {
            module_id: module_id.to_string(),
            tenant_id: tenant_id.to_string(),
            tables: Vec::new(),
            views: Vec::new(),
            functions: Vec::new(),
        }
    }

    pub fn add_table(&mut self, definition: TableDefinition) -> &mut Self {
        self.tables.push(definition);
        self
    }

    pub fn add_view(&mut self, definition: ViewDefinition) -> &mut Self {
        self.views.push(definition);
        self
    }

    pub fn add_function(&mut self, definition: FunctionDefinition) -> &mut Self {
        self.functions.push(definition);
        self
    }

    pub fn build(&self) -> ModuleResult<DatabaseManifest> {
        Ok(DatabaseManifest {
            module_id: self.module_id.clone(),
            tenant_id: self.tenant_id.clone(),
            tables: self.tables.clone(),
            views: self.views.clone(),
            functions: self.functions.clone(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableDefinition {
    pub name: String,
    pub columns: Vec<ColumnDefinition>,
    pub indexes: Vec<IndexDefinition>,
    pub constraints: Vec<ConstraintDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnDefinition {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDefinition {
    pub name: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintDefinition {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub columns: Vec<String>,
    pub reference_table: Option<String>,
    pub reference_columns: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    PrimaryKey,
    ForeignKey,
    Unique,
    Check(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewDefinition {
    pub name: String,
    pub query: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    pub name: String,
    pub parameters: Vec<ParameterDefinition>,
    pub return_type: String,
    pub body: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub data_type: String,
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseManifest {
    pub module_id: String,
    pub tenant_id: String,
    pub tables: Vec<TableDefinition>,
    pub views: Vec<ViewDefinition>,
    pub functions: Vec<FunctionDefinition>,
}

/// Module development macros for common patterns
#[macro_export]
macro_rules! adx_module {
    ($name:ident, $metadata:expr, $manifest:expr) => {
        pub struct $name {
            base: BaseModule,
        }

        impl $name {
            pub fn new() -> Self {
                Self {
                    base: BaseModule::new($metadata, $manifest),
                }
            }
        }

        #[async_trait::async_trait]
        impl AdxModule for $name {
            fn metadata(&self) -> &ModuleMetadata {
                self.base.metadata()
            }

            fn manifest(&self) -> &ModuleManifest {
                self.base.manifest()
            }

            async fn initialize(&mut self, config: serde_json::Value) -> ModuleResult<()> {
                self.base.initialize(config).await
            }

            async fn start(&mut self) -> ModuleResult<()> {
                self.base.start().await
            }

            async fn stop(&mut self) -> ModuleResult<()> {
                self.base.stop().await
            }

            async fn shutdown(&mut self) -> ModuleResult<()> {
                self.base.shutdown().await
            }

            async fn configure(&mut self, config: serde_json::Value) -> ModuleResult<()> {
                self.base.configure(config).await
            }

            async fn status(&self) -> ModuleResult<crate::traits::ModuleStatus> {
                self.base.status().await
            }

            async fn health(&self) -> ModuleResult<HealthStatus> {
                self.base.health().await
            }

            async fn resource_usage(&self) -> ModuleResult<ResourceUsage> {
                self.base.resource_usage().await
            }

            async fn handle_event(&mut self, event: ModuleEvent) -> ModuleResult<()> {
                self.base.handle_event(event).await
            }

            async fn execute_command(&mut self, command: String, args: Vec<String>) -> ModuleResult<serde_json::Value> {
                self.base.execute_command(command, args).await
            }

            fn validate_config(&self, config: &serde_json::Value) -> ModuleResult<()> {
                self.base.validate_config(config)
            }

            fn get_extension_points(&self) -> std::collections::HashMap<String, Box<dyn ExtensionPoint>> {
                self.base.get_extension_points()
            }
        }
    };
}

/// Example module using the SDK
pub mod example {
    use super::*;
    use crate::{ModuleMetadata, ModuleManifest, ModuleAuthor, ModuleCategory, VersionRequirement};
    use semver::Version;
    use chrono::Utc;

    // Example module implementation
    pub struct ExampleModule {
        base: BaseModule,
    }

    impl ExampleModule {
        pub fn new() -> Self {
            let metadata = ModuleMetadata {
                id: "example-module".to_string(),
                name: "Example Module".to_string(),
                version: Version::new(1, 0, 0),
                description: "An example module demonstrating the SDK".to_string(),
                long_description: Some("This is a comprehensive example module".to_string()),
                author: ModuleAuthor {
                    name: "ADX Core Team".to_string(),
                    email: Some("team@adxcore.com".to_string()),
                    website: Some("https://adxcore.com".to_string()),
                    organization: Some("ADX Core".to_string()),
                },
                license: "MIT".to_string(),
                homepage: Some("https://adxcore.com/modules/example".to_string()),
                repository: Some("https://github.com/adxcore/example-module".to_string()),
                documentation: Some("https://docs.adxcore.com/modules/example".to_string()),
                keywords: vec!["example".to_string(), "demo".to_string()],
                categories: vec![ModuleCategory::Development],
                adx_core_version: VersionRequirement {
                    min_version: Version::new(2, 0, 0),
                    max_version: None,
                    compatible_versions: vec![],
                },
                created_at: Utc::now(),
                updated_at: Utc::now(),
            };

            let manifest = ModuleManifest {
                metadata: metadata.clone(),
                dependencies: vec![],
                capabilities: crate::ModuleCapabilities {
                    ui_extensions: vec![],
                    api_extensions: vec![],
                    workflow_extensions: vec![],
                    database_extensions: vec![],
                    event_handlers: vec![],
                    cross_platform_features: crate::CrossPlatformFeatures {
                        web_support: true,
                        desktop_support: vec![],
                        mobile_support: vec![],
                        native_integrations: vec![],
                    },
                },
                permissions: vec![],
                resources: crate::ResourceRequirements {
                    min_memory_mb: 64,
                    max_memory_mb: 256,
                    min_cpu_cores: 0.1,
                    max_cpu_cores: 1.0,
                    storage_mb: 100,
                    network_bandwidth_mbps: None,
                    concurrent_operations: 10,
                },
                configuration: crate::ModuleConfiguration {
                    config_schema: serde_json::json!({}),
                    default_config: serde_json::json!({}),
                    required_config: vec![],
                    tenant_configurable: vec![],
                    user_configurable: vec![],
                },
                extension_points: crate::ExtensionPoints {
                    backend_entry: Some("./lib/backend.js".to_string()),
                    frontend_entry: Some("./lib/frontend.js".to_string()),
                    workflow_entry: Some("./lib/workflows.js".to_string()),
                    migration_entry: Some("./lib/migrations.js".to_string()),
                    test_entry: Some("./lib/tests.js".to_string()),
                },
                sandbox_config: crate::SandboxConfiguration {
                    isolation_level: crate::IsolationLevel::Process,
                    allowed_syscalls: vec![],
                    blocked_syscalls: vec![],
                    network_restrictions: crate::NetworkRestrictions {
                        allowed_domains: vec![],
                        blocked_domains: vec![],
                        allowed_ports: vec![],
                        blocked_ports: vec![],
                        max_connections: 10,
                    },
                    file_system_restrictions: crate::FileSystemRestrictions {
                        allowed_paths: vec!["/tmp".to_string()],
                        blocked_paths: vec![],
                        read_only_paths: vec![],
                        max_file_size: 10 * 1024 * 1024,
                        max_files: 100,
                    },
                    resource_limits: crate::ResourceLimits {
                        max_memory_mb: 256,
                        max_cpu_percent: 50.0,
                        max_execution_time_seconds: 300,
                        max_disk_io_mbps: 100,
                        max_network_io_mbps: 50,
                    },
                },
            };

            Self {
                base: BaseModule::new(metadata, manifest),
            }
        }
    }

    #[async_trait]
    impl AdxModule for ExampleModule {
        fn metadata(&self) -> &ModuleMetadata {
            self.base.metadata()
        }

        fn manifest(&self) -> &ModuleManifest {
            self.base.manifest()
        }

        async fn initialize(&mut self, config: Value) -> ModuleResult<()> {
            self.base.sdk().logger.info("Initializing example module");
            self.base.initialize(config).await
        }

        async fn start(&mut self) -> ModuleResult<()> {
            self.base.sdk().logger.info("Starting example module");
            
            // Custom start logic
            self.base.sdk().events.emit("module:started", serde_json::json!({
                "module_id": self.metadata().id,
                "timestamp": chrono::Utc::now()
            })).await?;
            
            self.base.start().await
        }

        async fn stop(&mut self) -> ModuleResult<()> {
            self.base.sdk().logger.info("Stopping example module");
            self.base.stop().await
        }

        async fn shutdown(&mut self) -> ModuleResult<()> {
            self.base.sdk().logger.info("Shutting down example module");
            self.base.shutdown().await
        }

        async fn configure(&mut self, config: Value) -> ModuleResult<()> {
            self.base.configure(config).await
        }

        async fn status(&self) -> ModuleResult<crate::traits::ModuleStatus> {
            self.base.status().await
        }

        async fn health(&self) -> ModuleResult<HealthStatus> {
            self.base.health().await
        }

        async fn resource_usage(&self) -> ModuleResult<ResourceUsage> {
            self.base.resource_usage().await
        }

        async fn handle_event(&mut self, event: ModuleEvent) -> ModuleResult<()> {
            self.base.sdk().logger.debug(&format!("Handling event: {:?}", event));
            self.base.handle_event(event).await
        }

        async fn execute_command(&mut self, command: String, args: Vec<String>) -> ModuleResult<Value> {
            match command.as_str() {
                "hello" => {
                    let name = args.first().unwrap_or(&"World".to_string()).clone();
                    Ok(serde_json::json!({
                        "message": format!("Hello, {}!", name)
                    }))
                }
                _ => self.base.execute_command(command, args).await
            }
        }

        fn validate_config(&self, config: &Value) -> ModuleResult<()> {
            self.base.validate_config(config)
        }

        fn get_extension_points(&self) -> HashMap<String, Box<dyn ExtensionPoint>> {
            self.base.get_extension_points()
        }
    }
}

// Error conversion for reqwest
impl From<reqwest::Error> for ModuleError {
    fn from(err: reqwest::Error) -> Self {
        ModuleError::NetworkError(err.to_string())
    }
}