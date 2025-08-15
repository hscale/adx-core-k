-- Security Service Database Schema

-- Create custom types
CREATE TYPE audit_event_category AS ENUM (
    'authentication',
    'authorization',
    'dataaccess',
    'datamodification',
    'systemaccess',
    'configuration',
    'security',
    'compliance',
    'privacy',
    'administrative'
);

CREATE TYPE audit_outcome AS ENUM (
    'success',
    'failure',
    'warning',
    'error'
);

CREATE TYPE compliance_report_type AS ENUM (
    'gdpr',
    'soc2',
    'iso27001',
    'hipaa',
    'pci',
    'custom'
);

CREATE TYPE compliance_status AS ENUM (
    'compliant',
    'noncompliant',
    'partiallycompliant',
    'underreview',
    'remediated'
);

CREATE TYPE risk_level AS ENUM (
    'low',
    'medium',
    'high',
    'critical'
);

CREATE TYPE gdpr_request_type AS ENUM (
    'dataexport',
    'datadeletion',
    'dataportability',
    'datarectification',
    'datarestriction',
    'dataobjection'
);

CREATE TYPE gdpr_request_status AS ENUM (
    'pending',
    'verified',
    'processing',
    'completed',
    'rejected',
    'expired'
);

CREATE TYPE deletion_method AS ENUM (
    'softdelete',
    'harddelete',
    'anonymize',
    'archive'
);

CREATE TYPE retention_job_status AS ENUM (
    'scheduled',
    'running',
    'completed',
    'failed',
    'cancelled'
);

CREATE TYPE scan_type AS ENUM (
    'vulnerability',
    'dependency',
    'configuration',
    'network',
    'application',
    'infrastructure'
);

CREATE TYPE scan_status AS ENUM (
    'queued',
    'running',
    'completed',
    'failed',
    'cancelled'
);

CREATE TYPE vulnerability_severity AS ENUM (
    'critical',
    'high',
    'medium',
    'low',
    'info'
);

CREATE TYPE zero_trust_policy_type AS ENUM (
    'networkaccess',
    'deviceverification',
    'userauthentication',
    'resourceaccess',
    'dataprotection'
);

CREATE TYPE security_event_type AS ENUM (
    'unauthorizedaccess',
    'suspiciousactivity',
    'policyviolation',
    'databreach',
    'malwaredetection',
    'anomalouslogin',
    'privilegeescalation',
    'dataexfiltration'
);

CREATE TYPE security_event_severity AS ENUM (
    'critical',
    'high',
    'medium',
    'low',
    'info'
);

CREATE TYPE security_event_status AS ENUM (
    'open',
    'inprogress',
    'resolved',
    'falsepositive',
    'suppressed'
);

-- Audit Logs Table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255),
    session_id VARCHAR(255),
    event_type VARCHAR(255) NOT NULL,
    event_category audit_event_category NOT NULL,
    resource_type VARCHAR(255) NOT NULL,
    resource_id VARCHAR(255),
    action VARCHAR(255) NOT NULL,
    outcome audit_outcome NOT NULL,
    ip_address INET,
    user_agent TEXT,
    request_id VARCHAR(255),
    details JSONB NOT NULL DEFAULT '{}',
    risk_score INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for audit logs
CREATE INDEX idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);
CREATE INDEX idx_audit_logs_event_category ON audit_logs(event_category);
CREATE INDEX idx_audit_logs_outcome ON audit_logs(outcome);
CREATE INDEX idx_audit_logs_risk_score ON audit_logs(risk_score);
CREATE INDEX idx_audit_logs_tenant_created ON audit_logs(tenant_id, created_at);

-- Compliance Reports Table
CREATE TABLE compliance_reports (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    report_type compliance_report_type NOT NULL,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    status compliance_status NOT NULL,
    findings JSONB NOT NULL DEFAULT '{}',
    recommendations JSONB NOT NULL DEFAULT '{}',
    risk_level risk_level NOT NULL,
    generated_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for compliance reports
CREATE INDEX idx_compliance_reports_tenant_id ON compliance_reports(tenant_id);
CREATE INDEX idx_compliance_reports_type ON compliance_reports(report_type);
CREATE INDEX idx_compliance_reports_status ON compliance_reports(status);
CREATE INDEX idx_compliance_reports_period ON compliance_reports(period_start, period_end);

-- GDPR Requests Table
CREATE TABLE gdpr_requests (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    request_type gdpr_request_type NOT NULL,
    status gdpr_request_status NOT NULL,
    requester_email VARCHAR(255) NOT NULL,
    verification_token VARCHAR(255),
    verified_at TIMESTAMPTZ,
    processed_at TIMESTAMPTZ,
    data_export_url TEXT,
    deletion_confirmation VARCHAR(255),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for GDPR requests
CREATE INDEX idx_gdpr_requests_tenant_id ON gdpr_requests(tenant_id);
CREATE INDEX idx_gdpr_requests_user_id ON gdpr_requests(user_id);
CREATE INDEX idx_gdpr_requests_status ON gdpr_requests(status);
CREATE INDEX idx_gdpr_requests_type ON gdpr_requests(request_type);
CREATE INDEX idx_gdpr_requests_created_at ON gdpr_requests(created_at);

-- Data Retention Policies Table
CREATE TABLE data_retention_policies (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    resource_type VARCHAR(255) NOT NULL,
    retention_period_days INTEGER NOT NULL CHECK (retention_period_days > 0),
    deletion_method deletion_method NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, resource_type)
);

-- Indexes for data retention policies
CREATE INDEX idx_retention_policies_tenant_id ON data_retention_policies(tenant_id);
CREATE INDEX idx_retention_policies_enabled ON data_retention_policies(enabled);
CREATE INDEX idx_retention_policies_resource_type ON data_retention_policies(resource_type);

-- Data Retention Jobs Table
CREATE TABLE data_retention_jobs (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    policy_id UUID NOT NULL REFERENCES data_retention_policies(id) ON DELETE CASCADE,
    resource_type VARCHAR(255) NOT NULL,
    scheduled_for TIMESTAMPTZ NOT NULL,
    status retention_job_status NOT NULL DEFAULT 'scheduled',
    records_processed BIGINT NOT NULL DEFAULT 0,
    records_deleted BIGINT NOT NULL DEFAULT 0,
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for data retention jobs
CREATE INDEX idx_retention_jobs_tenant_id ON data_retention_jobs(tenant_id);
CREATE INDEX idx_retention_jobs_policy_id ON data_retention_jobs(policy_id);
CREATE INDEX idx_retention_jobs_status ON data_retention_jobs(status);
CREATE INDEX idx_retention_jobs_scheduled_for ON data_retention_jobs(scheduled_for);

-- Security Scans Table
CREATE TABLE security_scans (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    scan_type scan_type NOT NULL,
    target VARCHAR(255) NOT NULL,
    status scan_status NOT NULL DEFAULT 'queued',
    severity_threshold VARCHAR(50) NOT NULL,
    vulnerabilities_found INTEGER NOT NULL DEFAULT 0,
    critical_count INTEGER NOT NULL DEFAULT 0,
    high_count INTEGER NOT NULL DEFAULT 0,
    medium_count INTEGER NOT NULL DEFAULT 0,
    low_count INTEGER NOT NULL DEFAULT 0,
    scan_results JSONB NOT NULL DEFAULT '{}',
    remediation_suggestions JSONB NOT NULL DEFAULT '[]',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for security scans
CREATE INDEX idx_security_scans_tenant_id ON security_scans(tenant_id);
CREATE INDEX idx_security_scans_status ON security_scans(status);
CREATE INDEX idx_security_scans_type ON security_scans(scan_type);
CREATE INDEX idx_security_scans_created_at ON security_scans(created_at);

-- Vulnerabilities Table
CREATE TABLE vulnerabilities (
    id UUID PRIMARY KEY,
    scan_id UUID NOT NULL REFERENCES security_scans(id) ON DELETE CASCADE,
    cve_id VARCHAR(255),
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    severity vulnerability_severity NOT NULL,
    cvss_score DECIMAL(3,1),
    affected_component VARCHAR(255) NOT NULL,
    fixed_version VARCHAR(255),
    references TEXT[] NOT NULL DEFAULT '{}',
    discovered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for vulnerabilities
CREATE INDEX idx_vulnerabilities_scan_id ON vulnerabilities(scan_id);
CREATE INDEX idx_vulnerabilities_severity ON vulnerabilities(severity);
CREATE INDEX idx_vulnerabilities_cve_id ON vulnerabilities(cve_id);
CREATE INDEX idx_vulnerabilities_cvss_score ON vulnerabilities(cvss_score);

-- Zero Trust Policies Table
CREATE TABLE zero_trust_policies (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    policy_type zero_trust_policy_type NOT NULL,
    conditions JSONB NOT NULL DEFAULT '{}',
    actions JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    priority INTEGER NOT NULL DEFAULT 0,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for zero trust policies
CREATE INDEX idx_zero_trust_policies_tenant_id ON zero_trust_policies(tenant_id);
CREATE INDEX idx_zero_trust_policies_enabled ON zero_trust_policies(enabled);
CREATE INDEX idx_zero_trust_policies_type ON zero_trust_policies(policy_type);
CREATE INDEX idx_zero_trust_policies_priority ON zero_trust_policies(priority);

-- Security Events Table
CREATE TABLE security_events (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    event_type security_event_type NOT NULL,
    severity security_event_severity NOT NULL,
    source_ip INET,
    user_id VARCHAR(255),
    device_id VARCHAR(255),
    resource VARCHAR(255),
    description TEXT NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    status security_event_status NOT NULL DEFAULT 'open',
    resolved_at TIMESTAMPTZ,
    resolved_by VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for security events
CREATE INDEX idx_security_events_tenant_id ON security_events(tenant_id);
CREATE INDEX idx_security_events_type ON security_events(event_type);
CREATE INDEX idx_security_events_severity ON security_events(severity);
CREATE INDEX idx_security_events_status ON security_events(status);
CREATE INDEX idx_security_events_created_at ON security_events(created_at);
CREATE INDEX idx_security_events_user_id ON security_events(user_id);

-- Device Registry Table (for zero trust device verification)
CREATE TABLE device_registry (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    device_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255),
    device_name VARCHAR(255),
    device_type VARCHAR(100),
    os_version VARCHAR(255),
    browser_version VARCHAR(255),
    screen_resolution VARCHAR(50),
    timezone VARCHAR(100),
    language VARCHAR(10),
    trust_score DECIMAL(3,2) NOT NULL DEFAULT 0.5,
    last_seen TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, device_id)
);

-- Indexes for device registry
CREATE INDEX idx_device_registry_tenant_id ON device_registry(tenant_id);
CREATE INDEX idx_device_registry_device_id ON device_registry(device_id);
CREATE INDEX idx_device_registry_user_id ON device_registry(user_id);
CREATE INDEX idx_device_registry_trust_score ON device_registry(trust_score);
CREATE INDEX idx_device_registry_last_seen ON device_registry(last_seen);

-- Encryption Keys Table (for key management)
CREATE TABLE encryption_keys (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    key_id VARCHAR(255) NOT NULL,
    key_type VARCHAR(100) NOT NULL,
    algorithm VARCHAR(100) NOT NULL,
    key_data BYTEA NOT NULL, -- Encrypted key data
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    rotated_at TIMESTAMPTZ,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    UNIQUE(tenant_id, key_id)
);

-- Indexes for encryption keys
CREATE INDEX idx_encryption_keys_tenant_id ON encryption_keys(tenant_id);
CREATE INDEX idx_encryption_keys_key_id ON encryption_keys(key_id);
CREATE INDEX idx_encryption_keys_status ON encryption_keys(status);
CREATE INDEX idx_encryption_keys_expires_at ON encryption_keys(expires_at);

-- Compliance Audit Trail Table
CREATE TABLE compliance_audit_trail (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    compliance_type VARCHAR(100) NOT NULL,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(255),
    resource_id VARCHAR(255),
    performed_by VARCHAR(255) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for compliance audit trail
CREATE INDEX idx_compliance_audit_tenant_id ON compliance_audit_trail(tenant_id);
CREATE INDEX idx_compliance_audit_type ON compliance_audit_trail(compliance_type);
CREATE INDEX idx_compliance_audit_timestamp ON compliance_audit_trail(timestamp);

-- Data Classification Table
CREATE TABLE data_classification (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    resource_type VARCHAR(255) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    classification_level VARCHAR(100) NOT NULL, -- public, internal, confidential, restricted
    data_categories TEXT[] NOT NULL DEFAULT '{}', -- pii, phi, financial, etc.
    retention_period_days INTEGER,
    encryption_required BOOLEAN NOT NULL DEFAULT false,
    access_restrictions JSONB NOT NULL DEFAULT '{}',
    classified_by VARCHAR(255) NOT NULL,
    classified_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    reviewed_at TIMESTAMPTZ,
    UNIQUE(tenant_id, resource_type, resource_id)
);

-- Indexes for data classification
CREATE INDEX idx_data_classification_tenant_id ON data_classification(tenant_id);
CREATE INDEX idx_data_classification_level ON data_classification(classification_level);
CREATE INDEX idx_data_classification_resource ON data_classification(resource_type, resource_id);
CREATE INDEX idx_data_classification_categories ON data_classification USING GIN(data_categories);

-- Security Metrics Table
CREATE TABLE security_metrics (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    metric_type VARCHAR(100) NOT NULL,
    metric_name VARCHAR(255) NOT NULL,
    metric_value DECIMAL(15,4) NOT NULL,
    metric_unit VARCHAR(50),
    tags JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for security metrics
CREATE INDEX idx_security_metrics_tenant_id ON security_metrics(tenant_id);
CREATE INDEX idx_security_metrics_type ON security_metrics(metric_type);
CREATE INDEX idx_security_metrics_name ON security_metrics(metric_name);
CREATE INDEX idx_security_metrics_recorded_at ON security_metrics(recorded_at);

-- Create functions for automatic timestamp updates
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at columns
CREATE TRIGGER update_compliance_reports_updated_at BEFORE UPDATE ON compliance_reports FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_gdpr_requests_updated_at BEFORE UPDATE ON gdpr_requests FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_data_retention_policies_updated_at BEFORE UPDATE ON data_retention_policies FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_zero_trust_policies_updated_at BEFORE UPDATE ON zero_trust_policies FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_device_registry_updated_at BEFORE UPDATE ON device_registry FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create views for common queries
CREATE VIEW audit_summary AS
SELECT 
    tenant_id,
    event_category,
    outcome,
    DATE_TRUNC('day', created_at) as audit_date,
    COUNT(*) as event_count,
    AVG(risk_score) as avg_risk_score
FROM audit_logs
GROUP BY tenant_id, event_category, outcome, DATE_TRUNC('day', created_at);

CREATE VIEW security_dashboard AS
SELECT 
    tenant_id,
    COUNT(CASE WHEN status = 'open' THEN 1 END) as open_events,
    COUNT(CASE WHEN severity = 'critical' AND status = 'open' THEN 1 END) as critical_events,
    COUNT(CASE WHEN severity = 'high' AND status = 'open' THEN 1 END) as high_events,
    MAX(created_at) as last_event_time
FROM security_events
GROUP BY tenant_id;

CREATE VIEW compliance_status_summary AS
SELECT 
    tenant_id,
    report_type,
    status,
    COUNT(*) as report_count,
    MAX(created_at) as latest_report
FROM compliance_reports
GROUP BY tenant_id, report_type, status;

-- Grant permissions (adjust as needed for your user)
-- GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO security_service_user;
-- GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO security_service_user;