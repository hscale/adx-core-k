-- Auth Service specific schema
-- Authentication, authorization, and user management tables

-- Password reset tokens table
CREATE TABLE IF NOT EXISTS password_reset_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Email verification tokens table
CREATE TABLE IF NOT EXISTS email_verification_tokens (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    token VARCHAR(255) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- MFA (Multi-Factor Authentication) table
CREATE TABLE IF NOT EXISTS user_mfa (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    mfa_type VARCHAR(20) NOT NULL CHECK (mfa_type IN ('totp', 'sms', 'email', 'backup_codes')),
    secret_key VARCHAR(255), -- For TOTP
    phone_number VARCHAR(20), -- For SMS
    backup_codes TEXT[], -- For backup codes
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, mfa_type)
);

-- SSO providers table
CREATE TABLE IF NOT EXISTS sso_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    provider_name VARCHAR(50) NOT NULL,
    provider_type VARCHAR(20) NOT NULL CHECK (provider_type IN ('saml', 'oauth2', 'oidc', 'ldap')),
    configuration JSONB NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, provider_name)
);

-- SSO user mappings table
CREATE TABLE IF NOT EXISTS sso_user_mappings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    sso_provider_id UUID NOT NULL REFERENCES sso_providers(id) ON DELETE CASCADE,
    external_user_id VARCHAR(255) NOT NULL,
    external_email VARCHAR(255),
    external_attributes JSONB DEFAULT '{}',
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(sso_provider_id, external_user_id)
);

-- API keys table for service-to-service authentication
CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE, -- NULL for service keys
    key_name VARCHAR(100) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    key_prefix VARCHAR(20) NOT NULL,
    permissions TEXT[] DEFAULT '{}',
    rate_limit_per_hour INTEGER DEFAULT 1000,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Login attempts table for security monitoring
CREATE TABLE IF NOT EXISTS login_attempts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    email VARCHAR(255) NOT NULL,
    ip_address INET NOT NULL,
    user_agent TEXT,
    success BOOLEAN NOT NULL,
    failure_reason VARCHAR(100),
    attempted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Rate limiting table
CREATE TABLE IF NOT EXISTS rate_limits (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE CASCADE,
    resource_type VARCHAR(50) NOT NULL, -- 'login', 'api_call', 'password_reset', etc.
    identifier VARCHAR(255) NOT NULL, -- IP address, user ID, API key, etc.
    request_count INTEGER NOT NULL DEFAULT 1,
    window_start TIMESTAMPTZ NOT NULL,
    window_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(resource_type, identifier, window_start)
);

-- Enable RLS on auth service tables
ALTER TABLE password_reset_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE email_verification_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_mfa ENABLE ROW LEVEL SECURITY;
ALTER TABLE sso_providers ENABLE ROW LEVEL SECURITY;
ALTER TABLE sso_user_mappings ENABLE ROW LEVEL SECURITY;
ALTER TABLE api_keys ENABLE ROW LEVEL SECURITY;
ALTER TABLE login_attempts ENABLE ROW LEVEL SECURITY;
ALTER TABLE rate_limits ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for auth service tables
CREATE POLICY tenant_isolation_password_reset_tokens ON password_reset_tokens
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_email_verification_tokens ON email_verification_tokens
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_mfa ON user_mfa
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_sso_providers ON sso_providers
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_sso_user_mappings ON sso_user_mappings
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_api_keys ON api_keys
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_login_attempts ON login_attempts
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID OR tenant_id IS NULL);

CREATE POLICY tenant_isolation_rate_limits ON rate_limits
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID OR tenant_id IS NULL);

-- Create indexes for auth service tables
CREATE INDEX idx_password_reset_tokens_user_id ON password_reset_tokens(user_id);
CREATE INDEX idx_password_reset_tokens_token ON password_reset_tokens(token);
CREATE INDEX idx_password_reset_tokens_expires_at ON password_reset_tokens(expires_at);
CREATE INDEX idx_password_reset_tokens_tenant_id ON password_reset_tokens(tenant_id);

CREATE INDEX idx_email_verification_tokens_user_id ON email_verification_tokens(user_id);
CREATE INDEX idx_email_verification_tokens_token ON email_verification_tokens(token);
CREATE INDEX idx_email_verification_tokens_email ON email_verification_tokens(email);
CREATE INDEX idx_email_verification_tokens_tenant_id ON email_verification_tokens(tenant_id);

CREATE INDEX idx_user_mfa_user_id ON user_mfa(user_id);
CREATE INDEX idx_user_mfa_tenant_id ON user_mfa(tenant_id);
CREATE INDEX idx_user_mfa_type ON user_mfa(mfa_type);
CREATE INDEX idx_user_mfa_enabled ON user_mfa(is_enabled);

CREATE INDEX idx_sso_providers_tenant_id ON sso_providers(tenant_id);
CREATE INDEX idx_sso_providers_type ON sso_providers(provider_type);
CREATE INDEX idx_sso_providers_enabled ON sso_providers(is_enabled);

CREATE INDEX idx_sso_user_mappings_user_id ON sso_user_mappings(user_id);
CREATE INDEX idx_sso_user_mappings_tenant_id ON sso_user_mappings(tenant_id);
CREATE INDEX idx_sso_user_mappings_provider_id ON sso_user_mappings(sso_provider_id);
CREATE INDEX idx_sso_user_mappings_external_user_id ON sso_user_mappings(external_user_id);

CREATE INDEX idx_api_keys_tenant_id ON api_keys(tenant_id);
CREATE INDEX idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX idx_api_keys_hash ON api_keys(key_hash);
CREATE INDEX idx_api_keys_prefix ON api_keys(key_prefix);
CREATE INDEX idx_api_keys_active ON api_keys(is_active);
CREATE INDEX idx_api_keys_expires_at ON api_keys(expires_at);

CREATE INDEX idx_login_attempts_tenant_id ON login_attempts(tenant_id);
CREATE INDEX idx_login_attempts_email ON login_attempts(email);
CREATE INDEX idx_login_attempts_ip_address ON login_attempts(ip_address);
CREATE INDEX idx_login_attempts_attempted_at ON login_attempts(attempted_at);
CREATE INDEX idx_login_attempts_success ON login_attempts(success);

CREATE INDEX idx_rate_limits_tenant_id ON rate_limits(tenant_id);
CREATE INDEX idx_rate_limits_user_id ON rate_limits(user_id);
CREATE INDEX idx_rate_limits_api_key_id ON rate_limits(api_key_id);
CREATE INDEX idx_rate_limits_resource_identifier ON rate_limits(resource_type, identifier);
CREATE INDEX idx_rate_limits_window ON rate_limits(window_start, window_end);

-- Apply updated_at triggers
CREATE TRIGGER update_user_mfa_updated_at BEFORE UPDATE ON user_mfa FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_sso_providers_updated_at BEFORE UPDATE ON sso_providers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_sso_user_mappings_updated_at BEFORE UPDATE ON sso_user_mappings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_api_keys_updated_at BEFORE UPDATE ON api_keys FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_rate_limits_updated_at BEFORE UPDATE ON rate_limits FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();