use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// AI Model and Provider Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIProvider {
    OpenAI,
    Anthropic,
    Local,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIModel {
    pub id: String,
    pub name: String,
    pub provider: AIProvider,
    pub capabilities: Vec<AICapability>,
    pub max_tokens: u32,
    pub cost_per_token: f64,
    pub tier_availability: Vec<SubscriptionTier>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AICapability {
    TextGeneration,
    TextClassification,
    TextSummarization,
    EntityExtraction,
    SentimentAnalysis,
    LanguageTranslation,
    CodeGeneration,
    ImageGeneration,
    ImageAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubscriptionTier {
    Free,
    Professional,
    Enterprise,
}

// AI Request and Response Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIRequest {
    pub model: String,
    pub prompt: String,
    pub parameters: AIParameters,
    pub context: RequestContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIParameters {
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
    pub top_p: Option<f32>,
    pub frequency_penalty: Option<f32>,
    pub presence_penalty: Option<f32>,
    pub stop_sequences: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub tenant_id: String,
    pub user_id: String,
    pub session_id: Option<String>,
    pub workflow_id: Option<String>,
    pub activity_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIResponse {
    pub id: String,
    pub content: String,
    pub model: String,
    pub usage: TokenUsage,
    pub finish_reason: FinishReason,
    pub created_at: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
    pub estimated_cost: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FinishReason {
    Stop,
    Length,
    ContentFilter,
    ToolCalls,
    Error,
}

// Workflow-specific Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIWorkflowRequest {
    pub workflow_type: AIWorkflowType,
    pub input_data: serde_json::Value,
    pub parameters: AIWorkflowParameters,
    pub context: RequestContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIWorkflowType {
    UserOnboarding,
    DocumentProcessing,
    EmailGeneration,
    ContentModeration,
    DataAnalysis,
    CustomWorkflow(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIWorkflowParameters {
    pub model_preferences: HashMap<AICapability, String>,
    pub quality_threshold: f32,
    pub max_retries: u32,
    pub timeout_seconds: u64,
    pub custom_parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIWorkflowResult {
    pub workflow_id: String,
    pub results: HashMap<String, serde_json::Value>,
    pub total_usage: TokenUsage,
    pub execution_time_ms: u64,
    pub quality_score: Option<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

// Activity-specific Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGenerationRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub parameters: AIParameters,
    pub context: RequestContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextGenerationResult {
    pub generated_text: String,
    pub usage: TokenUsage,
    pub quality_score: Option<f32>,
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextClassificationRequest {
    pub text: String,
    pub categories: Vec<String>,
    pub model: Option<String>,
    pub context: RequestContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextClassificationResult {
    pub category: String,
    pub confidence: f32,
    pub all_scores: HashMap<String, f32>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSummarizationRequest {
    pub text: String,
    pub max_length: Option<u32>,
    pub style: Option<SummarizationStyle>,
    pub model: Option<String>,
    pub context: RequestContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SummarizationStyle {
    Extractive,
    Abstractive,
    Bullet,
    Executive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextSummarizationResult {
    pub summary: String,
    pub key_points: Vec<String>,
    pub compression_ratio: f32,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractionRequest {
    pub text: String,
    pub entity_types: Vec<EntityType>,
    pub model: Option<String>,
    pub context: RequestContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityType {
    Person,
    Organization,
    Location,
    Date,
    Money,
    Email,
    Phone,
    Url,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityExtractionResult {
    pub entities: Vec<ExtractedEntity>,
    pub usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedEntity {
    pub text: String,
    pub entity_type: EntityType,
    pub confidence: f32,
    pub start_position: usize,
    pub end_position: usize,
    pub metadata: HashMap<String, serde_json::Value>,
}

// Usage Tracking and Monitoring Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUsageRecord {
    pub id: Uuid,
    pub tenant_id: String,
    pub user_id: String,
    pub workflow_id: Option<String>,
    pub activity_id: Option<String>,
    pub model: String,
    pub capability: AICapability,
    pub usage: TokenUsage,
    pub request_timestamp: DateTime<Utc>,
    pub response_timestamp: DateTime<Utc>,
    pub success: bool,
    pub error_code: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIUsageStats {
    pub tenant_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens: u64,
    pub total_cost: f64,
    pub usage_by_model: HashMap<String, ModelUsageStats>,
    pub usage_by_capability: HashMap<AICapability, CapabilityUsageStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsageStats {
    pub requests: u64,
    pub tokens: u64,
    pub cost: f64,
    pub avg_response_time_ms: f64,
    pub success_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityUsageStats {
    pub requests: u64,
    pub tokens: u64,
    pub cost: f64,
    pub avg_quality_score: Option<f32>,
}

// Health Check Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIServiceHealth {
    pub status: HealthStatus,
    pub providers: HashMap<AIProvider, ProviderHealth>,
    pub models: HashMap<String, ModelHealth>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderHealth {
    pub status: HealthStatus,
    pub response_time_ms: Option<u64>,
    pub error_rate: f32,
    pub last_error: Option<String>,
    pub last_check: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelHealth {
    pub status: HealthStatus,
    pub availability: f32,
    pub avg_response_time_ms: f64,
    pub error_rate: f32,
    pub last_check: DateTime<Utc>,
}