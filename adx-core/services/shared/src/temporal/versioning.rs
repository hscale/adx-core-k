use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn, error};

use crate::temporal::TemporalError;

/// Workflow version information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct WorkflowVersion {
    /// Major version (breaking changes)
    pub major: u32,
    
    /// Minor version (backward compatible changes)
    pub minor: u32,
    
    /// Patch version (bug fixes)
    pub patch: u32,
    
    /// Pre-release identifier (alpha, beta, rc)
    pub pre_release: Option<String>,
}

impl WorkflowVersion {
    /// Create a new workflow version
    pub fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: None,
        }
    }
    
    /// Create a pre-release version
    pub fn pre_release(major: u32, minor: u32, patch: u32, pre_release: String) -> Self {
        Self {
            major,
            minor,
            patch,
            pre_release: Some(pre_release),
        }
    }
    
    /// Parse version from string (e.g., "1.2.3" or "1.2.3-beta.1")
    pub fn parse(version_str: &str) -> Result<Self, TemporalError> {
        let parts: Vec<&str> = version_str.split('-').collect();
        let version_part = parts[0];
        let pre_release = parts.get(1).map(|s| s.to_string());
        
        let version_numbers: Vec<&str> = version_part.split('.').collect();
        if version_numbers.len() != 3 {
            return Err(TemporalError::VersioningError {
                workflow_type: "unknown".to_string(),
                message: format!("Invalid version format: {}", version_str),
            });
        }
        
        let major = version_numbers[0].parse::<u32>()
            .map_err(|_| TemporalError::VersioningError {
                workflow_type: "unknown".to_string(),
                message: format!("Invalid major version: {}", version_numbers[0]),
            })?;
        
        let minor = version_numbers[1].parse::<u32>()
            .map_err(|_| TemporalError::VersioningError {
                workflow_type: "unknown".to_string(),
                message: format!("Invalid minor version: {}", version_numbers[1]),
            })?;
        
        let patch = version_numbers[2].parse::<u32>()
            .map_err(|_| TemporalError::VersioningError {
                workflow_type: "unknown".to_string(),
                message: format!("Invalid patch version: {}", version_numbers[2]),
            })?;
        
        Ok(Self {
            major,
            minor,
            patch,
            pre_release,
        })
    }
    
    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &WorkflowVersion) -> bool {
        // Major version must match for compatibility
        if self.major != other.major {
            return false;
        }
        
        // Minor version can be higher (backward compatible)
        if self.minor < other.minor {
            return false;
        }
        
        // If minor versions match, patch can be higher
        if self.minor == other.minor && self.patch < other.patch {
            return false;
        }
        
        true
    }
    
    /// Check if this version is newer than another version
    pub fn is_newer_than(&self, other: &WorkflowVersion) -> bool {
        if self.major != other.major {
            return self.major > other.major;
        }
        
        if self.minor != other.minor {
            return self.minor > other.minor;
        }
        
        if self.patch != other.patch {
            return self.patch > other.patch;
        }
        
        // Handle pre-release versions
        match (&self.pre_release, &other.pre_release) {
            (None, Some(_)) => true,  // Release is newer than pre-release
            (Some(_), None) => false, // Pre-release is older than release
            (None, None) => false,    // Same version
            (Some(a), Some(b)) => a > b, // Compare pre-release strings
        }
    }
}

impl std::fmt::Display for WorkflowVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(pre_release) = &self.pre_release {
            write!(f, "{}.{}.{}-{}", self.major, self.minor, self.patch, pre_release)
        } else {
            write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
        }
    }
}

/// Workflow version registry for managing workflow versions
#[derive(Debug, Clone)]
pub struct WorkflowVersionRegistry {
    /// Map of workflow type to available versions
    versions: HashMap<String, Vec<WorkflowVersion>>,
    
    /// Map of workflow type to default version
    default_versions: HashMap<String, WorkflowVersion>,
    
    /// Map of workflow type to deprecated versions
    deprecated_versions: HashMap<String, Vec<WorkflowVersion>>,
}

impl WorkflowVersionRegistry {
    /// Create a new workflow version registry
    pub fn new() -> Self {
        Self {
            versions: HashMap::new(),
            default_versions: HashMap::new(),
            deprecated_versions: HashMap::new(),
        }
    }
    
    /// Register a new workflow version
    pub fn register_version(
        &mut self,
        workflow_type: &str,
        version: WorkflowVersion,
        is_default: bool,
    ) -> Result<(), TemporalError> {
        info!(
            workflow_type = workflow_type,
            version = %version,
            is_default = is_default,
            "Registering workflow version"
        );
        
        // Add to versions list
        let versions = self.versions.entry(workflow_type.to_string()).or_insert_with(Vec::new);
        
        // Check if version already exists
        if versions.contains(&version) {
            return Err(TemporalError::VersioningError {
                workflow_type: workflow_type.to_string(),
                message: format!("Version {} already registered", version),
            });
        }
        
        versions.push(version.clone());
        versions.sort_by(|a, b| {
            if a.is_newer_than(b) {
                std::cmp::Ordering::Greater
            } else if b.is_newer_than(a) {
                std::cmp::Ordering::Less
            } else {
                std::cmp::Ordering::Equal
            }
        });
        
        // Set as default if specified
        if is_default {
            self.default_versions.insert(workflow_type.to_string(), version);
        }
        
        Ok(())
    }
    
    /// Get the default version for a workflow type
    pub fn get_default_version(&self, workflow_type: &str) -> Option<&WorkflowVersion> {
        self.default_versions.get(workflow_type)
    }
    
    /// Get all versions for a workflow type
    pub fn get_versions(&self, workflow_type: &str) -> Option<&Vec<WorkflowVersion>> {
        self.versions.get(workflow_type)
    }
    
    /// Get the latest version for a workflow type
    pub fn get_latest_version(&self, workflow_type: &str) -> Option<&WorkflowVersion> {
        self.versions.get(workflow_type)?.last()
    }
    
    /// Check if a version is supported for a workflow type
    pub fn is_version_supported(&self, workflow_type: &str, version: &WorkflowVersion) -> bool {
        if let Some(versions) = self.versions.get(workflow_type) {
            versions.contains(version)
        } else {
            false
        }
    }
    
    /// Mark a version as deprecated
    pub fn deprecate_version(
        &mut self,
        workflow_type: &str,
        version: WorkflowVersion,
    ) -> Result<(), TemporalError> {
        warn!(
            workflow_type = workflow_type,
            version = %version,
            "Deprecating workflow version"
        );
        
        // Check if version exists
        if !self.is_version_supported(workflow_type, &version) {
            return Err(TemporalError::VersioningError {
                workflow_type: workflow_type.to_string(),
                message: format!("Version {} not found", version),
            });
        }
        
        // Add to deprecated versions
        let deprecated = self.deprecated_versions
            .entry(workflow_type.to_string())
            .or_insert_with(Vec::new);
        
        if !deprecated.contains(&version) {
            deprecated.push(version);
        }
        
        Ok(())
    }
    
    /// Check if a version is deprecated
    pub fn is_version_deprecated(&self, workflow_type: &str, version: &WorkflowVersion) -> bool {
        if let Some(deprecated) = self.deprecated_versions.get(workflow_type) {
            deprecated.contains(version)
        } else {
            false
        }
    }
    
    /// Get compatible versions for a given version
    pub fn get_compatible_versions(
        &self,
        workflow_type: &str,
        version: &WorkflowVersion,
    ) -> Vec<&WorkflowVersion> {
        if let Some(versions) = self.versions.get(workflow_type) {
            versions.iter()
                .filter(|v| v.is_compatible_with(version))
                .collect()
        } else {
            Vec::new()
        }
    }
    
    /// Validate workflow version compatibility
    pub fn validate_compatibility(
        &self,
        workflow_type: &str,
        requested_version: &WorkflowVersion,
        running_version: &WorkflowVersion,
    ) -> Result<(), TemporalError> {
        // Check if both versions are supported
        if !self.is_version_supported(workflow_type, requested_version) {
            return Err(TemporalError::VersioningError {
                workflow_type: workflow_type.to_string(),
                message: format!("Requested version {} is not supported", requested_version),
            });
        }
        
        if !self.is_version_supported(workflow_type, running_version) {
            return Err(TemporalError::VersioningError {
                workflow_type: workflow_type.to_string(),
                message: format!("Running version {} is not supported", running_version),
            });
        }
        
        // Check compatibility
        if !requested_version.is_compatible_with(running_version) {
            return Err(TemporalError::VersioningError {
                workflow_type: workflow_type.to_string(),
                message: format!(
                    "Version {} is not compatible with running version {}",
                    requested_version, running_version
                ),
            });
        }
        
        // Warn if using deprecated version
        if self.is_version_deprecated(workflow_type, requested_version) {
            warn!(
                workflow_type = workflow_type,
                version = %requested_version,
                "Using deprecated workflow version"
            );
        }
        
        Ok(())
    }
}

impl Default for WorkflowVersionRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow version manager for ADX Core
pub struct AdxWorkflowVersionManager {
    registry: WorkflowVersionRegistry,
}

impl AdxWorkflowVersionManager {
    /// Create a new workflow version manager
    pub fn new() -> Self {
        let mut registry = WorkflowVersionRegistry::new();
        
        // Register default ADX Core workflow versions
        Self::register_default_versions(&mut registry);
        
        Self { registry }
    }
    
    /// Register default workflow versions for ADX Core
    fn register_default_versions(registry: &mut WorkflowVersionRegistry) {
        let default_workflows = vec![
            ("user_registration_workflow", "1.0.0"),
            ("password_reset_workflow", "1.0.0"),
            ("mfa_setup_workflow", "1.0.0"),
            ("sso_authentication_workflow", "1.0.0"),
            ("user_onboarding_workflow", "1.0.0"),
            ("tenant_provisioning_workflow", "1.0.0"),
            ("tenant_monitoring_workflow", "1.0.0"),
            ("tenant_upgrade_workflow", "1.0.0"),
            ("tenant_suspension_workflow", "1.0.0"),
            ("tenant_termination_workflow", "1.0.0"),
            ("tenant_switching_workflow", "1.0.0"),
            ("user_profile_sync_workflow", "1.0.0"),
            ("user_preference_migration_workflow", "1.0.0"),
            ("user_data_export_workflow", "1.0.0"),
            ("user_deactivation_workflow", "1.0.0"),
            ("user_reactivation_workflow", "1.0.0"),
            ("bulk_user_operation_workflow", "1.0.0"),
            ("file_upload_workflow", "1.0.0"),
            ("file_processing_workflow", "1.0.0"),
            ("file_sharing_workflow", "1.0.0"),
            ("file_migration_workflow", "1.0.0"),
            ("file_cleanup_workflow", "1.0.0"),
            ("bulk_file_operation_workflow", "1.0.0"),
        ];
        
        for (workflow_type, version_str) in default_workflows {
            if let Ok(version) = WorkflowVersion::parse(version_str) {
                if let Err(e) = registry.register_version(workflow_type, version, true) {
                    error!(
                        workflow_type = workflow_type,
                        version = version_str,
                        error = %e,
                        "Failed to register default workflow version"
                    );
                }
            }
        }
    }
    
    /// Get the registry
    pub fn registry(&self) -> &WorkflowVersionRegistry {
        &self.registry
    }
    
    /// Get mutable registry
    pub fn registry_mut(&mut self) -> &mut WorkflowVersionRegistry {
        &mut self.registry
    }
    
    /// Register a new workflow version
    pub fn register_version(
        &mut self,
        workflow_type: &str,
        version: WorkflowVersion,
        is_default: bool,
    ) -> Result<(), TemporalError> {
        self.registry.register_version(workflow_type, version, is_default)
    }
    
    /// Get workflow version for execution
    pub fn get_execution_version(
        &self,
        workflow_type: &str,
        requested_version: Option<&WorkflowVersion>,
    ) -> Result<WorkflowVersion, TemporalError> {
        match requested_version {
            Some(version) => {
                // Validate requested version
                if !self.registry.is_version_supported(workflow_type, version) {
                    return Err(TemporalError::VersioningError {
                        workflow_type: workflow_type.to_string(),
                        message: format!("Requested version {} is not supported", version),
                    });
                }
                Ok(version.clone())
            }
            None => {
                // Use default version
                self.registry.get_default_version(workflow_type)
                    .cloned()
                    .ok_or_else(|| TemporalError::VersioningError {
                        workflow_type: workflow_type.to_string(),
                        message: "No default version found".to_string(),
                    })
            }
        }
    }
}

impl Default for AdxWorkflowVersionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_workflow_version_parsing() {
        let version = WorkflowVersion::parse("1.2.3").unwrap();
        assert_eq!(version.major, 1);
        assert_eq!(version.minor, 2);
        assert_eq!(version.patch, 3);
        assert_eq!(version.pre_release, None);
        
        let pre_release_version = WorkflowVersion::parse("1.2.3-beta.1").unwrap();
        assert_eq!(pre_release_version.major, 1);
        assert_eq!(pre_release_version.minor, 2);
        assert_eq!(pre_release_version.patch, 3);
        assert_eq!(pre_release_version.pre_release, Some("beta.1".to_string()));
    }
    
    #[test]
    fn test_version_compatibility() {
        let v1_0_0 = WorkflowVersion::new(1, 0, 0);
        let v1_1_0 = WorkflowVersion::new(1, 1, 0);
        let v1_1_1 = WorkflowVersion::new(1, 1, 1);
        let v2_0_0 = WorkflowVersion::new(2, 0, 0);
        
        // Same major version, higher minor/patch is compatible
        assert!(v1_1_0.is_compatible_with(&v1_0_0));
        assert!(v1_1_1.is_compatible_with(&v1_1_0));
        
        // Lower minor/patch is not compatible
        assert!(!v1_0_0.is_compatible_with(&v1_1_0));
        
        // Different major version is not compatible
        assert!(!v2_0_0.is_compatible_with(&v1_0_0));
        assert!(!v1_0_0.is_compatible_with(&v2_0_0));
    }
    
    #[test]
    fn test_version_comparison() {
        let v1_0_0 = WorkflowVersion::new(1, 0, 0);
        let v1_1_0 = WorkflowVersion::new(1, 1, 0);
        let v2_0_0 = WorkflowVersion::new(2, 0, 0);
        
        assert!(v1_1_0.is_newer_than(&v1_0_0));
        assert!(v2_0_0.is_newer_than(&v1_1_0));
        assert!(!v1_0_0.is_newer_than(&v1_1_0));
    }
    
    #[test]
    fn test_version_registry() {
        let mut registry = WorkflowVersionRegistry::new();
        
        let v1_0_0 = WorkflowVersion::new(1, 0, 0);
        let v1_1_0 = WorkflowVersion::new(1, 1, 0);
        
        registry.register_version("test_workflow", v1_0_0.clone(), true).unwrap();
        registry.register_version("test_workflow", v1_1_0.clone(), false).unwrap();
        
        assert_eq!(registry.get_default_version("test_workflow"), Some(&v1_0_0));
        assert_eq!(registry.get_latest_version("test_workflow"), Some(&v1_1_0));
        assert!(registry.is_version_supported("test_workflow", &v1_0_0));
        assert!(registry.is_version_supported("test_workflow", &v1_1_0));
    }
    
    #[test]
    fn test_version_deprecation() {
        let mut registry = WorkflowVersionRegistry::new();
        
        let v1_0_0 = WorkflowVersion::new(1, 0, 0);
        registry.register_version("test_workflow", v1_0_0.clone(), true).unwrap();
        
        assert!(!registry.is_version_deprecated("test_workflow", &v1_0_0));
        
        registry.deprecate_version("test_workflow", v1_0_0.clone()).unwrap();
        assert!(registry.is_version_deprecated("test_workflow", &v1_0_0));
    }
}