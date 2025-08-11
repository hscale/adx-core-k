-- Workflow Service specific schema
-- Temporal workflow tracking and management tables

-- Workflow executions table for tracking workflow instances
CREATE TABLE IF NOT EXISTS workflow_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    workflow_id VARCHAR(255) NOT NULL,
    workflow_type VARCHAR(100) NOT NULL,
    run_id VARCHAR(255) NOT NULL,
    task_queue VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'completed', 'failed', 'cancelled', 'terminated', 'timed_out')),
    input_data JSONB,
    result_data JSONB,
    error_message TEXT,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workflow_id, run_id)
);

-- Workflow activities table for tracking activity executions
CREATE TABLE IF NOT EXISTS workflow_activities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    activity_id VARCHAR(255) NOT NULL,
    activity_type VARCHAR(100) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'scheduled' CHECK (status IN ('scheduled', 'started', 'completed', 'failed', 'cancelled', 'timed_out')),
    input_data JSONB,
    result_data JSONB,
    error_message TEXT,
    attempt_count INTEGER NOT NULL DEFAULT 1,
    max_attempts INTEGER NOT NULL DEFAULT 3,
    scheduled_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Workflow schedules table for recurring workflows
CREATE TABLE IF NOT EXISTS workflow_schedules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    schedule_id VARCHAR(255) NOT NULL UNIQUE,
    workflow_type VARCHAR(100) NOT NULL,
    task_queue VARCHAR(100) NOT NULL,
    cron_expression VARCHAR(100) NOT NULL,
    input_data JSONB DEFAULT '{}',
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    next_run_at TIMESTAMPTZ,
    last_run_at TIMESTAMPTZ,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Workflow templates table for reusable workflow definitions
CREATE TABLE IF NOT EXISTS workflow_templates (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    template_name VARCHAR(100) NOT NULL,
    workflow_type VARCHAR(100) NOT NULL,
    description TEXT,
    input_schema JSONB NOT NULL, -- JSON schema for input validation
    default_input JSONB DEFAULT '{}',
    task_queue VARCHAR(100) NOT NULL,
    timeout_seconds INTEGER DEFAULT 3600,
    retry_policy JSONB DEFAULT '{}',
    is_public BOOLEAN NOT NULL DEFAULT false, -- Can be used by other tenants
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, template_name)
);

-- Workflow signals table for tracking signals sent to workflows
CREATE TABLE IF NOT EXISTS workflow_signals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    signal_name VARCHAR(100) NOT NULL,
    signal_data JSONB,
    sent_by UUID REFERENCES users(id) ON DELETE SET NULL,
    sent_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Workflow queries table for tracking queries sent to workflows
CREATE TABLE IF NOT EXISTS workflow_queries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    workflow_execution_id UUID NOT NULL REFERENCES workflow_executions(id) ON DELETE CASCADE,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    query_name VARCHAR(100) NOT NULL,
    query_data JSONB,
    result_data JSONB,
    queried_by UUID REFERENCES users(id) ON DELETE SET NULL,
    queried_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Workflow metrics table for performance tracking
CREATE TABLE IF NOT EXISTS workflow_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    workflow_type VARCHAR(100) NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(10,2) NOT NULL,
    metric_unit VARCHAR(20) NOT NULL, -- 'seconds', 'count', 'bytes', etc.
    tags JSONB DEFAULT '{}',
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- AI workflow configurations table
CREATE TABLE IF NOT EXISTS ai_workflow_configs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    workflow_type VARCHAR(100) NOT NULL,
    ai_provider VARCHAR(50) NOT NULL, -- 'openai', 'anthropic', 'local', etc.
    model_name VARCHAR(100) NOT NULL,
    configuration JSONB NOT NULL,
    is_enabled BOOLEAN NOT NULL DEFAULT true,
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, workflow_type, ai_provider)
);

-- Cross-service workflow orchestrations table
CREATE TABLE IF NOT EXISTS cross_service_workflows (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    orchestration_id VARCHAR(255) NOT NULL UNIQUE,
    orchestration_type VARCHAR(100) NOT NULL,
    involved_services TEXT[] NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
    input_data JSONB,
    result_data JSONB,
    error_message TEXT,
    started_by UUID REFERENCES users(id) ON DELETE SET NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Enable RLS on workflow service tables
ALTER TABLE workflow_executions ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_activities ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_schedules ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_templates ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_signals ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_queries ENABLE ROW LEVEL SECURITY;
ALTER TABLE workflow_metrics ENABLE ROW LEVEL SECURITY;
ALTER TABLE ai_workflow_configs ENABLE ROW LEVEL SECURITY;
ALTER TABLE cross_service_workflows ENABLE ROW LEVEL SECURITY;

-- Create RLS policies for workflow service tables
CREATE POLICY tenant_isolation_workflow_executions ON workflow_executions
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_workflow_activities ON workflow_activities
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_workflow_schedules ON workflow_schedules
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_workflow_templates ON workflow_templates
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID OR is_public = true);

CREATE POLICY tenant_isolation_workflow_signals ON workflow_signals
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_workflow_queries ON workflow_queries
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_workflow_metrics ON workflow_metrics
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_ai_workflow_configs ON ai_workflow_configs
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

CREATE POLICY tenant_isolation_cross_service_workflows ON cross_service_workflows
    FOR ALL TO PUBLIC
    USING (tenant_id = current_setting('app.current_tenant_id', true)::UUID);

-- Create indexes for workflow service tables
CREATE INDEX idx_workflow_executions_tenant_id ON workflow_executions(tenant_id);
CREATE INDEX idx_workflow_executions_user_id ON workflow_executions(user_id);
CREATE INDEX idx_workflow_executions_workflow_id ON workflow_executions(workflow_id);
CREATE INDEX idx_workflow_executions_workflow_type ON workflow_executions(workflow_type);
CREATE INDEX idx_workflow_executions_status ON workflow_executions(status);
CREATE INDEX idx_workflow_executions_started_at ON workflow_executions(started_at);
CREATE INDEX idx_workflow_executions_task_queue ON workflow_executions(task_queue);

CREATE INDEX idx_workflow_activities_workflow_execution_id ON workflow_activities(workflow_execution_id);
CREATE INDEX idx_workflow_activities_tenant_id ON workflow_activities(tenant_id);
CREATE INDEX idx_workflow_activities_activity_type ON workflow_activities(activity_type);
CREATE INDEX idx_workflow_activities_status ON workflow_activities(status);
CREATE INDEX idx_workflow_activities_scheduled_at ON workflow_activities(scheduled_at);

CREATE INDEX idx_workflow_schedules_tenant_id ON workflow_schedules(tenant_id);
CREATE INDEX idx_workflow_schedules_schedule_id ON workflow_schedules(schedule_id);
CREATE INDEX idx_workflow_schedules_workflow_type ON workflow_schedules(workflow_type);
CREATE INDEX idx_workflow_schedules_enabled ON workflow_schedules(is_enabled);
CREATE INDEX idx_workflow_schedules_next_run_at ON workflow_schedules(next_run_at);

CREATE INDEX idx_workflow_templates_tenant_id ON workflow_templates(tenant_id);
CREATE INDEX idx_workflow_templates_workflow_type ON workflow_templates(workflow_type);
CREATE INDEX idx_workflow_templates_public ON workflow_templates(is_public);

CREATE INDEX idx_workflow_signals_workflow_execution_id ON workflow_signals(workflow_execution_id);
CREATE INDEX idx_workflow_signals_tenant_id ON workflow_signals(tenant_id);
CREATE INDEX idx_workflow_signals_signal_name ON workflow_signals(signal_name);
CREATE INDEX idx_workflow_signals_sent_at ON workflow_signals(sent_at);

CREATE INDEX idx_workflow_queries_workflow_execution_id ON workflow_queries(workflow_execution_id);
CREATE INDEX idx_workflow_queries_tenant_id ON workflow_queries(tenant_id);
CREATE INDEX idx_workflow_queries_query_name ON workflow_queries(query_name);
CREATE INDEX idx_workflow_queries_queried_at ON workflow_queries(queried_at);

CREATE INDEX idx_workflow_metrics_tenant_id ON workflow_metrics(tenant_id);
CREATE INDEX idx_workflow_metrics_workflow_type ON workflow_metrics(workflow_type);
CREATE INDEX idx_workflow_metrics_metric_name ON workflow_metrics(metric_name);
CREATE INDEX idx_workflow_metrics_recorded_at ON workflow_metrics(recorded_at);

CREATE INDEX idx_ai_workflow_configs_tenant_id ON ai_workflow_configs(tenant_id);
CREATE INDEX idx_ai_workflow_configs_workflow_type ON ai_workflow_configs(workflow_type);
CREATE INDEX idx_ai_workflow_configs_provider ON ai_workflow_configs(ai_provider);
CREATE INDEX idx_ai_workflow_configs_enabled ON ai_workflow_configs(is_enabled);

CREATE INDEX idx_cross_service_workflows_tenant_id ON cross_service_workflows(tenant_id);
CREATE INDEX idx_cross_service_workflows_orchestration_id ON cross_service_workflows(orchestration_id);
CREATE INDEX idx_cross_service_workflows_type ON cross_service_workflows(orchestration_type);
CREATE INDEX idx_cross_service_workflows_status ON cross_service_workflows(status);
CREATE INDEX idx_cross_service_workflows_started_at ON cross_service_workflows(started_at);

-- Apply updated_at triggers
CREATE TRIGGER update_workflow_executions_updated_at BEFORE UPDATE ON workflow_executions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_workflow_activities_updated_at BEFORE UPDATE ON workflow_activities FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_workflow_schedules_updated_at BEFORE UPDATE ON workflow_schedules FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_workflow_templates_updated_at BEFORE UPDATE ON workflow_templates FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ai_workflow_configs_updated_at BEFORE UPDATE ON ai_workflow_configs FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_cross_service_workflows_updated_at BEFORE UPDATE ON cross_service_workflows FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();