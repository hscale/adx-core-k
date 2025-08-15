use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::ModuleServiceError;
use crate::types::{Module, ModuleStatus, InstallModuleRequest, InstallModuleResult, UpdateModuleRequest, UpdateModuleResult, UninstallModuleRequest, UninstallModuleResult};
use crate::repositories::{ModuleRepository, InstallationRepository};
use crate::services::{PackageService, SecurityService, SandboxService};
use crate::workflows::{install_module_workflow, update_module_workflow, uninstall_module_workflow};
use crate::models::{InstallModuleWorkflowInput, UpdateModuleWorkflowInput, UninstallModuleWorkflowInput};

/// Comprehensive module manager with hot-loading, dependency resolution, and version compatibility
#[async_trait]
pub trait ModuleManagerTrait {
    async fn install_module(&self, request: InstallModuleRequest) -> Result<InstallModuleResult, ModuleServiceError>;
    async fn update_module(&self, request: UpdateModuleRequest) -> Result<UpdateModuleResult, ModuleServiceError>;
    async fn uninstall_module(&self, request: UninstallModuleRequest) -> Result<UninstallModuleResult, ModuleServiceError>;
    async fn activate_module(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError>;
    async fn deactivate_module(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError>;
    async fn reload_module(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError>;
    async fn get_module_status(&self, module_id: &str, tenant_id: &str) -> Result<ModuleStatus, ModuleServiceError>;
    async fn list_installed_modules(&self, tenant_id: &str) -> Result<Vec<Module>, ModuleServiceError>;
    async fn check_dependencies(&self, module_id: &str) -> Result<DependencyCheckResult, ModuleServiceError>;
    async fn resolve_dependencies(&self, module_id: &str) -> Result<Vec<String>, ModuleServiceError>;
    async fn validate_compatibility(&self, module_id: &str, version: &str) -> Result<CompatibilityResult, ModuleServiceError>;
}

pub struct ModuleManager {
    module_repo: Arc<ModuleRepository>,
    installation_repo: Arc<InstallationRepository>,
    package_service: Arc<PackageService>,
    security_service: Arc<SecurityService>,
    sandbox_service: Arc<SandboxService>,
    loaded_modules: Arc<RwLock<HashMap<String, LoadedModule>>>,
    dependency_resolver: Arc<DependencyResolver>,
}

impl ModuleManager {
    pub fn new(
        module_repo: Arc<ModuleRepository>,
        installation_repo: Arc<InstallationRepository>,
        package_service: Arc<PackageService>,
        security_service: Arc<SecurityService>,
        sandbox_service: Arc<SandboxService>,
    ) -> Self {
        Self {
            module_repo,
            installation_repo,
            package_service,
            security_service,
            sandbox_service,
            loaded_modules: Arc::new(RwLock::new(HashMap::new())),
            dependency_resolver: Arc::new(DependencyResolver::new()),
        }
    }
}

#[async_trait]
impl ModuleManagerTrait for ModuleManager {
    async fn install_module(&self, request: InstallModuleRequest) -> Result<InstallModuleResult, ModuleServiceError> {
        // Validate request
        request.validate().map_err(|e| ModuleServiceError::ModuleValidationError(format!("Invalid request: {:?}", e)))?;

        // Check if module exists
        let module = self.module_repo.get_module_by_id(&request.module_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(request.module_id.clone()))?;

        // Check if already installed
        if !request.force_reinstall {
            if let Ok(Some(_)) = self.installation_repo.get_installation(&request.module_id, &request.tenant_id).await {
                return Err(ModuleServiceError::ModuleAlreadyExists(request.module_id));
            }
        }

        // Resolve dependencies
        let dependencies = self.resolve_dependencies(&request.module_id).await?;

        // Create workflow input
        let workflow_input = InstallModuleWorkflowInput {
            module_id: request.module_id.clone(),
            version: request.version.clone(),
            tenant_id: request.tenant_id.clone(),
            user_id: "system".to_string(), // Would come from auth context
            configuration: request.configuration.clone(),
            auto_activate: request.auto_activate,
            force_reinstall: request.force_reinstall,
        };

        // Execute installation workflow
        let workflow_result = install_module_workflow(workflow_input)
            .await
            .map_err(|e| ModuleServiceError::WorkflowError(e.to_string()))?;

        Ok(InstallModuleResult {
            module_id: workflow_result.module_id,
            version: workflow_result.version,
            installation_id: workflow_result.installation_id,
            status: workflow_result.status,
            dependencies_installed: workflow_result.dependencies_installed,
            configuration_applied: workflow_result.configuration_applied,
        })
    }

    async fn update_module(&self, request: UpdateModuleRequest) -> Result<UpdateModuleResult, ModuleServiceError> {
        // Validate request
        request.validate().map_err(|e| ModuleServiceError::ModuleValidationError(format!("Invalid request: {:?}", e)))?;

        // Check if module is installed
        let installation = self.installation_repo.get_installation(&request.module_id, &request.tenant_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(request.module_id.clone()))?;

        // Check compatibility
        let compatibility = self.validate_compatibility(&request.module_id, &request.target_version).await?;
        if !compatibility.compatible {
            return Err(ModuleServiceError::ModuleValidationError(
                format!("Version {} is not compatible: {:?}", request.target_version, compatibility.issues)
            ));
        }

        // Create workflow input
        let workflow_input = UpdateModuleWorkflowInput {
            module_id: request.module_id.clone(),
            target_version: request.target_version.clone(),
            tenant_id: request.tenant_id.clone(),
            user_id: "system".to_string(), // Would come from auth context
            backup_current: request.backup_current,
            rollback_on_failure: request.rollback_on_failure,
        };

        // Execute update workflow
        let workflow_result = update_module_workflow(workflow_input)
            .await
            .map_err(|e| ModuleServiceError::WorkflowError(e.to_string()))?;

        Ok(UpdateModuleResult {
            module_id: workflow_result.module_id,
            from_version: workflow_result.from_version,
            to_version: workflow_result.to_version,
            update_id: workflow_result.update_id,
            status: workflow_result.status,
            backup_id: workflow_result.backup_id,
            migration_applied: workflow_result.migration_applied,
        })
    }

    async fn uninstall_module(&self, request: UninstallModuleRequest) -> Result<UninstallModuleResult, ModuleServiceError> {
        // Check if module is installed
        let installation = self.installation_repo.get_installation(&request.module_id, &request.tenant_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(request.module_id.clone()))?;

        // Check dependencies
        let dependency_check = self.check_dependencies(&request.module_id).await?;
        if !dependency_check.can_uninstall && !request.force_uninstall {
            return Err(ModuleServiceError::DependencyResolutionError(
                format!("Cannot uninstall module due to dependencies: {:?}", dependency_check.dependent_modules)
            ));
        }

        // Create workflow input
        let workflow_input = UninstallModuleWorkflowInput {
            module_id: request.module_id.clone(),
            tenant_id: request.tenant_id.clone(),
            user_id: "system".to_string(), // Would come from auth context
            cleanup_data: request.cleanup_data,
            force_uninstall: request.force_uninstall,
        };

        // Execute uninstallation workflow
        let workflow_result = uninstall_module_workflow(workflow_input)
            .await
            .map_err(|e| ModuleServiceError::WorkflowError(e.to_string()))?;

        // Remove from loaded modules
        let mut loaded_modules = self.loaded_modules.write().await;
        let module_key = format!("{}:{}", request.tenant_id, request.module_id);
        loaded_modules.remove(&module_key);

        Ok(UninstallModuleResult {
            module_id: workflow_result.module_id,
            uninstallation_id: workflow_result.uninstallation_id,
            status: workflow_result.status,
            data_cleaned: workflow_result.data_cleaned,
            dependencies_removed: workflow_result.dependencies_removed,
        })
    }

    async fn activate_module(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError> {
        // Get installation
        let installation = self.installation_repo.get_installation(module_id, tenant_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(module_id.to_string()))?;

        if installation.status == "active" {
            return Ok(()); // Already active
        }

        // Load module into memory
        let loaded_module = self.load_module(module_id, tenant_id).await?;

        // Update status
        self.installation_repo.update_installation_status(&installation.id, &ModuleStatus::Active).await?;

        // Add to loaded modules
        let mut loaded_modules = self.loaded_modules.write().await;
        let module_key = format!("{}:{}", tenant_id, module_id);
        loaded_modules.insert(module_key, loaded_module);

        Ok(())
    }

    async fn deactivate_module(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError> {
        // Get installation
        let installation = self.installation_repo.get_installation(module_id, tenant_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(module_id.to_string()))?;

        if installation.status == "inactive" {
            return Ok(()); // Already inactive
        }

        // Remove from loaded modules
        let mut loaded_modules = self.loaded_modules.write().await;
        let module_key = format!("{}:{}", tenant_id, module_id);
        if let Some(loaded_module) = loaded_modules.remove(&module_key) {
            // Cleanup module resources
            loaded_module.cleanup().await?;
        }

        // Update status
        self.installation_repo.update_installation_status(&installation.id, &ModuleStatus::Inactive).await?;

        Ok(())
    }

    async fn reload_module(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError> {
        // Deactivate first
        self.deactivate_module(module_id, tenant_id).await?;
        
        // Then activate (which will reload)
        self.activate_module(module_id, tenant_id).await?;

        Ok(())
    }

    async fn get_module_status(&self, module_id: &str, tenant_id: &str) -> Result<ModuleStatus, ModuleServiceError> {
        let installation = self.installation_repo.get_installation(module_id, tenant_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(module_id.to_string()))?;

        let status = match installation.status.as_str() {
            "active" => ModuleStatus::Active,
            "inactive" => ModuleStatus::Inactive,
            "installed" => ModuleStatus::Installed,
            "installing" => ModuleStatus::Installing,
            "updating" => ModuleStatus::Updating,
            "uninstalling" => ModuleStatus::Uninstalling,
            "failed" => ModuleStatus::Failed,
            _ => ModuleStatus::Available,
        };

        Ok(status)
    }

    async fn list_installed_modules(&self, tenant_id: &str) -> Result<Vec<Module>, ModuleServiceError> {
        let installations = self.installation_repo.list_installations(tenant_id).await?;
        let mut modules = Vec::new();

        for installation in installations {
            if let Ok(Some(module)) = self.module_repo.get_module_by_id(&installation.module_id).await {
                modules.push(module);
            }
        }

        Ok(modules)
    }

    async fn check_dependencies(&self, module_id: &str) -> Result<DependencyCheckResult, ModuleServiceError> {
        let dependencies = self.module_repo.get_module_dependencies(module_id).await?;
        let mut missing_dependencies = Vec::new();
        let mut dependent_modules = Vec::new();

        // Check if dependencies are satisfied
        for dep in &dependencies {
            if self.module_repo.get_module_by_id(&dep.dependency_id).await?.is_none() {
                missing_dependencies.push(dep.dependency_id.clone());
            }
        }

        // Check what modules depend on this one
        // This would require a reverse dependency lookup
        
        Ok(DependencyCheckResult {
            satisfied: missing_dependencies.is_empty(),
            missing_dependencies,
            dependent_modules,
            can_uninstall: dependent_modules.is_empty(),
        })
    }

    async fn resolve_dependencies(&self, module_id: &str) -> Result<Vec<String>, ModuleServiceError> {
        self.dependency_resolver.resolve(module_id, &*self.module_repo).await
    }

    async fn validate_compatibility(&self, module_id: &str, version: &str) -> Result<CompatibilityResult, ModuleServiceError> {
        let module = self.module_repo.get_module_by_id(module_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(module_id.to_string()))?;

        let versions = self.module_repo.get_module_versions(module_id).await?;
        let target_version = versions
            .iter()
            .find(|v| v.version == version)
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(format!("Version {} not found", version)))?;

        // Check ADX Core compatibility
        let adx_compatibility = self.check_adx_core_compatibility(&module.manifest.adx_core).await?;
        
        // Check dependency compatibility
        let dep_compatibility = self.check_dependency_compatibility(module_id).await?;

        let mut issues = Vec::new();
        if !adx_compatibility {
            issues.push("ADX Core version incompatible".to_string());
        }
        if !dep_compatibility {
            issues.push("Dependency version conflicts".to_string());
        }

        Ok(CompatibilityResult {
            compatible: issues.is_empty(),
            issues,
            adx_core_compatible: adx_compatibility,
            dependencies_compatible: dep_compatibility,
        })
    }
}

impl ModuleManager {
    async fn load_module(&self, module_id: &str, tenant_id: &str) -> Result<LoadedModule, ModuleServiceError> {
        let module = self.module_repo.get_module_by_id(module_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(module_id.to_string()))?;

        let installation = self.installation_repo.get_installation(module_id, tenant_id)
            .await?
            .ok_or_else(|| ModuleServiceError::ModuleNotFound(module_id.to_string()))?;

        // Load module from file system
        let module_path = format!("/modules/{}/{}", tenant_id, module_id);
        
        // Create sandbox for module
        let sandbox_config = if let Some(config_json) = &installation.sandbox_config_json {
            serde_json::from_value(config_json.clone())?
        } else {
            self.sandbox_service.create_default_sandbox_config(module_id, tenant_id).await?
        };

        Ok(LoadedModule {
            module_id: module_id.to_string(),
            tenant_id: tenant_id.to_string(),
            module_path,
            sandbox_config,
            loaded_at: chrono::Utc::now(),
        })
    }

    async fn check_adx_core_compatibility(&self, compatibility: &crate::types::AdxCoreCompatibility) -> Result<bool, ModuleServiceError> {
        // Check current ADX Core version against module requirements
        let current_version = "2.0.0"; // Would be retrieved from system
        
        // Simple version check (in production, would use proper semver)
        Ok(compatibility.min_version <= current_version)
    }

    async fn check_dependency_compatibility(&self, module_id: &str) -> Result<bool, ModuleServiceError> {
        let dependencies = self.module_repo.get_module_dependencies(module_id).await?;
        
        for dep in dependencies {
            // Check if dependency version is compatible
            if let Some(dep_module) = self.module_repo.get_module_by_id(&dep.dependency_id).await? {
                // Version compatibility check would go here
                continue;
            } else if !dep.optional {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
}

// Supporting types and structures

#[derive(Debug, Clone)]
pub struct LoadedModule {
    pub module_id: String,
    pub tenant_id: String,
    pub module_path: String,
    pub sandbox_config: crate::types::SandboxConfig,
    pub loaded_at: chrono::DateTime<chrono::Utc>,
}

impl LoadedModule {
    pub async fn cleanup(&self) -> Result<(), ModuleServiceError> {
        // Cleanup module resources, stop processes, etc.
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct DependencyCheckResult {
    pub satisfied: bool,
    pub missing_dependencies: Vec<String>,
    pub dependent_modules: Vec<String>,
    pub can_uninstall: bool,
}

#[derive(Debug, Clone)]
pub struct CompatibilityResult {
    pub compatible: bool,
    pub issues: Vec<String>,
    pub adx_core_compatible: bool,
    pub dependencies_compatible: bool,
}

// Dependency resolver
pub struct DependencyResolver {
    // Would contain dependency resolution logic
}

impl DependencyResolver {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn resolve(&self, module_id: &str, module_repo: &ModuleRepository) -> Result<Vec<String>, ModuleServiceError> {
        let mut resolved = Vec::new();
        let mut visited = std::collections::HashSet::new();
        
        self.resolve_recursive(module_id, module_repo, &mut resolved, &mut visited).await?;
        
        Ok(resolved)
    }

    async fn resolve_recursive(
        &self,
        module_id: &str,
        module_repo: &ModuleRepository,
        resolved: &mut Vec<String>,
        visited: &mut std::collections::HashSet<String>,
    ) -> Result<(), ModuleServiceError> {
        if visited.contains(module_id) {
            return Err(ModuleServiceError::DependencyResolutionError(
                format!("Circular dependency detected: {}", module_id)
            ));
        }

        visited.insert(module_id.to_string());

        let dependencies = module_repo.get_module_dependencies(module_id).await?;
        
        for dep in dependencies {
            if !resolved.contains(&dep.dependency_id) {
                self.resolve_recursive(&dep.dependency_id, module_repo, resolved, visited).await?;
                resolved.push(dep.dependency_id);
            }
        }

        visited.remove(module_id);
        Ok(())
    }
}