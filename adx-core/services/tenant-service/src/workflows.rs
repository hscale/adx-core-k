use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::activities::{TenantActivities, TenantActivitiesImpl};
use crate::models::*;
use adx_shared::types::{TenantId, UserId};

// Workflow error types
#[derive(Debug, thiserror::Error)]
pub enum WorkflowError {
    #[error("Validation failed: {0:?}")]
    ValidationFailed(Vec<String>),
    #[error("Activity failed: {activity} - {error}")]
    ActivityFailed { activity: String, error: String },
    #[error("Tenant access denied: {0}")]
    TenantAccessDenied(String),
    #[error("Workflow execution failed: {0}")]
    ExecutionFailed(String),
}

// Workflow implementations
pub struct TenantWorkflows {
    activities: Arc<dyn TenantActivities>,
}

impl TenantWorkflows {
    pub fn new(activities: Arc<dyn TenantActivities>) -> Self {
        Self { activities }
    }

    // Create tenant workflow - complex tenant creation with database setup
    pub async fn create_tenant_workflow(
        &self,
        request: CreateTenantWorkflowRequest,
    ) -> Result<CreateTenantWorkflowResult, WorkflowError> {
        tracing::info!("Starting create tenant workflow for: {}", request.tenant_name);

        // Step 1: Validate tenant creation request
        let validation = self.activities
            .validate_tenant_creation(crate::activities::ValidateTenantCreationRequest {
                tenant_name: request.tenant_name.clone(),
                admin_email: request.admin_email.clone(),
                subscription_tier: request.subscription_tier.clone(),
            })
            .await
            .map_err(|e| WorkflowError::ActivityFailed {
                activity: "validate_tenant_creation".to_string(),
                error: e.to_string(),
            })?;

        if !validation.is_valid {
            return Err(WorkflowError::ValidationFailed(validation.errors));
        }

        let tenant_id = validation.tenant_id;

        // Step 2: Set up tenant database schema/database
        let database_setup = self.activities
            .setup_tenant_database(crate::activities::SetupTenantDatabaseRequest {
                tenant_id: tenant_id.clone(),
                isolation_level: request.isolation_level,
                initial_schema: None,
            })
            .await
            .map_err(|e| {
                // If database setup fails, we should clean up
                let cleanup_tenant_id = tenant_id.clone();
                let activities = self.activities.clone();
                tokio::spawn(async move {
                    if let Err(cleanup_err) = activities.cleanup_tenant_database(&cleanup_tenant_id).await {
                        tracing::error!("Failed to cleanup tenant database: {}", cleanup_err);
                    }
                });

                WorkflowError::ActivityFailed {
                    activity: "setup_tenant_database".to_string(),
                    error: e.to_string(),
                }
            })?;

        // Step 3: Create tenant configuration
        let tenant_config = self.activities
            .create_tenant_config(crate::activities::CreateTenantConfigRequest {
                tenant_id: tenant_id.clone(),
                tenant_name: request.tenant_name,
                subscription_tier: request.subscription_tier,
                quotas: request.quotas,
                features: request.features,
            })
            .await
            .map_err(|e| {
                // If config creation fails, clean up database
                let cleanup_tenant_id = tenant_id.clone();
                let activities = self.activities.clone();
                tokio::spawn(async move {
                    if let Err(cleanup_err) = activities.cleanup_tenant_database(&cleanup_tenant_id).await {
                        tracing::error!("Failed to cleanup tenant database: {}", cleanup_err);
                    }
                });

                WorkflowError::ActivityFailed {
                    activity: "create_tenant_config".to_string(),
                    error: e.to_string(),
                }
            })?;

        // Step 4: Create admin user (this would typically call the auth service)
        // For now, we'll simulate this step
        let admin_user_id = format!("admin-{}", uuid::Uuid::new_v4());

        // Step 5: Install default modules (this would typically call the module service)
        // For now, we'll just log this step
        for module_id in &request.default_modules {
            tracing::info!("Would install module {} for tenant {}", module_id, tenant_id);
        }

        tracing::info!("Successfully created tenant: {}", tenant_id);

        Ok(CreateTenantWorkflowResult {
            tenant_id,
            admin_user_id,
            database_connection: database_setup.connection_string,
        })
    }

    // Switch tenant workflow - complex tenant switching with session management
    pub async fn switch_tenant_workflow(
        &self,
        request: SwitchTenantWorkflowRequest,
    ) -> Result<SwitchTenantWorkflowResult, WorkflowError> {
        tracing::info!("Starting switch tenant workflow for user: {} to tenant: {}", 
                      request.user_id, request.target_tenant_id);

        // Step 1: Validate user has access to target tenant
        let access_validation = self.activities
            .validate_user_tenant_access(crate::activities::ValidateUserTenantAccessRequest {
                user_id: request.user_id.clone(),
                target_tenant_id: request.target_tenant_id.clone(),
            })
            .await
            .map_err(|e| WorkflowError::ActivityFailed {
                activity: "validate_user_tenant_access".to_string(),
                error: e.to_string(),
            })?;

        if !access_validation.has_access {
            return Err(WorkflowError::TenantAccessDenied(
                access_validation.reason.unwrap_or_else(|| "Access denied".to_string())
            ));
        }

        // Step 2: Save current session state (if switching from another tenant)
        if let Some(current_tenant_id) = &request.current_tenant_id {
            self.activities
                .save_session_state(crate::activities::SaveSessionStateRequest {
                    user_id: request.user_id.clone(),
                    current_tenant_id: current_tenant_id.clone(),
                    session_data: serde_json::json!({}), // Would contain actual session data
                })
                .await
                .map_err(|e| WorkflowError::ActivityFailed {
                    activity: "save_session_state".to_string(),
                    error: e.to_string(),
                })?;
        }

        // Step 3: Load target tenant context
        let tenant_context = self.activities
            .load_tenant_context(crate::activities::LoadTenantContextRequest {
                tenant_id: request.target_tenant_id.clone(),
                user_id: request.user_id.clone(),
            })
            .await
            .map_err(|e| WorkflowError::ActivityFailed {
                activity: "load_tenant_context".to_string(),
                error: e.to_string(),
            })?;

        // Step 4: Create new session for target tenant
        let new_session = self.activities
            .create_tenant_session(crate::activities::CreateTenantSessionRequest {
                user_id: request.user_id.clone(),
                tenant_id: request.target_tenant_id.clone(),
                tenant_context: tenant_context.clone(),
                session_duration: request.session_duration,
            })
            .await
            .map_err(|e| WorkflowError::ActivityFailed {
                activity: "create_tenant_session".to_string(),
                error: e.to_string(),
            })?;

        // Step 5: Update user's active tenant
        self.activities
            .update_user_active_tenant(crate::activities::UpdateUserActiveTenantRequest {
                user_id: request.user_id,
                new_active_tenant_id: request.target_tenant_id,
            })
            .await
            .map_err(|e| WorkflowError::ActivityFailed {
                activity: "update_user_active_tenant".to_string(),
                error: e.to_string(),
            })?;

        tracing::info!("Successfully switched tenant for user");

        Ok(SwitchTenantWorkflowResult {
            success: true,
            new_session_id: new_session.session_id,
            tenant_context,
            available_features: new_session.available_features,
        })
    }

    // Tenant migration workflow - migrate tenant between subscription tiers
    pub async fn migrate_tenant_workflow(
        &self,
        tenant_id: TenantId,
        target_tier: adx_shared::types::SubscriptionTier,
    ) -> Result<(), WorkflowError> {
        tracing::info!("Starting tenant migration workflow for tenant: {} to tier: {:?}", 
                      tenant_id, target_tier);

        // This would implement the complex logic for migrating a tenant
        // between subscription tiers, including:
        // 1. Backup current tenant data
        // 2. Update tenant configuration
        // 3. Migrate data if needed
        // 4. Update quotas and features
        // 5. Notify tenant admin

        // For now, we'll just simulate the workflow
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        tracing::info!("Successfully migrated tenant: {}", tenant_id);

        Ok(())
    }

    // Tenant suspension workflow - suspend tenant for non-payment or violations
    pub async fn suspend_tenant_workflow(
        &self,
        tenant_id: TenantId,
        reason: String,
    ) -> Result<(), WorkflowError> {
        tracing::info!("Starting tenant suspension workflow for tenant: {} with reason: {}", 
                      tenant_id, reason);

        // This would implement the complex logic for suspending a tenant:
        // 1. Validate suspension request
        // 2. Notify tenant admin
        // 3. Disable tenant access
        // 4. Preserve tenant data
        // 5. Update billing status

        // For now, we'll just simulate the workflow
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        tracing::info!("Successfully suspended tenant: {}", tenant_id);

        Ok(())
    }

    // Tenant termination workflow - permanently delete tenant and all data
    pub async fn terminate_tenant_workflow(
        &self,
        tenant_id: TenantId,
        export_data: bool,
    ) -> Result<(), WorkflowError> {
        tracing::info!("Starting tenant termination workflow for tenant: {} (export_data: {})", 
                      tenant_id, export_data);

        // This would implement the complex logic for terminating a tenant:
        // 1. Validate termination request
        // 2. Export tenant data if requested
        // 3. Notify all tenant users
        // 4. Delete all tenant data
        // 5. Clean up database resources
        // 6. Update billing status

        // For now, we'll just simulate the workflow
        if export_data {
            tracing::info!("Exporting tenant data before termination");
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
        }

        // Clean up database
        self.activities
            .cleanup_tenant_database(&tenant_id)
            .await
            .map_err(|e| WorkflowError::ActivityFailed {
                activity: "cleanup_tenant_database".to_string(),
                error: e.to_string(),
            })?;

        tracing::info!("Successfully terminated tenant: {}", tenant_id);

        Ok(())
    }
}

// Workflow factory for creating workflow instances
pub struct TenantWorkflowFactory {
    activities: Arc<dyn TenantActivities>,
}

impl TenantWorkflowFactory {
    pub fn new(activities: Arc<dyn TenantActivities>) -> Self {
        Self { activities }
    }

    pub fn create_workflows(&self) -> TenantWorkflows {
        TenantWorkflows::new(self.activities.clone())
    }
}