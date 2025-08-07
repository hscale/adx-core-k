# ADX CORE - AI Integration Patterns with Temporal.io

## Overview

ADX CORE's hybrid AI approach leverages Temporal.io's workflow orchestration capabilities to provide simple, reliable AI-enhanced workflows. Instead of building complex AI orchestration systems, we use Temporal's proven patterns to make AI integration straightforward and maintainable.

## Temporal-First AI Architecture

### Simple AI Integration Framework
```
┌─────────────────────────────────────────────────────────────────────────────────┐
│                         Temporal.io Workflow Engine                            │
│                        (Handles All Orchestration)                             │
└─────────────────────────────────────────────────────────────────────────────────┘
                                        │
        ┌───────────────────────────────┼───────────────────────────────┐
        │                               │                               │
┌───────────────────┐        ┌───────────────────┐        ┌───────────────────┐
│  Standard         │        │   AI-Enhanced     │        │   AI Activities   │
│  Workflows        │        │   Workflows       │        │   (Plugins)       │
│                   │        │                   │        │                   │
│ • User Onboard    │        │ • Smart Routing   │        │ • LLM Calls       │
│ • File Process    │        │ • Auto Optimize   │        │ • ML Inference    │
│ • Email Send      │        │ • Smart Retry     │        │ • Data Analysis   │
│ • Data Sync       │        │ • Context Aware   │        │ • Content Gen     │
│ • Backup          │        │ • Predictive      │        │ • Classification  │
└───────────────────┘        └───────────────────┘        └───────────────────┘
```

## Temporal-Based AI Workflows

### Simple AI-Enhanced Workflows
```rust
// AI-enhanced workflows using Temporal.io patterns
use temporal_sdk::{workflow, activity, WorkflowResult};

// Standard workflow that can be AI-enhanced
#[workflow]
pub async fn user_onboarding_workflow(
    user_data: UserRegistrationData,
    ai_enhanced: bool,
) -> WorkflowResult<OnboardingResult> {
    // Step 1: Create user account (always done)
    let user = create_user_activity(user_data.clone()).await?;
    
    // Step 2: AI-enhanced welcome message (if enabled)
    let welcome_message = if ai_enhanced {
        generate_personalized_welcome_activity(user.clone()).await?
    } else {
        get_default_welcome_message_activity().await?
    };
    
    // Step 3: Send welcome email
    send_welcome_email_activity(user.email.clone(), welcome_message).await?;
    
    // Step 4: AI-enhanced setup recommendations (if enabled)
    let setup_tasks = if ai_enhanced {
        generate_smart_setup_tasks_activity(user.clone()).await?
    } else {
        get_default_setup_tasks_activity().await?
    };
    
    // Step 5: Create setup tasks
    create_setup_tasks_activity(user.id, setup_tasks).await?;
    
    Ok(OnboardingResult {
        user_id: user.id,
        ai_enhanced,
        setup_tasks_count: setup_tasks.len(),
    })
}

// AI Activities - Simple, focused functions
#[activity]
pub async fn generate_personalized_welcome_activity(user: User) -> Result<String, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Generate a personalized welcome message for a new user:
        Name: {}
        Company: {}
        Industry: {}
        
        Make it friendly, professional, and mention 2-3 relevant features they might find useful.",
        user.name, 
        user.company_name.unwrap_or_default(),
        user.industry.unwrap_or_default()
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    Ok(response.content)
}

#[activity]
pub async fn generate_smart_setup_tasks_activity(user: User) -> Result<Vec<SetupTask>, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Based on this user profile, suggest 3-5 setup tasks that would be most valuable:
        Role: {}
        Company Size: {}
        Industry: {}
        
        Return as JSON array with task name and description.",
        user.role.unwrap_or_default(),
        user.company_size.unwrap_or_default(),
        user.industry.unwrap_or_default()
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let tasks: Vec<SetupTask> = serde_json::from_str(&response.content)?;
    Ok(tasks)
}

// Smart retry workflow with AI-enhanced error handling
#[workflow]
pub async fn smart_retry_workflow<T>(
    operation: impl Fn() -> WorkflowResult<T>,
    ai_enhanced: bool,
) -> WorkflowResult<T> {
    let mut attempts = 0;
    let max_attempts = 3;
    
    loop {
        attempts += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempts >= max_attempts {
                    return Err(error);
                }
                
                // AI-enhanced retry strategy
                let retry_delay = if ai_enhanced {
                    calculate_smart_retry_delay_activity(error.clone(), attempts).await?
                } else {
                    Duration::from_secs(2_u64.pow(attempts)) // Exponential backoff
                };
                
                temporal_sdk::sleep(retry_delay).await;
            }
        }
    }
}
```

### AI Activities for Common Use Cases
```rust
// Simple AI activities that can be used in any Temporal workflow

#[activity]
pub async fn ai_classify_content_activity(content: String, categories: Vec<String>) -> Result<String, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Classify this content into one of these categories: {}
        
        Content: {}
        
        Return only the category name.",
        categories.join(", "),
        content
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    Ok(response.content.trim().to_string())
}

#[activity]
pub async fn ai_summarize_activity(text: String, max_length: usize) -> Result<String, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Summarize this text in approximately {} words:
        
        {}",
        max_length / 5, // Rough words estimate
        text
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    Ok(response.content)
}

#[activity]
pub async fn ai_extract_entities_activity(text: String) -> Result<Vec<Entity>, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Extract entities (people, organizations, locations, dates) from this text.
        Return as JSON array with 'type' and 'value' fields:
        
        {}",
        text
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let entities: Vec<Entity> = serde_json::from_str(&response.content)?;
    Ok(entities)
}

#[activity]
pub async fn calculate_smart_retry_delay_activity(
    error: WorkflowError, 
    attempt: u32
) -> Result<Duration, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Given this error and attempt number, suggest an optimal retry delay:
        Error: {}
        Attempt: {}
        
        Consider:
        - Error type (network, rate limit, server error)
        - Attempt number
        - Best practices for retry delays
        
        Return delay in seconds as a number.",
        error.message,
        attempt
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let seconds: u64 = response.content.trim().parse().unwrap_or(2_u64.pow(attempt));
    Ok(Duration::from_secs(seconds))
}

// Document processing workflow with AI enhancement
#[workflow]
pub async fn document_processing_workflow(
    document: Document,
    ai_enhanced: bool,
) -> WorkflowResult<ProcessedDocument> {
    // Step 1: Extract text (always done)
    let text = extract_text_activity(document.clone()).await?;
    
    // Step 2: AI-enhanced processing (if enabled)
    let processed_data = if ai_enhanced {
        // AI activities run in parallel
        let (summary, entities, classification) = temporal_sdk::join!(
            ai_summarize_activity(text.clone(), 200),
            ai_extract_entities_activity(text.clone()),
            ai_classify_content_activity(text.clone(), vec![
                "Contract".to_string(),
                "Invoice".to_string(), 
                "Report".to_string(),
                "Other".to_string()
            ])
        );
        
        ProcessedData {
            summary: Some(summary?),
            entities: Some(entities?),
            classification: Some(classification?),
        }
    } else {
        ProcessedData::default()
    };
    
    // Step 3: Store processed document
    let result = store_processed_document_activity(document.id, processed_data).await?;
    
    Ok(result)
}
```

## AI Service Integration

### Simple AI Service Pattern
```rust
// Simple AI service that all activities can use
pub struct AIService {
    client: Arc<dyn AIClient>,
    model_config: ModelConfig,
    fallback_enabled: bool,
}

pub trait AIClient: Send + Sync {
    async fn generate_text(&self, prompt: &str) -> Result<AIResponse, AIError>;
    async fn classify(&self, text: &str, categories: &[String]) -> Result<String, AIError>;
    async fn extract_entities(&self, text: &str) -> Result<Vec<Entity>, AIError>;
    async fn summarize(&self, text: &str, max_length: usize) -> Result<String, AIError>;
}

impl AIService {
    pub async fn generate_text(&self, prompt: &str) -> Result<AIResponse, AIError> {
        match self.client.generate_text(prompt).await {
            Ok(response) => Ok(response),
            Err(error) if self.fallback_enabled => {
                // Simple fallback to template-based response
                warn!("AI service failed, using fallback: {}", error);
                Ok(AIResponse {
                    content: "I apologize, but I'm unable to generate a personalized response right now. Please try again later.".to_string(),
                    confidence: 0.0,
                    model_used: "fallback".to_string(),
                })
            }
            Err(error) => Err(error),
        }
    }
}

// AI-enhanced email workflow
#[workflow]
pub async fn smart_email_workflow(
    recipient: String,
    subject: String,
    content: String,
    ai_enhanced: bool,
) -> WorkflowResult<EmailResult> {
    // Step 1: AI enhancement (if enabled)
    let (final_subject, final_content) = if ai_enhanced {
        let enhanced_subject = ai_improve_email_subject_activity(subject.clone()).await?;
        let enhanced_content = ai_improve_email_content_activity(content.clone()).await?;
        (enhanced_subject, enhanced_content)
    } else {
        (subject, content)
    };
    
    // Step 2: Send email
    let email_result = send_email_activity(recipient, final_subject, final_content).await?;
    
    // Step 3: AI-powered follow-up scheduling (if enabled)
    if ai_enhanced && email_result.sent_successfully {
        let follow_up_delay = ai_suggest_followup_timing_activity(email_result.clone()).await?;
        
        // Schedule follow-up workflow
        temporal_sdk::start_child_workflow(
            follow_up_email_workflow,
            FollowUpData {
                original_email_id: email_result.email_id.clone(),
                delay: follow_up_delay,
            }
        ).await?;
    }
    
    Ok(email_result)
}

#[activity]
pub async fn ai_improve_email_subject_activity(subject: String) -> Result<String, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Improve this email subject line to be more engaging and clear:
        
        Original: {}
        
        Return only the improved subject line.",
        subject
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    Ok(response.content.trim().to_string())
}

#[activity]
pub async fn ai_suggest_followup_timing_activity(email_result: EmailResult) -> Result<Duration, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Based on this email context, suggest optimal follow-up timing:
        Subject: {}
        Sent at: {}
        Type: business email
        
        Return number of hours to wait before follow-up.",
        email_result.subject,
        email_result.sent_at
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let hours: u64 = response.content.trim().parse().unwrap_or(24);
    Ok(Duration::from_secs(hours * 3600))
}
```

## AI-Enhanced Error Handling

### Simple AI Error Recovery
```rust
// AI-enhanced error handling using Temporal's built-in retry mechanisms
#[workflow]
pub async fn resilient_workflow_with_ai<T>(
    operation: impl Fn() -> WorkflowResult<T>,
    ai_enhanced: bool,
) -> WorkflowResult<T> {
    let retry_policy = if ai_enhanced {
        // AI suggests retry strategy
        create_smart_retry_policy_activity().await?
    } else {
        // Default exponential backoff
        RetryPolicy::default()
    };
    
    temporal_sdk::retry_with_policy(retry_policy, operation).await
}

#[activity]
pub async fn create_smart_retry_policy_activity() -> Result<RetryPolicy, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = "Based on current system conditions and best practices, suggest retry policy parameters:
    - Initial interval (seconds)
    - Backoff coefficient 
    - Maximum interval (seconds)
    - Maximum attempts
    
    Return as JSON with these fields.";
    
    let response = ai_service.generate_text(prompt).await?;
    let policy_params: RetryPolicyParams = serde_json::from_str(&response.content)
        .unwrap_or_default();
    
    Ok(RetryPolicy {
        initial_interval: Duration::from_secs(policy_params.initial_interval),
        backoff_coefficient: policy_params.backoff_coefficient,
        maximum_interval: Duration::from_secs(policy_params.maximum_interval),
        maximum_attempts: policy_params.maximum_attempts,
    })
}

// AI-enhanced data processing workflow
#[workflow]
pub async fn data_processing_workflow(
    data_batch: DataBatch,
    ai_enhanced: bool,
) -> WorkflowResult<ProcessingResult> {
    let mut results = Vec::new();
    
    // Process each item
    for item in data_batch.items {
        let processed_item = if ai_enhanced {
            // AI-enhanced processing
            ai_process_data_item_activity(item).await?
        } else {
            // Standard processing
            standard_process_data_item_activity(item).await?
        };
        
        results.push(processed_item);
    }
    
    // AI-enhanced quality check (if enabled)
    if ai_enhanced {
        let quality_score = ai_assess_batch_quality_activity(results.clone()).await?;
        
        if quality_score < 0.8 {
            // Trigger quality improvement workflow
            temporal_sdk::start_child_workflow(
                quality_improvement_workflow,
                QualityImprovementData {
                    batch_id: data_batch.id,
                    quality_score,
                    items: results.clone(),
                }
            ).await?;
        }
    }
    
    Ok(ProcessingResult {
        batch_id: data_batch.id,
        processed_items: results,
        ai_enhanced,
    })
}

#[activity]
pub async fn ai_process_data_item_activity(item: DataItem) -> Result<ProcessedItem, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Process this data item and extract key information:
        Type: {}
        Content: {}
        
        Return structured data as JSON with relevant fields extracted.",
        item.item_type,
        item.content
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let processed_data: ProcessedData = serde_json::from_str(&response.content)?;
    
    Ok(ProcessedItem {
        original_id: item.id,
        processed_data,
        processing_method: "ai_enhanced".to_string(),
        confidence: response.confidence,
    })
}
```

## AI-Powered Workflow Optimization

### Simple Performance Optimization
```rust
// AI-enhanced workflow optimization using Temporal's built-in features
#[workflow]
pub async fn optimized_workflow<T>(
    operation: impl Fn() -> WorkflowResult<T>,
    ai_enhanced: bool,
) -> WorkflowResult<T> {
    let start_time = temporal_sdk::now();
    
    // Execute operation
    let result = operation().await?;
    
    // AI-enhanced performance analysis (if enabled)
    if ai_enhanced {
        let execution_time = temporal_sdk::now() - start_time;
        
        // Analyze performance and suggest improvements
        temporal_sdk::start_child_workflow(
            performance_analysis_workflow,
            PerformanceData {
                workflow_type: std::any::type_name::<T>().to_string(),
                execution_time,
                success: true,
            }
        ).await?;
    }
    
    Ok(result)
}

#[workflow]
pub async fn performance_analysis_workflow(
    performance_data: PerformanceData,
) -> WorkflowResult<()> {
    // AI analyzes performance and suggests improvements
    let suggestions = ai_analyze_performance_activity(performance_data).await?;
    
    // Store suggestions for future optimization
    store_optimization_suggestions_activity(suggestions).await?;
    
    Ok(())
}

#[activity]
pub async fn ai_analyze_performance_activity(
    performance_data: PerformanceData,
) -> Result<Vec<OptimizationSuggestion>, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Analyze this workflow performance and suggest optimizations:
        Workflow: {}
        Execution time: {:?}
        Success: {}
        
        Suggest 2-3 specific improvements as JSON array with 'type' and 'description' fields.",
        performance_data.workflow_type,
        performance_data.execution_time,
        performance_data.success
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let suggestions: Vec<OptimizationSuggestion> = serde_json::from_str(&response.content)
        .unwrap_or_default();
    
    Ok(suggestions)
}

// AI-enhanced user notification workflow
#[workflow]
pub async fn smart_notification_workflow(
    user_id: UserId,
    notification_type: NotificationType,
    content: String,
    ai_enhanced: bool,
) -> WorkflowResult<NotificationResult> {
    // Step 1: AI-enhanced personalization (if enabled)
    let personalized_content = if ai_enhanced {
        ai_personalize_notification_activity(user_id, content).await?
    } else {
        content
    };
    
    // Step 2: AI-enhanced delivery timing (if enabled)
    let optimal_delivery_time = if ai_enhanced {
        ai_calculate_optimal_delivery_time_activity(user_id, notification_type).await?
    } else {
        temporal_sdk::now()
    };
    
    // Step 3: Wait until optimal time
    if optimal_delivery_time > temporal_sdk::now() {
        temporal_sdk::sleep_until(optimal_delivery_time).await;
    }
    
    // Step 4: Send notification
    let result = send_notification_activity(user_id, personalized_content).await?;
    
    Ok(result)
}

#[activity]
pub async fn ai_personalize_notification_activity(
    user_id: UserId,
    content: String,
) -> Result<String, ActivityError> {
    let ai_service = get_ai_service().await?;
    let user_profile = get_user_profile(user_id).await?;
    
    let prompt = format!(
        "Personalize this notification for the user:
        User name: {}
        User preferences: {}
        Original content: {}
        
        Return personalized version that feels natural and relevant.",
        user_profile.name,
        serde_json::to_string(&user_profile.preferences)?,
        content
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    Ok(response.content)
}

#[activity]
pub async fn ai_calculate_optimal_delivery_time_activity(
    user_id: UserId,
    notification_type: NotificationType,
) -> Result<DateTime<Utc>, ActivityError> {
    let ai_service = get_ai_service().await?;
    let user_activity = get_user_activity_pattern(user_id).await?;
    
    let prompt = format!(
        "Based on user activity patterns, suggest optimal notification delivery time:
        Current time: {}
        User timezone: {}
        Notification type: {:?}
        User typically active: {}
        
        Return optimal time as ISO 8601 timestamp.",
        Utc::now(),
        user_activity.timezone,
        notification_type,
        user_activity.active_hours.join(", ")
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let optimal_time = DateTime::parse_from_rfc3339(response.content.trim())
        .unwrap_or_else(|_| Utc::now().into())
        .with_timezone(&Utc);
    
    Ok(optimal_time)
}
```

## Simple AI Learning and Feedback

### Feedback Collection Workflow
```rust
// Simple feedback collection using Temporal workflows
#[workflow]
pub async fn collect_ai_feedback_workflow(
    workflow_execution_id: String,
    user_id: UserId,
) -> WorkflowResult<()> {
    // Wait for user to potentially provide feedback
    let feedback_timeout = Duration::from_hours(24);
    
    let feedback = temporal_sdk::select! {
        feedback = wait_for_user_feedback_signal() => Some(feedback),
        _ = temporal_sdk::sleep(feedback_timeout) => None,
    };
    
    if let Some(feedback) = feedback {
        // Store feedback for future AI improvements
        store_ai_feedback_activity(workflow_execution_id, user_id, feedback).await?;
        
        // If feedback is negative, trigger improvement workflow
        if feedback.rating < 3 {
            temporal_sdk::start_child_workflow(
                ai_improvement_workflow,
                ImprovementData {
                    workflow_id: workflow_execution_id,
                    feedback,
                }
            ).await?;
        }
    }
    
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSession {
    pub session_id: String,
    pub learning_type: LearningType,
    pub data_sources: Vec<DataSource>,
    pub patterns_discovered: Vec<Pattern>,
    pub model_updates: Vec<ModelUpdate>,
    pub knowledge_updates: Vec<KnowledgeUpdate>,
    pub performance_improvements: Vec<PerformanceImprovement>,
    pub confidence_changes: Vec<ConfidenceChange>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LearningType {
    SupervisedLearning,
    UnsupervisedLearning,
    ReinforcementLearning,
    TransferLearning,
    FederatedLearning,
    ContinualLearning,
}

impl AILearningEngine {
    pub async fn learn_from_execution(
        &self,
        execution: &CompletedWorkflowExecution,
        outcome: &WorkflowOutcome,
        user_feedback: Option<&UserFeedback>,
    ) -> Result<LearningSession, AIError> {
        let session_id = Uuid::new_v4().to_string();
        
        // Extract learning data
        let execution_data = self.extract_execution_data(execution).await?;
        let outcome_data = self.extract_outcome_data(outcome).await?;
        let feedback_data = if let Some(feedback) = user_feedback {
            Some(self.extract_feedback_data(feedback).await?)
        } else {
            None
        };
        
        // Discover patterns
        let patterns = self.pattern_recognizer.discover_patterns(
            &execution_data,
            &outcome_data,
            feedback_data.as_ref(),
        ).await?;
        
        // Update models based on patterns
        let model_updates = self.update_models(&patterns, execution, outcome).await?;
        
        // Update knowledge base
        let knowledge_updates = self.update_knowledge_base(&patterns, execution, outcome).await?;
        
        // Track performance improvements
        let performance_improvements = self.track_performance_improvements(&model_updates).await?;
        
        // Update confidence scores
        let confidence_changes = self.update_confidence_scores(&patterns, &model_updates).await?;
        
        Ok(LearningSession {
            session_id,
            learning_type: LearningType::ContinualLearning,
            data_sources: vec![
                DataSource::WorkflowExecution,
                DataSource::UserFeedback,
                DataSource::PerformanceMetrics,
            ],
            patterns_discovered: patterns,
            model_updates,
            knowledge_updates,
            performance_improvements,
            confidence_changes,
        })
    }
    
    async fn discover_patterns(
        &self,
        execution_data: &ExecutionData,
        outcome_data: &OutcomeData,
        feedback_data: Option<&FeedbackData>,
    ) -> Result<Vec<Pattern>, AIError> {
        let mut patterns = Vec::new();
        
        // Temporal patterns
        let temporal_patterns = self.discover_temporal_patterns(execution_data).await?;
        patterns.extend(temporal_patterns);
        
        // Success/failure patterns
        let outcome_patterns = self.discover_outcome_patterns(execution_data, outcome_data).await?;
        patterns.extend(outcome_patterns);
        
        // User behavior patterns
        if let Some(feedback) = feedback_data {
            let user_patterns = self.discover_user_patterns(execution_data, feedback).await?;
            patterns.extend(user_patterns);
        }
        
        // Resource usage patterns
        let resource_patterns = self.discover_resource_patterns(execution_data).await?;
        patterns.extend(resource_patterns);
        
        // Context patterns
        let context_patterns = self.discover_context_patterns(execution_data, outcome_data).await?;
        patterns.extend(context_patterns);
        
        Ok(patterns)
    }
    
    async fn update_models(&self, patterns: &[Pattern], execution: &CompletedWorkflowExecution, outcome: &WorkflowOutcome) -> Result<Vec<ModelUpdate>, AIError> {
        let mut updates = Vec::new();
        
        for pattern in patterns {
            match pattern.pattern_type {
                PatternType::PerformancePrediction => {
                    let update = self.update_performance_model(pattern, execution, outcome).await?;
                    updates.push(update);
                }
                PatternType::ExceptionPrediction => {
                    let update = self.update_exception_model(pattern, execution, outcome).await?;
                    updates.push(update);
                }
                PatternType::UserPreference => {
                    let update = self.update_preference_model(pattern, execution, outcome).await?;
                    updates.push(update);
                }
                PatternType::ResourceOptimization => {
                    let update = self.update_optimization_model(pattern, execution, outcome).await?;
                    updates.push(update);
                }
                _ => {}
            }
        }
        
        Ok(updates)
    }
    
    pub async fn federated_learning_update(
        &self,
        tenant_models: &[TenantModel],
        global_model: &GlobalModel,
    ) -> Result<FederatedUpdate, AIError> {
        // Aggregate tenant model updates
        let aggregated_weights = self.aggregate_model_weights(tenant_models).await?;
        
        // Apply differential privacy
        let private_weights = self.apply_differential_privacy(&aggregated_weights).await?;
        
        // Update global model
        let updated_global_model = self.update_global_model(global_model, &private_weights).await?;
        
        // Validate updated model
        let validation_results = self.validate_federated_model(&updated_global_model).await?;
        
        Ok(FederatedUpdate {
            update_id: Uuid::new_v4().to_string(),
            participating_tenants: tenant_models.len(),
            global_model: updated_global_model,
            validation_results,
            privacy_budget_used: self.calculate_privacy_budget_usage(&private_weights),
            performance_improvement: self.calculate_federated_improvement(&validation_results),
        })
    }
}
```

## Simple AI Model Management

### Basic Model Selection
```rust
// Simple model selection based on tenant tier and task type
pub struct SimpleAIModelSelector {
    models: HashMap<String, AIModelConfig>,
}

#[derive(Debug, Clone)]
pub struct AIModelConfig {
    pub name: String,
    pub provider: String,
    pub cost_per_token: f64,
    pub max_tokens: u32,
    pub suitable_for: Vec<TaskType>,
    pub min_tier: LicenseTier,
}

impl SimpleAIModelSelector {
    pub fn select_model(&self, task_type: TaskType, tenant_tier: LicenseTier) -> Option<&AIModelConfig> {
        self.models
            .values()
            .filter(|model| {
                model.suitable_for.contains(&task_type) && 
                tenant_tier >= model.min_tier
            })
            .min_by_key(|model| model.cost_per_token as u64) // Select cheapest suitable model
    }
}

// Global AI service getter with simple model selection
pub async fn get_ai_service() -> Result<AIService, AIError> {
    let tenant_context = get_current_tenant_context().await?;
    let model_selector = SimpleAIModelSelector::new();
    
    let model_config = model_selector
        .select_model(TaskType::TextGeneration, tenant_context.license_tier)
        .ok_or(AIError::NoSuitableModel)?;
    
    Ok(AIService::new(model_config.clone()))
}

// Simple AI workflow patterns that can be used throughout the platform
#[workflow]
pub async fn ai_content_generation_workflow(
    prompt: String,
    content_type: ContentType,
    ai_enhanced: bool,
) -> WorkflowResult<GeneratedContent> {
    if !ai_enhanced {
        return Ok(GeneratedContent::default());
    }
    
    let generated_content = ai_generate_content_activity(prompt, content_type).await?;
    
    // Simple quality check
    let quality_score = ai_assess_content_quality_activity(generated_content.clone()).await?;
    
    if quality_score < 0.7 {
        // Retry with improved prompt
        let improved_prompt = ai_improve_prompt_activity(prompt).await?;
        let regenerated_content = ai_generate_content_activity(improved_prompt, content_type).await?;
        Ok(regenerated_content)
    } else {
        Ok(generated_content)
    }
}

#[activity]
pub async fn ai_generate_content_activity(
    prompt: String,
    content_type: ContentType,
) -> Result<GeneratedContent, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let enhanced_prompt = format!(
        "Generate {} content based on this request:
        
        {}
        
        Make it professional, clear, and appropriate for business use.",
        content_type.to_string().to_lowercase(),
        prompt
    );
    
    let response = ai_service.generate_text(&enhanced_prompt).await?;
    
    Ok(GeneratedContent {
        content: response.content,
        content_type,
        confidence: response.confidence,
        generated_at: Utc::now(),
    })
}

#[activity]
pub async fn ai_assess_content_quality_activity(
    content: GeneratedContent,
) -> Result<f64, ActivityError> {
    let ai_service = get_ai_service().await?;
    
    let prompt = format!(
        "Rate the quality of this generated content on a scale of 0.0 to 1.0:
        
        {}
        
        Consider clarity, professionalism, and relevance. Return only the numeric score.",
        content.content
    );
    
    let response = ai_service.generate_text(&prompt).await?;
    let quality_score: f64 = response.content.trim().parse().unwrap_or(0.5);
    
    Ok(quality_score.clamp(0.0, 1.0))
}

## Key Benefits of This Temporal-First Approach

1. **Simplicity**: Uses Temporal's proven patterns instead of building complex orchestration
2. **Reliability**: Leverages Temporal's durability and error handling
3. **Maintainability**: Simple activities are easy to test and debug
4. **Scalability**: Temporal handles the complex orchestration automatically
5. **Flexibility**: Easy to add AI enhancement to any existing workflow
6. **Fallback**: Always works even when AI services are unavailable
7. **Cost Control**: Simple model selection based on tenant tier
8. **Monitoring**: Built-in workflow visibility through Temporal UI

This approach makes AI integration straightforward and reliable, focusing on practical business value rather than complex AI orchestration systems.
```

This comprehensive AI integration patterns document provides:

1. **Advanced AI Planning Engine** with intent analysis and strategy generation
2. **Real-time AI Execution Engine** with intelligent decision making
3. **Sophisticated Exception Handling** with AI-powered resolution strategies
4. **Performance Optimization Engine** with multi-dimensional analysis
5. **Continuous Learning System** with pattern recognition and model updates
6. **Advanced Model Registry** with optimal model selection and A/B testing
7. **Federated Learning Support** for privacy-preserving model updates
8. **Context-Aware AI Systems** with comprehensive context analysis

Now let me enhance the deployment architecture with more detailed infrastructure specifications.

<function_calls>
<invoke name="fsWrite">
<parameter name="path">.kiro/specs/adx-core/00-overview/infrastructure-deployment.md