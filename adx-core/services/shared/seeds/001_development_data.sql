-- Development seed data for ADX Core
-- This script creates sample data for development and testing

-- Insert sample tenants
INSERT INTO tenants (id, name, slug, admin_email, subscription_tier, isolation_level, settings, quotas, features, is_active) VALUES
(
    '550e8400-e29b-41d4-a716-446655440001',
    'Acme Corporation',
    'acme-corp',
    'admin@acme.com',
    'enterprise',
    'row',
    '{"theme": "dark", "timezone": "UTC", "language": "en"}',
    '{"max_users": 1000, "max_storage_gb": 1000, "max_api_calls_per_hour": 10000, "max_workflows_per_hour": 500}',
    '{"advanced_analytics", "custom_branding", "sso", "api_access", "workflow_automation"}',
    true
),
(
    '550e8400-e29b-41d4-a716-446655440002',
    'TechStart Inc',
    'techstart-inc',
    'admin@techstart.com',
    'professional',
    'row',
    '{"theme": "light", "timezone": "America/New_York", "language": "en"}',
    '{"max_users": 100, "max_storage_gb": 100, "max_api_calls_per_hour": 5000, "max_workflows_per_hour": 100}',
    '{"basic_analytics", "file_sharing", "workflow_automation"}',
    true
),
(
    '550e8400-e29b-41d4-a716-446655440003',
    'Global Solutions Ltd',
    'global-solutions',
    'admin@globalsolutions.com',
    'free',
    'row',
    '{"theme": "light", "timezone": "Europe/London", "language": "en"}',
    '{"max_users": 10, "max_storage_gb": 5, "max_api_calls_per_hour": 1000, "max_workflows_per_hour": 20}',
    '{"basic_features"}',
    true
);

-- Insert sample users
INSERT INTO users (id, tenant_id, email, password_hash, first_name, last_name, status, roles, permissions, preferences) VALUES
-- Acme Corporation users
(
    '660e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'admin@acme.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: admin123
    'John',
    'Admin',
    'active',
    '{"admin", "user"}',
    '{"tenant:admin", "user:admin", "file:admin", "workflow:admin"}',
    '{"theme": "dark", "notifications": true, "language": "en"}'
),
(
    '660e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    'manager@acme.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: admin123
    'Sarah',
    'Manager',
    'active',
    '{"manager", "user"}',
    '{"user:read", "user:write", "file:read", "file:write", "workflow:execute"}',
    '{"theme": "light", "notifications": true, "language": "en"}'
),
(
    '660e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440001',
    'user@acme.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: admin123
    'Mike',
    'User',
    'active',
    '{"user"}',
    '{"file:read", "file:write", "workflow:execute"}',
    '{"theme": "light", "notifications": false, "language": "en"}'
),
-- TechStart Inc users
(
    '660e8400-e29b-41d4-a716-446655440004',
    '550e8400-e29b-41d4-a716-446655440002',
    'admin@techstart.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: admin123
    'Alice',
    'Johnson',
    'active',
    '{"admin", "user"}',
    '{"tenant:admin", "user:admin", "file:admin", "workflow:admin"}',
    '{"theme": "dark", "notifications": true, "language": "en"}'
),
(
    '660e8400-e29b-41d4-a716-446655440005',
    '550e8400-e29b-41d4-a716-446655440002',
    'developer@techstart.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: admin123
    'Bob',
    'Developer',
    'active',
    '{"developer", "user"}',
    '{"file:read", "file:write", "workflow:execute", "api:access"}',
    '{"theme": "dark", "notifications": true, "language": "en"}'
),
-- Global Solutions users
(
    '660e8400-e29b-41d4-a716-446655440006',
    '550e8400-e29b-41d4-a716-446655440003',
    'admin@globalsolutions.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: admin123
    'Emma',
    'Wilson',
    'active',
    '{"admin", "user"}',
    '{"tenant:admin", "user:admin", "file:admin", "workflow:admin"}',
    '{"theme": "light", "notifications": true, "language": "en"}'
);

-- Insert sample user sessions
INSERT INTO user_sessions (id, user_id, tenant_id, session_token, refresh_token, status, ip_address, user_agent, expires_at) VALUES
(
    '770e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'sess_' || encode(gen_random_bytes(32), 'hex'),
    'refresh_' || encode(gen_random_bytes(32), 'hex'),
    'active',
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36',
    NOW() + INTERVAL '7 days'
),
(
    '770e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440004',
    '550e8400-e29b-41d4-a716-446655440002',
    'sess_' || encode(gen_random_bytes(32), 'hex'),
    'refresh_' || encode(gen_random_bytes(32), 'hex'),
    'active',
    '10.0.0.50',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
    NOW() + INTERVAL '7 days'
);

-- Insert sample files
INSERT INTO files (id, tenant_id, user_id, filename, original_filename, mime_type, file_size, storage_path, storage_provider, status, metadata, checksum) VALUES
(
    '880e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    'document_001.pdf',
    'Company Policy.pdf',
    'application/pdf',
    1048576,
    '/storage/acme/documents/document_001.pdf',
    'local',
    'ready',
    '{"pages": 10, "author": "HR Department", "created_with": "Adobe Acrobat"}',
    'sha256:abcd1234567890abcdef1234567890abcdef1234567890abcdef1234567890ab'
),
(
    '880e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440002',
    'presentation_001.pptx',
    'Q4 Results.pptx',
    'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    5242880,
    '/storage/acme/presentations/presentation_001.pptx',
    'local',
    'ready',
    '{"slides": 25, "author": "Sarah Manager", "template": "Corporate"}',
    'sha256:efgh5678901234567890abcdef1234567890abcdef1234567890abcdef123456'
),
(
    '880e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440004',
    'code_archive.zip',
    'Project Source Code.zip',
    'application/zip',
    10485760,
    '/storage/techstart/code/code_archive.zip',
    'local',
    'ready',
    '{"files_count": 150, "language": "rust", "framework": "axum"}',
    'sha256:ijkl9012345678901234567890abcdef1234567890abcdef1234567890abcdef'
);

-- Insert sample file permissions
INSERT INTO file_permissions (id, file_id, tenant_id, user_id, permission_type, granted_by) VALUES
(
    '990e8400-e29b-41d4-a716-446655440001',
    '880e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440002',
    'read',
    '660e8400-e29b-41d4-a716-446655440001'
),
(
    '990e8400-e29b-41d4-a716-446655440002',
    '880e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440003',
    'read',
    '660e8400-e29b-41d4-a716-446655440001'
);

-- Insert sample workflow executions
INSERT INTO workflow_executions (id, tenant_id, user_id, workflow_id, workflow_type, run_id, task_queue, status, input_data, result_data) VALUES
(
    'aa0e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    'user_onboarding_001',
    'user_onboarding_workflow',
    'run_001',
    'user-onboarding-queue',
    'completed',
    '{"user_email": "newuser@acme.com", "tenant_id": "550e8400-e29b-41d4-a716-446655440001"}',
    '{"user_id": "new_user_123", "welcome_email_sent": true, "permissions_assigned": true}'
),
(
    'aa0e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440004',
    'file_processing_001',
    'file_processing_workflow',
    'run_002',
    'file-processing-queue',
    'running',
    '{"file_id": "880e8400-e29b-41d4-a716-446655440003", "processing_type": "virus_scan"}',
    NULL
);

-- Insert sample tenant billing information
INSERT INTO tenant_billing (id, tenant_id, billing_email, company_name, current_period_start, current_period_end, is_trial) VALUES
(
    'bb0e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'billing@acme.com',
    'Acme Corporation Ltd.',
    NOW() - INTERVAL '15 days',
    NOW() + INTERVAL '15 days',
    false
),
(
    'bb0e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440002',
    'billing@techstart.com',
    'TechStart Inc.',
    NOW() - INTERVAL '10 days',
    NOW() + INTERVAL '20 days',
    false
),
(
    'bb0e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440003',
    'admin@globalsolutions.com',
    'Global Solutions Ltd.',
    NOW() - INTERVAL '5 days',
    NOW() + INTERVAL '25 days',
    true
);

-- Insert sample tenant usage data
INSERT INTO tenant_usage (id, tenant_id, usage_type, usage_value, quota_limit, period_start, period_end) VALUES
-- Acme Corporation usage
(
    'cc0e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'api_calls',
    7500,
    10000,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
),
(
    'cc0e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    'storage_gb',
    250,
    1000,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
),
(
    'cc0e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440001',
    'users',
    15,
    1000,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
),
-- TechStart Inc usage
(
    'cc0e8400-e29b-41d4-a716-446655440004',
    '550e8400-e29b-41d4-a716-446655440002',
    'api_calls',
    3200,
    5000,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
),
(
    'cc0e8400-e29b-41d4-a716-446655440005',
    '550e8400-e29b-41d4-a716-446655440002',
    'storage_gb',
    45,
    100,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
);

-- Insert sample user profiles
INSERT INTO user_profiles (id, user_id, tenant_id, display_name, bio, timezone, language, job_title, department) VALUES
(
    'dd0e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'John Admin',
    'System Administrator with 10+ years of experience in enterprise software.',
    'UTC',
    'en',
    'System Administrator',
    'IT'
),
(
    'dd0e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    'Sarah Manager',
    'Operations Manager focused on process optimization and team leadership.',
    'America/New_York',
    'en',
    'Operations Manager',
    'Operations'
),
(
    'dd0e8400-e29b-41d4-a716-446655440003',
    '660e8400-e29b-41d4-a716-446655440004',
    '550e8400-e29b-41d4-a716-446655440002',
    'Alice Johnson',
    'Tech entrepreneur and startup founder with expertise in SaaS platforms.',
    'America/Los_Angeles',
    'en',
    'CEO & Founder',
    'Executive'
);

-- Insert sample user teams
INSERT INTO user_teams (id, tenant_id, team_name, team_description, team_lead_id, created_by) VALUES
(
    'ee0e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'IT Operations',
    'Responsible for system administration and infrastructure management.',
    '660e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001'
),
(
    'ee0e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    'Business Operations',
    'Handles day-to-day business operations and process management.',
    '660e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440001'
),
(
    'ee0e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440002',
    'Development Team',
    'Software development and engineering team.',
    '660e8400-e29b-41d4-a716-446655440004',
    '660e8400-e29b-41d4-a716-446655440004'
);

-- Insert sample team memberships
INSERT INTO user_team_memberships (id, team_id, user_id, tenant_id, role) VALUES
(
    'ff0e8400-e29b-41d4-a716-446655440001',
    'ee0e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    'lead'
),
(
    'ff0e8400-e29b-41d4-a716-446655440002',
    'ee0e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    'lead'
),
(
    'ff0e8400-e29b-41d4-a716-446655440003',
    'ee0e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440001',
    'member'
),
(
    'ff0e8400-e29b-41d4-a716-446655440004',
    'ee0e8400-e29b-41d4-a716-446655440003',
    '660e8400-e29b-41d4-a716-446655440004',
    '550e8400-e29b-41d4-a716-446655440002',
    'lead'
),
(
    'ff0e8400-e29b-41d4-a716-446655440005',
    'ee0e8400-e29b-41d4-a716-446655440003',
    '660e8400-e29b-41d4-a716-446655440005',
    '550e8400-e29b-41d4-a716-446655440002',
    'member'
);

-- Insert sample audit logs
INSERT INTO audit_logs (id, tenant_id, user_id, action, resource_type, resource_id, new_values, ip_address, user_agent) VALUES
(
    '110e8400-e29b-41d4-a716-446655440001',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    'user_login',
    'user_session',
    '770e8400-e29b-41d4-a716-446655440001',
    '{"session_id": "770e8400-e29b-41d4-a716-446655440001", "login_method": "password"}',
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
),
(
    '110e8400-e29b-41d4-a716-446655440002',
    '550e8400-e29b-41d4-a716-446655440001',
    '660e8400-e29b-41d4-a716-446655440001',
    'file_upload',
    'file',
    '880e8400-e29b-41d4-a716-446655440001',
    '{"filename": "Company Policy.pdf", "size": 1048576, "mime_type": "application/pdf"}',
    '192.168.1.100',
    'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36'
),
(
    '110e8400-e29b-41d4-a716-446655440003',
    '550e8400-e29b-41d4-a716-446655440002',
    '660e8400-e29b-41d4-a716-446655440004',
    'workflow_start',
    'workflow',
    'aa0e8400-e29b-41d4-a716-446655440002',
    '{"workflow_type": "file_processing_workflow", "input": {"file_id": "880e8400-e29b-41d4-a716-446655440003"}}',
    '10.0.0.50',
    'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36'
);

-- Update last login times for active users
UPDATE users SET last_login_at = NOW() - INTERVAL '1 hour' WHERE status = 'active';

-- Update session activity
UPDATE user_sessions SET last_activity_at = NOW() - INTERVAL '30 minutes' WHERE status = 'active';