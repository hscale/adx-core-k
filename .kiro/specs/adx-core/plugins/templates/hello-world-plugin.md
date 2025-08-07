# Hello World Plugin - Development Template

## Overview
The Hello World Plugin is a **minimal template** that demonstrates the basic structure and patterns for ADX CORE plugin development. This serves as the starting point for new plugin developers.

## Plugin Structure
```
hello-world-plugin/
├── Cargo.toml                 # Rust dependencies
├── plugin.toml                # Plugin metadata
├── src/
│   ├── lib.rs                 # Main plugin implementation
│   ├── workflows.rs           # Temporal workflows
│   ├── activities.rs          # Temporal activities
│   ├── handlers.rs            # API handlers
│   └── models.rs              # Data models
├── migrations/
│   └── 001_create_hello_table.sql
├── frontend/
│   ├── components/
│   │   └── HelloWorldWidget.tsx
│   └── hooks/
│       └── useHelloWorld.ts
├── tests/
│   ├── integration_tests.rs
│   └── workflow_tests.rs
└── README.md
```

## Plugin Implementation

### 1. Plugin Metadata (plugin.toml)
```toml
[plugin]
id = "hello-world"
name = "Hello World Plugin"
version = "1.0.0"
description = "A simple example plugin demonstrating basic ADX CORE plugin patterns"
author = "Your Name <your.email@example.com>"
license = "MIT"
category = "example"

[dependencies]
adx-core-sdk = "1.0.0"
temporal-sdk = "0.1.0"
tokio = "1.0"
serde = { version = "1.0", features = ["derive"] }

[plugin.permissions]
required = ["basic_access"]
optional = ["advanced_features"]

[plugin.workflows]
hello_world_workflow = "A simple workflow that demonstrates basic patterns"

[plugin.ui_components]
hello_world_widget = "A basic UI widget for the dashboard"
```

### 2. Main Plugin Implementation (src/lib.rs)
```rust
use adx_core_sdk::prelude::*;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

pub struct HelloWorldPlugin {
    config: HelloWorldConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloWorldConfig {
    pub greeting_message: String,
    pub enable_advanced_features: bool,
}

impl Default for HelloWorldConfig {
    fn default() -> Self {
        Self {
            greeting_message: "Hello, World!".to_string(),
            enable_advanced_features: false,
        }
    }
}

#[async_trait]
impl AdxPlugin for HelloWorldPlugin {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            id: "hello-world".to_string(),
            name: "Hello World Plugin".to_string(),
            version: "1.0.0".to_string(),
            description: "A simple example plugin demonstrating basic ADX CORE plugin patterns".to_string(),
            author: "Plugin Developer".to_string(),
            category: PluginCategory::Example,
            is_first_party: false,
            price: None, // Free plugin
            permissions: vec![Permission::BasicAccess],
            dependencies: vec![],
            temporal_workflows: vec!["hello_world_workflow".to_string()],
        }
    }

    async fn activate(&self, context: &PluginContext) -> Result<(), PluginError> {
        // Register UI components
        context.register_ui_component(
            "hello-world-widget",
            UiComponent::new("HelloWorldWidget", include_str!("../frontend/components/HelloWorldWidget.tsx"))
        ).await?;

        // Register API routes
        context.register_routes("/api/plugins/hello-world", self.create_routes()).await?;

        // Register Temporal workflows
        context.register_workflow("hello_world_workflow", hello_world_workflow).await?;

        // Run database migrations
        context.run_migrations(self.get_migrations()).await?;

        // Register event handlers
        context.register_event_handler("user.login", self.on_user_login).await?;

        println!("Hello World Plugin activated successfully!");
        Ok(())
    }

    async fn deactivate(&self) -> Result<(), PluginError> {
        println!("Hello World Plugin deactivated");
        Ok(())
    }

    async fn uninstall(&self) -> Result<(), PluginError> {
        println!("Hello World Plugin uninstalled");
        Ok(())
    }

    fn register_database_migrations(&self) -> Vec<Migration> {
        vec![
            Migration {
                version: "001_create_hello_table".to_string(),
                sql: include_str!("../migrations/001_create_hello_table.sql").to_string(),
            }
        ]
    }
}

impl HelloWorldPlugin {
    pub fn new(config: HelloWorldConfig) -> Self {
        Self { config }
    }

    fn create_routes(&self) -> Router<AppState> {
        Router::new()
            .route("/hello", get(hello_handler))
            .route("/greet/:name", get(greet_handler))
            .route("/workflow", post(start_hello_workflow))
    }

    async fn on_user_login(&self, event: &Event) -> Result<(), PluginError> {
        if let Some(user_data) = event.data.as_object() {
            println!("User logged in: {:?}", user_data.get("user_id"));
        }
        Ok(())
    }
}
```

### 3. Temporal Workflows (src/workflows.rs)
```rust
use temporal_sdk::{workflow, WorkflowResult};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloWorldRequest {
    pub name: String,
    pub message: Option<String>,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloWorldResponse {
    pub greeting: String,
    pub processed_at: chrono::DateTime<chrono::Utc>,
    pub workflow_id: String,
}

/// Simple workflow that demonstrates basic Temporal patterns
#[workflow]
pub async fn hello_world_workflow(
    request: HelloWorldRequest,
) -> WorkflowResult<HelloWorldResponse> {
    // Step 1: Validate input
    validate_hello_request_activity(request.clone()).await?;
    
    // Step 2: Generate personalized greeting
    let greeting = generate_greeting_activity(
        request.name.clone(),
        request.message.clone(),
    ).await?;
    
    // Step 3: Log the interaction
    log_hello_interaction_activity(
        request.tenant_id.clone(),
        request.name.clone(),
        greeting.clone(),
    ).await?;
    
    // Step 4: Send notification (if configured)
    if should_send_notification().await? {
        send_hello_notification_activity(
            request.name.clone(),
            greeting.clone(),
        ).await?;
    }
    
    Ok(HelloWorldResponse {
        greeting,
        processed_at: temporal_sdk::now(),
        workflow_id: temporal_sdk::workflow_id(),
    })
}

/// Example of a workflow with timeout and error handling
#[workflow]
pub async fn hello_world_with_timeout_workflow(
    request: HelloWorldRequest,
) -> WorkflowResult<HelloWorldResponse> {
    // Use select! for timeout handling
    let result = temporal_sdk::select! {
        response = hello_world_workflow(request.clone()) => response?,
        _ = temporal_sdk::sleep(Duration::from_minutes(5)) => {
            return Err(WorkflowError::Timeout("Hello world workflow timed out".to_string()));
        }
    };
    
    Ok(result)
}
```

### 4. Temporal Activities (src/activities.rs)
```rust
use temporal_sdk::{activity, ActivityError};
use serde::{Deserialize, Serialize};

#[activity]
pub async fn validate_hello_request_activity(
    request: HelloWorldRequest,
) -> Result<(), ActivityError> {
    if request.name.trim().is_empty() {
        return Err(ActivityError::InvalidInput("Name cannot be empty".to_string()));
    }
    
    if request.name.len() > 100 {
        return Err(ActivityError::InvalidInput("Name too long".to_string()));
    }
    
    Ok(())
}

#[activity]
pub async fn generate_greeting_activity(
    name: String,
    custom_message: Option<String>,
) -> Result<String, ActivityError> {
    let greeting = match custom_message {
        Some(msg) => format!("{}, {}!", msg, name),
        None => format!("Hello, {}! Welcome to ADX CORE!", name),
    };
    
    Ok(greeting)
}

#[activity]
pub async fn log_hello_interaction_activity(
    tenant_id: String,
    name: String,
    greeting: String,
) -> Result<(), ActivityError> {
    // In a real plugin, this would write to database
    println!("Logged interaction for tenant {}: {} -> {}", tenant_id, name, greeting);
    
    // Simulate database write
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    Ok(())
}

#[activity]
pub async fn send_hello_notification_activity(
    name: String,
    greeting: String,
) -> Result<(), ActivityError> {
    // In a real plugin, this would send actual notification
    println!("Sending notification: {} to {}", greeting, name);
    
    // Simulate notification sending
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    
    Ok(())
}

#[activity]
pub async fn should_send_notification_activity() -> Result<bool, ActivityError> {
    // Simple logic - in real plugin this might check user preferences
    Ok(true)
}
```

### 5. API Handlers (src/handlers.rs)
```rust
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::get,
};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct HelloResponse {
    message: String,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Deserialize)]
pub struct StartWorkflowRequest {
    name: String,
    message: Option<String>,
}

/// Simple hello endpoint
pub async fn hello_handler() -> Json<HelloResponse> {
    Json(HelloResponse {
        message: "Hello from Hello World Plugin!".to_string(),
        timestamp: chrono::Utc::now(),
    })
}

/// Personalized greeting endpoint
pub async fn greet_handler(
    Path(name): Path<String>,
) -> Json<HelloResponse> {
    Json(HelloResponse {
        message: format!("Hello, {}! This is from the Hello World Plugin.", name),
        timestamp: chrono::Utc::now(),
    })
}

/// Start hello world workflow
pub async fn start_hello_workflow(
    State(state): State<AppState>,
    Json(request): Json<StartWorkflowRequest>,
) -> Result<Json<HelloWorldResponse>, (StatusCode, String)> {
    let workflow_id = format!("hello-world-{}", uuid::Uuid::new_v4());
    
    let response = state.temporal_client
        .start_workflow(
            workflow_id,
            hello_world_workflow,
            HelloWorldRequest {
                name: request.name,
                message: request.message,
                tenant_id: state.current_tenant_id.clone(),
            }
        )
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    
    Ok(Json(response))
}
```

### 6. Frontend Component (frontend/components/HelloWorldWidget.tsx)
```typescript
import React, { useState } from 'react';
import { useTemporalWorkflow } from '@adx-core/react-hooks';

interface HelloWorldWidgetProps {
  tenantId: string;
}

export const HelloWorldWidget: React.FC<HelloWorldWidgetProps> = ({ tenantId }) => {
  const [name, setName] = useState('');
  const [customMessage, setCustomMessage] = useState('');
  const { execute, status, result, error } = useTemporalWorkflow<HelloWorldResponse>('hello-world');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    
    if (!name.trim()) {
      alert('Please enter a name');
      return;
    }

    await execute({
      name: name.trim(),
      message: customMessage.trim() || undefined,
      tenant_id: tenantId,
    });
  };

  return (
    <div className="hello-world-widget">
      <h3>Hello World Plugin</h3>
      
      <form onSubmit={handleSubmit} className="hello-form">
        <div className="form-group">
          <label htmlFor="name">Name:</label>
          <input
            id="name"
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Enter your name"
            disabled={status === 'running'}
          />
        </div>
        
        <div className="form-group">
          <label htmlFor="message">Custom Message (optional):</label>
          <input
            id="message"
            type="text"
            value={customMessage}
            onChange={(e) => setCustomMessage(e.target.value)}
            placeholder="Custom greeting message"
            disabled={status === 'running'}
          />
        </div>
        
        <button 
          type="submit" 
          disabled={status === 'running'}
          className="btn-primary"
        >
          {status === 'running' ? 'Processing...' : 'Say Hello'}
        </button>
      </form>
      
      {status === 'running' && (
        <div className="status-message">
          Processing your hello world request...
        </div>
      )}
      
      {status === 'completed' && result && (
        <div className="success-message">
          <h4>Success!</h4>
          <p>{result.greeting}</p>
          <small>Processed at: {new Date(result.processed_at).toLocaleString()}</small>
        </div>
      )}
      
      {status === 'failed' && error && (
        <div className="error-message">
          <h4>Error</h4>
          <p>{error}</p>
        </div>
      )}
    </div>
  );
};

export default HelloWorldWidget;
```

### 7. Database Migration (migrations/001_create_hello_table.sql)
```sql
-- Hello World Plugin Database Migration
CREATE TABLE plugin_hello_interactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    greeting TEXT NOT NULL,
    custom_message TEXT,
    workflow_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    INDEX idx_hello_interactions_tenant (tenant_id),
    INDEX idx_hello_interactions_created (created_at)
);

-- Add some sample data for testing
INSERT INTO plugin_hello_interactions (tenant_id, name, greeting, custom_message) VALUES
    ('00000000-0000-0000-0000-000000000001', 'Test User', 'Hello, Test User! Welcome to ADX CORE!', NULL),
    ('00000000-0000-0000-0000-000000000001', 'Demo User', 'Welcome, Demo User!', 'Welcome');
```

### 8. Tests (tests/workflow_tests.rs)
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use temporal_sdk::testing::WorkflowTestEnv;

    #[tokio::test]
    async fn test_hello_world_workflow() {
        let mut env = WorkflowTestEnv::new().await;
        
        // Register activities
        env.register_activity(validate_hello_request_activity);
        env.register_activity(generate_greeting_activity);
        env.register_activity(log_hello_interaction_activity);
        env.register_activity(send_hello_notification_activity);
        env.register_activity(should_send_notification_activity);
        
        // Execute workflow
        let result = env.execute_workflow(
            hello_world_workflow,
            HelloWorldRequest {
                name: "Test User".to_string(),
                message: Some("Welcome".to_string()),
                tenant_id: "test-tenant".to_string(),
            }
        ).await;
        
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.greeting, "Welcome, Test User!");
    }

    #[tokio::test]
    async fn test_hello_world_workflow_validation_error() {
        let mut env = WorkflowTestEnv::new().await;
        
        env.register_activity(validate_hello_request_activity);
        
        // Test with empty name
        let result = env.execute_workflow(
            hello_world_workflow,
            HelloWorldRequest {
                name: "".to_string(),
                message: None,
                tenant_id: "test-tenant".to_string(),
            }
        ).await;
        
        assert!(result.is_err());
    }
}
```

## Key Learning Points

### 1. **Plugin Structure**
- Clear separation of concerns (workflows, activities, handlers)
- Proper error handling and validation
- Database migrations for plugin-specific data
- Frontend components with Temporal integration

### 2. **Temporal Integration**
- Simple workflow with multiple activities
- Proper error handling and timeout management
- Activity idempotency and retry safety
- Workflow testing with replay capability

### 3. **Platform Integration**
- Plugin registration and lifecycle management
- UI component registration and rendering
- API route registration and handling
- Event handling and system integration

### 4. **Best Practices**
- Input validation and sanitization
- Proper tenant isolation
- Comprehensive testing including workflow replay
- Clear documentation and examples

This Hello World Plugin provides a **complete template** for developers to start building their own ADX CORE plugins with confidence!