-- AI Service Database Schema
-- This migration creates the initial schema for the AI service

-- AI usage records table for tracking all AI operations
CREATE TABLE ai_usage_records (
    id UUID PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    workflow_id VARCHAR(255),
    activity_id VARCHAR(255),
    model VARCHAR(255) NOT NULL,
    capability TEXT NOT NULL, -- JSON serialized AICapability
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    estimated_cost DECIMAL(10, 6) NOT NULL DEFAULT 0.0,
    request_timestamp TIMESTAMPTZ NOT NULL,
    response_timestamp TIMESTAMPTZ NOT NULL,
    success BOOLEAN NOT NULL DEFAULT true,
    error_code VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_ai_usage_tenant_id ON ai_usage_records(tenant_id);
CREATE INDEX idx_ai_usage_user_id ON ai_usage_records(user_id);
CREATE INDEX idx_ai_usage_model ON ai_usage_records(model);
CREATE INDEX idx_ai_usage_capability ON ai_usage_records(capability);
CREATE INDEX idx_ai_usage_timestamp ON ai_usage_records(request_timestamp);
CREATE INDEX idx_ai_usage_tenant_timestamp ON ai_usage_records(tenant_id, request_timestamp);
CREATE INDEX idx_ai_usage_workflow_id ON ai_usage_records(workflow_id) WHERE workflow_id IS NOT NULL;

-- AI model configurations table (for dynamic model management)
CREATE TABLE ai_models (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    provider VARCHAR(50) NOT NULL, -- 'OpenAI', 'Anthropic', 'Local'
    capabilities TEXT[] NOT NULL, -- Array of capability strings
    max_tokens INTEGER NOT NULL,
    cost_per_token DECIMAL(12, 8) NOT NULL DEFAULT 0.0,
    tier_availability TEXT[] NOT NULL, -- Array of subscription tiers
    is_active BOOLEAN NOT NULL DEFAULT true,
    configuration JSONB, -- Provider-specific configuration
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default models
INSERT INTO ai_models (id, name, provider, capabilities, max_tokens, cost_per_token, tier_availability) VALUES
('gpt-3.5-turbo', 'GPT-3.5 Turbo', 'OpenAI', 
 ARRAY['TextGeneration', 'TextClassification', 'TextSummarization', 'EntityExtraction', 'SentimentAnalysis'], 
 4096, 0.0000015, ARRAY['Professional', 'Enterprise']),
('gpt-4', 'GPT-4', 'OpenAI', 
 ARRAY['TextGeneration', 'TextClassification', 'TextSummarization', 'EntityExtraction', 'SentimentAnalysis', 'CodeGeneration'], 
 8192, 0.00003, ARRAY['Enterprise']),
('gpt-4-turbo', 'GPT-4 Turbo', 'OpenAI', 
 ARRAY['TextGeneration', 'TextClassification', 'TextSummarization', 'EntityExtraction', 'SentimentAnalysis', 'CodeGeneration'], 
 128000, 0.00001, ARRAY['Enterprise']),
('claude-3-haiku-20240307', 'Claude 3 Haiku', 'Anthropic', 
 ARRAY['TextGeneration', 'TextClassification', 'TextSummarization', 'EntityExtraction'], 
 4096, 0.00000025, ARRAY['Professional', 'Enterprise']),
('claude-3-sonnet-20240229', 'Claude 3 Sonnet', 'Anthropic', 
 ARRAY['TextGeneration', 'TextClassification', 'TextSummarization', 'EntityExtraction', 'SentimentAnalysis', 'CodeGeneration'], 
 4096, 0.000003, ARRAY['Enterprise']),
('llama2-7b', 'Llama 2 7B', 'Local', 
 ARRAY['TextGeneration', 'TextClassification', 'TextSummarization'], 
 4096, 0.0, ARRAY['Free', 'Professional', 'Enterprise']),
('mistral-7b', 'Mistral 7B', 'Local', 
 ARRAY['TextGeneration', 'CodeGeneration'], 
 8192, 0.0, ARRAY['Free', 'Professional', 'Enterprise']);

-- AI provider health status table
CREATE TABLE ai_provider_health (
    provider VARCHAR(50) PRIMARY KEY,
    status VARCHAR(20) NOT NULL, -- 'Healthy', 'Degraded', 'Unhealthy'
    last_check TIMESTAMPTZ NOT NULL,
    response_time_ms INTEGER,
    error_rate DECIMAL(5, 4) DEFAULT 0.0,
    last_error TEXT,
    consecutive_failures INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Insert default provider health records
INSERT INTO ai_provider_health (provider, status, last_check) VALUES
('OpenAI', 'Healthy', NOW()),
('Anthropic', 'Healthy', NOW()),
('Local', 'Healthy', NOW());

-- AI quotas and limits table (per tenant)
CREATE TABLE ai_quotas (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    capability VARCHAR(50) NOT NULL,
    max_requests_per_hour INTEGER NOT NULL DEFAULT 1000,
    max_tokens_per_hour BIGINT NOT NULL DEFAULT 100000,
    max_cost_per_hour DECIMAL(10, 2) NOT NULL DEFAULT 100.00,
    reset_period VARCHAR(20) NOT NULL DEFAULT 'hourly', -- 'hourly', 'daily', 'monthly'
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(tenant_id, capability)
);

-- AI workflow results table (for caching and analysis)
CREATE TABLE ai_workflow_results (
    id UUID PRIMARY KEY,
    workflow_id VARCHAR(255) NOT NULL,
    workflow_type VARCHAR(100) NOT NULL,
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    input_data JSONB NOT NULL,
    output_data JSONB,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    total_cost DECIMAL(10, 6) NOT NULL DEFAULT 0.0,
    execution_time_ms BIGINT NOT NULL DEFAULT 0,
    quality_score DECIMAL(3, 2), -- 0.00 to 1.00
    success BOOLEAN NOT NULL DEFAULT true,
    error_message TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for workflow results
CREATE INDEX idx_ai_workflow_results_workflow_id ON ai_workflow_results(workflow_id);
CREATE INDEX idx_ai_workflow_results_tenant_id ON ai_workflow_results(tenant_id);
CREATE INDEX idx_ai_workflow_results_workflow_type ON ai_workflow_results(workflow_type);
CREATE INDEX idx_ai_workflow_results_created_at ON ai_workflow_results(created_at);

-- AI content moderation table (for tracking filtered content)
CREATE TABLE ai_content_moderation (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255) NOT NULL,
    content_hash VARCHAR(64) NOT NULL, -- SHA-256 hash of content
    content_type VARCHAR(50) NOT NULL, -- 'prompt', 'response'
    moderation_result VARCHAR(20) NOT NULL, -- 'approved', 'flagged', 'blocked'
    flagged_categories TEXT[], -- Array of flagged categories
    confidence_score DECIMAL(3, 2), -- 0.00 to 1.00
    model_used VARCHAR(255),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for content moderation
CREATE INDEX idx_ai_content_moderation_tenant_id ON ai_content_moderation(tenant_id);
CREATE INDEX idx_ai_content_moderation_content_hash ON ai_content_moderation(content_hash);
CREATE INDEX idx_ai_content_moderation_result ON ai_content_moderation(moderation_result);

-- AI model performance metrics table
CREATE TABLE ai_model_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_id VARCHAR(255) NOT NULL,
    tenant_id VARCHAR(255) NOT NULL,
    capability VARCHAR(50) NOT NULL,
    avg_response_time_ms DECIMAL(8, 2) NOT NULL,
    success_rate DECIMAL(5, 4) NOT NULL, -- 0.0000 to 1.0000
    avg_quality_score DECIMAL(3, 2), -- 0.00 to 1.00
    total_requests BIGINT NOT NULL DEFAULT 0,
    total_tokens BIGINT NOT NULL DEFAULT 0,
    total_cost DECIMAL(12, 6) NOT NULL DEFAULT 0.0,
    period_start TIMESTAMPTZ NOT NULL,
    period_end TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for model metrics
CREATE INDEX idx_ai_model_metrics_model_id ON ai_model_metrics(model_id);
CREATE INDEX idx_ai_model_metrics_tenant_id ON ai_model_metrics(tenant_id);
CREATE INDEX idx_ai_model_metrics_period ON ai_model_metrics(period_start, period_end);

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers to automatically update updated_at
CREATE TRIGGER update_ai_usage_records_updated_at BEFORE UPDATE ON ai_usage_records FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ai_models_updated_at BEFORE UPDATE ON ai_models FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ai_provider_health_updated_at BEFORE UPDATE ON ai_provider_health FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ai_quotas_updated_at BEFORE UPDATE ON ai_quotas FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_ai_workflow_results_updated_at BEFORE UPDATE ON ai_workflow_results FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Views for common queries
CREATE VIEW ai_usage_summary AS
SELECT 
    tenant_id,
    model,
    capability,
    DATE_TRUNC('hour', request_timestamp) as hour,
    COUNT(*) as request_count,
    SUM(total_tokens) as total_tokens,
    SUM(estimated_cost) as total_cost,
    AVG(EXTRACT(EPOCH FROM (response_timestamp - request_timestamp)) * 1000) as avg_response_time_ms,
    COUNT(*) FILTER (WHERE success = true) as successful_requests
FROM ai_usage_records
GROUP BY tenant_id, model, capability, DATE_TRUNC('hour', request_timestamp);

CREATE VIEW ai_tenant_daily_usage AS
SELECT 
    tenant_id,
    DATE_TRUNC('day', request_timestamp) as day,
    COUNT(*) as total_requests,
    SUM(total_tokens) as total_tokens,
    SUM(estimated_cost) as total_cost,
    COUNT(DISTINCT model) as models_used,
    COUNT(DISTINCT capability) as capabilities_used
FROM ai_usage_records
GROUP BY tenant_id, DATE_TRUNC('day', request_timestamp);

-- Comments for documentation
COMMENT ON TABLE ai_usage_records IS 'Tracks all AI API usage for billing, analytics, and quota enforcement';
COMMENT ON TABLE ai_models IS 'Configuration and metadata for available AI models';
COMMENT ON TABLE ai_provider_health IS 'Health status tracking for AI providers';
COMMENT ON TABLE ai_quotas IS 'Per-tenant quotas and limits for AI usage';
COMMENT ON TABLE ai_workflow_results IS 'Results and metadata from AI-enhanced workflows';
COMMENT ON TABLE ai_content_moderation IS 'Content moderation results and flagged content tracking';
COMMENT ON TABLE ai_model_metrics IS 'Performance metrics aggregated by model and tenant';

COMMENT ON VIEW ai_usage_summary IS 'Hourly aggregated usage statistics by tenant, model, and capability';
COMMENT ON VIEW ai_tenant_daily_usage IS 'Daily usage summary by tenant for reporting and analytics';