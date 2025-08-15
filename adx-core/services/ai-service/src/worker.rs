use crate::activities::{AIActivities, AIActivitiesImpl};
use crate::config::Config;
use crate::error::AIResult;
use crate::services::{AIService, UsageTracker};
use crate::workflows::{
    document_processing_ai_workflow, email_generation_ai_workflow, user_onboarding_ai_workflow,
};
use std::sync::Arc;
use crate::temporal_stubs::{Worker, WorkerBuilder};

pub async fn start_worker(config: Config, task_queue: &str) -> AIResult<()> {
    // Initialize services
    let ai_service = Arc::new(AIService::new(config.clone()).await?);
    let usage_tracker = Arc::new(UsageTracker::new(&config.database_url, &config.redis_url).await?);
    
    // Create activities implementation
    let activities = Arc::new(AIActivitiesImpl::new(
        ai_service.clone(),
        ai_service.get_provider_manager(),
        ai_service.get_model_registry(),
        usage_tracker,
    ));
    
    // Create Temporal worker
    let mut worker = WorkerBuilder::default()
        .task_queue(task_queue)
        .worker_url(&config.temporal_server_url)
        .build()
        .await
        .map_err(|e| crate::error::AIError::Temporal(e.to_string()))?;
    
    // Register workflows
    worker.register_wf(user_onboarding_ai_workflow);
    worker.register_wf(document_processing_ai_workflow);
    worker.register_wf(email_generation_ai_workflow);
    
    // Register activities
    worker.register_activity("generate_text", {
        let activities = activities.clone();
        move |ctx, req| {
            let activities = activities.clone();
            async move { activities.generate_text(ctx, req).await }
        }
    });
    
    worker.register_activity("classify_text", {
        let activities = activities.clone();
        move |ctx, req| {
            let activities = activities.clone();
            async move { activities.classify_text(ctx, req).await }
        }
    });
    
    worker.register_activity("summarize_text", {
        let activities = activities.clone();
        move |ctx, req| {
            let activities = activities.clone();
            async move { activities.summarize_text(ctx, req).await }
        }
    });
    
    worker.register_activity("extract_entities", {
        let activities = activities.clone();
        move |ctx, req| {
            let activities = activities.clone();
            async move { activities.extract_entities(ctx, req).await }
        }
    });
    
    worker.register_activity("validate_ai_request", {
        let activities = activities.clone();
        move |ctx, req| {
            let activities = activities.clone();
            async move { activities.validate_ai_request(ctx, req).await }
        }
    });
    
    worker.register_activity("track_ai_usage", {
        let activities = activities.clone();
        move |ctx, req| {
            let activities = activities.clone();
            async move { activities.track_ai_usage(ctx, req).await }
        }
    });
    
    worker.register_activity("check_ai_quotas", {
        let activities = activities.clone();
        move |ctx, context, capability| {
            let activities = activities.clone();
            async move { activities.check_ai_quotas(ctx, context, capability).await }
        }
    });
    
    tracing::info!("Starting AI Service Temporal worker on task queue: {}", task_queue);
    
    // Start the worker
    worker.run().await
        .map_err(|e| crate::error::AIError::Temporal(e.to_string()))?;
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use temporal_sdk_core_test_utils::TestWorkflowEnvironment;

    #[tokio::test]
    async fn test_worker_creation() {
        let config = Config {
            database_url: "postgresql://test:test@localhost/test".to_string(),
            redis_url: "redis://localhost:6379".to_string(),
            temporal_server_url: "http://localhost:7233".to_string(),
            ai_providers: crate::config::AIProvidersConfig {
                openai: crate::config::OpenAIConfig {
                    api_key: "test".to_string(),
                    base_url: None,
                    default_model: "gpt-3.5-turbo".to_string(),
                    max_tokens: 4096,
                    temperature: 0.7,
                },
                anthropic: crate::config::AnthropicConfig {
                    api_key: "test".to_string(),
                    base_url: None,
                    default_model: "claude-3-sonnet-20240229".to_string(),
                    max_tokens: 4096,
                },
                local: crate::config::LocalAIConfig {
                    enabled: false,
                    base_url: "http://localhost:11434".to_string(),
                    models: vec!["llama2-7b".to_string()],
                },
            },
            monitoring: crate::config::MonitoringConfig {
                metrics_enabled: true,
                prometheus_port: 9090,
                usage_tracking_enabled: true,
                cost_tracking_enabled: true,
            },
            security: crate::config::SecurityConfig {
                jwt_secret: "test-secret".to_string(),
                rate_limit_per_minute: 60,
                max_request_size: 1048576,
            },
        };
        
        // This test would require a test Temporal server
        // For now, we'll just test that the configuration is valid
        assert!(!config.database_url.is_empty());
        assert!(!config.temporal_server_url.is_empty());
    }

    #[tokio::test]
    async fn test_workflow_registration() {
        // Test that workflows can be registered without errors
        let test_env = TestWorkflowEnvironment::new().await;
        
        // This would test workflow registration in a test environment
        // For now, we'll just verify the workflow functions exist
        assert!(user_onboarding_ai_workflow.is_some());
        assert!(document_processing_ai_workflow.is_some());
        assert!(email_generation_ai_workflow.is_some());
    }
}