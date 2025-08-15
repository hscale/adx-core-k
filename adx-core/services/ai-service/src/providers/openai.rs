use crate::config::OpenAIConfig;
use crate::error::{AIError, AIResult};
use crate::providers::AIProvider;
use crate::types::*;
use async_openai::{
    types::{
        ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
        ChatCompletionRequestUserMessage, CreateChatCompletionRequest,
    },
    Client,
};
use async_trait::async_trait;
use chrono::Utc;
use std::collections::HashMap;
use tiktoken_rs::tiktoken::{get_bpe_from_model, CoreBPE};

pub struct OpenAIProvider {
    client: Client<async_openai::config::OpenAIConfig>,
    config: OpenAIConfig,
    tokenizer: Option<CoreBPE>,
}

impl OpenAIProvider {
    pub fn new(config: &OpenAIConfig) -> Self {
        let mut openai_config = async_openai::config::OpenAIConfig::new()
            .with_api_key(&config.api_key);
        
        if let Some(base_url) = &config.base_url {
            openai_config = openai_config.with_api_base(base_url);
        }
        
        let client = Client::with_config(openai_config);
        
        // Initialize tokenizer for the default model
        let tokenizer = get_bpe_from_model(&config.default_model).ok();
        
        Self {
            client,
            config: config.clone(),
            tokenizer,
        }
    }
    
    fn count_tokens(&self, text: &str) -> u32 {
        if let Some(tokenizer) = &self.tokenizer {
            tokenizer.encode_with_special_tokens(text).len() as u32
        } else {
            // Fallback estimation: ~4 characters per token
            (text.len() / 4) as u32
        }
    }
    
    fn calculate_cost(&self, prompt_tokens: u32, completion_tokens: u32) -> f64 {
        let total_tokens = prompt_tokens + completion_tokens;
        (total_tokens as f64) * self.config.cost_per_token
    }
    
    async fn create_chat_completion(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
        model: Option<&str>,
        parameters: &AIParameters,
    ) -> AIResult<async_openai::types::CreateChatCompletionResponse> {
        let model = model.unwrap_or(&self.config.default_model);
        
        let request = CreateChatCompletionRequest {
            model: model.to_string(),
            messages,
            max_tokens: parameters.max_tokens.or(Some(self.config.max_tokens)),
            temperature: parameters.temperature.or(Some(self.config.temperature)),
            top_p: parameters.top_p,
            frequency_penalty: parameters.frequency_penalty,
            presence_penalty: parameters.presence_penalty,
            stop: parameters.stop_sequences.clone(),
            ..Default::default()
        };
        
        self.client
            .chat()
            .create(request)
            .await
            .map_err(|e| AIError::AIProvider(format!("OpenAI API error: {}", e)))
    }
}

#[async_trait]
impl AIProvider for OpenAIProvider {
    async fn generate_text(&self, request: &TextGenerationRequest) -> AIResult<TextGenerationResult> {
        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    request.prompt.clone(),
                ),
                name: None,
            },
        )];
        
        let response = self
            .create_chat_completion(messages, request.model.as_deref(), &request.parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from OpenAI".to_string()))?;
        
        let content = choice
            .message
            .content
            .as_ref()
            .ok_or_else(|| AIError::AIProvider("Empty response from OpenAI".to_string()))?;
        
        let usage = response.usage.unwrap_or_default();
        let prompt_tokens = usage.prompt_tokens.unwrap_or(0) as u32;
        let completion_tokens = usage.completion_tokens.unwrap_or(0) as u32;
        let total_tokens = usage.total_tokens.unwrap_or(0) as u32;
        
        Ok(TextGenerationResult {
            generated_text: content.clone(),
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
                estimated_cost: self.calculate_cost(prompt_tokens, completion_tokens),
            },
            quality_score: None, // Could be implemented with additional analysis
            metadata: HashMap::new(),
        })
    }
    
    async fn classify_text(&self, request: &TextClassificationRequest) -> AIResult<TextClassificationResult> {
        let prompt = format!(
            "Classify the following text into one of these categories: {}\n\nText: {}\n\nCategory:",
            request.categories.join(", "),
            request.text
        );
        
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: "You are a text classifier. Respond with only the category name.".to_string(),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(prompt),
                name: None,
            }),
        ];
        
        let parameters = AIParameters {
            max_tokens: Some(50),
            temperature: Some(0.0), // Low temperature for consistent classification
            ..Default::default()
        };
        
        let response = self
            .create_chat_completion(messages, request.model.as_deref(), &parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from OpenAI".to_string()))?;
        
        let content = choice
            .message
            .content
            .as_ref()
            .ok_or_else(|| AIError::AIProvider("Empty response from OpenAI".to_string()))?
            .trim();
        
        // Find the best matching category
        let category = request
            .categories
            .iter()
            .find(|cat| content.to_lowercase().contains(&cat.to_lowercase()))
            .unwrap_or(&request.categories[0])
            .clone();
        
        let usage = response.usage.unwrap_or_default();
        let prompt_tokens = usage.prompt_tokens.unwrap_or(0) as u32;
        let completion_tokens = usage.completion_tokens.unwrap_or(0) as u32;
        let total_tokens = usage.total_tokens.unwrap_or(0) as u32;
        
        // Create confidence scores (simplified)
        let mut all_scores = HashMap::new();
        for cat in &request.categories {
            let score = if cat == &category { 0.9 } else { 0.1 };
            all_scores.insert(cat.clone(), score);
        }
        
        Ok(TextClassificationResult {
            category,
            confidence: 0.9, // Simplified confidence
            all_scores,
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
                estimated_cost: self.calculate_cost(prompt_tokens, completion_tokens),
            },
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
        
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: "You are a professional summarizer. Provide clear, concise summaries.".to_string(),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(prompt),
                name: None,
            }),
        ];
        
        let parameters = AIParameters {
            max_tokens: Some(max_length * 2), // Allow some buffer
            temperature: Some(0.3),
            ..Default::default()
        };
        
        let response = self
            .create_chat_completion(messages, request.model.as_deref(), &parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from OpenAI".to_string()))?;
        
        let summary = choice
            .message
            .content
            .as_ref()
            .ok_or_else(|| AIError::AIProvider("Empty response from OpenAI".to_string()))?;
        
        let usage = response.usage.unwrap_or_default();
        let prompt_tokens = usage.prompt_tokens.unwrap_or(0) as u32;
        let completion_tokens = usage.completion_tokens.unwrap_or(0) as u32;
        let total_tokens = usage.total_tokens.unwrap_or(0) as u32;
        
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
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
                estimated_cost: self.calculate_cost(prompt_tokens, completion_tokens),
            },
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
        
        let messages = vec![
            ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
                content: "You are an entity extraction system. Return valid JSON only.".to_string(),
                name: None,
            }),
            ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(prompt),
                name: None,
            }),
        ];
        
        let parameters = AIParameters {
            max_tokens: Some(1000),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        let response = self
            .create_chat_completion(messages, request.model.as_deref(), &parameters)
            .await?;
        
        let choice = response
            .choices
            .first()
            .ok_or_else(|| AIError::AIProvider("No response from OpenAI".to_string()))?;
        
        let content = choice
            .message
            .content
            .as_ref()
            .ok_or_else(|| AIError::AIProvider("Empty response from OpenAI".to_string()))?;
        
        // Parse JSON response (simplified - in production, would need better error handling)
        let entities: Vec<ExtractedEntity> = serde_json::from_str(content)
            .unwrap_or_else(|_| Vec::new());
        
        let usage = response.usage.unwrap_or_default();
        let prompt_tokens = usage.prompt_tokens.unwrap_or(0) as u32;
        let completion_tokens = usage.completion_tokens.unwrap_or(0) as u32;
        let total_tokens = usage.total_tokens.unwrap_or(0) as u32;
        
        Ok(EntityExtractionResult {
            entities,
            usage: TokenUsage {
                prompt_tokens,
                completion_tokens,
                total_tokens,
                estimated_cost: self.calculate_cost(prompt_tokens, completion_tokens),
            },
        })
    }
    
    async fn health_check(&self) -> AIResult<ProviderHealth> {
        let start_time = std::time::Instant::now();
        
        // Simple health check with a minimal request
        let messages = vec![ChatCompletionRequestMessage::User(
            ChatCompletionRequestUserMessage {
                content: async_openai::types::ChatCompletionRequestUserMessageContent::Text(
                    "Hello".to_string(),
                ),
                name: None,
            },
        )];
        
        let parameters = AIParameters {
            max_tokens: Some(5),
            temperature: Some(0.0),
            ..Default::default()
        };
        
        match self.create_chat_completion(messages, None, &parameters).await {
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
            "gpt-3.5-turbo".to_string(),
            "gpt-4".to_string(),
            "gpt-4-turbo".to_string(),
            "gpt-4o".to_string(),
        ]
    }
    
    fn get_provider_type(&self) -> crate::types::AIProvider {
        crate::types::AIProvider::OpenAI
    }
}

impl Default for AIParameters {
    fn default() -> Self {
        Self {
            max_tokens: None,
            temperature: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop_sequences: None,
        }
    }
}