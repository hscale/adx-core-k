use crate::config::Config;
use crate::error::{AIError, AIResult};
use crate::models::AIModelRegistry;
use crate::providers::AIProviderManager;
use crate::types::*;
use sqlx::PgPool;
use std::sync::Arc;

pub struct AIService {
    config: Config,
    db_pool: Arc<PgPool>,
    provider_manager: Arc<AIProviderManager>,
    model_registry: Arc<AIModelRegistry>,
}

impl AIService {
    pub async fn new(config: Config) -> AIResult<Self> {
        // Initialize database connection
        let db_pool = Arc::new(
            PgPool::connect(&config.database_url)
                .await
                .map_err(AIError::Database)?,
        );
        
        // Run migrations
        sqlx::migrate!("./migrations")
            .run(&*db_pool)
            .await
            .map_err(AIError::Database)?;
        
        // Initialize AI providers
        let provider_manager = Arc::new(AIProviderManager::new(&config.ai_providers));
        
        // Initialize model registry
        let model_registry = Arc::new(AIModelRegistry::new());
        
        Ok(Self {
            config,
            db_pool,
            provider_manager,
            model_registry,
        })
    }
    
    pub fn get_provider_manager(&self) -> Arc<AIProviderManager> {
        self.provider_manager.clone()
    }
    
    pub fn get_model_registry(&self) -> Arc<AIModelRegistry> {
        self.model_registry.clone()
    }
    
    pub fn get_db_pool(&self) -> Arc<PgPool> {
        self.db_pool.clone()
    }
    
    pub async fn get_available_models(&self, tenant_tier: &SubscriptionTier) -> AIResult<Vec<AIModel>> {
        let models = self.model_registry.get_models_for_tier(tenant_tier);
        Ok(models.into_iter().cloned().collect())
    }
    
    pub async fn get_models_for_capability(
        &self,
        capability: &AICapability,
        tenant_tier: &SubscriptionTier,
    ) -> AIResult<Vec<AIModel>> {
        let models: Vec<AIModel> = self.model_registry
            .get_models_for_capability(capability)
            .into_iter()
            .filter(|model| model.tier_availability.contains(tenant_tier))
            .cloned()
            .collect();
        
        Ok(models)
    }
    
    pub async fn validate_model_access(
        &self,
        model_id: &str,
        tenant_tier: &SubscriptionTier,
    ) -> AIResult<bool> {
        if let Some(model) = self.model_registry.get_model(model_id) {
            Ok(model.tier_availability.contains(tenant_tier))
        } else {
            Ok(false)
        }
    }
    
    pub async fn estimate_request_cost(
        &self,
        model_id: &str,
        estimated_tokens: u32,
    ) -> AIResult<f64> {
        if let Some(model) = self.model_registry.get_model(model_id) {
            Ok((estimated_tokens as f64) * model.cost_per_token)
        } else {
            Err(AIError::ModelNotAvailable(format!("Model {} not found", model_id)))
        }
    }
    
    pub async fn get_service_health(&self) -> AIResult<AIServiceHealth> {
        let provider_health = self.provider_manager.health_check_all().await?;
        
        // Check overall status
        let overall_status = if provider_health.values().all(|h| matches!(h.status, HealthStatus::Healthy)) {
            HealthStatus::Healthy
        } else if provider_health.values().any(|h| matches!(h.status, HealthStatus::Healthy)) {
            HealthStatus::Degraded
        } else {
            HealthStatus::Unhealthy
        };
        
        // Get model health (simplified)
        let mut model_health = std::collections::HashMap::new();
        for model in self.model_registry.list_all_models() {
            if let Some(provider_health_info) = provider_health.get(&model.provider) {
                model_health.insert(model.id.clone(), ModelHealth {
                    status: provider_health_info.status.clone(),
                    availability: if matches!(provider_health_info.status, HealthStatus::Healthy) { 1.0 } else { 0.0 },
                    avg_response_time_ms: provider_health_info.response_time_ms.unwrap_or(0) as f64,
                    error_rate: provider_health_info.error_rate,
                    last_check: provider_health_info.last_check,
                });
            }
        }
        
        Ok(AIServiceHealth {
            status: overall_status,
            providers: provider_health,
            models: model_health,
            last_check: chrono::Utc::now(),
        })
    }
    
    pub async fn create_ai_request(
        &self,
        prompt: String,
        model: String,
        parameters: AIParameters,
        context: RequestContext,
    ) -> AIResult<AIRequest> {
        // Validate model access
        let tenant_tier = SubscriptionTier::Professional; // Would normally be retrieved from database
        
        if !self.validate_model_access(&model, &tenant_tier).await? {
            return Err(AIError::Authorization(
                format!("Model {} not available for tenant tier {:?}", model, tenant_tier)
            ));
        }
        
        Ok(AIRequest {
            model,
            prompt,
            parameters,
            context,
        })
    }
    
    pub async fn process_ai_request(&self, request: AIRequest) -> AIResult<AIResponse> {
        // Get model info
        let model_info = self.model_registry.get_model(&request.model)
            .ok_or_else(|| AIError::ModelNotAvailable(format!("Model {} not found", request.model)))?;
        
        // Get provider
        let provider = self.provider_manager.get_provider(&model_info.provider)?;
        
        // Create text generation request
        let text_request = TextGenerationRequest {
            prompt: request.prompt.clone(),
            model: Some(request.model.clone()),
            parameters: request.parameters.clone(),
            context: request.context.clone(),
        };
        
        // Generate text
        let result = provider.generate_text(&text_request).await?;
        
        // Create response
        Ok(AIResponse {
            id: uuid::Uuid::new_v4().to_string(),
            content: result.generated_text,
            model: request.model,
            usage: result.usage,
            finish_reason: FinishReason::Stop, // Simplified
            created_at: chrono::Utc::now(),
            metadata: result.metadata,
        })
    }
}