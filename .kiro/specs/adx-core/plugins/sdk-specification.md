# ADX CORE Plugin SDK Specification

## Overview
The ADX CORE Plugin SDK provides developers with tools, APIs, and templates to build plugins that integrate seamlessly with the platform's Temporal-First architecture.

## SDK Components

### 1. Core SDK (Rust)
```rust
// Main SDK crate
[dependencies]
adx-core-sdk = "1.0.0"

// SDK modules
use adx_core_sdk::{
    plugin::{AdxPlugin, PluginContext, PluginMetadata},
    temporal::{workflow, activity, WorkflowResult, ActivityError},
    api::{Router, AppState, ApiError},
    database::{Migration, Repository},
    ui::{UiComponent, ReactComponent},
    events::{Event, EventHandler},
    permissions::{Permission, PermissionCheck},
    tenant::{TenantContext, TenantId},
};
```

### 2. Frontend SDK (TypeScript)
```typescript
// React hooks and components
import {
  useTemporalWorkflow,
  usePluginContext,
  useTenantContext,
  PluginProvider,
  WorkflowStatus,
} from '@adx-core/react-sdk';

// Plugin development utilities
import {
  createPluginComponent,
  registerPluginRoute,
  usePluginApi,
} from '@adx-core/plugin-sdk';
```

### 3. CLI Tools
```bash
# Plugin development CLI
adx-core create-plugin <name>     # Create new plugin
adx-core dev                      # Start development server
adx-core test                     # Run plugin tests
adx-core build                    # Build plugin for production
adx-core publish                  # Publish to marketplace
```

## Plugin API Reference

### Core Plugin Trait
```rust
#[async_trait]
pub trait AdxPlugin: Send + Sync {
    /// Plugin metadata and configuration
    fn metadata(&self) -> PluginMetadata;
    
    /// Called when plugin is activated
    async fn activate(&self, context: &PluginContext) -> Result<(), PluginError>;
    
    /// Called when plugin is deactivated
    async fn deactivate(&self) -> Result<(), PluginError>;
    
    /// Called when plugin is uninstalled
    async fn uninstall(&self) -> Result<(), PluginError>;
    
    /// Register database migrations
    fn register_database_migrations(&self) -> Vec<Migration> { vec![] }
    
    /// Register API routes
    fn register_routes(&self) -> Vec<PluginRoute> { vec![] }
    
    /// Register UI components
    fn register_ui_components(&self) -> Vec<UiComponent> { vec![] }
    
    /// Register Temporal workflows
    fn register_workflows(&self) -> Vec<WorkflowDefinition> { vec![] }
    
    /// Register event handlers
    fn register_event_handlers(&self) -> Vec<EventHandler> { vec![] }
}
```

### Plugin Context
```rust
pub struct PluginContext {
    pub tenant_id: TenantId,
    pub plugin_id: String,
    pub config: PluginConfig,
    pub temporal_client: TemporalClient,
    pub database: DatabaseConnection,
    pub event_bus: EventBus,
}

impl PluginContext {
    /// Register a UI component
    pub async fn register_ui_component(
        &self,
        name: &str,
        component: UiComponent,
    ) -> Result<(), PluginError>;
    
    /// Register API routes
    pub async fn register_routes(
        &self,
        prefix: &str,
        router: Router<AppState>,
    ) -> Result<(), PluginError>;
    
    /// Register Temporal workflow
    pub async fn register_workflow<W>(
        &self,
        name: &str,
        workflow: W,
    ) -> Result<(), PluginError>
    where
        W: TemporalWorkflow + Send + Sync + 'static;
    
    /// Run database migrations
    pub async fn run_migrations(
        &self,
        migrations: Vec<Migration>,
    ) -> Result<(), PluginError>;
    
    /// Register event handler
    pub async fn register_event_handler<H>(
        &self,
        event_type: &str,
        handler: H,
    ) -> Result<(), PluginError>
    where
        H: EventHandler + Send + Sync + 'static;
}
```

## Temporal Integration

### Workflow Macros
```rust
// Workflow definition
#[workflow]
pub async fn my_plugin_workflow(
    input: MyWorkflowInput,
) -> WorkflowResult<MyWorkflowOutput> {
    // Workflow implementation
}

// Activity definition
#[activity]
pub async fn my_plugin_activity(
    input: MyActivityInput,
) -> Result<MyActivityOutput, ActivityError> {
    // Activity implementation
}
```

### Workflow Utilities
```rust
// Temporal SDK utilities for plugins
use adx_core_sdk::temporal::{
    // Workflow execution
    temporal_sdk,
    WorkflowResult,
    WorkflowError,
    
    // Activity execution
    ActivityError,
    ActivityResult,
    
    // Time and scheduling
    Duration,
    sleep,
    sleep_until,
    now,
    
    // Parallel execution
    join,
    select,
    
    // Signals and queries
    signal,
    query,
    
    // Child workflows
    start_child_workflow,
    execute_child_workflow,
};
```

## Database Integration

### Migration System
```rust
pub struct Migration {
    pub version: String,
    pub sql: String,
    pub rollback_sql: Option<String>,
}

// Example migration
Migration {
    version: "001_create_my_table".to_string(),
    sql: r#"
        CREATE TABLE plugin_my_data (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            tenant_id UUID NOT NULL,
            data JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
    "#.to_string(),
    rollback_sql: Some("DROP TABLE plugin_my_data;".to_string()),
}
```

### Repository Pattern
```rust
#[async_trait]
pub trait MyPluginRepository: Send + Sync {
    async fn create(&self, data: MyData) -> Result<MyData, DatabaseError>;
    async fn get_by_id(&self, id: Uuid) -> Result<Option<MyData>, DatabaseError>;
    async fn list_by_tenant(&self, tenant_id: TenantId) -> Result<Vec<MyData>, DatabaseError>;
    async fn update(&self, id: Uuid, data: MyData) -> Result<MyData, DatabaseError>;
    async fn delete(&self, id: Uuid) -> Result<(), DatabaseError>;
}
```

## Frontend Integration

### React Hooks
```typescript
// Temporal workflow hook
export const useTemporalWorkflow = <T>(workflowType: string) => {
  const [status, setStatus] = useState<WorkflowStatus>('idle');
  const [result, setResult] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);

  const execute = async (data: any) => {
    // Implementation
  };

  return { execute, status, result, error };
};

// Plugin context hook
export const usePluginContext = () => {
  const context = useContext(PluginContext);
  if (!context) {
    throw new Error('usePluginContext must be used within PluginProvider');
  }
  return context;
};

// Tenant context hook
export const useTenantContext = () => {
  const context = useContext(TenantContext);
  return context;
};
```

### Component Registration
```typescript
// Register React component
export const createPluginComponent = (
  name: string,
  component: React.ComponentType<any>
) => {
  return {
    name,
    component,
    type: 'react',
  };
};

// Example usage
const MyPluginWidget = createPluginComponent(
  'my-plugin-widget',
  ({ tenantId, config }) => {
    const { execute, status, result } = useTemporalWorkflow('my-workflow');
    
    return (
      <div className="my-plugin-widget">
        {/* Component implementation */}
      </div>
    );
  }
);
```

## Event System

### Event Handling
```rust
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<(), EventError>;
}

// Example event handler
pub struct MyEventHandler;

#[async_trait]
impl EventHandler for MyEventHandler {
    async fn handle(&self, event: &Event) -> Result<(), EventError> {
        match event.event_type.as_str() {
            "user.created" => {
                // Handle user creation
                println!("New user created: {:?}", event.data);
            }
            "file.uploaded" => {
                // Handle file upload
                println!("File uploaded: {:?}", event.data);
            }
            _ => {
                // Ignore unknown events
            }
        }
        Ok(())
    }
}
```

### Event Publishing
```rust
// Publish events from plugins
impl PluginContext {
    pub async fn publish_event(
        &self,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<(), EventError> {
        let event = Event {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            data,
            tenant_id: self.tenant_id,
            plugin_id: Some(self.plugin_id.clone()),
            timestamp: Utc::now(),
        };
        
        self.event_bus.publish(event).await
    }
}
```

## Security and Permissions

### Permission System
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Permission {
    // Basic permissions
    BasicAccess,
    ReadData,
    WriteData,
    DeleteData,
    
    // Advanced permissions
    AdminAccess,
    SystemConfig,
    UserManagement,
    TenantManagement,
    
    // Plugin-specific permissions
    Custom(String),
}

// Permission checking
impl PluginContext {
    pub async fn check_permission(
        &self,
        permission: Permission,
    ) -> Result<bool, PermissionError> {
        // Implementation
    }
    
    pub async fn require_permission(
        &self,
        permission: Permission,
    ) -> Result<(), PermissionError> {
        if !self.check_permission(permission).await? {
            return Err(PermissionError::AccessDenied);
        }
        Ok(())
    }
}
```

## Testing Framework

### Workflow Testing
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use adx_core_sdk::testing::{WorkflowTestEnv, MockPluginContext};

    #[tokio::test]
    async fn test_my_workflow() {
        let mut env = WorkflowTestEnv::new().await;
        
        // Register activities
        env.register_activity(my_activity);
        
        // Execute workflow
        let result = env.execute_workflow(
            my_workflow,
            MyWorkflowInput { /* test data */ }
        ).await;
        
        assert!(result.is_ok());
    }
}
```

### Integration Testing
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use adx_core_sdk::testing::TestHarness;

    #[tokio::test]
    async fn test_plugin_lifecycle() {
        let harness = TestHarness::new().await;
        let plugin = MyPlugin::new();
        
        // Test activation
        let result = plugin.activate(&harness.context).await;
        assert!(result.is_ok());
        
        // Test functionality
        // ...
        
        // Test deactivation
        let result = plugin.deactivate().await;
        assert!(result.is_ok());
    }
}
```

This SDK specification provides developers with everything they need to build robust, secure, and well-integrated plugins for the ADX CORE platform.