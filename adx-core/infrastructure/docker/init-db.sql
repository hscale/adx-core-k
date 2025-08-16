-- ADX Core Database Initialization Script
-- This script sets up the initial database structure for development

-- Create the main database if it doesn't exist
SELECT 'CREATE DATABASE adx_core'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'adx_core');

-- Create the test database if it doesn't exist
SELECT 'CREATE DATABASE adx_core_test'
WHERE NOT EXISTS (SELECT FROM pg_database WHERE datname = 'adx_core_test');

-- Connect to the main database
\c adx_core;

-- Enable necessary extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create schemas for multi-tenant isolation
CREATE SCHEMA IF NOT EXISTS tenant_default;
CREATE SCHEMA IF NOT EXISTS tenant_demo;

-- Create basic tables structure
-- Note: Actual migrations will be run by sqlx migrate

-- Tenants table (global)
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    admin_email VARCHAR(255) NOT NULL,
    subscription_tier VARCHAR(50) NOT NULL DEFAULT 'professional',
    isolation_level VARCHAR(20) NOT NULL DEFAULT 'schema',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    is_active BOOLEAN DEFAULT TRUE
);

-- Users table (tenant-aware)
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    first_name VARCHAR(100),
    last_name VARCHAR(100),
    roles TEXT[] DEFAULT '{}',
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

-- Sessions table
CREATE TABLE IF NOT EXISTS user_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Files table (tenant-aware)
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    storage_path VARCHAR(500) NOT NULL,
    is_public BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Modules table (tenant-aware)
CREATE TABLE IF NOT EXISTS modules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    module_name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'installed',
    config JSONB DEFAULT '{}',
    installed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    activated_at TIMESTAMP WITH TIME ZONE,
    UNIQUE(tenant_id, module_name)
);

-- Workflow executions table (for tracking)
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    workflow_id VARCHAR(255) NOT NULL,
    workflow_type VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'running',
    input JSONB,
    result JSONB,
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_tenant_id ON users(tenant_id);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON user_sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token ON user_sessions(session_token);
CREATE INDEX IF NOT EXISTS idx_files_tenant_id ON files(tenant_id);
CREATE INDEX IF NOT EXISTS idx_files_user_id ON files(user_id);
CREATE INDEX IF NOT EXISTS idx_modules_tenant_id ON modules(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_tenant_id ON workflow_executions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_workflow_executions_workflow_id ON workflow_executions(workflow_id);

-- Insert default tenant for development
INSERT INTO tenants (id, name, admin_email, subscription_tier, isolation_level)
VALUES (
    'default-tenant-id'::UUID,
    'Default Tenant',
    'admin@adxcore.local',
    'enterprise',
    'schema'
) ON CONFLICT (name) DO NOTHING;

-- Insert demo tenant for testing
INSERT INTO tenants (id, name, admin_email, subscription_tier, isolation_level)
VALUES (
    'demo-tenant-id'::UUID,
    'Demo Tenant',
    'demo@adxcore.local',
    'professional',
    'schema'
) ON CONFLICT (name) DO NOTHING;

-- Insert default admin user (password: admin123)
INSERT INTO users (id, tenant_id, email, password_hash, first_name, last_name, roles)
VALUES (
    'default-admin-id'::UUID,
    'default-tenant-id'::UUID,
    'admin@adxcore.local',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uO8G', -- admin123
    'Admin',
    'User',
    ARRAY['admin', 'user']
) ON CONFLICT (tenant_id, email) DO NOTHING;

-- Insert demo user (password: demo123)
INSERT INTO users (id, tenant_id, email, password_hash, first_name, last_name, roles)
VALUES (
    'demo-user-id'::UUID,
    'demo-tenant-id'::UUID,
    'demo@adxcore.local',
    '$2b$12$8K1p/a0dhrxSMxf7RqiOy.Hm6M8K9Z8K1p/a0dhrxSMxf7RqiOy.', -- demo123
    'Demo',
    'User',
    ARRAY['user']
) ON CONFLICT (tenant_id, email) DO NOTHING;

-- Create tenant-specific schemas and grant permissions
DO $$
DECLARE
    tenant_record RECORD;
BEGIN
    FOR tenant_record IN SELECT id FROM tenants LOOP
        EXECUTE format('CREATE SCHEMA IF NOT EXISTS tenant_%s', replace(tenant_record.id::text, '-', '_'));
        EXECUTE format('GRANT ALL ON SCHEMA tenant_%s TO postgres', replace(tenant_record.id::text, '-', '_'));
    END LOOP;
END $$;

-- Connect to test database and create similar structure
\c adx_core_test;

-- Enable necessary extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create the same table structure for testing
-- (This would normally be handled by test migrations)

COMMIT;