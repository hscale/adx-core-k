use crate::{
    activities::{CrossServiceActivities, CrossServiceActivitiesImpl, CreateBackupRequest, RestoreBackupRequest},
    config::WorkflowServiceConfig,
    error::{WorkflowServiceError, WorkflowServiceResult},
    management::{WorkflowManager, CancelWorkflowRequest, RetryWorkflowRequest, TerminateWorkflowRequest, BulkWorkflowOperationRequest},
    models::*,
    monitoring::{WorkflowMonitor, AnalyticsParams, TimeRange},
    server::TenantContext,
    templates::{WorkflowTemplateManager, CreateTemplateRequest, GetTemplatesParams, CreateFromTemplateRequest, UpdateTemplateRequest, PatternAnalysisParams, GenerateTemplateRequest},
    versioning::{WorkflowVersionManager, RegisterVersionRequest, MigrateWorkflowsRequest, RollbackMigrationRequest, DeprecateVersionRequest},
    workflows::*,
};
use axum::{
    extract::{Extension, Path, Query},
    response::Json,
    http::StatusCode,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing::{info, warn, error};
use uuid::Uuid;

// Workflow initiation handlers

pub async fn start_user_onboarding_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<UserOnboardingRequest>,
) -> WorkflowServiceResult<Json<WorkflowStartResponse>> {
    info!("Starting user onboarding workflow for email: {}", request.user_email);
    
    let workflow_id = format!("user_onboarding_{}", Uuid::new_v4());
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    
    // For now, execute workflow synchronously
    // In a real implementation, this would be submitted to Temporal
    let result = user_onboarding_workflow(request, &activities).await?;
    
    Ok(Json(WorkflowStartResponse {
        workflow_id: workflow_id.clone(),
        status: "completed".to_string(),
        result: Some(serde_json::to_value(result)?),
        started_at: Utc::now(),
    }))
}

pub async fn start_tenant_switching_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<TenantSwitchingRequest>,
) -> WorkflowServiceResult<Json<WorkflowStartResponse>> {
    info!("Starting tenant switching workflow for user: {}", request.user_id);
    
    let workflow_id = format!("tenant_switching_{}", Uuid::new_v4());
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    
    // Execute workflow
    let result = tenant_switching_workflow(request, &activities).await?;
    
    Ok(Json(WorkflowStartResponse {
        workflow_id: workflow_id.clone(),
        status: "completed".to_string(),
        result: Some(serde_json::to_value(result)?),
        started_at: Utc::now(),
    }))
}

pub async fn start_data_migration_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<DataMigrationRequest>,
) -> WorkflowServiceResult<Json<WorkflowStartResponse>> {
    info!("Starting data migration workflow: {}", request.migration_id);
    
    let workflow_id = format!("data_migration_{}", Uuid::new_v4());
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    
    // For large migrations, this would be submitted to Temporal as async
    // For now, execute synchronously
    let result = data_migration_workflow(request, &activities).await?;
    
    Ok(Json(WorkflowStartResponse {
        workflow_id: workflow_id.clone(),
        status: "completed".to_string(),
        result: Some(serde_json::to_value(result)?),
        started_at: Utc::now(),
    }))
}

pub async fn start_bulk_operation_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<BulkOperationRequest>,
) -> WorkflowServiceResult<Json<WorkflowStartResponse>> {
    info!("Starting bulk operation workflow: {}", request.operation_id);
    
    let workflow_id = format!("bulk_operation_{}", Uuid::new_v4());
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    
    // Execute workflow
    let result = bulk_operation_workflow(request, &activities).await?;
    
    Ok(Json(WorkflowStartResponse {
        workflow_id: workflow_id.clone(),
        status: "completed".to_string(),
        result: Some(serde_json::to_value(result)?),
        started_at: Utc::now(),
    }))
}

pub async fn start_compliance_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<ComplianceWorkflowRequest>,
) -> WorkflowServiceResult<Json<WorkflowStartResponse>> {
    info!("Starting compliance workflow: {}", request.compliance_id);
    
    let workflow_id = format!("compliance_{}", Uuid::new_v4());
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    
    // Execute workflow
    let result = compliance_workflow(request, &activities).await?;
    
    Ok(Json(WorkflowStartResponse {
        workflow_id: workflow_id.clone(),
        status: "completed".to_string(),
        result: Some(serde_json::to_value(result)?),
        started_at: Utc::now(),
    }))
}

// Workflow management handlers

pub async fn get_workflow_status(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
) -> WorkflowServiceResult<Json<WorkflowStatusResponse>> {
    info!("Getting workflow status for: {}", workflow_id);
    
    // In a real implementation, this would query Temporal for workflow status
    // For now, return a mock response
    Ok(Json(WorkflowStatusResponse {
        workflow_id: workflow_id.clone(),
        status: WorkflowExecutionStatus::Completed,
        progress: Some(WorkflowProgressInfo {
            current_step: "completed".to_string(),
            total_steps: 5,
            completed_steps: 5,
            percentage: 100.0,
            estimated_completion: None,
            last_updated: Utc::now(),
            status_message: Some("Workflow completed successfully".to_string()),
        }),
        result: Some(serde_json::json!({
            "status": "success",
            "message": "Workflow completed"
        })),
        error: None,
        started_at: Utc::now() - chrono::Duration::minutes(5),
        updated_at: Utc::now(),
    }))
}

pub async fn cancel_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
) -> WorkflowServiceResult<Json<WorkflowCancelResponse>> {
    info!("Cancelling workflow: {}", workflow_id);
    
    // In a real implementation, this would cancel the Temporal workflow
    Ok(Json(WorkflowCancelResponse {
        workflow_id,
        cancelled: true,
        cancelled_at: Utc::now(),
        message: "Workflow cancellation requested".to_string(),
    }))
}

pub async fn retry_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
) -> WorkflowServiceResult<Json<WorkflowRetryResponse>> {
    info!("Retrying workflow: {}", workflow_id);
    
    // In a real implementation, this would retry the failed Temporal workflow
    Ok(Json(WorkflowRetryResponse {
        workflow_id: workflow_id.clone(),
        new_workflow_id: format!("{}_retry_{}", workflow_id, Uuid::new_v4()),
        retried: true,
        retried_at: Utc::now(),
        message: "Workflow retry initiated".to_string(),
    }))
}

pub async fn list_workflows(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<ListWorkflowsParams>,
) -> WorkflowServiceResult<Json<ListWorkflowsResponse>> {
    info!("Listing workflows for tenant: {}", tenant_context.tenant_id);
    
    // In a real implementation, this would query Temporal for workflows
    let workflows = vec![
        WorkflowSummary {
            workflow_id: "user_onboarding_123".to_string(),
            workflow_type: "user_onboarding".to_string(),
            status: WorkflowExecutionStatus::Completed,
            started_at: Utc::now() - chrono::Duration::hours(2),
            updated_at: Utc::now() - chrono::Duration::hours(1),
            tenant_id: tenant_context.tenant_id.clone(),
            user_id: tenant_context.user_id.clone(),
        },
        WorkflowSummary {
            workflow_id: "tenant_switching_456".to_string(),
            workflow_type: "tenant_switching".to_string(),
            status: WorkflowExecutionStatus::Running,
            started_at: Utc::now() - chrono::Duration::minutes(30),
            updated_at: Utc::now() - chrono::Duration::minutes(5),
            tenant_id: tenant_context.tenant_id.clone(),
            user_id: tenant_context.user_id.clone(),
        },
    ];
    
    Ok(Json(ListWorkflowsResponse {
        workflows,
        total_count: 2,
        page: params.page.unwrap_or(1),
        page_size: params.page_size.unwrap_or(50),
        has_more: false,
    }))
}

pub async fn get_workflow_history(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<WorkflowHistoryParams>,
) -> WorkflowServiceResult<Json<WorkflowHistoryResponse>> {
    info!("Getting workflow history for tenant: {}", tenant_context.tenant_id);
    
    // In a real implementation, this would query Temporal for workflow history
    let history_events = vec![
        WorkflowHistoryEvent {
            event_id: 1,
            event_type: "WorkflowExecutionStarted".to_string(),
            timestamp: Utc::now() - chrono::Duration::hours(1),
            details: serde_json::json!({
                "workflow_type": "user_onboarding",
                "input": {}
            }),
        },
        WorkflowHistoryEvent {
            event_id: 2,
            event_type: "ActivityTaskScheduled".to_string(),
            timestamp: Utc::now() - chrono::Duration::minutes(55),
            details: serde_json::json!({
                "activity_type": "create_user_account",
                "activity_id": "1"
            }),
        },
        WorkflowHistoryEvent {
            event_id: 3,
            event_type: "WorkflowExecutionCompleted".to_string(),
            timestamp: Utc::now() - chrono::Duration::minutes(50),
            details: serde_json::json!({
                "result": {
                    "user_id": "user_123",
                    "status": "completed"
                }
            }),
        },
    ];
    
    Ok(Json(WorkflowHistoryResponse {
        workflow_id: params.workflow_id.unwrap_or_else(|| "unknown".to_string()),
        events: history_events,
        total_events: 3,
        next_page_token: None,
    }))
}

// Service coordination handlers

pub async fn coordinate_health_check(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<CoordinateHealthCheckRequest>,
) -> WorkflowServiceResult<Json<CoordinateHealthCheckResponse>> {
    info!("Coordinating health check for services: {:?}", request.services);
    
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    let result = activities.coordinate_service_health_check(request.services).await?;
    
    Ok(Json(CoordinateHealthCheckResponse {
        overall_healthy: result.overall_healthy,
        service_results: result.service_results,
        checked_at: result.checked_at,
    }))
}

pub async fn create_cross_service_backup(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<CreateBackupRequest>,
) -> WorkflowServiceResult<Json<CreateBackupResponse>> {
    info!("Creating cross-service backup: {}", request.backup_id);
    
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    let result = activities.create_cross_service_backup(request).await?;
    
    Ok(Json(CreateBackupResponse {
        backup_id: result.backup_id,
        backup_location: result.backup_location,
        services_backed_up: result.services_backed_up,
        backup_size_bytes: result.backup_size_bytes,
        created_at: result.created_at,
    }))
}

pub async fn restore_from_backup(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<RestoreBackupRequest>,
) -> WorkflowServiceResult<Json<RestoreBackupResponse>> {
    info!("Restoring from backup: {}", request.backup_id);
    
    let activities = CrossServiceActivitiesImpl::new((*config).clone());
    let result = activities.restore_from_backup(request).await?;
    
    Ok(Json(RestoreBackupResponse {
        backup_id: result.backup_id,
        services_restored: result.services_restored,
        records_restored: result.records_restored,
        restored_at: result.restored_at,
    }))
}

// Enhanced workflow monitoring handlers

pub async fn get_workflow_status_detailed(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
) -> WorkflowServiceResult<Json<crate::monitoring::WorkflowStatusDetail>> {
    info!("Getting detailed workflow status for: {}", workflow_id);
    
    let monitor = WorkflowMonitor::new(config);
    let status = monitor.get_workflow_status(&workflow_id).await?;
    
    Ok(Json(status))
}

pub async fn get_workflow_analytics(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<HashMap<String, String>>,
) -> WorkflowServiceResult<Json<crate::monitoring::WorkflowAnalytics>> {
    info!("Getting workflow analytics for tenant: {}", tenant_context.tenant_id);
    
    let monitor = WorkflowMonitor::new(config);
    
    // Parse time range from query parameters
    let start_time = params.get("start_time")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now() - chrono::Duration::days(7));
    
    let end_time = params.get("end_time")
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .unwrap_or_else(|| chrono::Utc::now());
    
    let analytics_params = AnalyticsParams {
        time_range: TimeRange { start: start_time, end: end_time },
        workflow_types: params.get("workflow_types").map(|s| s.split(',').map(|t| t.to_string()).collect()),
        tenant_id: Some(tenant_context.tenant_id),
        include_failed: params.get("include_failed").map(|s| s == "true").unwrap_or(true),
    };
    
    let analytics = monitor.get_workflow_analytics(analytics_params).await?;
    
    Ok(Json(analytics))
}

pub async fn get_workflow_health_report(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
) -> WorkflowServiceResult<Json<crate::monitoring::HealthReport>> {
    info!("Getting workflow health report");
    
    let monitor = WorkflowMonitor::new(config);
    let health_report = monitor.monitor_workflow_health().await?;
    
    Ok(Json(health_report))
}

pub async fn get_workflow_debug_info(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
) -> WorkflowServiceResult<Json<crate::monitoring::WorkflowDebugInfo>> {
    info!("Getting debug information for workflow: {}", workflow_id);
    
    let monitor = WorkflowMonitor::new(config);
    let debug_info = monitor.get_workflow_debug_info(&workflow_id).await?;
    
    Ok(Json(debug_info))
}

// Enhanced workflow management handlers

pub async fn cancel_workflow_enhanced(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<CancelWorkflowRequest>,
) -> WorkflowServiceResult<Json<crate::management::CancelWorkflowResponse>> {
    info!("Cancelling workflow with enhanced options: {}", request.workflow_id);
    
    let manager = WorkflowManager::new(config);
    let response = manager.cancel_workflow(request).await?;
    
    Ok(Json(response))
}

pub async fn retry_workflow_enhanced(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<RetryWorkflowRequest>,
) -> WorkflowServiceResult<Json<crate::management::RetryWorkflowResponse>> {
    info!("Retrying workflow with enhanced options: {}", request.workflow_id);
    
    let manager = WorkflowManager::new(config);
    let response = manager.retry_workflow(request).await?;
    
    Ok(Json(response))
}

pub async fn pause_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
    Json(request): Json<HashMap<String, String>>,
) -> WorkflowServiceResult<Json<crate::management::PauseWorkflowResponse>> {
    info!("Pausing workflow: {}", workflow_id);
    
    let manager = WorkflowManager::new(config);
    let reason = request.get("reason").cloned();
    let response = manager.pause_workflow(&workflow_id, reason).await?;
    
    Ok(Json(response))
}

pub async fn resume_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
) -> WorkflowServiceResult<Json<crate::management::ResumeWorkflowResponse>> {
    info!("Resuming workflow: {}", workflow_id);
    
    let manager = WorkflowManager::new(config);
    let response = manager.resume_workflow(&workflow_id).await?;
    
    Ok(Json(response))
}

pub async fn terminate_workflow(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<TerminateWorkflowRequest>,
) -> WorkflowServiceResult<Json<crate::management::TerminateWorkflowResponse>> {
    warn!("Terminating workflow: {}", request.workflow_id);
    
    let manager = WorkflowManager::new(config);
    let response = manager.terminate_workflow(request).await?;
    
    Ok(Json(response))
}

pub async fn get_workflow_management_options(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_id): Path<String>,
) -> WorkflowServiceResult<Json<crate::management::WorkflowManagementOptions>> {
    info!("Getting management options for workflow: {}", workflow_id);
    
    let manager = WorkflowManager::new(config);
    let options = manager.get_workflow_management_options(&workflow_id).await?;
    
    Ok(Json(options))
}

pub async fn bulk_workflow_operation(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<BulkWorkflowOperationRequest>,
) -> WorkflowServiceResult<Json<crate::management::BulkWorkflowOperationResponse>> {
    info!("Performing bulk workflow operation: {:?} on {} workflows", request.operation, request.workflow_ids.len());
    
    let manager = WorkflowManager::new(config);
    let response = manager.bulk_workflow_operation(request).await?;
    
    Ok(Json(response))
}

// Workflow versioning handlers

pub async fn register_workflow_version(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<RegisterVersionRequest>,
) -> WorkflowServiceResult<Json<crate::versioning::RegisterVersionResponse>> {
    info!("Registering workflow version: {} v{}", request.workflow_type, request.version);
    
    let version_manager = WorkflowVersionManager::new(config);
    let response = version_manager.register_workflow_version(request).await?;
    
    Ok(Json(response))
}

pub async fn get_workflow_versions(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_type): Path<String>,
) -> WorkflowServiceResult<Json<crate::versioning::WorkflowVersionsResponse>> {
    info!("Getting versions for workflow type: {}", workflow_type);
    
    let version_manager = WorkflowVersionManager::new(config);
    let response = version_manager.get_workflow_versions(&workflow_type).await?;
    
    Ok(Json(response))
}

pub async fn migrate_workflows(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<MigrateWorkflowsRequest>,
) -> WorkflowServiceResult<Json<crate::versioning::MigrateWorkflowsResponse>> {
    info!("Migrating workflows from {} v{} to v{}", request.workflow_type, request.from_version, request.to_version);
    
    let version_manager = WorkflowVersionManager::new(config);
    let response = version_manager.migrate_workflows(request).await?;
    
    Ok(Json(response))
}

pub async fn get_migration_status(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(migration_id): Path<String>,
) -> WorkflowServiceResult<Json<crate::versioning::MigrationStatusResponse>> {
    info!("Getting migration status for: {}", migration_id);
    
    let version_manager = WorkflowVersionManager::new(config);
    let response = version_manager.get_migration_status(&migration_id).await?;
    
    Ok(Json(response))
}

pub async fn rollback_migration(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<RollbackMigrationRequest>,
) -> WorkflowServiceResult<Json<crate::versioning::RollbackMigrationResponse>> {
    warn!("Rolling back migration: {}", request.migration_id);
    
    let version_manager = WorkflowVersionManager::new(config);
    let response = version_manager.rollback_migration(request).await?;
    
    Ok(Json(response))
}

pub async fn deprecate_version(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<DeprecateVersionRequest>,
) -> WorkflowServiceResult<Json<crate::versioning::DeprecateVersionResponse>> {
    info!("Deprecating workflow version: {} v{}", request.workflow_type, request.version);
    
    let version_manager = WorkflowVersionManager::new(config);
    let response = version_manager.deprecate_version(request).await?;
    
    Ok(Json(response))
}

pub async fn get_compatibility_matrix(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(workflow_type): Path<String>,
) -> WorkflowServiceResult<Json<crate::versioning::CompatibilityMatrixResponse>> {
    info!("Getting compatibility matrix for workflow type: {}", workflow_type);
    
    let version_manager = WorkflowVersionManager::new(config);
    let response = version_manager.get_compatibility_matrix(&workflow_type).await?;
    
    Ok(Json(response))
}

// Workflow template handlers

pub async fn create_workflow_template(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<CreateTemplateRequest>,
) -> WorkflowServiceResult<Json<crate::templates::CreateTemplateResponse>> {
    info!("Creating workflow template: {}", request.template_name);
    
    let template_manager = WorkflowTemplateManager::new(config);
    let response = template_manager.create_template(request).await?;
    
    Ok(Json(response))
}

pub async fn get_workflow_templates(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Query(params): Query<GetTemplatesParams>,
) -> WorkflowServiceResult<Json<crate::templates::GetTemplatesResponse>> {
    info!("Getting workflow templates");
    
    let template_manager = WorkflowTemplateManager::new(config);
    let response = template_manager.get_templates(params).await?;
    
    Ok(Json(response))
}

pub async fn get_workflow_template(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(template_id): Path<String>,
) -> WorkflowServiceResult<Json<crate::templates::WorkflowTemplate>> {
    info!("Getting workflow template: {}", template_id);
    
    let template_manager = WorkflowTemplateManager::new(config);
    let response = template_manager.get_template(&template_id).await?;
    
    Ok(Json(response))
}

pub async fn create_workflow_from_template(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<CreateFromTemplateRequest>,
) -> WorkflowServiceResult<Json<crate::templates::CreateFromTemplateResponse>> {
    info!("Creating workflow from template: {}", request.template_id);
    
    let template_manager = WorkflowTemplateManager::new(config);
    let response = template_manager.create_workflow_from_template(request).await?;
    
    Ok(Json(response))
}

pub async fn update_workflow_template(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<UpdateTemplateRequest>,
) -> WorkflowServiceResult<Json<crate::templates::UpdateTemplateResponse>> {
    info!("Updating workflow template: {}", request.template_id);
    
    let template_manager = WorkflowTemplateManager::new(config);
    let response = template_manager.update_template(request).await?;
    
    Ok(Json(response))
}

pub async fn delete_workflow_template(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(template_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> WorkflowServiceResult<Json<crate::templates::DeleteTemplateResponse>> {
    info!("Deleting workflow template: {}", template_id);
    
    let template_manager = WorkflowTemplateManager::new(config);
    let force = params.get("force").map(|s| s == "true").unwrap_or(false);
    let response = template_manager.delete_template(&template_id, force).await?;
    
    Ok(Json(response))
}

pub async fn analyze_workflow_patterns(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(params): Query<HashMap<String, String>>,
) -> WorkflowServiceResult<Json<crate::templates::PatternAnalysisResponse>> {
    info!("Analyzing workflow patterns for tenant: {}", tenant_context.tenant_id);
    
    let template_manager = WorkflowTemplateManager::new(config);
    
    let analysis_params = PatternAnalysisParams {
        tenant_id: Some(tenant_context.tenant_id),
        workflow_types: params.get("workflow_types").map(|s| s.split(',').map(|t| t.to_string()).collect()),
        time_range: None, // Could be parsed from params
        min_occurrences: params.get("min_occurrences").and_then(|s| s.parse().ok()),
    };
    
    let response = template_manager.analyze_workflow_patterns(analysis_params).await?;
    
    Ok(Json(response))
}

pub async fn generate_template_from_workflows(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Json(request): Json<GenerateTemplateRequest>,
) -> WorkflowServiceResult<Json<crate::templates::GenerateTemplateResponse>> {
    info!("Generating template from {} workflows", request.workflow_ids.len());
    
    let template_manager = WorkflowTemplateManager::new(config);
    let response = template_manager.generate_template_from_workflows(request).await?;
    
    Ok(Json(response))
}

pub async fn get_template_usage(
    Extension(config): Extension<Arc<WorkflowServiceConfig>>,
    Path(template_id): Path<String>,
) -> WorkflowServiceResult<Json<crate::templates::TemplateUsageResponse>> {
    info!("Getting usage statistics for template: {}", template_id);
    
    let template_manager = WorkflowTemplateManager::new(config);
    let response = template_manager.get_template_usage(&template_id).await?;
    
    Ok(Json(response))
}

// Request/Response types

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStartResponse {
    pub workflow_id: String,
    pub status: String,
    pub result: Option<serde_json::Value>,
    pub started_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatusResponse {
    pub workflow_id: String,
    pub status: WorkflowExecutionStatus,
    pub progress: Option<WorkflowProgressInfo>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkflowExecutionStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowProgressInfo {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub estimated_completion: Option<chrono::DateTime<Utc>>,
    pub last_updated: chrono::DateTime<Utc>,
    pub status_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowCancelResponse {
    pub workflow_id: String,
    pub cancelled: bool,
    pub cancelled_at: chrono::DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowRetryResponse {
    pub workflow_id: String,
    pub new_workflow_id: String,
    pub retried: bool,
    pub retried_at: chrono::DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ListWorkflowsParams {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub status: Option<String>,
    pub workflow_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListWorkflowsResponse {
    pub workflows: Vec<WorkflowSummary>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub workflow_id: String,
    pub workflow_type: String,
    pub status: WorkflowExecutionStatus,
    pub started_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub tenant_id: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WorkflowHistoryParams {
    pub workflow_id: Option<String>,
    pub page_token: Option<String>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowHistoryResponse {
    pub workflow_id: String,
    pub events: Vec<WorkflowHistoryEvent>,
    pub total_events: u64,
    pub next_page_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowHistoryEvent {
    pub event_id: u64,
    pub event_type: String,
    pub timestamp: chrono::DateTime<Utc>,
    pub details: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct CoordinateHealthCheckRequest {
    pub services: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CoordinateHealthCheckResponse {
    pub overall_healthy: bool,
    pub service_results: HashMap<String, bool>,
    pub checked_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct CreateBackupResponse {
    pub backup_id: String,
    pub backup_location: String,
    pub services_backed_up: Vec<String>,
    pub backup_size_bytes: u64,
    pub created_at: chrono::DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct RestoreBackupResponse {
    pub backup_id: String,
    pub services_restored: Vec<String>,
    pub records_restored: u64,
    pub restored_at: chrono::DateTime<Utc>,
}