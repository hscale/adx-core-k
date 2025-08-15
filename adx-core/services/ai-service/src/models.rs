use crate::types::*;
use std::collections::HashMap;

pub struct AIModelRegistry {
    models: HashMap<String, AIModel>,
}

impl AIModelRegistry {
    pub fn new() -> Self {
        let mut registry = Self {
            models: HashMap::new(),
        };
        
        registry.register_default_models();
        registry
    }
    
    fn register_default_models(&mut self) {
        // OpenAI Models
        self.register_model(AIModel {
            id: "gpt-3.5-turbo".to_string(),
            name: "GPT-3.5 Turbo".to_string(),
            provider: AIProvider::OpenAI,
            capabilities: vec![
                AICapability::TextGeneration,
                AICapability::TextClassification,
                AICapability::TextSummarization,
                AICapability::EntityExtraction,
                AICapability::SentimentAnalysis,
            ],
            max_tokens: 4096,
            cost_per_token: 0.0000015, // $0.0015 per 1K tokens
            tier_availability: vec![
                SubscriptionTier::Professional,
                SubscriptionTier::Enterprise,
            ],
        });
        
        self.register_model(AIModel {
            id: "gpt-4".to_string(),
            name: "GPT-4".to_string(),
            provider: AIProvider::OpenAI,
            capabilities: vec![
                AICapability::TextGeneration,
                AICapability::TextClassification,
                AICapability::TextSummarization,
                AICapability::EntityExtraction,
                AICapability::SentimentAnalysis,
                AICapability::CodeGeneration,
            ],
            max_tokens: 8192,
            cost_per_token: 0.00003, // $0.03 per 1K tokens
            tier_availability: vec![SubscriptionTier::Enterprise],
        });
        
        self.register_model(AIModel {
            id: "gpt-4-turbo".to_string(),
            name: "GPT-4 Turbo".to_string(),
            provider: AIProvider::OpenAI,
            capabilities: vec![
                AICapability::TextGeneration,
                AICapability::TextClassification,
                AICapability::TextSummarization,
                AICapability::EntityExtraction,
                AICapability::SentimentAnalysis,
                AICapability::CodeGeneration,
            ],
            max_tokens: 128000,
            cost_per_token: 0.00001, // $0.01 per 1K tokens
            tier_availability: vec![SubscriptionTier::Enterprise],
        });
        
        // Anthropic Models
        self.register_model(AIModel {
            id: "claude-3-haiku-20240307".to_string(),
            name: "Claude 3 Haiku".to_string(),
            provider: AIProvider::Anthropic,
            capabilities: vec![
                AICapability::TextGeneration,
                AICapability::TextClassification,
                AICapability::TextSummarization,
                AICapability::EntityExtraction,
            ],
            max_tokens: 4096,
            cost_per_token: 0.00000025, // $0.25 per 1M tokens
            tier_availability: vec![
                SubscriptionTier::Professional,
                SubscriptionTier::Enterprise,
            ],
        });
        
        self.register_model(AIModel {
            id: "claude-3-sonnet-20240229".to_string(),
            name: "Claude 3 Sonnet".to_string(),
            provider: AIProvider::Anthropic,
            capabilities: vec![
                AICapability::TextGeneration,
                AICapability::TextClassification,
                AICapability::TextSummarization,
                AICapability::EntityExtraction,
                AICapability::SentimentAnalysis,
                AICapability::CodeGeneration,
            ],
            max_tokens: 4096,
            cost_per_token: 0.000003, // $3 per 1M tokens
            tier_availability: vec![SubscriptionTier::Enterprise],
        });
        
        // Local/Open Source Models (for development and cost-effective options)
        self.register_model(AIModel {
            id: "llama2-7b".to_string(),
            name: "Llama 2 7B".to_string(),
            provider: AIProvider::Local,
            capabilities: vec![
                AICapability::TextGeneration,
                AICapability::TextClassification,
                AICapability::TextSummarization,
            ],
            max_tokens: 4096,
            cost_per_token: 0.0, // No cost for local models
            tier_availability: vec![
                SubscriptionTier::Free,
                SubscriptionTier::Professional,
                SubscriptionTier::Enterprise,
            ],
        });
        
        self.register_model(AIModel {
            id: "mistral-7b".to_string(),
            name: "Mistral 7B".to_string(),
            provider: AIProvider::Local,
            capabilities: vec![
                AICapability::TextGeneration,
                AICapability::CodeGeneration,
            ],
            max_tokens: 8192,
            cost_per_token: 0.0,
            tier_availability: vec![
                SubscriptionTier::Free,
                SubscriptionTier::Professional,
                SubscriptionTier::Enterprise,
            ],
        });
    }
    
    pub fn register_model(&mut self, model: AIModel) {
        self.models.insert(model.id.clone(), model);
    }
    
    pub fn get_model(&self, model_id: &str) -> Option<&AIModel> {
        self.models.get(model_id)
    }
    
    pub fn get_models_for_capability(&self, capability: &AICapability) -> Vec<&AIModel> {
        self.models
            .values()
            .filter(|model| model.capabilities.contains(capability))
            .collect()
    }
    
    pub fn get_models_for_tier(&self, tier: &SubscriptionTier) -> Vec<&AIModel> {
        self.models
            .values()
            .filter(|model| model.tier_availability.contains(tier))
            .collect()
    }
    
    pub fn get_best_model_for_capability_and_tier(
        &self,
        capability: &AICapability,
        tier: &SubscriptionTier,
    ) -> Option<&AIModel> {
        let mut available_models: Vec<&AIModel> = self.models
            .values()
            .filter(|model| {
                model.capabilities.contains(capability) && model.tier_availability.contains(tier)
            })
            .collect();
        
        // Sort by cost per token (ascending) and max tokens (descending)
        available_models.sort_by(|a, b| {
            a.cost_per_token
                .partial_cmp(&b.cost_per_token)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.max_tokens.cmp(&a.max_tokens))
        });
        
        available_models.first().copied()
    }
    
    pub fn list_all_models(&self) -> Vec<&AIModel> {
        self.models.values().collect()
    }
    
    pub fn get_models_by_provider(&self, provider: &AIProvider) -> Vec<&AIModel> {
        self.models
            .values()
            .filter(|model| std::mem::discriminant(&model.provider) == std::mem::discriminant(provider))
            .collect()
    }
}

impl Default for AIModelRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_registry_creation() {
        let registry = AIModelRegistry::new();
        assert!(!registry.models.is_empty());
    }

    #[test]
    fn test_get_model() {
        let registry = AIModelRegistry::new();
        let model = registry.get_model("gpt-3.5-turbo");
        assert!(model.is_some());
        assert_eq!(model.unwrap().name, "GPT-3.5 Turbo");
    }

    #[test]
    fn test_get_models_for_capability() {
        let registry = AIModelRegistry::new();
        let models = registry.get_models_for_capability(&AICapability::TextGeneration);
        assert!(!models.is_empty());
    }

    #[test]
    fn test_get_models_for_tier() {
        let registry = AIModelRegistry::new();
        let free_models = registry.get_models_for_tier(&SubscriptionTier::Free);
        let enterprise_models = registry.get_models_for_tier(&SubscriptionTier::Enterprise);
        
        assert!(!free_models.is_empty());
        assert!(!enterprise_models.is_empty());
        assert!(enterprise_models.len() >= free_models.len());
    }

    #[test]
    fn test_best_model_selection() {
        let registry = AIModelRegistry::new();
        let best_model = registry.get_best_model_for_capability_and_tier(
            &AICapability::TextGeneration,
            &SubscriptionTier::Professional,
        );
        
        assert!(best_model.is_some());
        let model = best_model.unwrap();
        assert!(model.capabilities.contains(&AICapability::TextGeneration));
        assert!(model.tier_availability.contains(&SubscriptionTier::Professional));
    }
}