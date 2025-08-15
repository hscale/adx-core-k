# ADX Core Module Service

A comprehensive module management system with Temporal workflows, marketplace integration, and advanced security features.

## Features

### Core Module Management
- **Hot-loading**: Modules can be loaded and unloaded without system restart
- **Dependency Resolution**: Automatic dependency management with version compatibility
- **Version Management**: Support for multiple module versions with migration capabilities
- **Tenant Isolation**: Complete isolation of modules per tenant

### Temporal-First Architecture
- **Workflow-Driven Operations**: All complex operations implemented as Temporal workflows
- **Automatic Retry**: Built-in retry and compensation logic for reliability
- **Observability**: Complete visibility into module operations through Temporal UI
- **Cross-Service Coordination**: Workflows coordinate operations across multiple services

### Security & Sandboxing
- **Comprehensive Security Scanning**: Static analysis, dependency checks, malware detection
- **Resource Limits**: CPU, memory, storage, and network bandwidth limits
- **Sandbox Enforcement**: Isolated execution environment for modules
- **Permission System**: Fine-grained permission control for module operations

### Marketplace Integration
- **Module Discovery**: Browse and search modules from the marketplace
- **Payment Processing**: Support for free, paid, and subscription-based modules
- **Ratings & Reviews**: Community-driven module evaluation
- **Automated Indexing**: Real-time synchronization with marketplace data

### Development SDK
- **Module Templates**: Pre-built templates for different module types
- **Development Tools**: Build, test, and package modules
- **Documentation Generator**: Automatic API documentation generation
- **Testing Framework**: Comprehensive testing tools for module development

## Architecture

### Service Modes
The module service operates in multiple modes:

1. **HTTP Server Mode** (`server`): REST API endpoints for module management
2. **Temporal Worker Mode** (`worker`): Executes workflows and activities
3. **Marketplace Indexer** (`indexer`): Synchronizes marketplace data
4. **Security Scanner** (`scanner`): Performs security scans

### Database Schema
- **modules**: Core module information and metadata
- **module_installations**: Per-tenant module installations
- **module_dependencies**: Module dependency relationships
- **module_versions**: Version history and compatibility
- **module_marketplace**: Marketplace-specific data
- **module_workflows**: Workflow execution tracking
- **module_security_scans**: Security scan results
- **module_sandbox**: Sandbox configurations and monitoring

### Temporal Workflows

#### Installation Workflow
1. Validate module and permissions
2. Download and verify package
3. Perform security scan
4. Install dependencies
5. Deploy module files
6. Configure sandbox
7. Register installation
8. Activate module (optional)

#### Update Workflow
1. Validate update request
2. Create backup (optional)
3. Download new version
4. Apply update with migration
5. Rollback on failure (optional)

#### Uninstallation Workflow
1. Validate uninstallation
2. Deactivate module
3. Clean up data (optional)
4. Remove installation
5. Update status

## API Endpoints

### Module Management
- `GET /api/v1/modules` - List modules
- `POST /api/v1/modules` - Create module
- `GET /api/v1/modules/{id}` - Get module details
- `PUT /api/v1/modules/{id}` - Update module
- `DELETE /api/v1/modules/{id}` - Delete module
- `POST /api/v1/modules/search` - Search modules

### Installation Management
- `GET /api/v1/installations` - List installations
- `POST /api/v1/installations` - Install module
- `GET /api/v1/installations/{id}` - Get installation
- `DELETE /api/v1/installations/{id}` - Uninstall module
- `POST /api/v1/installations/{id}/activate` - Activate module
- `POST /api/v1/installations/{id}/deactivate` - Deactivate module

### Marketplace
- `GET /api/v1/marketplace/modules` - Browse marketplace
- `POST /api/v1/marketplace/search` - Search marketplace
- `GET /api/v1/marketplace/featured` - Featured modules
- `GET /api/v1/marketplace/categories` - Module categories

### Workflows
- `POST /api/v1/workflows/install` - Start installation workflow
- `POST /api/v1/workflows/update` - Start update workflow
- `POST /api/v1/workflows/uninstall` - Start uninstallation workflow
- `GET /api/v1/workflows/{id}/status` - Get workflow status
- `POST /api/v1/workflows/{id}/cancel` - Cancel workflow

### Security
- `POST /api/v1/security/scan` - Initiate security scan
- `GET /api/v1/security/scans/{id}` - Get scan results
- `GET /api/v1/security/vulnerabilities` - List vulnerabilities

### SDK
- `GET /api/v1/sdk/templates` - List module templates
- `POST /api/v1/sdk/templates/{name}` - Create from template
- `POST /api/v1/sdk/validate` - Validate module
- `POST /api/v1/sdk/build` - Build module
- `POST /api/v1/sdk/test` - Test module
- `POST /api/v1/sdk/package` - Package module
- `POST /api/v1/sdk/publish` - Publish module

## Configuration

### Environment Variables
- `MODULE_SERVICE_HOST` - Server host (default: 0.0.0.0)
- `MODULE_SERVICE_PORT` - Server port (default: 8086)
- `DATABASE_URL` - PostgreSQL connection string
- `TEMPORAL_SERVER_URL` - Temporal server URL
- `REDIS_URL` - Redis connection string
- `MARKETPLACE_ENABLED` - Enable marketplace integration
- `SANDBOX_ENABLED` - Enable module sandboxing
- `SECURITY_SCAN_ENABLED` - Enable security scanning

### Service Configuration
```toml
[server]
host = "0.0.0.0"
port = 8086

[database]
url = "postgresql://user:pass@localhost/adx_core"
max_connections = 10

[temporal]
server_url = "localhost:7233"
namespace = "default"
task_queue = "module-task-queue"

[marketplace]
enabled = true
api_url = "https://marketplace.adxcore.com"

[sandbox]
enabled = true
max_memory_mb = 512
max_cpu_percent = 50.0
network_isolation = true

[security]
scan_enabled = true
signature_verification = true
```

## Development

### Running the Service
```bash
# Start HTTP server
cargo run --bin module-service server

# Start Temporal worker
cargo run --bin module-service worker

# Start marketplace indexer
cargo run --bin module-service indexer

# Start security scanner
cargo run --bin module-service scanner
```

### Testing
```bash
# Run unit tests
cargo test --lib

# Run integration tests
cargo test --test integration_tests

# Run workflow tests
cargo test --test workflow_tests
```

### Database Migrations
```bash
# Install sqlx-cli
cargo install sqlx-cli

# Run migrations
sqlx migrate run

# Create new migration
sqlx migrate add create_new_table
```

## Module Development

### Creating a Module
1. Use the SDK to create a module template
2. Implement module functionality
3. Add tests and documentation
4. Build and package the module
5. Publish to marketplace

### Module Structure
```
my-module/
├── manifest.json          # Module metadata
├── src/
│   ├── index.js          # Main entry point
│   ├── activities.js     # Temporal activities
│   └── workflows.js      # Temporal workflows
├── frontend/
│   ├── components/       # React components
│   └── routes.js         # Frontend routes
├── tests/
│   └── module.test.js    # Module tests
└── README.md             # Module documentation
```

### Module Manifest
```json
{
  "name": "my-module",
  "version": "1.0.0",
  "description": "My custom module",
  "author": {
    "name": "Developer Name",
    "email": "dev@example.com"
  },
  "adxCore": {
    "minVersion": "2.0.0"
  },
  "permissions": [
    "database:read",
    "api:external"
  ],
  "extensionPoints": {
    "backend": {
      "activities": ["./src/activities.js"],
      "workflows": ["./src/workflows.js"]
    },
    "frontend": {
      "components": ["./frontend/components.js"]
    }
  }
}
```

## Security

### Security Scanning
- **Static Analysis**: Code pattern analysis for vulnerabilities
- **Dependency Scanning**: Known vulnerability database checks
- **Malware Detection**: Signature-based malware scanning
- **License Compliance**: License compatibility verification

### Sandboxing
- **Resource Limits**: CPU, memory, storage constraints
- **Network Isolation**: Controlled network access
- **File System Restrictions**: Limited file system access
- **Process Isolation**: Separate process execution

### Permission System
- `database:read` - Read database access
- `database:write` - Write database access
- `api:external` - External API calls
- `file:read` - File system read access
- `file:write` - File system write access
- `workflow:execute` - Execute workflows
- `tenant:data` - Access tenant data
- `user:data` - Access user data

## Monitoring

### Health Checks
- `GET /health` - Service health status
- `GET /ready` - Service readiness check

### Metrics
- Module installation success/failure rates
- Workflow execution times
- Security scan results
- Resource usage per module
- Marketplace synchronization status

### Logging
- Structured logging with JSON format
- Correlation IDs for request tracing
- Security event logging
- Performance metrics logging

## Troubleshooting

### Common Issues
1. **Module Installation Fails**: Check security scan results and dependencies
2. **Workflow Timeout**: Increase timeout configuration or optimize workflow
3. **Sandbox Violations**: Review resource limits and permissions
4. **Marketplace Sync Issues**: Check API credentials and network connectivity

### Debug Mode
Set `RUST_LOG=debug` to enable detailed logging for troubleshooting.

## Contributing

1. Fork the repository
2. Create a feature branch
3. Add tests for new functionality
4. Ensure all tests pass
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.