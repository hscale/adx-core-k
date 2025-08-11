use async_trait::async_trait;
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};


use crate::models::*;
use crate::services::TenantService;
use adx_shared::types::{TenantId, UserId};

// Activity request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateTenantCreationRequest {
    pub tenant_name: String,
    pub admin_email: String,
    pub subscription_tier: adx_shared::types::SubscriptionTier,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantValidationResult {
    pub is_valid: bool,
    pub tenant_id: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SetupTenantDatabaseRequest {
    pub tenant_id: TenantId,
    pub isolation_level: adx_shared::types::TenantIsolationLevel,
    pub initial_schema: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DatabaseSetupResult {
    pub connection_string: String,
    pub schema_created: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantConfigRequest {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub subscription_tier: adx_shared::types::SubscriptionTier,
    pub quotas: adx_shared::types::TenantQuotas,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidateUserTenantAccessRequest {
    pub user_id: UserId,
    pub target_tenant_id: TenantId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserTenantAccessResult {
    pub has_access: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveSessionStateRequest {
    pub user_id: UserId,
    pub current_tenant_id: TenantId,
    pub session_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionStateResult {
    pub session_id: String,
    pub saved: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoadTenantContextRequest {
    pub tenant_id: TenantId,
    pub user_id: UserId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateTenantSessionRequest {
    pub user_id: UserId,
    pub tenant_id: TenantId,
    pub tenant_context: TenantContext,
    pub session_duration: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TenantSessionResult {
    pub session_id: String,
    pub available_features: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserActiveTenantRequest {
    pub user_id: UserId,
    pub new_active_tenant_id: TenantId,
}

// Activity trait definition
#[async_trait]
pub trait TenantActivities: Send + Sync {
    // Tenant creation activities
    async fn validate_tenant_creation(&self, request: ValidateTenantCreationRequest) -> Result<TenantValidationResult>;
    async fn setup_tenant_database(&self, request: SetupTenantDatabaseRequest) -> Result<DatabaseSetupResult>;
    async fn create_tenant_config(&self, request: CreateTenantConfigRequest) -> Result<Tenant>;
    async fn cleanup_tenant_database(&self, tenant_id: &TenantId) -> Result<()>;

    // Tenant switching activities
    async fn validate_user_tenant_access(&self, request: ValidateUserTenantAccessRequest) -> Result<UserTenantAccessResult>;
    async fn save_session_state(&self, request: SaveSessionStateRequest) -> Result<SessionStateResult>;
    async fn load_tenant_context(&self, request: LoadTenantContextRequest) -> Result<TenantContext>;
    async fn create_tenant_session(&self, request: CreateTenantSessionRequest) -> Result<TenantSessionResult>;
    async fn update_user_active_tenant(&self, request: UpdateUserActiveTenantRequest) -> Result<()>;
}

// Implementation of tenant activities
pub struct TenantActivitiesImpl {
    tenant_service: Arc<TenantService>,
}

impl TenantActivitiesImpl {
    pub fn new(tenant_service: Arc<TenantService>) -> Self {
        Self { tenant_service }
    }
}

#[async_trait]
impl TenantActivities for TenantActivitiesImpl {
    async fn validate_tenant_creation(&self, request: ValidateTenantCreationRequest) -> Result<TenantValidationResult> {
        let mut errors = Vec::new();

        // Validate tenant name
        if request.tenant_name.trim().is_empty() {
            errors.push("Tenant name cannot be empty".to_string());
        }

        if request.tenant_name.len() < 3 {
            errors.push("Tenant name must be at least 3 characters long".to_string());
        }

        if request.tenant_name.len() > 100 {
            errors.push("Tenant name cannot exceed 100 characters".to_string());
        }

        // Check if tenant name already exists
        if let Ok(Some(_)) = self.tenant_service.get_tenant_by_slug(&request.tenant_name.to_lowercase()).await {
            errors.push("Tenant with this name already exists".to_string());
        }

        // Validate admin email
        if !request.admin_email.contains('@') {
            errors.push("Invalid admin email format".to_string());
        }

        let is_valid = errors.is_empty();
        let tenant_id = if is_valid {
            uuid::Uuid::new_v4().to_string()
        } else {
            String::new()
        };

        Ok(TenantValidationResult {
            is_valid,
            tenant_id,
            errors,
        })
    }

    async fn setup_tenant_database(&self, request: SetupTenantDatabaseRequest) -> Result<DatabaseSetupResult> {
        // In a real implementation, this would:
        // 1. Create a new database schema or database based on isolation level
        // 2. Run migrations for the tenant
        // 3. Set up initial data
        
        tracing::info!("Setting up database for tenant: {}", request.tenant_id);
        
        // Simulate database setup
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let connection_string = match request.isolation_level {
            adx_shared::types::TenantIsolationLevel::Database => {
                format!("postgresql://user:pass@localhost/tenant_{}", request.tenant_id)
            }
            adx_shared::types::TenantIsolationLevel::Schema => {
                format!("postgresql://user:pass@localhost/adx_core?search_path=tenant_{}", request.tenant_id)
            }
            adx_shared::types::TenantIsolationLevel::Row => {
                "postgresql://user:pass@localhost/adx_core".to_string()
            }
        };

        Ok(DatabaseSetupResult {
            connection_string,
            schema_created: true,
        })
    }

    async fn create_tenant_config(&self, request: CreateTenantConfigRequest) -> Result<Tenant> {
        let create_request = CreateTenantRequest {
            name: request.tenant_name,
            admin_email: "admin@example.com".to_string(), // This would come from the workflow
            subscription_tier: Some(request.subscription_tier),
            isolation_level: None,
            features: Some(request.features),
            settings: None,
        };

        self.tenant_service.create_tenant(create_request).await
    }

    async fn cleanup_tenant_database(&self, tenant_id: &TenantId) -> Result<()> {
        // In a real implementation, this would clean up tenant-specific database resources
        tracing::info!("Cleaning up database for tenant: {}", tenant_id);
        
        // Simulate cleanup
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        
        Ok(())
    }

    async fn validate_user_tenant_access(&self, request: ValidateUserTenantAccessRequest) -> Result<UserTenantAccessResult> {
        match self.tenant_service.validate_tenant_access(&request.target_tenant_id, &request.user_id).await {
            Ok(has_access) => {
                let reason = if !has_access {
                    Some("User does not have access to the target tenant".to_string())
                } else {
                    None
                };

                Ok(UserTenantAccessResult {
                    has_access,
                    reason,
                })
            }
            Err(e) => Ok(UserTenantAccessResult {
                has_access: false,
                reason: Some(e.to_string()),
            }),
        }
    }

    async fn save_session_state(&self, request: SaveSessionStateRequest) -> Result<SessionStateResult> {
        // In a real implementation, this would save the current session state
        // to Redis or another session store
        tracing::info!("Saving session state for user: {}", request.user_id);
        
        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Simulate saving session state
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(SessionStateResult {
            session_id,
            saved: true,
        })
    }

    async fn load_tenant_context(&self, request: LoadTenantContextRequest) -> Result<TenantContext> {
        self.tenant_service.get_tenant_context(&request.tenant_id, &request.user_id).await
    }

    async fn create_tenant_session(&self, request: CreateTenantSessionRequest) -> Result<TenantSessionResult> {
        // In a real implementation, this would create a new session in the session store
        tracing::info!("Creating tenant session for user: {} in tenant: {}", request.user_id, request.tenant_id);
        
        let session_id = uuid::Uuid::new_v4().to_string();
        
        // Simulate session creation
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(TenantSessionResult {
            session_id,
            available_features: request.tenant_context.features,
        })
    }

    async fn update_user_active_tenant(&self, request: UpdateUserActiveTenantRequest) -> Result<()> {
        // In a real implementation, this would update the user's active tenant
        // in the user service or user database
        tracing::info!("Updating active tenant for user: {} to tenant: {}", request.user_id, request.new_active_tenant_id);
        
        // Simulate user update
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        
        Ok(())
    }
}