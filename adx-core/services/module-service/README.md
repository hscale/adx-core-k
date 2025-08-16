# ADX Core Module Service

A comprehensive module system with Temporal workflows for reliable module lifecycle management, marketplace integration, sandboxing, and security scanning.

## Features

### Core Module System
- **Trait-based Architecture**: Extensible module system with comprehensive trait definitions
- **Hot-loading**: Dynamic module loading and reloading without system restart
- **Dependency Resolution**: Automatic dependency resolution and version compatibility checking
- **Multi-language Support**: Support for Rust, JavaScript, Python, and WebAssembly modules

### Temporal Workflow Integration
- **Reliable Operations**: All complex module operations implemented as Temporal workflows
- **Automatic Retry**: Built-in retry and compensation logic for failed operations
- **Rollback Capabilities**: Automatic rollback on installation/update failures
- **Progress Tracking**: Real-time progress tracking for long-running operations

### Marketplace Integration
- **Module Discovery**: Search and browse modules with advanced filtering
- **Payment Processing**: Integrated payment processing with multiple providers
- **Reviews and Ratings**: Community-driven module reviews and ratings
- **Recommendations**: AI-powered module recommendations based on usage patterns

### Security and Sandboxing
- **Multi-level Isolation**: Process, container, and WASM-based sandboxing
- **Security Scanning**: Comprehensive security scanning with vulnerability detection
- **Resource Limits**: Configurable resource limits and monitoring
- **Permission System**: Fine-grained permission system for module capabilities

### Development SDK
- **Module SDK**: Comprehensive SDK for module development
- **Extension Points**: Multiple extension points for UI, API, workflows, and database
- **Development Tools**: Built-in logging, configuration, storage, and HTTP utilities
- **Cross-platform Support**: Support for web, desktop, and mobile platforms

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Module Service Architecture                  │
├─────────────────────────────────────────────────────────────────┤
│  HTTP API Layer                                                │
│  ├─ Module Management (Install/Update/Uninstall)               │
│  ├─ Marketplace Integration (Search/Purchase/Reviews)          │
│  ├─ Workflow Status Tracking                                   │
│  └─ Health and Monitoring                                      │
├─────────────────────────────────────────────────────────────────┤
│  Temporal Workflow Layer                                       │
│  ├─ Installation Workflows (with rollback)                     │
│  ├─ Update Workflows (with compatibility checks)               │
│  ├─ Uninstallation Workflows (with cleanup)                    │
│  └─ Marketplace Sync Workflows                                 │
├─────────────────────────────────────────────────────────────────┤
│  Module Management Layer                                       │
│  ├─ Module Manager (lifecycle management)                      │
│  ├─ Dependency Resolver (version compatibility)                │
│  ├─ Loader Registry (multi-language support)                   │
│  └─ Event Bus (inter-module communication)                     │
├─────────────────────────────────────────────────────────────────┤
│  Security and Sandboxing Layer                                 │
│  ├─ Security Scanner (vulnerability detection)                 │
│  ├─ Sandbox Manager (isolation enforcement)                    │
│  ├─ Resource Monitor (usage tracking)                          │
│  └─ Permission Enforcer (access control)                       │
├─────────────────────────────────────────────────────────────────┤
│  Storage and Marketplace Layer                                 │
│  ├─ Module Repository (PostgreSQL)                             │
│  ├─ Marketplace Client (external API)                          │
│  ├─ Payment Processor (multi-provider)                         │
│  └─ Analytics Engine (usage tracking)                          │
└─────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.88+
- PostgreSQL 14+
- Redis 6+
- Temporal Server
- Docker (for container sandboxing)

### Installation

1. Clone the repository:
```bash
git clone https://github.com/adxcore/adx-core.git
cd adx-core/services/module-service
```

2. Install dependencies:
```bash
cargo build
```

3. Set up the database:
```bash
# Create database
createdb adx_core_modules

# Run migrations (handled automatically on startup)
```

4. Configure the service:
```bash
cp config/default.toml config/local.toml
# Edit config/local.toml with your settings
```

5. Start the service:
```bash
# HTTP server mode
cargo run

# Temporal worker mode
cargo run -- worker
```

### Configuration

The service can be configured via environment variables or TOML files:

```toml
[server]
host = "0.0.0.0"
port = 8086
max_connections = 1000

[database]
url = "postgresql://localhost:5432/adx_core_modules"
max_connections = 20

[temporal]
server_url = "localhost:7233"
namespace = "adx-core"
task_queue = "module-service"

[marketplace]
base_url = "https://marketplace.adxcore.com"
api_key = "your-api-key"
enable_analytics = true

[sandbox]
default_isolation_level = "process"
max_sandboxes = 100
enable_wasm = true
enable_containers = true

[security]
enable_security_scanning = true
min_security_score = 70
```

## API Reference

### Module Management

#### Install Module
```http
POST /api/v1/modules/install
Content-Type: application/json

{
  "module_id": "example-module",
  "version": "1.0.0",
  "tenant_id": "tenant-123",
  "user_id": "user-456",
  "configuration": {},
  "auto_activate": true
}
```

#### Update Module
```http
PUT /api/v1/modules/{instance_id}/update
Content-Type: application/json

{
  "target_version": "1.1.0",
  "preserve_config": true,
  "backup_current": true
}
```

#### Uninstall Module
```http
DELETE /api/v1/modules/{instance_id}/uninstall
Content-Type: application/json

{
  "cleanup_data": true,
  "backup_data": true
}
```

#### List Tenant Modules
```http
GET /api/v1/tenants/{tenant_id}/modules
```

### Marketplace

#### Search Modules
```http
POST /api/v1/marketplace/search
Content-Type: application/json

{
  "query": "analytics",
  "categories": ["Analytics", "BusinessManagement"],
  "sort_by": "Rating",
  "limit": 20,
  "offset": 0
}
```

#### Get Featured Modules
```http
GET /api/v1/marketplace/featured
```

#### Purchase Module
```http
POST /api/v1/marketplace/purchase
Content-Type: application/json

{
  "module_id": "premium-analytics",
  "version": "2.0.0",
  "tenant_id": "tenant-123",
  "user_id": "user-456",
  "payment_method": {
    "CreditCard": {
      "token": "card-token-123"
    }
  }
}
```

### Workflow Operations

#### Install Module (Workflow)
```http
POST /api/v1/workflows/install-module
Content-Type: application/json

{
  "module_id": "complex-module",
  "tenant_id": "tenant-123",
  "user_id": "user-456",
  "auto_activate": true
}
```

Response for long-running operations:
```json
{
  "type": "Asynchronous",
  "operation_id": "workflow-123",
  "status_url": "/api/v1/workflows/workflow-123/status",
  "estimated_duration_seconds": 300
}
```

#### Check Workflow Status
```http
GET /api/v1/workflows/{operation_id}/status
```

## Module Development

### Creating a Module

1. **Define Module Metadata**:
```rust
use adx_module_sdk::*;

let metadata = ModuleMetadata {
    id: "my-awesome-module".to_string(),
    name: "My Awesome Module".to_string(),
    version: Version::new(1, 0, 0),
    description: "An awesome module for ADX Core".to_string(),
    author: ModuleAuthor {
        name: "Your Name".to_string(),
        email: Some("you@example.com".to_string()),
        // ...
    },
    // ...
};
```

2. **Implement Module Trait**:
```rust
use adx_module_sdk::*;

pub struct MyAwesomeModule {
    base: BaseModule,
}

#[async_trait]
impl AdxModule for MyAwesomeModule {
    async fn initialize(&mut self, config: Value) -> ModuleResult<()> {
        self.base.sdk().logger.info("Initializing My Awesome Module");
        
        // Custom initialization logic
        self.setup_database().await?;
        self.register_workflows().await?;
        
        self.base.initialize(config).await
    }

    async fn start(&mut self) -> ModuleResult<()> {
        self.base.sdk().logger.info("Starting My Awesome Module");
        
        // Start background tasks
        self.start_background_processor().await?;
        
        self.base.start().await
    }

    // Implement other required methods...
}
```

3. **Add Extension Points**:
```rust
impl MyAwesomeModule {
    pub fn new() -> Self {
        let mut module = Self {
            base: BaseModule::new(metadata, manifest),
        };

        // Register UI components
        module.base.sdk_mut().ui
            .add_page("/my-module", UIComponent {
                name: "MyModulePage".to_string(),
                component_type: UIComponentType::Page,
                props: HashMap::new(),
                permissions: vec!["module:my-module:read".to_string()],
            })
            .add_widget("my-widget", UIComponent {
                name: "MyWidget".to_string(),
                component_type: UIComponentType::Widget,
                props: HashMap::new(),
                permissions: vec![],
            });

        // Register workflows
        module.base.sdk_mut().workflows
            .add_workflow("process_data", WorkflowDefinition {
                name: "process_data".to_string(),
                description: "Process module data".to_string(),
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                timeout_seconds: 300,
                retry_policy: RetryPolicy {
                    max_attempts: 3,
                    initial_interval_seconds: 1,
                    backoff_coefficient: 2.0,
                    max_interval_seconds: 60,
                },
            });

        module
    }
}
```

4. **Build and Package**:
```bash
# Build the module
cargo build --release

# Create module package
adx-module-cli package --manifest manifest.json --output my-awesome-module.adx
```

### Module SDK Features

The Module SDK provides comprehensive utilities for module development:

#### Logging
```rust
// Structured logging with module context
self.sdk().logger.info("Processing user data");
self.sdk().logger.warn("Rate limit approaching");
self.sdk().logger.error("Failed to connect to external service");
```

#### Configuration Management
```rust
// Get configuration values
let api_key: String = self.sdk().config.get_typed("api_key").await?
    .ok_or_else(|| ModuleError::ConfigurationError("API key required".to_string()))?;

// Set configuration values
self.sdk().config.set_typed("last_sync", chrono::Utc::now()).await?;
```

#### Storage
```rust
// Store module data
let data = serde_json::to_vec(&my_data)?;
self.sdk().storage.store("user_preferences", &data).await?;

// Retrieve module data
if let Some(data) = self.sdk().storage.retrieve("user_preferences").await? {
    let preferences: UserPreferences = serde_json::from_slice(&data)?;
}
```

#### HTTP Client
```rust
// Make HTTP requests with built-in security and rate limiting
let response = self.sdk().http.get("https://api.example.com/data").await?;
let data: ApiResponse = response.json().await?;

// POST with JSON body
let response = self.sdk().http.post("https://api.example.com/webhook", 
    serde_json::json!({"event": "user_created", "user_id": user_id})
).await?;
```

#### Event Bus
```rust
// Emit events
self.sdk().events.emit("user:created", serde_json::json!({
    "user_id": user_id,
    "tenant_id": tenant_id
})).await?;

// Subscribe to events
self.sdk().events.subscribe("tenant:switched", Box::new(|data| {
    // Handle tenant switch event
    Ok(())
})).await?;
```

## Security

### Sandboxing

The module system provides multiple levels of isolation:

1. **No Isolation**: Direct execution (for trusted modules)
2. **Process Isolation**: Separate process with resource limits
3. **Container Isolation**: Docker container with network and filesystem restrictions
4. **WASM Isolation**: WebAssembly runtime with capability-based security

### Security Scanning

All modules undergo comprehensive security scanning:

- **Static Analysis**: Code analysis for vulnerabilities and malicious patterns
- **Dependency Scanning**: Known vulnerability detection in dependencies
- **Malware Detection**: Signature-based malware detection
- **Configuration Analysis**: Security policy validation

### Permission System

Modules must declare required permissions:

```json
{
  "permissions": [
    "database:read:user_data",
    "database:write:module_data",
    "network:access:api.example.com",
    "file:read:/tmp/module_cache",
    "workflow:execute:data_processing"
  ]
}
```

## Monitoring and Observability

### Health Checks

```http
GET /health
```

Response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0"
}
```

### Module Health

```http
GET /api/v1/modules/{instance_id}/health
```

### Resource Usage

```http
GET /api/v1/modules/{instance_id}/resources
```

### Metrics

The service exposes Prometheus metrics on port 9090:

- `module_installations_total`
- `module_execution_duration_seconds`
- `module_resource_usage`
- `security_scan_results`
- `workflow_executions_total`

## Development

### Running Tests

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test integration_tests

# Workflow tests
cargo test --test workflow_tests

# All tests
cargo test
```

### Development Mode

```bash
# Start with hot reloading
cargo watch -x run

# Start with debug logging
RUST_LOG=debug cargo run
```

### Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Support

- Documentation: https://docs.adxcore.com/modules
- Issues: https://github.com/adxcore/adx-core/issues
- Discussions: https://github.com/adxcore/adx-core/discussions
- Email: support@adxcore.com