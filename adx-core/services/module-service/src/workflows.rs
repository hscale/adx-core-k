use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use semver::Version;

use crate::{
    ModuleResult, ModuleError, InstallModuleRequest, InstallModuleResult,
    UpdateModuleRequest, UpdateModuleResult, UninstallModuleRequest, UninstallModuleResult,
    ModulePackage, ModuleInstance, ModuleStatus, SecurityScanResult,
};

// Temporal workflow implementations for module operations

/// Module installation workflow with comprehensive error handling and rollback
#[temporal_sdk::workflow]
pub async fn install_module_workflow(
    request: InstallModuleRequest,
) -> Result<InstallModuleResult, ModuleWorkflowError> {
    tracing::info!("Starting module installation workflow for: {}", request.module_id);

    // Step 1: Validate installation prerequisites
    let validation_result = temporal_sdk::workflow::call_activity(
        validate_module_installation,
        ValidateInstallationRequest {
            module_id: request.module_id.clone(),
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
        },
    ).await?;

    if !validation_result.is_valid {
        return Err(ModuleWorkflowError::ValidationFailed(validation_result.errors));
    }

    // Step 2: Resolve and validate dependencies
    let dependencies = temporal_sdk::workflow::call_activity(
        resolve_module_dependencies,
        ResolveDependenciesRequest {
            module_id: request.module_id.clone(),
            version: request.version.clone(),
            tenant_id: request.tenant_id.clone(),
        },
    ).await?;

    // Step 3: Install dependencies recursively
    let mut installed_dependencies = Vec::new();
    for dependency in dependencies.dependencies {
        if !dependency.already_installed {
            let dep_result = temporal_sdk::workflow::call_child_workflow(
                install_module_workflow,
                InstallModuleRequest {
                    module_id: dependency.module_id.clone(),
                    version: Some(dependency.version),
                    tenant_id: request.tenant_id.clone(),
                    user_id: request.user_id.clone(),
                    configuration: None,
                    auto_activate: false,
                },
            ).await?;
            installed_dependencies.push(dep_result.instance_id);
        }
    }

    // Step 4: Download and validate module package
    let package = temporal_sdk::workflow::call_activity(
        download_module_package,
        DownloadPackageRequest {
            module_id: request.module_id.clone(),
            version: request.version.clone(),
            tenant_id: request.tenant_id.clone(),
        },
    ).await.map_err(|e| {
        // Rollback installed dependencies on failure
        temporal_sdk::workflow::spawn_child_workflow(
            rollback_dependency_installations,
            RollbackDependenciesRequest {
                instance_ids: installed_dependencies.clone(),
            },
        );
        e
    })?;

    // Step 5: Security scan
    let security_scan = temporal_sdk::workflow::call_activity(
        scan_module_security,
        SecurityScanRequest {
            package: package.clone(),
            scan_level: SecurityScanLevel::Comprehensive,
        },
    ).await?;

    if !security_scan.passed {
        // Rollback on security failure
        temporal_sdk::workflow::spawn_child_workflow(
            rollback_dependency_installations,
            RollbackDependenciesRequest {
                instance_ids: installed_dependencies,
            },
        );
        return Err(ModuleWorkflowError::SecurityScanFailed(security_scan.issues));
    }

    // Step 6: Create module instance
    let instance = temporal_sdk::workflow::call_activity(
        create_module_instance,
        CreateInstanceRequest {
            module_id: request.module_id.clone(),
            tenant_id: request.tenant_id.clone(),
            version: package.metadata.version.clone(),
            configuration: request.configuration.clone(),
        },
    ).await?;

    // Step 7: Deploy module to sandbox
    let deployment = temporal_sdk::workflow::call_activity(
        deploy_module_to_sandbox,
        DeployToSandboxRequest {
            instance_id: instance.id,
            package: package.clone(),
            sandbox_config: package.manifest.sandbox_config.clone(),
        },
    ).await.map_err(|e| {
        // Rollback instance creation on deployment failure
        temporal_sdk::workflow::spawn_activity(
            cleanup_module_instance,
            CleanupInstanceRequest {
                instance_id: instance.id,
                cleanup_data: true,
            },
        );
        e
    })?;

    // Step 8: Initialize module
    let initialization = temporal_sdk::workflow::call_activity(
        initialize_module,
        InitializeModuleRequest {
            instance_id: instance.id,
            configuration: request.configuration.unwrap_or_default(),
        },
    ).await.map_err(|e| {
        // Rollback deployment on initialization failure
        temporal_sdk::workflow::spawn_activity(
            cleanup_module_deployment,
            CleanupDeploymentRequest {
                instance_id: instance.id,
                deployment_id: deployment.id,
            },
        );
        e
    })?;

    // Step 9: Register module extensions
    temporal_sdk::workflow::call_activity(
        register_module_extensions,
        RegisterExtensionsRequest {
            instance_id: instance.id,
            extensions: package.manifest.capabilities.clone(),
        },
    ).await?;

    // Step 10: Auto-activate if requested
    if request.auto_activate {
        temporal_sdk::workflow::call_activity(
            activate_module,
            ActivateModuleRequest {
                instance_id: instance.id,
            },
        ).await?;
    }

    // Step 11: Start monitoring
    temporal_sdk::workflow::call_activity(
        start_module_monitoring,
        StartMonitoringRequest {
            instance_id: instance.id,
        },
    ).await?;

    // Step 12: Send installation notification
    temporal_sdk::workflow::call_activity(
        send_installation_notification,
        InstallationNotificationRequest {
            module_id: request.module_id.clone(),
            instance_id: instance.id,
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            status: "completed".to_string(),
        },
    ).await?;

    tracing::info!("Successfully completed module installation workflow for: {}", request.module_id);

    Ok(InstallModuleResult {
        instance_id: instance.id,
        module_id: request.module_id,
        version: package.metadata.version,
        status: if request.auto_activate { 
            ModuleStatus::Active 
        } else { 
            ModuleStatus::Installed 
        },
        installation_path: deployment.path,
    })
}

/// Module update workflow with rollback capabilities
#[temporal_sdk::workflow]
pub async fn update_module_workflow(
    request: UpdateModuleRequest,
) -> Result<UpdateModuleResult, ModuleWorkflowError> {
    tracing::info!("Starting module update workflow for: {}", request.instance_id);

    // Step 1: Get current module instance
    let current_instance = temporal_sdk::workflow::call_activity(
        get_module_instance,
        GetInstanceRequest {
            instance_id: request.instance_id,
        },
    ).await?;

    // Step 2: Determine target version
    let target_version = match request.target_version {
        Some(version) => version,
        None => {
            let latest = temporal_sdk::workflow::call_activity(
                get_latest_module_version,
                GetLatestVersionRequest {
                    module_id: current_instance.module_id.clone(),
                },
            ).await?;
            latest.version
        }
    };

    // Step 3: Validate update compatibility
    let compatibility = temporal_sdk::workflow::call_activity(
        validate_module_update,
        ValidateUpdateRequest {
            current_instance: current_instance.clone(),
            target_version: target_version.clone(),
        },
    ).await?;

    if !compatibility.is_compatible {
        return Err(ModuleWorkflowError::IncompatibleUpdate(compatibility.issues));
    }

    // Step 4: Create backup if requested
    let backup_id = if request.backup_current {
        Some(temporal_sdk::workflow::call_activity(
            create_module_backup,
            CreateBackupRequest {
                instance_id: request.instance_id,
                backup_type: BackupType::Full,
            },
        ).await?.backup_id)
    } else {
        None
    };

    // Step 5: Download new version
    let new_package = temporal_sdk::workflow::call_activity(
        download_module_package,
        DownloadPackageRequest {
            module_id: current_instance.module_id.clone(),
            version: Some(target_version.clone()),
            tenant_id: current_instance.tenant_id.clone(),
        },
    ).await?;

    // Step 6: Security scan new version
    let security_scan = temporal_sdk::workflow::call_activity(
        scan_module_security,
        SecurityScanRequest {
            package: new_package.clone(),
            scan_level: SecurityScanLevel::Update,
        },
    ).await?;

    if !security_scan.passed {
        return Err(ModuleWorkflowError::SecurityScanFailed(security_scan.issues));
    }

    // Step 7: Deactivate current module if active
    let was_active = matches!(current_instance.status, ModuleStatus::Active);
    if was_active {
        temporal_sdk::workflow::call_activity(
            deactivate_module,
            DeactivateModuleRequest {
                instance_id: request.instance_id,
            },
        ).await?;
    }

    // Step 8: Update module deployment
    let update_result = temporal_sdk::workflow::call_activity(
        update_module_deployment,
        UpdateDeploymentRequest {
            instance_id: request.instance_id,
            new_package: new_package.clone(),
            preserve_config: request.preserve_config,
        },
    ).await.map_err(|e| {
        // Restore from backup on failure
        if let Some(backup_id) = &backup_id {
            temporal_sdk::workflow::spawn_activity(
                restore_module_from_backup,
                RestoreFromBackupRequest {
                    instance_id: request.instance_id,
                    backup_id: backup_id.clone(),
                },
            );
        }
        e
    })?;

    // Step 9: Update module instance record
    temporal_sdk::workflow::call_activity(
        update_module_instance,
        UpdateInstanceRequest {
            instance_id: request.instance_id,
            version: target_version.clone(),
            status: ModuleStatus::Installed,
        },
    ).await?;

    // Step 10: Reactivate if it was active before
    if was_active {
        temporal_sdk::workflow::call_activity(
            activate_module,
            ActivateModuleRequest {
                instance_id: request.instance_id,
            },
        ).await.map_err(|e| {
            // Restore from backup on activation failure
            if let Some(backup_id) = &backup_id {
                temporal_sdk::workflow::spawn_activity(
                    restore_module_from_backup,
                    RestoreFromBackupRequest {
                        instance_id: request.instance_id,
                        backup_id: backup_id.clone(),
                    },
                );
            }
            e
        })?;
    }

    // Step 11: Send update notification
    temporal_sdk::workflow::call_activity(
        send_update_notification,
        UpdateNotificationRequest {
            instance_id: request.instance_id,
            old_version: current_instance.version.clone(),
            new_version: target_version.clone(),
            tenant_id: current_instance.tenant_id.clone(),
        },
    ).await?;

    tracing::info!("Successfully completed module update workflow for: {}", request.instance_id);

    Ok(UpdateModuleResult {
        instance_id: request.instance_id,
        old_version: current_instance.version,
        new_version: target_version,
        backup_id,
        status: if was_active { ModuleStatus::Active } else { ModuleStatus::Installed },
    })
}

/// Module uninstallation workflow with cleanup
#[temporal_sdk::workflow]
pub async fn uninstall_module_workflow(
    request: UninstallModuleRequest,
) -> Result<UninstallModuleResult, ModuleWorkflowError> {
    tracing::info!("Starting module uninstallation workflow for: {}", request.instance_id);

    // Step 1: Get module instance
    let instance = temporal_sdk::workflow::call_activity(
        get_module_instance,
        GetInstanceRequest {
            instance_id: request.instance_id,
        },
    ).await?;

    // Step 2: Check for dependent modules
    let dependents = temporal_sdk::workflow::call_activity(
        check_module_dependents,
        CheckDependentsRequest {
            module_id: instance.module_id.clone(),
            tenant_id: instance.tenant_id.clone(),
        },
    ).await?;

    if !dependents.dependents.is_empty() {
        return Err(ModuleWorkflowError::HasDependents(dependents.dependents));
    }

    // Step 3: Create backup if requested
    let backup_id = if request.backup_data {
        Some(temporal_sdk::workflow::call_activity(
            create_module_backup,
            CreateBackupRequest {
                instance_id: request.instance_id,
                backup_type: BackupType::DataOnly,
            },
        ).await?.backup_id)
    } else {
        None
    };

    // Step 4: Deactivate module if active
    if matches!(instance.status, ModuleStatus::Active) {
        temporal_sdk::workflow::call_activity(
            deactivate_module,
            DeactivateModuleRequest {
                instance_id: request.instance_id,
            },
        ).await?;
    }

    // Step 5: Unregister module extensions
    temporal_sdk::workflow::call_activity(
        unregister_module_extensions,
        UnregisterExtensionsRequest {
            instance_id: request.instance_id,
        },
    ).await?;

    // Step 6: Stop monitoring
    temporal_sdk::workflow::call_activity(
        stop_module_monitoring,
        StopMonitoringRequest {
            instance_id: request.instance_id,
        },
    ).await?;

    // Step 7: Cleanup module resources
    let cleanup_summary = temporal_sdk::workflow::call_activity(
        cleanup_module_resources,
        CleanupResourcesRequest {
            instance_id: request.instance_id,
            cleanup_data: request.cleanup_data,
        },
    ).await?;

    // Step 8: Remove module deployment
    temporal_sdk::workflow::call_activity(
        remove_module_deployment,
        RemoveDeploymentRequest {
            instance_id: request.instance_id,
        },
    ).await?;

    // Step 9: Delete module instance record
    temporal_sdk::workflow::call_activity(
        delete_module_instance,
        DeleteInstanceRequest {
            instance_id: request.instance_id,
        },
    ).await?;

    // Step 10: Send uninstallation notification
    temporal_sdk::workflow::call_activity(
        send_uninstallation_notification,
        UninstallationNotificationRequest {
            module_id: instance.module_id.clone(),
            instance_id: request.instance_id,
            tenant_id: instance.tenant_id.clone(),
            cleanup_summary: cleanup_summary.clone(),
        },
    ).await?;

    tracing::info!("Successfully completed module uninstallation workflow for: {}", request.instance_id);

    Ok(UninstallModuleResult {
        instance_id: request.instance_id,
        backup_id,
        cleanup_summary,
    })
}

/// Module marketplace sync workflow
#[temporal_sdk::workflow]
pub async fn sync_marketplace_workflow() -> Result<MarketplaceSyncResult, ModuleWorkflowError> {
    tracing::info!("Starting marketplace sync workflow");

    // Step 1: Fetch latest module registry
    let registry = temporal_sdk::workflow::call_activity(
        fetch_marketplace_registry,
        FetchRegistryRequest {
            last_sync: None, // Get full registry
        },
    ).await?;

    // Step 2: Update local module catalog
    temporal_sdk::workflow::call_activity(
        update_module_catalog,
        UpdateCatalogRequest {
            modules: registry.modules,
            categories: registry.categories,
        },
    ).await?;

    // Step 3: Check for module updates
    let update_checks = temporal_sdk::workflow::call_activity(
        check_module_updates,
        CheckUpdatesRequest {
            installed_modules: registry.installed_modules,
        },
    ).await?;

    // Step 4: Send update notifications
    if !update_checks.available_updates.is_empty() {
        temporal_sdk::workflow::call_activity(
            send_update_notifications,
            UpdateNotificationsRequest {
                updates: update_checks.available_updates,
            },
        ).await?;
    }

    tracing::info!("Successfully completed marketplace sync workflow");

    Ok(MarketplaceSyncResult {
        modules_synced: registry.modules.len() as u32,
        updates_available: update_checks.available_updates.len() as u32,
        sync_completed_at: Utc::now(),
    })
}

/// Rollback workflow for failed installations
#[temporal_sdk::workflow]
pub async fn rollback_dependency_installations(
    request: RollbackDependenciesRequest,
) -> Result<(), ModuleWorkflowError> {
    tracing::info!("Starting dependency rollback workflow");

    for instance_id in request.instance_ids {
        // Attempt to uninstall each dependency
        let uninstall_request = UninstallModuleRequest {
            instance_id,
            cleanup_data: true,
            backup_data: false,
        };

        if let Err(e) = temporal_sdk::workflow::call_child_workflow(
            uninstall_module_workflow,
            uninstall_request,
        ).await {
            tracing::warn!("Failed to rollback dependency {}: {}", instance_id, e);
        }
    }

    Ok(())
}

// Workflow error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ModuleWorkflowError {
    ValidationFailed(Vec<String>),
    SecurityScanFailed(Vec<String>),
    IncompatibleUpdate(Vec<String>),
    HasDependents(Vec<String>),
    ActivityFailed(String),
    ChildWorkflowFailed(String),
    Timeout(String),
    Cancelled(String),
}

impl std::fmt::Display for ModuleWorkflowError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ModuleWorkflowError::ValidationFailed(errors) => {
                write!(f, "Validation failed: {}", errors.join(", "))
            }
            ModuleWorkflowError::SecurityScanFailed(issues) => {
                write!(f, "Security scan failed: {}", issues.join(", "))
            }
            ModuleWorkflowError::IncompatibleUpdate(issues) => {
                write!(f, "Incompatible update: {}", issues.join(", "))
            }
            ModuleWorkflowError::HasDependents(dependents) => {
                write!(f, "Module has dependents: {}", dependents.join(", "))
            }
            ModuleWorkflowError::ActivityFailed(msg) => write!(f, "Activity failed: {}", msg),
            ModuleWorkflowError::ChildWorkflowFailed(msg) => write!(f, "Child workflow failed: {}", msg),
            ModuleWorkflowError::Timeout(msg) => write!(f, "Workflow timeout: {}", msg),
            ModuleWorkflowError::Cancelled(msg) => write!(f, "Workflow cancelled: {}", msg),
        }
    }
}

impl std::error::Error for ModuleWorkflowError {}

// Workflow request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateInstallationRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveDependenciesRequest {
    pub module_id: String,
    pub version: Option<Version>,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyResolutionResult {
    pub dependencies: Vec<ResolvedDependency>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedDependency {
    pub module_id: String,
    pub version: Version,
    pub already_installed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadPackageRequest {
    pub module_id: String,
    pub version: Option<Version>,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanRequest {
    pub package: ModulePackage,
    pub scan_level: SecurityScanLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityScanLevel {
    Basic,
    Standard,
    Comprehensive,
    Update,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityScanResponse {
    pub passed: bool,
    pub issues: Vec<String>,
    pub scan_result: SecurityScanResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateInstanceRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub version: Version,
    pub configuration: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployToSandboxRequest {
    pub instance_id: Uuid,
    pub package: ModulePackage,
    pub sandbox_config: crate::SandboxConfiguration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub id: String,
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializeModuleRequest {
    pub instance_id: Uuid,
    pub configuration: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterExtensionsRequest {
    pub instance_id: Uuid,
    pub extensions: crate::ModuleCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateModuleRequest {
    pub instance_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartMonitoringRequest {
    pub instance_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationNotificationRequest {
    pub module_id: String,
    pub instance_id: Uuid,
    pub tenant_id: String,
    pub user_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollbackDependenciesRequest {
    pub instance_ids: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceSyncResult {
    pub modules_synced: u32,
    pub updates_available: u32,
    pub sync_completed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchRegistryRequest {
    pub last_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceRegistry {
    pub modules: Vec<crate::ModuleMetadata>,
    pub categories: Vec<crate::ModuleCategory>,
    pub installed_modules: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCatalogRequest {
    pub modules: Vec<crate::ModuleMetadata>,
    pub categories: Vec<crate::ModuleCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckUpdatesRequest {
    pub installed_modules: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateCheckResult {
    pub available_updates: Vec<ModuleUpdate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleUpdate {
    pub instance_id: Uuid,
    pub current_version: Version,
    pub latest_version: Version,
    pub update_type: UpdateType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UpdateType {
    Patch,
    Minor,
    Major,
    Security,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateNotificationsRequest {
    pub updates: Vec<ModuleUpdate>,
}

// Additional request/response types for other activities would be defined here...