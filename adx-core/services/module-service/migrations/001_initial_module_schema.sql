-- Module Service Database Schema
-- This migration creates the comprehensive module system database structure

-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Modules table - Core module information
CREATE TABLE modules (
    id VARCHAR(100) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    author_name VARCHAR(255) NOT NULL,
    author_email VARCHAR(255) NOT NULL,
    author_website VARCHAR(500),
    author_organization VARCHAR(255),
    category VARCHAR(50) NOT NULL,
    manifest_json JSONB NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'available',
    tenant_id VARCHAR(100), -- NULL for global modules
    package_url VARCHAR(500),
    package_hash VARCHAR(128),
    installation_id VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT modules_status_check CHECK (status IN (
        'available', 'installing', 'installed', 'activating', 'active',
        'deactivating', 'inactive', 'updating', 'uninstalling', 'failed', 'suspended'
    )),
    CONSTRAINT modules_category_check CHECK (category IN (
        'business_management', 'analytics', 'integration', 'workflow',
        'ui', 'security', 'storage', 'communication', 'development', 'other'
    ))
);

-- Module dependencies table
CREATE TABLE module_dependencies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    dependency_id VARCHAR(100) NOT NULL,
    version_requirement VARCHAR(100) NOT NULL,
    optional BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(module_id, dependency_id)
);

-- Module installations table - Track module installations per tenant
CREATE TABLE module_installations (
    id VARCHAR(100) PRIMARY KEY,
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    tenant_id VARCHAR(100) NOT NULL,
    version VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'installing',
    configuration_json JSONB,
    installation_path VARCHAR(500),
    sandbox_config_json JSONB,
    installed_by VARCHAR(100) NOT NULL,
    installed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    activated_at TIMESTAMP WITH TIME ZONE,
    last_used_at TIMESTAMP WITH TIME ZONE,
    
    CONSTRAINT installations_status_check CHECK (status IN (
        'installing', 'installed', 'activating', 'active',
        'deactivating', 'inactive', 'updating', 'uninstalling', 'failed'
    )),
    UNIQUE(module_id, tenant_id)
);

-- Module permissions table
CREATE TABLE module_permissions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    permission_type VARCHAR(50) NOT NULL,
    resource VARCHAR(255),
    granted BOOLEAN DEFAULT FALSE,
    granted_by VARCHAR(100),
    granted_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT permissions_type_check CHECK (permission_type IN (
        'database_read', 'database_write', 'api_external', 'file_read',
        'file_write', 'workflow_execute', 'tenant_data_access',
        'user_data_access', 'system_configuration', 'module_management', 'custom'
    ))
);

-- Module usage tracking table
CREATE TABLE module_usage (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    tenant_id VARCHAR(100) NOT NULL,
    usage_type VARCHAR(50) NOT NULL,
    usage_count BIGINT DEFAULT 0,
    resource_usage_json JSONB,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    INDEX idx_module_usage_module_tenant (module_id, tenant_id),
    INDEX idx_module_usage_recorded_at (recorded_at)
);

-- Module reviews table
CREATE TABLE module_reviews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    tenant_id VARCHAR(100) NOT NULL,
    user_id VARCHAR(100) NOT NULL,
    rating SMALLINT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    title VARCHAR(255),
    comment TEXT,
    helpful_count INTEGER DEFAULT 0,
    verified_purchase BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(module_id, tenant_id, user_id)
);

-- Module versions table - Track all versions of modules
CREATE TABLE module_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    changelog TEXT,
    package_url VARCHAR(500) NOT NULL,
    package_hash VARCHAR(128) NOT NULL,
    package_size_bytes BIGINT NOT NULL,
    manifest_json JSONB NOT NULL,
    security_scan_json JSONB,
    performance_metrics_json JSONB,
    compatibility_json JSONB,
    is_stable BOOLEAN DEFAULT TRUE,
    is_deprecated BOOLEAN DEFAULT FALSE,
    deprecation_reason TEXT,
    published_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(module_id, version)
);

-- Module marketplace table - Enhanced marketplace information
CREATE TABLE module_marketplace (
    id VARCHAR(100) PRIMARY KEY REFERENCES modules(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    long_description TEXT,
    current_version VARCHAR(50) NOT NULL,
    author_name VARCHAR(255) NOT NULL,
    author_email VARCHAR(255) NOT NULL,
    category VARCHAR(50) NOT NULL,
    subcategory VARCHAR(100),
    price_model VARCHAR(20),
    price_amount DECIMAL(10,2),
    price_currency VARCHAR(3),
    billing_period VARCHAR(20),
    rating_average REAL DEFAULT 0.0,
    rating_count INTEGER DEFAULT 0,
    download_count BIGINT DEFAULT 0,
    active_installation_count BIGINT DEFAULT 0,
    screenshots_json JSONB,
    demo_url VARCHAR(500),
    documentation_url VARCHAR(500),
    support_url VARCHAR(500),
    tags_json JSONB,
    featured BOOLEAN DEFAULT FALSE,
    verified BOOLEAN DEFAULT FALSE,
    published BOOLEAN DEFAULT FALSE,
    published_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT marketplace_price_model_check CHECK (price_model IN (
        'free', 'one_time', 'subscription', 'usage', 'enterprise'
    )),
    CONSTRAINT marketplace_billing_period_check CHECK (billing_period IN (
        'monthly', 'yearly', 'per_use'
    ))
);

-- Module workflows table - Track workflow executions
CREATE TABLE module_workflows (
    id VARCHAR(100) PRIMARY KEY,
    workflow_type VARCHAR(50) NOT NULL,
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    tenant_id VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    input_json JSONB NOT NULL,
    output_json JSONB,
    error_message TEXT,
    progress_json JSONB,
    started_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT workflows_status_check CHECK (status IN (
        'pending', 'running', 'completed', 'failed', 'cancelled', 'timed_out'
    )),
    CONSTRAINT workflows_type_check CHECK (workflow_type IN (
        'install', 'update', 'uninstall', 'activate', 'deactivate',
        'security_scan', 'marketplace_sync', 'payment_process'
    ))
);

-- Module sandbox table - Sandbox configurations and monitoring
CREATE TABLE module_sandbox (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    tenant_id VARCHAR(100) NOT NULL,
    sandbox_config_json JSONB NOT NULL,
    resource_usage_json JSONB,
    violations_json JSONB,
    last_violation_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    UNIQUE(module_id, tenant_id)
);

-- Module security scans table
CREATE TABLE module_security_scans (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    module_id VARCHAR(100) NOT NULL REFERENCES modules(id) ON DELETE CASCADE,
    version VARCHAR(50) NOT NULL,
    scan_type VARCHAR(50) NOT NULL,
    scanner_version VARCHAR(50) NOT NULL,
    passed BOOLEAN NOT NULL,
    score SMALLINT NOT NULL CHECK (score >= 0 AND score <= 100),
    vulnerabilities_json JSONB NOT NULL,
    scan_duration_seconds INTEGER NOT NULL,
    scanned_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    CONSTRAINT scans_scan_type_check CHECK (scan_type IN (
        'static_analysis', 'dependency_check', 'malware_scan',
        'license_check', 'comprehensive'
    ))
);

-- Indexes for performance
CREATE INDEX idx_modules_tenant_id ON modules(tenant_id);
CREATE INDEX idx_modules_category ON modules(category);
CREATE INDEX idx_modules_status ON modules(status);
CREATE INDEX idx_modules_created_at ON modules(created_at);

CREATE INDEX idx_module_installations_tenant_id ON module_installations(tenant_id);
CREATE INDEX idx_module_installations_status ON module_installations(status);
CREATE INDEX idx_module_installations_installed_at ON module_installations(installed_at);

CREATE INDEX idx_module_permissions_module_id ON module_permissions(module_id);
CREATE INDEX idx_module_permissions_type ON module_permissions(permission_type);

CREATE INDEX idx_module_reviews_module_id ON module_reviews(module_id);
CREATE INDEX idx_module_reviews_rating ON module_reviews(rating);
CREATE INDEX idx_module_reviews_created_at ON module_reviews(created_at);

CREATE INDEX idx_module_versions_module_id ON module_versions(module_id);
CREATE INDEX idx_module_versions_published_at ON module_versions(published_at);
CREATE INDEX idx_module_versions_stable ON module_versions(is_stable);

CREATE INDEX idx_module_marketplace_category ON module_marketplace(category);
CREATE INDEX idx_module_marketplace_featured ON module_marketplace(featured);
CREATE INDEX idx_module_marketplace_published ON module_marketplace(published);
CREATE INDEX idx_module_marketplace_rating ON module_marketplace(rating_average);
CREATE INDEX idx_module_marketplace_downloads ON module_marketplace(download_count);

CREATE INDEX idx_module_workflows_type ON module_workflows(workflow_type);
CREATE INDEX idx_module_workflows_status ON module_workflows(status);
CREATE INDEX idx_module_workflows_started_at ON module_workflows(started_at);

CREATE INDEX idx_module_sandbox_module_tenant ON module_sandbox(module_id, tenant_id);
CREATE INDEX idx_module_sandbox_violations ON module_sandbox(last_violation_at);

CREATE INDEX idx_module_security_scans_module_version ON module_security_scans(module_id, version);
CREATE INDEX idx_module_security_scans_passed ON module_security_scans(passed);
CREATE INDEX idx_module_security_scans_scanned_at ON module_security_scans(scanned_at);

-- Full-text search indexes
CREATE INDEX idx_modules_search ON modules USING gin(to_tsvector('english', name || ' ' || description));
CREATE INDEX idx_module_marketplace_search ON module_marketplace USING gin(to_tsvector('english', name || ' ' || description || ' ' || COALESCE(long_description, '')));

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_modules_updated_at BEFORE UPDATE ON modules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_module_reviews_updated_at BEFORE UPDATE ON module_reviews
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_module_marketplace_updated_at BEFORE UPDATE ON module_marketplace
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_module_sandbox_updated_at BEFORE UPDATE ON module_sandbox
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Views for common queries
CREATE VIEW module_stats AS
SELECT 
    m.id,
    m.name,
    m.category,
    m.status,
    COUNT(DISTINCT mi.tenant_id) as installation_count,
    AVG(mr.rating) as average_rating,
    COUNT(mr.id) as review_count,
    MAX(mi.last_used_at) as last_used_at
FROM modules m
LEFT JOIN module_installations mi ON m.id = mi.module_id
LEFT JOIN module_reviews mr ON m.id = mr.module_id
GROUP BY m.id, m.name, m.category, m.status;

CREATE VIEW active_module_installations AS
SELECT 
    mi.*,
    m.name as module_name,
    m.category,
    m.author_name
FROM module_installations mi
JOIN modules m ON mi.module_id = m.id
WHERE mi.status IN ('active', 'installed');

CREATE VIEW module_security_status AS
SELECT 
    m.id,
    m.name,
    m.version,
    mss.passed as security_passed,
    mss.score as security_score,
    mss.scanned_at as last_scan_at,
    CASE 
        WHEN mss.scanned_at < NOW() - INTERVAL '30 days' THEN 'outdated'
        WHEN mss.passed = false THEN 'failed'
        WHEN mss.score < 70 THEN 'warning'
        ELSE 'passed'
    END as security_status
FROM modules m
LEFT JOIN module_security_scans mss ON m.id = mss.module_id AND m.version = mss.version
WHERE mss.id IS NULL OR mss.id = (
    SELECT id FROM module_security_scans 
    WHERE module_id = m.id AND version = m.version 
    ORDER BY scanned_at DESC LIMIT 1
);