-- File Service specific schema
-- File storage, processing, and management tables

-- File versions table for version control
CREATE TABLE IF NOT EXISTS file_versions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    filename VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    storage_path TEXT NOT NULL,
    checksum VARCHAR(64) NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(file_id, version_number)
);

-- File shares table for sharing files with external users
CREATE TABLE IF NOT EXISTS file_shares (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    share_token VARCHAR(255) NOT NULL UNIQUE,
    share_type VARCHAR(20) NOT NULL CHECK (share_type IN ('public', 'password', 'email', 'time_limited')),
    password_hash VARCHAR(255), -- For password-protected shares
    allowed_emails TEXT[], -- For email-restricted shares
    download_limit INTEGER, -- Maximum number of downloads
    download_count INTEGER NOT NULL DEFAULT 0,
    expires_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- File processing jobs table for async file processing
CREATE TABLE IF NOT EXISTS file_processing_jobs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    job_type VARCHAR(50) NOT NULL, -- 'thumbnail', 'virus_scan', 'metadata_extraction', 'conversion'
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')),
    input_parameters JSONB DEFAULT '{}',
    output_data JSONB DEFAULT '{}',
    error_message TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- File thumbnails table
CREATE TABLE IF NOT EXISTS file_thumbnails (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    thumbnail_size VARCHAR(20) NOT NULL, -- 'small', 'medium', 'large'
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    storage_path TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(file_id, thumbnail_size)
);

-- File tags table for organization
CREATE TABLE IF NOT EXISTS file_tags (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7), -- Hex color code
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, name)
);

-- File tag assignments table
CREATE TABLE IF NOT EXISTS file_tag_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES file_tags(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(file_id, tag_id)
);

-- File folders table for organization
CREATE TABLE IF NOT EXISTS file_folders (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    parent_folder_id UUID REFERENCES file_folders(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    path TEXT NOT NULL, -- Full path for quick lookups
    description TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, parent_folder_id, name)
);

-- File folder assignments table
CREATE TABLE IF NOT EXISTS file_folder_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    folder_id UUID NOT NULL REFERENCES file_folders(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    assigned_by UUID NOT NULL REFERENCES users(id),
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(file_id, folder_id)
);

-- File access logs for audit and analytics
CREATE TABLE IF NOT EXISTS file_access_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID NOT NULL REFERENCES files(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    access_type VARCHAR(20) NOT NULL, -- 'view', 'download', 'upload', 'delete', 'share'
    ip_address INET,
    user_agent TEXT,
    share_token VARCHAR(255), -- If accessed via share link
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Storage providers configuration table
CREATE TABLE IF NOT EXISTS storage_providers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    provider_name VARCHAR(50) NOT NULL,
    provider_type VARCHAR(20) NOT NULL CHECK (provider_type IN ('local', 's3', 'gcs', 'azure', 'ftp')),
    configuration JSONB NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, provider_name)
);

-- Enable RLS on file service tables
ALTER TABLE file_versions ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_shares ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_processing_jobs ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_thumbnails ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_tags ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_tag_assignments ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_folders ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_folder_assignments ENABLE ROW LEVEL SECURITY;
ALTER TABLE file_access_logs ENABLE ROW LEVEL SECURITY;
ALTER TABLE storage_providers ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for file service tables
CREATE POLICY tenant_isolation_file_versions ON file_versions
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_shares ON file_shares
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_processing_jobs ON file_processing_jobs
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_thumbnails ON file_thumbnails
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_tags ON file_tags
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_tag_assignments ON file_tag_assignments
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_folders ON file_folders
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_folder_assignments ON file_folder_assignments
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_file_access_logs ON file_access_logs
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_storage_providers ON storage_providers
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create indexes for file service tables
CREATE INDEX idx_file_versions_file_id ON file_versions(file_id);
CREATE INDEX idx_file_versions_tenant_id ON file_versions(tenant_id);
CREATE INDEX idx_file_versions_version_number ON file_versions(version_number);
CREATE INDEX idx_file_versions_created_at ON file_versions(created_at);

CREATE INDEX idx_file_shares_file_id ON file_shares(file_id);
CREATE INDEX idx_file_shares_tenant_id ON file_shares(tenant_id);
CREATE INDEX idx_file_shares_token ON file_shares(share_token);
CREATE INDEX idx_file_shares_type ON file_shares(share_type);
CREATE INDEX idx_file_shares_active ON file_shares(is_active);
CREATE INDEX idx_file_shares_expires_at ON file_shares(expires_at);

CREATE INDEX idx_file_processing_jobs_file_id ON file_processing_jobs(file_id);
CREATE INDEX idx_file_processing_jobs_tenant_id ON file_processing_jobs(tenant_id);
CREATE INDEX idx_file_processing_jobs_type ON file_processing_jobs(job_type);
CREATE INDEX idx_file_processing_jobs_status ON file_processing_jobs(status);
CREATE INDEX idx_file_processing_jobs_created_at ON file_processing_jobs(created_at);

CREATE INDEX idx_file_thumbnails_file_id ON file_thumbnails(file_id);
CREATE INDEX idx_file_thumbnails_tenant_id ON file_thumbnails(tenant_id);
CREATE INDEX idx_file_thumbnails_size ON file_thumbnails(thumbnail_size);

CREATE INDEX idx_file_tags_tenant_id ON file_tags(tenant_id);
CREATE INDEX idx_file_tags_name ON file_tags(name);

CREATE INDEX idx_file_tag_assignments_file_id ON file_tag_assignments(file_id);
CREATE INDEX idx_file_tag_assignments_tag_id ON file_tag_assignments(tag_id);
CREATE INDEX idx_file_tag_assignments_tenant_id ON file_tag_assignments(tenant_id);

CREATE INDEX idx_file_folders_tenant_id ON file_folders(tenant_id);
CREATE INDEX idx_file_folders_parent_id ON file_folders(parent_folder_id);
CREATE INDEX idx_file_folders_path ON file_folders(path);

CREATE INDEX idx_file_folder_assignments_file_id ON file_folder_assignments(file_id);
CREATE INDEX idx_file_folder_assignments_folder_id ON file_folder_assignments(folder_id);
CREATE INDEX idx_file_folder_assignments_tenant_id ON file_folder_assignments(tenant_id);

CREATE INDEX idx_file_access_logs_file_id ON file_access_logs(file_id);
CREATE INDEX idx_file_access_logs_tenant_id ON file_access_logs(tenant_id);
CREATE INDEX idx_file_access_logs_user_id ON file_access_logs(user_id);
CREATE INDEX idx_file_access_logs_access_type ON file_access_logs(access_type);
CREATE INDEX idx_file_access_logs_accessed_at ON file_access_logs(accessed_at);

CREATE INDEX idx_storage_providers_tenant_id ON storage_providers(tenant_id);
CREATE INDEX idx_storage_providers_type ON storage_providers(provider_type);
CREATE INDEX idx_storage_providers_default ON storage_providers(is_default);
CREATE INDEX idx_storage_providers_enabled ON storage_providers(is_enabled);

-- Apply updated_at triggers
CREATE TRIGGER update_file_shares_updated_at BEFORE UPDATE ON file_shares FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_file_processing_jobs_updated_at BEFORE UPDATE ON file_processing_jobs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_file_folders_updated_at BEFORE UPDATE ON file_folders FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_storage_providers_updated_at BEFORE UPDATE ON storage_providers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();