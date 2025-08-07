# ADX CORE Plugin Development Guide

## Overview
This guide provides comprehensive instructions for developing plugins for the ADX CORE platform using the Temporal-First architecture.

## Getting Started

### Prerequisites
- Rust 1.70+ with Cargo
- Node.js 18+ for frontend development
- ADX CORE SDK installed
- Temporal.io development environment
- Basic understanding of Temporal workflows

### Development Environment Setup
```bash
# Install ADX CORE SDK
cargo install adx-core-sdk

# Create new plugin project
adx-core create-plugin my-awesome-plugin

# Start development environment
cd my-awesome-plugin
adx-core dev
```

## Plugin Architecture

### Core Principles
1. **Temporal-First**: All complex operations MUST use Temporal workflows
2. **Multi-Tenant**: Proper tenant isolation and context handling
3. **Security**: Input validation and permission checking
4. **Testing**: Comprehensive tests including workflow replay
5. **Documentation**: Clear docs with examples

### Plugin Structure
```
my-plugin/
├── Cargo.toml                 # Rust dependencies
├── plugin.toml                # Plugin metadata
├── src/
│   ├── lib.rs                 # Main plugin implementation
│   ├── workflows.rs           # Temporal workflows
│   ├── activities.rs          # Temporal activities
│   ├── handlers.rs            # API handlers
│   └── models.rs              # Data models
├── migrations/                # Database migrations
├── frontend/                  # React components
├── tests/                     # Tests including workflow tests
└── README.md                  # Plugin documentation
```

## Development Workflow

### 1. Plugin Metadata
Define your plugin in `plugin.toml`:
```toml
[plugin]
id = "my-awesome-plugin"
name = "My Awesome Plugin"
version = "1.0.0"
description = "An awesome plugin that does amazing things"
author = "Your Name <email@example.com>"
category = "business"

[plugin.permissions]
required = ["basic_access"]
optional = ["advanced_features"]
```

### 2. Implement Plugin Trait
```rust
#[async_trait]
impl AdxPlugin for MyAwesomePlugin {
    fn metadata(&self) -> PluginMetadata { /* ... */ }
    async fn activate(&self, context: &PluginContext) -> Result<(), PluginError> { /* ... */ }
    async fn deactivate(&self) -> Result<(), PluginError> { /* ... */ }
    async fn uninstall(&self) -> Result<(), PluginError> { /* ... */ }
}
```

### 3. Create Temporal Workflows
```rust
#[workflow]
pub async fn my_workflow(input: MyInput) -> WorkflowResult<MyOutput> {
    // Step 1: Validate input
    validate_input_activity(input.clone()).await?;
    
    // Step 2: Process data
    let result = process_data_activity(input.data).await?;
    
    // Step 3: Send notification
    send_notification_activity(result.clone()).await?;
    
    Ok(MyOutput { result })
}
```

## Best Practices

### Temporal Workflows
- Keep workflows deterministic
- Use activities for external calls
- Handle timeouts with `temporal_sdk::select!`
- Implement proper error handling
- Write workflow replay tests

### Security
- Validate all inputs
- Check permissions before operations
- Use tenant context for data isolation
- Sanitize user-provided data
- Log security-relevant events

### Performance
- Make activities idempotent
- Use parallel execution with `temporal_sdk::join!`
- Implement proper caching
- Optimize database queries
- Monitor resource usage

## Testing

### Workflow Tests
```rust
#[tokio::test]
async fn test_my_workflow() {
    let mut env = WorkflowTestEnv::new().await;
    env.register_activity(validate_input_activity);
    env.register_activity(process_data_activity);
    
    let result = env.execute_workflow(my_workflow, input).await;
    assert!(result.is_ok());
}
```

### Integration Tests
- Test API endpoints
- Test database operations
- Test UI components
- Test plugin lifecycle

## Publishing

### Plugin Marketplace
1. Complete security review
2. Performance testing
3. Documentation review
4. Submit to marketplace
5. Revenue sharing: 70% developer, 30% platform

### Quality Standards
- Security audit passed
- Performance benchmarks met
- Comprehensive documentation
- User experience review
- Multi-version compatibility

This guide provides the foundation for building high-quality plugins for the ADX CORE ecosystem.