-- Test seed data for ADX Core
-- This script creates minimal test data for automated testing

-- Insert test tenants
INSERT INTO tenants (id, name, slug, admin_email, subscription_tier, isolation_level, settings, quotas, features, is_active) VALUES
(
    '00000000-0000-0000-0000-000000000001',
    'Test Tenant 1',
    'test-tenant-1',
    'admin@test1.com',
    'professional',
    'row',
    '{"theme": "light", "timezone": "UTC", "language": "en"}',
    '{"max_users": 50, "max_storage_gb": 50, "max_api_calls_per_hour": 2000, "max_workflows_per_hour": 50}',
    '{"basic_analytics", "file_sharing", "workflow_automation"}',
    true
),
(
    '00000000-0000-0000-0000-000000000002',
    'Test Tenant 2',
    'test-tenant-2',
    'admin@test2.com',
    'enterprise',
    'schema',
    '{"theme": "dark", "timezone": "America/New_York", "language": "en"}',
    '{"max_users": 200, "max_storage_gb": 200, "max_api_calls_per_hour": 5000, "max_workflows_per_hour": 200}',
    '{"advanced_analytics", "custom_branding", "sso", "api_access", "workflow_automation"}',
    true
),
(
    '00000000-0000-0000-0000-000000000003',
    'Inactive Test Tenant',
    'inactive-test-tenant',
    'admin@inactive.com',
    'free',
    'row',
    '{"theme": "light", "timezone": "UTC", "language": "en"}',
    '{"max_users": 5, "max_storage_gb": 1, "max_api_calls_per_hour": 100, "max_workflows_per_hour": 5}',
    '{"basic_features"}',
    false
);

-- Create schema for schema-based tenant
SELECT create_tenant_schema('00000000-0000-0000-0000-000000000002', NULL);

-- Insert test users
INSERT INTO users (id, tenant_id, email, password_hash, first_name, last_name, status, roles, permissions, preferences, email_verified_at) VALUES
-- Test Tenant 1 users
(
    '00000000-0000-0000-0000-000000000101',
    '00000000-0000-0000-0000-000000000001',
    'admin@test1.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: test123
    'Test',
    'Admin',
    'active',
    '{"admin", "user"}',
    '{"tenant:admin", "user:admin", "file:admin", "workflow:admin"}',
    '{"theme": "light", "notifications": true, "language": "en"}',
    NOW()
),
(
    '00000000-0000-0000-0000-000000000102',
    '00000000-0000-0000-0000-000000000001',
    'user@test1.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: test123
    'Test',
    'User',
    'active',
    '{"user"}',
    '{"file:read", "file:write", "workflow:execute"}',
    '{"theme": "light", "notifications": false, "language": "en"}',
    NOW()
),
-- Test Tenant 2 users
(
    '00000000-0000-0000-0000-000000000201',
    '00000000-0000-0000-0000-000000000002',
    'admin@test2.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: test123
    'Test',
    'Admin2',
    'active',
    '{"admin", "user"}',
    '{"tenant:admin", "user:admin", "file:admin", "workflow:admin"}',
    '{"theme": "dark", "notifications": true, "language": "en"}',
    NOW()
),
(
    '00000000-0000-0000-0000-000000000202',
    '00000000-0000-0000-0000-000000000002',
    'user@test2.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: test123
    'Test',
    'User2',
    'active',
    '{"user"}',
    '{"file:read", "file:write", "workflow:execute"}',
    '{"theme": "dark", "notifications": true, "language": "en"}',
    NOW()
),
-- Pending verification user
(
    '00000000-0000-0000-0000-000000000103',
    '00000000-0000-0000-0000-000000000001',
    'pending@test1.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: test123
    'Pending',
    'User',
    'pending_verification',
    '{"user"}',
    '{"file:read"}',
    '{"theme": "light", "notifications": true, "language": "en"}',
    NULL
),
-- Suspended user
(
    '00000000-0000-0000-0000-000000000104',
    '00000000-0000-0000-0000-000000000001',
    'suspended@test1.com',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- password: test123
    'Suspended',
    'User',
    'suspended',
    '{"user"}',
    '{}',
    '{"theme": "light", "notifications": false, "language": "en"}',
    NOW() - INTERVAL '30 days'
);

-- Insert test user sessions
INSERT INTO user_sessions (id, user_id, tenant_id, session_token, refresh_token, status, ip_address, user_agent, expires_at) VALUES
(
    '00000000-0000-0000-0000-000000001001',
    '00000000-0000-0000-0000-000000000101',
    '00000000-0000-0000-0000-000000000001',
    'test_session_token_1',
    'test_refresh_token_1',
    'active',
    '127.0.0.1',
    'Test User Agent 1',
    NOW() + INTERVAL '7 days'
),
(
    '00000000-0000-0000-0000-000000001002',
    '00000000-0000-0000-0000-000000000201',
    '00000000-0000-0000-0000-000000000002',
    'test_session_token_2',
    'test_refresh_token_2',
    'active',
    '127.0.0.1',
    'Test User Agent 2',
    NOW() + INTERVAL '7 days'
),
-- Expired session for testing
(
    '00000000-0000-0000-0000-000000001003',
    '00000000-0000-0000-0000-000000000102',
    '00000000-0000-0000-0000-000000000001',
    'test_session_token_expired',
    'test_refresh_token_expired',
    'expired',
    '127.0.0.1',
    'Test User Agent Expired',
    NOW() - INTERVAL '1 day'
);

-- Insert test files
INSERT INTO files (id, tenant_id, user_id, filename, original_filename, mime_type, file_size, storage_path, storage_provider, status, metadata, checksum) VALUES
(
    '00000000-0000-0000-0000-000000002001',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000101',
    'test_document.pdf',
    'Test Document.pdf',
    'application/pdf',
    102400,
    '/test/storage/test_document.pdf',
    'local',
    'ready',
    '{"pages": 5, "author": "Test Author"}',
    'test_checksum_1'
),
(
    '00000000-0000-0000-0000-000000002002',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000102',
    'test_image.jpg',
    'Test Image.jpg',
    'image/jpeg',
    51200,
    '/test/storage/test_image.jpg',
    'local',
    'ready',
    '{"width": 1920, "height": 1080, "camera": "Test Camera"}',
    'test_checksum_2'
),
(
    '00000000-0000-0000-0000-000000002003',
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000201',
    'test_spreadsheet.xlsx',
    'Test Spreadsheet.xlsx',
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    25600,
    '/test/storage/test_spreadsheet.xlsx',
    'local',
    'ready',
    '{"sheets": 3, "rows": 100, "columns": 10}',
    'test_checksum_3'
),
-- File in processing state
(
    '00000000-0000-0000-0000-000000002004',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000101',
    'processing_file.zip',
    'Large Archive.zip',
    'application/zip',
    1048576,
    '/test/storage/processing_file.zip',
    'local',
    'processing',
    '{"compression": "deflate"}',
    'test_checksum_4'
);

-- Insert test file permissions
INSERT INTO file_permissions (id, file_id, tenant_id, user_id, permission_type, granted_by) VALUES
(
    '00000000-0000-0000-0000-000000003001',
    '00000000-0000-0000-0000-000000002001',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000102',
    'read',
    '00000000-0000-0000-0000-000000000101'
),
(
    '00000000-0000-0000-0000-000000003002',
    '00000000-0000-0000-0000-000000002002',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000101',
    'admin',
    '00000000-0000-0000-0000-000000000101'
);

-- Insert test workflow executions
INSERT INTO workflow_executions (id, tenant_id, user_id, workflow_id, workflow_type, run_id, task_queue, status, input_data, result_data) VALUES
(
    '00000000-0000-0000-0000-000000004001',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000101',
    'test_workflow_1',
    'test_workflow_type',
    'test_run_1',
    'test-queue',
    'completed',
    '{"test_input": "value1"}',
    '{"test_output": "result1"}'
),
(
    '00000000-0000-0000-0000-000000004002',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000102',
    'test_workflow_2',
    'test_workflow_type',
    'test_run_2',
    'test-queue',
    'running',
    '{"test_input": "value2"}',
    NULL
),
(
    '00000000-0000-0000-0000-000000004003',
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000201',
    'test_workflow_3',
    'test_workflow_type',
    'test_run_3',
    'test-queue',
    'failed',
    '{"test_input": "value3"}',
    NULL
);

-- Insert test tenant billing
INSERT INTO tenant_billing (id, tenant_id, billing_email, company_name, current_period_start, current_period_end, is_trial) VALUES
(
    '00000000-0000-0000-0000-000000005001',
    '00000000-0000-0000-0000-000000000001',
    'billing@test1.com',
    'Test Company 1',
    NOW() - INTERVAL '15 days',
    NOW() + INTERVAL '15 days',
    false
),
(
    '00000000-0000-0000-0000-000000005002',
    '00000000-0000-0000-0000-000000000002',
    'billing@test2.com',
    'Test Company 2',
    NOW() - INTERVAL '10 days',
    NOW() + INTERVAL '20 days',
    true
);

-- Insert test tenant usage
INSERT INTO tenant_usage (id, tenant_id, usage_type, usage_value, quota_limit, period_start, period_end) VALUES
(
    '00000000-0000-0000-0000-000000006001',
    '00000000-0000-0000-0000-000000000001',
    'api_calls',
    1500,
    2000,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
),
(
    '00000000-0000-0000-0000-000000006002',
    '00000000-0000-0000-0000-000000000001',
    'storage_gb',
    10,
    50,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
),
(
    '00000000-0000-0000-0000-000000006003',
    '00000000-0000-0000-0000-000000000002',
    'api_calls',
    3000,
    5000,
    DATE_TRUNC('month', NOW()),
    DATE_TRUNC('month', NOW()) + INTERVAL '1 month' - INTERVAL '1 day'
);

-- Insert test user profiles
INSERT INTO user_profiles (id, user_id, tenant_id, display_name, timezone, language, job_title) VALUES
(
    '00000000-0000-0000-0000-000000007001',
    '00000000-0000-0000-0000-000000000101',
    '00000000-0000-0000-0000-000000000001',
    'Test Admin',
    'UTC',
    'en',
    'Test Administrator'
),
(
    '00000000-0000-0000-0000-000000007002',
    '00000000-0000-0000-0000-000000000102',
    '00000000-0000-0000-0000-000000000001',
    'Test User',
    'UTC',
    'en',
    'Test User'
),
(
    '00000000-0000-0000-0000-000000007003',
    '00000000-0000-0000-0000-000000000201',
    '00000000-0000-0000-0000-000000000002',
    'Test Admin 2',
    'America/New_York',
    'en',
    'Test Administrator'
);

-- Insert test API keys
INSERT INTO api_keys (id, tenant_id, user_id, key_name, key_hash, key_prefix, permissions, rate_limit_per_hour) VALUES
(
    '00000000-0000-0000-0000-000000008001',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000101',
    'Test API Key 1',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- hash of 'test_api_key_1'
    'tak_test1',
    '{"api:read", "api:write"}',
    1000
),
(
    '00000000-0000-0000-0000-000000008002',
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000201',
    'Test API Key 2',
    '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewdBPj/VcAgaq.Oa', -- hash of 'test_api_key_2'
    'tak_test2',
    '{"api:read", "api:write", "api:admin"}',
    2000
);

-- Insert test audit logs
INSERT INTO audit_logs (id, tenant_id, user_id, action, resource_type, resource_id, new_values, ip_address) VALUES
(
    '00000000-0000-0000-0000-000000009001',
    '00000000-0000-0000-0000-000000000001',
    '00000000-0000-0000-0000-000000000101',
    'test_action_1',
    'test_resource',
    '00000000-0000-0000-0000-000000002001',
    '{"test_field": "test_value"}',
    '127.0.0.1'
),
(
    '00000000-0000-0000-0000-000000009002',
    '00000000-0000-0000-0000-000000000002',
    '00000000-0000-0000-0000-000000000201',
    'test_action_2',
    'test_resource',
    '00000000-0000-0000-0000-000000002003',
    '{"test_field": "test_value_2"}',
    '127.0.0.1'
);

-- Update timestamps for realistic test data
UPDATE users SET 
    last_login_at = NOW() - INTERVAL '1 hour',
    created_at = NOW() - INTERVAL '30 days',
    updated_at = NOW() - INTERVAL '1 day'
WHERE status = 'active';

UPDATE user_sessions SET 
    last_activity_at = NOW() - INTERVAL '15 minutes',
    created_at = NOW() - INTERVAL '2 hours'
WHERE status = 'active';

UPDATE files SET 
    created_at = NOW() - INTERVAL '7 days',
    updated_at = NOW() - INTERVAL '1 day'
WHERE status = 'ready';

UPDATE workflow_executions SET 
    started_at = NOW() - INTERVAL '2 hours',
    completed_at = CASE WHEN status = 'completed' THEN NOW() - INTERVAL '1 hour' ELSE NULL END,
    created_at = NOW() - INTERVAL '3 hours',
    updated_at = NOW() - INTERVAL '30 minutes';

-- Insert some performance test data for query optimization
INSERT INTO query_performance_log (query_hash, query_type, table_name, execution_time_ms, rows_affected, is_slow_query, executed_at)
SELECT 
    'test_query_' || generate_series,
    CASE (generate_series % 4)
        WHEN 0 THEN 'SELECT'
        WHEN 1 THEN 'INSERT'
        WHEN 2 THEN 'UPDATE'
        ELSE 'DELETE'
    END,
    CASE (generate_series % 5)
        WHEN 0 THEN 'users'
        WHEN 1 THEN 'files'
        WHEN 2 THEN 'tenants'
        WHEN 3 THEN 'user_sessions'
        ELSE 'workflow_executions'
    END,
    (random() * 1000)::INTEGER,
    (random() * 100)::INTEGER,
    random() > 0.9, -- 10% slow queries
    NOW() - (random() * INTERVAL '24 hours')
FROM generate_series(1, 100);