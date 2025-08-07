# ADX CORE - Plugin System Architecture

## Overview

ADX CORE implements a WordPress-style plugin architecture that provides powerful extensibility while maintaining security, performance, and reliability. The plugin system allows third-party developers to extend platform functionality without modifying the core codebase.

## Plugin Architecture Principles

### Core Principles
1. **WordPress-Style Familiarity**: Hooks, filters, and actions pattern
2. **Hot-Loading**: Install and activate plugins without system restart
3. **Sandboxing**: Secure isolation with resource limits
4. **Dependency Management**: Automatic dependency resolution
5. **Version Compatibility**: Backward and forward compatibility
6. **Marketplace Integration**: Centralized plugin distribution
7. **Revenue Sharing**: Monetization for plugin developers

### Plugin System Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                    Plugin Marketplace                           │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │   Plugin    │ │  Discovery  │ │  Reviews &  │ │  Payment    │ │
│  │  Registry   │ │ & Search    │ │   Ratings   │ │ Processing  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                    Plugin Manager                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │Installation │ │   Version   │ │Dependency   │ │   License   │ │
│  │& Activation │ │ Management  │ │ Resolution  │ │ Validation  │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                   Plugin Runtime                                │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │   Plugin    │ │    Hook     │ │  Resource   │ │   Security  │ │
│  │  Sandbox    │ │   System    │ │   Limits    │ │  Controls   │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────────┐
│                  Extension Points                               │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ │
│  │     UI      │ │     API     │ │  Workflow   │ │  Database   │ │
│  │ Components  │ │ Endpoints   │ │   Steps     │ │   Schema    │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

## Plugin Development Framework

### Plugin Trait Definition
```rust
// Core plugin trait that all plugins must implement
#[async_trait]
pub trait AdxPlugin: Send + Sync {
    // Plugin metadata
    fn metadata(&self) -> PluginMetadata;
    
    // Lifecycle hooks
    async fn activate(&self, context: &PluginContext) -> Result<(), PluginError>;
    async fn deactivate(&self) -> Result<(), PluginError>;
    async fn uninstall(&self) -> Result<(), PluginError>;
    
    // Extension points
    async fn register_ui_components(&self, registry: &mut UIComponentRegistry) -> Result<(), PluginError>;
    async fn register_api_endpoints(&self, registry: &mut APIEndpointRegistry) -> Result<(), PluginError>;
    async fn register_workflow_steps(&self, registry: &mut WorkflowStepRegistry) -> Result<(), PluginError>;
    async fn register_database_migrations(&self, registry: &mut MigrationRegistry) -> Result<(), PluginError>;
    
    // Hook system
    async fn register_hooks(&self, hook_registry: &mut HookRegistry) -> Result<(), PluginError>;
    
    // Configuration
    async fn get_config_schema(&self) -> Result<ConfigSchema, PluginError>;
    async fn validate_config(&self, config: &Value) -> Result<(), PluginError>;
    async fn update_config(&self, config: Value) -> Result<(), PluginError>;
}

// Plugin metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub website: Option<String>,
    pub license: String,
    pub dependencies: Vec<PluginDependency>,
    pub min_core_version: String,
    pub max_core_version: Option<String>,
    pub permissions: Vec<Permission>,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginDependency {
    pub plugin_id: String,
    pub version_requirement: String,
    pub optional: bool,
}
```

### Plugin Context and Services
```rust
// Plugin context provides access to core services
pub struct PluginContext {
    pub tenant_id: TenantId,
    pub plugin_id: String,
    pub config: Value,
    pub database: Arc<dyn PluginDatabase>,
    pub file_storage: Arc<dyn FileStorage>,
    pub http_client: Arc<HttpClient>,
    pub event_bus: Arc<EventBus>,
    pub logger: Arc<Logger>,
    pub metrics: Arc<MetricsCollector>,
}

impl PluginContext {
    // Register UI components
    pub async fn register_ui_component(
        &self,
        component_id: &str,
        component: Box<dyn UIComponent>,
    ) -> Result<(), PluginError> {
        self.ui_registry.register_component(
            &self.plugin_id,
            component_id,
            component,
        ).await
    }
    
    // Register API endpoints
    pub async fn register_endpoint(
        &self,
        path: &str,
        handler: Box<dyn APIHandler>,
    ) -> Result<(), PluginError> {
        self.api_registry.register_endpoint(
            &self.plugin_id,
            path,
            handler,
        ).await
    }
    
    // Register workflow steps
    pub async fn register_workflow_step(
        &self,
        step_id: &str,
        step: Box<dyn WorkflowStep>,
    ) -> Result<(), PluginError> {
        self.workflow_registry.register_step(
            &self.plugin_id,
            step_id,
            step,
        ).await
    }
    
    // Add action hooks (WordPress-style)
    pub async fn add_action<F>(&self, hook_name: &str, callback: F) -> Result<(), PluginError>
    where
        F: Fn(&ActionData) -> BoxFuture<'static, Result<(), PluginError>> + Send + Sync + 'static,
    {
        self.hook_registry.add_action(&self.plugin_id, hook_name, Box::new(callback)).await
    }
    
    // Add filter hooks (WordPress-style)
    pub async fn add_filter<F>(&self, hook_name: &str, callback: F) -> Result<(), PluginError>
    where
        F: Fn(Value) -> BoxFuture<'static, Result<Value, PluginError>> + Send + Sync + 'static,
    {
        self.hook_registry.add_filter(&self.plugin_id, hook_name, Box::new(callback)).await
    }
}
```

### Hook System Implementation
```rust
// WordPress-style hook system
pub struct HookRegistry {
    actions: Arc<RwLock<HashMap<String, Vec<ActionHook>>>>,
    filters: Arc<RwLock<HashMap<String, Vec<FilterHook>>>>>,
}

pub struct ActionHook {
    pub plugin_id: String,
    pub priority: i32,
    pub callback: Box<dyn Fn(&ActionData) -> BoxFuture<'static, Result<(), PluginError>> + Send + Sync>,
}

pub struct FilterHook {
    pub plugin_id: String,
    pub priority: i32,
    pub callback: Box<dyn Fn(Value) -> BoxFuture<'static, Result<Value, PluginError>> + Send + Sync>,
}

impl HookRegistry {
    // Execute action hooks
    pub async fn do_action(&self, hook_name: &str, data: &ActionData) -> Result<(), PluginError> {
        let actions = self.actions.read().await;
        if let Some(hooks) = actions.get(hook_name) {
            // Sort by priority
            let mut sorted_hooks = hooks.clone();
            sorted_hooks.sort_by_key(|h| h.priority);
            
            for hook in sorted_hooks {
                (hook.callback)(data).await?;
            }
        }
        Ok(())
    }
    
    // Apply filter hooks
    pub async fn apply_filters(&self, hook_name: &str, mut value: Value) -> Result<Value, PluginError> {
        let filters = self.filters.read().await;
        if let Some(hooks) = filters.get(hook_name) {
            // Sort by priority
            let mut sorted_hooks = hooks.clone();
            sorted_hooks.sort_by_key(|h| h.priority);
            
            for hook in sorted_hooks {
                value = (hook.callback)(value).await?;
            }
        }
        Ok(value)
    }
    
    // Remove hooks when plugin is deactivated
    pub async fn remove_plugin_hooks(&self, plugin_id: &str) -> Result<(), PluginError> {
        let mut actions = self.actions.write().await;
        let mut filters = self.filters.write().await;
        
        for hooks in actions.values_mut() {
            hooks.retain(|h| h.plugin_id != plugin_id);
        }
        
        for hooks in filters.values_mut() {
            hooks.retain(|h| h.plugin_id != plugin_id);
        }
        
        Ok(())
    }
}
```

## Plugin Manager Implementation

### Plugin Loading and Management
```rust
// Plugin manager handles plugin lifecycle
pub struct PluginManager {
    plugins: Arc<RwLock<HashMap<String, LoadedPlugin>>>,
    plugin_loader: Arc<PluginLoader>,
    dependency_resolver: Arc<DependencyResolver>,
    security_manager: Arc<PluginSecurityManager>,
    resource_manager: Arc<ResourceManager>,
    marketplace_client: Arc<MarketplaceClient>,
}

pub struct LoadedPlugin {
    pub metadata: PluginMetadata,
    pub instance: Box<dyn AdxPlugin>,
    pub status: PluginStatus,
    pub context: PluginContext,
    pub resource_usage: ResourceUsage,
    pub loaded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PluginStatus {
    Loaded,
    Active,
    Inactive,
    Error(String),
    Updating,
}

impl PluginManager {
    // Load plugin from file system
    pub async fn load_plugin(&mut self, plugin_path: &Path) -> Result<(), PluginError> {
        // Load plugin binary
        let plugin_binary = self.plugin_loader.load_binary(plugin_path).await?;
        
        // Validate plugin signature
        self.security_manager.validate_signature(&plugin_binary).await?;
        
        // Extract metadata
        let metadata = self.plugin_loader.extract_metadata(&plugin_binary).await?;
        
        // Check dependencies
        self.dependency_resolver.check_dependencies(&metadata.dependencies).await?;
        
        // Create plugin instance
        let plugin_instance = self.plugin_loader.instantiate_plugin(&plugin_binary).await?;
        
        // Create plugin context
        let context = self.create_plugin_context(&metadata).await?;
        
        // Create loaded plugin
        let loaded_plugin = LoadedPlugin {
            metadata: metadata.clone(),
            instance: plugin_instance,
            status: PluginStatus::Loaded,
            context,
            resource_usage: ResourceUsage::default(),
            loaded_at: Utc::now(),
        };
        
        // Store plugin
        self.plugins.write().await.insert(metadata.id.clone(), loaded_plugin);
        
        Ok(())
    }
    
    // Activate plugin
    pub async fn activate_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        let mut plugins = self.plugins.write().await;
        let plugin = plugins.get_mut(plugin_id)
            .ok_or(PluginError::PluginNotFound(plugin_id.to_string()))?;
        
        // Check if already active
        if plugin.status == PluginStatus::Active {
            return Ok(());
        }
        
        // Activate dependencies first
        for dep in &plugin.metadata.dependencies {
            if !dep.optional {
                self.activate_plugin(&dep.plugin_id).await?;
            }
        }
        
        // Activate plugin
        plugin.instance.activate(&plugin.context).await?;
        plugin.status = PluginStatus::Active;
        
        // Register extension points
        plugin.instance.register_ui_components(&mut self.ui_registry).await?;
        plugin.instance.register_api_endpoints(&mut self.api_registry).await?;
        plugin.instance.register_workflow_steps(&mut self.workflow_registry).await?;
        plugin.instance.register_hooks(&mut self.hook_registry).await?;
        
        Ok(())
    }
    
    // Hot-reload plugin without restart
    pub async fn reload_plugin(&mut self, plugin_id: &str) -> Result<(), PluginError> {
        // Deactivate current version
        self.deactivate_plugin(plugin_id).await?;
        
        // Remove from memory
        self.plugins.write().await.remove(plugin_id);
        
        // Load new version
        let plugin_path = self.get_plugin_path(plugin_id).await?;
        self.load_plugin(&plugin_path).await?;
        
        // Activate new version
        self.activate_plugin(plugin_id).await?;
        
        Ok(())
    }
    
    // Install plugin from marketplace
    pub async fn install_plugin_from_marketplace(
        &mut self,
        plugin_id: &str,
        tenant_id: TenantId,
    ) -> Result<(), PluginError> {
        // Get plugin info from marketplace
        let plugin_info = self.marketplace_client.get_plugin_info(plugin_id).await?;
        
        // Check license requirements
        if plugin_info.is_premium {
            self.validate_license_for_plugin(tenant_id, plugin_id).await?;
        }
        
        // Download plugin
        let plugin_binary = self.marketplace_client.download_plugin(plugin_id).await?;
        
        // Verify integrity
        self.security_manager.verify_plugin_integrity(&plugin_binary, &plugin_info.checksum).await?;
        
        // Install plugin
        let plugin_path = self.install_plugin_binary(&plugin_binary, plugin_id).await?;
        
        // Load and activate
        self.load_plugin(&plugin_path).await?;
        self.activate_plugin(plugin_id).await?;
        
        Ok(())
    }
}
```

### Plugin Sandboxing and Security
```rust
// Plugin security manager
pub struct PluginSecurityManager {
    sandbox_manager: Arc<SandboxManager>,
    permission_checker: Arc<PermissionChecker>,
    code_scanner: Arc<CodeScanner>,
}

impl PluginSecurityManager {
    // Validate plugin before loading
    pub async fn validate_plugin(&self, plugin_binary: &[u8]) -> Result<(), PluginError> {
        // Scan for malicious code
        let scan_result = self.code_scanner.scan_binary(plugin_binary).await?;
        if scan_result.has_threats() {
            return Err(PluginError::SecurityThreat(scan_result.threats));
        }
        
        // Check code signing
        self.verify_code_signature(plugin_binary).await?;
        
        // Validate permissions
        let metadata = self.extract_metadata(plugin_binary).await?;
        self.validate_permissions(&metadata.permissions).await?;
        
        Ok(())
    }
    
    // Create secure sandbox for plugin execution
    pub async fn create_sandbox(&self, plugin_id: &str) -> Result<PluginSandbox, PluginError> {
        let sandbox = PluginSandbox {
            plugin_id: plugin_id.to_string(),
            memory_limit: 256 * 1024 * 1024, // 256MB
            cpu_limit: 0.5, // 50% of one CPU core
            network_access: NetworkAccess::Restricted,
            file_system_access: FileSystemAccess::PluginDirectory,
            database_access: DatabaseAccess::PluginSchema,
            allowed_syscalls: self.get_allowed_syscalls(),
        };
        
        self.sandbox_manager.create_sandbox(sandbox).await
    }
    
    // Monitor plugin resource usage
    pub async fn monitor_plugin_resources(&self, plugin_id: &str) -> Result<ResourceUsage, PluginError> {
        let usage = self.sandbox_manager.get_resource_usage(plugin_id).await?;
        
        // Check limits
        if usage.memory_bytes > self.get_memory_limit(plugin_id) {
            return Err(PluginError::ResourceLimitExceeded("memory".to_string()));
        }
        
        if usage.cpu_usage > self.get_cpu_limit(plugin_id) {
            return Err(PluginError::ResourceLimitExceeded("cpu".to_string()));
        }
        
        Ok(usage)
    }
}

// Plugin sandbox implementation
pub struct PluginSandbox {
    pub plugin_id: String,
    pub memory_limit: usize,
    pub cpu_limit: f64,
    pub network_access: NetworkAccess,
    pub file_system_access: FileSystemAccess,
    pub database_access: DatabaseAccess,
    pub allowed_syscalls: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum NetworkAccess {
    None,
    Restricted, // Only HTTPS to approved domains
    Full,
}

#[derive(Debug, Clone)]
pub enum FileSystemAccess {
    None,
    PluginDirectory,
    TenantFiles,
}

#[derive(Debug, Clone)]
pub enum DatabaseAccess {
    None,
    PluginSchema,
    TenantData,
}
```

## Extension Points

### UI Component Extension
```rust
// UI component registry for plugin extensions
pub struct UIComponentRegistry {
    components: Arc<RwLock<HashMap<String, UIComponent>>>,
    component_tree: Arc<RwLock<ComponentTree>>,
}

pub trait UIComponent: Send + Sync {
    fn render(&self, props: &ComponentProps) -> Result<ComponentHTML, PluginError>;
    fn get_component_type(&self) -> ComponentType;
    fn get_dependencies(&self) -> Vec<String>;
}

#[derive(Debug, Clone)]
pub enum ComponentType {
    Page,
    Widget,
    Modal,
    Form,
    Chart,
    Table,
    Custom(String),
}

impl UIComponentRegistry {
    pub async fn register_component(
        &self,
        plugin_id: &str,
        component_id: &str,
        component: Box<dyn UIComponent>,
    ) -> Result<(), PluginError> {
        let full_id = format!("{}:{}", plugin_id, component_id);
        
        // Validate component
        self.validate_component(&component).await?;
        
        // Register component
        self.components.write().await.insert(full_id.clone(), component);
        
        // Update component tree
        self.component_tree.write().await.add_component(full_id, component_id.to_string());
        
        Ok(())
    }
    
    pub async fn render_component(
        &self,
        component_id: &str,
        props: &ComponentProps,
    ) -> Result<ComponentHTML, PluginError> {
        let components = self.components.read().await;
        let component = components.get(component_id)
            .ok_or(PluginError::ComponentNotFound(component_id.to_string()))?;
        
        component.render(props)
    }
}

// Example plugin UI component
pub struct ClientDashboardComponent;

impl UIComponent for ClientDashboardComponent {
    fn render(&self, props: &ComponentProps) -> Result<ComponentHTML, PluginError> {
        let tenant_id = props.get_tenant_id()?;
        let clients = self.get_clients(tenant_id)?;
        
        Ok(ComponentHTML {
            html: format!(r#"
                <div class="client-dashboard">
                    <h2>Client Dashboard</h2>
                    <div class="client-grid">
                        {}
                    </div>
                </div>
            "#, self.render_client_cards(&clients)),
            css: include_str!("client-dashboard.css").to_string(),
            js: include_str!("client-dashboard.js").to_string(),
        })
    }
    
    fn get_component_type(&self) -> ComponentType {
        ComponentType::Page
    }
    
    fn get_dependencies(&self) -> Vec<String> {
        vec!["chart.js".to_string(), "datatables".to_string()]
    }
}
```

### API Endpoint Extension
```rust
// API endpoint registry for plugin extensions
pub struct APIEndpointRegistry {
    endpoints: Arc<RwLock<HashMap<String, APIEndpoint>>>,
    middleware_stack: Arc<RwLock<Vec<Box<dyn APIMiddleware>>>>,
}

pub trait APIHandler: Send + Sync {
    async fn handle(&self, request: APIRequest) -> Result<APIResponse, PluginError>;
    fn get_method(&self) -> HttpMethod;
    fn get_path_pattern(&self) -> String;
    fn get_required_permissions(&self) -> Vec<Permission>;
}

impl APIEndpointRegistry {
    pub async fn register_endpoint(
        &self,
        plugin_id: &str,
        path: &str,
        handler: Box<dyn APIHandler>,
    ) -> Result<(), PluginError> {
        let full_path = format!("/plugins/{}{}", plugin_id, path);
        
        // Validate endpoint
        self.validate_endpoint(&handler).await?;
        
        // Register endpoint
        let endpoint = APIEndpoint {
            plugin_id: plugin_id.to_string(),
            path: full_path.clone(),
            handler,
            registered_at: Utc::now(),
        };
        
        self.endpoints.write().await.insert(full_path, endpoint);
        
        Ok(())
    }
    
    pub async fn handle_request(
        &self,
        path: &str,
        request: APIRequest,
    ) -> Result<APIResponse, PluginError> {
        let endpoints = self.endpoints.read().await;
        let endpoint = endpoints.get(path)
            .ok_or(PluginError::EndpointNotFound(path.to_string()))?;
        
        // Check permissions
        self.check_permissions(&request, &endpoint.handler.get_required_permissions()).await?;
        
        // Apply middleware
        let processed_request = self.apply_middleware(request).await?;
        
        // Handle request
        endpoint.handler.handle(processed_request).await
    }
}

// Example plugin API handler
pub struct ClientAPIHandler {
    client_service: Arc<ClientService>,
}

#[async_trait]
impl APIHandler for ClientAPIHandler {
    async fn handle(&self, request: APIRequest) -> Result<APIResponse, PluginError> {
        match (request.method, request.path.as_str()) {
            (HttpMethod::GET, "/clients") => {
                let clients = self.client_service.list_clients(request.tenant_id).await?;
                Ok(APIResponse::json(clients))
            }
            (HttpMethod::POST, "/clients") => {
                let client_data: CreateClientRequest = request.json()?;
                let client = self.client_service.create_client(request.tenant_id, client_data).await?;
                Ok(APIResponse::json(client).with_status(201))
            }
            (HttpMethod::GET, path) if path.starts_with("/clients/") => {
                let client_id = path.strip_prefix("/clients/").unwrap();
                let client = self.client_service.get_client(request.tenant_id, client_id).await?;
                Ok(APIResponse::json(client))
            }
            _ => Err(PluginError::MethodNotAllowed),
        }
    }
    
    fn get_method(&self) -> HttpMethod {
        HttpMethod::Any
    }
    
    fn get_path_pattern(&self) -> String {
        "/clients/*".to_string()
    }
    
    fn get_required_permissions(&self) -> Vec<Permission> {
        vec![
            Permission::new("clients", "read"),
            Permission::new("clients", "write"),
        ]
    }
}
```

### Workflow Step Extension
```rust
// Workflow step registry for plugin extensions
pub struct WorkflowStepRegistry {
    steps: Arc<RwLock<HashMap<String, WorkflowStepDefinition>>>,
    ai_enhancement_registry: Arc<AIEnhancementRegistry>,
}

pub trait WorkflowStep: Send + Sync {
    async fn execute(&self, input: WorkflowStepInput) -> Result<WorkflowStepOutput, PluginError>;
    fn get_step_type(&self) -> WorkflowStepType;
    fn get_input_schema(&self) -> JsonSchema;
    fn get_output_schema(&self) -> JsonSchema;
    fn supports_ai_enhancement(&self) -> bool;
    async fn ai_enhance(&self, input: WorkflowStepInput, ai_context: AIContext) -> Result<WorkflowStepOutput, PluginError>;
}

#[derive(Debug, Clone)]
pub enum WorkflowStepType {
    Action,
    Condition,
    Loop,
    Parallel,
    Custom(String),
}

impl WorkflowStepRegistry {
    pub async fn register_step(
        &self,
        plugin_id: &str,
        step_id: &str,
        step: Box<dyn WorkflowStep>,
    ) -> Result<(), PluginError> {
        let full_id = format!("{}:{}", plugin_id, step_id);
        
        // Validate step
        self.validate_step(&step).await?;
        
        // Register step
        let step_definition = WorkflowStepDefinition {
            plugin_id: plugin_id.to_string(),
            step_id: step_id.to_string(),
            step,
            registered_at: Utc::now(),
        };
        
        self.steps.write().await.insert(full_id, step_definition);
        
        Ok(())
    }
    
    pub async fn execute_step(
        &self,
        step_id: &str,
        input: WorkflowStepInput,
        ai_enhanced: bool,
    ) -> Result<WorkflowStepOutput, PluginError> {
        let steps = self.steps.read().await;
        let step_def = steps.get(step_id)
            .ok_or(PluginError::StepNotFound(step_id.to_string()))?;
        
        if ai_enhanced && step_def.step.supports_ai_enhancement() {
            let ai_context = self.ai_enhancement_registry.get_context(step_id).await?;
            step_def.step.ai_enhance(input, ai_context).await
        } else {
            step_def.step.execute(input).await
        }
    }
}

// Example plugin workflow step
pub struct SendEmailStep {
    email_service: Arc<EmailService>,
}

#[async_trait]
impl WorkflowStep for SendEmailStep {
    async fn execute(&self, input: WorkflowStepInput) -> Result<WorkflowStepOutput, PluginError> {
        let email_data: EmailData = input.data.try_into()?;
        
        let result = self.email_service.send_email(
            &email_data.to,
            &email_data.subject,
            &email_data.body,
            email_data.template.as_deref(),
        ).await?;
        
        Ok(WorkflowStepOutput {
            data: json!({
                "email_id": result.email_id,
                "status": "sent",
                "sent_at": result.sent_at
            }),
            next_steps: vec![],
        })
    }
    
    fn get_step_type(&self) -> WorkflowStepType {
        WorkflowStepType::Action
    }
    
    fn get_input_schema(&self) -> JsonSchema {
        json!({
            "type": "object",
            "properties": {
                "to": {"type": "string", "format": "email"},
                "subject": {"type": "string"},
                "body": {"type": "string"},
                "template": {"type": "string", "optional": true}
            },
            "required": ["to", "subject", "body"]
        })
    }
    
    fn get_output_schema(&self) -> JsonSchema {
        json!({
            "type": "object",
            "properties": {
                "email_id": {"type": "string"},
                "status": {"type": "string"},
                "sent_at": {"type": "string", "format": "date-time"}
            }
        })
    }
    
    fn supports_ai_enhancement(&self) -> bool {
        true
    }
    
    async fn ai_enhance(&self, mut input: WorkflowStepInput, ai_context: AIContext) -> Result<WorkflowStepOutput, PluginError> {
        // AI can enhance email content, subject, and timing
        if let Some(ai_service) = ai_context.get_service("email_optimization") {
            let enhanced_content = ai_service.optimize_email_content(&input.data).await?;
            input.data = enhanced_content;
        }
        
        self.execute(input).await
    }
}
```

## Plugin Marketplace

### Marketplace Architecture
```rust
// Plugin marketplace service
pub struct PluginMarketplace {
    plugin_registry: Arc<PluginRegistry>,
    payment_processor: Arc<PaymentProcessor>,
    review_system: Arc<ReviewSystem>,
    analytics_service: Arc<AnalyticsService>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplacePlugin {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub author: PluginAuthor,
    pub category: String,
    pub tags: Vec<String>,
    pub price: Option<Price>,
    pub license: LicenseType,
    pub compatibility: CompatibilityInfo,
    pub screenshots: Vec<String>,
    pub documentation_url: String,
    pub support_url: String,
    pub download_count: u64,
    pub rating: f64,
    pub review_count: u32,
    pub last_updated: DateTime<Utc>,
    pub featured: bool,
}

impl PluginMarketplace {
    pub async fn search_plugins(&self, query: &SearchQuery) -> Result<SearchResults, MarketplaceError> {
        let mut results = self.plugin_registry.search(query).await?;
        
        // Apply filters
        if let Some(category) = &query.category {
            results.retain(|p| p.category == *category);
        }
        
        if let Some(price_range) = &query.price_range {
            results.retain(|p| self.matches_price_range(p, price_range));
        }
        
        // Sort results
        match query.sort_by {
            SortBy::Popularity => results.sort_by_key(|p| std::cmp::Reverse(p.download_count)),
            SortBy::Rating => results.sort_by(|a, b| b.rating.partial_cmp(&a.rating).unwrap()),
            SortBy::Updated => results.sort_by_key(|p| std::cmp::Reverse(p.last_updated)),
            SortBy::Name => results.sort_by(|a, b| a.name.cmp(&b.name)),
        }
        
        // Paginate
        let total = results.len();
        let start = (query.page - 1) * query.per_page;
        let end = std::cmp::min(start + query.per_page, total);
        let plugins = results[start..end].to_vec();
        
        Ok(SearchResults {
            plugins,
            total,
            page: query.page,
            per_page: query.per_page,
            total_pages: (total + query.per_page - 1) / query.per_page,
        })
    }
    
    pub async fn install_plugin(
        &self,
        plugin_id: &str,
        tenant_id: TenantId,
        user_id: UserId,
    ) -> Result<InstallationResult, MarketplaceError> {
        let plugin = self.plugin_registry.get_plugin(plugin_id).await?;
        
        // Check if plugin is premium
        if let Some(price) = &plugin.price {
            // Process payment
            let payment_result = self.payment_processor.process_payment(
                user_id,
                tenant_id,
                price.clone(),
                format!("Plugin: {}", plugin.name),
            ).await?;
            
            if !payment_result.successful {
                return Err(MarketplaceError::PaymentFailed(payment_result.error));
            }
        }
        
        // Download plugin
        let plugin_binary = self.download_plugin(plugin_id).await?;
        
        // Install plugin
        let installation = PluginInstallation {
            plugin_id: plugin_id.to_string(),
            tenant_id,
            user_id,
            version: plugin.version.clone(),
            installed_at: Utc::now(),
            license_key: self.generate_license_key(&plugin, tenant_id).await?,
        };
        
        self.plugin_registry.record_installation(&installation).await?;
        
        // Update analytics
        self.analytics_service.record_download(plugin_id).await?;
        
        Ok(InstallationResult {
            plugin_binary,
            installation,
        })
    }
    
    pub async fn submit_review(
        &self,
        plugin_id: &str,
        user_id: UserId,
        review: PluginReview,
    ) -> Result<(), MarketplaceError> {
        // Validate user has installed the plugin
        self.validate_user_can_review(plugin_id, user_id).await?;
        
        // Submit review
        self.review_system.submit_review(plugin_id, user_id, review).await?;
        
        // Update plugin rating
        self.update_plugin_rating(plugin_id).await?;
        
        Ok(())
    }
}
```

## Default Plugins

### Client Management Plugin
```rust
// Client Management Plugin - First-party default plugin
pub struct ClientManagementPlugin {
    client_service: Arc<ClientService>,
    portal_builder: Arc<PortalBuilder>,
    project_tracker: Arc<ProjectTracker>,
}

#[async_trait]
impl AdxPlugin for ClientManagementPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "client-management".to_string(),
            name: "Client Management".to_string(),
            version: "1.0.0".to_string(),
            description: "Comprehensive client relationship management with portal builder".to_string(),
            author: "ADX Core Team".to_string(),
            website: Some("https://adxcore.com/plugins/client-management".to_string()),
            license: "Proprietary".to_string(),
            dependencies: vec![],
            min_core_version: "1.0.0".to_string(),
            max_core_version: None,
            permissions: vec![
                Permission::new("clients", "*"),
                Permission::new("projects", "*"),
                Permission::new("files", "read"),
                Permission::new("files", "share"),
            ],
            categories: vec!["business".to_string(), "crm".to_string()],
            tags: vec!["clients".to_string(), "projects".to_string(), "portal".to_string()],
        }
    }
    
    async fn activate(&self, context: &PluginContext) -> Result<(), PluginError> {
        // Register UI components
        context.register_ui_component("client-dashboard", Box::new(ClientDashboardComponent)).await?;
        context.register_ui_component("client-list", Box::new(ClientListComponent)).await?;
        context.register_ui_component("client-portal-builder", Box::new(PortalBuilderComponent)).await?;
        
        // Register API endpoints
        context.register_endpoint("/clients", Box::new(ClientAPIHandler::new(self.client_service.clone()))).await?;
        context.register_endpoint("/projects", Box::new(ProjectAPIHandler::new(self.project_tracker.clone()))).await?;
        context.register_endpoint("/portal", Box::new(PortalAPIHandler::new(self.portal_builder.clone()))).await?;
        
        // Register workflow steps
        context.register_workflow_step("create-client", Box::new(CreateClientStep::new(self.client_service.clone()))).await?;
        context.register_workflow_step("setup-portal", Box::new(SetupPortalStep::new(self.portal_builder.clone()))).await?;
        
        // Register hooks
        context.add_action("user_registered", |data| {
            Box::pin(async move {
                // Automatically create client record for new users
                self.handle_user_registration(data).await
            })
        }).await?;
        
        context.add_filter("file_permissions", |permissions| {
            Box::pin(async move {
                // Add client-specific file permissions
                self.add_client_file_permissions(permissions).await
            })
        }).await?;
        
        Ok(())
    }
    
    async fn deactivate(&self) -> Result<(), PluginError> {
        // Cleanup resources
        Ok(())
    }
    
    async fn uninstall(&self) -> Result<(), PluginError> {
        // Remove client data (with user confirmation)
        Ok(())
    }
    
    async fn register_database_migrations(&self, registry: &mut MigrationRegistry) -> Result<(), PluginError> {
        registry.add_migration("001_create_clients_table", include_str!("migrations/001_create_clients_table.sql"))?;
        registry.add_migration("002_create_projects_table", include_str!("migrations/002_create_projects_table.sql"))?;
        registry.add_migration("003_create_client_portals_table", include_str!("migrations/003_create_client_portals_table.sql"))?;
        Ok(())
    }
    
    async fn get_config_schema(&self) -> Result<ConfigSchema, PluginError> {
        Ok(json!({
            "type": "object",
            "properties": {
                "default_portal_theme": {
                    "type": "string",
                    "enum": ["light", "dark", "custom"],
                    "default": "light"
                },
                "auto_create_portal": {
                    "type": "boolean",
                    "default": true
                },
                "client_file_access": {
                    "type": "string",
                    "enum": ["none", "own_files", "project_files", "all_files"],
                    "default": "project_files"
                }
            }
        }))
    }
}
```

This comprehensive plugin architecture provides:

1. **WordPress-style familiarity** with hooks, filters, and actions
2. **Hot-loading capabilities** for seamless plugin updates
3. **Secure sandboxing** with resource limits and permission controls
4. **Comprehensive extension points** for UI, API, workflows, and database
5. **Marketplace integration** with payment processing and reviews
6. **Dependency management** with automatic resolution
7. **Default plugins** providing essential business functionality
8. **Developer-friendly SDK** with comprehensive documentation
9. **Revenue sharing model** for plugin ecosystem growth
10. **Enterprise-grade security** with code scanning and validation