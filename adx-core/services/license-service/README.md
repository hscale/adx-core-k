# License Service

The License Service is a comprehensive license and quota management system for ADX Core, built with Temporal-first architecture for reliable workflow orchestration.

## Features

### License Management
- **License Provisioning**: Complete license setup with payment provider integration
- **License Validation**: Real-time license validation and status checking
- **License Renewal**: Automated and manual license renewal workflows
- **Multi-tier Support**: Free, Professional, Enterprise, and Custom tiers
- **Payment Integration**: Stripe and PayPal integration for billing

### Quota Management
- **Real-time Enforcement**: Immediate quota checking and enforcement
- **Multi-dimensional Quotas**: API calls, storage, users, workflows, and more
- **Usage Tracking**: Detailed usage logging and analytics
- **Warning Notifications**: Proactive notifications at configurable thresholds
- **Flexible Limits**: Per-tenant customization and overrides

### Billing Integration
- **Multiple Payment Providers**: Stripe, PayPal, and enterprise billing systems
- **Usage-based Billing**: Automatic calculation of usage charges
- **Invoice Generation**: Automated invoice creation and delivery
- **Payment Processing**: Secure payment handling with retry logic
- **Billing History**: Complete transaction and payment tracking

### Compliance and Audit
- **Comprehensive Logging**: All license and quota events logged
- **Compliance Reports**: Automated compliance reporting and scoring
- **Audit Trails**: Immutable audit logs for regulatory compliance
- **Issue Resolution**: Workflow-based compliance issue resolution
- **Real-time Monitoring**: Continuous compliance monitoring

## Architecture

### Temporal-First Design
All complex operations are implemented as Temporal workflows:

- **License Provisioning Workflow**: Complete license setup process
- **Quota Enforcement Workflow**: Real-time quota checking and enforcement
- **License Renewal Workflow**: Automated renewal with payment processing

### Dual-Mode Operation
The service operates in two modes:

1. **HTTP Server Mode** (`--mode server`): REST API endpoints for direct operations
2. **Temporal Worker Mode** (`--mode worker`): Workflow and activity execution

### Database Schema
- **Licenses**: Core license information and status
- **Quota Definitions**: Configurable quota types and limits
- **Tenant Quotas**: Per-tenant quota assignments and usage
- **Usage Logs**: Detailed usage tracking
- **Billing History**: Payment and invoice records
- **Compliance Logs**: Audit and compliance events

## Configuration

### Environment Variables
```bash
# Database
LICENSE_SERVICE_DATABASE_URL=postgresql://localhost:5432/adx_core
LICENSE_SERVICE_REDIS_URL=redis://localhost:6379

# Server
LICENSE_SERVICE_SERVER_PORT=8087

# Temporal
LICENSE_SERVICE_TEMPORAL_SERVER_URL=http://localhost:7233
LICENSE_SERVICE_TEMPORAL_NAMESPACE=default
LICENSE_SERVICE_TEMPORAL_TASK_QUEUE=license-service-queue

# Stripe
LICENSE_SERVICE_STRIPE_SECRET_KEY=sk_test_...
LICENSE_SERVICE_STRIPE_PUBLISHABLE_KEY=pk_test_...
LICENSE_SERVICE_STRIPE_WEBHOOK_SECRET=whsec_...

# PayPal
LICENSE_SERVICE_PAYPAL_CLIENT_ID=...
LICENSE_SERVICE_PAYPAL_CLIENT_SECRET=...
LICENSE_SERVICE_PAYPAL_ENVIRONMENT=sandbox

# Billing
LICENSE_SERVICE_BILLING_INVOICE_PREFIX=ADX
LICENSE_SERVICE_BILLING_DEFAULT_CURRENCY=USD
LICENSE_SERVICE_BILLING_TAX_RATE=0.08
LICENSE_SERVICE_BILLING_GRACE_PERIOD_DAYS=7

# Quotas
LICENSE_SERVICE_QUOTAS_ENFORCEMENT_ENABLED=true
LICENSE_SERVICE_QUOTAS_REAL_TIME_MONITORING=true
LICENSE_SERVICE_QUOTAS_WARNING_NOTIFICATION_ENABLED=true
```

## API Endpoints

### License Management
```
POST   /licenses                    # Create license
GET    /licenses/:id                # Get license by ID
PUT    /licenses/:id                # Update license
GET    /licenses/tenant/:tenant_id  # Get license by tenant
GET    /licenses/validate/:key      # Validate license key
GET    /licenses/expiring           # Get expiring licenses
```

### Quota Management
```
GET    /quotas/tenant/:tenant_id          # Get tenant quotas
GET    /quotas/tenant/:tenant_id/summary  # Get quota usage summary
POST   /quotas/check                      # Check quota availability
POST   /quotas/enforce                    # Enforce quota usage
POST   /quotas/reset                      # Reset quota usage
```

### Billing
```
GET    /billing/tenant/:tenant_id    # Get billing history
POST   /billing/invoice              # Generate invoice
PUT    /billing/:id/status           # Update payment status
```

### Compliance
```
GET    /compliance/tenant/:tenant_id/logs    # Get compliance logs
GET    /compliance/tenant/:tenant_id/report  # Generate compliance report
POST   /compliance/:id/resolve               # Resolve compliance issue
```

### Workflows
```
POST   /workflows/provision-license  # Start license provisioning workflow
POST   /workflows/enforce-quota      # Start quota enforcement workflow
POST   /workflows/renew-license      # Start license renewal workflow
```

### Analytics
```
GET    /analytics/tenant/:tenant_id  # Get license analytics
```

## Usage Examples

### License Provisioning
```bash
curl -X POST http://localhost:8087/workflows/provision-license \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "123e4567-e89b-12d3-a456-426614174000",
    "subscription_tier": "professional",
    "billing_cycle": "monthly",
    "customer_email": "admin@company.com",
    "customer_name": "Company Admin",
    "payment_method": "stripe",
    "features": ["api_access", "file_storage", "workflows"],
    "setup_billing": true
  }'
```

### Quota Enforcement
```bash
curl -X POST http://localhost:8087/quotas/enforce \
  -H "Content-Type: application/json" \
  -d '{
    "tenant_id": "123e4567-e89b-12d3-a456-426614174000",
    "quota_name": "api_calls_per_hour",
    "amount": 1,
    "operation_type": "api_call",
    "user_id": "user123"
  }'
```

### License Validation
```bash
curl http://localhost:8087/licenses/validate/ADX-12345678-ABCDEFGH
```

## Development

### Running the Service
```bash
# Start HTTP server
cargo run --bin license-service -- --mode server

# Start Temporal worker
cargo run --bin license-service -- --mode worker
```

### Database Migrations
```bash
# Run migrations
sqlx migrate run --database-url postgresql://localhost:5432/adx_core

# Create new migration
sqlx migrate add -r new_migration_name
```

### Testing
```bash
# Run unit tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with coverage
cargo tarpaulin --out html
```

## Monitoring

### Health Check
```bash
curl http://localhost:8087/health
```

### Metrics
The service exposes Prometheus metrics at `/metrics` including:
- License creation/renewal rates
- Quota enforcement metrics
- Payment processing success rates
- Compliance violation counts
- Workflow execution metrics

### Logging
Structured logging with configurable levels:
```bash
RUST_LOG=license_service=debug cargo run
```

## Security

### Authentication
All endpoints require valid JWT tokens with appropriate permissions:
- `license:read` - View license information
- `license:write` - Modify licenses
- `quota:enforce` - Enforce quotas
- `billing:read` - View billing information
- `compliance:read` - View compliance logs

### Data Protection
- All sensitive data encrypted at rest
- PCI DSS compliance for payment processing
- GDPR compliance for personal data
- SOC 2 Type II compliance

### Rate Limiting
Built-in rate limiting per tenant and endpoint:
- License operations: 100/hour
- Quota checks: 1000/hour
- Billing operations: 50/hour

## Deployment

### Docker
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin license-service

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/license-service /usr/local/bin/
EXPOSE 8087
CMD ["license-service", "--mode", "server"]
```

### Kubernetes
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: license-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: license-service
  template:
    metadata:
      labels:
        app: license-service
    spec:
      containers:
      - name: license-service
        image: adxcore/license-service:latest
        ports:
        - containerPort: 8087
        env:
        - name: LICENSE_SERVICE_DATABASE_URL
          valueFrom:
            secretKeyRef:
              name: license-service-secrets
              key: database-url
        - name: LICENSE_SERVICE_STRIPE_SECRET_KEY
          valueFrom:
            secretKeyRef:
              name: license-service-secrets
              key: stripe-secret-key
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Run the test suite
6. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.