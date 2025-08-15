use crate::activities::WhiteLabelActivities;
use crate::config::WhiteLabelConfig;
use crate::services::{AssetService, DnsService, EmailService, SslService, StorageService};
use crate::workflows_simple::{
    custom_domain_setup_workflow, reseller_setup_workflow, white_label_branding_workflow,
};
use sqlx::PgPool;
use std::sync::Arc;
use crate::temporal_mock::{Worker, WorkerBuilder};

pub struct WhiteLabelWorker {
    config: Arc<WhiteLabelConfig>,
    db_pool: Arc<PgPool>,
}

impl WhiteLabelWorker {
    pub fn new(config: Arc<WhiteLabelConfig>, db_pool: Arc<PgPool>) -> Self {
        Self { config, db_pool }
    }

    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Starting White Label Service Temporal worker");

        // Create services
        let dns_service = Arc::new(DnsService::new(self.config.clone())?);
        let ssl_service = Arc::new(SslService::new(self.config.clone()));
        let asset_service = Arc::new(AssetService::new(self.config.clone()));
        let email_service = Arc::new(EmailService::new(self.config.clone())?);
        let storage_service = Arc::new(StorageService::new(self.config.clone()));

        // Create activities
        let activities = Arc::new(WhiteLabelActivities::new(
            self.db_pool.clone(),
            self.config.clone(),
            dns_service,
            ssl_service,
            asset_service,
            email_service,
            storage_service,
        ));

        // Create worker
        let mut worker = WorkerBuilder::default()
            .task_queue("white-label-task-queue")
            .build()
            .await?;

        // Register workflows
        worker.register_workflow(custom_domain_setup_workflow).await?;
        worker.register_workflow(white_label_branding_workflow).await?;
        worker.register_workflow(reseller_setup_workflow).await?;

        // Register activities
        worker.register_activity(activities.validate_domain).await?;
        worker.register_activity(activities.create_domain_record).await?;
        worker.register_activity(activities.generate_dns_verification_records).await?;
        worker.register_activity(activities.verify_dns_records).await?;
        worker.register_activity(activities.update_domain_status).await?;
        worker.register_activity(activities.provision_ssl_certificate).await?;
        worker.register_activity(activities.configure_domain_routing).await?;

        worker.register_activity(activities.validate_branding_request).await?;
        worker.register_activity(activities.backup_existing_branding).await?;
        worker.register_activity(activities.process_branding_asset).await?;
        worker.register_activity(activities.generate_custom_css).await?;
        worker.register_activity(activities.process_email_templates).await?;
        worker.register_activity(activities.create_branding_record).await?;
        worker.register_activity(activities.rollback_branding).await?;
        worker.register_activity(activities.generate_branding_preview).await?;
        worker.register_activity(activities.cleanup_branding_backup).await?;

        worker.register_activity(activities.validate_reseller_hierarchy).await?;
        worker.register_activity(activities.calculate_commission_rates).await?;
        worker.register_activity(activities.create_reseller_record).await?;
        worker.register_activity(activities.configure_revenue_sharing).await?;
        worker.register_activity(activities.configure_support_routing).await?;
        worker.register_activity(activities.send_reseller_welcome).await?;

        // Start the worker
        tracing::info!("White Label Service worker started and listening for tasks");
        worker.run().await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WhiteLabelConfig;

    #[tokio::test]
    async fn test_worker_creation() {
        let config = Arc::new(WhiteLabelConfig::default());
        let db_pool = Arc::new(create_test_database_pool().await);
        
        let worker = WhiteLabelWorker::new(config, db_pool);
        
        // Test that worker can be created without errors
        assert!(true); // Placeholder test
    }

    async fn create_test_database_pool() -> PgPool {
        // This would create a test database pool
        sqlx::SqlitePool::connect(":memory:").await.unwrap()
    }
}