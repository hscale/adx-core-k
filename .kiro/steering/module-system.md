# ADX CORE Module System Development Guidelines

## Core Principles

The ADX CORE module system provides extensibility through a comprehensive architecture that supports hot-loading, sandboxing, and marketplace distribution. All module operations leverage Temporal workflows for reliability and observability.

## Module Architecture Patterns

### Module Structure
```
module-name/
├── manifest.json           # Module metadata and dependencies
├── src/
│   ├── backend/           # Temporal activities and workflows
│   ├── frontend/          # Micro-frontend components
│   └── shared/            # Common types and utilities
├── workflows/
│   ├── install.ts         # Installation workflow
│   ├── activate.ts        # Activation workflow
│   └── uninstall.ts       # Cleanup workflow
├── tests/
│   ├── unit/              # Unit tests
│   ├── integration/       # Integration tests
│   └── e2e/               # End-to-end tests
└── docs/
    ├── README.md          # Module documentation
    └── api.md             # API reference
```

### Module Manifest Schema
```json
{
  "name": "module-name",
  "version": "1.0.0",
  "description": "Module description",
  "author": "Author Name",
  "license": "MIT",
  "adxCore": {
    "minVersion": "2.0.0",
    "maxVersion": "2.x.x"
  },
  "dependencies": {
    "other-module": "^1.0.0"
  },
  "permissions": [
    "database:read",
    "database:write",
    "api:external",
    "files:read"
  ],
  "extensionPoints": {
    "backend": {
      "activities": ["./src/backend/activities.js"],
      "workflows": ["./src/backend/workflows.js"],
      "endpoints": ["./src/backend/routes.js"]
    },
    "frontend": {
      "components": ["./src/frontend/components.js"],
      "routes": ["./src/frontend/routes.js"],
      "hooks": ["./src/frontend/hooks.js"]
    }
  },
  "resources": {
    "memory": "256MB",
    "cpu": "0.5",
    "storage": "100MB"
  }
}
```

## Temporal-First Module Operations

### Module Installation Workflow
```rust
#[workflow]
pub async fn install_module_workflow(
    request: InstallModuleRequest,
) -> Result<InstallModuleResult, WorkflowError> {
    // Step 1: Validate module and dependencies
    let validation = call_activity(
        ModuleActivities::validate_module,
        ValidateModuleRequest {
            module_id: request.module_id.clone(),
            tenant_id: request.tenant_id.clone(),
            version: request.version.clone(),
        },
    ).await?;
    
    // Step 2: Download and verify module package
    let package = call_activity(
        ModuleActivities::download_module,
        DownloadModuleRequest {
            module_id: request.module_id.clone(),
            version: request.version.clone(),
            checksum: validation.checksum,
        },
    ).await?;
    
    // Step 3: Install dependencies
    for dependency in validation.dependencies {
        call_activity(
            ModuleActivities::install_dependency,
            InstallDependencyRequest {
                dependency_id: dependency.id,
                version: dependency.version,
                tenant_id: request.tenant_id.clone(),
            },
        ).await?;
    }
    
    // Step 4: Deploy module components
    let deployment = call_activity(
        ModuleActivities::deploy_module,
        DeployModuleRequest {
            package,
            tenant_id: request.tenant_id.clone(),
            sandbox_config: validation.sandbox_config,
        },
    ).await?;
    
    // Step 5: Register module with system
    call_activity(
        ModuleActivities::register_module,
        RegisterModuleRequest {
            module_id: request.module_id.clone(),
            deployment_id: deployment.id,
            tenant_id: request.tenant_id.clone(),
        },
    ).await?;
    
    Ok(InstallModuleResult {
        module_id: request.module_id,
        deployment_id: deployment.id,
        status: ModuleStatus::Installed,
    })
}
```

### Module Activation Workflow
```rust
#[workflow]
pub async fn activate_module_workflow(
    request: ActivateModuleRequest,
) -> Result<ActivateModuleResult, WorkflowError> {
    // Step 1: Validate module is installed
    let module_info = call_activity(
        ModuleActivities::get_module_info,
        GetModuleInfoRequest {
            module_id: request.module_id.clone(),
            tenant_id: request.tenant_id.clone(),
        },
    ).await?;
    
    // Step 2: Initialize module database schema
    if let Some(schema) = module_info.database_schema {
        call_activity(
            ModuleActivities::initialize_schema,
            InitializeSchemaRequest {
                schema,
                tenant_id: request.tenant_id.clone(),
            },
        ).await?;
    }
    
    // Step 3: Register Temporal activities and workflows
    call_activity(
        ModuleActivities::register_workflows,
        RegisterWorkflowsRequest {
            module_id: request.module_id.clone(),
            workflows: module_info.workflows,
            activities: module_info.activities,
        },
    ).await?;
    
    // Step 4: Register API endpoints
    call_activity(
        ModuleActivities::register_endpoints,
        RegisterEndpointsRequest {
            module_id: request.module_id.clone(),
            endpoints: module_info.endpoints,
            tenant_id: request.tenant_id.clone(),
        },
    ).await?;
    
    // Step 5: Register frontend components
    call_activity(
        ModuleActivities::register_frontend_components,
        RegisterFrontendComponentsRequest {
            module_id: request.module_id.clone(),
            components: module_info.frontend_components,
            tenant_id: request.tenant_id.clone(),
        },
    ).await?;
    
    Ok(ActivateModuleResult {
        module_id: request.module_id,
        status: ModuleStatus::Active,
    })
}
```

## Module Development Patterns

### Backend Module Development
```rust
// Module activity implementation
#[async_trait]
pub trait ModuleActivities {
    async fn process_module_data(
        &self,
        request: ProcessModuleDataRequest,
    ) -> Result<ProcessModuleDataResult, ActivityError>;
    
    async fn validate_module_input(
        &self,
        request: ValidateModuleInputRequest,
    ) -> Result<ValidationResult, ActivityError>;
    
    async fn cleanup_module_resources(
        &self,
        request: CleanupModuleResourcesRequest,
    ) -> Result<(), ActivityError>;
}

// Module workflow implementation
#[workflow]
pub async fn module_business_process_workflow(
    request: ModuleBusinessProcessRequest,
) -> Result<ModuleBusinessProcessResult, WorkflowError> {
    // Validate input using module-specific validation
    let validation = call_activity(
        ModuleActivities::validate_module_input,
        ValidateModuleInputRequest {
            data: request.data.clone(),
            tenant_context: request.tenant_context.clone(),
        },
    ).await?;
    
    if !validation.is_valid {
        return Err(WorkflowError::ValidationFailed(validation.errors));
    }
    
    // Process data using module logic
    let result = call_activity(
        ModuleActivities::process_module_data,
        ProcessModuleDataRequest {
            data: request.data,
            processing_options: request.options,
        },
    ).await?;
    
    Ok(ModuleBusinessProcessResult {
        processed_data: result.data,
        metadata: result.metadata,
    })
}
```

### Frontend Module Development
```typescript
// Module micro-frontend component
import { ModuleComponent, useModuleContext } from '@adx-core/module-sdk';

export const MyModuleComponent: ModuleComponent = () => {
    const { moduleConfig, tenantContext, userContext } = useModuleContext();
    
    const handleModuleAction = async (data: any) => {
        // Trigger module workflow through BFF or API Gateway
        const response = await fetch('/api/modules/my-module/workflows/process', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                'Authorization': `Bearer ${userContext.token}`,
                'X-Tenant-ID': tenantContext.tenantId,
            },
            body: JSON.stringify({
                data,
                options: moduleConfig.processingOptions,
            }),
        });
        
        const result = await response.json();
        
        if (result.operationId) {
            // Long-running workflow - poll for status
            pollWorkflowStatus(result.operationId);
        } else {
            // Synchronous result
            handleResult(result);
        }
    };
    
    return (
        <div className="module-container">
            <h2>{moduleConfig.displayName}</h2>
            {/* Module UI components */}
        </div>
    );
};

// Module route registration
export const moduleRoutes = [
    {
        path: '/my-module',
        component: MyModuleComponent,
        permissions: ['module:my-module:read'],
    },
    {
        path: '/my-module/settings',
        component: MyModuleSettingsComponent,
        permissions: ['module:my-module:admin'],
    },
];
```

## Module Sandboxing and Security

### Resource Limits
```rust
// Module resource enforcement
pub struct ModuleSandbox {
    memory_limit: usize,
    cpu_limit: f64,
    storage_limit: usize,
    network_allowed: bool,
    database_permissions: Vec<DatabasePermission>,
}

impl ModuleSandbox {
    pub async fn enforce_limits(&self, module_id: &str) -> Result<(), SandboxError> {
        // Enforce memory limits
        self.enforce_memory_limit(module_id).await?;
        
        // Enforce CPU limits
        self.enforce_cpu_limit(module_id).await?;
        
        // Enforce storage limits
        self.enforce_storage_limit(module_id).await?;
        
        // Enforce network restrictions
        if !self.network_allowed {
            self.block_network_access(module_id).await?;
        }
        
        Ok(())
    }
}
```

### Permission System
```rust
// Module permission validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModulePermission {
    DatabaseRead(String),      // Table name
    DatabaseWrite(String),     // Table name
    ApiExternal(String),       // Domain pattern
    FileRead(String),          // Path pattern
    FileWrite(String),         // Path pattern
    WorkflowExecute(String),   // Workflow type
    TenantDataAccess,          // Access to tenant data
    UserDataAccess,            // Access to user data
}

pub async fn validate_module_permission(
    module_id: &str,
    permission: &ModulePermission,
    context: &RequestContext,
) -> Result<bool, PermissionError> {
    let module_manifest = get_module_manifest(module_id).await?;
    
    // Check if permission is declared in manifest
    if !module_manifest.permissions.contains(permission) {
        return Ok(false);
    }
    
    // Check tenant-specific permissions
    let tenant_permissions = get_tenant_module_permissions(
        &context.tenant_id,
        module_id,
    ).await?;
    
    Ok(tenant_permissions.allows(permission))
}
```

## Module Marketplace Integration

### Module Publishing Workflow
```rust
#[workflow]
pub async fn publish_module_workflow(
    request: PublishModuleRequest,
) -> Result<PublishModuleResult, WorkflowError> {
    // Step 1: Validate module package
    let validation = call_activity(
        MarketplaceActivities::validate_module_package,
        ValidateModulePackageRequest {
            package_data: request.package_data.clone(),
            author_id: request.author_id.clone(),
        },
    ).await?;
    
    // Step 2: Run security scan
    let security_scan = call_activity(
        MarketplaceActivities::security_scan_module,
        SecurityScanRequest {
            package_data: request.package_data.clone(),
            scan_level: ScanLevel::Comprehensive,
        },
    ).await?;
    
    if !security_scan.passed {
        return Err(WorkflowError::SecurityScanFailed(security_scan.issues));
    }
    
    // Step 3: Test module compatibility
    let compatibility_test = call_activity(
        MarketplaceActivities::test_module_compatibility,
        CompatibilityTestRequest {
            package_data: request.package_data.clone(),
            adx_versions: vec!["2.0.0", "2.1.0", "2.2.0"],
        },
    ).await?;
    
    // Step 4: Upload to marketplace
    let upload_result = call_activity(
        MarketplaceActivities::upload_module,
        UploadModuleRequest {
            package_data: request.package_data,
            metadata: validation.metadata,
            compatibility: compatibility_test.results,
        },
    ).await?;
    
    // Step 5: Update marketplace index
    call_activity(
        MarketplaceActivities::update_marketplace_index,
        UpdateMarketplaceIndexRequest {
            module_id: upload_result.module_id.clone(),
            version: upload_result.version.clone(),
        },
    ).await?;
    
    Ok(PublishModuleResult {
        module_id: upload_result.module_id,
        version: upload_result.version,
        marketplace_url: upload_result.marketplace_url,
    })
}
```

## Default Modules

### Client Management Module
```rust
// Client management workflows
#[workflow]
pub async fn create_client_workflow(
    request: CreateClientRequest,
) -> Result<CreateClientResult, WorkflowError> {
    // Validate client data
    let validation = call_activity(
        ClientActivities::validate_client_data,
        request.client_data.clone(),
    ).await?;
    
    // Create client record
    let client = call_activity(
        ClientActivities::create_client,
        CreateClientActivityRequest {
            client_data: request.client_data,
            tenant_id: request.tenant_id,
        },
    ).await?;
    
    // Set up client workspace
    call_activity(
        ClientActivities::setup_client_workspace,
        SetupClientWorkspaceRequest {
            client_id: client.id.clone(),
            workspace_template: request.workspace_template,
        },
    ).await?;
    
    // Send welcome notification
    call_activity(
        NotificationActivities::send_client_welcome,
        SendClientWelcomeRequest {
            client_id: client.id.clone(),
            contact_info: client.contact_info,
        },
    ).await?;
    
    Ok(CreateClientResult {
        client_id: client.id,
        workspace_id: client.workspace_id,
    })
}
```

## Module Testing Patterns

### Unit Testing
```rust
#[cfg(test)]
mod module_tests {
    use super::*;
    use temporal_sdk_core_test_utils::TestWorkflowEnvironment;
    
    #[tokio::test]
    async fn test_module_installation() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        let mock_activities = MockModuleActivities::new()
            .expect_validate_module()
            .returning(|_| Ok(ValidationResult::valid()))
            .expect_download_module()
            .returning(|_| Ok(ModulePackage::default()));
        
        let result = test_env.execute_workflow(
            install_module_workflow,
            InstallModuleRequest::default(),
        ).await;
        
        assert!(result.is_ok());
    }
}
```

### Integration Testing
```rust
#[tokio::test]
async fn test_module_end_to_end() {
    let test_env = IntegrationTestEnvironment::new().await;
    
    // Install module
    let install_response = test_env.api_client
        .post("/api/modules/install")
        .json(&InstallModuleRequest::default())
        .send()
        .await
        .unwrap();
    
    assert_eq!(install_response.status(), 200);
    
    // Activate module
    let activate_response = test_env.api_client
        .post("/api/modules/activate")
        .json(&ActivateModuleRequest::default())
        .send()
        .await
        .unwrap();
    
    assert_eq!(activate_response.status(), 200);
    
    // Test module functionality
    let module_response = test_env.api_client
        .post("/api/modules/my-module/workflows/process")
        .json(&ModuleProcessRequest::default())
        .send()
        .await
        .unwrap();
    
    assert_eq!(module_response.status(), 200);
}
```

## Development Guidelines

### Module Development Best Practices
1. **Temporal-First**: Implement all complex operations as Temporal workflows
2. **Sandboxed**: Respect resource limits and permission boundaries
3. **Tenant-Aware**: Support multi-tenant isolation
4. **Testable**: Include comprehensive test suites
5. **Documented**: Provide clear documentation and examples
6. **Versioned**: Use semantic versioning for compatibility
7. **Secure**: Follow security best practices and pass security scans

### Module API Design
1. **RESTful**: Follow REST conventions for HTTP endpoints
2. **Workflow-Driven**: Use workflows for complex operations
3. **Event-Driven**: Emit events for integration with other modules
4. **Cacheable**: Design for caching where appropriate
5. **Paginated**: Support pagination for list operations
6. **Filtered**: Support filtering and searching
7. **Monitored**: Include metrics and logging

This module system provides a comprehensive foundation for extending ADX CORE while maintaining security, reliability, and observability through Temporal workflows.