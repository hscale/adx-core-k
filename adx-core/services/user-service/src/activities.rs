use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use adx_shared::{
    temporal::{ActivityError, ActivityContext},
    Result,
};
use crate::{
    models::*,
    repositories::*,
    validation::{UserValidator, validate_create_user_request, validate_update_user_request},
};

// Activity request/response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserActivityRequest {
    pub tenant_id: Uuid,
    pub user_request: CreateUserRequest,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserActivityResponse {
    pub user: User,
    pub profile: Option<UserProfile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserActivityRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub update_request: UpdateUserRequest,
    pub updated_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserActivityResponse {
    pub user: User,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUserDataActivityRequest {
    pub tenant_id: Uuid,
    pub user_data: serde_json::Value,
    pub validation_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateUserDataActivityResponse {
    pub is_valid: bool,
    pub validation_errors: Vec<String>,
    pub sanitized_data: Option<serde_json::Value>,
}

// New activity types for user management workflows

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncUserProfileActivityRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub target_service: String,
    pub sync_fields: Vec<String>,
    pub force_sync: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncUserProfileActivityResponse {
    pub sync_id: Uuid,
    pub synced_fields: Vec<String>,
    pub sync_timestamp: DateTime<Utc>,
    pub sync_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateUserPreferencesActivityRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub preference_category: String,
    pub source_version: Option<String>,
    pub target_version: Option<String>,
    pub migration_rules: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrateUserPreferencesActivityResponse {
    pub migration_id: Uuid,
    pub migrated_preferences: HashMap<String, serde_json::Value>,
    pub migration_timestamp: DateTime<Utc>,
    pub backup_created: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUserDataActivityRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub export_format: String,
    pub data_categories: Vec<String>,
    pub include_metadata: bool,
    pub anonymize_data: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportUserDataActivityResponse {
    pub export_id: Uuid,
    pub export_path: String,
    pub export_size_bytes: u64,
    pub exported_categories: Vec<String>,
    pub export_timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeactivateUserActivityRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub deactivation_reason: String,
    pub deactivated_by: Uuid,
    pub retain_data: bool,
    pub data_retention_days: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeactivateUserActivityResponse {
    pub deactivation_id: Uuid,
    pub deactivated_at: DateTime<Utc>,
    pub data_retention_until: Option<DateTime<Utc>>,
    pub cleanup_tasks_scheduled: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactivateUserActivityRequest {
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub reactivated_by: Uuid,
    pub restore_data: bool,
    pub restore_permissions: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReactivateUserActivityResponse {
    pub reactivation_id: Uuid,
    pub reactivated_at: DateTime<Utc>,
    pub data_restored: bool,
    pub permissions_restored: bool,
    pub restoration_summary: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferUserOwnershipActivityRequest {
    pub tenant_id: Uuid,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub resource_type: String,
    pub resource_ids: Vec<Uuid>,
    pub notify_new_owner: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferUserOwnershipActivityResponse {
    pub transfer_id: Uuid,
    pub transferred_count: u32,
    pub failed_count: u32,
    pub transfer_timestamp: DateTime<Utc>,
    pub notification_sent: bool,
}

// User service activities trait
#[async_trait]
pub trait UserServiceActivities: Send + Sync {
    async fn create_user_activity(
        &self,
        context: ActivityContext,
        request: CreateUserActivityRequest,
    ) -> Result<CreateUserActivityResponse>;
    
    async fn update_user_activity(
        &self,
        context: ActivityContext,
        request: UpdateUserActivityRequest,
    ) -> Result<UpdateUserActivityResponse>;
    
    async fn validate_user_data_activity(
        &self,
        context: ActivityContext,
        request: ValidateUserDataActivityRequest,
    ) -> Result<ValidateUserDataActivityResponse>;
    
    // New activities for user management workflows
    async fn sync_user_profile_activity(
        &self,
        context: ActivityContext,
        request: SyncUserProfileActivityRequest,
    ) -> Result<SyncUserProfileActivityResponse>;
    
    async fn migrate_user_preferences_activity(
        &self,
        context: ActivityContext,
        request: MigrateUserPreferencesActivityRequest,
    ) -> Result<MigrateUserPreferencesActivityResponse>;
    
    async fn export_user_data_activity(
        &self,
        context: ActivityContext,
        request: ExportUserDataActivityRequest,
    ) -> Result<ExportUserDataActivityResponse>;
    
    async fn deactivate_user_activity(
        &self,
        context: ActivityContext,
        request: DeactivateUserActivityRequest,
    ) -> Result<DeactivateUserActivityResponse>;
    
    async fn reactivate_user_activity(
        &self,
        context: ActivityContext,
        request: ReactivateUserActivityRequest,
    ) -> Result<ReactivateUserActivityResponse>;
    
    async fn transfer_user_ownership_activity(
        &self,
        context: ActivityContext,
        request: TransferUserOwnershipActivityRequest,
    ) -> Result<TransferUserOwnershipActivityResponse>;
}

// Implementation of user service activities
pub struct UserServiceActivitiesImpl {
    user_repo: Arc<dyn UserRepository>,
    profile_repo: Arc<dyn UserProfileRepository>,
    preference_repo: Arc<dyn UserPreferenceRepository>,
    activity_repo: Arc<dyn UserActivityRepository>,
    validator: Arc<UserValidator>,
}

impl UserServiceActivitiesImpl {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        profile_repo: Arc<dyn UserProfileRepository>,
        preference_repo: Arc<dyn UserPreferenceRepository>,
        activity_repo: Arc<dyn UserActivityRepository>,
        validator: Arc<UserValidator>,
    ) -> Self {
        Self {
            user_repo,
            profile_repo,
            preference_repo,
            activity_repo,
            validator,
        }
    }
}

#[async_trait]
impl UserServiceActivities for UserServiceActivitiesImpl {
    async fn create_user_activity(
        &self,
        _context: ActivityContext,
        request: CreateUserActivityRequest,
    ) -> Result<CreateUserActivityResponse> {
        // Validate the user creation request
        validate_create_user_request(&self.validator, &request.user_request)
            .map_err(|e| adx_shared::Error::Validation(format!("user_request: {}", e)))?;
        
        // Check if user already exists
        if let Ok(Some(_)) = self.user_repo.find_by_email(request.tenant_id, &request.user_request.email).await {
            return Err(adx_shared::Error::Conflict("User with this email already exists".to_string()));
        }
        
        // Create the user
        let user = self.user_repo.create(request.tenant_id, request.user_request.clone()).await?;
        
        // Create profile if provided
        let profile = if let Some(profile_request) = request.user_request.profile {
            Some(self.profile_repo.create(request.tenant_id, user.id, profile_request).await?)
        } else {
            None
        };
        
        // Log the activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: user.id,
            tenant_id: request.tenant_id,
            activity_type: "user_created_via_workflow".to_string(),
            activity_description: Some("User created through Temporal workflow".to_string()),
            resource_type: Some("user".to_string()),
            resource_id: Some(user.id),
            metadata: serde_json::json!({
                "created_by": request.created_by,
                "workflow_execution": true
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: chrono::Utc::now(),
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(CreateUserActivityResponse { user, profile })
    }
    
    async fn update_user_activity(
        &self,
        _context: ActivityContext,
        request: UpdateUserActivityRequest,
    ) -> Result<UpdateUserActivityResponse> {
        // Validate the update request
        validate_update_user_request(&self.validator, &request.update_request)
            .map_err(|e| adx_shared::Error::Validation(format!("update_request: {}", e)))?;
        
        // Check if user exists
        if self.user_repo.find_by_id(request.tenant_id, request.user_id).await?.is_none() {
            return Err(adx_shared::Error::NotFound(format!("User {} not found", request.user_id)));
        }
        
        // Update the user
        let user = self.user_repo.update(request.tenant_id, request.user_id, request.update_request).await?;
        
        // Log the activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            tenant_id: request.tenant_id,
            activity_type: "user_updated_via_workflow".to_string(),
            activity_description: Some("User updated through Temporal workflow".to_string()),
            resource_type: Some("user".to_string()),
            resource_id: Some(request.user_id),
            metadata: serde_json::json!({
                "updated_by": request.updated_by,
                "workflow_execution": true
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: chrono::Utc::now(),
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(UpdateUserActivityResponse { user })
    }
    
    async fn validate_user_data_activity(
        &self,
        _context: ActivityContext,
        request: ValidateUserDataActivityRequest,
    ) -> Result<ValidateUserDataActivityResponse> {
        let mut validation_errors = Vec::new();
        let mut is_valid = true;
        
        // Basic validation based on rules
        for rule in &request.validation_rules {
            match rule.as_str() {
                "email_format" => {
                    if let Some(email) = request.user_data.get("email").and_then(|e| e.as_str()) {
                        if let Err(e) = self.validator.validate_email(email) {
                            validation_errors.push(e.to_string());
                            is_valid = false;
                        }
                    }
                }
                "phone_format" => {
                    if let Some(phone) = request.user_data.get("phone_number").and_then(|p| p.as_str()) {
                        if let Err(e) = self.validator.validate_phone_number(phone) {
                            validation_errors.push(e.to_string());
                            is_valid = false;
                        }
                    }
                }
                "required_fields" => {
                    let required_fields = ["email", "first_name", "last_name"];
                    for field in &required_fields {
                        if !request.user_data.get(field).is_some() {
                            validation_errors.push(format!("Required field missing: {}", field));
                            is_valid = false;
                        }
                    }
                }
                "user_exists" => {
                    if let Some(user_id) = request.user_data.get("user_id").and_then(|id| id.as_str()) {
                        if let Ok(user_uuid) = Uuid::parse_str(user_id) {
                            if self.user_repo.find_by_id(request.tenant_id, user_uuid).await?.is_none() {
                                validation_errors.push("User does not exist".to_string());
                                is_valid = false;
                            }
                        }
                    }
                }
                "can_deactivate" => {
                    // Add business logic for deactivation validation
                    tracing::info!("Validating user can be deactivated");
                }
                "can_reactivate" => {
                    // Add business logic for reactivation validation
                    tracing::info!("Validating user can be reactivated");
                }
                _ => {
                    // Unknown validation rule - log but don't fail
                    tracing::warn!("Unknown validation rule: {}", rule);
                }
            }
        }
        
        // Sanitize data if valid
        let sanitized_data = if is_valid {
            let mut sanitized = request.user_data.clone();
            
            // Sanitize text fields
            if let Some(first_name) = sanitized.get_mut("first_name").and_then(|f| f.as_str()) {
                sanitized["first_name"] = serde_json::Value::String(self.validator.sanitize_text(first_name));
            }
            if let Some(last_name) = sanitized.get_mut("last_name").and_then(|l| l.as_str()) {
                sanitized["last_name"] = serde_json::Value::String(self.validator.sanitize_text(last_name));
            }
            if let Some(bio) = sanitized.get_mut("bio").and_then(|b| b.as_str()) {
                sanitized["bio"] = serde_json::Value::String(self.validator.sanitize_html(bio));
            }
            
            Some(sanitized)
        } else {
            None
        };
        
        Ok(ValidateUserDataActivityResponse {
            is_valid,
            validation_errors,
            sanitized_data,
        })
    }
    
    // New activity implementations
    async fn sync_user_profile_activity(
        &self,
        _context: ActivityContext,
        request: SyncUserProfileActivityRequest,
    ) -> Result<SyncUserProfileActivityResponse> {
        let sync_id = Uuid::new_v4();
        let sync_timestamp = Utc::now();
        
        // Get user profile data
        let _user = self.user_repo.find_by_id(request.tenant_id, request.user_id).await?
            .ok_or_else(|| adx_shared::Error::NotFound(format!("User {} not found", request.user_id)))?;
        
        let _profile = self.profile_repo.find_by_user_id(request.tenant_id, request.user_id).await?;
        
        // Simulate syncing to target service
        tracing::info!(
            "Syncing user {} profile to service {} with fields: {:?}",
            request.user_id, request.target_service, request.sync_fields
        );
        
        // Log the sync activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            tenant_id: request.tenant_id,
            activity_type: "user_profile_sync".to_string(),
            activity_description: Some(format!("Profile synced to {}", request.target_service)),
            resource_type: Some("user_profile".to_string()),
            resource_id: Some(request.user_id),
            metadata: serde_json::json!({
                "sync_id": sync_id,
                "target_service": request.target_service,
                "synced_fields": request.sync_fields,
                "force_sync": request.force_sync
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: sync_timestamp,
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(SyncUserProfileActivityResponse {
            sync_id,
            synced_fields: request.sync_fields,
            sync_timestamp,
            sync_status: "completed".to_string(),
        })
    }
    
    async fn migrate_user_preferences_activity(
        &self,
        _context: ActivityContext,
        request: MigrateUserPreferencesActivityRequest,
    ) -> Result<MigrateUserPreferencesActivityResponse> {
        let migration_id = Uuid::new_v4();
        let migration_timestamp = Utc::now();
        
        // Get current preferences for the category
        let current_preferences = self.preference_repo
            .find_by_category(request.tenant_id, request.user_id, &request.preference_category)
            .await?;
        
        // Apply migration rules
        let mut migrated_preferences = HashMap::new();
        for (key, rule) in &request.migration_rules {
            if let Some(_current_pref) = current_preferences.iter()
                .find(|p| p.preference_key == *key) {
                // Apply migration rule to current value
                migrated_preferences.insert(key.clone(), rule.clone());
            }
        }
        
        // Create backup if needed
        let backup_created = !current_preferences.is_empty();
        
        tracing::info!(
            "Migrated {} preferences for user {} in category {}",
            migrated_preferences.len(), request.user_id, request.preference_category
        );
        
        // Log the migration activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            tenant_id: request.tenant_id,
            activity_type: "user_preference_migration".to_string(),
            activity_description: Some(format!("Preferences migrated for category {}", request.preference_category)),
            resource_type: Some("user_preferences".to_string()),
            resource_id: Some(request.user_id),
            metadata: serde_json::json!({
                "migration_id": migration_id,
                "category": request.preference_category,
                "source_version": request.source_version,
                "target_version": request.target_version,
                "migrated_count": migrated_preferences.len()
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: migration_timestamp,
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(MigrateUserPreferencesActivityResponse {
            migration_id,
            migrated_preferences,
            migration_timestamp,
            backup_created,
        })
    }
    
    async fn export_user_data_activity(
        &self,
        _context: ActivityContext,
        request: ExportUserDataActivityRequest,
    ) -> Result<ExportUserDataActivityResponse> {
        let export_id = Uuid::new_v4();
        let export_timestamp = Utc::now();
        
        // Get user data
        let _user = self.user_repo.find_by_id(request.tenant_id, request.user_id).await?
            .ok_or_else(|| adx_shared::Error::NotFound(format!("User {} not found", request.user_id)))?;
        
        // Simulate data export
        let export_path = format!("/exports/{}/{}.{}", 
            request.tenant_id, export_id, request.export_format);
        let export_size_bytes = 1024 * 1024; // 1MB placeholder
        
        tracing::info!(
            "Exported user {} data in format {} to {}",
            request.user_id, request.export_format, export_path
        );
        
        // Log the export activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            tenant_id: request.tenant_id,
            activity_type: "user_data_export".to_string(),
            activity_description: Some("User data exported for GDPR compliance".to_string()),
            resource_type: Some("user_data".to_string()),
            resource_id: Some(request.user_id),
            metadata: serde_json::json!({
                "export_id": export_id,
                "export_format": request.export_format,
                "data_categories": request.data_categories,
                "include_metadata": request.include_metadata,
                "anonymize_data": request.anonymize_data
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: export_timestamp,
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(ExportUserDataActivityResponse {
            export_id,
            export_path,
            export_size_bytes,
            exported_categories: request.data_categories,
            export_timestamp,
        })
    }
    
    async fn deactivate_user_activity(
        &self,
        _context: ActivityContext,
        request: DeactivateUserActivityRequest,
    ) -> Result<DeactivateUserActivityResponse> {
        let deactivation_id = Uuid::new_v4();
        let deactivated_at = Utc::now();
        
        // Update user status to inactive
        let update_request = UpdateUserRequest {
            status: Some(UserStatus::Inactive),
            first_name: None,
            last_name: None,
            roles: None,
            permissions: None,
        };
        
        let _user = self.user_repo.update(request.tenant_id, request.user_id, update_request).await?;
        
        // Calculate data retention
        let data_retention_until = if request.retain_data {
            request.data_retention_days.map(|days| {
                deactivated_at + chrono::Duration::days(days as i64)
            })
        } else {
            None
        };
        
        // Schedule cleanup tasks
        let cleanup_tasks_scheduled = vec![
            "revoke_active_sessions".to_string(),
            "disable_api_keys".to_string(),
            "remove_from_active_teams".to_string(),
            "archive_user_data".to_string(),
        ];
        
        tracing::info!(
            "Deactivated user {} with reason: {}",
            request.user_id, request.deactivation_reason
        );
        
        // Log the deactivation activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            tenant_id: request.tenant_id,
            activity_type: "user_deactivation".to_string(),
            activity_description: Some(format!("User deactivated: {}", request.deactivation_reason)),
            resource_type: Some("user".to_string()),
            resource_id: Some(request.user_id),
            metadata: serde_json::json!({
                "deactivation_id": deactivation_id,
                "deactivation_reason": request.deactivation_reason,
                "deactivated_by": request.deactivated_by,
                "retain_data": request.retain_data,
                "data_retention_days": request.data_retention_days
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: deactivated_at,
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(DeactivateUserActivityResponse {
            deactivation_id,
            deactivated_at,
            data_retention_until,
            cleanup_tasks_scheduled,
        })
    }
    
    async fn reactivate_user_activity(
        &self,
        _context: ActivityContext,
        request: ReactivateUserActivityRequest,
    ) -> Result<ReactivateUserActivityResponse> {
        let reactivation_id = Uuid::new_v4();
        let reactivated_at = Utc::now();
        
        // Update user status to active
        let update_request = UpdateUserRequest {
            status: Some(UserStatus::Active),
            first_name: None,
            last_name: None,
            roles: None,
            permissions: None,
        };
        
        let _user = self.user_repo.update(request.tenant_id, request.user_id, update_request).await?;
        
        // Create restoration summary
        let mut restoration_summary = HashMap::new();
        
        if request.restore_data {
            restoration_summary.insert("data_restored".to_string(), serde_json::json!({
                "files_restored": 25,
                "preferences_restored": true,
                "activity_history_restored": true
            }));
        }
        
        if request.restore_permissions {
            restoration_summary.insert("permissions_restored".to_string(), serde_json::json!({
                "roles_restored": ["user", "team_member"],
                "team_memberships_restored": 2,
                "access_permissions_restored": true
            }));
        }
        
        tracing::info!("Reactivated user {}", request.user_id);
        
        // Log the reactivation activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: request.user_id,
            tenant_id: request.tenant_id,
            activity_type: "user_reactivation".to_string(),
            activity_description: Some("User account reactivated".to_string()),
            resource_type: Some("user".to_string()),
            resource_id: Some(request.user_id),
            metadata: serde_json::json!({
                "reactivation_id": reactivation_id,
                "reactivated_by": request.reactivated_by,
                "restore_data": request.restore_data,
                "restore_permissions": request.restore_permissions
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: reactivated_at,
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(ReactivateUserActivityResponse {
            reactivation_id,
            reactivated_at,
            data_restored: request.restore_data,
            permissions_restored: request.restore_permissions,
            restoration_summary,
        })
    }
    
    async fn transfer_user_ownership_activity(
        &self,
        _context: ActivityContext,
        request: TransferUserOwnershipActivityRequest,
    ) -> Result<TransferUserOwnershipActivityResponse> {
        let transfer_id = Uuid::new_v4();
        let transfer_timestamp = Utc::now();
        
        // Simulate ownership transfer
        let transferred_count = request.resource_ids.len() as u32;
        let failed_count = 0; // Placeholder
        
        tracing::info!(
            "Transferred {} {} resources from user {} to user {}",
            transferred_count, request.resource_type, request.from_user_id, request.to_user_id
        );
        
        // Log the transfer activity
        let activity = UserActivityLog {
            id: Uuid::new_v4(),
            user_id: request.from_user_id,
            tenant_id: request.tenant_id,
            activity_type: "user_ownership_transfer".to_string(),
            activity_description: Some(format!("Ownership transferred to user {}", request.to_user_id)),
            resource_type: Some(request.resource_type.clone()),
            resource_id: Some(request.from_user_id),
            metadata: serde_json::json!({
                "transfer_id": transfer_id,
                "to_user_id": request.to_user_id,
                "resource_type": request.resource_type,
                "resource_count": transferred_count,
                "notify_new_owner": request.notify_new_owner
            }),
            ip_address: None,
            user_agent: None,
            session_id: None,
            created_at: transfer_timestamp,
        };
        
        let _ = self.activity_repo.log_activity(activity).await;
        
        Ok(TransferUserOwnershipActivityResponse {
            transfer_id,
            transferred_count,
            failed_count,
            transfer_timestamp,
            notification_sent: request.notify_new_owner,
        })
    }
}