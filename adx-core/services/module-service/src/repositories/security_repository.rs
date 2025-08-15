use async_trait::async_trait;
use sqlx::PgPool;

use crate::error::ModuleServiceError;
use crate::models::ModuleSecurityScanRecord;
use crate::activities::PerformSecurityScanResponse;

#[async_trait]
pub trait SecurityRepositoryTrait {
    async fn store_scan_results(&self, module_id: &str, version: &str, scan_type: &str, scan_results: &PerformSecurityScanResponse) -> Result<(), ModuleServiceError>;
    async fn get_latest_scan(&self, module_id: &str, version: &str) -> Result<Option<ModuleSecurityScanRecord>, ModuleServiceError>;
    async fn list_scans(&self, module_id: &str) -> Result<Vec<ModuleSecurityScanRecord>, ModuleServiceError>;
}

pub struct SecurityRepository {
    pool: PgPool,
}

impl SecurityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl SecurityRepositoryTrait for SecurityRepository {
    async fn store_scan_results(&self, module_id: &str, version: &str, scan_type: &str, scan_results: &PerformSecurityScanResponse) -> Result<(), ModuleServiceError> {
        let vulnerabilities_json = serde_json::to_value(&scan_results.vulnerabilities)?;
        
        sqlx::query(
            r#"
            INSERT INTO module_security_scans (
                module_id, version, scan_type, scanner_version, passed, score,
                vulnerabilities_json, scan_duration_seconds, scanned_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
            "#,
        )
        .bind(module_id)
        .bind(version)
        .bind(scan_type)
        .bind("1.0.0") // Scanner version
        .bind(scan_results.passed)
        .bind(scan_results.score as i16)
        .bind(vulnerabilities_json)
        .bind(scan_results.scan_duration_seconds as i32)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_latest_scan(&self, module_id: &str, version: &str) -> Result<Option<ModuleSecurityScanRecord>, ModuleServiceError> {
        let record = sqlx::query_as::<_, ModuleSecurityScanRecord>(
            r#"
            SELECT * FROM module_security_scans 
            WHERE module_id = $1 AND version = $2 
            ORDER BY scanned_at DESC 
            LIMIT 1
            "#
        )
        .bind(module_id)
        .bind(version)
        .fetch_optional(&self.pool)
        .await?;

        Ok(record)
    }

    async fn list_scans(&self, module_id: &str) -> Result<Vec<ModuleSecurityScanRecord>, ModuleServiceError> {
        let records = sqlx::query_as::<_, ModuleSecurityScanRecord>(
            "SELECT * FROM module_security_scans WHERE module_id = $1 ORDER BY scanned_at DESC"
        )
        .bind(module_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}