use ai_service::{Config, AIService};
use std::env;

#[tokio::test]
async fn test_ai_service_configuration() {
    // Test that the AI service can be configured with environment variables
    let config = Config {
        database_url: "postgresql://test:test@localhost/test".to_string(),
        redis_url: "redis://localhost:6379".to_string(),
        temporal_server_url: "http://localhost:7233".to_string(),
        ai_providers: ai_service::config::AIProvidersConfig {
            openai: ai_service::config::OpenAIConfig {
                api_key: "test-key".to_string(),
                base_url: None,
                default_model: "gpt-3.5-turbo".to_string(),
                max_tokens: 4096,
                temperature: 0.7,
            },
            anthropic: ai_service::config::AnthropicConfig {
                api_key: "test-key".to_string(),
                base_url: None,
                default_model: "claude-3-sonnet-20240229".to_string(),
                max_tokens: 4096,
            },
            local: ai_service::config::LocalAIConfig {
                enabled: false,
                base_url: "http://localhost:11434".to_string(),
                models: vec!["llama2-7b".to_string()],
            },
        },
        monitoring: ai_service::config::MonitoringConfig {
            metrics_enabled: true,
            prometheus_port: 9090,
            usage_tracking_enabled: true,
            cost_tracking_enabled: true,
        },
        security: ai_service::config::SecurityConfig {
            jwt_secret: "test-secret".to_string(),
            rate_limit_per_minute: 60,
            max_request_size: 1048576,
        },
    };
    
    // Verify configuration is valid
    assert!(!config.database_url.is_empty());
    assert!(!config.ai_providers.openai.api_key.is_empty());
    assert!(config.monitoring.metrics_enabled);
}

#[tokio::test]
async fn test_model_registry() {
    use ai_service::models::AIModelRegistry;
    use ai_service::types::{AICapability, SubscriptionTier};
    
    let registry = AIModelRegistry::new();
    
    // Test that default models are loaded
    let models = registry.list_all_models();
    assert!(!models.is_empty());
    
    // Test model lookup
    let gpt_model = registry.get_model("gpt-3.5-turbo");
    assert!(gpt_model.is_some());
    assert_eq!(gpt_model.unwrap().name, "GPT-3.5 Turbo");
    
    // Test capability filtering
    let text_gen_models = registry.get_models_for_capability(&AICapability::TextGeneration);
    assert!(!text_gen_models.is_empty());
    
    // Test tier filtering
    let free_models = registry.get_models_for_tier(&SubscriptionTier::Free);
    assert!(!free_models.is_empty());
    
    // Test best model selection
    let best_model = registry.get_best_model_for_capability_and_tier(
        &AICapability::TextGeneration,
        &SubscriptionTier::Professional,
    );
    assert!(best_model.is_some());
}

#[tokio::test]
async fn test_ai_request_types() {
    use ai_service::types::*;
    use std::collections::HashMap;
    
    // Test AI request serialization
    let request = AIRequest {
        model: "gpt-3.5-turbo".to_string(),
        prompt: "Hello, world!".to_string(),
        parameters: AIParameters {
            max_tokens: Some(100),
            temperature: Some(0.7),
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
        },
        context: RequestContext {
            tenant_id: "test-tenant".to_string(),
            user_id: "test-user".to_string(),
            session_id: None,
            workflow_id: None,
            activity_id: None,
        },
    };
    
    // Test serialization
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("gpt-3.5-turbo"));
    assert!(json.contains("Hello, world!"));
    
    // Test deserialization
    let deserialized: AIRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.model, request.model);
    assert_eq!(deserialized.prompt, request.prompt);
}

#[tokio::test]
async fn test_workflow_types() {
    use ai_service::workflows::*;
    
    // Test user onboarding request
    let onboarding_request = UserOnboardingAIRequest {
        user_id: "user123".to_string(),
        tenant_id: "tenant456".to_string(),
        user_profile: UserProfile {
            name: "John Doe".to_string(),
            email: "john@example.com".to_string(),
            role: "Developer".to_string(),
            department: Some("Engineering".to_string()),
            experience_level: ExperienceLevel::Intermediate,
            interests: vec!["API Development".to_string()],
        },
        onboarding_preferences: OnboardingPreferences {
            communication_style: CommunicationStyle::Technical,
            learning_pace: LearningPace::Fast,
            preferred_features: vec!["API Gateway".to_string()],
            goals: vec!["Build APIs".to_string()],
        },
    };
    
    // Test serialization
    let json = serde_json::to_string(&onboarding_request).unwrap();
    assert!(json.contains("John Doe"));
    assert!(json.contains("Developer"));
    
    // Test document processing request
    let doc_request = DocumentProcessingAIRequest {
        document_id: "doc123".to_string(),
        tenant_id: "tenant456".to_string(),
        user_id: "user123".to_string(),
        document_content: "Contract content...".to_string(),
        document_type: DocumentType::Contract,
        processing_options: DocumentProcessingOptions {
            extract_entities: true,
            generate_summary: true,
            classify_content: true,
            extract_key_points: true,
            sentiment_analysis: false,
            custom_instructions: None,
        },
    };
    
    let json = serde_json::to_string(&doc_request).unwrap();
    assert!(json.contains("Contract"));
    assert!(json.contains("extract_entities"));
}

#[test]
fn test_error_handling() {
    use ai_service::error::{AIError, ActivityError};
    
    // Test AI error types
    let db_error = AIError::Database(sqlx::Error::RowNotFound);
    assert!(matches!(db_error, AIError::Database(_)));
    
    let provider_error = AIError::AIProvider("Provider unavailable".to_string());
    assert!(matches!(provider_error, AIError::AIProvider(_)));
    
    // Test activity error types
    let activity_error = ActivityError::ModelUnavailable("Model not found".to_string());
    assert!(activity_error.is_retryable());
    
    let validation_error = ActivityError::InvalidInput("Invalid prompt".to_string());
    assert!(!validation_error.is_retryable());
}

#[test]
fn test_provider_types() {
    use ai_service::types::AIProvider;
    
    // Test provider enum
    let openai = AIProvider::OpenAI;
    let anthropic = AIProvider::Anthropic;
    let local = AIProvider::Local;
    
    // Test serialization
    let json = serde_json::to_string(&openai).unwrap();
    assert_eq!(json, "\"OpenAI\"");
    
    let json = serde_json::to_string(&anthropic).unwrap();
    assert_eq!(json, "\"Anthropic\"");
    
    let json = serde_json::to_string(&local).unwrap();
    assert_eq!(json, "\"Local\"");
}

#[test]
fn test_capability_types() {
    use ai_service::types::AICapability;
    
    let capabilities = vec![
        AICapability::TextGeneration,
        AICapability::TextClassification,
        AICapability::TextSummarization,
        AICapability::EntityExtraction,
        AICapability::SentimentAnalysis,
    ];
    
    for capability in capabilities {
        let json = serde_json::to_string(&capability).unwrap();
        assert!(!json.is_empty());
        
        let deserialized: AICapability = serde_json::from_str(&json).unwrap();
        assert_eq!(std::mem::discriminant(&capability), std::mem::discriminant(&deserialized));
    }
}