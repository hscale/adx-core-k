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

/// Workflow versioning and migration management service
pub struct WorkflowVersionManager {
    config: Arc<WorkflowServiceConfig>,
    version_registry: Arc<VersionRegistry>,
    migration_engine: Arc<MigrationEngine>,
    compatibility_checker: Arc<CompatibilityChecker>,
}

impl WorkflowVersionManager {
    pub fn new(config: Arc<WorkflowServiceConfig>) -> Self {
        let version_registry = Arc::new(VersionRegistry::new());
        let migration_engine = Arc::new(MigrationEngine::new());
        let compatibility_checker = Arc::new(CompatibilityChecker::new());

        Self {
            config,
            version_registry,
            migration_engine,
            compatibility_checker,
        }
    }

    /// Register a new workflow version
    pub async fn register_workflow_version(&self, request: RegisterVersionRequest) -> WorkflowServiceResult<RegisterVersionResponse> {
        info!("Registering workflow version: {} v{}", request.workflow_type, request.version);

        // Validate version format
        self.validate_version_format(&request.version)?;

        // Check compatibility with existing versions
        let compatibility = self.compatibility_checker.check_compatibility(&request).await?;

        // Register the version
        let registration = self.version_registry.register_version(&request, compatibility).await?;

        Ok(RegisterVersionResponse {
            workflow_type: request.workflow_type,
            version: request.version,
            registered: registration.success,
            registered_at: registration.registered_at,
            compatibility_info: registration.compatibility_info,
            migration_required: registration.migration_required,
            breaking_changes: registration.breaking_changes,
        })
    }

    /// Get available versions for a workflow type
    pub async fn get_workflow_versions(&self, workflow_type: &str) -> WorkflowServiceResult<WorkflowVersionsResponse> {
        info!("Getting versions for workflow type: {}", workflow_type);

        let versions = self.version_registry.get_versions(workflow_type).await?;

        Ok(WorkflowVersionsResponse {
            workflow_type: workflow_type.to_string(),
            versions,
            current_version: self.version_registry.get_current_version(workflow_type).await?,
            deprecated_versions: self.version_registry.get_deprecated_versions(workflow_type).await?,
        })
    }

    /// Migrate workflows to a new version
    pub async fn migrate_workflows(&self, request: MigrateWorkflowsRequest) -> WorkflowServiceResult<MigrateWorkflowsResponse> {
        info!("Migrating workflows from {} v{} to v{}", 
            request.workflow_type, request.from_version, request.to_version);

        // Validate migration path
        let migration_plan = self.create_migration_plan(&request).await?;

        // Execute migration
        let migration_result = self.migration_engine.execute_migration(&migration_plan).await?;

        Ok(MigrateWorkflowsResponse {
            migration_id: migration_result.migration_id,
            workflow_type: request.workflow_type,
            from_version: request.from_version,
            to_version: request.to_version,
            total_workflows: migration_result.total_workflows,
            migrated_workflows: migration_result.migrated_workflows,
            failed_migrations: migration_result.failed_migrations,
            migration_status: migration_result.status,
            started_at: migration_result.started_at,
            completed_at: migration_result.completed_at,
            errors: migration_result.errors,
        })
    }

    /// Get migration status
    pub async fn get_migration_status(&self, migration_id: &str) -> WorkflowServiceResult<MigrationStatusResponse> {
        info!("Getting migration status for: {}", migration_id);

        let status = self.migration_engine.get_migration_status(migration_id).await?;

        Ok(MigrationStatusResponse {
            migration_id: migration_id.to_string(),
            status: status.status,
            progress: status.progress,
            total_workflows: status.total_workflows,
            processed_workflows: status.processed_workflows,
            successful_migrations: status.successful_migrations,
            failed_migrations: status.failed_migrations,
            current_batch: status.current_batch,
            estimated_completion: status.estimated_completion,
            errors: status.errors,
            started_at: status.started_at,
            updated_at: status.updated_at,
        })
    }

    /// Rollback a migration
    pub async fn rollback_migration(&self, request: RollbackMigrationRequest) -> WorkflowServiceResult<RollbackMigrationResponse> {
        warn!("Rolling back migration: {} with reason: {}", request.migration_id, request.reason);

        let rollback_result = self.migration_engine.rollback_migration(&request).await?;

        Ok(RollbackMigrationResponse {
            migration_id: request.migration_id,
            rollback_id: rollback_result.rollback_id,
            rolled_back: rollback_result.success,
            rollback_started_at: rollback_result.started_at,
            rollback_completed_at: rollback_result.completed_at,
            workflows_rolled_back: rollback_result.workflows_rolled_back,
            message: rollback_result.message,
        })
    }

    /// Deprecate a workflow version
    pub async fn deprecate_version(&self, request: DeprecateVersionRequest) -> WorkflowServiceResult<DeprecateVersionResponse> {
        info!("Deprecating workflow version: {} v{}", request.workflow_type, request.version);

        let deprecation_result = self.version_registry.deprecate_version(&request).await?;

        Ok(DeprecateVersionResponse {
            workflow_type: request.workflow_type,
            version: request.version,
            deprecated: deprecation_result.success,
            deprecated_at: deprecation_result.deprecated_at,
            sunset_date: request.sunset_date,
            migration_deadline: request.migration_deadline,
            affected_workflows: deprecation_result.affected_workflows,
        })
    }

    /// Get version compatibility matrix
    pub async fn get_compatibility_matrix(&self, workflow_type: &str) -> WorkflowServiceResult<CompatibilityMatrixResponse> {
        info!("Getting compatibility matrix for workflow type: {}", workflow_type);

        let matrix = self.compatibility_checker.get_compatibility_matrix(workflow_type).await?;

        Ok(CompatibilityMatrixResponse {
            workflow_type: workflow_type.to_string(),
            compatibility_matrix: matrix.compatibility_matrix,
            breaking_changes: matrix.breaking_changes,
            migration_paths: matrix.migration_paths,
            recommendations: matrix.recommendations,
        })
    }

    // Private helper methods

    fn validate_version_format(&self, version: &str) -> WorkflowServiceResult<()> {
        // Validate semantic versioning format (e.g., "1.2.3")
        let parts: Vec<&str> = version.split('.').collect();
        if parts.len() != 3 {
            return Err(WorkflowServiceError::InvalidVersion(
                format!("Version must be in format 'major.minor.patch', got: {}", version)
            ));
        }

        for part in parts {
            if part.parse::<u32>().is_err() {
                return Err(WorkflowServiceError::InvalidVersion(
                    format!("Version parts must be numeric, got: {}", version)
                ));
            }
        }

        Ok(())
    }

    async fn create_migration_plan(&self, request: &MigrateWorkflowsRequest) -> WorkflowServiceResult<MigrationPlan> {
        // Analyze differences between versions
        let version_diff = self.version_registry.compare_versions(
            &request.workflow_type,
            &request.from_version,
            &request.to_version,
        ).await?;

        // Create migration steps
        let migration_steps = self.create_migration_steps(&version_diff)?;

        // Estimate migration complexity and time
        let complexity = self.estimate_migration_complexity(&version_diff);
        let estimated_duration = self.estimate_migration_duration(&request, &complexity);

        Ok(MigrationPlan {
            migration_id: Uuid::new_v4().to_string(),
            workflow_type: request.workflow_type.clone(),
            from_version: request.from_version.clone(),
            to_version: request.to_version.clone(),
            migration_steps,
            complexity,
            estimated_duration,
            batch_size: request.batch_size.unwrap_or(100),
            dry_run: request.dry_run.unwrap_or(false),
            rollback_enabled: request.enable_rollback.unwrap_or(true),
        })
    }

    fn create_migration_steps(&self, version_diff: &VersionDiff) -> WorkflowServiceResult<Vec<MigrationStep>> {
        let mut steps = Vec::new();

        // Add schema migration steps
        if !version_diff.schema_changes.is_empty() {
            steps.push(MigrationStep {
                step_id: "schema_migration".to_string(),
                step_type: MigrationStepType::SchemaUpdate,
                description: "Update workflow schema".to_string(),
                required: true,
                rollback_supported: true,
                estimated_duration: std::time::Duration::from_secs(30),
            });
        }

        // Add data transformation steps
        if !version_diff.data_transformations.is_empty() {
            steps.push(MigrationStep {
                step_id: "data_transformation".to_string(),
                step_type: MigrationStepType::DataTransformation,
                description: "Transform workflow data".to_string(),
                required: true,
                rollback_supported: true,
                estimated_duration: std::time::Duration::from_secs(120),
            });
        }

        // Add activity mapping steps
        if !version_diff.activity_changes.is_empty() {
            steps.push(MigrationStep {
                step_id: "activity_mapping".to_string(),
                step_type: MigrationStepType::ActivityMapping,
                description: "Map activities to new version".to_string(),
                required: true,
                rollback_supported: false,
                estimated_duration: std::time::Duration::from_secs(60),
            });
        }

        // Add validation step
        steps.push(MigrationStep {
            step_id: "validation".to_string(),
            step_type: MigrationStepType::Validation,
            description: "Validate migrated workflows".to_string(),
            required: true,
            rollback_supported: false,
            estimated_duration: std::time::Duration::from_secs(45),
        });

        Ok(steps)
    }

    fn estimate_migration_complexity(&self, version_diff: &VersionDiff) -> MigrationComplexity {
        let mut complexity_score = 0;

        // Schema changes add complexity
        complexity_score += version_diff.schema_changes.len() * 2;

        // Breaking changes add significant complexity
        complexity_score += version_diff.breaking_changes.len() * 5;

        // Data transformations add complexity
        complexity_score += version_diff.data_transformations.len() * 3;

        match complexity_score {
            0..=5 => MigrationComplexity::Low,
            6..=15 => MigrationComplexity::Medium,
            16..=30 => MigrationComplexity::High,
            _ => MigrationComplexity::Critical,
        }
    }

    fn estimate_migration_duration(&self, request: &MigrateWorkflowsRequest, complexity: &MigrationComplexity) -> std::time::Duration {
        let base_duration = match complexity {
            MigrationComplexity::Low => std::time::Duration::from_secs(300),      // 5 minutes
            MigrationComplexity::Medium => std::time::Duration::from_secs(900),   // 15 minutes
            MigrationComplexity::High => std::time::Duration::from_secs(1800),    // 30 minutes
            MigrationComplexity::Critical => std::time::Duration::from_secs(3600), // 1 hour
        };

        // Adjust based on number of workflows (assuming 100 workflows per minute)
        let workflow_factor = (request.workflow_ids.as_ref().map(|ids| ids.len()).unwrap_or(1000) as f64 / 100.0).ceil() as u64;
        
        base_duration + std::time::Duration::from_secs(workflow_factor * 60)
    }
}

/// Version registry for managing workflow versions
pub struct VersionRegistry {
    // In a real implementation, this would connect to a database
}

impl VersionRegistry {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn register_version(&self, request: &RegisterVersionRequest, compatibility: CompatibilityInfo) -> WorkflowServiceResult<VersionRegistration> {
        // Mock implementation
        Ok(VersionRegistration {
            success: true,
            registered_at: Utc::now(),
            compatibility_info: compatibility,
            migration_required: false,
            breaking_changes: vec![],
        })
    }

    pub async fn get_versions(&self, workflow_type: &str) -> WorkflowServiceResult<Vec<WorkflowVersionInfo>> {
        // Mock implementation
        Ok(vec![
            WorkflowVersionInfo {
                version: "1.0.0".to_string(),
                status: VersionStatus::Active,
                registered_at: Utc::now() - chrono::Duration::days(30),
                deprecated_at: None,
                sunset_date: None,
                active_workflows: 150,
                description: "Initial version".to_string(),
                breaking_changes: vec![],
            },
            WorkflowVersionInfo {
                version: "1.1.0".to_string(),
                status: VersionStatus::Active,
                registered_at: Utc::now() - chrono::Duration::days(15),
                deprecated_at: None,
                sunset_date: None,
                active_workflows: 75,
                description: "Added retry improvements".to_string(),
                breaking_changes: vec![],
            },
            WorkflowVersionInfo {
                version: "2.0.0".to_string(),
                status: VersionStatus::Beta,
                registered_at: Utc::now() - chrono::Duration::days(5),
                deprecated_at: None,
                sunset_date: None,
                active_workflows: 10,
                description: "Major refactor with new activity structure".to_string(),
                breaking_changes: vec![
                    "Activity signatures changed".to_string(),
                    "Workflow input format updated".to_string(),
                ],
            },
        ])
    }

    pub async fn get_current_version(&self, workflow_type: &str) -> WorkflowServiceResult<String> {
        Ok("1.1.0".to_string())
    }

    pub async fn get_deprecated_versions(&self, workflow_type: &str) -> WorkflowServiceResult<Vec<String>> {
        Ok(vec![])
    }

    pub async fn compare_versions(&self, workflow_type: &str, from_version: &str, to_version: &str) -> WorkflowServiceResult<VersionDiff> {
        // Mock implementation
        Ok(VersionDiff {
            from_version: from_version.to_string(),
            to_version: to_version.to_string(),
            schema_changes: vec![
                SchemaChange {
                    field: "user_data.email".to_string(),
                    change_type: SchemaChangeType::FieldAdded,
                    description: "Added email validation".to_string(),
                    breaking: false,
                },
            ],
            breaking_changes: vec![],
            data_transformations: vec![],
            activity_changes: vec![
                ActivityChange {
                    activity_name: "validate_user".to_string(),
                    change_type: ActivityChangeType::SignatureChanged,
                    description: "Added email parameter".to_string(),
                    breaking: false,
                },
            ],
        })
    }

    pub async fn deprecate_version(&self, request: &DeprecateVersionRequest) -> WorkflowServiceResult<DeprecationResult> {
        Ok(DeprecationResult {
            success: true,
            deprecated_at: Utc::now(),
            affected_workflows: 25,
        })
    }
}

/// Migration execution engine
pub struct MigrationEngine {
    // In a real implementation, this would manage migration execution
}

impl MigrationEngine {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn execute_migration(&self, plan: &MigrationPlan) -> WorkflowServiceResult<MigrationResult> {
        info!("Executing migration plan: {}", plan.migration_id);

        // Mock implementation
        Ok(MigrationResult {
            migration_id: plan.migration_id.clone(),
            total_workflows: 100,
            migrated_workflows: 95,
            failed_migrations: 5,
            status: MigrationStatus::Completed,
            started_at: Utc::now() - chrono::Duration::minutes(30),
            completed_at: Some(Utc::now()),
            errors: vec![
                "Failed to migrate workflow_123: schema validation error".to_string(),
            ],
        })
    }

    pub async fn get_migration_status(&self, migration_id: &str) -> WorkflowServiceResult<MigrationStatusInfo> {
        Ok(MigrationStatusInfo {
            status: MigrationStatus::Completed,
            progress: 95.0,
            total_workflows: 100,
            processed_workflows: 100,
            successful_migrations: 95,
            failed_migrations: 5,
            current_batch: 10,
            estimated_completion: None,
            errors: vec![],
            started_at: Utc::now() - chrono::Duration::minutes(30),
            updated_at: Utc::now(),
        })
    }

    pub async fn rollback_migration(&self, request: &RollbackMigrationRequest) -> WorkflowServiceResult<RollbackResult> {
        Ok(RollbackResult {
            rollback_id: Uuid::new_v4().to_string(),
            success: true,
            started_at: Utc::now(),
            completed_at: Some(Utc::now() + chrono::Duration::minutes(15)),
            workflows_rolled_back: 95,
            message: "Migration rollback completed successfully".to_string(),
        })
    }
}

/// Compatibility checking service
pub struct CompatibilityChecker {
    // In a real implementation, this would analyze compatibility
}

impl CompatibilityChecker {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_compatibility(&self, request: &RegisterVersionRequest) -> WorkflowServiceResult<CompatibilityInfo> {
        Ok(CompatibilityInfo {
            is_compatible: true,
            compatibility_level: CompatibilityLevel::Backward,
            breaking_changes: vec![],
            warnings: vec![],
            migration_required: false,
        })
    }

    pub async fn get_compatibility_matrix(&self, workflow_type: &str) -> WorkflowServiceResult<CompatibilityMatrix> {
        Ok(CompatibilityMatrix {
            compatibility_matrix: vec![
                ("1.0.0".to_string(), "1.1.0".to_string(), CompatibilityLevel::Backward),
                ("1.1.0".to_string(), "2.0.0".to_string(), CompatibilityLevel::None),
            ],
            breaking_changes: HashMap::new(),
            migration_paths: vec![
                MigrationPath {
                    from_version: "1.0.0".to_string(),
                    to_version: "2.0.0".to_string(),
                    direct_migration: false,
                    intermediate_versions: vec!["1.1.0".to_string()],
                    complexity: MigrationComplexity::High,
                },
            ],
            recommendations: vec![
                "Migrate to 1.1.0 before upgrading to 2.0.0".to_string(),
            ],
        })
    }
}

// Data structures for versioning

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterVersionRequest {
    pub workflow_type: String,
    pub version: String,
    pub description: String,
    pub schema: serde_json::Value,
    pub breaking_changes: Vec<String>,
    pub migration_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterVersionResponse {
    pub workflow_type: String,
    pub version: String,
    pub registered: bool,
    pub registered_at: DateTime<Utc>,
    pub compatibility_info: CompatibilityInfo,
    pub migration_required: bool,
    pub breaking_changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowVersionsResponse {
    pub workflow_type: String,
    pub versions: Vec<WorkflowVersionInfo>,
    pub current_version: String,
    pub deprecated_versions: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowVersionInfo {
    pub version: String,
    pub status: VersionStatus,
    pub registered_at: DateTime<Utc>,
    pub deprecated_at: Option<DateTime<Utc>>,
    pub sunset_date: Option<DateTime<Utc>>,
    pub active_workflows: u32,
    pub description: String,
    pub breaking_changes: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum VersionStatus {
    Active,
    Beta,
    Deprecated,
    Sunset,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateWorkflowsRequest {
    pub workflow_type: String,
    pub from_version: String,
    pub to_version: String,
    pub workflow_ids: Option<Vec<String>>, // If None, migrate all
    pub batch_size: Option<u32>,
    pub dry_run: Option<bool>,
    pub enable_rollback: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrateWorkflowsResponse {
    pub migration_id: String,
    pub workflow_type: String,
    pub from_version: String,
    pub to_version: String,
    pub total_workflows: u32,
    pub migrated_workflows: u32,
    pub failed_migrations: u32,
    pub migration_status: MigrationStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationStatusResponse {
    pub migration_id: String,
    pub status: MigrationStatus,
    pub progress: f64,
    pub total_workflows: u32,
    pub processed_workflows: u32,
    pub successful_migrations: u32,
    pub failed_migrations: u32,
    pub current_batch: u32,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub errors: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MigrationStatus {
    Pending,
    Running,
    Completed,
    Failed,
    RolledBack,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackMigrationRequest {
    pub migration_id: String,
    pub reason: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RollbackMigrationResponse {
    pub migration_id: String,
    pub rollback_id: String,
    pub rolled_back: bool,
    pub rollback_started_at: DateTime<Utc>,
    pub rollback_completed_at: Option<DateTime<Utc>>,
    pub workflows_rolled_back: u32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeprecateVersionRequest {
    pub workflow_type: String,
    pub version: String,
    pub reason: String,
    pub sunset_date: Option<DateTime<Utc>>,
    pub migration_deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeprecateVersionResponse {
    pub workflow_type: String,
    pub version: String,
    pub deprecated: bool,
    pub deprecated_at: DateTime<Utc>,
    pub sunset_date: Option<DateTime<Utc>>,
    pub migration_deadline: Option<DateTime<Utc>>,
    pub affected_workflows: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompatibilityMatrixResponse {
    pub workflow_type: String,
    pub compatibility_matrix: Vec<(String, String, CompatibilityLevel)>,
    pub breaking_changes: HashMap<String, Vec<String>>,
    pub migration_paths: Vec<MigrationPath>,
    pub recommendations: Vec<String>,
}

// Internal data structures

#[derive(Debug, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    pub is_compatible: bool,
    pub compatibility_level: CompatibilityLevel,
    pub breaking_changes: Vec<String>,
    pub warnings: Vec<String>,
    pub migration_required: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum CompatibilityLevel {
    Backward,    // Fully backward compatible
    Forward,     // Forward compatible only
    None,        // No compatibility
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VersionDiff {
    pub from_version: String,
    pub to_version: String,
    pub schema_changes: Vec<SchemaChange>,
    pub breaking_changes: Vec<String>,
    pub data_transformations: Vec<DataTransformation>,
    pub activity_changes: Vec<ActivityChange>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SchemaChange {
    pub field: String,
    pub change_type: SchemaChangeType,
    pub description: String,
    pub breaking: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SchemaChangeType {
    FieldAdded,
    FieldRemoved,
    FieldTypeChanged,
    FieldRenamed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataTransformation {
    pub field: String,
    pub transformation: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityChange {
    pub activity_name: String,
    pub change_type: ActivityChangeType,
    pub description: String,
    pub breaking: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ActivityChangeType {
    SignatureChanged,
    ActivityAdded,
    ActivityRemoved,
    ActivityRenamed,
}

#[derive(Debug)]
pub struct MigrationPlan {
    pub migration_id: String,
    pub workflow_type: String,
    pub from_version: String,
    pub to_version: String,
    pub migration_steps: Vec<MigrationStep>,
    pub complexity: MigrationComplexity,
    pub estimated_duration: std::time::Duration,
    pub batch_size: u32,
    pub dry_run: bool,
    pub rollback_enabled: bool,
}

#[derive(Debug)]
pub struct MigrationStep {
    pub step_id: String,
    pub step_type: MigrationStepType,
    pub description: String,
    pub required: bool,
    pub rollback_supported: bool,
    pub estimated_duration: std::time::Duration,
}

#[derive(Debug)]
pub enum MigrationStepType {
    SchemaUpdate,
    DataTransformation,
    ActivityMapping,
    Validation,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum MigrationComplexity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug)]
pub struct VersionRegistration {
    pub success: bool,
    pub registered_at: DateTime<Utc>,
    pub compatibility_info: CompatibilityInfo,
    pub migration_required: bool,
    pub breaking_changes: Vec<String>,
}

#[derive(Debug)]
pub struct MigrationResult {
    pub migration_id: String,
    pub total_workflows: u32,
    pub migrated_workflows: u32,
    pub failed_migrations: u32,
    pub status: MigrationStatus,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub errors: Vec<String>,
}

#[derive(Debug)]
pub struct MigrationStatusInfo {
    pub status: MigrationStatus,
    pub progress: f64,
    pub total_workflows: u32,
    pub processed_workflows: u32,
    pub successful_migrations: u32,
    pub failed_migrations: u32,
    pub current_batch: u32,
    pub estimated_completion: Option<DateTime<Utc>>,
    pub errors: Vec<String>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug)]
pub struct RollbackResult {
    pub rollback_id: String,
    pub success: bool,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub workflows_rolled_back: u32,
    pub message: String,
}

#[derive(Debug)]
pub struct DeprecationResult {
    pub success: bool,
    pub deprecated_at: DateTime<Utc>,
    pub affected_workflows: u32,
}

#[derive(Debug)]
pub struct CompatibilityMatrix {
    pub compatibility_matrix: Vec<(String, String, CompatibilityLevel)>,
    pub breaking_changes: HashMap<String, Vec<String>>,
    pub migration_paths: Vec<MigrationPath>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MigrationPath {
    pub from_version: String,
    pub to_version: String,
    pub direct_migration: bool,
    pub intermediate_versions: Vec<String>,
    pub complexity: MigrationComplexity,
}