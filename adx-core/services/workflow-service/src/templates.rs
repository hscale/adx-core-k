use crate::{
    config::WorkflowServiceConfig,
    error::{WorkflowServiceError, WorkflowServiceResult},
    models::*,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing::{info, warn, error};
use uuid::Uuid;

/// Workflow template management service for reusable patterns
pub struct WorkflowTemplateManager {
    config: Arc<WorkflowServiceConfig>,
    template_registry: Arc<TemplateRegistry>,
    pattern_analyzer: Arc<PatternAnalyzer>,
    template_generator: Arc<TemplateGenerator>,
}

impl WorkflowTemplateManager {
    pub fn new(config: Arc<WorkflowServiceConfig>) -> Self {
        let template_registry = Arc::new(TemplateRegistry::new());
        let pattern_analyzer = Arc::new(PatternAnalyzer::new());
        let template_generator = Arc::new(TemplateGenerator::new());

        Self {
            config,
            template_registry,
            pattern_analyzer,
            template_generator,
        }
    }

    /// Create a new workflow template
    pub async fn create_template(&self, request: CreateTemplateRequest) -> WorkflowServiceResult<CreateTemplateResponse> {
        info!("Creating workflow template: {}", request.template_name);

        // Validate template structure
        self.validate_template_structure(&request.template_definition)?;

        // Analyze template patterns
        let pattern_analysis = self.pattern_analyzer.analyze_template(&request.template_definition).await?;

        // Register template
        let registration = self.template_registry.register_template(&request, pattern_analysis).await?;

        Ok(CreateTemplateResponse {
            template_id: registration.template_id,
            template_name: request.template_name,
            version: registration.version,
            created: registration.success,
            created_at: registration.created_at,
            validation_results: registration.validation_results,
            pattern_analysis: registration.pattern_analysis,
        })
    }

    /// Get available workflow templates
    pub async fn get_templates(&self, params: GetTemplatesParams) -> WorkflowServiceResult<GetTemplatesResponse> {
        info!("Getting workflow templates with filters: {:?}", params);

        let templates = self.template_registry.get_templates(&params).await?;
        let total_count = templates.len() as u32;

        Ok(GetTemplatesResponse {
            templates,
            total_count,
            categories: self.template_registry.get_categories().await?,
            tags: self.template_registry.get_tags().await?,
        })
    }

    /// Get a specific template
    pub async fn get_template(&self, template_id: &str) -> WorkflowServiceResult<WorkflowTemplate> {
        info!("Getting workflow template: {}", template_id);

        let template = self.template_registry.get_template(template_id).await?;

        Ok(template)
    }

    /// Create workflow from template
    pub async fn create_workflow_from_template(&self, request: CreateFromTemplateRequest) -> WorkflowServiceResult<CreateFromTemplateResponse> {
        info!("Creating workflow from template: {} with name: {}", request.template_id, request.workflow_name);

        // Get template
        let template = self.template_registry.get_template(&request.template_id).await?;

        // Validate parameters
        self.validate_template_parameters(&template, &request.parameters)?;

        // Generate workflow from template
        let workflow_definition = self.template_generator.generate_workflow(&template, &request).await?;

        // Create workflow instance
        let workflow_id = format!("{}_{}", request.workflow_name, Uuid::new_v4());

        Ok(CreateFromTemplateResponse {
            workflow_id,
            template_id: request.template_id,
            workflow_name: request.workflow_name,
            workflow_definition,
            parameters_used: request.parameters,
            created_at: Utc::now(),
        })
    }

    /// Update an existing template
    pub async fn update_template(&self, request: UpdateTemplateRequest) -> WorkflowServiceResult<UpdateTemplateResponse> {
        info!("Updating workflow template: {}", request.template_id);

        // Validate template exists
        let existing_template = self.template_registry.get_template(&request.template_id).await?;

        // Validate new structure if provided
        if let Some(ref definition) = request.template_definition {
            self.validate_template_structure(definition)?;
        }

        // Update template
        let update_result = self.template_registry.update_template(&request).await?;

        Ok(UpdateTemplateResponse {
            template_id: request.template_id,
            updated: update_result.success,
            updated_at: update_result.updated_at,
            new_version: update_result.new_version,
            changes_summary: update_result.changes_summary,
        })
    }

    /// Delete a template
    pub async fn delete_template(&self, template_id: &str, force: bool) -> WorkflowServiceResult<DeleteTemplateResponse> {
        info!("Deleting workflow template: {} (force: {})", template_id, force);

        // Check for active workflows using this template
        let active_workflows = self.template_registry.get_active_workflows_using_template(template_id).await?;

        if !active_workflows.is_empty() && !force {
            return Err(WorkflowServiceError::TemplateInUse(
                format!("Template {} is used by {} active workflows", template_id, active_workflows.len())
            ));
        }

        // Delete template
        let deletion_result = self.template_registry.delete_template(template_id, force).await?;

        Ok(DeleteTemplateResponse {
            template_id: template_id.to_string(),
            deleted: deletion_result.success,
            deleted_at: deletion_result.deleted_at,
            affected_workflows: active_workflows.len() as u32,
            cleanup_performed: force,
        })
    }

    /// Analyze workflow patterns to suggest templates
    pub async fn analyze_workflow_patterns(&self, params: PatternAnalysisParams) -> WorkflowServiceResult<PatternAnalysisResponse> {
        info!("Analyzing workflow patterns for tenant: {:?}", params.tenant_id);

        let analysis = self.pattern_analyzer.analyze_workflow_patterns(&params).await?;

        Ok(PatternAnalysisResponse {
            analysis_id: Uuid::new_v4().to_string(),
            patterns_found: analysis.patterns_found,
            template_suggestions: analysis.template_suggestions,
            optimization_opportunities: analysis.optimization_opportunities,
            reusability_score: analysis.reusability_score,
            analyzed_at: Utc::now(),
        })
    }

    /// Generate template from existing workflows
    pub async fn generate_template_from_workflows(&self, request: GenerateTemplateRequest) -> WorkflowServiceResult<GenerateTemplateResponse> {
        info!("Generating template from {} workflows", request.workflow_ids.len());

        // Analyze common patterns across workflows
        let pattern_analysis = self.pattern_analyzer.analyze_workflows(&request.workflow_ids).await?;

        // Generate template definition
        let template_definition = self.template_generator.generate_template_from_patterns(&pattern_analysis).await?;

        // Create template
        let template_name = request.template_name.clone();
        let confidence_score = pattern_analysis.confidence_score;
        let extracted_parameters = pattern_analysis.extracted_parameters.clone();
        let create_request = CreateTemplateRequest {
            template_name: request.template_name,
            description: request.description,
            category: request.category,
            tags: request.tags,
            template_definition,
            parameters: extracted_parameters,
            author: request.author,
        };

        let registration = self.template_registry.register_template(&create_request, pattern_analysis.clone()).await?;

        Ok(GenerateTemplateResponse {
            template_id: registration.template_id,
            template_name,
            generated: registration.success,
            generated_at: registration.created_at,
            source_workflows: request.workflow_ids,
            pattern_analysis,
            confidence_score,
        })
    }

    /// Get template usage statistics
    pub async fn get_template_usage(&self, template_id: &str) -> WorkflowServiceResult<TemplateUsageResponse> {
        info!("Getting usage statistics for template: {}", template_id);

        let usage_stats = self.template_registry.get_template_usage(template_id).await?;

        Ok(TemplateUsageResponse {
            template_id: template_id.to_string(),
            total_uses: usage_stats.total_uses,
            active_workflows: usage_stats.active_workflows,
            success_rate: usage_stats.success_rate,
            average_duration: usage_stats.average_duration,
            usage_by_tenant: usage_stats.usage_by_tenant,
            usage_trends: usage_stats.usage_trends,
            last_used: usage_stats.last_used,
        })
    }

    // Private helper methods

    fn validate_template_structure(&self, definition: &TemplateDefinition) -> WorkflowServiceResult<()> {
        // Validate required fields
        if definition.steps.is_empty() {
            return Err(WorkflowServiceError::InvalidTemplate(
                "Template must have at least one step".to_string()
            ));
        }

        // Validate step references
        for step in &definition.steps {
            if let Some(ref depends_on) = step.depends_on {
                for dependency in depends_on {
                    if !definition.steps.iter().any(|s| s.step_id == *dependency) {
                        return Err(WorkflowServiceError::InvalidTemplate(
                            format!("Step {} depends on non-existent step: {}", step.step_id, dependency)
                        ));
                    }
                }
            }
        }

        // Validate parameters
        for param in &definition.parameters {
            if param.name.is_empty() {
                return Err(WorkflowServiceError::InvalidTemplate(
                    "Parameter name cannot be empty".to_string()
                ));
            }
        }

        Ok(())
    }

    fn validate_template_parameters(&self, template: &WorkflowTemplate, parameters: &HashMap<String, serde_json::Value>) -> WorkflowServiceResult<()> {
        // Check required parameters
        for param in &template.definition.parameters {
            if param.required && !parameters.contains_key(&param.name) {
                return Err(WorkflowServiceError::MissingParameter(
                    format!("Required parameter '{}' is missing", param.name)
                ));
            }

            // Validate parameter types if provided
            if let Some(value) = parameters.get(&param.name) {
                if !self.validate_parameter_type(value, &param.parameter_type) {
                    return Err(WorkflowServiceError::InvalidParameter(
                        format!("Parameter '{}' has invalid type", param.name)
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_parameter_type(&self, value: &serde_json::Value, expected_type: &ParameterType) -> bool {
        match expected_type {
            ParameterType::String => value.is_string(),
            ParameterType::Number => value.is_number(),
            ParameterType::Boolean => value.is_boolean(),
            ParameterType::Array => value.is_array(),
            ParameterType::Object => value.is_object(),
            ParameterType::Any => true,
        }
    }
}

/// Template registry for managing workflow templates
pub struct TemplateRegistry {
    // In a real implementation, this would connect to a database
}

impl TemplateRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn register_template(&self, request: &CreateTemplateRequest, analysis: crate::templates::PatternAnalysisResult) -> WorkflowServiceResult<TemplateRegistration> {
        // Mock implementation
        Ok(TemplateRegistration {
            template_id: Uuid::new_v4().to_string(),
            version: "1.0.0".to_string(),
            success: true,
            created_at: Utc::now(),
            validation_results: ValidationResults {
                valid: true,
                warnings: vec![],
                errors: vec![],
            },
            pattern_analysis: analysis,
        })
    }

    pub async fn get_templates(&self, params: &GetTemplatesParams) -> WorkflowServiceResult<Vec<WorkflowTemplateSummary>> {
        // Mock implementation
        Ok(vec![
            WorkflowTemplateSummary {
                template_id: "template_1".to_string(),
                name: "User Onboarding".to_string(),
                description: "Standard user onboarding workflow".to_string(),
                category: "User Management".to_string(),
                tags: vec!["onboarding".to_string(), "user".to_string()],
                version: "1.0.0".to_string(),
                author: "ADX Core Team".to_string(),
                created_at: Utc::now() - chrono::Duration::days(30),
                updated_at: Utc::now() - chrono::Duration::days(5),
                usage_count: 150,
                success_rate: 94.5,
            },
            WorkflowTemplateSummary {
                template_id: "template_2".to_string(),
                name: "Data Migration".to_string(),
                description: "Template for data migration workflows".to_string(),
                category: "Data Management".to_string(),
                tags: vec!["migration".to_string(), "data".to_string()],
                version: "2.1.0".to_string(),
                author: "Data Team".to_string(),
                created_at: Utc::now() - chrono::Duration::days(60),
                updated_at: Utc::now() - chrono::Duration::days(10),
                usage_count: 75,
                success_rate: 89.2,
            },
        ])
    }

    pub async fn get_template(&self, template_id: &str) -> WorkflowServiceResult<WorkflowTemplate> {
        // Mock implementation
        Ok(WorkflowTemplate {
            template_id: template_id.to_string(),
            name: "User Onboarding".to_string(),
            description: "Standard user onboarding workflow template".to_string(),
            category: "User Management".to_string(),
            tags: vec!["onboarding".to_string(), "user".to_string()],
            version: "1.0.0".to_string(),
            author: "ADX Core Team".to_string(),
            created_at: Utc::now() - chrono::Duration::days(30),
            updated_at: Utc::now() - chrono::Duration::days(5),
            definition: TemplateDefinition {
                steps: vec![
                    TemplateStep {
                        step_id: "validate_user".to_string(),
                        step_type: StepType::Activity,
                        name: "Validate User Data".to_string(),
                        description: "Validate user registration data".to_string(),
                        activity_type: Some("validate_user_data".to_string()),
                        parameters: HashMap::new(),
                        depends_on: None,
                        timeout: Some(std::time::Duration::from_secs(30)),
                        retry_policy: Some(RetryPolicy {
                            max_attempts: 3,
                            initial_delay: std::time::Duration::from_secs(1),
                            backoff_multiplier: 2.0,
                        }),
                    },
                    TemplateStep {
                        step_id: "create_account".to_string(),
                        step_type: StepType::Activity,
                        name: "Create User Account".to_string(),
                        description: "Create user account in auth service".to_string(),
                        activity_type: Some("create_user_account".to_string()),
                        parameters: HashMap::new(),
                        depends_on: Some(vec!["validate_user".to_string()]),
                        timeout: Some(std::time::Duration::from_secs(60)),
                        retry_policy: Some(RetryPolicy {
                            max_attempts: 3,
                            initial_delay: std::time::Duration::from_secs(2),
                            backoff_multiplier: 2.0,
                        }),
                    },
                ],
                parameters: vec![
                    TemplateParameter {
                        name: "user_email".to_string(),
                        description: "User email address".to_string(),
                        parameter_type: ParameterType::String,
                        required: true,
                        default_value: None,
                        validation_rules: vec!["email_format".to_string()],
                    },
                    TemplateParameter {
                        name: "send_welcome_email".to_string(),
                        description: "Whether to send welcome email".to_string(),
                        parameter_type: ParameterType::Boolean,
                        required: false,
                        default_value: Some(serde_json::json!(true)),
                        validation_rules: vec![],
                    },
                ],
                outputs: vec![
                    TemplateOutput {
                        name: "user_id".to_string(),
                        description: "Created user ID".to_string(),
                        output_type: ParameterType::String,
                    },
                ],
                error_handling: ErrorHandling {
                    default_retry_policy: RetryPolicy {
                        max_attempts: 3,
                        initial_delay: std::time::Duration::from_secs(1),
                        backoff_multiplier: 2.0,
                    },
                    compensation_steps: vec![],
                },
            },
            usage_stats: TemplateUsageStats {
                total_uses: 150,
                active_workflows: 25,
                success_rate: 94.5,
                average_duration: std::time::Duration::from_secs(300),
                usage_by_tenant: HashMap::new(),
                usage_trends: vec![],
                last_used: Some(Utc::now() - chrono::Duration::hours(2)),
            },
        })
    }

    pub async fn update_template(&self, request: &UpdateTemplateRequest) -> WorkflowServiceResult<TemplateUpdateResult> {
        Ok(TemplateUpdateResult {
            success: true,
            updated_at: Utc::now(),
            new_version: "1.1.0".to_string(),
            changes_summary: vec!["Updated description".to_string()],
        })
    }

    pub async fn delete_template(&self, template_id: &str, force: bool) -> WorkflowServiceResult<TemplateDeletionResult> {
        Ok(TemplateDeletionResult {
            success: true,
            deleted_at: Utc::now(),
        })
    }

    pub async fn get_active_workflows_using_template(&self, template_id: &str) -> WorkflowServiceResult<Vec<String>> {
        // Mock implementation
        Ok(vec!["workflow_1".to_string(), "workflow_2".to_string()])
    }

    pub async fn get_categories(&self) -> WorkflowServiceResult<Vec<String>> {
        Ok(vec![
            "User Management".to_string(),
            "Data Management".to_string(),
            "Integration".to_string(),
            "Compliance".to_string(),
        ])
    }

    pub async fn get_tags(&self) -> WorkflowServiceResult<Vec<String>> {
        Ok(vec![
            "onboarding".to_string(),
            "migration".to_string(),
            "user".to_string(),
            "data".to_string(),
            "integration".to_string(),
        ])
    }

    pub async fn get_template_usage(&self, template_id: &str) -> WorkflowServiceResult<TemplateUsageStats> {
        Ok(TemplateUsageStats {
            total_uses: 150,
            active_workflows: 25,
            success_rate: 94.5,
            average_duration: std::time::Duration::from_secs(300),
            usage_by_tenant: HashMap::new(),
            usage_trends: vec![],
            last_used: Some(Utc::now() - chrono::Duration::hours(2)),
        })
    }
}

/// Pattern analysis service
pub struct PatternAnalyzer {
    // In a real implementation, this would analyze workflow patterns
}

impl PatternAnalyzer {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn analyze_template(&self, definition: &TemplateDefinition) -> WorkflowServiceResult<PatternAnalysisResult> {
        Ok(PatternAnalysisResult {
            patterns_found: vec![
                WorkflowPattern {
                    pattern_type: PatternType::Sequential,
                    description: "Sequential step execution".to_string(),
                    confidence: 0.95,
                    steps_involved: vec!["validate_user".to_string(), "create_account".to_string()],
                },
            ],
            template_suggestions: vec![],
            optimization_opportunities: vec![
                "Consider parallel execution for independent steps".to_string(),
            ],
            reusability_score: 0.85,
            confidence_score: 0.92,
            extracted_parameters: vec![],
        })
    }

    pub async fn analyze_workflow_patterns(&self, params: &PatternAnalysisParams) -> WorkflowServiceResult<PatternAnalysisResult> {
        Ok(PatternAnalysisResult {
            patterns_found: vec![
                WorkflowPattern {
                    pattern_type: PatternType::Sequential,
                    description: "Common sequential pattern".to_string(),
                    confidence: 0.88,
                    steps_involved: vec!["step1".to_string(), "step2".to_string()],
                },
            ],
            template_suggestions: vec![
                TemplateSuggestion {
                    suggested_name: "Common User Flow".to_string(),
                    description: "Template for common user operations".to_string(),
                    confidence: 0.82,
                    potential_savings: "30% development time".to_string(),
                },
            ],
            optimization_opportunities: vec![
                "Combine similar validation steps".to_string(),
            ],
            reusability_score: 0.78,
            confidence_score: 0.85,
            extracted_parameters: vec![],
        })
    }

    pub async fn analyze_workflows(&self, workflow_ids: &[String]) -> WorkflowServiceResult<PatternAnalysisResult> {
        Ok(PatternAnalysisResult {
            patterns_found: vec![],
            template_suggestions: vec![],
            optimization_opportunities: vec![],
            reusability_score: 0.75,
            confidence_score: 0.80,
            extracted_parameters: vec![
                TemplateParameter {
                    name: "user_id".to_string(),
                    description: "User identifier".to_string(),
                    parameter_type: ParameterType::String,
                    required: true,
                    default_value: None,
                    validation_rules: vec![],
                },
            ],
        })
    }
}

/// Template generation service
pub struct TemplateGenerator {
    // In a real implementation, this would generate templates
}

impl TemplateGenerator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn generate_workflow(&self, template: &WorkflowTemplate, request: &CreateFromTemplateRequest) -> WorkflowServiceResult<WorkflowDefinition> {
        // Mock implementation - would generate actual workflow definition
        Ok(WorkflowDefinition {
            workflow_id: format!("{}_{}", request.workflow_name, Uuid::new_v4()),
            workflow_type: template.name.clone(),
            version: template.version.clone(),
            steps: template.definition.steps.clone(),
            parameters: request.parameters.clone(),
        })
    }

    pub async fn generate_template_from_patterns(&self, analysis: &PatternAnalysisResult) -> WorkflowServiceResult<TemplateDefinition> {
        // Mock implementation
        Ok(TemplateDefinition {
            steps: vec![],
            parameters: analysis.extracted_parameters.clone(),
            outputs: vec![],
            error_handling: ErrorHandling {
                default_retry_policy: RetryPolicy {
                    max_attempts: 3,
                    initial_delay: std::time::Duration::from_secs(1),
                    backoff_multiplier: 2.0,
                },
                compensation_steps: vec![],
            },
        })
    }
}

// Data structures for templates

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTemplateRequest {
    pub template_name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub template_definition: TemplateDefinition,
    pub parameters: Vec<TemplateParameter>,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTemplateResponse {
    pub template_id: String,
    pub template_name: String,
    pub version: String,
    pub created: bool,
    pub created_at: DateTime<Utc>,
    pub validation_results: ValidationResults,
    pub pattern_analysis: PatternAnalysisResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTemplatesParams {
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub author: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetTemplatesResponse {
    pub templates: Vec<WorkflowTemplateSummary>,
    pub total_count: u32,
    pub categories: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub version: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub definition: TemplateDefinition,
    pub usage_stats: TemplateUsageStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowTemplateSummary {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub version: String,
    pub author: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub usage_count: u32,
    pub success_rate: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateDefinition {
    pub steps: Vec<TemplateStep>,
    pub parameters: Vec<TemplateParameter>,
    pub outputs: Vec<TemplateOutput>,
    pub error_handling: ErrorHandling,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateStep {
    pub step_id: String,
    pub step_type: StepType,
    pub name: String,
    pub description: String,
    pub activity_type: Option<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub depends_on: Option<Vec<String>>,
    pub timeout: Option<std::time::Duration>,
    pub retry_policy: Option<RetryPolicy>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum StepType {
    Activity,
    SubWorkflow,
    Condition,
    Parallel,
    Loop,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub parameter_type: ParameterType,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub validation_rules: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Any,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateOutput {
    pub name: String,
    pub description: String,
    pub output_type: ParameterType,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ErrorHandling {
    pub default_retry_policy: RetryPolicy,
    pub compensation_steps: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RetryPolicy {
    pub max_attempts: u32,
    pub initial_delay: std::time::Duration,
    pub backoff_multiplier: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFromTemplateRequest {
    pub template_id: String,
    pub workflow_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub tenant_id: String,
    pub user_id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateFromTemplateResponse {
    pub workflow_id: String,
    pub template_id: String,
    pub workflow_name: String,
    pub workflow_definition: WorkflowDefinition,
    pub parameters_used: HashMap<String, serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub workflow_id: String,
    pub workflow_type: String,
    pub version: String,
    pub steps: Vec<TemplateStep>,
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTemplateRequest {
    pub template_id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub template_definition: Option<TemplateDefinition>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTemplateResponse {
    pub template_id: String,
    pub updated: bool,
    pub updated_at: DateTime<Utc>,
    pub new_version: String,
    pub changes_summary: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTemplateResponse {
    pub template_id: String,
    pub deleted: bool,
    pub deleted_at: DateTime<Utc>,
    pub affected_workflows: u32,
    pub cleanup_performed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternAnalysisParams {
    pub tenant_id: Option<String>,
    pub workflow_types: Option<Vec<String>>,
    pub time_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub min_occurrences: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternAnalysisResponse {
    pub analysis_id: String,
    pub patterns_found: Vec<WorkflowPattern>,
    pub template_suggestions: Vec<TemplateSuggestion>,
    pub optimization_opportunities: Vec<String>,
    pub reusability_score: f64,
    pub analyzed_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PatternAnalysisResult {
    pub patterns_found: Vec<WorkflowPattern>,
    pub template_suggestions: Vec<TemplateSuggestion>,
    pub optimization_opportunities: Vec<String>,
    pub reusability_score: f64,
    pub confidence_score: f64,
    pub extracted_parameters: Vec<TemplateParameter>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WorkflowPattern {
    pub pattern_type: PatternType,
    pub description: String,
    pub confidence: f64,
    pub steps_involved: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PatternType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
    ErrorHandling,
    Compensation,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TemplateSuggestion {
    pub suggested_name: String,
    pub description: String,
    pub confidence: f64,
    pub potential_savings: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTemplateRequest {
    pub template_name: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<String>,
    pub workflow_ids: Vec<String>,
    pub author: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateTemplateResponse {
    pub template_id: String,
    pub template_name: String,
    pub generated: bool,
    pub generated_at: DateTime<Utc>,
    pub source_workflows: Vec<String>,
    pub pattern_analysis: PatternAnalysisResult,
    pub confidence_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateUsageResponse {
    pub template_id: String,
    pub total_uses: u32,
    pub active_workflows: u32,
    pub success_rate: f64,
    pub average_duration: std::time::Duration,
    pub usage_by_tenant: HashMap<String, u32>,
    pub usage_trends: Vec<UsageTrend>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TemplateUsageStats {
    pub total_uses: u32,
    pub active_workflows: u32,
    pub success_rate: f64,
    pub average_duration: std::time::Duration,
    pub usage_by_tenant: HashMap<String, u32>,
    pub usage_trends: Vec<UsageTrend>,
    pub last_used: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UsageTrend {
    pub date: DateTime<Utc>,
    pub usage_count: u32,
}

// Internal data structures

#[derive(Debug)]
pub struct TemplateRegistration {
    pub template_id: String,
    pub version: String,
    pub success: bool,
    pub created_at: DateTime<Utc>,
    pub validation_results: ValidationResults,
    pub pattern_analysis: PatternAnalysisResult,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationResults {
    pub valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct TemplateUpdateResult {
    pub success: bool,
    pub updated_at: DateTime<Utc>,
    pub new_version: String,
    pub changes_summary: Vec<String>,
}

#[derive(Debug)]
pub struct TemplateDeletionResult {
    pub success: bool,
    pub deleted_at: DateTime<Utc>,
}