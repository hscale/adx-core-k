use crate::error::{ActivityError, AIResult};
use crate::models::AIModelRegistry;
use crate::providers::AIProviderManager;
use crate::services::{AIService, UsageTracker};
use crate::types::*;
use async_trait::async_trait;
use std::sync::Arc;
use crate::temporal_stubs::ActContext;

#[async_trait]
pub trait AIActivities {
    async fn generate_text(&self, ctx: ActContext, request: TextGenerationRequest) -> Result<TextGenerationResult, ActivityError>;
    async fn classify_text(&self, ctx: ActContext, request: TextClassificationRequest) -> Result<TextClassificationResult, ActivityError>;
    async fn summarize_text(&self, ctx: ActContext, request: TextSummarizationRequest) -> Result<TextSummarizationResult, ActivityError>;
    async fn extract_entities(&self, ctx: ActContext, request: EntityExtractionRequest) -> Result<EntityExtractionResult, ActivityError>;
    async fn validate_ai_request(&self, ctx: ActContext, request: AIRequest) -> Result<ValidationResult, ActivityError>;
    async fn track_ai_usage(&self, ctx: ActContext, usage_record: AIUsageRecord) -> Result<(), ActivityError>;
    async fn check_ai_quotas(&self, ctx: ActContext, context: RequestContext, capability: AICapability) -> Result<QuotaCheckResult, ActivityError>;
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub estimated_cost: f64,
    pub estimated_tokens: u32,
}

#[derive(Debug, Clone)]
pub struct QuotaCheckResult {
    pub allowed: bool,
    pub remaining_requests: u32,
    pub remaining_tokens: u64,
    pub reset_time: chrono::DateTime<chrono::Utc>,
    pub reason: Option<String>,
}

pub struct AIActivitiesImpl {
    ai_service: Arc<AIService>,
    provider_manager: Arc<AIProviderManager>,
    model_registry: Arc<AIModelRegistry>,
    usage_tracker: Arc<UsageTracker>,
}

impl AIActivitiesImpl {
    pub fn new(
        ai_service: Arc<AIService>,
        provider_manager: Arc<AIProviderManager>,
        model_registry: Arc<AIModelRegistry>,
        usage_tracker: Arc<UsageTracker>,
    ) -> Self {
        Self {
            ai_service,
            provider_manager,
            model_registry,
            usage_tracker,
        }
    }
    
    fn select_model_for_request(&self, capability: &AICapability, context: &RequestContext) -> Result<String, ActivityError> {
        // Get tenant subscription tier (simplified - would normally query database)
        let tier = SubscriptionTier::Professional; // Default for now
        
        let model = self.model_registry
            .get_best_model_for_capability_and_tier(capability, &tier)
            .ok_or_else(|| ActivityError::ModelUnavailable(
                format!("No model available for capability {:?} and tier {:?}", capability, tier)
            ))?;
        
        Ok(model.id.clone())
    }
    
    async fn validate_content(&self, content: &str) -> Result<(), ActivityError> {
        // Basic content validation (could be enhanced with more sophisticated filtering)
        if content.trim().is_empty() {
            return Err(ActivityError::InvalidInput("Empty content".to_string()));
        }
        
        if content.len() > 100000 { // 100KB limit
            return Err(ActivityError::InvalidInput("Content too large".to_string()));
        }
        
        // Check for potentially harmful content (simplified)
        let harmful_patterns = ["hack", "exploit", "malware", "virus"];
        for pattern in &harmful_patterns {
            if content.to_lowercase().contains(pattern) {
                return Err(ActivityError::ContentPolicyViolation(
                    format!("Content contains potentially harmful pattern: {}", pattern)
                ));
            }
        }
        
        Ok(())
    }
}

#[async_trait]
impl AIActivities for AIActivitiesImpl {
    async fn generate_text(&self, _ctx: ActContext, request: TextGenerationRequest) -> Result<TextGenerationResult, ActivityError> {
        // Validate content
        self.validate_content(&request.prompt).await?;
        
        // Check quotas
        let quota_check = self.check_ai_quotas(
            _ctx.clone(),
            request.context.clone(),
            AICapability::TextGeneration,
        ).await?;
        
        if !quota_check.allowed {
            return Err(ActivityError::QuotaExceeded(
                quota_check.reason.unwrap_or_else(|| "Quota exceeded".to_string())
            ));
        }
        
        // Select appropriate model if not specified
        let model = if let Some(ref model) = request.model {
            model.clone()
        } else {
            self.select_model_for_request(&AICapability::TextGeneration, &request.context)?
        };
        
        // Get model info to determine provider
        let model_info = self.model_registry.get_model(&model)
            .ok_or_else(|| ActivityError::ModelUnavailable(format!("Model {} not found", model)))?;
        
        // Get provider
        let provider = self.provider_manager.get_provider(&model_info.provider)
            .map_err(|e| ActivityError::ExternalServiceError(e.to_string()))?;
        
        // Generate text
        let result = provider.generate_text(&request).await
            .map_err(|e| ActivityError::GenerationFailed(e.to_string()))?;
        
        // Track usage
        let usage_record = AIUsageRecord {
            id: uuid::Uuid::new_v4(),
            tenant_id: request.context.tenant_id.clone(),
            user_id: request.context.user_id.clone(),
            workflow_id: request.context.workflow_id.clone(),
            activity_id: request.context.activity_id.clone(),
            model: model.clone(),
            capability: AICapability::TextGeneration,
            usage: result.usage.clone(),
            request_timestamp: chrono::Utc::now(),
            response_timestamp: chrono::Utc::now(),
            success: true,
            error_code: None,
        };
        
        self.track_ai_usage(_ctx, usage_record).await?;
        
        Ok(result)
    }
    
    async fn classify_text(&self, _ctx: ActContext, request: TextClassificationRequest) -> Result<TextClassificationResult, ActivityError> {
        // Validate content
        self.validate_content(&request.text).await?;
        
        // Check quotas
        let quota_check = self.check_ai_quotas(
            _ctx.clone(),
            request.context.clone(),
            AICapability::TextClassification,
        ).await?;
        
        if !quota_check.allowed {
            return Err(ActivityError::QuotaExceeded(
                quota_check.reason.unwrap_or_else(|| "Quota exceeded".to_string())
            ));
        }
        
        // Select appropriate model if not specified
        let model = if let Some(ref model) = request.model {
            model.clone()
        } else {
            self.select_model_for_request(&AICapability::TextClassification, &request.context)?
        };
        
        // Get model info to determine provider
        let model_info = self.model_registry.get_model(&model)
            .ok_or_else(|| ActivityError::ModelUnavailable(format!("Model {} not found", model)))?;
        
        // Get provider
        let provider = self.provider_manager.get_provider(&model_info.provider)
            .map_err(|e| ActivityError::ExternalServiceError(e.to_string()))?;
        
        // Classify text
        let result = provider.classify_text(&request).await
            .map_err(|e| ActivityError::GenerationFailed(e.to_string()))?;
        
        // Track usage
        let usage_record = AIUsageRecord {
            id: uuid::Uuid::new_v4(),
            tenant_id: request.context.tenant_id.clone(),
            user_id: request.context.user_id.clone(),
            workflow_id: request.context.workflow_id.clone(),
            activity_id: request.context.activity_id.clone(),
            model: model.clone(),
            capability: AICapability::TextClassification,
            usage: result.usage.clone(),
            request_timestamp: chrono::Utc::now(),
            response_timestamp: chrono::Utc::now(),
            success: true,
            error_code: None,
        };
        
        self.track_ai_usage(_ctx, usage_record).await?;
        
        Ok(result)
    }
    
    async fn summarize_text(&self, _ctx: ActContext, request: TextSummarizationRequest) -> Result<TextSummarizationResult, ActivityError> {
        // Validate content
        self.validate_content(&request.text).await?;
        
        // Check quotas
        let quota_check = self.check_ai_quotas(
            _ctx.clone(),
            request.context.clone(),
            AICapability::TextSummarization,
        ).await?;
        
        if !quota_check.allowed {
            return Err(ActivityError::QuotaExceeded(
                quota_check.reason.unwrap_or_else(|| "Quota exceeded".to_string())
            ));
        }
        
        // Select appropriate model if not specified
        let model = if let Some(ref model) = request.model {
            model.clone()
        } else {
            self.select_model_for_request(&AICapability::TextSummarization, &request.context)?
        };
        
        // Get model info to determine provider
        let model_info = self.model_registry.get_model(&model)
            .ok_or_else(|| ActivityError::ModelUnavailable(format!("Model {} not found", model)))?;
        
        // Get provider
        let provider = self.provider_manager.get_provider(&model_info.provider)
            .map_err(|e| ActivityError::ExternalServiceError(e.to_string()))?;
        
        // Summarize text
        let result = provider.summarize_text(&request).await
            .map_err(|e| ActivityError::GenerationFailed(e.to_string()))?;
        
        // Track usage
        let usage_record = AIUsageRecord {
            id: uuid::Uuid::new_v4(),
            tenant_id: request.context.tenant_id.clone(),
            user_id: request.context.user_id.clone(),
            workflow_id: request.context.workflow_id.clone(),
            activity_id: request.context.activity_id.clone(),
            model: model.clone(),
            capability: AICapability::TextSummarization,
            usage: result.usage.clone(),
            request_timestamp: chrono::Utc::now(),
            response_timestamp: chrono::Utc::now(),
            success: true,
            error_code: None,
        };
        
        self.track_ai_usage(_ctx, usage_record).await?;
        
        Ok(result)
    }
    
    async fn extract_entities(&self, _ctx: ActContext, request: EntityExtractionRequest) -> Result<EntityExtractionResult, ActivityError> {
        // Validate content
        self.validate_content(&request.text).await?;
        
        // Check quotas
        let quota_check = self.check_ai_quotas(
            _ctx.clone(),
            request.context.clone(),
            AICapability::EntityExtraction,
        ).await?;
        
        if !quota_check.allowed {
            return Err(ActivityError::QuotaExceeded(
                quota_check.reason.unwrap_or_else(|| "Quota exceeded".to_string())
            ));
        }
        
        // Select appropriate model if not specified
        let model = if let Some(ref model) = request.model {
            model.clone()
        } else {
            self.select_model_for_request(&AICapability::EntityExtraction, &request.context)?
        };
        
        // Get model info to determine provider
        let model_info = self.model_registry.get_model(&model)
            .ok_or_else(|| ActivityError::ModelUnavailable(format!("Model {} not found", model)))?;
        
        // Get provider
        let provider = self.provider_manager.get_provider(&model_info.provider)
            .map_err(|e| ActivityError::ExternalServiceError(e.to_string()))?;
        
        // Extract entities
        let result = provider.extract_entities(&request).await
            .map_err(|e| ActivityError::GenerationFailed(e.to_string()))?;
        
        // Track usage
        let usage_record = AIUsageRecord {
            id: uuid::Uuid::new_v4(),
            tenant_id: request.context.tenant_id.clone(),
            user_id: request.context.user_id.clone(),
            workflow_id: request.context.workflow_id.clone(),
            activity_id: request.context.activity_id.clone(),
            model: model.clone(),
            capability: AICapability::EntityExtraction,
            usage: result.usage.clone(),
            request_timestamp: chrono::Utc::now(),
            response_timestamp: chrono::Utc::now(),
            success: true,
            error_code: None,
        };
        
        self.track_ai_usage(_ctx, usage_record).await?;
        
        Ok(result)
    }
    
    async fn validate_ai_request(&self, _ctx: ActContext, request: AIRequest) -> Result<ValidationResult, ActivityError> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        // Validate prompt
        if request.prompt.trim().is_empty() {
            errors.push("Prompt cannot be empty".to_string());
        }
        
        if request.prompt.len() > 50000 {
            errors.push("Prompt too long (max 50,000 characters)".to_string());
        }
        
        // Validate model
        if let Some(model_info) = self.model_registry.get_model(&request.model) {
            // Check if model supports the required capability (inferred from context)
            // This is simplified - in practice, you'd determine capability from the request type
            
            // Estimate tokens and cost
            let estimated_tokens = (request.prompt.len() / 4) as u32; // Rough estimation
            let estimated_cost = (estimated_tokens as f64) * model_info.cost_per_token;
            
            if estimated_tokens > model_info.max_tokens {
                warnings.push(format!(
                    "Request may exceed model token limit ({} > {})",
                    estimated_tokens, model_info.max_tokens
                ));
            }
            
            Ok(ValidationResult {
                is_valid: errors.is_empty(),
                errors,
                warnings,
                estimated_cost,
                estimated_tokens,
            })
        } else {
            errors.push(format!("Model '{}' not found", request.model));
            Ok(ValidationResult {
                is_valid: false,
                errors,
                warnings,
                estimated_cost: 0.0,
                estimated_tokens: 0,
            })
        }
    }
    
    async fn track_ai_usage(&self, _ctx: ActContext, usage_record: AIUsageRecord) -> Result<(), ActivityError> {
        self.usage_tracker.record_usage(usage_record).await
            .map_err(|e| ActivityError::ExternalServiceError(format!("Failed to track usage: {}", e)))
    }
    
    async fn check_ai_quotas(&self, _ctx: ActContext, context: RequestContext, capability: AICapability) -> Result<QuotaCheckResult, ActivityError> {
        // Check tenant quotas (simplified implementation)
        let current_usage = self.usage_tracker.get_current_usage(&context.tenant_id, &capability).await
            .map_err(|e| ActivityError::ExternalServiceError(format!("Failed to check quotas: {}", e)))?;
        
        // Default quotas (would normally be retrieved from database based on subscription tier)
        let quota_limits = match capability {
            AICapability::TextGeneration => (1000, 100000), // requests, tokens
            AICapability::TextClassification => (2000, 50000),
            AICapability::TextSummarization => (500, 200000),
            AICapability::EntityExtraction => (1000, 100000),
            _ => (100, 10000),
        };
        
        let allowed = current_usage.requests < quota_limits.0 && current_usage.tokens < quota_limits.1;
        let reason = if !allowed {
            Some(format!(
                "Quota exceeded: {}/{} requests, {}/{} tokens",
                current_usage.requests, quota_limits.0,
                current_usage.tokens, quota_limits.1
            ))
        } else {
            None
        };
        
        Ok(QuotaCheckResult {
            allowed,
            remaining_requests: quota_limits.0.saturating_sub(current_usage.requests),
            remaining_tokens: quota_limits.1.saturating_sub(current_usage.tokens),
            reset_time: chrono::Utc::now() + chrono::Duration::hours(1), // Reset every hour
            reason,
        })
    }
}

#[derive(Debug, Clone)]
pub struct CurrentUsage {
    pub requests: u32,
    pub tokens: u64,
}