use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

use crate::error::{WorkflowError, ActivityError};
use crate::models::{
    InstallModuleWorkflowInput, InstallModuleWorkflowOutput,
    UpdateModuleWorkflowInput, UpdateModuleWorkflowOutput,
    UninstallModuleWorkflowInput, UninstallModuleWorkflowOutput,
    MarketplaceSyncWorkflowInput, MarketplaceSyncWorkflowOutput,
    SecurityScanWorkflowInput, SecurityScanWorkflowOutput,
};
use crate::activities::ModuleActivities;

// Temporal workflow implementations (using placeholder SDK)
// In production, these would use the actual Temporal Rust SDK

/// Module Installation Workflow
/// Handles the complete installation process with rollback capabilities
pub async fn install_module_workflow(
    input: InstallModuleWorkflowInput,
) -> Result<InstallModuleWorkflowOutput, WorkflowError> {
    let workflow_id = format!("install-module-{}-{}", input.module_id, Uuid::new_v4());
    
    // Step 1: Validate module and tenant permissions
    let validation_result = call_activity(
        ModuleActivities::validate_module_installation,
        ValidateModuleInstallationRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
            tenant_id: input.tenant_id.clone(),
            user_id: input.user_id.clone(),
            force_reinstall: input.force_reinstall,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(30),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(1),
                maximum_interval: Duration::from_secs(10),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::InstallationWorkflowFailed(format!("Validation failed: {}", e)))?;

    if !validation_result.is_valid {
        return Err(WorkflowError::InstallationWorkflowFailed(
            format!("Module validation failed: {:?}", validation_result.errors)
        ));
    }

    let mut compensation_activities = Vec::new();

    // Step 2: Download and verify module package
    let download_result = call_activity(
        ModuleActivities::download_module_package,
        DownloadModulePackageRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
            checksum: validation_result.expected_checksum,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(300), // 5 minutes for download
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(5),
                maximum_interval: Duration::from_secs(30),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::InstallationWorkflowFailed(format!("Download failed: {}", e)))?;

    // Step 3: Security scan
    let security_scan_result = call_activity(
        ModuleActivities::perform_security_scan,
        PerformSecurityScanRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
            package_path: download_result.package_path.clone(),
            deep_scan: true,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(600), // 10 minutes for security scan
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 2,
                initial_interval: Duration::from_secs(10),
                maximum_interval: Duration::from_secs(60),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::InstallationWorkflowFailed(format!("Security scan failed: {}", e)))?;

    if !security_scan_result.passed {
        // Cleanup downloaded package
        let _ = call_activity(
            ModuleActivities::cleanup_package,
            CleanupPackageRequest {
                package_path: download_result.package_path,
            },
            ActivityOptions::default(),
        ).await;

        return Err(WorkflowError::InstallationWorkflowFailed(
            format!("Security scan failed: {} vulnerabilities found", security_scan_result.vulnerabilities.len())
        ));
    }

    // Step 4: Resolve and install dependencies
    let mut installed_dependencies = Vec::new();
    for dependency in &validation_result.dependencies {
        let dep_install_result = call_activity(
            ModuleActivities::install_dependency,
            InstallDependencyRequest {
                dependency_id: dependency.id.clone(),
                version_requirement: dependency.version_requirement.clone(),
                tenant_id: input.tenant_id.clone(),
                optional: dependency.optional,
            },
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(180),
                retry_policy: Some(RetryPolicy {
                    maximum_attempts: 3,
                    initial_interval: Duration::from_secs(2),
                    maximum_interval: Duration::from_secs(20),
                    backoff_coefficient: 2.0,
                }),
            },
        ).await.map_err(|e| {
            // Schedule compensation for already installed dependencies
            for installed_dep in &installed_dependencies {
                compensation_activities.push(CompensationActivity {
                    activity_type: "uninstall_dependency".to_string(),
                    input: serde_json::json!({
                        "dependency_id": installed_dep,
                        "tenant_id": input.tenant_id
                    }),
                });
            }
            WorkflowError::InstallationWorkflowFailed(format!("Dependency installation failed: {}", e))
        })?;

        if dep_install_result.installed {
            installed_dependencies.push(dependency.id.clone());
        }
    }

    // Step 5: Extract and deploy module
    let deployment_result = call_activity(
        ModuleActivities::deploy_module,
        DeployModuleRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
            package_path: download_result.package_path.clone(),
            tenant_id: input.tenant_id.clone(),
            configuration: input.configuration.clone(),
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(300),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 2,
                initial_interval: Duration::from_secs(5),
                maximum_interval: Duration::from_secs(30),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| {
        // Schedule compensation
        compensation_activities.push(CompensationActivity {
            activity_type: "cleanup_package".to_string(),
            input: serde_json::json!({
                "package_path": download_result.package_path
            }),
        });
        
        for installed_dep in &installed_dependencies {
            compensation_activities.push(CompensationActivity {
                activity_type: "uninstall_dependency".to_string(),
                input: serde_json::json!({
                    "dependency_id": installed_dep,
                    "tenant_id": input.tenant_id
                }),
            });
        }
        
        WorkflowError::InstallationWorkflowFailed(format!("Deployment failed: {}", e))
    })?;

    // Step 6: Configure sandbox
    let sandbox_result = call_activity(
        ModuleActivities::configure_sandbox,
        ConfigureSandboxRequest {
            module_id: input.module_id.clone(),
            tenant_id: input.tenant_id.clone(),
            resource_limits: validation_result.resource_limits,
            security_policy: validation_result.security_policy,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(60),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(1),
                maximum_interval: Duration::from_secs(10),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| {
        // Schedule compensation
        compensation_activities.push(CompensationActivity {
            activity_type: "cleanup_deployment".to_string(),
            input: serde_json::json!({
                "deployment_id": deployment_result.deployment_id,
                "tenant_id": input.tenant_id
            }),
        });
        
        WorkflowError::InstallationWorkflowFailed(format!("Sandbox configuration failed: {}", e))
    })?;

    // Step 7: Register module installation
    let installation_result = call_activity(
        ModuleActivities::register_installation,
        RegisterInstallationRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
            tenant_id: input.tenant_id.clone(),
            user_id: input.user_id.clone(),
            deployment_id: deployment_result.deployment_id.clone(),
            configuration: input.configuration.clone(),
            sandbox_config: sandbox_result.sandbox_config,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(30),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(1),
                maximum_interval: Duration::from_secs(10),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::InstallationWorkflowFailed(format!("Registration failed: {}", e)))?;

    // Step 8: Auto-activate if requested
    let mut status = crate::types::ModuleStatus::Installed;
    if input.auto_activate {
        let activation_result = call_activity(
            ModuleActivities::activate_module,
            ActivateModuleRequest {
                installation_id: installation_result.installation_id.clone(),
                module_id: input.module_id.clone(),
                tenant_id: input.tenant_id.clone(),
            },
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(120),
                retry_policy: Some(RetryPolicy {
                    maximum_attempts: 2,
                    initial_interval: Duration::from_secs(2),
                    maximum_interval: Duration::from_secs(20),
                    backoff_coefficient: 2.0,
                }),
            },
        ).await.map_err(|e| WorkflowError::InstallationWorkflowFailed(format!("Activation failed: {}", e)))?;

        if activation_result.activated {
            status = crate::types::ModuleStatus::Active;
        }
    }

    // Step 9: Cleanup temporary files
    let _ = call_activity(
        ModuleActivities::cleanup_package,
        CleanupPackageRequest {
            package_path: download_result.package_path,
        },
        ActivityOptions::default(),
    ).await;

    Ok(InstallModuleWorkflowOutput {
        installation_id: installation_result.installation_id,
        module_id: input.module_id,
        version: input.version,
        status,
        dependencies_installed: installed_dependencies,
        configuration_applied: input.configuration.is_some(),
        sandbox_configured: true,
    })
}

/// Module Update Workflow
/// Handles module updates with backup and rollback capabilities
pub async fn update_module_workflow(
    input: UpdateModuleWorkflowInput,
) -> Result<UpdateModuleWorkflowOutput, WorkflowError> {
    let workflow_id = format!("update-module-{}-{}", input.module_id, Uuid::new_v4());
    
    // Step 1: Validate update request
    let validation_result = call_activity(
        ModuleActivities::validate_module_update,
        ValidateModuleUpdateRequest {
            module_id: input.module_id.clone(),
            current_version: "".to_string(), // Would be fetched from database
            target_version: input.target_version.clone(),
            tenant_id: input.tenant_id.clone(),
        },
        ActivityOptions::default(),
    ).await.map_err(|e| WorkflowError::UpdateWorkflowFailed(format!("Validation failed: {}", e)))?;

    let mut backup_id = None;

    // Step 2: Create backup if requested
    if input.backup_current {
        let backup_result = call_activity(
            ModuleActivities::backup_module,
            BackupModuleRequest {
                module_id: input.module_id.clone(),
                tenant_id: input.tenant_id.clone(),
                backup_type: "pre_update".to_string(),
            },
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(300),
                retry_policy: Some(RetryPolicy {
                    maximum_attempts: 2,
                    initial_interval: Duration::from_secs(5),
                    maximum_interval: Duration::from_secs(30),
                    backoff_coefficient: 2.0,
                }),
            },
        ).await.map_err(|e| WorkflowError::UpdateWorkflowFailed(format!("Backup failed: {}", e)))?;

        backup_id = Some(backup_result.backup_id);
    }

    // Step 3: Download new version
    let download_result = call_activity(
        ModuleActivities::download_module_package,
        DownloadModulePackageRequest {
            module_id: input.module_id.clone(),
            version: input.target_version.clone(),
            checksum: validation_result.expected_checksum,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(300),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(5),
                maximum_interval: Duration::from_secs(30),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::UpdateWorkflowFailed(format!("Download failed: {}", e)))?;

    // Step 4: Perform update with rollback capability
    let update_result = call_activity(
        ModuleActivities::perform_module_update,
        PerformModuleUpdateRequest {
            module_id: input.module_id.clone(),
            target_version: input.target_version.clone(),
            package_path: download_result.package_path.clone(),
            tenant_id: input.tenant_id.clone(),
            migration_scripts: validation_result.migration_scripts,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(600),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 1, // No retry for updates to avoid inconsistent state
                initial_interval: Duration::from_secs(1),
                maximum_interval: Duration::from_secs(1),
                backoff_coefficient: 1.0,
            }),
        },
    ).await.map_err(|e| {
        // If rollback is enabled and we have a backup, initiate rollback
        if input.rollback_on_failure && backup_id.is_some() {
            let _ = spawn_child_workflow(
                rollback_module_workflow,
                RollbackModuleWorkflowInput {
                    module_id: input.module_id.clone(),
                    tenant_id: input.tenant_id.clone(),
                    backup_id: backup_id.unwrap(),
                },
            );
        }
        WorkflowError::UpdateWorkflowFailed(format!("Update failed: {}", e))
    })?;

    // Step 5: Cleanup
    let _ = call_activity(
        ModuleActivities::cleanup_package,
        CleanupPackageRequest {
            package_path: download_result.package_path,
        },
        ActivityOptions::default(),
    ).await;

    Ok(UpdateModuleWorkflowOutput {
        update_id: Uuid::new_v4().to_string(),
        module_id: input.module_id,
        from_version: validation_result.current_version,
        to_version: input.target_version,
        status: crate::types::ModuleStatus::Active,
        backup_id,
        migration_applied: update_result.migration_applied,
    })
}

/// Module Uninstallation Workflow
/// Handles complete module removal with data cleanup
pub async fn uninstall_module_workflow(
    input: UninstallModuleWorkflowInput,
) -> Result<UninstallModuleWorkflowOutput, WorkflowError> {
    let workflow_id = format!("uninstall-module-{}-{}", input.module_id, Uuid::new_v4());
    
    // Step 1: Validate uninstallation
    let validation_result = call_activity(
        ModuleActivities::validate_module_uninstallation,
        ValidateModuleUninstallationRequest {
            module_id: input.module_id.clone(),
            tenant_id: input.tenant_id.clone(),
            force_uninstall: input.force_uninstall,
        },
        ActivityOptions::default(),
    ).await.map_err(|e| WorkflowError::UninstallationWorkflowFailed(format!("Validation failed: {}", e)))?;

    // Step 2: Deactivate module if active
    if validation_result.is_active {
        let _ = call_activity(
            ModuleActivities::deactivate_module,
            DeactivateModuleRequest {
                module_id: input.module_id.clone(),
                tenant_id: input.tenant_id.clone(),
            },
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(120),
                retry_policy: Some(RetryPolicy {
                    maximum_attempts: 2,
                    initial_interval: Duration::from_secs(2),
                    maximum_interval: Duration::from_secs(20),
                    backoff_coefficient: 2.0,
                }),
            },
        ).await;
    }

    // Step 3: Clean up module data if requested
    let mut data_cleaned = false;
    if input.cleanup_data {
        let cleanup_result = call_activity(
            ModuleActivities::cleanup_module_data,
            CleanupModuleDataRequest {
                module_id: input.module_id.clone(),
                tenant_id: input.tenant_id.clone(),
                cleanup_type: "complete".to_string(),
            },
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(300),
                retry_policy: Some(RetryPolicy {
                    maximum_attempts: 2,
                    initial_interval: Duration::from_secs(5),
                    maximum_interval: Duration::from_secs(30),
                    backoff_coefficient: 2.0,
                }),
            },
        ).await.map_err(|e| WorkflowError::UninstallationWorkflowFailed(format!("Data cleanup failed: {}", e)))?;

        data_cleaned = cleanup_result.cleaned;
    }

    // Step 4: Remove module files and configuration
    let removal_result = call_activity(
        ModuleActivities::remove_module_installation,
        RemoveModuleInstallationRequest {
            module_id: input.module_id.clone(),
            tenant_id: input.tenant_id.clone(),
            remove_dependencies: true,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(180),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 2,
                initial_interval: Duration::from_secs(2),
                maximum_interval: Duration::from_secs(20),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::UninstallationWorkflowFailed(format!("Removal failed: {}", e)))?;

    // Step 5: Update installation record
    let _ = call_activity(
        ModuleActivities::update_installation_status,
        UpdateInstallationStatusRequest {
            module_id: input.module_id.clone(),
            tenant_id: input.tenant_id.clone(),
            status: crate::types::ModuleStatus::Failed, // Uninstalled
        },
        ActivityOptions::default(),
    ).await;

    Ok(UninstallModuleWorkflowOutput {
        uninstallation_id: Uuid::new_v4().to_string(),
        module_id: input.module_id,
        status: crate::types::ModuleStatus::Failed, // Represents uninstalled
        data_cleaned,
        dependencies_removed: removal_result.dependencies_removed,
    })
}

/// Marketplace Sync Workflow
/// Synchronizes modules with the marketplace
pub async fn marketplace_sync_workflow(
    input: MarketplaceSyncWorkflowInput,
) -> Result<MarketplaceSyncWorkflowOutput, WorkflowError> {
    let workflow_id = format!("marketplace-sync-{}", Uuid::new_v4());
    
    // Step 1: Fetch marketplace data
    let marketplace_data = call_activity(
        ModuleActivities::fetch_marketplace_data,
        FetchMarketplaceDataRequest {
            sync_type: input.sync_type.clone(),
            module_ids: input.module_ids.clone(),
            force_update: input.force_update,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(300),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(5),
                maximum_interval: Duration::from_secs(30),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::MarketplaceSyncFailed(format!("Fetch failed: {}", e)))?;

    // Step 2: Process updates
    let mut modules_synced = 0;
    let mut modules_updated = 0;
    let mut modules_added = 0;
    let mut modules_removed = 0;
    let mut errors = Vec::new();

    for module_data in marketplace_data.modules {
        let sync_result = call_activity(
            ModuleActivities::sync_module_data,
            SyncModuleDataRequest {
                module_data: module_data.clone(),
                force_update: input.force_update,
            },
            ActivityOptions {
                start_to_close_timeout: Duration::from_secs(60),
                retry_policy: Some(RetryPolicy {
                    maximum_attempts: 2,
                    initial_interval: Duration::from_secs(2),
                    maximum_interval: Duration::from_secs(10),
                    backoff_coefficient: 2.0,
                }),
            },
        ).await;

        match sync_result {
            Ok(result) => {
                modules_synced += 1;
                match result.action {
                    SyncAction::Updated => modules_updated += 1,
                    SyncAction::Added => modules_added += 1,
                    SyncAction::Removed => modules_removed += 1,
                    SyncAction::NoChange => {},
                }
            }
            Err(e) => {
                errors.push(format!("Module {}: {}", module_data.id, e));
            }
        }
    }

    Ok(MarketplaceSyncWorkflowOutput {
        sync_id: Uuid::new_v4().to_string(),
        modules_synced,
        modules_updated,
        modules_added,
        modules_removed,
        errors,
    })
}

/// Security Scan Workflow
/// Performs comprehensive security scanning of modules
pub async fn security_scan_workflow(
    input: SecurityScanWorkflowInput,
) -> Result<SecurityScanWorkflowOutput, WorkflowError> {
    let workflow_id = format!("security-scan-{}-{}", input.module_id, Uuid::new_v4());
    
    // Step 1: Download module for scanning
    let download_result = call_activity(
        ModuleActivities::download_module_for_scan,
        DownloadModuleForScanRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(300),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(5),
                maximum_interval: Duration::from_secs(30),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::SecurityScanWorkflowFailed(format!("Download failed: {}", e)))?;

    // Step 2: Perform security scan
    let scan_result = call_activity(
        ModuleActivities::perform_security_scan,
        PerformSecurityScanRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
            package_path: download_result.package_path.clone(),
            deep_scan: input.deep_scan,
        },
        ActivityOptions {
            start_to_close_timeout: Duration::from_secs(600),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 2,
                initial_interval: Duration::from_secs(10),
                maximum_interval: Duration::from_secs(60),
                backoff_coefficient: 2.0,
            }),
        },
    ).await.map_err(|e| WorkflowError::SecurityScanWorkflowFailed(format!("Scan failed: {}", e)))?;

    // Step 3: Store scan results
    let _ = call_activity(
        ModuleActivities::store_scan_results,
        StoreScanResultsRequest {
            module_id: input.module_id.clone(),
            version: input.version.clone(),
            scan_type: input.scan_type.clone(),
            scan_results: scan_result.clone(),
        },
        ActivityOptions::default(),
    ).await;

    // Step 4: Cleanup
    let _ = call_activity(
        ModuleActivities::cleanup_package,
        CleanupPackageRequest {
            package_path: download_result.package_path,
        },
        ActivityOptions::default(),
    ).await;

    Ok(SecurityScanWorkflowOutput {
        scan_id: Uuid::new_v4().to_string(),
        module_id: input.module_id,
        version: input.version,
        passed: scan_result.passed,
        score: scan_result.score,
        vulnerabilities_count: scan_result.vulnerabilities.len() as u32,
        scan_duration_seconds: scan_result.scan_duration_seconds,
    })
}

// Helper types and functions for workflow activities

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityOptions {
    pub start_to_close_timeout: Duration,
    pub retry_policy: Option<RetryPolicy>,
}

impl Default for ActivityOptions {
    fn default() -> Self {
        Self {
            start_to_close_timeout: Duration::from_secs(60),
            retry_policy: Some(RetryPolicy {
                maximum_attempts: 3,
                initial_interval: Duration::from_secs(1),
                maximum_interval: Duration::from_secs(10),
                backoff_coefficient: 2.0,
            }),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub maximum_attempts: u32,
    pub initial_interval: Duration,
    pub maximum_interval: Duration,
    pub backoff_coefficient: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompensationActivity {
    pub activity_type: String,
    pub input: serde_json::Value,
}

// Placeholder functions for Temporal SDK integration
async fn call_activity<I, O>(
    activity: fn(I) -> Result<O, ActivityError>,
    input: I,
    options: ActivityOptions,
) -> Result<O, ActivityError> {
    // In production, this would use the actual Temporal SDK
    // For now, we'll simulate the activity call
    activity(input)
}

async fn spawn_child_workflow<I, O>(
    workflow: fn(I) -> Result<O, WorkflowError>,
    input: I,
) -> Result<(), WorkflowError> {
    // In production, this would spawn a child workflow
    // For now, we'll simulate it
    let _ = workflow(input)?;
    Ok(())
}

// Request/Response types for activities (these would be defined in activities.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateModuleInstallationRequest {
    pub module_id: String,
    pub version: String,
    pub tenant_id: String,
    pub user_id: String,
    pub force_reinstall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateModuleInstallationResponse {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub expected_checksum: String,
    pub dependencies: Vec<ModuleDependency>,
    pub resource_limits: crate::types::ResourceLimits,
    pub security_policy: crate::types::SecurityPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleDependency {
    pub id: String,
    pub version_requirement: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadModulePackageRequest {
    pub module_id: String,
    pub version: String,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadModulePackageResponse {
    pub package_path: String,
    pub verified: bool,
}

// Additional request/response types would be defined here...