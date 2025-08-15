pub mod openai;
pub mod anthropic;
pub mod local;

use crate::error::{AIError, AIResult};
use crate::types::*;
use async_trait::async_trait;

#[async_trait]
pub trait AIProvider: Send + Sync {
    async fn generate_text(&self, request: &TextGenerationRequest) -> AIResult<TextGenerationResult>;
    async fn classify_text(&self, request: &TextClassificationRequest) -> AIResult<TextClassificationResult>;
    async fn summarize_text(&self, request: &TextSummarizationRequest) -> AIResult<TextSummarizationResult>;
    async fn extract_entities(&self, request: &EntityExtractionRequest) -> AIResult<EntityExtractionResult>;
    async fn health_check(&self) -> AIResult<ProviderHealth>;
    fn get_supported_models(&self) -> Vec<String>;
    fn get_provider_type(&self) -> crate::types::AIProvider;
}

pub struct AIProviderManager {
    openai: Option<openai::OpenAIProvider>,
    anthropic: Option<anthropic::AnthropicProvider>,
    local: Option<local::LocalAIProvider>,
}

impl AIProviderManager {
    pub fn new(config: &crate::config::AIProvidersConfig) -> Self {
        let openai = if !config.openai.api_key.is_empty() {
            Some(openai::OpenAIProvider::new(&config.openai))
        } else {
            None
        };
        
        let anthropic = if !config.anthropic.api_key.is_empty() {
            Some(anthropic::AnthropicProvider::new(&config.anthropic))
        } else {
            None
        };
        
        let local = if config.local.enabled {
            Some(local::LocalAIProvider::new(&config.local))
        } else {
            None
        };
        
        Self {
            openai,
            anthropic,
            local,
        }
    }
    
    pub fn get_provider(&self, provider_type: &crate::types::AIProvider) -> AIResult<&dyn AIProvider> {
        match provider_type {
            crate::types::AIProvider::OpenAI => {
                self.openai.as_ref()
                    .map(|p| p as &dyn AIProvider)
                    .ok_or_else(|| AIError::AIProvider("OpenAI provider not configured".to_string()))
            }
            crate::types::AIProvider::Anthropic => {
                self.anthropic.as_ref()
                    .map(|p| p as &dyn AIProvider)
                    .ok_or_else(|| AIError::AIProvider("Anthropic provider not configured".to_string()))
            }
            crate::types::AIProvider::Local => {
                self.local.as_ref()
                    .map(|p| p as &dyn AIProvider)
                    .ok_or_else(|| AIError::AIProvider("Local AI provider not configured".to_string()))
            }
        }
    }
    
    pub async fn health_check_all(&self) -> AIResult<std::collections::HashMap<crate::types::AIProvider, ProviderHealth>> {
        let mut health_results = std::collections::HashMap::new();
        
        if let Some(openai) = &self.openai {
            match openai.health_check().await {
                Ok(health) => {
                    health_results.insert(crate::types::AIProvider::OpenAI, health);
                }
                Err(e) => {
                    health_results.insert(crate::types::AIProvider::OpenAI, ProviderHealth {
                        status: HealthStatus::Unhealthy,
                        response_time_ms: None,
                        error_rate: 1.0,
                        last_error: Some(e.to_string()),
                        last_check: chrono::Utc::now(),
                    });
                }
            }
        }
        
        if let Some(anthropic) = &self.anthropic {
            match anthropic.health_check().await {
                Ok(health) => {
                    health_results.insert(crate::types::AIProvider::Anthropic, health);
                }
                Err(e) => {
                    health_results.insert(crate::types::AIProvider::Anthropic, ProviderHealth {
                        status: HealthStatus::Unhealthy,
                        response_time_ms: None,
                        error_rate: 1.0,
                        last_error: Some(e.to_string()),
                        last_check: chrono::Utc::now(),
                    });
                }
            }
        }
        
        if let Some(local) = &self.local {
            match local.health_check().await {
                Ok(health) => {
                    health_results.insert(crate::types::AIProvider::Local, health);
                }
                Err(e) => {
                    health_results.insert(crate::types::AIProvider::Local, ProviderHealth {
                        status: HealthStatus::Unhealthy,
                        response_time_ms: None,
                        error_rate: 1.0,
                        last_error: Some(e.to_string()),
                        last_check: chrono::Utc::now(),
                    });
                }
            }
        }
        
        Ok(health_results)
    }
}