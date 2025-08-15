use crate::error::{AIError, AIResult};
use crate::services::{AIService, HealthMonitor, UsageTracker};
use crate::types::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
// use shared::middleware::TenantContext; // Commented out until shared crate is available

// Temporary TenantContext for compilation
#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub user_id: String,
}
use std::collections::HashMap;
use std::sync::Arc;

pub type AppState = Arc<AppStateInner>;

pub struct AppStateInner {
    pub ai_service: Arc<AIService>,
    pub usage_tracker: Arc<UsageTracker>,
    pub health_monitor: Arc<HealthMonitor>,
}

// Health check endpoint
pub async fn health_check(
    State(state): State<AppState>,
) -> Result<Json<AIServiceHealth>, AIError> {
    let health = state.health_monitor.get_current_health().await?;
    Ok(Json(health))
}

// Get available models for tenant
pub async fn get_models(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
) -> Result<Json<Vec<AIModel>>, AIError> {
    let tenant_tier = SubscriptionTier::Professional; // Would normally be from tenant_context
    let models = state.ai_service.get_available_models(&tenant_tier).await?;
    Ok(Json(models))
}

// Get models for specific capability
#[derive(Debug, Deserialize)]
pub struct GetModelsQuery {
    capability: Option<String>,
}

pub async fn get_models_for_capability(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(query): Query<GetModelsQuery>,
) -> Result<Json<Vec<AIModel>>, AIError> {
    let tenant_tier = SubscriptionTier::Professional; // Would normally be from tenant_context
    
    if let Some(capability_str) = query.capability {
        let capability: AICapability = serde_json::from_str(&format!("\"{}\"", capability_str))
            .map_err(|_| AIError::BadRequest("Invalid capability".to_string()))?;
        
        let models = state.ai_service.get_models_for_capability(&capability, &tenant_tier).await?;
        Ok(Json(models))
    } else {
        let models = state.ai_service.get_available_models(&tenant_tier).await?;
        Ok(Json(models))
    }
}

// Generate text endpoint
#[derive(Debug, Deserialize)]
pub struct GenerateTextRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub parameters: Option<AIParameters>,
}

#[derive(Debug, Serialize)]
pub struct GenerateTextResponse {
    pub id: String,
    pub generated_text: String,
    pub model: String,
    pub usage: TokenUsage,
    pub created_at: DateTime<Utc>,
}

pub async fn generate_text(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<GenerateTextRequest>,
) -> Result<Json<GenerateTextResponse>, AIError> {
    let context = RequestContext {
        tenant_id: tenant_context.tenant_id.clone(),
        user_id: tenant_context.user_id.clone(),
        session_id: None,
        workflow_id: None,
        activity_id: None,
    };
    
    let ai_request = state.ai_service.create_ai_request(
        request.prompt,
        request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string()),
        request.parameters.unwrap_or_default(),
        context,
    ).await?;
    
    let response = state.ai_service.process_ai_request(ai_request).await?;
    
    Ok(Json(GenerateTextResponse {
        id: response.id,
        generated_text: response.content,
        model: response.model,
        usage: response.usage,
        created_at: response.created_at,
    }))
}

// Classify text endpoint
#[derive(Debug, Deserialize)]
pub struct ClassifyTextRequest {
    pub text: String,
    pub categories: Vec<String>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ClassifyTextResponse {
    pub category: String,
    pub confidence: f32,
    pub all_scores: HashMap<String, f32>,
    pub usage: TokenUsage,
}

pub async fn classify_text(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<ClassifyTextRequest>,
) -> Result<Json<ClassifyTextResponse>, AIError> {
    // This would normally use the activities through a workflow
    // For direct endpoint, we'll create a simplified version
    let model_registry = state.ai_service.get_model_registry();
    let provider_manager = state.ai_service.get_provider_manager();
    
    let model = request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    let model_info = model_registry.get_model(&model)
        .ok_or_else(|| AIError::ModelNotAvailable(format!("Model {} not found", model)))?;
    
    let provider = provider_manager.get_provider(&model_info.provider)?;
    
    let classification_request = TextClassificationRequest {
        text: request.text,
        categories: request.categories,
        model: Some(model),
        context: RequestContext {
            tenant_id: tenant_context.tenant_id.clone(),
            user_id: tenant_context.user_id.clone(),
            session_id: None,
            workflow_id: None,
            activity_id: None,
        },
    };
    
    let result = provider.classify_text(&classification_request).await
        .map_err(|e| AIError::AIProvider(e.to_string()))?;
    
    Ok(Json(ClassifyTextResponse {
        category: result.category,
        confidence: result.confidence,
        all_scores: result.all_scores,
        usage: result.usage,
    }))
}

// Summarize text endpoint
#[derive(Debug, Deserialize)]
pub struct SummarizeTextRequest {
    pub text: String,
    pub max_length: Option<u32>,
    pub style: Option<SummarizationStyle>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SummarizeTextResponse {
    pub summary: String,
    pub key_points: Vec<String>,
    pub compression_ratio: f32,
    pub usage: TokenUsage,
}

pub async fn summarize_text(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<SummarizeTextRequest>,
) -> Result<Json<SummarizeTextResponse>, AIError> {
    let model_registry = state.ai_service.get_model_registry();
    let provider_manager = state.ai_service.get_provider_manager();
    
    let model = request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    let model_info = model_registry.get_model(&model)
        .ok_or_else(|| AIError::ModelNotAvailable(format!("Model {} not found", model)))?;
    
    let provider = provider_manager.get_provider(&model_info.provider)?;
    
    let summarization_request = TextSummarizationRequest {
        text: request.text,
        max_length: request.max_length,
        style: request.style,
        model: Some(model),
        context: RequestContext {
            tenant_id: tenant_context.tenant_id.clone(),
            user_id: tenant_context.user_id.clone(),
            session_id: None,
            workflow_id: None,
            activity_id: None,
        },
    };
    
    let result = provider.summarize_text(&summarization_request).await
        .map_err(|e| AIError::AIProvider(e.to_string()))?;
    
    Ok(Json(SummarizeTextResponse {
        summary: result.summary,
        key_points: result.key_points,
        compression_ratio: result.compression_ratio,
        usage: result.usage,
    }))
}

// Extract entities endpoint
#[derive(Debug, Deserialize)]
pub struct ExtractEntitiesRequest {
    pub text: String,
    pub entity_types: Vec<EntityType>,
    pub model: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExtractEntitiesResponse {
    pub entities: Vec<ExtractedEntity>,
    pub usage: TokenUsage,
}

pub async fn extract_entities(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<ExtractEntitiesRequest>,
) -> Result<Json<ExtractEntitiesResponse>, AIError> {
    let model_registry = state.ai_service.get_model_registry();
    let provider_manager = state.ai_service.get_provider_manager();
    
    let model = request.model.unwrap_or_else(|| "gpt-3.5-turbo".to_string());
    let model_info = model_registry.get_model(&model)
        .ok_or_else(|| AIError::ModelNotAvailable(format!("Model {} not found", model)))?;
    
    let provider = provider_manager.get_provider(&model_info.provider)?;
    
    let extraction_request = EntityExtractionRequest {
        text: request.text,
        entity_types: request.entity_types,
        model: Some(model),
        context: RequestContext {
            tenant_id: tenant_context.tenant_id.clone(),
            user_id: tenant_context.user_id.clone(),
            session_id: None,
            workflow_id: None,
            activity_id: None,
        },
    };
    
    let result = provider.extract_entities(&extraction_request).await
        .map_err(|e| AIError::AIProvider(e.to_string()))?;
    
    Ok(Json(ExtractEntitiesResponse {
        entities: result.entities,
        usage: result.usage,
    }))
}

// Usage statistics endpoint
#[derive(Debug, Deserialize)]
pub struct UsageStatsQuery {
    pub period_start: Option<DateTime<Utc>>,
    pub period_end: Option<DateTime<Utc>>,
}

pub async fn get_usage_stats(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(query): Query<UsageStatsQuery>,
) -> Result<Json<AIUsageStats>, AIError> {
    let period_end = query.period_end.unwrap_or_else(Utc::now);
    let period_start = query.period_start.unwrap_or_else(|| period_end - chrono::Duration::days(30));
    
    let stats = state.usage_tracker.get_usage_stats(
        &tenant_context.tenant_id,
        period_start,
        period_end,
    ).await?;
    
    Ok(Json(stats))
}

// Cost breakdown endpoint
pub async fn get_cost_breakdown(
    State(state): State<AppState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(query): Query<UsageStatsQuery>,
) -> Result<Json<HashMap<String, f64>>, AIError> {
    let period_end = query.period_end.unwrap_or_else(Utc::now);
    let period_start = query.period_start.unwrap_or_else(|| period_end - chrono::Duration::days(30));
    
    let cost_breakdown = state.usage_tracker.get_cost_breakdown(
        &tenant_context.tenant_id,
        period_start,
        period_end,
    ).await?;
    
    Ok(Json(cost_breakdown))
}

// Provider health endpoint
pub async fn get_provider_health(
    State(state): State<AppState>,
    Path(provider): Path<String>,
) -> Result<Json<ProviderHealth>, AIError> {
    let provider_type: AIProvider = match provider.to_lowercase().as_str() {
        "openai" => AIProvider::OpenAI,
        "anthropic" => AIProvider::Anthropic,
        "local" => AIProvider::Local,
        _ => return Err(AIError::BadRequest("Invalid provider".to_string())),
    };
    
    let health = state.health_monitor.get_current_health().await?;
    
    if let Some(provider_health) = health.providers.get(&provider_type) {
        Ok(Json(provider_health.clone()))
    } else {
        Err(AIError::NotFound("Provider not found".to_string()))
    }
}

// Health history endpoint
#[derive(Debug, Deserialize)]
pub struct HealthHistoryQuery {
    pub hours: Option<u32>,
}

pub async fn get_health_history(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<HealthHistoryQuery>,
) -> Result<Json<Vec<crate::services::health_monitor::HealthCheckResult>>, AIError> {
    let provider_type: AIProvider = match provider.to_lowercase().as_str() {
        "openai" => AIProvider::OpenAI,
        "anthropic" => AIProvider::Anthropic,
        "local" => AIProvider::Local,
        _ => return Err(AIError::BadRequest("Invalid provider".to_string())),
    };
    
    let hours = query.hours.unwrap_or(24);
    let history = state.health_monitor.get_health_history(Some(provider_type), hours).await?;
    
    Ok(Json(history))
}

// Availability metrics endpoint
pub async fn get_availability_metrics(
    State(state): State<AppState>,
    Path(provider): Path<String>,
    Query(query): Query<HealthHistoryQuery>,
) -> Result<Json<crate::services::health_monitor::AvailabilityMetrics>, AIError> {
    let provider_type: AIProvider = match provider.to_lowercase().as_str() {
        "openai" => AIProvider::OpenAI,
        "anthropic" => AIProvider::Anthropic,
        "local" => AIProvider::Local,
        _ => return Err(AIError::BadRequest("Invalid provider".to_string())),
    };
    
    let hours = query.hours.unwrap_or(24);
    let metrics = state.health_monitor.get_availability_metrics(provider_type, hours).await?;
    
    Ok(Json(metrics))
}

// Alert conditions endpoint
pub async fn get_alert_conditions(
    State(state): State<AppState>,
) -> Result<Json<Vec<crate::services::health_monitor::AlertCondition>>, AIError> {
    let alerts = state.health_monitor.get_alert_conditions().await?;
    Ok(Json(alerts))
}