use crate::activities::{AIActivities, ValidationResult};
use crate::error::ActivityError;
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::temporal_stubs::{WfContext, WorkflowResult, workflow};

// User Onboarding AI Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingAIRequest {
    pub user_id: String,
    pub tenant_id: String,
    pub user_profile: UserProfile,
    pub onboarding_preferences: OnboardingPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub name: String,
    pub email: String,
    pub role: String,
    pub department: Option<String>,
    pub experience_level: ExperienceLevel,
    pub interests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExperienceLevel {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingPreferences {
    pub communication_style: CommunicationStyle,
    pub learning_pace: LearningPace,
    pub preferred_features: Vec<String>,
    pub goals: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommunicationStyle {
    Formal,
    Casual,
    Technical,
    Simple,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningPace {
    Fast,
    Medium,
    Slow,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingAIResult {
    pub personalized_welcome: String,
    pub recommended_features: Vec<String>,
    pub learning_path: Vec<LearningStep>,
    pub initial_setup_tasks: Vec<SetupTask>,
    pub ai_usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningStep {
    pub title: String,
    pub description: String,
    pub estimated_time_minutes: u32,
    pub difficulty: String,
    pub resources: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupTask {
    pub title: String,
    pub description: String,
    pub priority: TaskPriority,
    pub estimated_time_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskPriority {
    High,
    Medium,
    Low,
}

pub async fn user_onboarding_ai_workflow(
    ctx: WfContext,
    request: UserOnboardingAIRequest,
) -> WorkflowResult<UserOnboardingAIResult> {
    let activities = ctx.activity(());
    
    // Step 1: Generate personalized welcome message
    let welcome_prompt = format!(
        "Create a personalized welcome message for a new user named {} with role {} in department {:?}. 
        Their experience level is {:?} and they're interested in: {}. 
        Communication style should be {:?}. Keep it warm, professional, and encouraging.",
        request.user_profile.name,
        request.user_profile.role,
        request.user_profile.department,
        request.user_profile.experience_level,
        request.user_profile.interests.join(", "),
        request.onboarding_preferences.communication_style
    );
    
    let welcome_request = TextGenerationRequest {
        prompt: welcome_prompt,
        model: None, // Let the system choose the best model
        parameters: AIParameters {
            max_tokens: Some(300),
            temperature: Some(0.7),
            ..Default::default()
        },
        context: RequestContext {
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            workflow_id: Some(ctx.workflow_info().workflow_id.clone()),
            activity_id: Some("generate_welcome".to_string()),
            session_id: None,
        },
    };
    
    let welcome_result = activities.generate_text(welcome_request).await?;
    
    // Step 2: Generate feature recommendations based on user profile
    let features_prompt = format!(
        "Based on a user's profile - Role: {}, Experience: {:?}, Interests: {}, Goals: {} - 
        recommend the top 5 most relevant features from our platform. 
        Return as a JSON array of feature names with brief explanations.",
        request.user_profile.role,
        request.user_profile.experience_level,
        request.user_profile.interests.join(", "),
        request.onboarding_preferences.goals.join(", ")
    );
    
    let features_request = TextGenerationRequest {
        prompt: features_prompt,
        model: None,
        parameters: AIParameters {
            max_tokens: Some(400),
            temperature: Some(0.5),
            ..Default::default()
        },
        context: RequestContext {
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            workflow_id: Some(ctx.workflow_info().workflow_id.clone()),
            activity_id: Some("recommend_features".to_string()),
            session_id: None,
        },
    };
    
    let features_result = activities.generate_text(features_request).await?;
    
    // Parse feature recommendations (simplified)
    let recommended_features: Vec<String> = features_result.generated_text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .take(5)
        .map(|s| s.trim().to_string())
        .collect();
    
    // Step 3: Create personalized learning path
    let learning_prompt = format!(
        "Create a personalized learning path for a {} user with {:?} experience level. 
        Their learning pace preference is {:?} and they want to achieve: {}. 
        Create 4-6 learning steps with titles, descriptions, estimated time, and difficulty level.",
        request.user_profile.role,
        request.user_profile.experience_level,
        request.onboarding_preferences.learning_pace,
        request.onboarding_preferences.goals.join(", ")
    );
    
    let learning_request = TextGenerationRequest {
        prompt: learning_prompt,
        model: None,
        parameters: AIParameters {
            max_tokens: Some(600),
            temperature: Some(0.6),
            ..Default::default()
        },
        context: RequestContext {
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            workflow_id: Some(ctx.workflow_info().workflow_id.clone()),
            activity_id: Some("create_learning_path".to_string()),
            session_id: None,
        },
    };
    
    let learning_result = activities.generate_text(learning_request).await?;
    
    // Parse learning path (simplified)
    let learning_path = parse_learning_path(&learning_result.generated_text);
    
    // Step 4: Generate initial setup tasks
    let setup_prompt = format!(
        "Create a prioritized list of initial setup tasks for a new {} user. 
        Consider their experience level ({:?}) and preferred features: {}. 
        Include task titles, descriptions, priorities (High/Medium/Low), and estimated time.",
        request.user_profile.role,
        request.user_profile.experience_level,
        request.onboarding_preferences.preferred_features.join(", ")
    );
    
    let setup_request = TextGenerationRequest {
        prompt: setup_prompt,
        model: None,
        parameters: AIParameters {
            max_tokens: Some(500),
            temperature: Some(0.4),
            ..Default::default()
        },
        context: RequestContext {
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            workflow_id: Some(ctx.workflow_info().workflow_id.clone()),
            activity_id: Some("create_setup_tasks".to_string()),
            session_id: None,
        },
    };
    
    let setup_result = activities.generate_text(setup_request).await?;
    
    // Parse setup tasks (simplified)
    let setup_tasks = parse_setup_tasks(&setup_result.generated_text);
    
    // Calculate total AI usage
    let total_usage = TokenUsage {
        prompt_tokens: welcome_result.usage.prompt_tokens + 
                      features_result.usage.prompt_tokens + 
                      learning_result.usage.prompt_tokens + 
                      setup_result.usage.prompt_tokens,
        completion_tokens: welcome_result.usage.completion_tokens + 
                          features_result.usage.completion_tokens + 
                          learning_result.usage.completion_tokens + 
                          setup_result.usage.completion_tokens,
        total_tokens: welcome_result.usage.total_tokens + 
                     features_result.usage.total_tokens + 
                     learning_result.usage.total_tokens + 
                     setup_result.usage.total_tokens,
        estimated_cost: welcome_result.usage.estimated_cost + 
                       features_result.usage.estimated_cost + 
                       learning_result.usage.estimated_cost + 
                       setup_result.usage.estimated_cost,
    };
    
    Ok(UserOnboardingAIResult {
        personalized_welcome: welcome_result.generated_text,
        recommended_features,
        learning_path,
        initial_setup_tasks: setup_tasks,
        ai_usage: total_usage,
    })
}

// Document Processing AI Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessingAIRequest {
    pub document_id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub document_content: String,
    pub document_type: DocumentType,
    pub processing_options: DocumentProcessingOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DocumentType {
    Contract,
    Report,
    Email,
    Proposal,
    Manual,
    Other(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessingOptions {
    pub extract_entities: bool,
    pub generate_summary: bool,
    pub classify_content: bool,
    pub extract_key_points: bool,
    pub sentiment_analysis: bool,
    pub custom_instructions: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentProcessingAIResult {
    pub document_id: String,
    pub summary: Option<String>,
    pub classification: Option<String>,
    pub entities: Vec<ExtractedEntity>,
    pub key_points: Vec<String>,
    pub sentiment: Option<SentimentResult>,
    pub metadata: HashMap<String, serde_json::Value>,
    pub ai_usage: TokenUsage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SentimentResult {
    pub sentiment: String,
    pub confidence: f32,
    pub emotions: HashMap<String, f32>,
}

pub async fn document_processing_ai_workflow(
    ctx: WfContext,
    request: DocumentProcessingAIRequest,
) -> WorkflowResult<DocumentProcessingAIResult> {
    let activities = ctx.activity(());
    let mut total_usage = TokenUsage {
        prompt_tokens: 0,
        completion_tokens: 0,
        total_tokens: 0,
        estimated_cost: 0.0,
    };
    
    let mut result = DocumentProcessingAIResult {
        document_id: request.document_id.clone(),
        summary: None,
        classification: None,
        entities: Vec::new(),
        key_points: Vec::new(),
        sentiment: None,
        metadata: HashMap::new(),
        ai_usage: total_usage.clone(),
    };
    
    let context = RequestContext {
        tenant_id: request.tenant_id.clone(),
        user_id: request.user_id.clone(),
        workflow_id: Some(ctx.workflow_info().workflow_id.clone()),
        activity_id: None,
        session_id: None,
    };
    
    // Step 1: Generate summary if requested
    if request.processing_options.generate_summary {
        let summary_request = TextSummarizationRequest {
            text: request.document_content.clone(),
            max_length: Some(300),
            style: Some(match request.document_type {
                DocumentType::Contract => SummarizationStyle::Executive,
                DocumentType::Report => SummarizationStyle::Executive,
                DocumentType::Email => SummarizationStyle::Bullet,
                _ => SummarizationStyle::Abstractive,
            }),
            model: None,
            context: RequestContext {
                activity_id: Some("summarize_document".to_string()),
                ..context.clone()
            },
        };
        
        let summary_result = activities.summarize_text(summary_request).await?;
        result.summary = Some(summary_result.summary);
        result.key_points = summary_result.key_points;
        
        total_usage.prompt_tokens += summary_result.usage.prompt_tokens;
        total_usage.completion_tokens += summary_result.usage.completion_tokens;
        total_usage.total_tokens += summary_result.usage.total_tokens;
        total_usage.estimated_cost += summary_result.usage.estimated_cost;
    }
    
    // Step 2: Classify document if requested
    if request.processing_options.classify_content {
        let categories = match request.document_type {
            DocumentType::Contract => vec![
                "Service Agreement".to_string(),
                "Employment Contract".to_string(),
                "NDA".to_string(),
                "Purchase Agreement".to_string(),
                "License Agreement".to_string(),
            ],
            DocumentType::Email => vec![
                "Business".to_string(),
                "Support".to_string(),
                "Marketing".to_string(),
                "Internal".to_string(),
                "Customer".to_string(),
            ],
            _ => vec![
                "Technical".to_string(),
                "Business".to_string(),
                "Legal".to_string(),
                "Financial".to_string(),
                "Marketing".to_string(),
            ],
        };
        
        let classification_request = TextClassificationRequest {
            text: request.document_content.clone(),
            categories,
            model: None,
            context: RequestContext {
                activity_id: Some("classify_document".to_string()),
                ..context.clone()
            },
        };
        
        let classification_result = activities.classify_text(classification_request).await?;
        result.classification = Some(classification_result.category);
        
        total_usage.prompt_tokens += classification_result.usage.prompt_tokens;
        total_usage.completion_tokens += classification_result.usage.completion_tokens;
        total_usage.total_tokens += classification_result.usage.total_tokens;
        total_usage.estimated_cost += classification_result.usage.estimated_cost;
    }
    
    // Step 3: Extract entities if requested
    if request.processing_options.extract_entities {
        let entity_types = vec![
            EntityType::Person,
            EntityType::Organization,
            EntityType::Location,
            EntityType::Date,
            EntityType::Money,
            EntityType::Email,
            EntityType::Phone,
        ];
        
        let entity_request = EntityExtractionRequest {
            text: request.document_content.clone(),
            entity_types,
            model: None,
            context: RequestContext {
                activity_id: Some("extract_entities".to_string()),
                ..context.clone()
            },
        };
        
        let entity_result = activities.extract_entities(entity_request).await?;
        result.entities = entity_result.entities;
        
        total_usage.prompt_tokens += entity_result.usage.prompt_tokens;
        total_usage.completion_tokens += entity_result.usage.completion_tokens;
        total_usage.total_tokens += entity_result.usage.total_tokens;
        total_usage.estimated_cost += entity_result.usage.estimated_cost;
    }
    
    // Step 4: Sentiment analysis if requested
    if request.processing_options.sentiment_analysis {
        let sentiment_prompt = format!(
            "Analyze the sentiment of the following {} document. 
            Provide the overall sentiment (Positive, Negative, Neutral) and confidence score.
            Also identify key emotions present (if any): joy, anger, fear, sadness, surprise, trust.
            
            Document: {}",
            format!("{:?}", request.document_type).to_lowercase(),
            request.document_content
        );
        
        let sentiment_request = TextGenerationRequest {
            prompt: sentiment_prompt,
            model: None,
            parameters: AIParameters {
                max_tokens: Some(200),
                temperature: Some(0.3),
                ..Default::default()
            },
            context: RequestContext {
                activity_id: Some("analyze_sentiment".to_string()),
                ..context.clone()
            },
        };
        
        let sentiment_result = activities.generate_text(sentiment_request).await?;
        
        // Parse sentiment result (simplified)
        let sentiment = parse_sentiment_result(&sentiment_result.generated_text);
        result.sentiment = Some(sentiment);
        
        total_usage.prompt_tokens += sentiment_result.usage.prompt_tokens;
        total_usage.completion_tokens += sentiment_result.usage.completion_tokens;
        total_usage.total_tokens += sentiment_result.usage.total_tokens;
        total_usage.estimated_cost += sentiment_result.usage.estimated_cost;
    }
    
    // Add metadata
    result.metadata.insert("document_type".to_string(), 
        serde_json::to_value(&request.document_type).unwrap());
    result.metadata.insert("processing_options".to_string(), 
        serde_json::to_value(&request.processing_options).unwrap());
    result.metadata.insert("processed_at".to_string(), 
        serde_json::to_value(chrono::Utc::now()).unwrap());
    
    result.ai_usage = total_usage;
    
    Ok(result)
}

// Email Generation AI Workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailGenerationAIRequest {
    pub tenant_id: String,
    pub user_id: String,
    pub email_type: EmailType,
    pub recipient_info: RecipientInfo,
    pub email_context: EmailContext,
    pub generation_options: EmailGenerationOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailType {
    Welcome,
    FollowUp,
    Reminder,
    Notification,
    Marketing,
    Support,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecipientInfo {
    pub name: Option<String>,
    pub role: Option<String>,
    pub company: Option<String>,
    pub relationship: Option<String>,
    pub communication_style: Option<CommunicationStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailContext {
    pub subject_hint: Option<String>,
    pub key_points: Vec<String>,
    pub call_to_action: Option<String>,
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,
    pub reference_data: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailGenerationOptions {
    pub tone: EmailTone,
    pub length: EmailLength,
    pub include_signature: bool,
    pub personalization_level: PersonalizationLevel,
    pub language: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailTone {
    Professional,
    Friendly,
    Urgent,
    Casual,
    Formal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmailLength {
    Brief,
    Medium,
    Detailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PersonalizationLevel {
    Low,
    Medium,
    High,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailGenerationAIResult {
    pub subject: String,
    pub body: String,
    pub suggested_send_time: Option<chrono::DateTime<chrono::Utc>>,
    pub personalization_elements: Vec<String>,
    pub ai_usage: TokenUsage,
}

pub async fn email_generation_ai_workflow(
    ctx: WfContext,
    request: EmailGenerationAIRequest,
) -> WorkflowResult<EmailGenerationAIResult> {
    let activities = ctx.activity(());
    
    // Build comprehensive prompt for email generation
    let mut prompt = format!(
        "Generate a {} email with the following specifications:\n\n",
        format!("{:?}", request.email_type).to_lowercase()
    );
    
    // Add recipient information
    if let Some(name) = &request.recipient_info.name {
        prompt.push_str(&format!("Recipient: {}\n", name));
    }
    if let Some(role) = &request.recipient_info.role {
        prompt.push_str(&format!("Recipient Role: {}\n", role));
    }
    if let Some(company) = &request.recipient_info.company {
        prompt.push_str(&format!("Company: {}\n", company));
    }
    
    // Add email context
    if let Some(subject_hint) = &request.email_context.subject_hint {
        prompt.push_str(&format!("Subject should relate to: {}\n", subject_hint));
    }
    
    if !request.email_context.key_points.is_empty() {
        prompt.push_str(&format!("Key points to include:\n"));
        for point in &request.email_context.key_points {
            prompt.push_str(&format!("- {}\n", point));
        }
    }
    
    if let Some(cta) = &request.email_context.call_to_action {
        prompt.push_str(&format!("Call to action: {}\n", cta));
    }
    
    // Add generation options
    prompt.push_str(&format!("\nTone: {:?}\n", request.generation_options.tone));
    prompt.push_str(&format!("Length: {:?}\n", request.generation_options.length));
    prompt.push_str(&format!("Personalization: {:?}\n", request.generation_options.personalization_level));
    
    prompt.push_str("\nGenerate both subject line and email body. Format as:\nSUBJECT: [subject line]\nBODY:\n[email body]");
    
    let generation_request = TextGenerationRequest {
        prompt,
        model: None,
        parameters: AIParameters {
            max_tokens: Some(match request.generation_options.length {
                EmailLength::Brief => 300,
                EmailLength::Medium => 600,
                EmailLength::Detailed => 1000,
            }),
            temperature: Some(0.7),
            ..Default::default()
        },
        context: RequestContext {
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            workflow_id: Some(ctx.workflow_info().workflow_id.clone()),
            activity_id: Some("generate_email".to_string()),
            session_id: None,
        },
    };
    
    let generation_result = activities.generate_text(generation_request).await?;
    
    // Parse the generated email
    let (subject, body) = parse_email_content(&generation_result.generated_text);
    
    // Suggest optimal send time based on email type and recipient info
    let suggested_send_time = calculate_optimal_send_time(&request);
    
    // Extract personalization elements
    let personalization_elements = extract_personalization_elements(&body, &request.recipient_info);
    
    Ok(EmailGenerationAIResult {
        subject,
        body,
        suggested_send_time,
        personalization_elements,
        ai_usage: generation_result.usage,
    })
}

// Helper functions for parsing AI responses
fn parse_learning_path(content: &str) -> Vec<LearningStep> {
    // Simplified parsing - in production, would use more sophisticated parsing
    content
        .split('\n')
        .filter(|line| !line.trim().is_empty())
        .take(6)
        .enumerate()
        .map(|(i, line)| LearningStep {
            title: format!("Step {}: {}", i + 1, line.trim()),
            description: format!("Complete {}", line.trim()),
            estimated_time_minutes: 15 + (i as u32 * 10),
            difficulty: if i < 2 { "Beginner" } else if i < 4 { "Intermediate" } else { "Advanced" }.to_string(),
            resources: vec!["Documentation".to_string(), "Video Tutorial".to_string()],
        })
        .collect()
}

fn parse_setup_tasks(content: &str) -> Vec<SetupTask> {
    // Simplified parsing
    content
        .split('\n')
        .filter(|line| !line.trim().is_empty())
        .take(5)
        .enumerate()
        .map(|(i, line)| SetupTask {
            title: line.trim().to_string(),
            description: format!("Complete: {}", line.trim()),
            priority: match i {
                0..=1 => TaskPriority::High,
                2..=3 => TaskPriority::Medium,
                _ => TaskPriority::Low,
            },
            estimated_time_minutes: 10 + (i as u32 * 5),
        })
        .collect()
}

fn parse_sentiment_result(content: &str) -> SentimentResult {
    // Simplified parsing - would use more sophisticated NLP in production
    let sentiment = if content.to_lowercase().contains("positive") {
        "Positive"
    } else if content.to_lowercase().contains("negative") {
        "Negative"
    } else {
        "Neutral"
    }.to_string();
    
    let confidence = 0.8; // Simplified confidence
    
    let mut emotions = HashMap::new();
    if content.to_lowercase().contains("joy") { emotions.insert("joy".to_string(), 0.7); }
    if content.to_lowercase().contains("anger") { emotions.insert("anger".to_string(), 0.6); }
    if content.to_lowercase().contains("trust") { emotions.insert("trust".to_string(), 0.8); }
    
    SentimentResult {
        sentiment,
        confidence,
        emotions,
    }
}

fn parse_email_content(content: &str) -> (String, String) {
    let lines: Vec<&str> = content.lines().collect();
    let mut subject = String::new();
    let mut body = String::new();
    let mut in_body = false;
    
    for line in lines {
        if line.starts_with("SUBJECT:") {
            subject = line.replace("SUBJECT:", "").trim().to_string();
        } else if line.starts_with("BODY:") {
            in_body = true;
        } else if in_body {
            body.push_str(line);
            body.push('\n');
        }
    }
    
    if subject.is_empty() {
        subject = "Generated Email".to_string();
    }
    
    (subject, body.trim().to_string())
}

fn calculate_optimal_send_time(request: &EmailGenerationAIRequest) -> Option<chrono::DateTime<chrono::Utc>> {
    // Simplified logic - in production, would consider timezone, email type, recipient behavior, etc.
    let now = chrono::Utc::now();
    
    match request.email_type {
        EmailType::Welcome => Some(now + chrono::Duration::minutes(5)), // Send immediately
        EmailType::Reminder => Some(now + chrono::Duration::hours(2)), // Send in 2 hours
        EmailType::Marketing => Some(now + chrono::Duration::days(1)), // Send tomorrow
        _ => Some(now + chrono::Duration::hours(1)), // Default: 1 hour
    }
}

fn extract_personalization_elements(body: &str, recipient_info: &RecipientInfo) -> Vec<String> {
    let mut elements = Vec::new();
    
    if let Some(name) = &recipient_info.name {
        if body.contains(name) {
            elements.push(format!("Personalized with recipient name: {}", name));
        }
    }
    
    if let Some(company) = &recipient_info.company {
        if body.contains(company) {
            elements.push(format!("Referenced company: {}", company));
        }
    }
    
    if let Some(role) = &recipient_info.role {
        if body.contains(role) {
            elements.push(format!("Tailored for role: {}", role));
        }
    }
    
    elements
}