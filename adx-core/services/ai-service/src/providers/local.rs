use crate::config::LocalAIConfig;
use crate::error::{AIError, AIResult};
use crate::providers::AIProvider;
use crate::types::*;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct LocalAIRequest {
    model: String,
    prompt: String,
    max_tokens: u32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct LocalAIResponse {
    id: String,
    object: String,
    created: u64,
    model: String,
    choices: Vec<LocalAIChoice>,
    usage: LocalAIUsage,
}

#[derive(Debug, Deserialize)]
struct LocalAIChoice {
    text: String,
    index: u32,
    finish_reason: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LocalAIUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

pub struct LocalAIProvider {
    client: Client,
    config: LocalAIConfig,
}

impl LocalAIProvider {
    pub fn new(config: &LocalAIConfig) -> Self {
        let client = Client::new();
        
        Self {
            client,
            config: config.clone(),
        }
    }
    
    async fn generate_completion(
        &self,
        prompt: &str,
        model: Option<&str>,
        parameters: &AIParameters,
    ) -> AIResult<LocalAIResponse> {
        let model = model.unwrap_or_else(|| {
            self.config.models.first()
                .map(|s| s.as_str())
                .unwrap_or("llama2-7b")
        });
        
        let request = LocalAIRequest {
            model: model.to_string(),
            prompt: prompt.to_string(),
            max_tokens: parameters.max_tokens.unwrap_or(1000),
            temperature: parameters.temperature.unwrap_or(0.7),
            top_p: parameters.top_p,
            stop: parameters.stop_sequences.clone(),
        };
        
        let response = self
            .client
            .post(&format!("{}/v1/completions", self.config.base_url))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::HttpClient(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::AIProvider(format!("Local AI error: {}", error_text)));
        }
        
        response
            .json::<LocalAIResponse>()
            .await
            .map_err(|e| AIError::AIProvider(format!("Failed to parse Local AI response: {}", e)))
    }
}

#[async_trait]
impl AIProvider for LocalAIProvider {
    async fn generate_text(&self, request: &TextGenerationRequest) -> AIResult<TextGenerationResult> {
        let response = self
            .generate_completion(&request.prompt, request.model.as_deref(), &request.parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from Local AI".to_string()))?;
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
            estimated_cost: 0.0, // Local models have no cost
        };
        
        Ok(TextGenerationResult {
            generated_text: choice.text.clone(),
            usage,
            quality_score: None,
            metadata: HashMap::new(),
        })
    }
    
    async fn classify_text(&self, request: &TextClassificationRequest) -> AIResult<TextClassificationResult> {
        let prompt = format!(
            "Classify the following text into one of these categories: {}\n\nText: {}\n\nCategory:",
            request.categories.join(", "),
            request.text
        );
        
        let parameters = AIParameters {
            max_tokens: Some(50),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        let response = self
            .generate_completion(&prompt, request.model.as_deref(), &parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from Local AI".to_string()))?;
        
        let result_text = choice.text.trim();
        
        // Find the best matching category
        let category = request
            .categories
            .iter()
            .find(|cat| result_text.to_lowercase().contains(&cat.to_lowercase()))
            .unwrap_or(&request.categories[0])
            .clone();
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
            estimated_cost: 0.0,
        };
        
        // Create confidence scores (simplified)
        let mut all_scores = HashMap::new();
        for cat in &request.categories {
            let score = if cat == &category { 0.8 } else { 0.2 };
            all_scores.insert(cat.clone(), score);
        }
        
        Ok(TextClassificationResult {
            category,
            confidence: 0.8, // Lower confidence for local models
            all_scores,
            usage,
        })
    }
    
    async fn summarize_text(&self, request: &TextSummarizationRequest) -> AIResult<TextSummarizationResult> {
        let style_instruction = match request.style.as_ref().unwrap_or(&SummarizationStyle::Abstractive) {
            SummarizationStyle::Extractive => "Extract the most important sentences",
            SummarizationStyle::Abstractive => "Create a concise summary",
            SummarizationStyle::Bullet => "Create a bullet-point summary",
            SummarizationStyle::Executive => "Create an executive summary",
        };
        
        let max_length = request.max_length.unwrap_or(200);
        let prompt = format!(
            "{} of the following text in approximately {} words:\n\n{}\n\nSummary:",
            style_instruction, max_length, request.text
        );
        
        let parameters = AIParameters {
            max_tokens: Some(max_length * 2),
            temperature: Some(0.3),
            ..Default::default()
        };
        
        let response = self
            .generate_completion(&prompt, request.model.as_deref(), &parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from Local AI".to_string()))?;
        
        let summary = &choice.text;
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
            estimated_cost: 0.0,
        };
        
        // Extract key points (simplified)
        let key_points: Vec<String> = summary
            .split('\n')
            .filter(|line| !line.trim().is_empty())
            .take(5)
            .map(|s| s.trim().to_string())
            .collect();
        
        let compression_ratio = summary.len() as f32 / request.text.len() as f32;
        
        Ok(TextSummarizationResult {
            summary: summary.clone(),
            key_points,
            compression_ratio,
            usage,
        })
    }
    
    async fn extract_entities(&self, request: &EntityExtractionRequest) -> AIResult<EntityExtractionResult> {
        let entity_types_str = request
            .entity_types
            .iter()
            .map(|et| format!("{:?}", et))
            .collect::<Vec<_>>()
            .join(", ");
        
        let prompt = format!(
            "Extract entities of the following types from the text: {}\n\nText: {}\n\nEntities (one per line with type):",
            entity_types_str, request.text
        );
        
        let parameters = AIParameters {
            max_tokens: Some(500),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        let response = self
            .generate_completion(&prompt, request.model.as_deref(), &parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from Local AI".to_string()))?;
        
        // Parse simple text response (not JSON for local models)
        let entities: Vec<ExtractedEntity> = choice
            .text
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let text = parts[0].trim().to_string();
                    let entity_type = match parts[1].trim().to_lowercase().as_str() {
                        "person" => EntityType::Person,
                        "organization" => EntityType::Organization,
                        "location" => EntityType::Location,
                        "date" => EntityType::Date,
                        "money" => EntityType::Money,
                        "email" => EntityType::Email,
                        "phone" => EntityType::Phone,
                        "url" => EntityType::Url,
                        other => EntityType::Custom(other.to_string()),
                    };
                    
                    // Find position in original text (simplified)
                    let start_position = request.text.find(&text).unwrap_or(0);
                    let end_position = start_position + text.len();
                    
                    Some(ExtractedEntity {
                        text,
                        entity_type,
                        confidence: 0.7, // Lower confidence for local models
                        start_position,
                        end_position,
                        metadata: HashMap::new(),
                    })
                } else {
                    None
                }
            })
            .collect();
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.prompt_tokens,
            completion_tokens: response.usage.completion_tokens,
            total_tokens: response.usage.total_tokens,
            estimated_cost: 0.0,
        };
        
        Ok(EntityExtractionResult {
            entities,
            usage,
        })
    }
    
    async fn health_check(&self) -> AIResult<ProviderHealth> {
        let start_time = std::time::Instant::now();
        
        let parameters = AIParameters {
            max_tokens: Some(5),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        match self.generate_completion("Hello", None, &parameters).await {
            Ok(_) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                Ok(ProviderHealth {
                    status: HealthStatus::Healthy,
                    response_time_ms: Some(response_time),
                    error_rate: 0.0,
                    last_error: None,
                    last_check: Utc::now(),
                })
            }
            Err(e) => Ok(ProviderHealth {
                status: HealthStatus::Unhealthy,
                response_time_ms: None,
                error_rate: 1.0,
                last_error: Some(e.to_string()),
                last_check: Utc::now(),
            }),
        }
    }
    
    fn get_supported_models(&self) -> Vec<String> {
        self.config.models.clone()
    }
    
    fn get_provider_type(&self) -> crate::types::AIProvider {
        crate::types::AIProvider::Local
    }
}