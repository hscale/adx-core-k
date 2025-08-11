-- Database health checks, connection validation, and advanced indexing
-- This migration adds comprehensive health monitoring and optimized indexing

-- Database health monitoring table
CREATE TABLE IF NOT EXISTS database_health_checks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    check_name VARCHAR(100) NOT NULL,
    check_type VARCHAR(50) NOT NULL, -- 'connection', 'performance', 'storage', 'replication'
    status VARCHAR(20) NOT NULL CHECK (status IN ('healthy', 'warning', 'critical', 'unknown')),
    details JSONB DEFAULT '{}',
    response_time_ms INTEGER,
    threshold_warning INTEGER,
    threshold_critical INTEGER,
    last_healthy_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Connection pool monitoring table
CREATE TABLE IF NOT EXISTS connection_pool_stats (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    pool_name VARCHAR(100) NOT NULL,
    active_connections INTEGER NOT NULL DEFAULT 0,
    idle_connections INTEGER NOT NULL DEFAULT 0,
    max_connections INTEGER NOT NULL,
    total_connections INTEGER NOT NULL DEFAULT 0,
    connection_wait_time_ms INTEGER DEFAULT 0,
    query_count BIGINT DEFAULT 0,
    error_count BIGINT DEFAULT 0,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Query performance monitoring table
CREATE TABLE IF NOT EXISTS query_performance_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    query_hash VARCHAR(64) NOT NULL, -- Hash of the normalized query
    query_type VARCHAR(50) NOT NULL, -- 'SELECT', 'INSERT', 'UPDATE', 'DELETE'
    table_name VARCHAR(100),
    execution_time_ms INTEGER NOT NULL,
    rows_affected INTEGER DEFAULT 0,
    query_plan_hash VARCHAR(64),
    is_slow_query BOOLEAN NOT NULL DEFAULT false,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Database locks monitoring table
CREATE TABLE IF NOT EXISTS database_locks_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    lock_type VARCHAR(50) NOT NULL,
    lock_mode VARCHAR(50) NOT NULL,
    table_name VARCHAR(100),
    transaction_id BIGINT,
    process_id INTEGER,
    lock_duration_ms INTEGER,
    is_blocking BOOLEAN NOT NULL DEFAULT false,
    blocked_queries_count INTEGER DEFAULT 0,
    detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

-- Enhanced database health check function
CREATE OR REPLACE FUNCTION enhanced_database_health_check()
RETURNS TABLE(
    check_name TEXT,
    status TEXT,
    details JSONB,
    response_time_ms INTEGER
) AS $
DECLARE
    start_time TIMESTAMPTZ;
    end_time TIMESTAMPTZ;
    response_time INTEGER;
    connection_count INTEGER;
    active_sessions INTEGER;
    slow_queries INTEGER;
    table_count INTEGER;
    index_count INTEGER;
    db_size BIGINT;
BEGIN
    -- Connection test
    start_time := clock_timestamp();
    PERFORM 1;
    end_time := clock_timestamp();
    response_time := EXTRACT(MILLISECONDS FROM (end_time - start_time))::INTEGER;
    
    RETURN QUERY SELECT 
        'database_connection'::TEXT,
        CASE WHEN response_time < 100 THEN 'healthy'
             WHEN response_time < 500 THEN 'warning'
             ELSE 'critical' END::TEXT,
        jsonb_build_object(
            'response_time_ms', response_time,
            'timestamp', NOW()
        )::JSONB,
        response_time;
    
    -- Connection count check
    SELECT COUNT(*) INTO connection_count
    FROM pg_stat_activity
    WHERE state = 'active';
    
    RETURN QUERY SELECT 
        'active_connections'::TEXT,
        CASE WHEN connection_count < 50 THEN 'healthy'
             WHEN connection_count < 100 THEN 'warning'
             ELSE 'critical' END::TEXT,
        jsonb_build_object(
            'active_connections', connection_count,
            'max_connections', (SELECT setting::INTEGER FROM pg_settings WHERE name = 'max_connections')
        )::JSONB,
        0;
    
    -- Tenant count and isolation check
    SELECT COUNT(*) INTO table_count FROM tenants WHERE is_active = true;
    
    RETURN QUERY SELECT 
        'tenant_isolation'::TEXT,
        CASE WHEN table_count > 0 THEN 'healthy' ELSE 'warning' END::TEXT,
        jsonb_build_object(
            'active_tenants', table_count,
            'rls_enabled', (
                SELECT COUNT(*) FROM pg_policies 
                WHERE schemaname = 'public' AND policyname LIKE 'tenant_isolation_%'
            )
        )::JSONB,
        0;
    
    -- Slow queries check
    SELECT COUNT(*) INTO slow_queries
    FROM query_performance_log
    WHERE executed_at > NOW() - INTERVAL '1 hour'
    AND is_slow_query = true;
    
    RETURN QUERY SELECT 
        'query_performance'::TEXT,
        CASE WHEN slow_queries < 10 THEN 'healthy'
             WHEN slow_queries < 50 THEN 'warning'
             ELSE 'critical' END::TEXT,
        jsonb_build_object(
            'slow_queries_last_hour', slow_queries,
            'avg_query_time_ms', (
                SELECT AVG(execution_time_ms)::INTEGER
                FROM query_performance_log
                WHERE executed_at > NOW() - INTERVAL '1 hour'
            )
        )::JSONB,
        0;
    
    -- Database size check
    SELECT pg_database_size(current_database()) INTO db_size;
    
    RETURN QUERY SELECT 
        'database_size'::TEXT,
        CASE WHEN db_size < 10737418240 THEN 'healthy' -- 10GB
             WHEN db_size < 53687091200 THEN 'warning' -- 50GB
             ELSE 'critical' END::TEXT,
        jsonb_build_object(
            'size_bytes', db_size,
            'size_gb', ROUND(db_size / 1073741824.0, 2)
        )::JSONB,
        0;
    
    -- Index usage check
    SELECT COUNT(*) INTO index_count
    FROM pg_stat_user_indexes
    WHERE idx_scan = 0 AND schemaname = 'public';
    
    RETURN QUERY SELECT 
        'index_usage'::TEXT,
        CASE WHEN index_count = 0 THEN 'healthy'
             WHEN index_count < 5 THEN 'warning'
             ELSE 'critical' END::TEXT,
        jsonb_build_object(
            'unused_indexes', index_count,
            'total_indexes', (SELECT COUNT(*) FROM pg_stat_user_indexes WHERE schemaname = 'public')
        )::JSONB,
        0;
END;
$ LANGUAGE plpgsql;

-- Function to validate tenant database connections
CREATE OR REPLACE FUNCTION validate_tenant_connections()
RETURNS TABLE(
    tenant_id UUID,
    tenant_name TEXT,
    connection_status TEXT,
    schema_exists BOOLEAN,
    table_count INTEGER,
    last_activity TIMESTAMPTZ
) AS $
DECLARE
    tenant_record RECORD;
    schema_count INTEGER;
    table_count_val INTEGER;
    last_activity_val TIMESTAMPTZ;
BEGIN
    FOR tenant_record IN 
        SELECT t.id, t.name, t.isolation_level 
        FROM tenants t 
        WHERE t.is_active = true
    LOOP
        -- Check if tenant schema exists (for schema-based isolation)
        IF tenant_record.isolation_level = 'schema' THEN
            SELECT COUNT(*) INTO schema_count
            FROM information_schema.schemata
            WHERE schema_name = 'tenant_' || REPLACE(tenant_record.id::TEXT, '-', '_');
            
            IF schema_count > 0 THEN
                -- Count tables in tenant schema
                EXECUTE format('SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = %L',
                    'tenant_' || REPLACE(tenant_record.id::TEXT, '-', '_'))
                INTO table_count_val;
            ELSE
                table_count_val := 0;
            END IF;
        ELSE
            schema_count := 1; -- Row-level security doesn't need separate schemas
            table_count_val := 0; -- Not applicable for RLS
        END IF;
        
        -- Get last activity for tenant
        SELECT MAX(last_activity_at) INTO last_activity_val
        FROM user_sessions
        WHERE tenant_id = tenant_record.id
        AND status = 'active';
        
        RETURN QUERY SELECT 
            tenant_record.id,
            tenant_record.name::TEXT,
            CASE WHEN schema_count > 0 THEN 'connected' ELSE 'disconnected' END::TEXT,
            schema_count > 0,
            table_count_val,
            last_activity_val;
    END LOOP;
END;
$ LANGUAGE plpgsql;

-- Function to analyze and suggest index optimizations
CREATE OR REPLACE FUNCTION analyze_index_performance()
RETURNS TABLE(
    table_name TEXT,
    index_name TEXT,
    index_size TEXT,
    index_scans BIGINT,
    tuples_read BIGINT,
    tuples_fetched BIGINT,
    recommendation TEXT
) AS $
BEGIN
    RETURN QUERY
    SELECT 
        schemaname || '.' || tablename AS table_name,
        indexrelname AS index_name,
        pg_size_pretty(pg_relation_size(indexrelid)) AS index_size,
        idx_scan AS index_scans,
        idx_tup_read AS tuples_read,
        idx_tup_fetch AS tuples_fetched,
        CASE 
            WHEN idx_scan = 0 THEN 'Consider dropping - never used'
            WHEN idx_scan < 100 AND pg_relation_size(indexrelid) > 1048576 THEN 'Consider dropping - rarely used and large'
            WHEN idx_tup_read > idx_tup_fetch * 100 THEN 'Index may be inefficient - high read to fetch ratio'
            ELSE 'Index appears to be performing well'
        END AS recommendation
    FROM pg_stat_user_indexes
    WHERE schemaname = 'public'
    ORDER BY pg_relation_size(indexrelid) DESC;
END;
$ LANGUAGE plpgsql;

-- Function to monitor connection pool health
CREATE OR REPLACE FUNCTION monitor_connection_pool()
RETURNS VOID AS $
DECLARE
    active_conn INTEGER;
    idle_conn INTEGER;
    max_conn INTEGER;
    total_conn INTEGER;
BEGIN
    -- Get connection statistics
    SELECT COUNT(*) INTO active_conn
    FROM pg_stat_activity
    WHERE state = 'active';
    
    SELECT COUNT(*) INTO idle_conn
    FROM pg_stat_activity
    WHERE state = 'idle';
    
    SELECT setting::INTEGER INTO max_conn
    FROM pg_settings
    WHERE name = 'max_connections';
    
    total_conn := active_conn + idle_conn;
    
    -- Insert connection pool statistics
    INSERT INTO connection_pool_stats (
        pool_name,
        active_connections,
        idle_connections,
        max_connections,
        total_connections,
        recorded_at
    ) VALUES (
        'main_pool',
        active_conn,
        idle_conn,
        max_conn,
        total_conn,
        NOW()
    );
    
    -- Clean up old statistics (keep last 24 hours)
    DELETE FROM connection_pool_stats
    WHERE recorded_at < NOW() - INTERVAL '24 hours';
END;
$ LANGUAGE plpgsql;

-- Create composite indexes for better multi-tenant query performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_tenant_email_status 
ON users(tenant_id, email, status);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_tenant_user_status_created 
ON files(tenant_id, user_id, status, created_at);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_sessions_tenant_status_expires 
ON user_sessions(tenant_id, status, expires_at);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_audit_logs_tenant_created_action 
ON audit_logs(tenant_id, created_at, action);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workflow_executions_tenant_type_status 
ON workflow_executions(tenant_id, workflow_type, status);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_permissions_tenant_user_type 
ON file_permissions(tenant_id, user_id, permission_type);

-- Create partial indexes for common filtered queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_users_active_tenant 
ON users(tenant_id, email) WHERE status = 'active';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_ready_tenant 
ON files(tenant_id, created_at) WHERE status = 'ready';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_sessions_active_tenant 
ON user_sessions(tenant_id, user_id, last_activity_at) WHERE status = 'active';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tenants_active 
ON tenants(created_at, subscription_tier) WHERE is_active = true;

-- Create GIN indexes for JSONB columns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tenants_settings_gin 
ON tenants USING GIN(settings);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tenants_quotas_gin 
ON tenants USING GIN(quotas);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_files_metadata_gin 
ON files USING GIN(metadata);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_preferences_value_gin 
ON user_preferences USING GIN(preference_value);

-- Create indexes for foreign key relationships to improve join performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tenant_memberships_tenant_user 
ON tenant_memberships(tenant_id, user_id, is_active);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_file_versions_file_tenant 
ON file_versions(file_id, tenant_id, version_number);

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workflow_activities_workflow_tenant 
ON workflow_activities(workflow_execution_id, tenant_id, status);

-- Create indexes for monitoring tables
CREATE INDEX idx_database_health_checks_name_created ON database_health_checks(check_name, created_at);
CREATE INDEX idx_connection_pool_stats_recorded ON connection_pool_stats(recorded_at);
CREATE INDEX idx_query_performance_log_executed ON query_performance_log(executed_at);
CREATE INDEX idx_query_performance_log_slow ON query_performance_log(is_slow_query, executed_at);
CREATE INDEX idx_database_locks_log_detected ON database_locks_log(detected_at);
CREATE INDEX idx_database_locks_log_blocking ON database_locks_log(is_blocking, resolved_at);

-- Apply updated_at trigger to monitoring tables
CREATE TRIGGER update_database_health_checks_updated_at 
BEFORE UPDATE ON database_health_checks 
FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create a function to automatically update database health checks
CREATE OR REPLACE FUNCTION update_database_health_status()
RETURNS VOID AS $
DECLARE
    health_record RECORD;
BEGIN
    -- Run enhanced health checks and update the monitoring table
    FOR health_record IN 
        SELECT * FROM enhanced_database_health_check()
    LOOP
        INSERT INTO database_health_checks (
            check_name,
            check_type,
            status,
            details,
            response_time_ms,
            last_healthy_at,
            updated_at
        ) VALUES (
            health_record.check_name,
            'automated',
            health_record.status,
            health_record.details,
            health_record.response_time_ms,
            CASE WHEN health_record.status = 'healthy' THEN NOW() ELSE NULL END,
            NOW()
        )
        ON CONFLICT (check_name) DO UPDATE SET
            status = EXCLUDED.status,
            details = EXCLUDED.details,
            response_time_ms = EXCLUDED.response_time_ms,
            last_healthy_at = CASE WHEN EXCLUDED.status = 'healthy' THEN NOW() ELSE database_health_checks.last_healthy_at END,
            updated_at = NOW();
    END LOOP;
    
    -- Monitor connection pool
    PERFORM monitor_connection_pool();
END;
$ LANGUAGE plpgsql;

-- Add unique constraint to database_health_checks to prevent duplicates
ALTER TABLE database_health_checks ADD CONSTRAINT unique_check_name UNIQUE (check_name);