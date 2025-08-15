use crate::config::AnthropicConfig;
use crate::error::{AIError, AIResult};
use crate::providers::AIProvider;
use crate::types::*;
use async_trait::async_trait;
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<AnthropicMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
struct AnthropicMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicResponse {
    id: String,
    #[serde(rename = "type")]
    response_type: String,
    role: String,
    content: Vec<AnthropicContent>,
    model: String,
    stop_reason: Option<String>,
    stop_sequence: Option<String>,
    usage: AnthropicUsage,
}

#[derive(Debug, Deserialize)]
struct AnthropicContent {
    #[serde(rename = "type")]
    content_type: String,
    text: String,
}

#[derive(Debug, Deserialize)]
struct AnthropicUsage {
    input_tokens: u32,
    output_tokens: u32,
}

pub struct AnthropicProvider {
    client: Client,
    config: AnthropicConfig,
}

impl AnthropicProvider {
    pub fn new(config: &AnthropicConfig) -> Self {
        let client = Client::new();
        
        Self {
            client,
            config: config.clone(),
        }
    }
    
    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        // Anthropic pricing is typically different for input vs output tokens
        // For simplicity, using average cost per token
        let total_tokens = input_tokens + output_tokens;
        (total_tokens as f64) * 0.000003 // Approximate cost
    }
    
    async fn create_message(
        &self,
        messages: Vec<AnthropicMessage>,
        model: Option<&str>,
        parameters: &AIParameters,
    ) -> AIResult<AnthropicResponse> {
        let model = model.unwrap_or(&self.config.default_model);
        let base_url = self.config.base_url.as_deref().unwrap_or("https://api.anthropic.com");
        
        let request = AnthropicRequest {
            model: model.to_string(),
            max_tokens: parameters.max_tokens.unwrap_or(self.config.max_tokens),
            messages,
            temperature: parameters.temperature,
            top_p: parameters.top_p,
            stop_sequences: parameters.stop_sequences.clone(),
        };
        
        let response = self
            .client
            .post(&format!("{}/v1/messages", base_url))
            .header("Content-Type", "application/json")
            .header("x-api-key", &self.config.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| AIError::HttpClient(e))?;
        
        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(AIError::AIProvider(format!("Anthropic API error: {}", error_text)));
        }
        
        response
            .json::<AnthropicResponse>()
            .await
            .map_err(|e| AIError::AIProvider(format!("Failed to parse Anthropic response: {}", e)))
    }
}

#[async_trait]
impl AIProvider for AnthropicProvider {
    async fn generate_text(&self, request: &TextGenerationRequest) -> AIResult<TextGenerationResult> {
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: request.prompt.clone(),
        }];
        
        let response = self
            .create_message(messages, request.model.as_deref(), &request.parameters)
            .await?;
        
        let content = response
            .content
            .first()
            .ok_or_else(|| AIError::AIProvider("No content in Anthropic response".to_string()))?;
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            estimated_cost: self.calculate_cost(response.usage.input_tokens, response.usage.output_tokens),
        };
        
        Ok(TextGenerationResult {
            generated_text: content.text.clone(),
            usage,
            quality_score: None,
            metadata: HashMap::new(),
        })
    }
    
    async fn classify_text(&self, request: &TextClassificationRequest) -> AIResult<TextClassificationResult> {
        let prompt = format!(
            "Classify the following text into one of these categories: {}\n\nText: {}\n\nRespond with only the category name.",
            request.categories.join(", "),
            request.text
        );
        
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt,
        }];
        
        let parameters = AIParameters {
            max_tokens: Some(50),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        let response = self
            .create_message(messages, request.model.as_deref(), &parameters)
            .await?;
        
        let content = response
            .content
            .first()
            .ok_or_else(|| AIError::AIProvider("No content in Anthropic response".to_string()))?;
        
        let result_text = content.text.trim();
        
        // Find the best matching category
        let category = request
            .categories
            .iter()
            .find(|cat| result_text.to_lowercase().contains(&cat.to_lowercase()))
            .unwrap_or(&request.categories[0])
            .clone();
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            estimated_cost: self.calculate_cost(response.usage.input_tokens, response.usage.output_tokens),
        };
        
        // Create confidence scores (simplified)
        let mut all_scores = HashMap::new();
        for cat in &request.categories {
            let score = if cat == &category { 0.9 } else { 0.1 };
            all_scores.insert(cat.clone(), score);
        }
        
        Ok(TextClassificationResult {
            category,
            confidence: 0.9,
            all_scores,
            usage,
        })
    }
    
    async fn summarize_text(&self, request: &TextSummarizationRequest) -> AIResult<TextSummarizationResult> {
        let style_instruction = match request.style.as_ref().unwrap_or(&SummarizationStyle::Abstractive) {
            SummarizationStyle::Extractive => "Extract the most important sentences",
            SummarizationStyle::Abstractive => "Create a concise summary in your own words",
            SummarizationStyle::Bullet => "Create a bullet-point summary",
            SummarizationStyle::Executive => "Create an executive summary",
        };
        
        let max_length = request.max_length.unwrap_or(200);
        let prompt = format!(
            "{} of the following text in approximately {} words:\n\n{}",
            style_instruction, max_length, request.text
        );
        
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt,
        }];
        
        let parameters = AIParameters {
            max_tokens: Some(max_length * 2),
            temperature: Some(0.3),
            ..Default::default()
        };
        
        let response = self
            .create_message(messages, request.model.as_deref(), &parameters)
            .await?;
        
        let content = response
            .content
            .first()
            .ok_or_else(|| AIError::AIProvider("No content in Anthropic response".to_string()))?;
        
        let summary = &content.text;
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            estimated_cost: self.calculate_cost(response.usage.input_tokens, response.usage.output_tokens),
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
            "Extract entities of the following types from the text: {}\n\nText: {}\n\nReturn the entities in JSON format with fields: text, type, start_position, end_position, confidence",
            entity_types_str, request.text
        );
        
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: prompt,
        }];
        
        let parameters = AIParameters {
            max_tokens: Some(1000),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        let response = self
            .create_message(messages, request.model.as_deref(), &parameters)
            .await?;
        
        let content = response
            .content
            .first()
            .ok_or_else(|| AIError::AIProvider("No content in Anthropic response".to_string()))?;
        
        // Parse JSON response (simplified)
        let entities: Vec<ExtractedEntity> = serde_json::from_str(&content.text)
            .unwrap_or_else(|_| Vec::new());
        
        let usage = TokenUsage {
            prompt_tokens: response.usage.input_tokens,
            completion_tokens: response.usage.output_tokens,
            total_tokens: response.usage.input_tokens + response.usage.output_tokens,
            estimated_cost: self.calculate_cost(response.usage.input_tokens, response.usage.output_tokens),
        };
        
        Ok(EntityExtractionResult {
            entities,
            usage,
        })
    }
    
    async fn health_check(&self) -> AIResult<ProviderHealth> {
        let start_time = std::time::Instant::now();
        
        let messages = vec![AnthropicMessage {
            role: "user".to_string(),
            content: "Hello".to_string(),
        }];
        
        let parameters = AIParameters {
            max_tokens: Some(5),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        match self.create_message(messages, None, &parameters).await {
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
        vec![
            "claude-3-haiku-20240307".to_string(),
            "claude-3-sonnet-20240229".to_string(),
            "claude-3-opus-20240229".to_string(),
        ]
    }
    
    fn get_provider_type(&self) -> crate::types::AIProvider {
        crate::types::AIProvider::Anthropic
    }
}