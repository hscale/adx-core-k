use crate::{
    error::{WorkflowServiceError, WorkflowServiceResult},
    models::*,
    config::WorkflowServiceConfig,
};
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;
use tracing::{info, warn, error};

#[async_trait]
pub trait CrossServiceActivities: Send + Sync {
    // Auth Service Activities
    async fn create_user_account(&self, request: CreateUserAccountRequest) -> WorkflowServiceResult<CreateUserAccountResult>;
    async fn validate_user_credentials(&self, request: ValidateUserCredentialsRequest) -> WorkflowServiceResult<ValidateUserCredentialsResult>;
    async fn update_user_session(&self, request: UpdateUserSessionRequest) -> WorkflowServiceResult<UpdateUserSessionResult>;
    async fn revoke_user_sessions(&self, request: RevokeUserSessionsRequest) -> WorkflowServiceResult<RevokeUserSessionsResult>;

    // User Service Activities
    async fn create_user_profile(&self, request: CreateUserProfileRequest) -> WorkflowServiceResult<CreateUserProfileResult>;
    async fn update_user_tenant_context(&self, request: UpdateUserTenantContextRequest) -> WorkflowServiceResult<UpdateUserTenantContextResult>;
    async fn get_user_data_for_export(&self, request: GetUserDataRequest) -> WorkflowServiceResult<GetUserDataResult>;
    async fn delete_user_data(&self, request: DeleteUserDataRequest) -> WorkflowServiceResult<DeleteUserDataResult>;

    // Tenant Service Activities
    async fn validate_tenant_access(&self, request: ValidateTenantAccessRequest) -> WorkflowServiceResult<ValidateTenantAccessResult>;
    async fn get_tenant_context(&self, request: GetTenantContextRequest) -> WorkflowServiceResult<GetTenantContextResult>;
    async fn update_tenant_user_membership(&self, request: UpdateTenantUserMembershipRequest) -> WorkflowServiceResult<UpdateTenantUserMembershipResult>;
    async fn get_tenant_data_for_migration(&self, request: GetTenantDataRequest) -> WorkflowServiceResult<GetTenantDataResult>;

    // File Service Activities
    async fn setup_user_file_workspace(&self, request: SetupUserFileWorkspaceRequest) -> WorkflowServiceResult<SetupUserFileWorkspaceResult>;
    async fn migrate_user_files(&self, request: MigrateUserFilesRequest) -> WorkflowServiceResult<MigrateUserFilesResult>;
    async fn export_user_files(&self, request: ExportUserFilesRequest) -> WorkflowServiceResult<ExportUserFilesResult>;
    async fn delete_user_files(&self, request: DeleteUserFilesRequest) -> WorkflowServiceResult<DeleteUserFilesResult>;

    // Cross-Service Coordination Activities
    async fn coordinate_service_health_check(&self, services: Vec<String>) -> WorkflowServiceResult<ServiceHealthCheckResult>;
    async fn create_cross_service_backup(&self, request: CreateBackupRequest) -> WorkflowServiceResult<CreateBackupResult>;
    async fn restore_from_backup(&self, request: RestoreBackupRequest) -> WorkflowServiceResult<RestoreBackupResult>;
    async fn send_notification(&self, request: SendNotificationRequest) -> WorkflowServiceResult<SendNotificationResult>;
}

pub struct CrossServiceActivitiesImpl {
    config: WorkflowServiceConfig,
    http_client: Client,
}

impl CrossServiceActivitiesImpl {
    pub fn new(config: WorkflowServiceConfig) -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(config.server.timeout_seconds))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            config,
            http_client,
        }
    }

    async fn call_service<T: serde::de::DeserializeOwned>(
        &self,
        service_url: &str,
        endpoint: &str,
        method: &str,
        payload: Option<Value>,
        tenant_id: &str,
        user_id: Option<&str>,
    ) -> WorkflowServiceResult<T> {
        let url = format!("{}{}", service_url, endpoint);
        let mut request = match method {
            "GET" => self.http_client.get(&url),
            "POST" => self.http_client.post(&url),
            "PUT" => self.http_client.put(&url),
            "DELETE" => self.http_client.delete(&url),
            _ => return Err(WorkflowServiceError::Internal("Unsupported HTTP method".to_string())),
        };

        request = request
            .header("Content-Type", "application/json")
            .header("X-Tenant-ID", tenant_id);

        if let Some(user_id) = user_id {
            request = request.header("X-User-ID", user_id);
        }

        if let Some(payload) = payload {
            request = request.json(&payload);
        }

        let response = request.send().await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(WorkflowServiceError::ServiceCommunication {
                service: service_url.to_string(),
                message: error_text,
            });
        }

        let result = response.json::<T>().await?;
        Ok(result)
    }
}

#[async_trait]
impl CrossServiceActivities for CrossServiceActivitiesImpl {
    async fn create_user_account(&self, request: CreateUserAccountRequest) -> WorkflowServiceResult<CreateUserAccountResult> {
        info!("Creating user account for email: {}", request.email);
        
        let payload = json!({
            "email": request.email,
            "name": request.name,
            "role": request.role,
            "tenant_id": request.tenant_id,
            "send_welcome_email": request.send_welcome_email
        });

        let result = self.call_service::<CreateUserAccountResult>(
            &self.config.services.auth_service,
            "/api/v1/users",
            "POST",
            Some(payload),
            &request.tenant_id,
            None,
        ).await?;

        info!("User account created with ID: {}", result.user_id);
        Ok(result)
    }

    async fn validate_user_credentials(&self, request: ValidateUserCredentialsRequest) -> WorkflowServiceResult<ValidateUserCredentialsResult> {
        info!("Validating user credentials for user: {}", request.user_id);
        
        let result = self.call_service::<ValidateUserCredentialsResult>(
            &self.config.services.auth_service,
            &format!("/api/v1/users/{}/validate", request.user_id),
            "GET",
            None,
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn update_user_session(&self, request: UpdateUserSessionRequest) -> WorkflowServiceResult<UpdateUserSessionResult> {
        info!("Updating user session for user: {}", request.user_id);
        
        let payload = json!({
            "tenant_id": request.new_tenant_id,
            "session_data": request.session_data
        });

        let result = self.call_service::<UpdateUserSessionResult>(
            &self.config.services.auth_service,
            &format!("/api/v1/users/{}/session", request.user_id),
            "PUT",
            Some(payload),
            &request.new_tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn revoke_user_sessions(&self, request: RevokeUserSessionsRequest) -> WorkflowServiceResult<RevokeUserSessionsResult> {
        info!("Revoking user sessions for user: {}", request.user_id);
        
        let result = self.call_service::<RevokeUserSessionsResult>(
            &self.config.services.auth_service,
            &format!("/api/v1/users/{}/sessions", request.user_id),
            "DELETE",
            None,
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn create_user_profile(&self, request: CreateUserProfileRequest) -> WorkflowServiceResult<CreateUserProfileResult> {
        info!("Creating user profile for user: {}", request.user_id);
        
        let payload = json!({
            "user_id": request.user_id,
            "profile_data": request.profile_data,
            "preferences": request.preferences
        });

        let result = self.call_service::<CreateUserProfileResult>(
            &self.config.services.user_service,
            "/api/v1/profiles",
            "POST",
            Some(payload),
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn update_user_tenant_context(&self, request: UpdateUserTenantContextRequest) -> WorkflowServiceResult<UpdateUserTenantContextResult> {
        info!("Updating user tenant context for user: {} to tenant: {}", request.user_id, request.new_tenant_id);
        
        let payload = json!({
            "new_tenant_id": request.new_tenant_id,
            "preserve_preferences": request.preserve_preferences
        });

        let result = self.call_service::<UpdateUserTenantContextResult>(
            &self.config.services.user_service,
            &format!("/api/v1/users/{}/tenant-context", request.user_id),
            "PUT",
            Some(payload),
            &request.new_tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn get_user_data_for_export(&self, request: GetUserDataRequest) -> WorkflowServiceResult<GetUserDataResult> {
        info!("Getting user data for export: {}", request.user_id);
        
        let result = self.call_service::<GetUserDataResult>(
            &self.config.services.user_service,
            &format!("/api/v1/users/{}/export", request.user_id),
            "GET",
            None,
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn delete_user_data(&self, request: DeleteUserDataRequest) -> WorkflowServiceResult<DeleteUserDataResult> {
        info!("Deleting user data for user: {}", request.user_id);
        
        let payload = json!({
            "delete_options": request.delete_options
        });

        let result = self.call_service::<DeleteUserDataResult>(
            &self.config.services.user_service,
            &format!("/api/v1/users/{}", request.user_id),
            "DELETE",
            Some(payload),
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn validate_tenant_access(&self, request: ValidateTenantAccessRequest) -> WorkflowServiceResult<ValidateTenantAccessResult> {
        info!("Validating tenant access for user: {} to tenant: {}", request.user_id, request.tenant_id);
        
        let result = self.call_service::<ValidateTenantAccessResult>(
            &self.config.services.tenant_service,
            &format!("/api/v1/tenants/{}/access/{}", request.tenant_id, request.user_id),
            "GET",
            None,
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn get_tenant_context(&self, request: GetTenantContextRequest) -> WorkflowServiceResult<GetTenantContextResult> {
        info!("Getting tenant context for tenant: {}", request.tenant_id);
        
        let result = self.call_service::<GetTenantContextResult>(
            &self.config.services.tenant_service,
            &format!("/api/v1/tenants/{}/context", request.tenant_id),
            "GET",
            None,
            &request.tenant_id,
            request.user_id.as_deref(),
        ).await?;

        Ok(result)
    }

    async fn update_tenant_user_membership(&self, request: UpdateTenantUserMembershipRequest) -> WorkflowServiceResult<UpdateTenantUserMembershipResult> {
        info!("Updating tenant user membership for user: {} in tenant: {}", request.user_id, request.tenant_id);
        
        let payload = json!({
            "role": request.role,
            "permissions": request.permissions,
            "active": request.active
        });

        let result = self.call_service::<UpdateTenantUserMembershipResult>(
            &self.config.services.tenant_service,
            &format!("/api/v1/tenants/{}/users/{}", request.tenant_id, request.user_id),
            "PUT",
            Some(payload),
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn get_tenant_data_for_migration(&self, request: GetTenantDataRequest) -> WorkflowServiceResult<GetTenantDataResult> {
        info!("Getting tenant data for migration: {}", request.tenant_id);
        
        let result = self.call_service::<GetTenantDataResult>(
            &self.config.services.tenant_service,
            &format!("/api/v1/tenants/{}/export", request.tenant_id),
            "GET",
            None,
            &request.tenant_id,
            None,
        ).await?;

        Ok(result)
    }

    async fn setup_user_file_workspace(&self, request: SetupUserFileWorkspaceRequest) -> WorkflowServiceResult<SetupUserFileWorkspaceResult> {
        info!("Setting up file workspace for user: {}", request.user_id);
        
        let payload = json!({
            "user_id": request.user_id,
            "workspace_config": request.workspace_config
        });

        let result = self.call_service::<SetupUserFileWorkspaceResult>(
            &self.config.services.file_service,
            "/api/v1/workspaces",
            "POST",
            Some(payload),
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn migrate_user_files(&self, request: MigrateUserFilesRequest) -> WorkflowServiceResult<MigrateUserFilesResult> {
        info!("Migrating files for user: {}", request.user_id);
        
        let payload = json!({
            "source_tenant_id": request.source_tenant_id,
            "target_tenant_id": request.target_tenant_id,
            "migration_options": request.migration_options
        });

        let result = self.call_service::<MigrateUserFilesResult>(
            &self.config.services.file_service,
            &format!("/api/v1/users/{}/files/migrate", request.user_id),
            "POST",
            Some(payload),
            &request.target_tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn export_user_files(&self, request: ExportUserFilesRequest) -> WorkflowServiceResult<ExportUserFilesResult> {
        info!("Exporting files for user: {}", request.user_id);
        
        let result = self.call_service::<ExportUserFilesResult>(
            &self.config.services.file_service,
            &format!("/api/v1/users/{}/files/export", request.user_id),
            "GET",
            None,
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn delete_user_files(&self, request: DeleteUserFilesRequest) -> WorkflowServiceResult<DeleteUserFilesResult> {
        info!("Deleting files for user: {}", request.user_id);
        
        let payload = json!({
            "delete_options": request.delete_options
        });

        let result = self.call_service::<DeleteUserFilesResult>(
            &self.config.services.file_service,
            &format!("/api/v1/users/{}/files", request.user_id),
            "DELETE",
            Some(payload),
            &request.tenant_id,
            Some(&request.user_id),
        ).await?;

        Ok(result)
    }

    async fn coordinate_service_health_check(&self, services: Vec<String>) -> WorkflowServiceResult<ServiceHealthCheckResult> {
        info!("Coordinating health check for services: {:?}", services);
        
        let mut health_results = HashMap::new();
        
        for service in services {
            let service_url = match service.as_str() {
                "auth" => &self.config.services.auth_service,
                "user" => &self.config.services.user_service,
                "tenant" => &self.config.services.tenant_service,
                "file" => &self.config.services.file_service,
                _ => continue,
            };
            
            let health_check = self.http_client
                .get(&format!("{}/health", service_url))
                .send()
                .await;
            
            let is_healthy = health_check.map(|r| r.status().is_success()).unwrap_or(false);
            health_results.insert(service, is_healthy);
        }
        
        let all_healthy = health_results.values().all(|&healthy| healthy);
        
        Ok(ServiceHealthCheckResult {
            overall_healthy: all_healthy,
            service_results: health_results,
            checked_at: Utc::now(),
        })
    }

    async fn create_cross_service_backup(&self, request: CreateBackupRequest) -> WorkflowServiceResult<CreateBackupResult> {
        info!("Creating cross-service backup: {}", request.backup_id);
        
        // This would coordinate backup creation across all services
        // For now, return a mock result
        let backup_id = request.backup_id.clone();
        Ok(CreateBackupResult {
            backup_id: backup_id.clone(),
            backup_location: format!("/backups/{}", backup_id),
            services_backed_up: vec!["auth".to_string(), "user".to_string(), "tenant".to_string(), "file".to_string()],
            backup_size_bytes: 1024 * 1024, // Mock size
            created_at: Utc::now(),
        })
    }

    async fn restore_from_backup(&self, request: RestoreBackupRequest) -> WorkflowServiceResult<RestoreBackupResult> {
        info!("Restoring from backup: {}", request.backup_id);
        
        // This would coordinate restoration across all services
        // For now, return a mock result
        Ok(RestoreBackupResult {
            backup_id: request.backup_id,
            services_restored: vec!["auth".to_string(), "user".to_string(), "tenant".to_string(), "file".to_string()],
            records_restored: 1000, // Mock count
            restored_at: Utc::now(),
        })
    }

    async fn send_notification(&self, request: SendNotificationRequest) -> WorkflowServiceResult<SendNotificationResult> {
        info!("Sending notification: {}", request.notification_type);
        
        // This would integrate with a notification service
        // For now, return a mock result
        Ok(SendNotificationResult {
            notification_id: uuid::Uuid::new_v4().to_string(),
            sent_at: Utc::now(),
            delivery_status: "sent".to_string(),
        })
    }
}

// Activity Request/Result Types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserAccountRequest {
    pub email: String,
    pub name: String,
    pub role: String,
    pub tenant_id: String,
    pub send_welcome_email: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserAccountResult {
    pub user_id: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUserCredentialsRequest {
    pub user_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUserCredentialsResult {
    pub valid: bool,
    pub user_id: String,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserSessionRequest {
    pub user_id: String,
    pub new_tenant_id: String,
    pub session_data: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserSessionResult {
    pub session_id: String,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeUserSessionsRequest {
    pub user_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevokeUserSessionsResult {
    pub sessions_revoked: u32,
    pub revoked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserProfileRequest {
    pub user_id: String,
    pub tenant_id: String,
    pub profile_data: HashMap<String, String>,
    pub preferences: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserProfileResult {
    pub profile_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserTenantContextRequest {
    pub user_id: String,
    pub new_tenant_id: String,
    pub preserve_preferences: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserTenantContextResult {
    pub updated_at: DateTime<Utc>,
    pub new_context: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserDataRequest {
    pub user_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetUserDataResult {
    pub user_data: serde_json::Value,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserDataRequest {
    pub user_id: String,
    pub tenant_id: String,
    pub delete_options: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserDataResult {
    pub records_deleted: u64,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTenantAccessRequest {
    pub user_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateTenantAccessResult {
    pub has_access: bool,
    pub role: Option<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTenantContextRequest {
    pub tenant_id: String,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTenantContextResult {
    pub tenant_context: TenantContext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantUserMembershipRequest {
    pub user_id: String,
    pub tenant_id: String,
    pub role: String,
    pub permissions: Vec<String>,
    pub active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateTenantUserMembershipResult {
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTenantDataRequest {
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetTenantDataResult {
    pub tenant_data: serde_json::Value,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupUserFileWorkspaceRequest {
    pub user_id: String,
    pub tenant_id: String,
    pub workspace_config: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupUserFileWorkspaceResult {
    pub workspace_id: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateUserFilesRequest {
    pub user_id: String,
    pub source_tenant_id: String,
    pub target_tenant_id: String,
    pub migration_options: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateUserFilesResult {
    pub files_migrated: u64,
    pub migrated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUserFilesRequest {
    pub user_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUserFilesResult {
    pub export_path: String,
    pub files_exported: u64,
    pub exported_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserFilesRequest {
    pub user_id: String,
    pub tenant_id: String,
    pub delete_options: HashMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteUserFilesResult {
    pub files_deleted: u64,
    pub deleted_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealthCheckResult {
    pub overall_healthy: bool,
    pub service_results: HashMap<String, bool>,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBackupRequest {
    pub backup_id: String,
    pub tenant_id: String,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateBackupResult {
    pub backup_id: String,
    pub backup_location: String,
    pub services_backed_up: Vec<String>,
    pub backup_size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreBackupRequest {
    pub backup_id: String,
    pub tenant_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestoreBackupResult {
    pub backup_id: String,
    pub services_restored: Vec<String>,
    pub records_restored: u64,
    pub restored_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendNotificationRequest {
    pub notification_type: String,
    pub recipient: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendNotificationResult {
    pub notification_id: String,
    pub sent_at: DateTime<Utc>,
    pub delivery_status: String,
}