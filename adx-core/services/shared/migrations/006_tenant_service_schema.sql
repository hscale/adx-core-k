-- Tenant Service specific schema
-- Tenant management, billing, and subscription tables

-- Tenant billing information table
CREATE TABLE IF NOT EXISTS tenant_billing (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    billing_email VARCHAR(255) NOT NULL,
    company_name VARCHAR(255),
    tax_id VARCHAR(100),
    billing_address JSONB DEFAULT '{}',
    payment_method_id VARCHAR(255), -- Stripe payment method ID
    subscription_id VARCHAR(255), -- Stripe subscription ID
    current_period_start TIMESTAMPTZ,
    current_period_end TIMESTAMPTZ,
    trial_end TIMESTAMPTZ,
    is_trial BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id)
);

-- Tenant usage tracking table
CREATE TABLE IF NOT EXISTS tenant_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    usage_type VARCHAR(50) NOT NULL, -- 'api_calls', 'storage_gb', 'users', 'workflows'
    usage_value BIGINT NOT NULL DEFAULT 0,
    quota_limit BIGINT,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, usage_type, period_start)
);

-- Tenant feature flags table
CREATE TABLE IF NOT EXISTS tenant_features (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    feature_name VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    configuration JSONB DEFAULT '{}',
    enabled_by UUID REFERENCES users(id) ON DELETE SET NULL,
    enabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, feature_name)
);

-- Tenant invitations table
CREATE TABLE IF NOT EXISTS tenant_invitations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    invited_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    invitation_token VARCHAR(255) NOT NULL UNIQUE,
    roles TEXT[] DEFAULT '{"user"}',
    permissions TEXT[] DEFAULT '{}',
    expires_at TIMESTAMPTZ NOT NULL,
    accepted_at TIMESTAMPTZ,
    accepted_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

-- Tenant domains table for custom domains
CREATE TABLE IF NOT EXISTS tenant_domains (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    domain VARCHAR(255) NOT NULL UNIQUE,
    is_verified BOOLEAN NOT NULL DEFAULT false,
    verification_token VARCHAR(255) NOT NULL,
    ssl_certificate_id VARCHAR(255),
    dns_records JSONB DEFAULT '{}',
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tenant branding table for white-label customization
CREATE TABLE IF NOT EXISTS tenant_branding (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    logo_url TEXT,
    favicon_url TEXT,
    primary_color VARCHAR(7), -- Hex color
    secondary_color VARCHAR(7), -- Hex color
    custom_css TEXT,
    email_templates JSONB DEFAULT '{}',
    terms_of_service_url TEXT,
    privacy_policy_url TEXT,
    support_email VARCHAR(255),
    support_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id)
);

-- Tenant webhooks table for external integrations
CREATE TABLE IF NOT EXISTS tenant_webhooks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    webhook_name VARCHAR(100) NOT NULL,
    endpoint_url TEXT NOT NULL,
    secret_key VARCHAR(255) NOT NULL,
    events TEXT[] NOT NULL, -- Array of event types to listen for
    is_active BOOLEAN NOT NULL DEFAULT true,
    retry_count INTEGER NOT NULL DEFAULT 3,
    timeout_seconds INTEGER NOT NULL DEFAULT 30,
    last_triggered_at TIMESTAMPTZ,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, webhook_name)
);

-- Tenant backup configurations table
CREATE TABLE IF NOT EXISTS tenant_backups (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    backup_name VARCHAR(100) NOT NULL,
    backup_type VARCHAR(20) NOT NULL CHECK (backup_type IN ('full', 'incremental', 'schema_only')),
    storage_location TEXT NOT NULL,
    backup_size_bytes BIGINT,
    encryption_key_id VARCHAR(255),
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'failed')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    error_message TEXT,
    retention_days INTEGER NOT NULL DEFAULT 30,
    created_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Tenant compliance settings table
CREATE TABLE IF NOT EXISTS tenant_compliance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    compliance_type VARCHAR(50) NOT NULL, -- 'gdpr', 'hipaa', 'sox', 'pci_dss'
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    configuration JSONB DEFAULT '{}',
    data_retention_days INTEGER,
    audit_log_retention_days INTEGER DEFAULT 2555, -- 7 years
    encryption_at_rest BOOLEAN NOT NULL DEFAULT true,
    encryption_in_transit BOOLEAN NOT NULL DEFAULT true,
    enabled_by UUID REFERENCES users(id) ON DELETE SET NULL,
    enabled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, compliance_type)
);

-- Enable RLS on tenant service tables
ALTER TABLE tenant_billing ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_usage ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_features ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_invitations ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_domains ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_branding ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_webhooks ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_backups ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_compliance ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for tenant service tables
CREATE POLICY tenant_isolation_tenant_billing ON tenant_billing
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_usage ON tenant_usage
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_features ON tenant_features
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_invitations ON tenant_invitations
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_domains ON tenant_domains
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_branding ON tenant_branding
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_webhooks ON tenant_webhooks
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_backups ON tenant_backups
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_tenant_compliance ON tenant_compliance
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create indexes for tenant service tables
CREATE INDEX idx_tenant_billing_tenant_id ON tenant_billing(tenant_id);
CREATE INDEX idx_tenant_billing_subscription_id ON tenant_billing(subscription_id);
CREATE INDEX idx_tenant_billing_trial ON tenant_billing(is_trial);
CREATE INDEX idx_tenant_billing_period_end ON tenant_billing(current_period_end);

CREATE INDEX idx_tenant_usage_tenant_id ON tenant_usage(tenant_id);
CREATE INDEX idx_tenant_usage_type ON tenant_usage(usage_type);
CREATE INDEX idx_tenant_usage_period ON tenant_usage(period_start, period_end);

CREATE INDEX idx_tenant_features_tenant_id ON tenant_features(tenant_id);
CREATE INDEX idx_tenant_features_name ON tenant_features(feature_name);
CREATE INDEX idx_tenant_features_enabled ON tenant_features(is_enabled);

CREATE INDEX idx_tenant_invitations_tenant_id ON tenant_invitations(tenant_id);
CREATE INDEX idx_tenant_invitations_email ON tenant_invitations(email);
CREATE INDEX idx_tenant_invitations_token ON tenant_invitations(invitation_token);
CREATE INDEX idx_tenant_invitations_expires_at ON tenant_invitations(expires_at);

CREATE INDEX idx_tenant_domains_tenant_id ON tenant_domains(tenant_id);
CREATE INDEX idx_tenant_domains_domain ON tenant_domains(domain);
CREATE INDEX idx_tenant_domains_verified ON tenant_domains(is_verified);

CREATE INDEX idx_tenant_branding_tenant_id ON tenant_branding(tenant_id);

CREATE INDEX idx_tenant_webhooks_tenant_id ON tenant_webhooks(tenant_id);
CREATE INDEX idx_tenant_webhooks_active ON tenant_webhooks(is_active);
CREATE INDEX idx_tenant_webhooks_events ON tenant_webhooks USING GIN(events);

CREATE INDEX idx_tenant_backups_tenant_id ON tenant_backups(tenant_id);
CREATE INDEX idx_tenant_backups_status ON tenant_backups(status);
CREATE INDEX idx_tenant_backups_created_at ON tenant_backups(created_at);

CREATE INDEX idx_tenant_compliance_tenant_id ON tenant_compliance(tenant_id);
CREATE INDEX idx_tenant_compliance_type ON tenant_compliance(compliance_type);
CREATE INDEX idx_tenant_compliance_enabled ON tenant_compliance(is_enabled);

-- Apply updated_at triggers
CREATE TRIGGER update_tenant_billing_updated_at BEFORE UPDATE ON tenant_billing FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tenant_usage_updated_at BEFORE UPDATE ON tenant_usage FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tenant_features_updated_at BEFORE UPDATE ON tenant_features FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tenant_domains_updated_at BEFORE UPDATE ON tenant_domains FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tenant_branding_updated_at BEFORE UPDATE ON tenant_branding FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tenant_webhooks_updated_at BEFORE UPDATE ON tenant_webhooks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tenant_backups_updated_at BEFORE UPDATE ON tenant_backups FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_tenant_compliance_updated_at BEFORE UPDATE ON tenant_compliance FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();