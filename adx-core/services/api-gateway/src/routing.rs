use axum::http::{Method, Uri};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::error::{ApiGatewayError, ApiResult};

/// Operation classification for intelligent routing
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    /// Simple CRUD operations that should be routed directly to services
    Direct(DirectOperation),
    /// Complex operations that should be implemented as Temporal workflows
    Workflow(WorkflowOperation),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DirectOperation {
    Read,
    Create,
    Update,
    Delete,
    List,
    Health,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowOperation {
    // User operations
    UserRegistration,
    UserOnboarding,
    UserDeactivation,
    BulkUserOperation,
    
    // Tenant operations
    TenantCreation,
    TenantMigration,
    TenantSwitching,
    TenantSuspension,
    TenantTermination,
    
    // File operations
    FileUpload,
    FileProcessing,
    FileSharing,
    FileMigration,
    BulkFileOperation,
    
    // Cross-service operations
    DataMigration,
    SystemMaintenance,
    ComplianceOperation,
    
    // Custom workflow
    Custom(String),
}

/// Service routing information
#[derive(Debug, Clone)]
pub struct ServiceRoute {
    pub service_name: String,
    pub base_url: String,
    pub timeout_seconds: u64,
}

/// Workflow routing information
#[derive(Debug, Clone)]
pub struct WorkflowRoute {
    pub workflow_type: String,
    pub task_queue: String,
    pub estimated_duration_seconds: Option<u64>,
    pub is_synchronous: bool,
}

/// Intelligent router for API Gateway
pub struct IntelligentRouter {
    service_routes: HashMap<String, ServiceRoute>,
    workflow_routes: HashMap<String, WorkflowRoute>,
}

impl IntelligentRouter {
    pub fn new() -> Self {
        let mut router = Self {
            service_routes: HashMap::new(),
            workflow_routes: HashMap::new(),
        };
        
        router.initialize_default_routes();
        router
    }

    /// Initialize default routing rules
    fn initialize_default_routes(&mut self) {
        // Service routes
        self.add_service_route("auth", "http://localhost:8081", 10);
        self.add_service_route("user", "http://localhost:8082", 10);
        self.add_service_route("file", "http://localhost:8083", 30);
        self.add_service_route("workflow", "http://localhost:8084", 60);
        self.add_service_route("tenant", "http://localhost:8085", 10);

        // Workflow routes
        self.add_workflow_route("user_registration", "user-task-queue", Some(30), false);
        self.add_workflow_route("user_onboarding", "user-task-queue", Some(60), false);
        self.add_workflow_route("create_tenant", "tenant-task-queue", Some(120), false);
        self.add_workflow_route("migrate_tenant", "tenant-task-queue", Some(300), false);
        self.add_workflow_route("switch_tenant", "tenant-task-queue", Some(10), true);
        self.add_workflow_route("file_upload", "file-task-queue", Some(180), false);
        self.add_workflow_route("file_processing", "file-task-queue", Some(300), false);
        self.add_workflow_route("bulk_operation", "bulk-task-queue", Some(600), false);
    }

    /// Add a service route
    pub fn add_service_route(&mut self, service: &str, base_url: &str, timeout_seconds: u64) {
        self.service_routes.insert(
            service.to_string(),
            ServiceRoute {
                service_name: service.to_string(),
                base_url: base_url.to_string(),
                timeout_seconds,
            },
        );
    }

    /// Add a workflow route
    pub fn add_workflow_route(
        &mut self,
        workflow_type: &str,
        task_queue: &str,
        estimated_duration_seconds: Option<u64>,
        is_synchronous: bool,
    ) {
        self.workflow_routes.insert(
            workflow_type.to_string(),
            WorkflowRoute {
                workflow_type: workflow_type.to_string(),
                task_queue: task_queue.to_string(),
                estimated_duration_seconds,
                is_synchronous,
            },
        );
    }

    /// Classify an operation based on HTTP method and path
    pub fn classify_operation(&self, method: &Method, path: &str) -> ApiResult<OperationType> {
        debug!(
            method = %method,
            path = path,
            "Classifying operation for intelligent routing"
        );

        // Workflow operations (complex multi-step processes)
        if path.starts_with("/api/v1/workflows/") {
            return self.classify_workflow_operation(path);
        }

        // Direct operations (simple CRUD)
        match (method.as_str(), path) {
            // Health checks
            ("GET", "/health") | ("GET", "/api/v1/health") => {
                Ok(OperationType::Direct(DirectOperation::Health))
            }

            // Auth service - direct operations
            ("POST", "/api/v1/auth/login") |
            ("POST", "/api/v1/auth/refresh") |
            ("POST", "/api/v1/auth/logout") => {
                Ok(OperationType::Direct(DirectOperation::Create))
            }

            // User service - direct operations
            ("GET", path) if path.starts_with("/api/v1/users") && !path.contains("/bulk") => {
                if path.ends_with("/users") {
                    Ok(OperationType::Direct(DirectOperation::List))
                } else {
                    Ok(OperationType::Direct(DirectOperation::Read))
                }
            }
            ("PUT", path) if path.starts_with("/api/v1/users/") && !path.contains("/bulk") => {
                Ok(OperationType::Direct(DirectOperation::Update))
            }
            ("DELETE", path) if path.starts_with("/api/v1/users/") && !path.contains("/bulk") => {
                Ok(OperationType::Direct(DirectOperation::Delete))
            }

            // Tenant service - direct operations
            ("GET", path) if path.starts_with("/api/v1/tenants") => {
                if path.ends_with("/tenants") {
                    Ok(OperationType::Direct(DirectOperation::List))
                } else {
                    Ok(OperationType::Direct(DirectOperation::Read))
                }
            }
            ("PUT", path) if path.starts_with("/api/v1/tenants/") => {
                Ok(OperationType::Direct(DirectOperation::Update))
            }

            // File service - direct operations (small files only)
            ("GET", path) if path.starts_with("/api/v1/files") => {
                if path.ends_with("/files") {
                    Ok(OperationType::Direct(DirectOperation::List))
                } else {
                    Ok(OperationType::Direct(DirectOperation::Read))
                }
            }

            // Complex operations that should be workflows
            ("POST", "/api/v1/users") => {
                // User creation can be complex with onboarding
                Ok(OperationType::Workflow(WorkflowOperation::UserRegistration))
            }
            ("POST", path) if path.starts_with("/api/v1/users/") && path.contains("/bulk") => {
                Ok(OperationType::Workflow(WorkflowOperation::BulkUserOperation))
            }
            ("POST", "/api/v1/tenants") => {
                Ok(OperationType::Workflow(WorkflowOperation::TenantCreation))
            }
            ("DELETE", path) if path.starts_with("/api/v1/tenants/") => {
                Ok(OperationType::Workflow(WorkflowOperation::TenantTermination))
            }
            ("POST", path) if path.starts_with("/api/v1/files") => {
                Ok(OperationType::Workflow(WorkflowOperation::FileUpload))
            }
            ("POST", path) if path.contains("/switch-tenant") => {
                Ok(OperationType::Workflow(WorkflowOperation::TenantSwitching))
            }

            // Default to direct operation for unmatched patterns
            ("GET", _) => Ok(OperationType::Direct(DirectOperation::Read)),
            ("POST", _) => Ok(OperationType::Direct(DirectOperation::Create)),
            ("PUT", _) => Ok(OperationType::Direct(DirectOperation::Update)),
            ("DELETE", _) => Ok(OperationType::Direct(DirectOperation::Delete)),
            
            _ => Err(ApiGatewayError::InvalidRequest {
                message: format!("Unsupported operation: {} {}", method, path),
            }),
        }
    }

    /// Classify workflow operations
    fn classify_workflow_operation(&self, path: &str) -> ApiResult<OperationType> {
        let workflow_type = path
            .strip_prefix("/api/v1/workflows/")
            .ok_or_else(|| ApiGatewayError::InvalidRequest {
                message: "Invalid workflow path".to_string(),
            })?
            .split('/')
            .next()
            .unwrap_or("");

        let operation = match workflow_type {
            "user-registration" => WorkflowOperation::UserRegistration,
            "user-onboarding" => WorkflowOperation::UserOnboarding,
            "user-deactivation" => WorkflowOperation::UserDeactivation,
            "bulk-user-operation" => WorkflowOperation::BulkUserOperation,
            "create-tenant" => WorkflowOperation::TenantCreation,
            "migrate-tenant" => WorkflowOperation::TenantMigration,
            "switch-tenant" => WorkflowOperation::TenantSwitching,
            "suspend-tenant" => WorkflowOperation::TenantSuspension,
            "terminate-tenant" => WorkflowOperation::TenantTermination,
            "file-upload" => WorkflowOperation::FileUpload,
            "file-processing" => WorkflowOperation::FileProcessing,
            "file-sharing" => WorkflowOperation::FileSharing,
            "file-migration" => WorkflowOperation::FileMigration,
            "bulk-file-operation" => WorkflowOperation::BulkFileOperation,
            "data-migration" => WorkflowOperation::DataMigration,
            "system-maintenance" => WorkflowOperation::SystemMaintenance,
            "compliance-operation" => WorkflowOperation::ComplianceOperation,
            _ => WorkflowOperation::Custom(workflow_type.to_string()),
        };

        Ok(OperationType::Workflow(operation))
    }

    /// Get service route for direct operations
    pub fn get_service_route(&self, _operation: &DirectOperation, path: &str) -> ApiResult<ServiceRoute> {
        let service_name = self.extract_service_name(path)?;
        
        self.service_routes
            .get(&service_name)
            .cloned()
            .ok_or_else(|| ApiGatewayError::ServiceUnavailable {
                service: service_name,
            })
    }

    /// Get workflow route for workflow operations
    pub fn get_workflow_route(&self, operation: &WorkflowOperation) -> ApiResult<WorkflowRoute> {
        let workflow_type = match operation {
            WorkflowOperation::UserRegistration => "user_registration",
            WorkflowOperation::UserOnboarding => "user_onboarding",
            WorkflowOperation::UserDeactivation => "user_deactivation",
            WorkflowOperation::BulkUserOperation => "bulk_user_operation",
            WorkflowOperation::TenantCreation => "create_tenant",
            WorkflowOperation::TenantMigration => "migrate_tenant",
            WorkflowOperation::TenantSwitching => "switch_tenant",
            WorkflowOperation::TenantSuspension => "suspend_tenant",
            WorkflowOperation::TenantTermination => "terminate_tenant",
            WorkflowOperation::FileUpload => "file_upload",
            WorkflowOperation::FileProcessing => "file_processing",
            WorkflowOperation::FileSharing => "file_sharing",
            WorkflowOperation::FileMigration => "file_migration",
            WorkflowOperation::BulkFileOperation => "bulk_file_operation",
            WorkflowOperation::DataMigration => "data_migration",
            WorkflowOperation::SystemMaintenance => "system_maintenance",
            WorkflowOperation::ComplianceOperation => "compliance_operation",
            WorkflowOperation::Custom(name) => name,
        };

        self.workflow_routes
            .get(workflow_type)
            .cloned()
            .ok_or_else(|| ApiGatewayError::WorkflowNotFound {
                workflow_id: workflow_type.to_string(),
            })
    }

    /// Extract service name from path
    fn extract_service_name(&self, path: &str) -> ApiResult<String> {
        if path.starts_with("/api/v1/") {
            let parts: Vec<&str> = path.split('/').collect();
            if parts.len() >= 4 {
                let service = match parts[3] {
                    "auth" => "auth",
                    "users" => "user",
                    "tenants" => "tenant",
                    "files" => "file",
                    "workflows" => "workflow",
                    _ => return Err(ApiGatewayError::InvalidRequest {
                        message: format!("Unknown service in path: {}", path),
                    }),
                };
                return Ok(service.to_string());
            }
        }

        // Health endpoints
        if path == "/health" || path == "/api/v1/health" {
            return Ok("health".to_string());
        }

        Err(ApiGatewayError::InvalidRequest {
            message: format!("Cannot extract service name from path: {}", path),
        })
    }

    /// Build target URL for service routing
    pub fn build_service_url(&self, service_route: &ServiceRoute, path: &str) -> String {
        format!("{}{}", service_route.base_url, path)
    }
}

impl Default for IntelligentRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_operation_classification() {
        let router = IntelligentRouter::new();

        // Test direct operations
        let result = router.classify_operation(&Method::GET, "/api/v1/users/123");
        assert!(matches!(result, Ok(OperationType::Direct(DirectOperation::Read))));

        let result = router.classify_operation(&Method::GET, "/api/v1/users");
        assert!(matches!(result, Ok(OperationType::Direct(DirectOperation::List))));

        let result = router.classify_operation(&Method::PUT, "/api/v1/users/123");
        assert!(matches!(result, Ok(OperationType::Direct(DirectOperation::Update))));

        // Test workflow operations
        let result = router.classify_operation(&Method::POST, "/api/v1/users");
        assert!(matches!(result, Ok(OperationType::Workflow(WorkflowOperation::UserRegistration))));

        let result = router.classify_operation(&Method::POST, "/api/v1/tenants");
        assert!(matches!(result, Ok(OperationType::Workflow(WorkflowOperation::TenantCreation))));

        let result = router.classify_operation(&Method::POST, "/api/v1/workflows/create-tenant");
        assert!(matches!(result, Ok(OperationType::Workflow(WorkflowOperation::TenantCreation))));
    }

    #[test]
    fn test_service_name_extraction() {
        let router = IntelligentRouter::new();

        assert_eq!(router.extract_service_name("/api/v1/users/123").unwrap(), "user");
        assert_eq!(router.extract_service_name("/api/v1/tenants").unwrap(), "tenant");
        assert_eq!(router.extract_service_name("/api/v1/files/upload").unwrap(), "file");
        assert_eq!(router.extract_service_name("/health").unwrap(), "health");

        assert!(router.extract_service_name("/invalid/path").is_err());
    }

    #[test]
    fn test_service_route_retrieval() {
        let router = IntelligentRouter::new();

        let route = router.service_routes.get("user").unwrap();
        assert_eq!(route.service_name, "user");
        assert_eq!(route.base_url, "http://localhost:8082");
        assert_eq!(route.timeout_seconds, 10);
    }

    #[test]
    fn test_workflow_route_retrieval() {
        let router = IntelligentRouter::new();

        let route = router.workflow_routes.get("create_tenant").unwrap();
        assert_eq!(route.workflow_type, "create_tenant");
        assert_eq!(route.task_queue, "tenant-task-queue");
        assert_eq!(route.estimated_duration_seconds, Some(120));
        assert!(!route.is_synchronous);
    }

    #[test]
    fn test_url_building() {
        let router = IntelligentRouter::new();
        let service_route = ServiceRoute {
            service_name: "user".to_string(),
            base_url: "http://localhost:8082".to_string(),
            timeout_seconds: 10,
        };

        let url = router.build_service_url(&service_route, "/api/v1/users/123");
        assert_eq!(url, "http://localhost:8082/api/v1/users/123");
    }
}