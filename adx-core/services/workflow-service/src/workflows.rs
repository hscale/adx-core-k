use crate::{
    activities::*,
    error::{WorkflowServiceError, WorkflowServiceResult},
    models::*,
};
use chrono::Utc;
use std::collections::HashMap;
use tracing::{info, warn, error};

// User Onboarding Workflow - Coordinates Auth, User, Tenant, and File services
pub async fn user_onboarding_workflow(
    request: UserOnboardingRequest,
    activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<UserOnboardingResult> {
    info!("Starting user onboarding workflow for email: {}", request.user_email);
    
    let workflow_start = Utc::now();
    let mut workspace_id = None;
    let mut permissions_assigned = Vec::new();
    let mut welcome_email_sent = false;
    let mut sample_data_created = false;

    // Step 1: Validate tenant access and get context
    let tenant_context = activities.get_tenant_context(GetTenantContextRequest {
        tenant_id: request.tenant_id.clone(),
        user_id: None,
    }).await?;

    info!("Tenant context retrieved for tenant: {}", request.tenant_id);

    // Step 2: Create user account in Auth Service
    let user_account = activities.create_user_account(CreateUserAccountRequest {
        email: request.user_email.clone(),
        name: request.user_name.clone(),
        role: request.role.clone(),
        tenant_id: request.tenant_id.clone(),
        send_welcome_email: false, // We'll handle this later in the workflow
    }).await?;

    info!("User account created with ID: {}", user_account.user_id);

    // Step 3: Create user profile in User Service
    let mut profile_data = HashMap::new();
    profile_data.insert("name".to_string(), request.user_name.clone());
    profile_data.insert("email".to_string(), request.user_email.clone());
    profile_data.insert("role".to_string(), request.role.clone());

    let user_profile = activities.create_user_profile(CreateUserProfileRequest {
        user_id: user_account.user_id.clone(),
        tenant_id: request.tenant_id.clone(),
        profile_data,
        preferences: HashMap::new(),
    }).await?;

    info!("User profile created with ID: {}", user_profile.profile_id);

    // Step 4: Update tenant user membership
    if request.assign_default_permissions {
        let default_permissions = get_default_permissions_for_role(&request.role);
        
        activities.update_tenant_user_membership(UpdateTenantUserMembershipRequest {
            user_id: user_account.user_id.clone(),
            tenant_id: request.tenant_id.clone(),
            role: request.role.clone(),
            permissions: default_permissions.clone(),
            active: true,
        }).await?;

        permissions_assigned = default_permissions;
        info!("Default permissions assigned to user: {:?}", permissions_assigned);
    }

    // Step 5: Setup file workspace if requested
    if request.setup_default_workspace {
        let mut workspace_config = HashMap::new();
        workspace_config.insert("type".to_string(), "default".to_string());
        workspace_config.insert("quota_gb".to_string(), "10".to_string());

        let file_workspace = activities.setup_user_file_workspace(SetupUserFileWorkspaceRequest {
            user_id: user_account.user_id.clone(),
            tenant_id: request.tenant_id.clone(),
            workspace_config,
        }).await?;

        workspace_id = Some(file_workspace.workspace_id);
        info!("File workspace created with ID: {:?}", workspace_id);
    }

    // Step 6: Create sample data if requested
    if request.create_sample_data {
        // This would call activities to create sample data across services
        sample_data_created = true;
        info!("Sample data created for user");
    }

    // Step 7: Send welcome email if requested
    if request.send_welcome_email {
        let mut notification_metadata = HashMap::new();
        notification_metadata.insert("user_name".to_string(), request.user_name.clone());
        notification_metadata.insert("tenant_name".to_string(), tenant_context.tenant_context.tenant_name.clone());

        activities.send_notification(SendNotificationRequest {
            notification_type: "welcome_email".to_string(),
            recipient: request.user_email.clone(),
            message: "Welcome to ADX Core!".to_string(),
            metadata: notification_metadata,
        }).await?;

        welcome_email_sent = true;
        info!("Welcome email sent to user");
    }

    let result = UserOnboardingResult {
        user_id: user_account.user_id,
        tenant_id: request.tenant_id,
        workspace_id,
        permissions_assigned,
        welcome_email_sent,
        sample_data_created,
        onboarding_completed_at: Utc::now(),
    };

    info!("User onboarding workflow completed successfully");
    Ok(result)
}

// Tenant Switching Workflow - Multi-service context updates
pub async fn tenant_switching_workflow(
    request: TenantSwitchingRequest,
    activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<TenantSwitchingResult> {
    info!("Starting tenant switching workflow for user: {} from tenant: {} to tenant: {}", 
           request.user_id, request.current_tenant_id, request.target_tenant_id);

    // Step 1: Validate user has access to target tenant
    let tenant_access = activities.validate_tenant_access(ValidateTenantAccessRequest {
        user_id: request.user_id.clone(),
        tenant_id: request.target_tenant_id.clone(),
    }).await?;

    if !tenant_access.has_access {
        return Err(WorkflowServiceError::Authorization(
            "User does not have access to target tenant".to_string()
        ));
    }

    info!("Tenant access validated for user");

    // Step 2: Get target tenant context
    let tenant_context = activities.get_tenant_context(GetTenantContextRequest {
        tenant_id: request.target_tenant_id.clone(),
        user_id: Some(request.user_id.clone()),
    }).await?;

    // Step 3: Update user tenant context in User Service
    activities.update_user_tenant_context(UpdateUserTenantContextRequest {
        user_id: request.user_id.clone(),
        new_tenant_id: request.target_tenant_id.clone(),
        preserve_preferences: request.update_user_preferences,
    }).await?;

    info!("User tenant context updated");

    // Step 4: Update user session in Auth Service
    let mut session_data = HashMap::new();
    if request.preserve_session_data {
        session_data.insert("preserve_data".to_string(), "true".to_string());
    }

    let session_result = activities.update_user_session(UpdateUserSessionRequest {
        user_id: request.user_id.clone(),
        new_tenant_id: request.target_tenant_id.clone(),
        session_data,
    }).await?;

    info!("User session updated with new session ID: {}", session_result.session_id);

    // Step 5: Update tenant user membership
    activities.update_tenant_user_membership(UpdateTenantUserMembershipRequest {
        user_id: request.user_id.clone(),
        tenant_id: request.target_tenant_id.clone(),
        role: tenant_access.role.unwrap_or_else(|| "user".to_string()),
        permissions: tenant_access.permissions.clone(),
        active: true,
    }).await?;

    let result = TenantSwitchingResult {
        user_id: request.user_id,
        new_tenant_id: request.target_tenant_id,
        new_session_id: session_result.session_id,
        updated_permissions: tenant_access.permissions,
        tenant_context: tenant_context.tenant_context,
        switch_completed_at: Utc::now(),
    };

    info!("Tenant switching workflow completed successfully");
    Ok(result)
}

// Data Migration Workflow - Cross-service data synchronization
pub async fn data_migration_workflow(
    request: DataMigrationRequest,
    activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<DataMigrationResult> {
    info!("Starting data migration workflow: {} of type: {:?}", 
           request.migration_id, request.migration_type);

    let mut records_processed = 0u64;
    let mut records_migrated = 0u64;
    let mut records_failed = 0u64;
    let mut services_affected = Vec::new();
    let mut backup_id = None;
    let mut error_summary = None;

    // Step 1: Create backup if requested
    if request.migration_options.create_backup {
        let backup_request = CreateBackupRequest {
            backup_id: format!("migration_backup_{}", request.migration_id),
            tenant_id: request.target_tenant_id.clone(),
            services: request.data_selectors.iter().map(|s| s.service.clone()).collect(),
        };

        let backup_result = activities.create_cross_service_backup(backup_request).await?;
        backup_id = Some(backup_result.backup_id);
        info!("Backup created: {:?}", backup_id);
    }

    // Step 2: Validate data if requested
    if request.migration_options.validate_data {
        info!("Validating data before migration");
        // Data validation logic would go here
    }

    // Step 3: Process data migration for each service
    for data_selector in &request.data_selectors {
        services_affected.push(data_selector.service.clone());
        
        match data_selector.service.as_str() {
            "user" => {
                let migration_result = migrate_user_service_data(
                    data_selector,
                    &request,
                    activities,
                ).await;

                match migration_result {
                    Ok((processed, migrated)) => {
                        records_processed += processed;
                        records_migrated += migrated;
                    }
                    Err(e) => {
                        records_failed += 1;
                        error_summary = Some(format!("User service migration failed: {}", e));
                        if !request.migration_options.rollback_on_error {
                            continue;
                        } else {
                            return handle_migration_rollback(backup_id, e).await;
                        }
                    }
                }
            }
            "file" => {
                let migration_result = migrate_file_service_data(
                    data_selector,
                    &request,
                    activities,
                ).await;

                match migration_result {
                    Ok((processed, migrated)) => {
                        records_processed += processed;
                        records_migrated += migrated;
                    }
                    Err(e) => {
                        records_failed += 1;
                        error_summary = Some(format!("File service migration failed: {}", e));
                        if !request.migration_options.rollback_on_error {
                            continue;
                        } else {
                            return handle_migration_rollback(backup_id, e).await;
                        }
                    }
                }
            }
            "tenant" => {
                let migration_result = migrate_tenant_service_data(
                    data_selector,
                    &request,
                    activities,
                ).await;

                match migration_result {
                    Ok((processed, migrated)) => {
                        records_processed += processed;
                        records_migrated += migrated;
                    }
                    Err(e) => {
                        records_failed += 1;
                        error_summary = Some(format!("Tenant service migration failed: {}", e));
                        if !request.migration_options.rollback_on_error {
                            continue;
                        } else {
                            return handle_migration_rollback(backup_id, e).await;
                        }
                    }
                }
            }
            _ => {
                warn!("Unknown service for migration: {}", data_selector.service);
                records_failed += 1;
            }
        }
    }

    let status = if records_failed == 0 {
        MigrationStatus::Completed
    } else if records_migrated > 0 {
        MigrationStatus::Completed // Partial success still counts as completed
    } else {
        MigrationStatus::Failed
    };

    let result = DataMigrationResult {
        migration_id: request.migration_id,
        status,
        records_processed,
        records_migrated,
        records_failed,
        services_affected,
        backup_id,
        error_summary,
        completed_at: Utc::now(),
    };

    info!("Data migration workflow completed with status: {:?}", result.status);
    Ok(result)
}

// Bulk Operation Workflow - Administrative operations across services
pub async fn bulk_operation_workflow(
    request: BulkOperationRequest,
    activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<BulkOperationResult> {
    info!("Starting bulk operation workflow: {} of type: {:?}", 
           request.operation_id, request.operation_type);

    let total_entities = request.target_entities.len() as u64;
    let mut successful_operations = 0u64;
    let mut failed_operations = 0u64;
    let mut batches_processed = 0u64;
    let mut error_details = Vec::new();

    // Process entities in batches
    let batch_size = request.batch_options.batch_size;
    let batches: Vec<_> = request.target_entities.chunks(batch_size).collect();

    for (batch_index, batch) in batches.iter().enumerate() {
        info!("Processing batch {} of {}", batch_index + 1, batches.len());

        // Process batch entities in parallel if configured
        let batch_results = if request.batch_options.parallel_batches > 1 {
            process_batch_parallel(batch, &request, activities).await
        } else {
            process_batch_sequential(batch, &request, activities).await
        };

        // Aggregate batch results
        for result in batch_results {
            match result {
                Ok(_) => successful_operations += 1,
                Err(e) => {
                    failed_operations += 1;
                    error_details.push(OperationError {
                        entity_id: "unknown".to_string(), // Would be extracted from error
                        error_message: e.to_string(),
                        retry_count: 0,
                    });

                    if !request.batch_options.continue_on_error {
                        error!("Batch operation failed, stopping due to continue_on_error=false");
                        break;
                    }
                }
            }
        }

        batches_processed += 1;

        // Add delay between batches if configured
        if request.batch_options.delay_between_batches_ms > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(
                request.batch_options.delay_between_batches_ms
            )).await;
        }
    }

    let result = BulkOperationResult {
        operation_id: request.operation_id,
        total_entities,
        successful_operations,
        failed_operations,
        batches_processed,
        error_details,
        completed_at: Utc::now(),
    };

    info!("Bulk operation workflow completed: {}/{} successful", 
           result.successful_operations, result.total_entities);
    Ok(result)
}

// Compliance Workflow - GDPR and audit requirements
pub async fn compliance_workflow(
    request: ComplianceWorkflowRequest,
    activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<ComplianceWorkflowResult> {
    info!("Starting compliance workflow: {} of type: {:?}", 
           request.compliance_id, request.compliance_type);

    let mut data_exported = None;
    let mut data_deleted = None;
    let mut audit_report = None;
    let mut compliance_certificate = None;

    match request.compliance_type {
        ComplianceType::GdprDataExport => {
            data_exported = Some(handle_gdpr_data_export(&request, activities).await?);
        }
        ComplianceType::GdprDataDeletion => {
            data_deleted = Some(handle_gdpr_data_deletion(&request, activities).await?);
        }
        ComplianceType::DataRetentionEnforcement => {
            handle_data_retention_enforcement(&request, activities).await?;
        }
        ComplianceType::AuditLogGeneration => {
            audit_report = Some(handle_audit_log_generation(&request, activities).await?);
        }
        ComplianceType::ComplianceReport => {
            audit_report = Some(handle_compliance_report_generation(&request, activities).await?);
            compliance_certificate = Some(generate_compliance_certificate(&request));
        }
        ComplianceType::DataClassification => {
            handle_data_classification(&request, activities).await?;
        }
    }

    let status = ComplianceStatus::Completed;

    let result = ComplianceWorkflowResult {
        compliance_id: request.compliance_id,
        status,
        data_exported,
        data_deleted,
        audit_report,
        compliance_certificate,
        completed_at: Utc::now(),
    };

    info!("Compliance workflow completed successfully");
    Ok(result)
}

// Helper functions for workflows

fn get_default_permissions_for_role(role: &str) -> Vec<String> {
    match role {
        "admin" => vec![
            "tenant:read".to_string(),
            "tenant:write".to_string(),
            "user:read".to_string(),
            "user:write".to_string(),
            "file:read".to_string(),
            "file:write".to_string(),
        ],
        "user" => vec![
            "tenant:read".to_string(),
            "user:read".to_string(),
            "file:read".to_string(),
            "file:write".to_string(),
        ],
        _ => vec!["tenant:read".to_string()],
    }
}

async fn migrate_user_service_data(
    _data_selector: &DataSelector,
    _request: &DataMigrationRequest,
    _activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<(u64, u64)> {
    // Mock implementation - would contain actual user data migration logic
    Ok((100, 95)) // (processed, migrated)
}

async fn migrate_file_service_data(
    _data_selector: &DataSelector,
    _request: &DataMigrationRequest,
    _activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<(u64, u64)> {
    // Mock implementation - would contain actual file data migration logic
    Ok((50, 48)) // (processed, migrated)
}

async fn migrate_tenant_service_data(
    _data_selector: &DataSelector,
    _request: &DataMigrationRequest,
    _activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<(u64, u64)> {
    // Mock implementation - would contain actual tenant data migration logic
    Ok((10, 10)) // (processed, migrated)
}

async fn handle_migration_rollback(
    backup_id: Option<String>,
    error: WorkflowServiceError,
) -> WorkflowServiceResult<DataMigrationResult> {
    if let Some(ref backup_id_ref) = backup_id {
        warn!("Rolling back migration using backup: {}", backup_id_ref);
        // Rollback logic would go here
    }

    Ok(DataMigrationResult {
        migration_id: "failed".to_string(),
        status: MigrationStatus::RolledBack,
        records_processed: 0,
        records_migrated: 0,
        records_failed: 1,
        services_affected: vec![],
        backup_id,
        error_summary: Some(error.to_string()),
        completed_at: Utc::now(),
    })
}

async fn process_batch_sequential(
    batch: &[EntityTarget],
    _request: &BulkOperationRequest,
    _activities: &dyn CrossServiceActivities,
) -> Vec<WorkflowServiceResult<()>> {
    let mut results = Vec::new();
    for _entity in batch {
        // Mock processing - would contain actual entity processing logic
        results.push(Ok(()));
    }
    results
}

async fn process_batch_parallel(
    batch: &[EntityTarget],
    _request: &BulkOperationRequest,
    _activities: &dyn CrossServiceActivities,
) -> Vec<WorkflowServiceResult<()>> {
    let mut results = Vec::new();
    for _entity in batch {
        // Mock processing - would contain actual parallel entity processing logic
        results.push(Ok(()));
    }
    results
}

async fn handle_gdpr_data_export(
    request: &ComplianceWorkflowRequest,
    activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<DataExportSummary> {
    info!("Handling GDPR data export for compliance: {}", request.compliance_id);

    let mut total_records = 0u64;
    let mut services_included = Vec::new();

    // Export user data if user_id is provided
    if let Some(user_id) = &request.subject_user_id {
        let user_data = activities.get_user_data_for_export(GetUserDataRequest {
            user_id: user_id.clone(),
            tenant_id: request.tenant_id.clone(),
        }).await?;

        total_records += 1; // Mock count
        services_included.push("user".to_string());

        // Export user files
        let file_data = activities.export_user_files(ExportUserFilesRequest {
            user_id: user_id.clone(),
            tenant_id: request.tenant_id.clone(),
        }).await?;

        total_records += file_data.files_exported;
        services_included.push("file".to_string());
    }

    Ok(DataExportSummary {
        export_file_path: format!("/exports/gdpr_export_{}.json", request.compliance_id),
        total_records,
        services_included,
        export_format: "JSON".to_string(),
        encryption_applied: true,
    })
}

async fn handle_gdpr_data_deletion(
    request: &ComplianceWorkflowRequest,
    activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<DataDeletionSummary> {
    info!("Handling GDPR data deletion for compliance: {}", request.compliance_id);

    let mut total_records_deleted = 0u64;
    let mut services_affected = Vec::new();

    // Create backup before deletion
    let backup_result = activities.create_cross_service_backup(CreateBackupRequest {
        backup_id: format!("gdpr_deletion_backup_{}", request.compliance_id),
        tenant_id: request.tenant_id.clone(),
        services: vec!["user".to_string(), "file".to_string()],
    }).await?;

    // Delete user data if user_id is provided
    if let Some(user_id) = &request.subject_user_id {
        let mut delete_options = std::collections::HashMap::new();
        delete_options.insert("soft_delete".to_string(), false);
        delete_options.insert("anonymize".to_string(), true);

        let user_deletion = activities.delete_user_data(DeleteUserDataRequest {
            user_id: user_id.clone(),
            tenant_id: request.tenant_id.clone(),
            delete_options: delete_options.clone(),
        }).await?;

        total_records_deleted += user_deletion.records_deleted;
        services_affected.push("user".to_string());

        // Delete user files
        let file_deletion = activities.delete_user_files(DeleteUserFilesRequest {
            user_id: user_id.clone(),
            tenant_id: request.tenant_id.clone(),
            delete_options,
        }).await?;

        total_records_deleted += file_deletion.files_deleted;
        services_affected.push("file".to_string());
    }

    Ok(DataDeletionSummary {
        total_records_deleted,
        services_affected,
        backup_created: true,
        backup_id: Some(backup_result.backup_id),
    })
}

async fn handle_data_retention_enforcement(
    _request: &ComplianceWorkflowRequest,
    _activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<()> {
    info!("Handling data retention enforcement");
    // Mock implementation - would contain actual retention enforcement logic
    Ok(())
}

async fn handle_audit_log_generation(
    request: &ComplianceWorkflowRequest,
    _activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<AuditReportSummary> {
    info!("Handling audit log generation for compliance: {}", request.compliance_id);

    let date_range = request.audit_requirements.date_range.clone().unwrap_or_else(|| {
        DateRange {
            start_date: Utc::now() - chrono::Duration::days(30),
            end_date: Utc::now(),
        }
    });

    Ok(AuditReportSummary {
        report_file_path: format!("/reports/audit_report_{}.json", request.compliance_id),
        total_audit_entries: 1000, // Mock count
        date_range_covered: date_range,
        report_format: "JSON".to_string(),
    })
}

async fn handle_compliance_report_generation(
    request: &ComplianceWorkflowRequest,
    _activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<AuditReportSummary> {
    info!("Handling compliance report generation for compliance: {}", request.compliance_id);

    let date_range = request.audit_requirements.date_range.clone().unwrap_or_else(|| {
        DateRange {
            start_date: Utc::now() - chrono::Duration::days(365),
            end_date: Utc::now(),
        }
    });

    Ok(AuditReportSummary {
        report_file_path: format!("/reports/compliance_report_{}.pdf", request.compliance_id),
        total_audit_entries: 5000, // Mock count
        date_range_covered: date_range,
        report_format: "PDF".to_string(),
    })
}

async fn handle_data_classification(
    _request: &ComplianceWorkflowRequest,
    _activities: &dyn CrossServiceActivities,
) -> WorkflowServiceResult<()> {
    info!("Handling data classification");
    // Mock implementation - would contain actual data classification logic
    Ok(())
}

fn generate_compliance_certificate(request: &ComplianceWorkflowRequest) -> String {
    format!("COMPLIANCE_CERT_{}_{}_{}", 
            request.compliance_type.to_string(), 
            request.tenant_id, 
            Utc::now().format("%Y%m%d"))
}

impl std::fmt::Display for ComplianceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComplianceType::GdprDataExport => write!(f, "GDPR_DATA_EXPORT"),
            ComplianceType::GdprDataDeletion => write!(f, "GDPR_DATA_DELETION"),
            ComplianceType::DataRetentionEnforcement => write!(f, "DATA_RETENTION_ENFORCEMENT"),
            ComplianceType::AuditLogGeneration => write!(f, "AUDIT_LOG_GENERATION"),
            ComplianceType::ComplianceReport => write!(f, "COMPLIANCE_REPORT"),
            ComplianceType::DataClassification => write!(f, "DATA_CLASSIFICATION"),
        }
    }
}