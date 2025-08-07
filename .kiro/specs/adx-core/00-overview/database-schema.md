# ADX CORE - Database Schema Design

## Overview

ADX CORE uses PostgreSQL as the primary database with a multi-tenant architecture that ensures complete data isolation between tenants while maintaining performance and scalability.

## Multi-Tenant Strategy

### Tenant Isolation Approach
- **Schema-per-tenant**: Each tenant gets a dedicated PostgreSQL schema
- **Shared infrastructure**: Common tables in `public` schema
- **Row-level security**: Additional security layer for sensitive operations
- **Connection pooling**: Efficient resource utilization across tenants

### Schema Organization
```
adx_core_db/
├── public/                    # System-wide tables
│   ├── tenants
│   ├── users
│   ├── tenant_memberships
│   ├── system_settings
│   └── audit_logs
├── tenant_<uuid>/             # Per-tenant schemas
│   ├── files
│   ├── licenses
│   ├── workflows
│   ├── plugins
│   └── custom_data
└── shared/                    # Shared reference data
    ├── countries
    ├── currencies
    ├── timezones
    └── translations
```

## Core Schema Definition

### System Tables (Public Schema)

```sql
-- Tenants table - Core tenant information
CREATE TABLE public.tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(100) UNIQUE NOT NULL,
    domain VARCHAR(255) UNIQUE,
    status tenant_status NOT NULL DEFAULT 'active',
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT tenants_slug_format CHECK (slug ~ '^[a-z0-9-]+$'),
    CONSTRAINT tenants_domain_format CHECK (domain ~ '^[a-z0-9.-]+$')
);

-- Custom enum types
CREATE TYPE tenant_status AS ENUM ('active', 'suspended', 'deleted');
CREATE TYPE user_status AS ENUM ('active', 'inactive', 'suspended', 'pending_verification');
CREATE TYPE membership_role AS ENUM ('owner', 'admin', 'member', 'viewer', 'custom');
CREATE TYPE license_tier AS ENUM ('starter', 'professional', 'enterprise');
CREATE TYPE file_visibility AS ENUM ('private', 'tenant', 'public');

-- Users table - Global user accounts
CREATE TABLE public.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(320) UNIQUE NOT NULL,
    email_verified BOOLEAN NOT NULL DEFAULT FALSE,
    password_hash VARCHAR(255),
    name VARCHAR(255) NOT NULL,
    avatar_url VARCHAR(500),
    preferences JSONB DEFAULT '{}',
    status user_status NOT NULL DEFAULT 'pending_verification',
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    
    -- Multi-factor authentication
    mfa_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    mfa_secret VARCHAR(32),
    mfa_backup_codes TEXT[],
    
    -- SSO integration
    sso_provider VARCHAR(50),
    sso_external_id VARCHAR(255),
    
    -- Constraints
    CONSTRAINT users_email_format CHECK (email ~ '^[^@]+@[^@]+\.[^@]+$'),
    CONSTRAINT users_mfa_secret_length CHECK (mfa_secret IS NULL OR length(mfa_secret) = 32)
);

-- Tenant memberships - User-tenant relationships
CREATE TABLE public.tenant_memberships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES public.tenants(id) ON DELETE CASCADE,
    role membership_role NOT NULL DEFAULT 'member',
    permissions JSONB DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    invited_by UUID REFERENCES public.users(id),
    invited_at TIMESTAMPTZ,
    joined_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(user_id, tenant_id),
    CONSTRAINT membership_status_valid CHECK (status IN ('active', 'inactive', 'pending'))
);

-- System audit logs
CREATE TABLE public.audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES public.tenants(id),
    user_id UUID REFERENCES public.users(id),
    action VARCHAR(100) NOT NULL,
    resource_type VARCHAR(50) NOT NULL,
    resource_id VARCHAR(255),
    old_values JSONB,
    new_values JSONB,
    metadata JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Indexes for performance
    INDEX idx_audit_logs_tenant_id (tenant_id),
    INDEX idx_audit_logs_user_id (user_id),
    INDEX idx_audit_logs_action (action),
    INDEX idx_audit_logs_created_at (created_at)
);

-- System settings
CREATE TABLE public.system_settings (
    key VARCHAR(100) PRIMARY KEY,
    value JSONB NOT NULL,
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
```

### Tenant-Specific Tables (Per-Tenant Schema)

```sql
-- Files table (per tenant)
CREATE TABLE tenant_<uuid>.files (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    original_name VARCHAR(255) NOT NULL,
    path VARCHAR(1000) NOT NULL,
    size_bytes BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    storage_provider VARCHAR(50) NOT NULL DEFAULT 'local',
    storage_path VARCHAR(1000) NOT NULL,
    visibility file_visibility NOT NULL DEFAULT 'private',
    
    -- File metadata
    metadata JSONB DEFAULT '{}',
    tags TEXT[],
    description TEXT,
    
    -- Versioning
    version INTEGER NOT NULL DEFAULT 1,
    parent_file_id UUID REFERENCES tenant_<uuid>.files(id),
    is_current_version BOOLEAN NOT NULL DEFAULT TRUE,
    
    -- Access control
    owner_id UUID NOT NULL, -- References public.users(id)
    permissions JSONB DEFAULT '{}',
    
    -- Lifecycle
    expires_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    deleted_at TIMESTAMPTZ,
    
    -- Constraints
    CONSTRAINT files_size_positive CHECK (size_bytes > 0),
    CONSTRAINT files_version_positive CHECK (version > 0),
    CONSTRAINT files_checksum_format CHECK (checksum ~ '^[a-f0-9]{64}$')
);

-- File shares (per tenant)
CREATE TABLE tenant_<uuid>.file_shares (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    file_id UUID NOT NULL REFERENCES tenant_<uuid>.files(id) ON DELETE CASCADE,
    shared_by UUID NOT NULL, -- References public.users(id)
    shared_with UUID, -- References public.users(id) (NULL for public shares)
    share_token VARCHAR(64) UNIQUE,
    permissions JSONB NOT NULL DEFAULT '{"read": true}',
    expires_at TIMESTAMPTZ,
    password_hash VARCHAR(255),
    download_limit INTEGER,
    download_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT file_shares_token_format CHECK (share_token ~ '^[a-zA-Z0-9_-]{64}$'),
    CONSTRAINT file_shares_download_limit_positive CHECK (download_limit IS NULL OR download_limit > 0)
);

-- Licenses (per tenant)
CREATE TABLE tenant_<uuid>.licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tier license_tier NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'active',
    features JSONB NOT NULL DEFAULT '{}',
    limits JSONB NOT NULL DEFAULT '{}',
    usage JSONB NOT NULL DEFAULT '{}',
    
    -- Billing information
    billing_cycle VARCHAR(20) NOT NULL DEFAULT 'monthly',
    price_cents INTEGER NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    
    -- Lifecycle
    starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    trial_ends_at TIMESTAMPTZ,
    canceled_at TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT licenses_status_valid CHECK (status IN ('active', 'expired', 'canceled', 'suspended')),
    CONSTRAINT licenses_billing_cycle_valid CHECK (billing_cycle IN ('monthly', 'yearly', 'one_time')),
    CONSTRAINT licenses_price_non_negative CHECK (price_cents >= 0)
);

-- Workflows (per tenant)
CREATE TABLE tenant_<uuid>.workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    definition JSONB NOT NULL,
    version INTEGER NOT NULL DEFAULT 1,
    status VARCHAR(20) NOT NULL DEFAULT 'draft',
    
    -- AI enhancement
    ai_enhanced BOOLEAN NOT NULL DEFAULT FALSE,
    ai_config JSONB DEFAULT '{}',
    
    -- Metadata
    tags TEXT[],
    category VARCHAR(50),
    created_by UUID NOT NULL, -- References public.users(id)
    updated_by UUID, -- References public.users(id)
    
    -- Lifecycle
    published_at TIMESTAMPTZ,
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT workflows_status_valid CHECK (status IN ('draft', 'published', 'archived')),
    CONSTRAINT workflows_version_positive CHECK (version > 0)
);

-- Workflow executions (per tenant)
CREATE TABLE tenant_<uuid>.workflow_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_id UUID NOT NULL REFERENCES tenant_<uuid>.workflows(id),
    temporal_workflow_id VARCHAR(255) NOT NULL,
    temporal_run_id VARCHAR(255) NOT NULL,
    
    -- Execution details
    status VARCHAR(20) NOT NULL DEFAULT 'running',
    input JSONB,
    output JSONB,
    error_message TEXT,
    
    -- AI enhancement tracking
    ai_enhanced BOOLEAN NOT NULL DEFAULT FALSE,
    ai_decisions JSONB DEFAULT '[]',
    fallback_used BOOLEAN NOT NULL DEFAULT FALSE,
    
    -- Performance metrics
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    duration_ms INTEGER,
    
    -- Metadata
    triggered_by UUID, -- References public.users(id)
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT workflow_executions_status_valid CHECK (status IN ('running', 'completed', 'failed', 'canceled')),
    CONSTRAINT workflow_executions_duration_positive CHECK (duration_ms IS NULL OR duration_ms >= 0),
    UNIQUE(temporal_workflow_id, temporal_run_id)
);

-- Plugins (per tenant)
CREATE TABLE tenant_<uuid>.plugins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    plugin_id VARCHAR(100) NOT NULL, -- Plugin identifier from marketplace
    name VARCHAR(255) NOT NULL,
    version VARCHAR(20) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'inactive',
    
    -- Configuration
    config JSONB DEFAULT '{}',
    permissions JSONB DEFAULT '{}',
    
    -- Installation details
    installed_by UUID NOT NULL, -- References public.users(id)
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    activated_at TIMESTAMPTZ,
    last_updated_at TIMESTAMPTZ,
    
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT plugins_status_valid CHECK (status IN ('active', 'inactive', 'error', 'updating')),
    CONSTRAINT plugins_version_format CHECK (version ~ '^[0-9]+\.[0-9]+\.[0-9]+')
);

-- Custom data tables (per tenant) - for plugin extensibility
CREATE TABLE tenant_<uuid>.custom_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    data JSONB NOT NULL DEFAULT '{}',
    created_by UUID NOT NULL, -- References public.users(id)
    updated_by UUID, -- References public.users(id)
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(entity_type, entity_id)
);
```

### Shared Reference Tables

```sql
-- Countries reference data
CREATE TABLE shared.countries (
    code VARCHAR(2) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    name_local VARCHAR(100),
    continent VARCHAR(20) NOT NULL,
    currency_code VARCHAR(3),
    phone_prefix VARCHAR(10),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Currencies reference data
CREATE TABLE shared.currencies (
    code VARCHAR(3) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    decimal_places INTEGER NOT NULL DEFAULT 2,
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Timezones reference data
CREATE TABLE shared.timezones (
    name VARCHAR(100) PRIMARY KEY,
    offset_hours DECIMAL(3,1) NOT NULL,
    dst_offset_hours DECIMAL(3,1),
    country_code VARCHAR(2) REFERENCES shared.countries(code),
    is_active BOOLEAN NOT NULL DEFAULT TRUE
);

-- Translations
CREATE TABLE shared.translations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    namespace VARCHAR(50) NOT NULL DEFAULT 'default',
    key VARCHAR(255) NOT NULL,
    language VARCHAR(5) NOT NULL,
    value TEXT NOT NULL,
    tenant_id UUID REFERENCES public.tenants(id), -- NULL for system translations
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    -- Constraints
    UNIQUE(namespace, key, language, tenant_id)
);
```

## Indexes and Performance Optimization

### Primary Indexes
```sql
-- Users table indexes
CREATE INDEX idx_users_email ON public.users(email);
CREATE INDEX idx_users_status ON public.users(status);
CREATE INDEX idx_users_sso_provider_external_id ON public.users(sso_provider, sso_external_id);

-- Tenant memberships indexes
CREATE INDEX idx_tenant_memberships_user_id ON public.tenant_memberships(user_id);
CREATE INDEX idx_tenant_memberships_tenant_id ON public.tenant_memberships(tenant_id);
CREATE INDEX idx_tenant_memberships_role ON public.tenant_memberships(role);

-- Files indexes (per tenant)
CREATE INDEX idx_files_owner_id ON tenant_<uuid>.files(owner_id);
CREATE INDEX idx_files_parent_file_id ON tenant_<uuid>.files(parent_file_id);
CREATE INDEX idx_files_visibility ON tenant_<uuid>.files(visibility);
CREATE INDEX idx_files_storage_provider ON tenant_<uuid>.files(storage_provider);
CREATE INDEX idx_files_created_at ON tenant_<uuid>.files(created_at);
CREATE INDEX idx_files_tags ON tenant_<uuid>.files USING GIN(tags);
CREATE INDEX idx_files_metadata ON tenant_<uuid>.files USING GIN(metadata);

-- Workflow executions indexes (per tenant)
CREATE INDEX idx_workflow_executions_workflow_id ON tenant_<uuid>.workflow_executions(workflow_id);
CREATE INDEX idx_workflow_executions_status ON tenant_<uuid>.workflow_executions(status);
CREATE INDEX idx_workflow_executions_started_at ON tenant_<uuid>.workflow_executions(started_at);
CREATE INDEX idx_workflow_executions_triggered_by ON tenant_<uuid>.workflow_executions(triggered_by);

-- Audit logs indexes
CREATE INDEX idx_audit_logs_tenant_created ON public.audit_logs(tenant_id, created_at);
CREATE INDEX idx_audit_logs_user_created ON public.audit_logs(user_id, created_at);
CREATE INDEX idx_audit_logs_resource ON public.audit_logs(resource_type, resource_id);
```

### Composite Indexes for Common Queries
```sql
-- User tenant access patterns
CREATE INDEX idx_memberships_user_tenant_status ON public.tenant_memberships(user_id, tenant_id, status);

-- File access patterns
CREATE INDEX idx_files_owner_visibility_created ON tenant_<uuid>.files(owner_id, visibility, created_at);
CREATE INDEX idx_files_current_version ON tenant_<uuid>.files(parent_file_id, is_current_version) WHERE is_current_version = TRUE;

-- Workflow execution patterns
CREATE INDEX idx_workflow_executions_status_started ON tenant_<uuid>.workflow_executions(status, started_at);
CREATE INDEX idx_workflow_executions_ai_enhanced ON tenant_<uuid>.workflow_executions(ai_enhanced, created_at);
```

## Data Relationships and Constraints

### Foreign Key Relationships
```sql
-- Cross-schema foreign keys (handled at application level)
-- tenant_<uuid>.files.owner_id -> public.users.id
-- tenant_<uuid>.workflows.created_by -> public.users.id
-- tenant_<uuid>.workflow_executions.triggered_by -> public.users.id

-- Referential integrity triggers
CREATE OR REPLACE FUNCTION validate_cross_schema_user_reference()
RETURNS TRIGGER AS $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM public.users WHERE id = NEW.owner_id) THEN
        RAISE EXCEPTION 'Referenced user does not exist: %', NEW.owner_id;
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply to relevant tables
CREATE TRIGGER validate_file_owner
    BEFORE INSERT OR UPDATE ON tenant_<uuid>.files
    FOR EACH ROW EXECUTE FUNCTION validate_cross_schema_user_reference();
```

### Row Level Security (RLS)
```sql
-- Enable RLS on sensitive tables
ALTER TABLE public.tenant_memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE public.audit_logs ENABLE ROW LEVEL SECURITY;

-- Policies for tenant isolation
CREATE POLICY tenant_membership_isolation ON public.tenant_memberships
    FOR ALL TO application_role
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

CREATE POLICY audit_log_tenant_isolation ON public.audit_logs
    FOR SELECT TO application_role
    USING (tenant_id = current_setting('app.current_tenant_id')::UUID OR tenant_id IS NULL);
```

## Migration Strategy

### Schema Versioning
```sql
-- Migration tracking
CREATE TABLE public.schema_migrations (
    version VARCHAR(20) PRIMARY KEY,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    checksum VARCHAR(64) NOT NULL
);

-- Tenant schema versioning
CREATE TABLE public.tenant_schema_versions (
    tenant_id UUID NOT NULL REFERENCES public.tenants(id),
    version VARCHAR(20) NOT NULL,
    applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (tenant_id, version)
);
```

### Backup and Recovery
```sql
-- Point-in-time recovery setup
-- Automated daily backups with 30-day retention
-- Cross-region backup replication for disaster recovery
-- Tenant-specific backup and restore capabilities

-- Backup metadata tracking
CREATE TABLE public.backup_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID REFERENCES public.tenants(id),
    backup_type VARCHAR(20) NOT NULL, -- 'full', 'incremental', 'schema'
    status VARCHAR(20) NOT NULL DEFAULT 'running',
    size_bytes BIGINT,
    location VARCHAR(500),
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    
    CONSTRAINT backup_logs_type_valid CHECK (backup_type IN ('full', 'incremental', 'schema')),
    CONSTRAINT backup_logs_status_valid CHECK (status IN ('running', 'completed', 'failed'))
);
```

This comprehensive database schema provides:

1. **Complete multi-tenant isolation** with schema-per-tenant approach
2. **Scalable architecture** supporting millions of users and thousands of tenants
3. **Flexible plugin system** with custom data storage capabilities
4. **Comprehensive audit logging** for compliance and security
5. **Performance optimization** with strategic indexing
6. **Data integrity** with proper constraints and validation
7. **Backup and recovery** capabilities for business continuity
8. **AI workflow tracking** for hybrid orchestration analytics