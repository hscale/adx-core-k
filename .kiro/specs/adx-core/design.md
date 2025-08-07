# ADX CORE v2 - Temporal-First Microservices Design Document

## Overview

ADX CORE is a temporal-first, multi-tenant SaaS platform built with microservices architecture for both backend and frontend. The platform uses Temporal.io workflows as the PRIMARY orchestration mechanism, with domain-aligned microservices, Module Federation-based frontend microservices, and optional Backend-for-Frontend (BFF) services for optimal performance and team autonomy.

## Temporal-First Microservices Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                           Frontend Microservices Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │    Shell    │  │    Auth     │  │   Tenant    │  │    File     │  │    User     │  │
│  │ Application │  │ Micro-App   │  │ Micro-App   │  │ Micro-App   │  │ Micro-App   │  │
│  │(Port 3000)  │  │(Port 3001)  │  │(Port 3002)  │  │(Port 3003)  │  │(Port 3004)  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
│           │              │              │              │              │           │
│           └──────────────┼──────────────┼──────────────┼──────────────┘           │
│                    Module Federation + Event Bus                                 │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │ Web Browser │  │   Desktop   │  │ Mobile Web  │  │ Native Apps │  │  Workflow   │  │
│  │   (React)   │  │   (Tauri)   │  │(Responsive) │  │(Tauri 2.0)  │  │ Micro-App   │  │
│  │             │  │Win/Mac/Linux│  │             │  │ iOS/Android │  │(Port 3005)  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        Optional BFF Services Layer                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   Auth BFF  │  │ Tenant BFF  │  │  File BFF   │  │  User BFF   │  │Workflow BFF │  │
│  │(Port 4001)  │  │(Port 4002)  │  │(Port 4003)  │  │(Port 4004)  │  │(Port 4005)  │  │
│  │Node.js/TS   │  │Node.js/TS   │  │ Rust/Axum   │  │ Rust/Axum   │  │ Rust/Axum   │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
│           │              │              │              │              │           │
│           └──────────────┼──────────────┼──────────────┼──────────────┘           │
│                    Temporal Workflow Clients + Redis Caching                    │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
┌─────────────────────────────────────────────────────────────────────────────────┐
│                      Temporal-First API Gateway                                │
│                    (Port 8080 - Workflow Orchestration)                       │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
┌─────────────────────────────────────────────────────────────────────────────────┐
│                     Backend Microservices Layer                                │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │Auth Service │  │Tenant Service│  │File Service │  │User Service │  │Workflow Svc │  │
│  │(Port 8081)  │  │(Port 8085)  │  │(Port 8083)  │  │(Port 8082)  │  │(Port 8084)  │  │
│  │HTTP + Worker│  │HTTP + Worker│  │HTTP + Worker│  │HTTP + Worker│  │Cross-Service│  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
│           │              │              │              │              │           │
│           └──────────────┼──────────────┼──────────────┼──────────────┘           │
│                    Temporal Activities + Direct Endpoints                      │
└─────────────────────────────────────────────────────────────────────────────────┘
                                         │
┌─────────────────────────────────────────────────────────────────────────────────┐
│                        Infrastructure Layer                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │  Temporal   │  │ PostgreSQL  │  │    Redis    │  │File Storage │  │ Monitoring  │  │
│  │  Cluster    │  │  (Primary)  │  │  (Cache)    │  │(Multi-Prov) │  │   Stack     │  │
│  │(Port 7233)  │  │             │  │             │  │             │  │             │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────────────────────────┘
```

### Core Components

#### 1. Frontend Microservices Layer
- **Shell Application (Port 3000)**: Module Federation host that orchestrates all micro-frontends
  - Global routing and navigation
  - Shared state management and event bus
  - Authentication context and theme providers
  - Error boundaries and fallback components
- **Domain Micro-Frontends**: Independent applications mirroring backend services
  - Auth Micro-App (Port 3001): Login, registration, MFA, SSO flows
  - Tenant Micro-App (Port 3002): Tenant switching, management, settings
  - File Micro-App (Port 3003): File upload, management, sharing
  - User Micro-App (Port 3004): User profiles, preferences, settings
  - Workflow Micro-App (Port 3005): Workflow monitoring, AI features
- **Cross-Platform Support**: Universal deployment across web, desktop (Tauri), and mobile

#### 2. Optional BFF Services Layer
- **Purpose**: Temporal workflow clients providing optimized APIs for micro-frontends
- **Auth BFF (Port 4001)**: Aggregates auth, user, and tenant data (Node.js/TypeScript)
- **Tenant BFF (Port 4002)**: Tenant management and switching optimization (Node.js/TypeScript)
- **File BFF (Port 4003)**: File operations with metadata aggregation (Rust/Axum)
- **User BFF (Port 4004)**: User profile and preference management (Rust/Axum)
- **Workflow BFF (Port 4005)**: Workflow status and AI integration (Rust/Axum)
- **Capabilities**: Redis caching, request batching, response shaping, performance optimization

#### 3. Temporal-First API Gateway (Port 8080)
- **Purpose**: Single entry point with intelligent routing between direct calls and workflows
- **Responsibilities**:
  - Simple operations: Direct routing to backend services
  - Complex operations: Temporal workflow initiation and tracking
  - Cross-service orchestration through workflow coordination
  - Real-time workflow status and progress endpoints
  - Authentication, authorization, and rate limiting
  - API versioning and backward compatibility

#### 4. Backend Microservices (Dual-Mode Architecture)
- **Auth Service (Port 8081)**: Authentication, SSO, MFA, RBAC
  - HTTP Server: Direct login, token validation, user CRUD
  - Temporal Worker: User registration, password reset, MFA setup workflows
- **User Service (Port 8082)**: User management and preferences
  - HTTP Server: User profile CRUD, preference management
  - Temporal Worker: User onboarding, profile sync, preference workflows
- **File Service (Port 8083)**: File storage and management
  - HTTP Server: File metadata, direct upload/download
  - Temporal Worker: File processing, virus scanning, migration workflows
- **Workflow Service (Port 8084)**: Cross-service workflow orchestration
  - Dedicated service for complex multi-service workflows
  - AI integration and workflow intelligence
- **Tenant Service (Port 8085)**: Multi-tenant management
  - HTTP Server: Tenant CRUD, membership management
  - Temporal Worker: Tenant provisioning, billing, suspension workflows

#### 5. Temporal Workflow Orchestration
- **Core Workflows**: All complex multi-step operations implemented as Temporal workflows
  - User registration and onboarding workflows
  - File upload, processing, and migration workflows
  - Tenant provisioning and management workflows
  - Cross-service coordination workflows
  - AI-enhanced business process workflows
- **Workflow Benefits**: Automatic retry, compensation, observability, and versioning
- **Cross-Service Coordination**: Services communicate through workflow orchestration, not direct calls

#### 6. Infrastructure Layer
- **Temporal Cluster (Port 7233)**: Primary orchestration engine for all complex operations
- **PostgreSQL**: Primary database with per-service schemas and connection pools
- **Redis**: Caching layer for BFF services, sessions, and rate limiting
- **File Storage**: Multi-provider abstraction (S3, GCS, Azure, local)
- **Monitoring Stack**: Prometheus, Grafana, OpenTelemetry for observability

## Microservices Communication Patterns

### 1. Frontend Micro-Frontend Communication
```typescript
// Event Bus Pattern for Cross-Micro-Frontend Communication
interface EventBus {
  // Typed events for type safety
  emit<T>(event: string, data: T): void;
  on<T>(event: string, handler: (data: T) => void): void;
  off(event: string, handler: Function): void;
}

// Example: Auth micro-app notifies other micro-apps of login
authMicroApp.eventBus.emit('user:login', { 
  userId: 'user123', 
  tenantId: 'tenant456' 
});

// Tenant micro-app listens for auth events
tenantMicroApp.eventBus.on('user:login', (data) => {
  // Update tenant context
  updateTenantContext(data.tenantId);
});
```

### 2. BFF to Backend Communication (Temporal Workflow Clients)
```rust
// BFF services act as Temporal workflow clients, not direct service clients
pub struct AuthBFF {
    temporal_client: Arc<TemporalClient>,
    cache: Arc<RedisClient>,
}

impl AuthBFF {
    // Aggregate data through workflow execution
    pub async fn get_user_dashboard_data(&self, user_id: &str) -> Result<DashboardData, Error> {
        // Check cache first
        if let Some(cached) = self.cache.get(&format!("dashboard:{}", user_id)).await? {
            return Ok(cached);
        }
        
        // Execute workflows in parallel for data aggregation
        let workflows = vec![
            self.temporal_client.start_workflow("get_user_profile", user_id),
            self.temporal_client.start_workflow("get_user_tenants", user_id),
            self.temporal_client.start_workflow("get_recent_activity", user_id),
        ];
        
        let results = futures::future::try_join_all(workflows).await?;
        let dashboard_data = DashboardData::from_workflow_results(results);
        
        // Cache aggregated result
        self.cache.set(&format!("dashboard:{}", user_id), &dashboard_data, 300).await?;
        
        Ok(dashboard_data)
    }
}
```

### 3. Backend Service Communication (Temporal Workflows Only)
```rust
// Services DO NOT call each other directly - only through Temporal workflows
#[temporal::workflow]
pub async fn user_onboarding_workflow(request: UserOnboardingRequest) -> Result<OnboardingResult, Error> {
    // Step 1: Create user account (Auth Service activity)
    let user = call_activity(
        AuthServiceActivities::create_user,
        CreateUserRequest::from(request.clone()),
    ).await?;
    
    // Step 2: Create default tenant (Tenant Service activity)
    let tenant = call_activity(
        TenantServiceActivities::create_default_tenant,
        CreateTenantRequest { user_id: user.id },
    ).await?;
    
    // Step 3: Setup file storage (File Service activity)
    let storage = call_activity(
        FileServiceActivities::setup_user_storage,
        SetupStorageRequest { 
            user_id: user.id, 
            tenant_id: tenant.id 
        },
    ).await?;
    
    // Step 4: Send welcome notification (Notification Service activity)
    call_activity(
        NotificationServiceActivities::send_welcome_email,
        WelcomeEmailRequest {
            user_id: user.id,
            email: user.email,
            tenant_name: tenant.name,
        },
    ).await?;
    
    Ok(OnboardingResult {
        user_id: user.id,
        tenant_id: tenant.id,
        storage_quota: storage.quota,
    })
}
```

### 4. API Gateway Routing Logic
```rust
// Intelligent routing between direct calls and workflow initiation
pub struct ApiGateway {
    temporal_client: Arc<TemporalClient>,
    service_clients: HashMap<String, Arc<dyn ServiceClient>>,
}

impl ApiGateway {
    pub async fn handle_request(&self, request: ApiRequest) -> Result<ApiResponse, Error> {
        match self.classify_request(&request) {
            RequestType::Simple => {
                // Direct routing to backend service
                let service = self.service_clients.get(&request.service_name)
                    .ok_or_else(|| Error::ServiceNotFound)?;
                service.handle_request(request).await
            }
            RequestType::Complex => {
                // Initiate Temporal workflow
                let workflow_id = format!("{}-{}", request.operation, Uuid::new_v4());
                let handle = self.temporal_client
                    .start_workflow(
                        request.workflow_type,
                        workflow_id.clone(),
                        request.task_queue,
                        request.payload,
                    )
                    .await?;
                
                // Return operation tracking information
                Ok(ApiResponse::WorkflowStarted {
                    operation_id: workflow_id,
                    status_url: format!("/api/workflows/{}/status", workflow_id),
                    estimated_duration: self.estimate_workflow_duration(&request.workflow_type),
                })
            }
        }
    }
    
    fn classify_request(&self, request: &ApiRequest) -> RequestType {
        match (request.method.as_str(), request.path.as_str()) {
            // Simple operations - direct routing
            ("GET", path) if path.starts_with("/api/users/") => RequestType::Simple,
            ("PUT", path) if path.starts_with("/api/users/") && path.ends_with("/profile") => RequestType::Simple,
            
            // Complex operations - workflow initiation
            ("POST", "/api/users/register") => RequestType::Complex,
            ("POST", "/api/files/upload") => RequestType::Complex,
            ("POST", "/api/tenants/switch") => RequestType::Complex,
            ("DELETE", path) if path.starts_with("/api/tenants/") => RequestType::Complex,
            
            _ => RequestType::Simple, // Default to simple
        }
    }
}

## Module System Architecture

### Comprehensive Module Framework

```rust
// Core module trait - comprehensive extension system
#[async_trait]
pub trait AdxModule: Send + Sync {
    // Module metadata and capabilities
    fn metadata(&self) -> ModuleMetadata;
    fn capabilities(&self) -> ModuleCapabilities;
    fn dependencies(&self) -> Vec<ModuleDependency>;
    
    // Lifecycle hooks
    async fn install(&self, context: &ModuleContext) -> Result<(), ModuleError>;
    async fn activate(&self, context: &ModuleContext) -> Result<(), ModuleError>;
    async fn deactivate(&self) -> Result<(), ModuleError>;
    async fn uninstall(&self) -> Result<(), ModuleError>;
    async fn update(&self, from_version: &str, context: &ModuleContext) -> Result<(), ModuleError>;
    
    // Extension points
    fn register_routes(&self) -> Vec<ModuleRoute>;
    fn register_ui_components(&self) -> Vec<UiComponent>;
    fn register_workflows(&self) -> Vec<WorkflowDefinition>;
    fn register_database_migrations(&self) -> Vec<Migration>;
    fn register_event_handlers(&self) -> Vec<EventHandler>;
    fn register_permissions(&self) -> Vec<Permission>;
    fn register_settings(&self) -> Vec<ModuleSetting>;
    
    // Cross-platform support
    fn register_desktop_features(&self) -> Vec<DesktopFeature>;
    fn register_mobile_features(&self) -> Vec<MobileFeature>;
    fn register_web_components(&self) -> Vec<WebComponent>;
}

// Module manager with comprehensive lifecycle management
pub struct ModuleManager {
    modules: HashMap<String, Box<dyn AdxModule>>,
    module_registry: Arc<ModuleRegistry>,
    marketplace: Arc<ModuleMarketplace>,
    event_bus: Arc<EventBus>,
    dependency_resolver: Arc<DependencyResolver>,
    sandbox_manager: Arc<SandboxManager>,
}

impl ModuleManager {
    // Advanced module loading with dependency resolution
    pub async fn install_module(&mut self, module_id: &str, tenant_id: TenantId) -> Result<(), ModuleError> {
        let module_info = self.marketplace.get_module_info(module_id).await?;
        
        // Resolve and validate dependencies
        let dependencies = self.dependency_resolver.resolve_dependencies(&module_info.dependencies).await?;
        
        // Download and validate module
        let module_package = self.marketplace.download_module(module_id).await?;
        self.validate_module_security(&module_package).await?;
        
        // Install dependencies first
        for dep in dependencies {
            if !self.is_module_installed(&dep.id) {
                self.install_module(&dep.id, tenant_id).await?;
            }
        }
        
        // Load and install module
        let module = self.load_module_from_package(module_package).await?;
        module.install(&ModuleContext::new(tenant_id)).await?;
        self.register_module(module_id.to_string(), module).await?;
        
        Ok(())
    }
    
    // Hot-reload modules without restart
    pub async fn reload_module(&mut self, module_id: &str) -> Result<(), ModuleError> {
        self.deactivate_module(module_id).await?;
        self.load_module_by_id(module_id).await?;
        self.activate_module(module_id).await?;
        Ok(())
    }
    
    // Batch operations for multiple modules
    pub async fn bulk_install_modules(&mut self, module_ids: Vec<String>, tenant_id: TenantId) -> Result<Vec<ModuleInstallResult>, ModuleError> {
        let mut results = Vec::new();
        
        for module_id in module_ids {
            let result = match self.install_module(&module_id, tenant_id).await {
                Ok(_) => ModuleInstallResult::Success(module_id.clone()),
                Err(e) => ModuleInstallResult::Failed(module_id.clone(), e),
            };
            results.push(result);
        }
        
        Ok(results)
    }
}
```

### Module Extension Points and Event System

```rust
// Advanced event system with typed events and middleware
pub struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn EventHandler>>>,
    middleware: Vec<Box<dyn EventMiddleware>>,
    event_history: Arc<RwLock<EventHistory>>,
}

impl EventBus {
    // Typed action hooks with middleware support
    pub async fn emit_event<T: Event>(&self, event: T) -> Result<(), Error> {
        // Apply middleware
        let processed_event = self.apply_middleware(event).await?;
        
        // Record event for debugging and analytics
        self.event_history.write().await.record_event(&processed_event);
        
        // Execute handlers
        if let Some(handlers) = self.handlers.get(&processed_event.event_type()) {
            for handler in handlers {
                handler.handle(&processed_event).await?;
            }
        }
        Ok(())
    }
    
    // Advanced filter system with validation
    pub async fn apply_filters<T: FilterableData>(&self, filter_name: &str, data: T) -> Result<T, Error> {
        let mut result = data;
        
        if let Some(handlers) = self.handlers.get(filter_name) {
            for handler in handlers {
                result = handler.filter(result).await?;
                
                // Validate filtered data
                if !result.is_valid() {
                    return Err(Error::InvalidFilterResult(filter_name.to_string()));
                }
            }
        }
        
        Ok(result)
    }
    
    // Module-specific event namespacing
    pub async fn emit_module_event(&self, module_id: &str, event_name: &str, data: &dyn Any) -> Result<(), Error> {
        let namespaced_event = format!("module:{}:{}", module_id, event_name);
        self.emit_event(GenericEvent::new(namespaced_event, data)).await
    }
}

// Module capability system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleCapabilities {
    pub ui_extensions: Vec<UiExtensionPoint>,
    pub api_extensions: Vec<ApiExtensionPoint>,
    pub workflow_extensions: Vec<WorkflowExtensionPoint>,
    pub database_extensions: Vec<DatabaseExtensionPoint>,
    pub cross_platform_features: CrossPlatformFeatures,
    pub required_permissions: Vec<Permission>,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossPlatformFeatures {
    pub web_support: bool,
    pub desktop_support: Vec<DesktopPlatform>,
    pub mobile_support: Vec<MobilePlatform>,
    pub native_integrations: Vec<NativeIntegration>,
}
```

### Module Marketplace Architecture

```rust
// Comprehensive module marketplace with advanced features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleListing {
    pub id: String,
    pub name: String,
    pub description: String,
    pub long_description: String,
    pub version: String,
    pub author: ModuleAuthor,
    pub category: ModuleCategory,
    pub subcategory: Option<String>,
    pub price: Option<Decimal>,
    pub pricing_model: PricingModel,
    pub rating: f32,
    pub review_count: u32,
    pub downloads: u64,
    pub active_installations: u64,
    pub screenshots: Vec<Screenshot>,
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
pub enum PricingModel {
    Free,
    OneTime(Decimal),
    Subscription { monthly: Decimal, yearly: Option<Decimal> },
    Usage { base_price: Decimal, per_unit: Decimal, unit_type: String },
    Enterprise, // Contact for pricing
}

pub struct ModuleMarketplace {
    registry: Arc<ModuleRegistry>,
    payment_processor: Arc<PaymentProcessor>,
    storage: Arc<dyn FileStorage>,
    recommendation_engine: Arc<RecommendationEngine>,
    security_scanner: Arc<SecurityScanner>,
    analytics: Arc<MarketplaceAnalytics>,
    review_system: Arc<ReviewSystem>,
}

impl ModuleMarketplace {
    // Advanced search with AI-powered recommendations
    pub async fn search_modules(&self, query: &str, filters: ModuleFilters, user_context: &UserContext) -> Result<ModuleSearchResults, Error> {
        let mut results = self.registry.search(query, filters).await?;
        
        // Apply AI-powered ranking based on user context
        results = self.recommendation_engine.rank_results(results, user_context).await?;
        
        // Add compatibility indicators
        for result in &mut results.modules {
            result.compatibility_score = self.calculate_compatibility_score(result, user_context).await?;
        }
        
        Ok(results)
    }
    
    // Intelligent module recommendations
    pub async fn get_recommendations(&self, user_context: &UserContext, limit: usize) -> Result<Vec<ModuleListing>, Error> {
        let user_profile = self.analytics.get_user_profile(&user_context.user_id).await?;
        let tenant_profile = self.analytics.get_tenant_profile(&user_context.tenant_id).await?;
        
        self.recommendation_engine.generate_recommendations(user_profile, tenant_profile, limit).await
    }
    
    // Comprehensive module installation with workflow orchestration
    pub async fn install_module(&self, module_id: &str, tenant_id: TenantId, user_id: UserId) -> Result<InstallationResult, Error> {
        let module_info = self.registry.get_module_info(module_id).await?;
        
        // Security validation
        let security_results = self.security_scanner.scan_module(module_id).await?;
        if !security_results.is_safe() {
            return Err(Error::SecurityValidationFailed(security_results.issues));
        }
        
        // License and payment processing
        if let Some(price) = module_info.price {
            let payment_result = self.payment_processor.process_payment(
                tenant_id,
                user_id,
                price,
                &module_info.pricing_model,
            ).await?;
            
            if !payment_result.is_successful() {
                return Err(Error::PaymentFailed(payment_result.error));
            }
        }
        
        // Initiate installation workflow
        let installation_workflow = InstallationWorkflow::new(
            module_id.to_string(),
            tenant_id,
            user_id,
            module_info,
        );
        
        let workflow_result = self.execute_installation_workflow(installation_workflow).await?;
        
        // Record analytics
        self.analytics.record_installation(module_id, tenant_id, user_id).await?;
        
        Ok(workflow_result)
    }
    
    // Module discovery and browsing
    pub async fn browse_categories(&self) -> Result<Vec<ModuleCategory>, Error> {
        self.registry.get_categories_with_counts().await
    }
    
    // Featured and trending modules
    pub async fn get_featured_modules(&self, limit: usize) -> Result<Vec<ModuleListing>, Error> {
        self.registry.get_featured_modules(limit).await
    }
    
    pub async fn get_trending_modules(&self, timeframe: Timeframe, limit: usize) -> Result<Vec<ModuleListing>, Error> {
        self.analytics.get_trending_modules(timeframe, limit).await
    }
}
```

### Default First-Party Modules

```rust
// Default Module: Client & Customer Management
pub struct ClientManagementModule {
    client_repo: Arc<dyn ClientRepository>,
    portal_builder: ClientPortalBuilder,
    notification_service: Arc<NotificationService>,
    workflow_service: Arc<WorkflowService>,
}

#[async_trait]
impl AdxModule for ClientManagementModule {
    fn metadata(&self) -> ModuleMetadata {
        ModuleMetadata {
            id: "adx-core-client-management".to_string(),
            name: "Client & Customer Management".to_string(),
            version: "1.0.0".to_string(),
            description: "Comprehensive client and customer management with branded portals".to_string(),
            long_description: "Full-featured client management system with customer portals, project tracking, and communication tools".to_string(),
            author: ModuleAuthor {
                name: "ADX CORE Team".to_string(),
                email: "modules@adxcore.com".to_string(),
                website: "https://adxcore.com".to_string(),
            },
            category: ModuleCategory::BusinessManagement,
            is_default: true, // Ships with platform
            price: None, // Free with core platform
            pricing_model: PricingModel::Free,
            permissions: vec![Permission::ClientManagement, Permission::PortalAccess, Permission::FileSharing],
            min_platform_version: "2.0.0".to_string(),
        }
    }

    fn capabilities(&self) -> ModuleCapabilities {
        ModuleCapabilities {
            ui_extensions: vec![
                UiExtensionPoint::Dashboard("client-dashboard".to_string()),
                UiExtensionPoint::Navigation("client-nav".to_string()),
                UiExtensionPoint::Settings("client-settings".to_string()),
            ],
            api_extensions: vec![
                ApiExtensionPoint::RestEndpoints(vec!["/api/clients", "/api/client-portal", "/api/client-projects"]),
                ApiExtensionPoint::GraphQLTypes(vec!["Client", "ClientProject", "PortalAccess"]),
            ],
            workflow_extensions: vec![
                WorkflowExtensionPoint::Activities(vec!["client_onboarding", "portal_setup", "client_notification"]),
                WorkflowExtensionPoint::Workflows(vec!["client_lifecycle", "portal_management"]),
            ],
            database_extensions: vec![
                DatabaseExtensionPoint::Tables(vec!["clients", "client_projects", "client_file_access"]),
                DatabaseExtensionPoint::Views(vec!["client_dashboard_view", "portal_activity_view"]),
            ],
            cross_platform_features: CrossPlatformFeatures {
                web_support: true,
                desktop_support: vec![DesktopPlatform::Windows, DesktopPlatform::MacOS, DesktopPlatform::Linux],
                mobile_support: vec![MobilePlatform::iOS, MobilePlatform::Android],
                native_integrations: vec![
                    NativeIntegration::FileSystem,
                    NativeIntegration::Notifications,
                    NativeIntegration::Calendar,
                ],
            },
            required_permissions: vec![Permission::ClientManagement, Permission::PortalAccess],
            resource_requirements: ResourceRequirements {
                min_memory_mb: 128,
                max_memory_mb: 512,
                cpu_cores: 1,
                storage_mb: 50,
                network_required: true,
            },
        }
    }

    fn dependencies(&self) -> Vec<ModuleDependency> {
        vec![
            ModuleDependency {
                module_id: "adx-core-file-management".to_string(),
                version_requirement: ">=1.0.0".to_string(),
                optional: false,
            },
            ModuleDependency {
                module_id: "adx-core-notifications".to_string(),
                version_requirement: ">=1.0.0".to_string(),
                optional: true,
            },
        ]
    }

    async fn install(&self, context: &ModuleContext) -> Result<(), ModuleError> {
        // Create database tables
        for migration in self.register_database_migrations() {
            context.execute_migration(migration).await?;
        }
        
        // Set up default configuration
        context.set_module_config("default_portal_theme", "professional").await?;
        context.set_module_config("auto_create_portal", "true").await?;
        
        Ok(())
    }

    async fn activate(&self, context: &ModuleContext) -> Result<(), ModuleError> {
        // Register client management UI components
        context.register_ui_component("client-dashboard", ClientDashboard).await?;
        context.register_ui_component("client-list", ClientListView).await?;
        context.register_ui_component("client-portal-builder", ClientPortalBuilder).await?;
        context.register_ui_component("client-project-tracker", ClientProjectTracker).await?;
        
        // Register API endpoints
        context.register_endpoint("/api/clients", self.client_endpoints()).await?;
        context.register_endpoint("/api/client-portal", self.portal_endpoints()).await?;
        context.register_endpoint("/api/client-projects", self.project_endpoints()).await?;
        context.register_endpoint("/api/client-communications", self.communication_endpoints()).await?;
        
        // Register Temporal workflows
        context.register_workflow("client_onboarding_workflow", self.client_onboarding_workflow()).await?;
        context.register_workflow("client_portal_setup_workflow", self.portal_setup_workflow()).await?;
        context.register_workflow("client_project_lifecycle_workflow", self.project_lifecycle_workflow()).await?;
        
        // Register event handlers
        context.add_event_handler("file_shared", |event| {
            self.handle_file_shared_with_client(event).await
        }).await?;
        
        context.add_event_handler("user_login", |event| {
            self.handle_client_portal_login(event).await
        }).await?;
        
        // Register filters for extending core functionality
        context.add_filter("file_permissions", |permissions| {
            self.add_client_file_permissions(permissions).await
        }).await?;
        
        context.add_filter("dashboard_widgets", |widgets| {
            self.add_client_dashboard_widgets(widgets).await
        }).await?;
        
        Ok(())
    }

    fn register_database_migrations(&self) -> Vec<Migration> {
        vec![
            Migration {
                version: "001_create_clients_table".to_string(),
                sql: include_str!("../migrations/001_create_clients_table.sql").to_string(),
                rollback_sql: Some(include_str!("../migrations/001_rollback_clients_table.sql").to_string()),
            },
            Migration {
                version: "002_create_client_projects_table".to_string(),
                sql: include_str!("../migrations/002_create_client_projects_table.sql").to_string(),
                rollback_sql: Some(include_str!("../migrations/002_rollback_client_projects_table.sql").to_string()),
            },
            Migration {
                version: "003_create_client_file_access_table".to_string(),
                sql: include_str!("../migrations/003_create_client_file_access_table.sql").to_string(),
                rollback_sql: Some(include_str!("../migrations/003_rollback_client_file_access_table.sql").to_string()),
            },
            Migration {
                version: "004_create_client_communications_table".to_string(),
                sql: include_str!("../migrations/004_create_client_communications_table.sql").to_string(),
                rollback_sql: Some(include_str!("../migrations/004_rollback_client_communications_table.sql").to_string()),
            },
        ]
    }

    fn register_settings(&self) -> Vec<ModuleSetting> {
        vec![
            ModuleSetting {
                key: "default_portal_theme".to_string(),
                name: "Default Portal Theme".to_string(),
                description: "Default theme for client portals".to_string(),
                setting_type: SettingType::Select(vec!["professional".to_string(), "modern".to_string(), "minimal".to_string()]),
                default_value: "professional".to_string(),
                required: true,
            },
            ModuleSetting {
                key: "auto_create_portal".to_string(),
                name: "Auto-Create Client Portals".to_string(),
                description: "Automatically create portal access when adding new clients".to_string(),
                setting_type: SettingType::Boolean,
                default_value: "true".to_string(),
                required: false,
            },
            ModuleSetting {
                key: "portal_session_timeout".to_string(),
                name: "Portal Session Timeout (minutes)".to_string(),
                description: "How long client portal sessions remain active".to_string(),
                setting_type: SettingType::Number { min: 15, max: 1440 },
                default_value: "480".to_string(),
                required: false,
            },
        ]
    }
}

// Client data models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Client {
    pub id: ClientId,
    pub company_id: TenantId,
    pub name: String,
    pub email: String,
    pub phone: Option<String>,
    pub company_name: Option<String>,
    pub address: Option<Address>,
    pub client_type: ClientType,
    pub status: ClientStatus,
    pub assigned_team_id: Option<TeamId>,
    pub assigned_user_id: Option<UserId>,
    pub custom_fields: HashMap<String, serde_json::Value>,
    pub portal_access: PortalAccess,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientType {
    Individual,
    SmallBusiness,
    Enterprise,
    NonProfit,
    Government,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClientStatus {
    Prospect,
    Active,
    Inactive,
    Former,
    Blocked,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortalAccess {
    pub enabled: bool,
    pub login_url: String,
    pub permissions: Vec<PortalPermission>,
    pub custom_branding: Option<BrandingConfig>,
    pub last_login: Option<DateTime<Utc>>,
    pub login_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortalPermission {
    ViewFiles,
    DownloadFiles,
    UploadFiles,
    ViewProjects,
    CommentOnFiles,
    ViewInvoices,
    ViewReports,
}

// Client repository trait
#[async_trait]
pub trait ClientRepository: Send + Sync {
    async fn create_client(&self, client: CreateClient) -> Result<Client, Error>;
    async fn get_client_by_id(&self, id: ClientId) -> Result<Option<Client>, Error>;
    async fn list_company_clients(&self, company_id: TenantId) -> Result<Vec<Client>, Error>;
    async fn update_client(&self, id: ClientId, updates: UpdateClient) -> Result<Client, Error>;
    async fn delete_client(&self, id: ClientId) -> Result<(), Error>;
    async fn grant_file_access(&self, client_id: ClientId, file_id: FileId, permission: PortalPermission) -> Result<(), Error>;
    async fn revoke_file_access(&self, client_id: ClientId, file_id: FileId) -> Result<(), Error>;
}
```

## Frontend Microservices Architecture

### Module Federation Configuration

#### Shell Application (Host)
```typescript
// micro-frontends/shell/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'shell',
      remotes: {
        authMicroApp: 'http://localhost:3001/assets/remoteEntry.js',
        tenantMicroApp: 'http://localhost:3002/assets/remoteEntry.js',
        fileMicroApp: 'http://localhost:3003/assets/remoteEntry.js',
        userMicroApp: 'http://localhost:3004/assets/remoteEntry.js',
        workflowMicroApp: 'http://localhost:3005/assets/remoteEntry.js',
      },
      shared: {
        react: { singleton: true, requiredVersion: '^18.2.0' },
        'react-dom': { singleton: true, requiredVersion: '^18.2.0' },
        'react-router-dom': { singleton: true, requiredVersion: '^6.20.0' },
        '@tanstack/react-query': { singleton: true, requiredVersion: '^5.8.0' },
        'zustand': { singleton: true, requiredVersion: '^4.4.0' },
        '@adx-core/design-system': { singleton: true },
        'tailwindcss': { singleton: true },
      },
    }),
  ],
  server: { port: 3000 },
  preview: { port: 3000 },
});
```

#### Micro-Frontend (Remote)
```typescript
// micro-frontends/auth-micro-app/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'authMicroApp',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './LoginPage': './src/pages/LoginPage.tsx',
        './RegisterPage': './src/pages/RegisterPage.tsx',
        './MFASetupPage': './src/pages/MFASetupPage.tsx',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        'react-router-dom': { singleton: true },
        '@tanstack/react-query': { singleton: true },
        'zustand': { singleton: true },
        '@adx-core/design-system': { singleton: true },
      },
    }),
  ],
  server: { port: 3001 },
  preview: { port: 3001 },
});
```

### Frontend Microservices Technology Stack

```typescript
// Technology choices per micro-frontend
{
  "shell_application": {
    "framework": "React 18+ with TypeScript",
    "port": 3000,
    "responsibilities": [
      "Global routing and navigation",
      "Authentication context management", 
      "Theme and i18n providers",
      "Error boundaries and fallbacks",
      "Cross-micro-frontend event bus",
      "Performance monitoring"
    ]
  },
  "micro_frontends": {
    "auth_micro_app": {
      "framework": "React + TypeScript",
      "port": 3001,
      "domain": "Authentication flows, SSO, MFA",
      "bff_service": "Auth BFF (Port 4001)"
    },
    "tenant_micro_app": {
      "framework": "React + TypeScript", 
      "port": 3002,
      "domain": "Tenant switching, management, settings",
      "bff_service": "Tenant BFF (Port 4002)"
    },
    "file_micro_app": {
      "framework": "React + TypeScript",
      "port": 3003, 
      "domain": "File upload, management, sharing",
      "bff_service": "File BFF (Port 4003)"
    },
    "user_micro_app": {
      "framework": "React + TypeScript",
      "port": 3004,
      "domain": "User profiles, preferences, settings", 
      "bff_service": "User BFF (Port 4004)"
    },
    "workflow_micro_app": {
      "framework": "React + TypeScript",
      "port": 3005,
      "domain": "Workflow monitoring, AI features",
      "bff_service": "Workflow BFF (Port 4005)"
    }
  },
  "shared_infrastructure": {
    "design_system": "@adx-core/design-system",
    "styling": "TailwindCSS with shared configuration",
    "state_management": "Zustand (global) + React Query (server state)",
    "routing": "React Router v6 with micro-frontend coordination",
    "testing": "Vitest + React Testing Library + Playwright E2E",
    "i18n": "react-i18next with namespace-based translations",
    "build_tool": "Vite with Module Federation Plugin",
    "cross_platform": "Tauri 2.0 for desktop and mobile"
  }
}
```

### Platform-Specific Builds

```bash
# Development (all platforms)
npm run dev              # Web development server
npm run dev:desktop      # Tauri desktop development
npm run dev:mobile       # Tauri mobile development (iOS/Android)

# Production builds
npm run build:web        # Web application bundle
npm run build:desktop    # Desktop applications (Windows, macOS, Linux)
npm run build:mobile     # Mobile applications (iOS, Android)
npm run build:all        # All platform builds

# Testing
npm run test             # Unit and integration tests
npm run test:e2e         # End-to-end tests across platforms
npm run test:mobile      # Mobile-specific testing
```

### Tauri 2.0 Configuration

```json
// src-tauri/tauri.conf.json
{
  "productName": "ADX CORE",
  "version": "1.0.0",
  "build": {
    "beforeBuildCommand": "npm run build",
    "beforeDevCommand": "npm run dev",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "ADX CORE",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self'; connect-src ipc: http://ipc.localhost"
    }
  },
  "bundle": {
    "active": true,
    "targets": {
      "desktop": ["deb", "msi", "dmg", "appimage"],
      "mobile": ["ios", "android"]
    },
    "identifier": "com.adxcore.app",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"],
    "resources": ["resources/*"],
    "copyright": "Copyright © 2024 ADX CORE",
    "category": "Productivity",
    "shortDescription": "Enterprise SaaS Platform",
    "longDescription": "ADX CORE is a multi-tenant SaaS platform for enterprise applications"
  },
  "plugins": {
    "fs": {
      "scope": ["$APPDATA/adx-core/*", "$DOCUMENT/*"]
    },
    "notification": {},
    "os": {},
    "shell": {
      "scope": [
        {
          "name": "open-url",
          "cmd": "open",
          "args": ["{{url}}"]
        }
      ]
    }
  }
}
```

### Mobile-Specific Configuration

```json
// src-tauri/gen/android/app/src/main/AndroidManifest.xml additions
<uses-permission android:name="android.permission.INTERNET" />
<uses-permission android:name="android.permission.CAMERA" />
<uses-permission android:name="android.permission.READ_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.WRITE_EXTERNAL_STORAGE" />
<uses-permission android:name="android.permission.ACCESS_FINE_LOCATION" />
<uses-permission android:name="android.permission.ACCESS_COARSE_LOCATION" />

// iOS Info.plist additions
<key>NSCameraUsageDescription</key>
<string>ADX CORE needs camera access for file uploads</string>
<key>NSLocationWhenInUseUsageDescription</key>
<string>ADX CORE needs location access for location-based features</string>
<key>NSPhotoLibraryUsageDescription</key>
<string>ADX CORE needs photo library access for file uploads</string>
```

## Components and Inter

### 1. Repository Layer (Rust Traits)

```rust
// Core repository abstractions
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn create_user(&self, user: CreateUser) -> Result<User, Error>;
    async fn get_user_by_id(&self, id: UserId) -> Result<Option<User>, Error>;
    async fn update_user(&self, id: UserId, updates: UpdateUser) -> Result<User, Error>;
    async fn delete_user(&self, id: UserId) -> Result<(), Error>;
}

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn create_tenant(&self, tenant: CreateTenant) -> Result<Tenant, Error>;
    async fn get_tenant_by_id(&self, id: TenantId) -> Result<Option<Tenant>, Error>;
    async fn list_user_tenants(&self, user_id: UserId) -> Result<Vec<Tenant>, Error>;
}

#[async_trait]
pub trait LicenseRepository: Send + Sync {
    async fn create_license(&self, license: CreateLicense) -> Result<License, Error>;
    async fn validate_license(&self, tenant_id: TenantId) -> Result<LicenseStatus, Error>;
    async fn update_quota_usage(&self, tenant_id: TenantId, usage: QuotaUsage) -> Result<(), Error>;
}

#[async_trait]
pub trait TranslationRepository: Send + Sync {
    async fn get_translation(
        &self,
        key: &str,
        language: &str,
        namespace: &str,
        tenant_id: Option<TenantId>,
    ) -> Result<String, Error>;
    
    async fn set_translation(
        &self,
        key: &str,
        language: &str,
        value: &str,
        namespace: &str,
        tenant_id: Option<TenantId>,
    ) -> Result<(), Error>;
    
    async fn get_translations_for_language(
        &self,
        language: &str,
        namespace: &str,
        tenant_id: Option<TenantId>,
    ) -> Result<HashMap<String, String>, Error>;
    
    async fn update_user_preferences(&self, user_id: UserId, preferences: UserPreferences) -> Result<(), Error>;
}
```

### 2. Service Layer

```rust
// Business logic services
pub struct AuthService {
    user_repo: Arc<dyn UserRepository>,
    temporal_client: TemporalClient,
}

impl AuthService {
    pub async fn register_user(&self, request: RegisterRequest) -> Result<User, Error> {
        // Start user registration workflow
        let workflow_id = format!("user-registration-{}", Uuid::new_v4());
        
        self.temporal_client
            .start_workflow(
                workflow_id,
                UserRegistrationWorkflow,
                request,
            )
            .await
    }
    
    pub async fn authenticate(&self, credentials: Credentials) -> Result<AuthToken, Error> {
        // Authentication logic with MFA support
    }
    
    pub async fn update_user_preferences(&self, user_id: UserId, preferences: UserPreferences) -> Result<(), Error> {
        self.user_repo.update_user_preferences(user_id, preferences).await
    }
}

// Internationalization service
pub struct I18nService {
    translation_repo: Arc<dyn TranslationRepository>,
    cache: Arc<RwLock<HashMap<String, HashMap<String, String>>>>, // namespace -> key -> value
}

impl I18nService {
    pub async fn get_translation(
        &self,
        key: &str,
        language: &str,
        namespace: Option<&str>,
        tenant_id: Option<TenantId>,
    ) -> Result<String, Error> {
        let namespace = namespace.unwrap_or("default");
        let cache_key = format!("{}:{}:{}", namespace, language, tenant_id.map_or("global".to_string(), |id| id.to_string()));
        
        // Check cache first
        if let Some(translations) = self.cache.read().await.get(&cache_key) {
            if let Some(value) = translations.get(key) {
                return Ok(value.clone());
            }
        }
        
        // Fallback to database
        self.translation_repo
            .get_translation(key, language, namespace, tenant_id)
            .await
            .or_else(|_| {
                // Fallback to English if translation not found
                self.translation_repo
                    .get_translation(key, "en", namespace, tenant_id)
                    .await
            })
    }
    
    pub async fn set_translation(
        &self,
        key: &str,
        language: &str,
        value: &str,
        namespace: Option<&str>,
        tenant_id: Option<TenantId>,
    ) -> Result<(), Error> {
        self.translation_repo
            .set_translation(key, language, value, namespace.unwrap_or("default"), tenant_id)
            .await?;
            
        // Invalidate cache
        self.invalidate_cache(namespace.unwrap_or("default"), language, tenant_id).await;
        Ok(())
    }
}
```

### 3. Temporal Workflows

```rust
#[temporal::workflow]
pub async fn user_registration_workflow(request: RegisterRequest) -> Result<User, Error> {
    // Step 1: Create user account
    let user = create_user_activity(request.clone()).await?;
    
    // Step 2: Send verification email
    send_verification_email_activity(user.id, user.email.clone()).await?;
    
    // Step 3: Setup default tenant
    let tenant = create_default_tenant_activity(user.id).await?;
    
    // Step 4: Provision basic license
    provision_trial_license_activity(tenant.id).await?;
    
    Ok(user)
}

#[temporal::workflow]
pub async fn file_migration_workflow(
    from_provider: StorageProvider,
    to_provider: StorageProvider,
    tenant_id: TenantId,
) -> Result<MigrationResult, Error> {
    // Step 1: List all files
    let files = list_tenant_files_activity(tenant_id, from_provider).await?;
    
    // Step 2: Migrate in batches
    let mut migrated = 0;
    for batch in files.chunks(100) {
        migrate_file_batch_activity(batch, from_provider, to_provider).await?;
        migrated += batch.len();
        
        // Update progress
        update_migration_progress_activity(tenant_id, migrated, files.len()).await?;
    }
    
    // Step 3: Verify migration
    verify_migration_activity(tenant_id, to_provider, files.len()).await?;
    
    // Step 4: Cleanup old files (optional)
    if should_cleanup() {
        cleanup_old_files_activity(tenant_id, from_provider).await?;
    }
    
    Ok(MigrationResult {
        files_migrated: files.len(),
        duration: get_workflow_duration(),
    })
}
```

## Simplified Hybrid AI Workflow Architecture

### Temporal-First Approach

ADX CORE uses Temporal.io as the foundation for all workflow orchestration, keeping AI integration simple and reliable:

```rust
// Simple workflow service that leverages Temporal's built-in capabilities
pub struct WorkflowService {
    temporal_client: TemporalClient,
    ai_service: Arc<AIService>,
}

impl WorkflowService {
    // Execute any workflow with optional AI enhancement
    pub async fn execute_workflow<T>(
        &self,
        workflow_fn: impl Fn(bool) -> WorkflowResult<T>,
        ai_enhanced: bool,
    ) -> Result<T, Error> {
        // Let Temporal handle all the orchestration complexity
        let result = self.temporal_client
            .start_workflow(workflow_fn, ai_enhanced)
            .await?;
        
        Ok(result)
    }
}

// Example: Simple AI-enhanced user onboarding
#[workflow]
pub async fn user_onboarding_workflow(ai_enhanced: bool) -> WorkflowResult<OnboardingResult> {
    // Step 1: Create user (always done)
    let user = create_user_activity().await?;
    
    // Step 2: Generate welcome message (AI-enhanced if enabled)
    let welcome_message = if ai_enhanced {
        ai_generate_welcome_activity(user.clone()).await?
    } else {
        get_default_welcome_activity().await?
    };
    
    // Step 3: Send welcome email
    send_welcome_email_activity(user.email, welcome_message).await?;
    
    Ok(OnboardingResult { user_id: user.id, ai_enhanced })
}

// Standard workflow templates available to all users
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: WorkflowCategory,
    pub steps: Vec<WorkflowStep>,
    pub triggers: Vec<WorkflowTrigger>,
    pub is_ai_enhanced: bool, // Indicates if AI plugins can enhance this workflow
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowCategory {
    UserManagement,
    FileOperations,
    DataProcessing,
    Notifications,
    Integrations,
    Business,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: String,
    pub name: String,
    pub step_type: StepType,
    pub conditions: Vec<Condition>,
    pub actions: Vec<Action>,
    pub ai_enhancement_points: Vec<AIEnhancementPoint>, // Where AI can add intelligence
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepType {
    RuleBased,      // Standard rule-based execution
    AIEnhanced,     // Can be enhanced by AI plugins
    Conditional,    // Conditional branching
    Parallel,       // Parallel execution
    Sequential,     // Sequential execution
}
```

### AI Intelligence Plugin Architecture

AI capabilities are provided through optional plugins that enhance the core orchestration infrastructure:

```rust
// AI Engine Plugin trait
#[async_trait]
pub trait AIWorkflowPlugin: AdxPlugin {
    // AI-powered workflow planning
    async fn plan_workflow(
        &self,
        context: &WorkflowContext,
        user_intent: &str,
        historical_data: &WorkflowHistory,
    ) -> Result<WorkflowPlan, AIError>;
    
    // Intelligent exception handling
    async fn handle_workflow_exception(
        &self,
        execution: &WorkflowExecution,
        error: &WorkflowError,
        context: &WorkflowContext,
    ) -> Result<RecoveryAction, AIError>;
    
    // Workflow optimization recommendations
    async fn optimize_workflow(
        &self,
        workflow: &WorkflowDefinition,
        performance_data: &PerformanceMetrics,
    ) -> Result<OptimizationRecommendations, AIError>;
    
    // Learning and adaptation
    async fn learn_from_execution(
        &self,
        execution: &CompletedWorkflowExecution,
        outcome: &WorkflowOutcome,
    ) -> Result<(), AIError>;
}

// Tiered AI capabilities based on subscription
pub struct AIWorkflowEnhancer {
    ai_plugins: HashMap<TenantId, Vec<Box<dyn AIWorkflowPlugin>>>,
    license_service: Arc<LicenseService>,
    ai_model_registry: Arc<AIModelRegistry>,
}

impl AIWorkflowEnhancer {
    pub async fn enhance_workflow_execution(
        &self,
        workflow: &WorkflowDefinition,
        execution: &mut WorkflowExecution,
        tenant_id: TenantId,
    ) -> Result<(), Error> {
        let license = self.license_service.get_license(tenant_id).await?;
        
        match license.tier {
            LicenseTier::Basic => {
                // All users: Powerful workflow orchestration with rule-based automation
                // No AI enhancement, but full workflow capabilities
            },
            LicenseTier::Premium => {
                // Premium users: AI-enhanced workflows with intelligent planning and optimization
                if let Some(ai_plugins) = self.ai_plugins.get(&tenant_id) {
                    for plugin in ai_plugins {
                        plugin.enhance_workflow_step(workflow, execution).await?;
                    }
                }
            },
            LicenseTier::Enterprise => {
                // Enterprise users: Custom AI models and advanced workflow intelligence
                let custom_models = self.ai_model_registry.get_custom_models(tenant_id).await?;
                for model in custom_models {
                    model.apply_advanced_intelligence(workflow, execution).await?;
                }
            },
        }
        
        Ok(())
    }
}

// AI enhancement points in workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIEnhancementPoint {
    pub id: String,
    pub enhancement_type: AIEnhancementType,
    pub description: String,
    pub required_license_tier: LicenseTier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIEnhancementType {
    IntelligentRouting,     // AI-powered decision making
    PredictiveAnalysis,     // Predict workflow outcomes
    AutomaticOptimization,  // Optimize workflow parameters
    ExceptionPrediction,    // Predict and prevent failures
    PersonalizedExecution,  // Personalize workflow behavior
    SmartScheduling,        // Intelligent timing and resource allocation
}
```

### Seamless Integration Architecture

The hybrid approach ensures seamless integration between core orchestration and AI intelligence:

```rust
// Workflow execution engine that handles both rule-based and AI-enhanced workflows
pub struct HybridWorkflowEngine {
    core_orchestrator: WorkflowOrchestrationService,
    ai_enhancer: AIWorkflowEnhancer,
    execution_monitor: Arc<ExecutionMonitor>,
}

impl HybridWorkflowEngine {
    pub async fn execute_workflow(
        &self,
        workflow: WorkflowDefinition,
        input: WorkflowInput,
        tenant_id: TenantId,
    ) -> Result<WorkflowResult, Error> {
        // Start with core orchestration (available to all users)
        let mut execution = self.core_orchestrator
            .execute_workflow(workflow.clone(), input, tenant_id)
            .await?;
        
        // Apply AI enhancements based on license tier
        self.ai_enhancer
            .enhance_workflow_execution(&workflow, &mut execution, tenant_id)
            .await?;
        
        // Monitor execution with differentiation between rule-based and AI-enhanced steps
        self.execution_monitor
            .track_hybrid_execution(&execution)
            .await?;
        
        // Ensure consistent performance regardless of AI plugin status
        let result = self.ensure_reliable_execution(execution).await?;
        
        Ok(result)
    }
    
    async fn ensure_reliable_execution(
        &self,
        execution: WorkflowExecution,
    ) -> Result<WorkflowResult, Error> {
        // Fallback to rule-based execution if AI plugins fail
        if execution.has_ai_failures() {
            execution.fallback_to_rule_based().await?;
        }
        
        // Maintain backward compatibility
        execution.ensure_backward_compatibility().await?;
        
        Ok(execution.get_result())
    }
}

// Comprehensive monitoring for both workflow types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAnalytics {
    pub execution_id: String,
    pub workflow_type: WorkflowType,
    pub rule_based_steps: Vec<StepMetrics>,
    pub ai_enhanced_steps: Vec<AIStepMetrics>,
    pub performance_metrics: PerformanceMetrics,
    pub reliability_metrics: ReliabilityMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowType {
    RuleBasedOnly,
    AIEnhanced,
    Hybrid,
}
```

This hybrid architecture provides:

1. **Universal Foundation**: All users get powerful Temporal.io-based workflow orchestration
2. **Tiered AI Intelligence**: Premium and enterprise users get AI enhancements through plugins
3. **Platform Flexibility**: AI capabilities remain optional and don't affect core functionality
4. **Seamless Integration**: Rule-based and AI-enhanced workflows work together transparently
5. **Reliable Fallbacks**: System maintains performance even if AI plugins fail
6. **Clear Differentiation**: Monitoring and analytics clearly show which steps use AI vs rules

## Data Models

### Core Entities

```rust
// User and authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub email: String,
    pub name: String,
    pub preferences: UserPreferences,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub email_verified: bool,
    pub mfa_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub language: String,           // "en", "es", "fr", "de", "ja", "zh"
    pub theme: Theme,               // Dark, Light, System
    pub timezone: String,           // "UTC", "America/New_York", etc.
    pub date_format: String,        // "MM/DD/YYYY", "DD/MM/YYYY", etc.
    pub number_format: String,      // "1,234.56", "1.234,56", etc.
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark,
    System,  // Follow system preference
}

// Multi-tenancy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tenant {
    pub id: TenantId,
    pub name: String,
    pub domain: Option<String>,
    pub settings: TenantSettings,
    pub created_at: DateTime<Utc>,
    pub owner_id: UserId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantMembership {
    pub tenant_id: TenantId,
    pub user_id: UserId,
    pub role: TenantRole,
    pub joined_at: DateTime<Utc>,
}

// Licensing and quotas
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct License {
    pub id: LicenseId,
    pub tenant_id: TenantId,
    pub license_type: LicenseType,
    pub quotas: QuotaLimits,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuotaLimits {
    pub api_calls_per_month: u64,
    pub storage_bytes: u64,
    pub bandwidth_bytes_per_month: u64,
    pub concurrent_users: u32,
}

// File management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct File {
    pub id: FileId,
    pub tenant_id: TenantId,
    pub name: String,
    pub size_bytes: u64,
    pub content_type: String,
    pub storage_provider: StorageProvider,
    pub storage_path: String,
    pub permissions: FilePermissions,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

### Database Schema (PostgreSQL)

```sql
-- Users table
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255),
    preferences JSONB DEFAULT '{"language": "en", "theme": "System", "timezone": "UTC", "date_format": "MM/DD/YYYY", "number_format": "1,234.56"}',
    email_verified BOOLEAN DEFAULT FALSE,
    mfa_enabled BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Translations table for multi-language support
CREATE TABLE translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key VARCHAR(255) NOT NULL,
    language VARCHAR(10) NOT NULL,
    value TEXT NOT NULL,
    namespace VARCHAR(100) DEFAULT 'default',
    tenant_id UUID REFERENCES tenants(id), -- NULL for global translations
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(key, language, namespace, tenant_id)
);

-- Tenants table
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) UNIQUE,
    settings JSONB DEFAULT '{}',
    owner_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Tenant memberships
CREATE TABLE tenant_memberships (
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    user_id UUID NOT NULL REFERENCES users(id),
    role VARCHAR(50) NOT NULL,
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (tenant_id, user_id)
);

-- Licenses
CREATE TABLE licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    license_type VARCHAR(50) NOT NULL,
    quotas JSONB NOT NULL,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Files
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    content_type VARCHAR(255),
    storage_provider VARCHAR(50) NOT NULL,
    storage_path VARCHAR(1000) NOT NULL,
    permissions JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Client Management Plugin Tables
CREATE TABLE clients (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    company_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(50),
    company_name VARCHAR(255),
    address JSONB,
    client_type VARCHAR(50) NOT NULL DEFAULT 'Individual',
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    assigned_team_id UUID REFERENCES teams(id),
    assigned_user_id UUID REFERENCES users(id),
    custom_fields JSONB DEFAULT '{}',
    portal_access JSONB DEFAULT '{"enabled": false, "permissions": []}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE client_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id UUID NOT NULL REFERENCES clients(id),
    company_id UUID NOT NULL REFERENCES tenants(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'Active',
    start_date DATE,
    end_date DATE,
    budget DECIMAL(12,2),
    assigned_team_id UUID REFERENCES teams(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE client_file_access (
    client_id UUID NOT NULL REFERENCES clients(id),
    file_id UUID NOT NULL REFERENCES files(id),
    permission VARCHAR(50) NOT NULL,
    granted_by UUID NOT NULL REFERENCES users(id),
    granted_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    PRIMARY KEY (client_id, file_id)
);

-- Indexes for performance
CREATE INDEX idx_tenant_memberships_user_id ON tenant_memberships(user_id);
CREATE INDEX idx_files_tenant_id ON files(tenant_id);
CREATE INDEX idx_licenses_tenant_id ON licenses(tenant_id);
CREATE INDEX idx_clients_company_id ON clients(company_id);
CREATE INDEX idx_clients_assigned_user_id ON clients(assigned_user_id);
CREATE INDEX idx_client_projects_client_id ON client_projects(client_id);
CREATE INDEX idx_client_projects_company_id ON client_projects(company_id);
CREATE INDEX idx_client_file_access_client_id ON client_file_access(client_id);
CREATE INDEX idx_client_file_access_file_id ON client_file_access(file_id);
```

## Error Handling

### Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Authentication failed: {0}")]
    Authentication(String),
    
    #[error("Authorization failed: {0}")]
    Authorization(String),
    
    #[error("Tenant not found: {tenant_id}")]
    TenantNotFound { tenant_id: TenantId },
    
    #[error("License violation: {0}")]
    LicenseViolation(String),
    
    #[error("Quota exceeded: {quota_type}")]
    QuotaExceeded { quota_type: String },
    
    #[error("File storage error: {0}")]
    FileStorage(String),
    
    #[error("Temporal workflow error: {0}")]
    Workflow(String),
}

// Result type alias
pub type Result<T> = std::result::Result<T, Error>;
```

## Testing Strategy

### Unit Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_user_registration() {
        let mut mock_repo = MockUserRepository::new();
        mock_repo
            .expect_create_user()
            .with(eq(create_user_request()))
            .times(1)
            .returning(|_| Ok(mock_user()));
            
        let service = AuthService::new(Arc::new(mock_repo));
        let result = service.register_user(create_user_request()).await;
        
        assert!(result.is_ok());
    }
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_full_user_workflow() {
    let test_db = setup_test_database().await;
    let temporal_client = setup_test_temporal().await;
    
    // Test complete user registration workflow
    let workflow_result = temporal_client
        .execute_workflow(UserRegistrationWorkflow, test_request())
        .await;
        
    assert!(workflow_result.is_ok());
    
    // Verify database state
    let user = test_db.get_user_by_email("test@example.com").await.unwrap();
    assert!(user.is_some());
}
```

This design provides a solid foundation for implementing ADX CORE. Ready to move to the implementation tasks?