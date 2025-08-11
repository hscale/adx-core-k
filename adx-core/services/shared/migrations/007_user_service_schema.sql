-- User Service specific schema
-- User profiles, preferences, and social features

-- User profiles table for extended user information
CREATE TABLE IF NOT EXISTS user_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    display_name VARCHAR(255),
    bio TEXT,
    avatar_url TEXT,
    cover_image_url TEXT,
    location VARCHAR(255),
    website_url TEXT,
    timezone VARCHAR(50) DEFAULT 'UTC',
    language VARCHAR(10) DEFAULT 'en',
    date_format VARCHAR(20) DEFAULT 'YYYY-MM-DD',
    time_format VARCHAR(10) DEFAULT '24h',
    phone_number VARCHAR(20),
    phone_verified_at TIMESTAMPTZ,
    birth_date DATE,
    gender VARCHAR(20),
    job_title VARCHAR(255),
    department VARCHAR(255),
    manager_id UUID REFERENCES users(id) ON DELETE SET NULL,
    hire_date DATE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, tenant_id)
);

-- User preferences table for application settings
CREATE TABLE IF NOT EXISTS user_preferences (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    preference_category VARCHAR(50) NOT NULL, -- 'ui', 'notifications', 'privacy', 'workflow'
    preference_key VARCHAR(100) NOT NULL,
    preference_value JSONB NOT NULL,
    is_inherited BOOLEAN NOT NULL DEFAULT false, -- Inherited from tenant defaults
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, tenant_id, preference_category, preference_key)
);

-- User notification settings table
CREATE TABLE IF NOT EXISTS user_notification_settings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    notification_type VARCHAR(50) NOT NULL, -- 'email', 'push', 'sms', 'in_app'
    event_category VARCHAR(50) NOT NULL, -- 'workflow', 'file', 'tenant', 'security'
    event_name VARCHAR(100) NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    delivery_schedule VARCHAR(20) DEFAULT 'immediate', -- 'immediate', 'daily', 'weekly', 'never'
    quiet_hours_start TIME,
    quiet_hours_end TIME,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, tenant_id, notification_type, event_category, event_name)
);

-- User activity log table for tracking user actions
CREATE TABLE IF NOT EXISTS user_activity_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    activity_type VARCHAR(50) NOT NULL, -- 'login', 'logout', 'file_upload', 'workflow_start'
    activity_description TEXT,
    resource_type VARCHAR(50), -- 'file', 'workflow', 'tenant', 'user'
    resource_id UUID,
    metadata JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    session_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User teams table for team organization
CREATE TABLE IF NOT EXISTS user_teams (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    team_name VARCHAR(255) NOT NULL,
    team_description TEXT,
    team_lead_id UUID REFERENCES users(id) ON DELETE SET NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, team_name)
);

-- User team memberships table
CREATE TABLE IF NOT EXISTS user_team_memberships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    team_id UUID NOT NULL REFERENCES user_teams(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member', -- 'member', 'lead', 'admin'
    joined_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    left_at TIMESTAMPTZ,
    is_active BOOLEAN NOT NULL DEFAULT true,
    UNIQUE(team_id, user_id)
);

-- User skills table for competency tracking
CREATE TABLE IF NOT EXISTS user_skills (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    skill_name VARCHAR(100) NOT NULL,
    skill_category VARCHAR(50), -- 'technical', 'soft', 'domain'
    proficiency_level INTEGER CHECK (proficiency_level >= 1 AND proficiency_level <= 5),
    years_experience INTEGER,
    is_certified BOOLEAN NOT NULL DEFAULT false,
    certification_name VARCHAR(255),
    certification_date DATE,
    verified_by UUID REFERENCES users(id) ON DELETE SET NULL,
    verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, tenant_id, skill_name)
);

-- User connections table for social features
CREATE TABLE IF NOT EXISTS user_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    requester_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    requestee_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    connection_type VARCHAR(20) NOT NULL DEFAULT 'colleague', -- 'colleague', 'mentor', 'friend'
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'accepted', 'declined', 'blocked')),
    message TEXT,
    requested_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    responded_at TIMESTAMPTZ,
    UNIQUE(requester_id, requestee_id, tenant_id),
    CHECK (requester_id != requestee_id)
);

-- User saved searches table
CREATE TABLE IF NOT EXISTS user_saved_searches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    search_name VARCHAR(255) NOT NULL,
    search_type VARCHAR(50) NOT NULL, -- 'files', 'users', 'workflows', 'global'
    search_query JSONB NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, tenant_id, search_name)
);

-- User bookmarks table for quick access
CREATE TABLE IF NOT EXISTS user_bookmarks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    bookmark_name VARCHAR(255) NOT NULL,
    bookmark_type VARCHAR(50) NOT NULL, -- 'file', 'workflow', 'user', 'external_url'
    resource_id UUID,
    external_url TEXT,
    description TEXT,
    tags TEXT[] DEFAULT '{}',
    is_favorite BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, tenant_id, bookmark_name)
);

-- Enable RLS on user service tables
ALTER TABLE user_profiles ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_preferences ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_notification_settings ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_activity_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_teams ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_team_memberships ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_skills ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_connections ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_saved_searches ENABLE ROW LEVEL SECURITY;
ALTER TABLE user_bookmarks ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for user service tables
CREATE POLICY tenant_isolation_user_profiles ON user_profiles
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_preferences ON user_preferences
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_notification_settings ON user_notification_settings
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_activity_log ON user_activity_log
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_teams ON user_teams
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_team_memberships ON user_team_memberships
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_skills ON user_skills
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_connections ON user_connections
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_saved_searches ON user_saved_searches
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_user_bookmarks ON user_bookmarks
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create indexes for user service tables
CREATE INDEX idx_user_profiles_user_id ON user_profiles(user_id);
CREATE INDEX idx_user_profiles_tenant_id ON user_profiles(tenant_id);
CREATE INDEX idx_user_profiles_display_name ON user_profiles(display_name);
CREATE INDEX idx_user_profiles_manager_id ON user_profiles(manager_id);

CREATE INDEX idx_user_preferences_user_id ON user_preferences(user_id);
CREATE INDEX idx_user_preferences_tenant_id ON user_preferences(tenant_id);
CREATE INDEX idx_user_preferences_category ON user_preferences(preference_category);
CREATE INDEX idx_user_preferences_key ON user_preferences(preference_key);

CREATE INDEX idx_user_notification_settings_user_id ON user_notification_settings(user_id);
CREATE INDEX idx_user_notification_settings_tenant_id ON user_notification_settings(tenant_id);
CREATE INDEX idx_user_notification_settings_type ON user_notification_settings(notification_type);
CREATE INDEX idx_user_notification_settings_enabled ON user_notification_settings(is_enabled);

CREATE INDEX idx_user_activity_log_user_id ON user_activity_log(user_id);
CREATE INDEX idx_user_activity_log_tenant_id ON user_activity_log(tenant_id);
CREATE INDEX idx_user_activity_log_activity_type ON user_activity_log(activity_type);
CREATE INDEX idx_user_activity_log_created_at ON user_activity_log(created_at);
CREATE INDEX idx_user_activity_log_resource ON user_activity_log(resource_type, resource_id);

CREATE INDEX idx_user_teams_tenant_id ON user_teams(tenant_id);
CREATE INDEX idx_user_teams_team_lead_id ON user_teams(team_lead_id);
CREATE INDEX idx_user_teams_active ON user_teams(is_active);

CREATE INDEX idx_user_team_memberships_team_id ON user_team_memberships(team_id);
CREATE INDEX idx_user_team_memberships_user_id ON user_team_memberships(user_id);
CREATE INDEX idx_user_team_memberships_tenant_id ON user_team_memberships(tenant_id);
CREATE INDEX idx_user_team_memberships_active ON user_team_memberships(is_active);

CREATE INDEX idx_user_skills_user_id ON user_skills(user_id);
CREATE INDEX idx_user_skills_tenant_id ON user_skills(tenant_id);
CREATE INDEX idx_user_skills_name ON user_skills(skill_name);
CREATE INDEX idx_user_skills_category ON user_skills(skill_category);
CREATE INDEX idx_user_skills_proficiency ON user_skills(proficiency_level);

CREATE INDEX idx_user_connections_requester_id ON user_connections(requester_id);
CREATE INDEX idx_user_connections_requestee_id ON user_connections(requestee_id);
CREATE INDEX idx_user_connections_tenant_id ON user_connections(tenant_id);
CREATE INDEX idx_user_connections_status ON user_connections(status);

CREATE INDEX idx_user_saved_searches_user_id ON user_saved_searches(user_id);
CREATE INDEX idx_user_saved_searches_tenant_id ON user_saved_searches(tenant_id);
CREATE INDEX idx_user_saved_searches_type ON user_saved_searches(search_type);
CREATE INDEX idx_user_saved_searches_default ON user_saved_searches(is_default);

CREATE INDEX idx_user_bookmarks_user_id ON user_bookmarks(user_id);
CREATE INDEX idx_user_bookmarks_tenant_id ON user_bookmarks(tenant_id);
CREATE INDEX idx_user_bookmarks_type ON user_bookmarks(bookmark_type);
CREATE INDEX idx_user_bookmarks_favorite ON user_bookmarks(is_favorite);
CREATE INDEX idx_user_bookmarks_tags ON user_bookmarks USING GIN(tags);

-- Apply updated_at triggers
CREATE TRIGGER update_user_profiles_updated_at BEFORE UPDATE ON user_profiles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_preferences_updated_at BEFORE UPDATE ON user_preferences FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_notification_settings_updated_at BEFORE UPDATE ON user_notification_settings FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_teams_updated_at BEFORE UPDATE ON user_teams FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_skills_updated_at BEFORE UPDATE ON user_skills FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_saved_searches_updated_at BEFORE UPDATE ON user_saved_searches FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_user_bookmarks_updated_at BEFORE UPDATE ON user_bookmarks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();