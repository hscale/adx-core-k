-- Tenant isolation setup for ADX Core
-- This migration sets up row-level security and schema-based tenant isolation

-- Enable Row Level Security on multi-tenant tables
ALTER TABLE users ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_sessions ENABLE ROW LEVEL SECURITY;
ALTER TABLE files ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_permissions ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_logs ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for users table
CREATE POLICY tenant_isolation_users ON users
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create RLS policies for user_sessions table
CREATE POLICY tenant_isolation_user_sessions ON user_sessions
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create RLS policies for files table
CREATE POLICY tenant_isolation_files ON files
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create RLS policies for file_permissions table
CREATE POLICY tenant_isolation_file_permissions ON file_permissions
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create RLS policies for tenant_memberships table
CREATE POLICY tenant_isolation_tenant_memberships ON tenant_memberships
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create RLS policies for audit_logs table
CREATE POLICY tenant_isolation_audit_logs ON audit_logs
    FOR ALL
    TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID OR tenant_id IS NULL);

-- Function to create tenant schema
CREATE OR REPLACE FUNCTION create_tenant_schema(tenant_id UUID, schema_name VARCHAR(63))
RETURNS VOID AS $$
DECLARE
    full_schema_name VARCHAR(63);
BEGIN
    -- Generate schema name if not provided
    IF schema_name IS NULL THEN
        full_schema_name := 'tenant_' || REPLACE(tenant_id::TEXT, '-', '_');
    ELSE
        full_schema_name := schema_name;
    END IF;
    
    -- Create the schema
    EXECUTE format('CREATE SCHEMA IF NOT EXISTS %I', full_schema_name);
    
    -- Insert into tenant_schemas table
    INSERT INTO tenant_schemas (tenant_id, schema_name)
    VALUES (tenant_id, full_schema_name)
    ON CONFLICT (tenant_id, schema_name) DO NOTHING;
    
    -- Create tenant-specific tables in the schema
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.tenant_data (
            id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
            data_type VARCHAR(100) NOT NULL,
            data JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )', full_schema_name);
    
    EXECUTE format('
        CREATE TABLE IF NOT EXISTS %I.tenant_settings (
            key VARCHAR(255) PRIMARY KEY,
            value JSONB NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )', full_schema_name);
    
    -- Create indexes for tenant-specific tables
    EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_tenant_data_type ON %I.tenant_data(data_type)', 
                   REPLACE(full_schema_name, '-', '_'), full_schema_name);
    EXECUTE format('CREATE INDEX IF NOT EXISTS idx_%I_tenant_data_created_at ON %I.tenant_data(created_at)', 
                   REPLACE(full_schema_name, '-', '_'), full_schema_name);
END;
$$ LANGUAGE plpgsql;

-- Function to drop tenant schema
CREATE OR REPLACE FUNCTION drop_tenant_schema(tenant_id UUID)
RETURNS VOID AS $$
DECLARE
    schema_record RECORD;
BEGIN
    -- Get all schemas for this tenant
    FOR schema_record IN 
        SELECT schema_name FROM tenant_schemas WHERE tenant_schemas.tenant_id = drop_tenant_schema.tenant_id
    LOOP
        -- Drop the schema and all its contents
        EXECUTE format('DROP SCHEMA IF EXISTS %I CASCADE', schema_record.schema_name);
        
        -- Remove from tenant_schemas table
        DELETE FROM tenant_schemas 
        WHERE tenant_schemas.tenant_id = drop_tenant_schema.tenant_id 
        AND schema_name = schema_record.schema_name;
    END LOOP;
END;
$$ LANGUAGE plpgsql;

-- Function to set tenant context for RLS
CREATE OR REPLACE FUNCTION set_tenant_context(tenant_id UUID)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', tenant_id::TEXT, true);
END;
$$ LANGUAGE plpgsql;

-- Function to clear tenant context
CREATE OR REPLACE FUNCTION clear_tenant_context()
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant_id', '', true);
END;
$$ LANGUAGE plpgsql;

-- Function to get current tenant context
CREATE OR REPLACE FUNCTION get_current_tenant_id()
RETURNS UUID AS $$
BEGIN
    RETURN current_setting('app.current_tenant_id', true)::UUID;
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Create tenant management functions
CREATE OR REPLACE FUNCTION create_tenant_with_isolation(
    tenant_name VARCHAR(255),
    admin_email VARCHAR(255),
    subscription_tier subscription_tier DEFAULT 'free',
    isolation_level tenant_isolation_level DEFAULT 'row'
)
RETURNS UUID AS $$
DECLARE
    new_tenant_id UUID;
    tenant_slug VARCHAR(100);
BEGIN
    -- Generate tenant slug from name
    tenant_slug := LOWER(REGEXP_REPLACE(tenant_name, '[^a-zA-Z0-9]+', '-', 'g'));
    tenant_slug := TRIM(BOTH '-' FROM tenant_slug);
    
    -- Ensure slug is unique
    WHILE EXISTS (SELECT 1 FROM tenants WHERE slug = tenant_slug) LOOP
        tenant_slug := tenant_slug || '-' || EXTRACT(EPOCH FROM NOW())::INTEGER;
    END LOOP;
    
    -- Create tenant record
    INSERT INTO tenants (name, slug, admin_email, subscription_tier, isolation_level)
    VALUES (tenant_name, tenant_slug, admin_email, subscription_tier, isolation_level)
    RETURNING id INTO new_tenant_id;
    
    -- Create schema if using schema-based isolation
    IF isolation_level = 'schema' THEN
        PERFORM create_tenant_schema(new_tenant_id, NULL);
    END IF;
    
    RETURN new_tenant_id;
END;
$$ LANGUAGE plpgsql;

-- Create database health check function
CREATE OR REPLACE FUNCTION database_health_check()
RETURNS TABLE(
    check_name TEXT,
    status TEXT,
    details JSONB
) AS $$
BEGIN
    -- Check database connection
    RETURN QUERY SELECT 
        'database_connection'::TEXT,
        'healthy'::TEXT,
        jsonb_build_object('timestamp', NOW())::JSONB;
    
    -- Check tenant count
    RETURN QUERY SELECT 
        'tenant_count'::TEXT,
        'healthy'::TEXT,
        jsonb_build_object('count', (SELECT COUNT(*) FROM tenants))::JSONB;
    
    -- Check active sessions
    RETURN QUERY SELECT 
        'active_sessions'::TEXT,
        'healthy'::TEXT,
        jsonb_build_object('count', (SELECT COUNT(*) FROM user_sessions WHERE status = 'active'))::JSONB;
    
    -- Check RLS policies
    RETURN QUERY SELECT 
        'rls_policies'::TEXT,
        CASE WHEN COUNT(*) > 0 THEN 'healthy' ELSE 'warning' END::TEXT,
        jsonb_build_object('policy_count', COUNT(*))::JSONB
    FROM pg_policies 
    WHERE schemaname = 'public' AND policyname LIKE 'tenant_isolation_%';
END;
$$ LANGUAGE plpgsql;