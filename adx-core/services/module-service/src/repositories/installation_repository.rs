use async_trait::async_trait;
use sqlx::PgPool;

use crate::error::ModuleServiceError;
use crate::models::ModuleInstallationRecord;
use crate::types::ModuleStatus;

#[async_trait]
pub trait InstallationRepositoryTrait {
    async fn create_installation(&self, installation: &ModuleInstallationRecord) -> Result<ModuleInstallationRecord, ModuleServiceError>;
    async fn get_installation(&self, module_id: &str, tenant_id: &str) -> Result<Option<ModuleInstallationRecord>, ModuleServiceError>;
    async fn get_installation_by_id(&self, id: &str) -> Result<Option<ModuleInstallationRecord>, ModuleServiceError>;
    async fn update_installation_status(&self, id: &str, status: &ModuleStatus) -> Result<(), ModuleServiceError>;
    async fn delete_installation(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError>;
    async fn list_installations(&self, tenant_id: &str) -> Result<Vec<ModuleInstallationRecord>, ModuleServiceError>;
}

pub struct InstallationRepository {
    pool: PgPool,
}

impl InstallationRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl InstallationRepositoryTrait for InstallationRepository {
    async fn create_installation(&self, installation: &ModuleInstallationRecord) -> Result<ModuleInstallationRecord, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleInstallationRecord>(
            r#"
            INSERT INTO module_installations (
                id, module_id, tenant_id, version, status, configuration_json,
                installation_path, sandbox_config_json, installed_by, installed_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            RETURNING *
            "#,
        )
        .bind(&installation.id)
        .bind(&installation.module_id)
        .bind(&installation.tenant_id)
        .bind(&installation.version)
        .bind(&installation.status)
        .bind(&installation.configuration_json)
        .bind(&installation.installation_path)
        .bind(&installation.sandbox_config_json)
        .bind(&installation.installed_by)
        .bind(installation.installed_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_installation(&self, module_id: &str, tenant_id: &str) -> Result<Option<ModuleInstallationRecord>, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleInstallationRecord>(
            "SELECT * FROM module_installations WHERE module_id = $1 AND tenant_id = $2"
        )
        .bind(module_id)
        .bind(tenant_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_installation_by_id(&self, id: &str) -> Result<Option<ModuleInstallationRecord>, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleInstallationRecord>(
            "SELECT * FROM module_installations WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn update_installation_status(&self, id: &str, status: &ModuleStatus) -> Result<(), ModuleServiceError> {
        let status_str = serde_json::to_string(status)?.trim_matches('"').to_string();
        
        sqlx::query(
            "UPDATE module_installations SET status = $1 WHERE id = $2"
        )
        .bind(status_str)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_installation(&self, module_id: &str, tenant_id: &str) -> Result<(), ModuleServiceError> {
        sqlx::query(
            "DELETE FROM module_installations WHERE module_id = $1 AND tenant_id = $2"
        )
        .bind(module_id)
        .bind(tenant_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_installations(&self, tenant_id: &str) -> Result<Vec<ModuleInstallationRecord>, ModuleServiceError> {
        let records = sqlx::query_as::<_, ModuleInstallationRecord>(
            "SELECT * FROM module_installations WHERE tenant_id = $1 ORDER BY installed_at DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}