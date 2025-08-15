# White Label Service

The White Label Service provides comprehensive white-labeling capabilities for ADX CORE, including custom domains, branding customization, and multi-level reseller management. All operations are implemented as Temporal workflows for reliability and observability.

## Features

### Custom Domain Management
- **DNS Verification**: Automated DNS record verification with multiple provider support
- **SSL Certificate Provisioning**: Automatic SSL certificate generation and renewal
- **Domain Routing**: Load balancer configuration for custom domains
- **Multi-Provider Support**: Cloudflare, Route53, GoDaddy DNS providers

### White Label Branding
- **Asset Management**: Logo, favicon, and custom image processing
- **Color Schemes**: Customizable color palettes with validation
- **Typography**: Custom font selection and sizing
- **CSS Generation**: Automated CSS generation with asset integration
- **Email Templates**: Branded email template processing with Handlebars
- **Rollback Support**: Backup and rollback functionality for branding changes

### Reseller Management
- **Multi-Level Hierarchy**: Support for complex reseller hierarchies
- **Commission Management**: Flexible commission rate calculation
- **Revenue Sharing**: Multiple revenue sharing models (flat, tiered, progressive)
- **Support Routing**: Hierarchical support contact routing
- **Feature Control**: Per-reseller feature access control

## Architecture

### Temporal-First Design
All complex operations are implemented as Temporal workflows:

- `custom_domain_setup_workflow`: DNS verification and SSL provisioning
- `white_label_branding_workflow`: Asset processing with rollback capability
- `reseller_setup_workflow`: Multi-level reseller hierarchy setup

### Dual-Mode Operation
The service operates in two modes:

1. **HTTP Server Mode** (`cargo run --bin white-label-service server`)
   - REST API endpoints for direct operations
   - Workflow initiation and status tracking
   - Asset upload and management

2. **Temporal Worker Mode** (`cargo run --bin white-label-service worker`)
   - Workflow and activity execution
   - Background processing
   - Automatic retry and error handling

## API Endpoints

### Domain Management
```
POST   /api/v1/white-label/domains              # Create custom domain
GET    /api/v1/white-label/domains              # List domains
GET    /api/v1/white-label/domains/{id}         # Get domain details
DELETE /api/v1/white-label/domains/{id}         # Delete domain
POST   /api/v1/white-label/domains/{id}/verify  # Verify domain
POST   /api/v1/white-label/domains/{id}/ssl     # Provision SSL
```

### Branding Management
```
POST   /api/v1/white-label/branding             # Create/update branding
GET    /api/v1/white-label/branding             # Get current branding
PUT    /api/v1/white-label/branding             # Update branding
DELETE /api/v1/white-label/branding             # Delete branding
GET    /api/v1/white-label/branding/preview     # Get preview URL
POST   /api/v1/white-label/branding/rollback    # Rollback changes
```

### Reseller Management
```
POST   /api/v1/white-label/resellers            # Create reseller
GET    /api/v1/white-label/resellers            # List resellers
GET    /api/v1/white-label/resellers/{id}       # Get reseller details
PUT    /api/v1/white-label/resellers/{id}       # Update reseller
DELETE /api/v1/white-label/resellers/{id}       # Delete reseller
GET    /api/v1/white-label/resellers/{id}/hierarchy # Get hierarchy
```

### Asset Management
```
POST   /api/v1/white-label/assets               # Upload asset
GET    /api/v1/white-label/assets               # List assets
GET    /api/v1/white-label/assets/{id}          # Get asset details
DELETE /api/v1/white-label/assets/{id}          # Delete asset
```

### Workflow Status
```
GET    /api/v1/white-label/workflows/{id}/status # Get workflow status
POST   /api/v1/white-label/workflows/{id}/cancel # Cancel workflow
```

## Configuration

The service is configured via environment variables:

```bash
# Database
WHITE_LABEL_DATABASE_URL=postgresql://localhost:5432/adx_core

# Temporal
WHITE_LABEL_TEMPORAL_SERVER_URL=http://localhost:7233

# Server
WHITE_LABEL_SERVER_PORT=8087

# Domain Configuration
WHITE_LABEL_DOMAIN_CONFIG_MAX_DOMAINS_PER_TENANT=5
WHITE_LABEL_DOMAIN_CONFIG_VERIFICATION_TIMEOUT_SECONDS=300

# SSL Configuration
WHITE_LABEL_SSL_CONFIG_PROVIDER=letsencrypt
WHITE_LABEL_SSL_CONFIG_AUTO_RENEWAL=true

# Asset Configuration
WHITE_LABEL_ASSET_CONFIG_MAX_FILE_SIZE_MB=10
WHITE_LABEL_ASSET_CONFIG_STORAGE_PATH=./storage/assets

# Email Configuration
WHITE_LABEL_EMAIL_CONFIG_SMTP_HOST=localhost
WHITE_LABEL_EMAIL_CONFIG_SMTP_PORT=587
WHITE_LABEL_EMAIL_CONFIG_FROM_EMAIL=noreply@adxcore.com

# Storage Configuration
WHITE_LABEL_STORAGE_CONFIG_PROVIDER=local
```

## Database Schema

The service uses the following main tables:

- `custom_domains`: Domain configurations and verification status
- `white_label_branding`: Branding configurations and assets
- `branding_assets`: Uploaded branding assets with metadata
- `reseller_hierarchies`: Multi-level reseller relationships
- `revenue_sharing_configs`: Revenue sharing configurations
- `support_routing_configs`: Support routing configurations
- `branding_backups`: Temporary backups for rollback functionality
- `ssl_certificates`: SSL certificate tracking
- `dns_records`: DNS record configurations

## Workflows

### Custom Domain Setup Workflow

1. **Domain Validation**: Validate domain format and availability
2. **DNS Record Generation**: Create verification DNS records
3. **DNS Verification**: Verify DNS records are properly configured
4. **SSL Provisioning**: Generate and install SSL certificates
5. **Routing Configuration**: Configure load balancer routing

### White Label Branding Workflow

1. **Request Validation**: Validate branding request and assets
2. **Backup Creation**: Create backup of existing branding
3. **Asset Processing**: Process and optimize uploaded assets
4. **CSS Generation**: Generate custom CSS with branding
5. **Template Processing**: Process email templates with branding
6. **Database Update**: Update branding configuration
7. **Preview Generation**: Generate branding preview
8. **Cleanup**: Clean up temporary files and backups

### Reseller Setup Workflow

1. **Hierarchy Validation**: Validate reseller hierarchy constraints
2. **Commission Calculation**: Calculate effective commission rates
3. **Branding Setup**: Set up branding overrides if provided
4. **Database Creation**: Create reseller record
5. **Revenue Configuration**: Configure revenue sharing
6. **Support Routing**: Set up support routing
7. **Welcome Notification**: Send welcome email to reseller

## Development

### Running the Service

```bash
# Start HTTP server
cargo run --bin white-label-service server

# Start Temporal worker
cargo run --bin white-label-service worker

# Run tests
cargo test

# Run integration tests
cargo test --test integration_tests
```

### Adding New Features

1. **Define Types**: Add new types to `src/types.rs`
2. **Create Activities**: Implement activities in `src/activities.rs`
3. **Build Workflows**: Create workflows in `src/workflows.rs`
4. **Add Handlers**: Create HTTP handlers in `src/handlers.rs`
5. **Update Database**: Add migrations to `migrations/`
6. **Write Tests**: Add tests to `tests/`

### Testing

The service includes comprehensive tests:

- **Unit Tests**: Individual component testing
- **Integration Tests**: Cross-component workflow testing
- **Workflow Tests**: Temporal workflow execution testing
- **API Tests**: HTTP endpoint testing

## Security Considerations

- **Domain Validation**: Strict domain format and TLD validation
- **Asset Security**: File type validation and virus scanning
- **SSL Security**: Automatic certificate renewal and validation
- **Access Control**: Tenant-based access control for all operations
- **Input Validation**: Comprehensive input validation and sanitization
- **Audit Logging**: Complete audit trail for all operations

## Monitoring and Observability

- **Temporal UI**: Workflow execution monitoring
- **Structured Logging**: JSON-formatted logs with correlation IDs
- **Metrics**: Prometheus metrics for key operations
- **Health Checks**: Service health monitoring endpoints
- **Error Tracking**: Comprehensive error tracking and alerting

## Deployment

The service can be deployed in various configurations:

- **Standalone**: Single instance with both server and worker
- **Distributed**: Separate server and worker instances
- **Containerized**: Docker containers with orchestration
- **Cloud Native**: Kubernetes deployment with auto-scaling

## Support

For issues and questions:

- Check the logs for detailed error information
- Use Temporal UI to monitor workflow execution
- Review the API documentation for endpoint details
- Consult the configuration guide for setup issues