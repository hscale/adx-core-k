use async_trait::async_trait;
use sqlx::PgPool;

use crate::error::ModuleServiceError;
use crate::models::ModuleWorkflowRecord;

#[async_trait]
pub trait WorkflowRepositoryTrait {
    async fn create_workflow(&self, workflow: &ModuleWorkflowRecord) -> Result<ModuleWorkflowRecord, ModuleServiceError>;
    async fn get_workflow(&self, id: &str) -> Result<Option<ModuleWorkflowRecord>, ModuleServiceError>;
    async fn update_workflow_status(&self, id: &str, status: &str, output: Option<serde_json::Value>, error: Option<&str>) -> Result<(), ModuleServiceError>;
    async fn list_workflows(&self, tenant_id: &str) -> Result<Vec<ModuleWorkflowRecord>, ModuleServiceError>;
}

pub struct WorkflowRepository {
    pool: PgPool,
}

impl WorkflowRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl WorkflowRepositoryTrait for WorkflowRepository {
    async fn create_workflow(&self, workflow: &ModuleWorkflowRecord) -> Result<ModuleWorkflowRecord, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleWorkflowRecord>(
            r#"
            INSERT INTO module_workflows (
                id, workflow_type, module_id, tenant_id, status, input_json, started_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
        )
        .bind(&workflow.id)
        .bind(&workflow.workflow_type)
        .bind(&workflow.module_id)
        .bind(&workflow.tenant_id)
        .bind(&workflow.status)
        .bind(&workflow.input_json)
        .bind(workflow.started_at)
        .fetch_one(&self.pool)
        .await?;

        Ok(record)
    }

    async fn get_workflow(&self, id: &str) -> Result<Option<ModuleWorkflowRecord>, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleWorkflowRecord>(
            "SELECT * FROM module_workflows WHERE id = $1"
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn update_workflow_status(&self, id: &str, status: &str, output: Option<serde_json::Value>, error: Option<&str>) -> Result<(), ModuleServiceError> {
        sqlx::query(
            r#"
            UPDATE module_workflows 
            SET status = $1, output_json = $2, error_message = $3, completed_at = CASE WHEN $1 IN ('completed', 'failed', 'cancelled') THEN NOW() ELSE completed_at END
            WHERE id = $4
            "#
        )
        .bind(status)
        .bind(output)
        .bind(error)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn list_workflows(&self, tenant_id: &str) -> Result<Vec<ModuleWorkflowRecord>, ModuleServiceError> {
        let records = sqlx::query_as::<_, ModuleWorkflowRecord>(
            "SELECT * FROM module_workflows WHERE tenant_id = $1 ORDER BY started_at DESC"
        )
        .bind(tenant_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}