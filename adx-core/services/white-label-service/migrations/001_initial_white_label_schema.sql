-- White Label Service Database Schema

-- Custom domains table
CREATE TABLE custom_domains (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    domain VARCHAR(255) NOT NULL UNIQUE,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    verification_token VARCHAR(255) NOT NULL,
    ssl_certificate_id VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    
    CONSTRAINT custom_domains_status_check 
        CHECK (status IN ('pending', 'verifying', 'verified', 'failed', 'expired', 'suspended'))
);

-- Indexes for custom domains
CREATE INDEX idx_custom_domains_tenant_id ON custom_domains(tenant_id);
CREATE INDEX idx_custom_domains_domain ON custom_domains(domain);
CREATE INDEX idx_custom_domains_status ON custom_domains(status);
CREATE INDEX idx_custom_domains_expires_at ON custom_domains(expires_at);

-- White label branding table
CREATE TABLE white_label_branding (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL UNIQUE,
    brand_name VARCHAR(255) NOT NULL,
    logo_url TEXT,
    favicon_url TEXT,
    primary_color VARCHAR(7) NOT NULL DEFAULT '#3498db',
    secondary_color VARCHAR(7) NOT NULL DEFAULT '#2c3e50',
    accent_color VARCHAR(7) NOT NULL DEFAULT '#e74c3c',
    font_family VARCHAR(255) NOT NULL DEFAULT 'Arial, sans-serif',
    custom_css TEXT,
    email_templates JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT white_label_branding_primary_color_check 
        CHECK (primary_color ~ '^#[0-9A-Fa-f]{6}$'),
    CONSTRAINT white_label_branding_secondary_color_check 
        CHECK (secondary_color ~ '^#[0-9A-Fa-f]{6}$'),
    CONSTRAINT white_label_branding_accent_color_check 
        CHECK (accent_color ~ '^#[0-9A-Fa-f]{6}$')
);

-- Indexes for white label branding
CREATE INDEX idx_white_label_branding_tenant_id ON white_label_branding(tenant_id);

-- Branding assets table
CREATE TABLE branding_assets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    asset_type VARCHAR(50) NOT NULL,
    original_filename VARCHAR(255) NOT NULL,
    file_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    dimensions_width INTEGER,
    dimensions_height INTEGER,
    checksum VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT branding_assets_asset_type_check 
        CHECK (asset_type IN ('logo', 'favicon', 'background_image', 'email_header', 'email_footer', 'custom_icon')),
    CONSTRAINT branding_assets_file_size_check 
        CHECK (file_size > 0)
);

-- Indexes for branding assets
CREATE INDEX idx_branding_assets_tenant_id ON branding_assets(tenant_id);
CREATE INDEX idx_branding_assets_asset_type ON branding_assets(asset_type);
CREATE INDEX idx_branding_assets_checksum ON branding_assets(checksum);

-- Reseller hierarchies table
CREATE TABLE reseller_hierarchies (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_reseller_id UUID REFERENCES reseller_hierarchies(id) ON DELETE CASCADE,
    tenant_id VARCHAR(255) NOT NULL UNIQUE,
    reseller_name VARCHAR(255) NOT NULL,
    reseller_type VARCHAR(50) NOT NULL,
    commission_rate DECIMAL(5,4) NOT NULL DEFAULT 0.0000,
    revenue_share_model JSONB NOT NULL,
    support_contact JSONB NOT NULL,
    branding_overrides UUID REFERENCES white_label_branding(id) ON DELETE SET NULL,
    allowed_features TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT reseller_hierarchies_reseller_type_check 
        CHECK (reseller_type IN ('direct_reseller', 'sub_reseller', 'partner', 'distributor')),
    CONSTRAINT reseller_hierarchies_commission_rate_check 
        CHECK (commission_rate >= 0.0000 AND commission_rate <= 1.0000)
);

-- Indexes for reseller hierarchies
CREATE INDEX idx_reseller_hierarchies_parent_reseller_id ON reseller_hierarchies(parent_reseller_id);
CREATE INDEX idx_reseller_hierarchies_tenant_id ON reseller_hierarchies(tenant_id);
CREATE INDEX idx_reseller_hierarchies_reseller_type ON reseller_hierarchies(reseller_type);

-- Revenue sharing configurations table
CREATE TABLE revenue_sharing_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reseller_id UUID NOT NULL REFERENCES reseller_hierarchies(id) ON DELETE CASCADE,
    parent_reseller_id UUID REFERENCES reseller_hierarchies(id) ON DELETE CASCADE,
    revenue_share_model JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(reseller_id)
);

-- Indexes for revenue sharing configs
CREATE INDEX idx_revenue_sharing_configs_reseller_id ON revenue_sharing_configs(reseller_id);
CREATE INDEX idx_revenue_sharing_configs_parent_reseller_id ON revenue_sharing_configs(parent_reseller_id);

-- Support routing configurations table
CREATE TABLE support_routing_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reseller_id UUID NOT NULL REFERENCES reseller_hierarchies(id) ON DELETE CASCADE,
    support_contact JSONB NOT NULL,
    hierarchy_level INTEGER NOT NULL DEFAULT 1,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(reseller_id),
    CONSTRAINT support_routing_configs_hierarchy_level_check 
        CHECK (hierarchy_level >= 1 AND hierarchy_level <= 10)
);

-- Indexes for support routing configs
CREATE INDEX idx_support_routing_configs_reseller_id ON support_routing_configs(reseller_id);
CREATE INDEX idx_support_routing_configs_hierarchy_level ON support_routing_configs(hierarchy_level);

-- Branding backups table (for rollback functionality)
CREATE TABLE branding_backups (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    original_branding_id UUID,
    backup_data JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '30 days')
);

-- Indexes for branding backups
CREATE INDEX idx_branding_backups_tenant_id ON branding_backups(tenant_id);
CREATE INDEX idx_branding_backups_expires_at ON branding_backups(expires_at);

-- SSL certificates table
CREATE TABLE ssl_certificates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES custom_domains(id) ON DELETE CASCADE,
    certificate_id VARCHAR(255) NOT NULL UNIQUE,
    certificate_arn TEXT,
    provider VARCHAR(50) NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    issued_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    auto_renewal BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT ssl_certificates_provider_check 
        CHECK (provider IN ('letsencrypt', 'aws_acm', 'cloudflare')),
    CONSTRAINT ssl_certificates_status_check 
        CHECK (status IN ('pending', 'issued', 'failed', 'expired', 'revoked'))
);

-- Indexes for SSL certificates
CREATE INDEX idx_ssl_certificates_domain_id ON ssl_certificates(domain_id);
CREATE INDEX idx_ssl_certificates_certificate_id ON ssl_certificates(certificate_id);
CREATE INDEX idx_ssl_certificates_status ON ssl_certificates(status);
CREATE INDEX idx_ssl_certificates_expires_at ON ssl_certificates(expires_at);

-- DNS records table (for tracking DNS configuration)
CREATE TABLE dns_records (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES custom_domains(id) ON DELETE CASCADE,
    record_type VARCHAR(10) NOT NULL,
    name VARCHAR(255) NOT NULL,
    value TEXT NOT NULL,
    ttl INTEGER NOT NULL DEFAULT 300,
    provider VARCHAR(50),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT dns_records_record_type_check 
        CHECK (record_type IN ('A', 'AAAA', 'CNAME', 'TXT', 'MX', 'NS')),
    CONSTRAINT dns_records_ttl_check 
        CHECK (ttl >= 60 AND ttl <= 86400)
);

-- Indexes for DNS records
CREATE INDEX idx_dns_records_domain_id ON dns_records(domain_id);
CREATE INDEX idx_dns_records_record_type ON dns_records(record_type);
CREATE INDEX idx_dns_records_name ON dns_records(name);

-- Domain routing configurations table
CREATE TABLE domain_routing_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain_id UUID NOT NULL REFERENCES custom_domains(id) ON DELETE CASCADE,
    tenant_id VARCHAR(255) NOT NULL,
    ssl_certificate_id UUID REFERENCES ssl_certificates(id) ON DELETE SET NULL,
    auto_redirect BOOLEAN NOT NULL DEFAULT true,
    load_balancer_config JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(domain_id)
);

-- Indexes for domain routing configs
CREATE INDEX idx_domain_routing_configs_domain_id ON domain_routing_configs(domain_id);
CREATE INDEX idx_domain_routing_configs_tenant_id ON domain_routing_configs(tenant_id);

-- Function to automatically update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for updated_at columns
CREATE TRIGGER update_white_label_branding_updated_at 
    BEFORE UPDATE ON white_label_branding 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_reseller_hierarchies_updated_at 
    BEFORE UPDATE ON reseller_hierarchies 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_ssl_certificates_updated_at 
    BEFORE UPDATE ON ssl_certificates 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_domain_routing_configs_updated_at 
    BEFORE UPDATE ON domain_routing_configs 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to clean up expired branding backups
CREATE OR REPLACE FUNCTION cleanup_expired_branding_backups()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM branding_backups WHERE expires_at < NOW();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE custom_domains IS 'Stores custom domain configurations for white-label tenants';
COMMENT ON TABLE white_label_branding IS 'Stores branding configurations including colors, fonts, and assets';
COMMENT ON TABLE branding_assets IS 'Stores uploaded branding assets like logos and favicons';
COMMENT ON TABLE reseller_hierarchies IS 'Manages multi-level reseller relationships and configurations';
COMMENT ON TABLE revenue_sharing_configs IS 'Defines revenue sharing models for resellers';
COMMENT ON TABLE support_routing_configs IS 'Configures support routing based on reseller hierarchy';
COMMENT ON TABLE branding_backups IS 'Temporary backups of branding configurations for rollback functionality';
COMMENT ON TABLE ssl_certificates IS 'Tracks SSL certificates for custom domains';
COMMENT ON TABLE dns_records IS 'Stores DNS record configurations for custom domains';
COMMENT ON TABLE domain_routing_configs IS 'Configures load balancer routing for custom domains';