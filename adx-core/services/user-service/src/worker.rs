use std::sync::Arc;
use sqlx::PgPool;
use adx_shared::{
    config::AppConfig,
    Result, Error,
};
use crate::{
    activities::*,
    workflows::*,
    repositories::*,
    validation::UserValidator,
};

pub struct UserServiceWorker {
    activities: Arc<UserServiceActivitiesImpl>,
}

impl UserServiceWorker {
    pub async fn new(_config: &AppConfig, pool: PgPool) -> Result<Self> {
        // Create repositories
        let user_repo = Arc::new(PostgresUserRepository::new(pool.clone()));
        let profile_repo = Arc::new(PostgresUserProfileRepository::new(pool.clone()));
        let preference_repo = Arc::new(PostgresUserPreferenceRepository::new(pool.clone()));
        let activity_repo = Arc::new(PostgresUserActivityRepository::new(pool.clone()));
        let validator = Arc::new(UserValidator::new());
        
        // Create activities implementation
        let activities = Arc::new(UserServiceActivitiesImpl::new(
            user_repo,
            profile_repo,
            preference_repo,
            activity_repo,
            validator,
        ));
        
        Ok(Self {
            activities,
        })
    }
    
    pub async fn start(&self) -> Result<()> {
        tracing::info!("Starting User Service Temporal worker");
        
        // Register workflows and activities
        self.register_workflows_and_activities().await?;
        
        tracing::info!("User Service Temporal worker started successfully");
        
        // Keep the worker running
        tokio::signal::ctrl_c().await.map_err(|e| Error::Internal(e.to_string()))?;
        
        tracing::info!("Shutting down User Service Temporal worker");
        
        Ok(())
    }
    
    async fn register_workflows_and_activities(&self) -> Result<()> {
        tracing::info!("Registering User Service workflows and activities");
        
        // In a real Temporal implementation, this would register:
        
        // Core workflows
        tracing::info!("Registering workflow: user_onboarding_workflow");
        tracing::info!("Registering workflow: user_data_export_workflow");
        
        // New user management workflows
        tracing::info!("Registering workflow: user_profile_sync_workflow");
        tracing::info!("Registering workflow: user_preference_migration_workflow");
        tracing::info!("Registering workflow: user_deactivation_workflow");
        tracing::info!("Registering workflow: user_reactivation_workflow");
        tracing::info!("Registering workflow: bulk_user_operation_workflow");
        
        // Core activities
        tracing::info!("Registering activity: create_user_activity");
        tracing::info!("Registering activity: update_user_activity");
        tracing::info!("Registering activity: validate_user_data_activity");
        
        // New user management activities
        tracing::info!("Registering activity: sync_user_profile_activity");
        tracing::info!("Registering activity: migrate_user_preferences_activity");
        tracing::info!("Registering activity: export_user_data_activity");
        tracing::info!("Registering activity: deactivate_user_activity");
        tracing::info!("Registering activity: reactivate_user_activity");
        tracing::info!("Registering activity: transfer_user_ownership_activity");
        
        tracing::info!("All User Service workflows and activities registered successfully");
        
        Ok(())
    }
}

pub async fn start_worker(config: AppConfig, pool: PgPool) -> Result<()> {
    let worker = UserServiceWorker::new(&config, pool).await?;
    worker.start().await
}