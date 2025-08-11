# ADX Core Database Migrations

This directory contains all database migration files for the ADX Core platform. The migrations are designed to support multi-tenant architecture with comprehensive tenant isolation.

## Migration Files

### Core Schema Migrations

1. **001_initial_schema.sql** - Initial database schema with core tables
   - Creates basic tenant, user, file, and session tables
   - Sets up enum types and basic indexes
   - Implements updated_at triggers

2. **002_tenant_isolation_setup.sql** - Tenant isolation infrastructure
   - Enables Row Level Security (RLS) on multi-tenant tables
   - Creates tenant schema management functions
   - Sets up tenant context functions for RLS

3. **003_auth_service_schema.sql** - Authentication service tables
   - Password reset and email verification tokens
   - Multi-Factor Authentication (MFA) support
   - SSO providers and user mappings
   - API keys and rate limiting

4. **004_file_service_schema.sql** - File management tables
   - File versions and sharing
   - File processing jobs and thumbnails
   - File organization (tags, folders)
   - Storage provider configurations

5. **005_workflow_service_schema.sql** - Temporal workflow tracking
   - Workflow executions and activities
   - Workflow schedules and templates
   - AI workflow configurations
   - Cross-service workflow orchestration

6. **006_tenant_service_schema.sql** - Tenant management tables
   - Tenant billing and usage tracking
   - Feature flags and invitations
   - Custom domains and branding
   - Compliance and backup configurations

7. **007_user_service_schema.sql** - Extended user management
   - User profiles and preferences
   - Notification settings and activity logs
   - Team organization and skills tracking
   - User connections and bookmarks

8. **008_database_health_and_indexing.sql** - Health monitoring and optimization
   - Database health check functions
   - Connection pool monitoring
   - Query performance tracking
   - Advanced indexing strategies

## Tenant Isolation Strategies

The database supports three tenant isolation levels:

### 1. Row Level Security (RLS) - Default
- All tenant data in shared tables with `tenant_id` column
- PostgreSQL RLS policies enforce tenant boundaries
- Most cost-effective and performant for most use cases

### 2. Schema-based Isolation
- Each tenant gets a separate PostgreSQL schema
- Tenant-specific tables created in dedicated schemas
- Better isolation but more complex management

### 3. Database-level Isolation
- Each tenant gets a separate database
- Maximum isolation but highest resource overhead
- Suitable for enterprise customers with strict requirements

## Migration Management

### Running Migrations

```bash
# Using the database setup script
./scripts/db-setup.sh migrate

# Using the db-manager directly
cargo run --bin db-manager migrate

# Using Docker
docker-compose -f infrastructure/docker/docker-compose.database.yml --profile migrate up
```

### Migration Best Practices

1. **Backward Compatibility**: All migrations should be backward compatible
2. **Idempotent Operations**: Use `IF NOT EXISTS` and `ON CONFLICT` clauses
3. **Index Creation**: Use `CREATE INDEX CONCURRENTLY` for large tables
4. **Data Migration**: Separate schema changes from data migrations
5. **Rollback Strategy**: Always have a rollback plan for complex migrations

## Database Seeding

### Development Data

```bash
# Seed development data
./scripts/db-setup.sh seed --environment development

# Or using db-manager
cargo run --bin db-manager seed --environment development
```

### Test Data

```bash
# Seed test data (includes cleanup)
./scripts/db-setup.sh seed --environment test

# Or using db-manager
cargo run --bin db-manager seed --environment test
```

## Health Monitoring

### Basic Health Check

```bash
./scripts/db-setup.sh health
```

### Enhanced Health Check

```bash
./scripts/db-setup.sh health-check
```

### Performance Analysis

```bash
# Analyze index performance
./scripts/db-setup.sh analyze-indexes

# Monitor connection pool
./scripts/db-setup.sh monitor-pool
```

## Database Configuration

### Environment Variables

- `DATABASE_URL` - PostgreSQL connection string
- `DB_MAX_CONNECTIONS` - Maximum connection pool size (default: 20)
- `DB_MIN_CONNECTIONS` - Minimum connection pool size (default: 1)
- `DB_ACQUIRE_TIMEOUT_SECONDS` - Connection acquire timeout (default: 30)
- `DB_IDLE_TIMEOUT_SECONDS` - Idle connection timeout (default: 600)
- `DB_MAX_LIFETIME_SECONDS` - Maximum connection lifetime (default: 1800)

### Connection Pool Configuration

```rust
use adx_shared::database::{DatabaseConfig, create_database_pool_with_config};

let config = DatabaseConfig {
    database_url: "postgresql://localhost/adx_core".to_string(),
    max_connections: 20,
    min_connections: 2,
    acquire_timeout_seconds: 30,
    idle_timeout_seconds: 600,
    max_lifetime_seconds: 1800,
    test_before_acquire: true,
};

let pool = create_database_pool_with_config(config).await?;
```

## Troubleshooting

### Common Issues

1. **Migration Failures**
   - Check database connectivity
   - Verify user permissions
   - Review migration logs for specific errors

2. **Performance Issues**
   - Run index analysis: `./scripts/db-setup.sh analyze-indexes`
   - Check connection pool: `./scripts/db-setup.sh monitor-pool`
   - Review slow query logs

3. **Tenant Isolation Problems**
   - Validate RLS policies are enabled
   - Check tenant context is properly set
   - Verify tenant_id columns exist and are indexed

### Debugging Commands

```bash
# Validate database integrity
./scripts/db-setup.sh validate

# Show database statistics
./scripts/db-setup.sh stats

# Clean test data
./scripts/db-setup.sh clean
```

## Development Workflow

### Local Development Setup

1. Start database services:
   ```bash
   docker-compose -f infrastructure/docker/docker-compose.database.yml up -d
   ```

2. Run migrations and seed data:
   ```bash
   ./scripts/db-setup.sh setup
   ```

3. Verify setup:
   ```bash
   ./scripts/db-setup.sh health-check
   ```

### Testing Workflow

1. Start test database:
   ```bash
   docker-compose -f infrastructure/docker/docker-compose.database.yml --profile test up -d postgres-test
   ```

2. Set test environment:
   ```bash
   export DATABASE_URL="postgresql://postgres:postgres@localhost:5433/adx_core_test"
   export ENVIRONMENT="test"
   ```

3. Run test migrations and seeding:
   ```bash
   ./scripts/db-setup.sh setup --environment test
   ```

## Security Considerations

1. **Connection Security**: Always use SSL/TLS in production
2. **User Permissions**: Follow principle of least privilege
3. **Tenant Isolation**: Regularly audit RLS policies
4. **Data Encryption**: Enable encryption at rest and in transit
5. **Audit Logging**: Monitor all database access and changes

## Performance Optimization

1. **Indexing Strategy**: 
   - Composite indexes for multi-tenant queries
   - Partial indexes for filtered queries
   - GIN indexes for JSONB columns

2. **Connection Pooling**:
   - Tune pool size based on workload
   - Monitor connection utilization
   - Use connection timeouts appropriately

3. **Query Optimization**:
   - Use EXPLAIN ANALYZE for slow queries
   - Monitor query performance logs
   - Optimize tenant context switching

## Backup and Recovery

1. **Regular Backups**: Implement automated backup strategy
2. **Point-in-Time Recovery**: Enable WAL archiving
3. **Tenant-specific Backups**: Support individual tenant data export
4. **Disaster Recovery**: Test recovery procedures regularly

For more information, see the main ADX Core documentation.