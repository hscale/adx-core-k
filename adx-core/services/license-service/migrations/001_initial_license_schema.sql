-- License and Quota Management Schema
-- This migration creates tables for license management, quota enforcement, and billing

-- License types and subscription tiers
CREATE TYPE subscription_tier AS ENUM ('free', 'professional', 'enterprise', 'custom');
CREATE TYPE license_status AS ENUM ('active', 'expired', 'suspended', 'cancelled', 'pending');
CREATE TYPE billing_cycle AS ENUM ('monthly', 'yearly', 'one_time', 'usage_based');
CREATE TYPE payment_status AS ENUM ('pending', 'completed', 'failed', 'refunded', 'cancelled');

-- Licenses table - core license information
CREATE TABLE licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    license_key VARCHAR(255) UNIQUE NOT NULL,
    subscription_tier subscription_tier NOT NULL DEFAULT 'free',
    status license_status NOT NULL DEFAULT 'pending',
    billing_cycle billing_cycle NOT NULL DEFAULT 'monthly',
    
    -- Pricing information
    base_price DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    
    -- License validity
    starts_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    auto_renew BOOLEAN NOT NULL DEFAULT true,
    
    -- Features and limits
    features JSONB NOT NULL DEFAULT '[]',
    custom_quotas JSONB,
    
    -- Billing information
    stripe_subscription_id VARCHAR(255),
    stripe_customer_id VARCHAR(255),
    paypal_subscription_id VARCHAR(255),
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by UUID,
    
    CONSTRAINT fk_licenses_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

-- Quota definitions - defines what can be limited
CREATE TABLE quota_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    unit VARCHAR(50) NOT NULL, -- 'requests', 'storage_gb', 'users', 'workflows', etc.
    category VARCHAR(50) NOT NULL, -- 'api', 'storage', 'compute', 'features'
    
    -- Default limits per tier
    free_limit BIGINT NOT NULL DEFAULT 0,
    professional_limit BIGINT NOT NULL DEFAULT 0,
    enterprise_limit BIGINT NOT NULL DEFAULT -1, -- -1 means unlimited
    
    -- Enforcement settings
    enforce_hard_limit BOOLEAN NOT NULL DEFAULT true,
    warning_threshold_percent INTEGER NOT NULL DEFAULT 80,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tenant quotas - actual quota assignments per tenant
CREATE TABLE tenant_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    quota_definition_id UUID NOT NULL,
    
    -- Current quota settings
    quota_limit BIGINT NOT NULL, -- -1 means unlimited
    current_usage BIGINT NOT NULL DEFAULT 0,
    
    -- Usage tracking
    last_reset_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reset_period_days INTEGER NOT NULL DEFAULT 30,
    
    -- Overrides
    custom_limit BIGINT, -- Override default tier limit
    notes TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_tenant_quotas_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT fk_tenant_quotas_definition FOREIGN KEY (quota_definition_id) REFERENCES quota_definitions(id) ON DELETE CASCADE,
    CONSTRAINT unique_tenant_quota UNIQUE (tenant_id, quota_definition_id)
);

-- Usage tracking - detailed usage logs
CREATE TABLE usage_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    quota_definition_id UUID NOT NULL,
    
    -- Usage details
    amount BIGINT NOT NULL DEFAULT 1,
    operation_type VARCHAR(100), -- 'api_call', 'file_upload', 'workflow_execution', etc.
    resource_id UUID, -- ID of the resource being used
    
    -- Context
    user_id UUID,
    ip_address INET,
    user_agent TEXT,
    
    -- Metadata
    metadata JSONB,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_usage_logs_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT fk_usage_logs_definition FOREIGN KEY (quota_definition_id) REFERENCES quota_definitions(id) ON DELETE CASCADE
);

-- Billing history - payment and invoice tracking
CREATE TABLE billing_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    license_id UUID NOT NULL,
    
    -- Invoice information
    invoice_number VARCHAR(100) UNIQUE NOT NULL,
    amount DECIMAL(10,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    tax_amount DECIMAL(10,2) NOT NULL DEFAULT 0.00,
    
    -- Billing period
    billing_period_start TIMESTAMPTZ NOT NULL,
    billing_period_end TIMESTAMPTZ NOT NULL,
    
    -- Payment information
    payment_status payment_status NOT NULL DEFAULT 'pending',
    payment_method VARCHAR(50), -- 'stripe', 'paypal', 'bank_transfer', 'check'
    payment_reference VARCHAR(255), -- External payment ID
    paid_at TIMESTAMPTZ,
    
    -- Usage-based billing details
    usage_details JSONB,
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_billing_history_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
    CONSTRAINT fk_billing_history_license FOREIGN KEY (license_id) REFERENCES licenses(id) ON DELETE CASCADE
);

-- Compliance audit logs
CREATE TABLE compliance_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    
    -- Event information
    event_type VARCHAR(100) NOT NULL, -- 'quota_exceeded', 'license_expired', 'payment_failed', etc.
    event_category VARCHAR(50) NOT NULL, -- 'quota', 'license', 'billing', 'compliance'
    severity VARCHAR(20) NOT NULL DEFAULT 'info', -- 'info', 'warning', 'error', 'critical'
    
    -- Event details
    description TEXT NOT NULL,
    details JSONB,
    
    -- Context
    user_id UUID,
    resource_id UUID,
    ip_address INET,
    
    -- Resolution
    resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_at TIMESTAMPTZ,
    resolved_by UUID,
    resolution_notes TEXT,
    
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT fk_compliance_logs_tenant FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX idx_licenses_tenant_id ON licenses(tenant_id);
CREATE INDEX idx_licenses_status ON licenses(status);
CREATE INDEX idx_licenses_expires_at ON licenses(expires_at);
CREATE INDEX idx_licenses_stripe_subscription ON licenses(stripe_subscription_id);

CREATE INDEX idx_tenant_quotas_tenant_id ON tenant_quotas(tenant_id);
CREATE INDEX idx_tenant_quotas_definition_id ON tenant_quotas(quota_definition_id);

CREATE INDEX idx_usage_logs_tenant_id ON usage_logs(tenant_id);
CREATE INDEX idx_usage_logs_recorded_at ON usage_logs(recorded_at);
CREATE INDEX idx_usage_logs_operation_type ON usage_logs(operation_type);

CREATE INDEX idx_billing_history_tenant_id ON billing_history(tenant_id);
CREATE INDEX idx_billing_history_license_id ON billing_history(license_id);
CREATE INDEX idx_billing_history_status ON billing_history(payment_status);
CREATE INDEX idx_billing_history_period ON billing_history(billing_period_start, billing_period_end);

CREATE INDEX idx_compliance_logs_tenant_id ON compliance_logs(tenant_id);
CREATE INDEX idx_compliance_logs_event_type ON compliance_logs(event_type);
CREATE INDEX idx_compliance_logs_created_at ON compliance_logs(created_at);
CREATE INDEX idx_compliance_logs_resolved ON compliance_logs(resolved);

-- Insert default quota definitions
INSERT INTO quota_definitions (name, description, unit, category, free_limit, professional_limit, enterprise_limit) VALUES
('api_calls_per_hour', 'API calls per hour limit', 'requests', 'api', 100, 1000, -1),
('api_calls_per_day', 'API calls per day limit', 'requests', 'api', 1000, 10000, -1),
('storage_gb', 'Storage limit in gigabytes', 'gigabytes', 'storage', 1, 100, -1),
('file_upload_size_mb', 'Maximum file upload size in MB', 'megabytes', 'storage', 10, 100, 1000),
('concurrent_workflows', 'Maximum concurrent workflows', 'workflows', 'compute', 1, 10, -1),
('workflow_executions_per_hour', 'Workflow executions per hour', 'workflows', 'compute', 10, 100, -1),
('users_per_tenant', 'Maximum users per tenant', 'users', 'features', 5, 50, -1),
('modules_installed', 'Maximum installed modules', 'modules', 'features', 3, 20, -1),
('custom_domains', 'Custom domain support', 'domains', 'features', 0, 1, -1),
('sso_connections', 'SSO connection limit', 'connections', 'features', 0, 5, -1);