-- Initialize ADX Core database with basic schema

-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- Create basic tables for development
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    admin_email VARCHAR(255) NOT NULL,
    subscription_tier VARCHAR(50) NOT NULL DEFAULT 'free',
    isolation_level VARCHAR(50) NOT NULL DEFAULT 'schema',
    is_active BOOLEAN NOT NULL DEFAULT true,
    settings JSONB DEFAULT '{}',
    quotas JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    display_name VARCHAR(255),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    roles TEXT[] DEFAULT ARRAY['user'],
    permissions TEXT[] DEFAULT ARRAY[],
    preferences JSONB DEFAULT '{}',
    quotas JSONB DEFAULT '{}',
    last_login TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(email, tenant_id)
);

CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    session_token VARCHAR(255) NOT NULL UNIQUE,
    device_id VARCHAR(255),
    ip_address INET,
    user_agent TEXT,
    expires_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_id VARCHAR(255) NOT NULL,
    run_id VARCHAR(255) NOT NULL,
    workflow_type VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    input JSONB,
    result JSONB,
    error_message TEXT,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(workflow_id, run_id)
);

-- Create indexes for performance
CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
CREATE INDEX idx_sessions_token ON sessions(session_token);
CREATE INDEX idx_workflow_executions_tenant_id ON workflow_executions(tenant_id);
CREATE INDEX idx_workflow_executions_user_id ON workflow_executions(user_id);
CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_workflow_executions_updated_at BEFORE UPDATE ON workflow_executions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Insert sample data for development
INSERT INTO tenants (id, name, admin_email, subscription_tier) VALUES 
    ('550e8400-e29b-41d4-a716-446655440000', 'Default Tenant', 'admin@adxcore.com', 'enterprise')
ON CONFLICT (name) DO NOTHING;

INSERT INTO users (id, email, password_hash, display_name, tenant_id, roles) VALUES 
    ('550e8400-e29b-41d4-a716-446655440001', 'admin@adxcore.com', '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/RK.s5uO.G', 'Admin User', '550e8400-e29b-41d4-a716-446655440000', ARRAY['admin'])
ON CONFLICT (email, tenant_id) DO NOTHING;