use async_trait::async_trait;
use std::collections::HashMap;
use std::path::Path;
use std::time::Instant;
use uuid::Uuid;

use crate::error::{ActivityError, ModuleServiceError};
use crate::repositories::{ModuleRepository, InstallationRepository, SecurityRepository};
use crate::services::{PackageService, SecurityService, SandboxService, MarketplaceService};
use crate::types::{ModuleStatus, SecurityScanResults, SecurityVulnerability, VulnerabilitySeverity};

// Activity trait definitions
#[async_trait]
pub trait ModuleActivities {
    // Installation activities
    async fn validate_module_installation(&self, request: ValidateModuleInstallationRequest) -> Result<ValidateModuleInstallationResponse, ActivityError>;
    async fn download_module_package(&self, request: DownloadModulePackageRequest) -> Result<DownloadModulePackageResponse, ActivityError>;
    async fn perform_security_scan(&self, request: PerformSecurityScanRequest) -> Result<PerformSecurityScanResponse, ActivityError>;
    async fn install_dependency(&self, request: InstallDependencyRequest) -> Result<InstallDependencyResponse, ActivityError>;
    async fn deploy_module(&self, request: DeployModuleRequest) -> Result<DeployModuleResponse, ActivityError>;
    async fn configure_sandbox(&self, request: ConfigureSandboxRequest) -> Result<ConfigureSandboxResponse, ActivityError>;
    async fn register_installation(&self, request: RegisterInstallationRequest) -> Result<RegisterInstallationResponse, ActivityError>;
    async fn activate_module(&self, request: ActivateModuleRequest) -> Result<ActivateModuleResponse, ActivityError>;
    async fn cleanup_package(&self, request: CleanupPackageRequest) -> Result<(), ActivityError>;

    // Update activities
    async fn validate_module_update(&self, request: ValidateModuleUpdateRequest) -> Result<ValidateModuleUpdateResponse, ActivityError>;
    async fn backup_module(&self, request: BackupModuleRequest) -> Result<BackupModuleResponse, ActivityError>;
    async fn perform_module_update(&self, request: PerformModuleUpdateRequest) -> Result<PerformModuleUpdateResponse, ActivityError>;

    // Uninstallation activities
    async fn validate_module_uninstallation(&self, request: ValidateModuleUninstallationRequest) -> Result<ValidateModuleUninstallationResponse, ActivityError>;
    async fn deactivate_module(&self, request: DeactivateModuleRequest) -> Result<DeactivateModuleResponse, ActivityError>;
    async fn cleanup_module_data(&self, request: CleanupModuleDataRequest) -> Result<CleanupModuleDataResponse, ActivityError>;
    async fn remove_module_installation(&self, request: RemoveModuleInstallationRequest) -> Result<RemoveModuleInstallationResponse, ActivityError>;
    async fn update_installation_status(&self, request: UpdateInstallationStatusRequest) -> Result<(), ActivityError>;

    // Marketplace activities
    async fn fetch_marketplace_data(&self, request: FetchMarketplaceDataRequest) -> Result<FetchMarketplaceDataResponse, ActivityError>;
    async fn sync_module_data(&self, request: SyncModuleDataRequest) -> Result<SyncModuleDataResponse, ActivityError>;

    // Security activities
    async fn download_module_for_scan(&self, request: DownloadModuleForScanRequest) -> Result<DownloadModuleForScanResponse, ActivityError>;
    async fn store_scan_results(&self, request: StoreScanResultsRequest) -> Result<(), ActivityError>;
}

// Implementation of module activities
pub struct ModuleActivitiesImpl {
    module_repo: ModuleRepository,
    installation_repo: InstallationRepository,
    security_repo: SecurityRepository,
    package_service: PackageService,
    security_service: SecurityService,
    sandbox_service: SandboxService,
    marketplace_service: MarketplaceService,
}

impl ModuleActivitiesImpl {
    pub fn new(
        module_repo: ModuleRepository,
        installation_repo: InstallationRepository,
        security_repo: SecurityRepository,
        package_service: PackageService,
        security_service: SecurityService,
        sandbox_service: SandboxService,
        marketplace_service: MarketplaceService,
    ) -> Self {
        Self {
            module_repo,
            installation_repo,
            security_repo,
            package_service,
            security_service,
            sandbox_service,
            marketplace_service,
        }
    }
}

#[async_trait]
impl ModuleActivities for ModuleActivitiesImpl {
    async fn validate_module_installation(&self, request: ValidateModuleInstallationRequest) -> Result<ValidateModuleInstallationResponse, ActivityError> {
        // Check if module exists
        let module = self.module_repo.get_module_by_id(&request.module_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?
            .ok_or_else(|| ActivityError::ValidationFailed(format!("Module {} not found", request.module_id)))?;

        // Check if already installed and not force reinstall
        if !request.force_reinstall {
            if let Ok(Some(_)) = self.installation_repo.get_installation(&request.module_id, &request.tenant_id).await {
                return Err(ActivityError::ValidationFailed("Module already installed".to_string()));
            }
        }

        // Validate tenant permissions
        // This would check tenant quotas, permissions, etc.
        
        // Get module dependencies
        let dependencies = self.module_repo.get_module_dependencies(&request.module_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        let module_dependencies: Vec<crate::workflows::ModuleDependency> = dependencies
            .into_iter()
            .map(|dep| crate::workflows::ModuleDependency {
                id: dep.dependency_id,
                version_requirement: dep.version_requirement,
                optional: dep.optional,
            })
            .collect();

        // Get expected checksum from module version
        let versions = self.module_repo.get_module_versions(&request.module_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        let version_record = versions
            .into_iter()
            .find(|v| v.version == request.version)
            .ok_or_else(|| ActivityError::ValidationFailed(format!("Version {} not found", request.version)))?;

        Ok(ValidateModuleInstallationResponse {
            is_valid: true,
            errors: vec![],
            expected_checksum: version_record.package_hash,
            dependencies: module_dependencies,
            resource_limits: crate::types::ResourceLimits {
                max_memory_mb: 512,
                max_cpu_percent: 50.0,
                max_storage_mb: 100,
                max_network_bandwidth_mbps: Some(10.0),
                max_execution_time_seconds: Some(300),
            },
            security_policy: crate::types::SecurityPolicy {
                allow_eval: false,
                allow_dynamic_imports: true,
                allow_worker_threads: false,
                allow_child_processes: false,
                content_security_policy: Some("default-src 'self'".to_string()),
            },
        })
    }

    async fn download_module_package(&self, request: DownloadModulePackageRequest) -> Result<DownloadModulePackageResponse, ActivityError> {
        let package_path = self.package_service.download_package(
            &request.module_id,
            &request.version,
            &request.checksum,
        ).await.map_err(|e| ActivityError::DownloadFailed(e.to_string()))?;

        // Verify checksum
        let verified = self.package_service.verify_checksum(&package_path, &request.checksum)
            .await
            .map_err(|e| ActivityError::ValidationFailed(e.to_string()))?;

        if !verified {
            // Cleanup invalid package
            let _ = std::fs::remove_file(&package_path);
            return Err(ActivityError::ValidationFailed("Package checksum verification failed".to_string()));
        }

        Ok(DownloadModulePackageResponse {
            package_path,
            verified,
        })
    }

    async fn perform_security_scan(&self, request: PerformSecurityScanRequest) -> Result<PerformSecurityScanResponse, ActivityError> {
        let start_time = Instant::now();

        let scan_result = self.security_service.scan_package(
            &request.package_path,
            request.deep_scan,
        ).await.map_err(|e| ActivityError::SecurityScanFailed(e.to_string()))?;

        let scan_duration = start_time.elapsed();

        Ok(PerformSecurityScanResponse {
            passed: scan_result.passed,
            score: scan_result.score,
            vulnerabilities: scan_result.vulnerabilities,
            scan_duration_seconds: scan_duration.as_secs() as u32,
        })
    }

    async fn install_dependency(&self, request: InstallDependencyRequest) -> Result<InstallDependencyResponse, ActivityError> {
        // Check if dependency is already installed
        if let Ok(Some(_)) = self.installation_repo.get_installation(&request.dependency_id, &request.tenant_id).await {
            return Ok(InstallDependencyResponse {
                installed: false,
                already_exists: true,
            });
        }

        // For now, we'll simulate dependency installation
        // In a real implementation, this would recursively install the dependency
        
        Ok(InstallDependencyResponse {
            installed: true,
            already_exists: false,
        })
    }

    async fn deploy_module(&self, request: DeployModuleRequest) -> Result<DeployModuleResponse, ActivityError> {
        let deployment_id = Uuid::new_v4().to_string();
        
        // Extract package
        let extraction_path = self.package_service.extract_package(
            &request.package_path,
            &deployment_id,
        ).await.map_err(|e| ActivityError::InstallationFailed(e.to_string()))?;

        // Deploy to tenant-specific location
        let deployment_path = self.package_service.deploy_to_tenant(
            &extraction_path,
            &request.tenant_id,
            &request.module_id,
        ).await.map_err(|e| ActivityError::InstallationFailed(e.to_string()))?;

        Ok(DeployModuleResponse {
            deployment_id,
            deployment_path,
            extracted_files: vec![], // Would list actual files
        })
    }

    async fn configure_sandbox(&self, request: ConfigureSandboxRequest) -> Result<ConfigureSandboxResponse, ActivityError> {
        let sandbox_config = self.sandbox_service.create_sandbox_config(
            &request.module_id,
            &request.tenant_id,
            &request.resource_limits,
            &request.security_policy,
        ).await.map_err(|e| ActivityError::InstallationFailed(e.to_string()))?;

        Ok(ConfigureSandboxResponse {
            sandbox_config,
            configured: true,
        })
    }

    async fn register_installation(&self, request: RegisterInstallationRequest) -> Result<RegisterInstallationResponse, ActivityError> {
        let installation_id = Uuid::new_v4().to_string();

        let installation = crate::models::ModuleInstallationRecord {
            id: installation_id.clone(),
            module_id: request.module_id.clone(),
            tenant_id: request.tenant_id.clone(),
            version: request.version.clone(),
            status: "installed".to_string(),
            configuration_json: request.configuration.map(|c| serde_json::to_value(c).unwrap()),
            installation_path: None,
            sandbox_config_json: Some(serde_json::to_value(request.sandbox_config).unwrap()),
            installed_by: request.user_id.clone(),
            installed_at: chrono::Utc::now(),
            activated_at: None,
            last_used_at: None,
        };

        self.installation_repo.create_installation(&installation)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        Ok(RegisterInstallationResponse {
            installation_id,
            registered: true,
        })
    }

    async fn activate_module(&self, request: ActivateModuleRequest) -> Result<ActivateModuleResponse, ActivityError> {
        // Update installation status to active
        self.installation_repo.update_installation_status(
            &request.installation_id,
            &ModuleStatus::Active,
        ).await.map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        // Register module endpoints, workflows, etc.
        // This would involve registering the module's extension points

        Ok(ActivateModuleResponse {
            activated: true,
        })
    }

    async fn cleanup_package(&self, request: CleanupPackageRequest) -> Result<(), ActivityError> {
        if Path::new(&request.package_path).exists() {
            std::fs::remove_file(&request.package_path)
                .map_err(|e| ActivityError::InstallationFailed(format!("Cleanup failed: {}", e)))?;
        }
        Ok(())
    }

    async fn validate_module_update(&self, request: ValidateModuleUpdateRequest) -> Result<ValidateModuleUpdateResponse, ActivityError> {
        // Get current installation
        let installation = self.installation_repo.get_installation(&request.module_id, &request.tenant_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?
            .ok_or_else(|| ActivityError::ValidationFailed("Module not installed".to_string()))?;

        // Check if target version exists
        let versions = self.module_repo.get_module_versions(&request.module_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        let target_version = versions
            .iter()
            .find(|v| v.version == request.target_version)
            .ok_or_else(|| ActivityError::ValidationFailed(format!("Target version {} not found", request.target_version)))?;

        Ok(ValidateModuleUpdateResponse {
            is_valid: true,
            current_version: installation.version,
            expected_checksum: target_version.package_hash.clone(),
            migration_scripts: vec![], // Would contain actual migration scripts
        })
    }

    async fn backup_module(&self, request: BackupModuleRequest) -> Result<BackupModuleResponse, ActivityError> {
        let backup_id = Uuid::new_v4().to_string();
        
        // Create backup of current module state
        // This would backup files, configuration, and data
        
        Ok(BackupModuleResponse {
            backup_id,
            backup_path: format!("/backups/{}", backup_id),
            created: true,
        })
    }

    async fn perform_module_update(&self, request: PerformModuleUpdateRequest) -> Result<PerformModuleUpdateResponse, ActivityError> {
        // Extract new version
        let extraction_path = self.package_service.extract_package(
            &request.package_path,
            &format!("update-{}", Uuid::new_v4()),
        ).await.map_err(|e| ActivityError::InstallationFailed(e.to_string()))?;

        // Apply update
        // This would involve replacing files, running migrations, etc.

        // Run migration scripts
        let mut migration_applied = false;
        for script in &request.migration_scripts {
            // Execute migration script
            migration_applied = true;
        }

        Ok(PerformModuleUpdateResponse {
            updated: true,
            migration_applied,
        })
    }

    async fn validate_module_uninstallation(&self, request: ValidateModuleUninstallationRequest) -> Result<ValidateModuleUninstallationResponse, ActivityError> {
        // Check if module is installed
        let installation = self.installation_repo.get_installation(&request.module_id, &request.tenant_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?
            .ok_or_else(|| ActivityError::ValidationFailed("Module not installed".to_string()))?;

        let is_active = installation.status == "active";

        // Check for dependent modules if not force uninstall
        if !request.force_uninstall {
            // Check for modules that depend on this one
            // This would query the dependencies table
        }

        Ok(ValidateModuleUninstallationResponse {
            is_valid: true,
            is_active,
            dependent_modules: vec![],
        })
    }

    async fn deactivate_module(&self, request: DeactivateModuleRequest) -> Result<DeactivateModuleResponse, ActivityError> {
        // Update status to inactive
        let installation = self.installation_repo.get_installation(&request.module_id, &request.tenant_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?
            .ok_or_else(|| ActivityError::ValidationFailed("Module not installed".to_string()))?;

        self.installation_repo.update_installation_status(
            &installation.id,
            &ModuleStatus::Inactive,
        ).await.map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        Ok(DeactivateModuleResponse {
            deactivated: true,
        })
    }

    async fn cleanup_module_data(&self, request: CleanupModuleDataRequest) -> Result<CleanupModuleDataResponse, ActivityError> {
        // Clean up module-specific data
        // This would involve removing database records, files, etc.
        
        Ok(CleanupModuleDataResponse {
            cleaned: true,
            data_removed: vec!["database_tables".to_string(), "uploaded_files".to_string()],
        })
    }

    async fn remove_module_installation(&self, request: RemoveModuleInstallationRequest) -> Result<RemoveModuleInstallationResponse, ActivityError> {
        // Remove installation record
        self.installation_repo.delete_installation(&request.module_id, &request.tenant_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        // Remove dependencies if requested
        let mut dependencies_removed = vec![];
        if request.remove_dependencies {
            // This would check and remove unused dependencies
        }

        Ok(RemoveModuleInstallationResponse {
            removed: true,
            dependencies_removed,
        })
    }

    async fn update_installation_status(&self, request: UpdateInstallationStatusRequest) -> Result<(), ActivityError> {
        let installation = self.installation_repo.get_installation(&request.module_id, &request.tenant_id)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?
            .ok_or_else(|| ActivityError::ValidationFailed("Module not installed".to_string()))?;

        self.installation_repo.update_installation_status(&installation.id, &request.status)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        Ok(())
    }

    async fn fetch_marketplace_data(&self, request: FetchMarketplaceDataRequest) -> Result<FetchMarketplaceDataResponse, ActivityError> {
        let modules = self.marketplace_service.fetch_modules(
            &request.sync_type,
            request.module_ids.as_ref(),
            request.force_update,
        ).await.map_err(|e| ActivityError::ExternalServiceError(e.to_string()))?;

        Ok(FetchMarketplaceDataResponse {
            modules,
        })
    }

    async fn sync_module_data(&self, request: SyncModuleDataRequest) -> Result<SyncModuleDataResponse, ActivityError> {
        let action = self.marketplace_service.sync_module(&request.module_data, request.force_update)
            .await
            .map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        Ok(SyncModuleDataResponse {
            action,
        })
    }

    async fn download_module_for_scan(&self, request: DownloadModuleForScanRequest) -> Result<DownloadModuleForScanResponse, ActivityError> {
        let package_path = self.package_service.download_package_for_scan(
            &request.module_id,
            &request.version,
        ).await.map_err(|e| ActivityError::DownloadFailed(e.to_string()))?;

        Ok(DownloadModuleForScanResponse {
            package_path,
        })
    }

    async fn store_scan_results(&self, request: StoreScanResultsRequest) -> Result<(), ActivityError> {
        self.security_repo.store_scan_results(
            &request.module_id,
            &request.version,
            &request.scan_type,
            &request.scan_results,
        ).await.map_err(|e| ActivityError::DatabaseFailed(e.to_string()))?;

        Ok(())
    }
}

// Request/Response types for activities
use serde::{Deserialize, Serialize};

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
    pub dependencies: Vec<crate::workflows::ModuleDependency>,
    pub resource_limits: crate::types::ResourceLimits,
    pub security_policy: crate::types::SecurityPolicy,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformSecurityScanRequest {
    pub module_id: String,
    pub version: String,
    pub package_path: String,
    pub deep_scan: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformSecurityScanResponse {
    pub passed: bool,
    pub score: u8,
    pub vulnerabilities: Vec<SecurityVulnerability>,
    pub scan_duration_seconds: u32,
}

// Additional request/response types would be defined here...
// (I'll include a few more key ones)

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallDependencyRequest {
    pub dependency_id: String,
    pub version_requirement: String,
    pub tenant_id: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallDependencyResponse {
    pub installed: bool,
    pub already_exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployModuleRequest {
    pub module_id: String,
    pub version: String,
    pub package_path: String,
    pub tenant_id: String,
    pub configuration: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployModuleResponse {
    pub deployment_id: String,
    pub deployment_path: String,
    pub extracted_files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureSandboxRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub resource_limits: crate::types::ResourceLimits,
    pub security_policy: crate::types::SecurityPolicy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigureSandboxResponse {
    pub sandbox_config: crate::types::SandboxConfig,
    pub configured: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterInstallationRequest {
    pub module_id: String,
    pub version: String,
    pub tenant_id: String,
    pub user_id: String,
    pub deployment_id: String,
    pub configuration: Option<HashMap<String, serde_json::Value>>,
    pub sandbox_config: crate::types::SandboxConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterInstallationResponse {
    pub installation_id: String,
    pub registered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateModuleRequest {
    pub installation_id: String,
    pub module_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivateModuleResponse {
    pub activated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupPackageRequest {
    pub package_path: String,
}

// Update-related types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateModuleUpdateRequest {
    pub module_id: String,
    pub current_version: String,
    pub target_version: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateModuleUpdateResponse {
    pub is_valid: bool,
    pub current_version: String,
    pub expected_checksum: String,
    pub migration_scripts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupModuleRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub backup_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupModuleResponse {
    pub backup_id: String,
    pub backup_path: String,
    pub created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformModuleUpdateRequest {
    pub module_id: String,
    pub target_version: String,
    pub package_path: String,
    pub tenant_id: String,
    pub migration_scripts: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformModuleUpdateResponse {
    pub updated: bool,
    pub migration_applied: bool,
}

// Uninstallation-related types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateModuleUninstallationRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub force_uninstall: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateModuleUninstallationResponse {
    pub is_valid: bool,
    pub is_active: bool,
    pub dependent_modules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeactivateModuleRequest {
    pub module_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeactivateModuleResponse {
    pub deactivated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupModuleDataRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub cleanup_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanupModuleDataResponse {
    pub cleaned: bool,
    pub data_removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveModuleInstallationRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub remove_dependencies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoveModuleInstallationResponse {
    pub removed: bool,
    pub dependencies_removed: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInstallationStatusRequest {
    pub module_id: String,
    pub tenant_id: String,
    pub status: ModuleStatus,
}

// Marketplace-related types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchMarketplaceDataRequest {
    pub sync_type: String,
    pub module_ids: Option<Vec<String>>,
    pub force_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FetchMarketplaceDataResponse {
    pub modules: Vec<crate::types::MarketplaceListing>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncModuleDataRequest {
    pub module_data: crate::types::MarketplaceListing,
    pub force_update: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncModuleDataResponse {
    pub action: SyncAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncAction {
    Added,
    Updated,
    Removed,
    NoChange,
}

// Security-related types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadModuleForScanRequest {
    pub module_id: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadModuleForScanResponse {
    pub package_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoreScanResultsRequest {
    pub module_id: String,
    pub version: String,
    pub scan_type: String,
    pub scan_results: PerformSecurityScanResponse,
}