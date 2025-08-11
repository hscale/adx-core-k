use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use adx_shared::{
    temporal::{
        ActivityContext, AdxActivity, TenantAwareActivity, DatabaseActivity,
        ActivityError, utils::database_retry_policy
    },
    auth::{UserContext, TenantContext},
    database::DatabasePool,
    Error, Result,
};

use crate::repositories::{UserRepository, user::{User, UserStatus}};

/// SSO provider types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SsoProvider {
    Google,
    Microsoft,
    Okta,
    Auth0,
    Saml,
    Oidc,
}

/// SSO user attributes from external provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoUserAttributes {
    pub provider_user_id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub avatar_url: Option<String>,
    pub groups: Vec<String>,
    pub roles: Vec<String>,
    pub custom_attributes: HashMap<String, serde_json::Value>,
}

/// Request for provisioning SSO user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionSsoUserRequest {
    pub provider: SsoProvider,
    pub provider_tenant_id: Option<String>,
    pub user_attributes: SsoUserAttributes,
    pub auto_create_user: bool,
    pub default_roles: Vec<String>,
    pub role_mapping: HashMap<String, Vec<String>>,
    pub update_existing_user: bool,
    pub require_email_verification: bool,
}

/// Response from provisioning SSO user
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionSsoUserResponse {
    pub user_id: String,
    pub email: String,
    pub user_created: bool,
    pub user_updated: bool,
    pub mapped_roles: Vec<String>,
    pub sso_linked: bool,
    pub requires_verification: bool,
    pub provider: SsoProvider,
    pub provider_user_id: String,
}

/// SSO user mapping for linking external accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SsoUserMapping {
    pub id: String,
    pub tenant_id: String,
    pub user_id: String,
    pub provider: SsoProvider,
    pub provider_user_id: String,
    pub provider_tenant_id: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Activity for provisioning SSO users
pub struct ProvisionSsoUserActivity {
    database_pool: DatabasePool,
}

impl ProvisionSsoUserActivity {
    pub fn new(database_pool: DatabasePool) -> Self {
        Self { database_pool }
    }

    /// Check if user already exists by email
    async fn find_existing_user(
        &self,
        tenant_id: &str,
        email: &str,
    ) -> Result<Option<User>, ActivityError> {
        let user_repo = UserRepository::new(
            self.database_pool.clone(),
            tenant_id.to_string(),
        );

        user_repo.find_by_email(email).await
            .map_err(|e| ActivityError::DatabaseError {
                message: format!("Failed to find existing user: {}", e),
            })
    }

    /// Check if SSO mapping already exists
    async fn find_sso_mapping(
        &self,
        tenant_id: &str,
        provider: &SsoProvider,
        provider_user_id: &str,
    ) -> Result<Option<SsoUserMapping>, ActivityError> {
        // In a real implementation, this would query the sso_user_mappings table
        // For now, we'll return None to indicate no existing mapping
        Ok(None)
    }

    /// Create SSO user mapping
    async fn create_sso_mapping(
        &self,
        tenant_id: &str,
        user_id: &str,
        provider: &SsoProvider,
        provider_user_id: &str,
        provider_tenant_id: Option<&str>,
    ) -> Result<SsoUserMapping, ActivityError> {
        let mapping = SsoUserMapping {
            id: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            provider: provider.clone(),
            provider_user_id: provider_user_id.to_string(),
            provider_tenant_id: provider_tenant_id.map(|s| s.to_string()),
            last_login_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // In a real implementation, this would insert into sso_user_mappings table
        // For now, we'll just return the mapping
        tracing::info!("Created SSO mapping for user {} with provider {:?}", user_id, provider);
        
        Ok(mapping)
    }

    /// Update SSO user mapping
    async fn update_sso_mapping(
        &self,
        mapping: &mut SsoUserMapping,
    ) -> Result<(), ActivityError> {
        mapping.last_login_at = Some(Utc::now());
        mapping.updated_at = Utc::now();

        // In a real implementation, this would update the sso_user_mappings table
        tracing::info!("Updated SSO mapping for user {}", mapping.user_id);
        
        Ok(())
    }

    /// Map SSO groups/roles to local roles
    fn map_sso_roles(
        &self,
        sso_groups: &[String],
        sso_roles: &[String],
        role_mapping: &HashMap<String, Vec<String>>,
        default_roles: &[String],
    ) -> Vec<String> {
        let mut mapped_roles = Vec::new();

        // Add default roles
        mapped_roles.extend_from_slice(default_roles);

        // Map SSO groups to roles
        for group in sso_groups {
            if let Some(roles) = role_mapping.get(group) {
                mapped_roles.extend_from_slice(roles);
            }
        }

        // Map SSO roles directly
        for role in sso_roles {
            if let Some(roles) = role_mapping.get(role) {
                mapped_roles.extend_from_slice(roles);
            } else {
                // If no mapping exists, use the role directly if it's valid
                if self.is_valid_role(role) {
                    mapped_roles.push(role.clone());
                }
            }
        }

        // Remove duplicates and ensure at least 'user' role
        mapped_roles.sort();
        mapped_roles.dedup();
        
        if mapped_roles.is_empty() {
            mapped_roles.push("user".to_string());
        }

        mapped_roles
    }

    /// Validate if a role is valid for the system
    fn is_valid_role(&self, role: &str) -> bool {
        matches!(role, "user" | "admin" | "manager" | "viewer" | "editor")
    }

    /// Create new user from SSO attributes
    async fn create_sso_user(
        &self,
        tenant_id: &str,
        attributes: &SsoUserAttributes,
        mapped_roles: Vec<String>,
        require_verification: bool,
    ) -> Result<User, ActivityError> {
        let user_repo = UserRepository::new(
            self.database_pool.clone(),
            tenant_id.to_string(),
        );

        // Create user preferences with SSO attributes
        let mut preferences = serde_json::json!({
            "sso_provisioned": true,
            "avatar_url": attributes.avatar_url,
            "display_name": attributes.display_name,
        });

        // Add custom attributes to preferences
        for (key, value) in &attributes.custom_attributes {
            preferences[format!("sso_{}", key)] = value.clone();
        }

        let user = User {
            id: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            email: attributes.email.clone(),
            password_hash: "".to_string(), // SSO users don't have passwords
            first_name: attributes.first_name.clone(),
            last_name: attributes.last_name.clone(),
            status: if require_verification {
                UserStatus::PendingVerification
            } else {
                UserStatus::Active
            },
            roles: mapped_roles,
            permissions: vec![], // Permissions are derived from roles
            preferences,
            last_login_at: Some(Utc::now()),
            email_verified_at: if require_verification { None } else { Some(Utc::now()) },
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        user_repo.create(user).await
            .map_err(|e| ActivityError::DatabaseError {
                message: format!("Failed to create SSO user: {}", e),
            })
    }

    /// Update existing user with SSO attributes
    async fn update_sso_user(
        &self,
        tenant_id: &str,
        mut user: User,
        attributes: &SsoUserAttributes,
        mapped_roles: Vec<String>,
    ) -> Result<User, ActivityError> {
        let user_repo = UserRepository::new(
            self.database_pool.clone(),
            tenant_id.to_string(),
        );

        // Update user attributes from SSO
        if let Some(ref first_name) = attributes.first_name {
            user.first_name = Some(first_name.clone());
        }
        
        if let Some(ref last_name) = attributes.last_name {
            user.last_name = Some(last_name.clone());
        }

        // Update roles (merge with existing roles)
        let mut all_roles = user.roles.clone();
        all_roles.extend(mapped_roles);
        all_roles.sort();
        all_roles.dedup();
        user.roles = all_roles;

        // Update preferences with SSO attributes
        if let Some(ref avatar_url) = attributes.avatar_url {
            user.preferences["avatar_url"] = serde_json::Value::String(avatar_url.clone());
        }
        
        if let Some(ref display_name) = attributes.display_name {
            user.preferences["display_name"] = serde_json::Value::String(display_name.clone());
        }

        user.preferences["sso_provisioned"] = serde_json::Value::Bool(true);
        user.preferences["last_sso_login"] = serde_json::Value::String(Utc::now().to_rfc3339());

        // Add custom attributes
        for (key, value) in &attributes.custom_attributes {
            user.preferences[format!("sso_{}", key)] = value.clone();
        }

        user.last_login_at = Some(Utc::now());
        user.updated_at = Utc::now();

        user_repo.update(user).await
            .map_err(|e| ActivityError::DatabaseError {
                message: format!("Failed to update SSO user: {}", e),
            })
    }

    /// Validate SSO provider configuration
    fn validate_provider_config(
        &self,
        provider: &SsoProvider,
        provider_tenant_id: Option<&str>,
    ) -> Result<(), ActivityError> {
        match provider {
            SsoProvider::Saml | SsoProvider::Oidc => {
                // These providers typically require tenant-specific configuration
                if provider_tenant_id.is_none() {
                    return Err(ActivityError::ValidationError {
                        field: "provider_tenant_id".to_string(),
                        message: format!("{:?} provider requires tenant ID", provider),
                    });
                }
            }
            _ => {
                // Other providers may work without tenant ID
            }
        }

        Ok(())
    }
}

#[async_trait]
impl AdxActivity<ProvisionSsoUserRequest, ProvisionSsoUserResponse> for ProvisionSsoUserActivity {
    async fn execute(
        &self,
        context: ActivityContext,
        input: ProvisionSsoUserRequest,
    ) -> Result<ProvisionSsoUserResponse, ActivityError> {
        // Validate input
        self.validate_input(&input)?;

        // Validate provider configuration
        self.validate_provider_config(&input.provider, input.provider_tenant_id.as_deref())?;

        // Map SSO roles to local roles
        let mapped_roles = self.map_sso_roles(
            &input.user_attributes.groups,
            &input.user_attributes.roles,
            &input.role_mapping,
            &input.default_roles,
        );

        // Check if SSO mapping already exists
        let existing_mapping = self.find_sso_mapping(
            &context.tenant_context.tenant_id,
            &input.provider,
            &input.user_attributes.provider_user_id,
        ).await?;

        let mut user_created = false;
        let mut user_updated = false;
        let mut sso_linked = false;

        let user = if let Some(mut mapping) = existing_mapping {
            // SSO mapping exists, get the linked user
            let user_repo = UserRepository::new(
                self.database_pool.clone(),
                context.tenant_context.tenant_id.clone(),
            );

            let user = user_repo.find_by_id(&mapping.user_id).await
                .map_err(|e| ActivityError::DatabaseError {
                    message: format!("Failed to find linked user: {}", e),
                })?
                .ok_or_else(|| ActivityError::NotFoundError {
                    resource_type: "user".to_string(),
                    resource_id: mapping.user_id.clone(),
                })?;

            // Update SSO mapping
            self.update_sso_mapping(&mut mapping).await?;

            // Update user if requested
            if input.update_existing_user {
                user_updated = true;
                self.update_sso_user(
                    &context.tenant_context.tenant_id,
                    user,
                    &input.user_attributes,
                    mapped_roles.clone(),
                ).await?
            } else {
                user
            }
        } else {
            // No SSO mapping exists, check if user exists by email
            let existing_user = self.find_existing_user(
                &context.tenant_context.tenant_id,
                &input.user_attributes.email,
            ).await?;

            let user = if let Some(user) = existing_user {
                // User exists, link SSO account
                self.create_sso_mapping(
                    &context.tenant_context.tenant_id,
                    &user.id,
                    &input.provider,
                    &input.user_attributes.provider_user_id,
                    input.provider_tenant_id.as_deref(),
                ).await?;

                sso_linked = true;

                // Update user if requested
                if input.update_existing_user {
                    user_updated = true;
                    self.update_sso_user(
                        &context.tenant_context.tenant_id,
                        user,
                        &input.user_attributes,
                        mapped_roles.clone(),
                    ).await?
                } else {
                    user
                }
            } else {
                // User doesn't exist, create if auto-create is enabled
                if !input.auto_create_user {
                    return Err(ActivityError::NotFoundError {
                        resource_type: "user".to_string(),
                        resource_id: input.user_attributes.email.clone(),
                    });
                }

                // Create new user
                let user = self.create_sso_user(
                    &context.tenant_context.tenant_id,
                    &input.user_attributes,
                    mapped_roles.clone(),
                    input.require_email_verification,
                ).await?;

                user_created = true;

                // Create SSO mapping
                self.create_sso_mapping(
                    &context.tenant_context.tenant_id,
                    &user.id,
                    &input.provider,
                    &input.user_attributes.provider_user_id,
                    input.provider_tenant_id.as_deref(),
                ).await?;

                sso_linked = true;

                user
            }
        };

        Ok(ProvisionSsoUserResponse {
            user_id: user.id,
            email: user.email,
            user_created,
            user_updated,
            mapped_roles,
            sso_linked,
            requires_verification: user.status == UserStatus::PendingVerification,
            provider: input.provider,
            provider_user_id: input.user_attributes.provider_user_id,
        })
    }

    fn activity_type(&self) -> &'static str {
        "provision_sso_user_activity"
    }

    fn validate_input(&self, input: &ProvisionSsoUserRequest) -> Result<(), ActivityError> {
        // Validate user attributes
        if input.user_attributes.provider_user_id.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "user_attributes.provider_user_id".to_string(),
                message: "Provider user ID is required".to_string(),
            });
        }

        if input.user_attributes.email.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "user_attributes.email".to_string(),
                message: "Email is required".to_string(),
            });
        }

        // Basic email validation
        if !input.user_attributes.email.contains('@') || !input.user_attributes.email.contains('.') {
            return Err(ActivityError::ValidationError {
                field: "user_attributes.email".to_string(),
                message: "Invalid email format".to_string(),
            });
        }

        // Validate default roles
        for role in &input.default_roles {
            if role.trim().is_empty() {
                return Err(ActivityError::ValidationError {
                    field: "default_roles".to_string(),
                    message: "Role names cannot be empty".to_string(),
                });
            }
        }

        // Validate role mapping
        for (key, roles) in &input.role_mapping {
            if key.trim().is_empty() {
                return Err(ActivityError::ValidationError {
                    field: "role_mapping".to_string(),
                    message: "Role mapping keys cannot be empty".to_string(),
                });
            }

            for role in roles {
                if role.trim().is_empty() {
                    return Err(ActivityError::ValidationError {
                        field: "role_mapping".to_string(),
                        message: "Mapped role names cannot be empty".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn default_options(&self) -> adx_shared::temporal::ActivityExecutionOptions {
        let mut options = adx_shared::temporal::ActivityExecutionOptions::default();
        options.retry_policy = Some(database_retry_policy());
        options.tags.push("sso_provisioning".to_string());
        options.tags.push("authentication".to_string());
        options.tags.push("user_management".to_string());
        options
    }
}

#[async_trait]
impl TenantAwareActivity<ProvisionSsoUserRequest, ProvisionSsoUserResponse> for ProvisionSsoUserActivity {
    async fn validate_tenant_access(
        &self,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<(), ActivityError> {
        // Check if tenant is active
        if !tenant_context.is_active {
            return Err(ActivityError::AuthorizationError {
                message: "Cannot provision SSO users for inactive tenant".to_string(),
            });
        }

        // Check if tenant has SSO features enabled
        if !tenant_context.features.contains(&"sso".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "SSO feature not enabled for tenant".to_string(),
            });
        }

        // Check if user has permission to provision SSO users
        if user_context.user_id != "system" && 
           !user_context.permissions.contains(&"sso:provision_users".to_string()) &&
           !user_context.roles.contains(&"admin".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "Insufficient permissions to provision SSO users".to_string(),
            });
        }

        Ok(())
    }

    async fn check_tenant_quotas(
        &self,
        tenant_context: &TenantContext,
        resource_type: &str,
        requested_amount: u64,
    ) -> Result<(), ActivityError> {
        if resource_type == "sso_users" {
            // Check if tenant has SSO features enabled
            if !tenant_context.features.contains(&"sso".to_string()) {
                return Err(ActivityError::QuotaExceededError {
                    resource_type: "sso_users".to_string(),
                    current_usage: 0,
                    limit: 0,
                    requested: requested_amount,
                });
            }

            // Check user quota
            if let Some(max_users) = tenant_context.quotas.max_users {
                let user_repo = UserRepository::new(
                    self.database_pool.clone(),
                    tenant_context.tenant_id.clone(),
                );
                
                let current_count = user_repo.count(None).await.map_err(|e| ActivityError::DatabaseError {
                    message: format!("Failed to count users: {}", e),
                })? as u32;

                if current_count + requested_amount as u32 > max_users {
                    return Err(ActivityError::QuotaExceededError {
                        resource_type: "users".to_string(),
                        current_usage: current_count as u64,
                        limit: max_users as u64,
                        requested: requested_amount,
                    });
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl DatabaseActivity<ProvisionSsoUserRequest, ProvisionSsoUserResponse> for ProvisionSsoUserActivity {
    async fn get_tenant_connection(
        &self,
        _tenant_context: &TenantContext,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, ActivityError> {
        Ok(Box::new(self.database_pool.clone()))
    }

    async fn execute_transaction<F, R>(
        &self,
        _tenant_context: &TenantContext,
        transaction: F,
    ) -> Result<R, ActivityError>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, ActivityError>> + Send>> + Send,
        R: Send + Sync,
    {
        // For now, execute without explicit transaction
        // TODO: Implement proper transaction support when needed
        transaction().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use adx_shared::auth::{TenantSettings, UserPreferences};

    fn create_test_context() -> ActivityContext {
        ActivityContext {
            activity_id: "test-activity-id".to_string(),
            activity_type: "provision_sso_user_activity".to_string(),
            workflow_id: "test-workflow-id".to_string(),
            workflow_run_id: "test-run-id".to_string(),
            attempt: 1,
            user_context: UserContext {
                user_id: "system".to_string(),
                email: "system@adxcore.com".to_string(),
                display_name: Some("System".to_string()),
                roles: vec!["system".to_string()],
                permissions: vec!["sso:provision_users".to_string()],
                quotas: adx_shared::UserQuotas {
                    api_calls_per_hour: 10000,
                    storage_gb: 100,
                    concurrent_workflows: 50,
                    file_upload_size_mb: 1000,
                },
                preferences: UserPreferences::default(),
                last_login: None,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            tenant_context: TenantContext {
                tenant_id: "test-tenant-id".to_string(),
                tenant_name: "Test Tenant".to_string(),
                subscription_tier: adx_shared::SubscriptionTier::Enterprise,
                features: vec!["sso".to_string(), "user_management".to_string()],
                quotas: adx_shared::TenantQuotas {
                    max_users: Some(1000),
                    max_storage_gb: Some(10000),
                    max_api_calls_per_hour: Some(100000),
                    max_workflows_per_hour: Some(5000),
                },
                settings: TenantSettings::default(),
                is_active: true,
                created_at: Utc::now(),
                updated_at: Utc::now(),
            },
            metadata: adx_shared::temporal::ActivityMetadata {
                start_time: Utc::now(),
                timeout: std::time::Duration::from_secs(300),
                heartbeat_timeout: Some(std::time::Duration::from_secs(30)),
                retry_policy: None,
                tags: vec![],
                custom: HashMap::new(),
            },
            heartbeat_details: None,
        }
    }

    #[test]
    fn test_map_sso_roles() {
        let activity = ProvisionSsoUserActivity::new(DatabasePool::new_mock());

        let sso_groups = vec!["Administrators".to_string(), "Users".to_string()];
        let sso_roles = vec!["admin".to_string(), "viewer".to_string()];
        let mut role_mapping = HashMap::new();
        role_mapping.insert("Administrators".to_string(), vec!["admin".to_string()]);
        role_mapping.insert("Users".to_string(), vec!["user".to_string()]);
        let default_roles = vec!["user".to_string()];

        let mapped_roles = activity.map_sso_roles(&sso_groups, &sso_roles, &role_mapping, &default_roles);

        assert!(mapped_roles.contains(&"admin".to_string()));
        assert!(mapped_roles.contains(&"user".to_string()));
        assert!(mapped_roles.contains(&"viewer".to_string()));
        
        // Should be deduplicated
        assert_eq!(mapped_roles.iter().filter(|&r| r == "user").count(), 1);
    }

    #[test]
    fn test_is_valid_role() {
        let activity = ProvisionSsoUserActivity::new(DatabasePool::new_mock());

        assert!(activity.is_valid_role("user"));
        assert!(activity.is_valid_role("admin"));
        assert!(activity.is_valid_role("manager"));
        assert!(activity.is_valid_role("viewer"));
        assert!(activity.is_valid_role("editor"));
        assert!(!activity.is_valid_role("invalid_role"));
        assert!(!activity.is_valid_role(""));
    }

    #[test]
    fn test_validate_provider_config() {
        let activity = ProvisionSsoUserActivity::new(DatabasePool::new_mock());

        // SAML requires tenant ID
        assert!(activity.validate_provider_config(&SsoProvider::Saml, None).is_err());
        assert!(activity.validate_provider_config(&SsoProvider::Saml, Some("tenant123")).is_ok());

        // OIDC requires tenant ID
        assert!(activity.validate_provider_config(&SsoProvider::Oidc, None).is_err());
        assert!(activity.validate_provider_config(&SsoProvider::Oidc, Some("tenant123")).is_ok());

        // Google doesn't require tenant ID
        assert!(activity.validate_provider_config(&SsoProvider::Google, None).is_ok());
        assert!(activity.validate_provider_config(&SsoProvider::Google, Some("tenant123")).is_ok());
    }

    #[test]
    fn test_validate_input() {
        let activity = ProvisionSsoUserActivity::new(DatabasePool::new_mock());

        // Valid input
        let valid_input = ProvisionSsoUserRequest {
            provider: SsoProvider::Google,
            provider_tenant_id: None,
            user_attributes: SsoUserAttributes {
                provider_user_id: "google123".to_string(),
                email: "user@example.com".to_string(),
                first_name: Some("John".to_string()),
                last_name: Some("Doe".to_string()),
                display_name: Some("John Doe".to_string()),
                avatar_url: Some("https://example.com/avatar.jpg".to_string()),
                groups: vec!["Users".to_string()],
                roles: vec!["user".to_string()],
                custom_attributes: HashMap::new(),
            },
            auto_create_user: true,
            default_roles: vec!["user".to_string()],
            role_mapping: HashMap::new(),
            update_existing_user: true,
            require_email_verification: false,
        };
        assert!(activity.validate_input(&valid_input).is_ok());

        // Invalid inputs
        let empty_provider_user_id = ProvisionSsoUserRequest {
            user_attributes: SsoUserAttributes {
                provider_user_id: "".to_string(),
                email: "user@example.com".to_string(),
                first_name: None,
                last_name: None,
                display_name: None,
                avatar_url: None,
                groups: vec![],
                roles: vec![],
                custom_attributes: HashMap::new(),
            },
            ..valid_input.clone()
        };
        assert!(activity.validate_input(&empty_provider_user_id).is_err());

        let empty_email = ProvisionSsoUserRequest {
            user_attributes: SsoUserAttributes {
                provider_user_id: "google123".to_string(),
                email: "".to_string(),
                first_name: None,
                last_name: None,
                display_name: None,
                avatar_url: None,
                groups: vec![],
                roles: vec![],
                custom_attributes: HashMap::new(),
            },
            ..valid_input.clone()
        };
        assert!(activity.validate_input(&empty_email).is_err());

        let invalid_email = ProvisionSsoUserRequest {
            user_attributes: SsoUserAttributes {
                provider_user_id: "google123".to_string(),
                email: "invalid-email".to_string(),
                first_name: None,
                last_name: None,
                display_name: None,
                avatar_url: None,
                groups: vec![],
                roles: vec![],
                custom_attributes: HashMap::new(),
            },
            ..valid_input.clone()
        };
        assert!(activity.validate_input(&invalid_email).is_err());

        let empty_default_role = ProvisionSsoUserRequest {
            default_roles: vec!["".to_string()],
            ..valid_input.clone()
        };
        assert!(activity.validate_input(&empty_default_role).is_err());
    }
}