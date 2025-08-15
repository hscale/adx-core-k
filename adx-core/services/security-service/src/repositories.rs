use crate::{
    error::{SecurityError, SecurityResult},
    models::{
        AuditLog, AuditEventCategory, AuditOutcome, ComplianceReport, ComplianceReportType,
        ComplianceStatus, GdprRequest, GdprRequestType, GdprRequestStatus, DataRetentionPolicy,
        DataRetentionJob, RetentionJobStatus, SecurityScan, ScanType, ScanStatus, Vulnerability,
        ZeroTrustPolicy, ZeroTrustPolicyType, SecurityEvent, SecurityEventType,
        SecurityEventSeverity, SecurityEventStatus
    },
};
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tracing::{info, error};
use uuid::Uuid;

// Audit Repository
#[derive(Clone)]
pub struct AuditRepository {
    pool: Arc<PgPool>,
}

impl AuditRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn batch_insert_logs(&self, logs: Vec<AuditLog>) -> SecurityResult<()> {
        let mut tx = self.pool.begin().await?;

        for log in logs {
            sqlx::query!(
                r#"
                INSERT INTO audit_logs (
                    id, tenant_id, user_id, session_id, event_type, event_category,
                    resource_type, resource_id, action, outcome, ip_address, user_agent,
                    request_id, details, risk_score, created_at
                ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                "#,
                log.id,
                log.tenant_id,
                log.user_id,
                log.session_id,
                log.event_type,
                log.event_category as AuditEventCategory,
                log.resource_type,
                log.resource_id,
                log.action,
                log.outcome as AuditOutcome,
                log.ip_address,
                log.user_agent,
                log.request_id,
                log.details,
                log.risk_score,
                log.created_at
            )
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_audit_logs(
        &self,
        tenant_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        event_category: Option<AuditEventCategory>,
        user_id: Option<&str>,
        resource_type: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<Vec<AuditLog>> {
        let offset = (page - 1) * page_size;

        let mut query = sqlx::QueryBuilder::new(
            "SELECT id, tenant_id, user_id, session_id, event_type, event_category,
             resource_type, resource_id, action, outcome, ip_address, user_agent,
             request_id, details, risk_score, created_at FROM audit_logs WHERE tenant_id = "
        );
        query.push_bind(tenant_id);

        if let Some(start) = start_date {
            query.push(" AND created_at >= ").push_bind(start);
        }
        if let Some(end) = end_date {
            query.push(" AND created_at <= ").push_bind(end);
        }
        if let Some(category) = event_category {
            query.push(" AND event_category = ").push_bind(category);
        }
        if let Some(uid) = user_id {
            query.push(" AND user_id = ").push_bind(uid);
        }
        if let Some(rtype) = resource_type {
            query.push(" AND resource_type = ").push_bind(rtype);
        }

        query.push(" ORDER BY created_at DESC LIMIT ").push_bind(page_size);
        query.push(" OFFSET ").push_bind(offset);

        let logs = query
            .build_query_as::<AuditLog>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(logs)
    }

    pub async fn count_audit_logs(
        &self,
        tenant_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        event_category: Option<AuditEventCategory>,
        user_id: Option<&str>,
        resource_type: Option<&str>,
    ) -> SecurityResult<i64> {
        let mut query = sqlx::QueryBuilder::new(
            "SELECT COUNT(*) FROM audit_logs WHERE tenant_id = "
        );
        query.push_bind(tenant_id);

        if let Some(start) = start_date {
            query.push(" AND created_at >= ").push_bind(start);
        }
        if let Some(end) = end_date {
            query.push(" AND created_at <= ").push_bind(end);
        }
        if let Some(category) = event_category {
            query.push(" AND event_category = ").push_bind(category);
        }
        if let Some(uid) = user_id {
            query.push(" AND user_id = ").push_bind(uid);
        }
        if let Some(rtype) = resource_type {
            query.push(" AND resource_type = ").push_bind(rtype);
        }

        let count: i64 = query
            .build_query_scalar()
            .fetch_one(&*self.pool)
            .await?;

        Ok(count)
    }

    pub async fn get_all_audit_logs(
        &self,
        tenant_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
    ) -> SecurityResult<Vec<AuditLog>> {
        let logs = sqlx::query_as!(
            AuditLog,
            r#"
            SELECT id, tenant_id, user_id, session_id, event_type, 
                   event_category as "event_category: AuditEventCategory",
                   resource_type, resource_id, action, 
                   outcome as "outcome: AuditOutcome",
                   ip_address, user_agent, request_id, details, risk_score, created_at
            FROM audit_logs 
            WHERE tenant_id = $1 AND created_at >= $2 AND created_at <= $3
            ORDER BY created_at DESC
            "#,
            tenant_id,
            start_date,
            end_date
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn delete_old_logs(&self, tenant_id: &str, cutoff_date: DateTime<Utc>) -> SecurityResult<i64> {
        let result = sqlx::query!(
            "DELETE FROM audit_logs WHERE tenant_id = $1 AND created_at < $2",
            tenant_id,
            cutoff_date
        )
        .execute(&*self.pool)
        .await?;

        Ok(result.rows_affected() as i64)
    }
}

// GDPR Repository
#[derive(Clone)]
pub struct GdprRepository {
    pool: Arc<PgPool>,
}

impl GdprRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_request(&self, request: GdprRequest) -> SecurityResult<GdprRequest> {
        sqlx::query!(
            r#"
            INSERT INTO gdpr_requests (
                id, tenant_id, user_id, request_type, status, requester_email,
                verification_token, verified_at, processed_at, data_export_url,
                deletion_confirmation, notes, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            request.id,
            request.tenant_id,
            request.user_id,
            request.request_type as GdprRequestType,
            request.status as GdprRequestStatus,
            request.requester_email,
            request.verification_token,
            request.verified_at,
            request.processed_at,
            request.data_export_url,
            request.deletion_confirmation,
            request.notes,
            request.created_at,
            request.updated_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(request)
    }

    pub async fn get_request(&self, request_id: Uuid) -> SecurityResult<Option<GdprRequest>> {
        let request = sqlx::query_as!(
            GdprRequest,
            r#"
            SELECT id, tenant_id, user_id, 
                   request_type as "request_type: GdprRequestType",
                   status as "status: GdprRequestStatus",
                   requester_email, verification_token, verified_at, processed_at,
                   data_export_url, deletion_confirmation, notes, created_at, updated_at
            FROM gdpr_requests WHERE id = $1
            "#,
            request_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(request)
    }

    pub async fn update_request(&self, request: GdprRequest) -> SecurityResult<GdprRequest> {
        sqlx::query!(
            r#"
            UPDATE gdpr_requests SET
                status = $2, verified_at = $3, processed_at = $4,
                data_export_url = $5, deletion_confirmation = $6,
                notes = $7, updated_at = $8
            WHERE id = $1
            "#,
            request.id,
            request.status as GdprRequestStatus,
            request.verified_at,
            request.processed_at,
            request.data_export_url,
            request.deletion_confirmation,
            request.notes,
            request.updated_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(request)
    }

    pub async fn get_pending_request(
        &self,
        tenant_id: &str,
        user_id: &str,
        request_type: GdprRequestType,
    ) -> SecurityResult<Option<GdprRequest>> {
        let request = sqlx::query_as!(
            GdprRequest,
            r#"
            SELECT id, tenant_id, user_id, 
                   request_type as "request_type: GdprRequestType",
                   status as "status: GdprRequestStatus",
                   requester_email, verification_token, verified_at, processed_at,
                   data_export_url, deletion_confirmation, notes, created_at, updated_at
            FROM gdpr_requests 
            WHERE tenant_id = $1 AND user_id = $2 AND request_type = $3 
            AND status IN ('pending', 'verified', 'processing')
            ORDER BY created_at DESC LIMIT 1
            "#,
            tenant_id,
            user_id,
            request_type as GdprRequestType
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(request)
    }

    pub async fn get_verified_requests(&self) -> SecurityResult<Vec<GdprRequest>> {
        let requests = sqlx::query_as!(
            GdprRequest,
            r#"
            SELECT id, tenant_id, user_id, 
                   request_type as "request_type: GdprRequestType",
                   status as "status: GdprRequestStatus",
                   requester_email, verification_token, verified_at, processed_at,
                   data_export_url, deletion_confirmation, notes, created_at, updated_at
            FROM gdpr_requests WHERE status = 'verified'
            ORDER BY created_at ASC
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(requests)
    }

    pub async fn get_tenant_requests(
        &self,
        tenant_id: &str,
        request_type: Option<GdprRequestType>,
        status: Option<GdprRequestStatus>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<Vec<GdprRequest>> {
        let offset = (page - 1) * page_size;

        let mut query = sqlx::QueryBuilder::new(
            r#"
            SELECT id, tenant_id, user_id, request_type, status,
                   requester_email, verification_token, verified_at, processed_at,
                   data_export_url, deletion_confirmation, notes, created_at, updated_at
            FROM gdpr_requests WHERE tenant_id = 
            "#
        );
        query.push_bind(tenant_id);

        if let Some(rtype) = request_type {
            query.push(" AND request_type = ").push_bind(rtype);
        }
        if let Some(stat) = status {
            query.push(" AND status = ").push_bind(stat);
        }

        query.push(" ORDER BY created_at DESC LIMIT ").push_bind(page_size);
        query.push(" OFFSET ").push_bind(offset);

        let requests = query
            .build_query_as::<GdprRequest>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(requests)
    }
}

// Data Retention Repository
#[derive(Clone)]
pub struct RetentionRepository {
    pool: Arc<PgPool>,
}

impl RetentionRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_policy(&self, policy: DataRetentionPolicy) -> SecurityResult<DataRetentionPolicy> {
        sqlx::query!(
            r#"
            INSERT INTO data_retention_policies (
                id, tenant_id, resource_type, retention_period_days,
                deletion_method, enabled, created_by, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            "#,
            policy.id,
            policy.tenant_id,
            policy.resource_type,
            policy.retention_period_days,
            policy.deletion_method as crate::models::DeletionMethod,
            policy.enabled,
            policy.created_by,
            policy.created_at,
            policy.updated_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(policy)
    }

    pub async fn get_policy(&self, policy_id: Uuid) -> SecurityResult<Option<DataRetentionPolicy>> {
        let policy = sqlx::query_as!(
            DataRetentionPolicy,
            r#"
            SELECT id, tenant_id, resource_type, retention_period_days,
                   deletion_method as "deletion_method: crate::models::DeletionMethod",
                   enabled, created_by, created_at, updated_at
            FROM data_retention_policies WHERE id = $1
            "#,
            policy_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(policy)
    }

    pub async fn update_policy(&self, policy: DataRetentionPolicy) -> SecurityResult<DataRetentionPolicy> {
        sqlx::query!(
            r#"
            UPDATE data_retention_policies SET
                retention_period_days = $2, deletion_method = $3,
                enabled = $4, updated_at = $5
            WHERE id = $1
            "#,
            policy.id,
            policy.retention_period_days,
            policy.deletion_method as crate::models::DeletionMethod,
            policy.enabled,
            policy.updated_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(policy)
    }

    pub async fn delete_policy(&self, policy_id: Uuid) -> SecurityResult<()> {
        sqlx::query!("DELETE FROM data_retention_policies WHERE id = $1", policy_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_tenant_policies(&self, tenant_id: &str) -> SecurityResult<Vec<DataRetentionPolicy>> {
        let policies = sqlx::query_as!(
            DataRetentionPolicy,
            r#"
            SELECT id, tenant_id, resource_type, retention_period_days,
                   deletion_method as "deletion_method: crate::models::DeletionMethod",
                   enabled, created_by, created_at, updated_at
            FROM data_retention_policies WHERE tenant_id = $1
            ORDER BY created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(policies)
    }

    pub async fn get_policy_by_resource(
        &self,
        tenant_id: &str,
        resource_type: &str,
    ) -> SecurityResult<Option<DataRetentionPolicy>> {
        let policy = sqlx::query_as!(
            DataRetentionPolicy,
            r#"
            SELECT id, tenant_id, resource_type, retention_period_days,
                   deletion_method as "deletion_method: crate::models::DeletionMethod",
                   enabled, created_by, created_at, updated_at
            FROM data_retention_policies 
            WHERE tenant_id = $1 AND resource_type = $2
            "#,
            tenant_id,
            resource_type
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(policy)
    }

    pub async fn get_active_policies(&self) -> SecurityResult<Vec<DataRetentionPolicy>> {
        let policies = sqlx::query_as!(
            DataRetentionPolicy,
            r#"
            SELECT id, tenant_id, resource_type, retention_period_days,
                   deletion_method as "deletion_method: crate::models::DeletionMethod",
                   enabled, created_by, created_at, updated_at
            FROM data_retention_policies WHERE enabled = true
            ORDER BY created_at DESC
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(policies)
    }

    pub async fn create_job(&self, job: DataRetentionJob) -> SecurityResult<DataRetentionJob> {
        sqlx::query!(
            r#"
            INSERT INTO data_retention_jobs (
                id, tenant_id, policy_id, resource_type, scheduled_for,
                status, records_processed, records_deleted, error_message,
                started_at, completed_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            job.id,
            job.tenant_id,
            job.policy_id,
            job.resource_type,
            job.scheduled_for,
            job.status as RetentionJobStatus,
            job.records_processed,
            job.records_deleted,
            job.error_message,
            job.started_at,
            job.completed_at,
            job.created_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(job)
    }

    pub async fn get_job(&self, job_id: Uuid) -> SecurityResult<Option<DataRetentionJob>> {
        let job = sqlx::query_as!(
            DataRetentionJob,
            r#"
            SELECT id, tenant_id, policy_id, resource_type, scheduled_for,
                   status as "status: RetentionJobStatus",
                   records_processed, records_deleted, error_message,
                   started_at, completed_at, created_at
            FROM data_retention_jobs WHERE id = $1
            "#,
            job_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(job)
    }

    pub async fn update_job(&self, job: DataRetentionJob) -> SecurityResult<DataRetentionJob> {
        sqlx::query!(
            r#"
            UPDATE data_retention_jobs SET
                status = $2, records_processed = $3, records_deleted = $4,
                error_message = $5, started_at = $6, completed_at = $7
            WHERE id = $1
            "#,
            job.id,
            job.status as RetentionJobStatus,
            job.records_processed,
            job.records_deleted,
            job.error_message,
            job.started_at,
            job.completed_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(job)
    }

    pub async fn get_pending_jobs(&self) -> SecurityResult<Vec<DataRetentionJob>> {
        let jobs = sqlx::query_as!(
            DataRetentionJob,
            r#"
            SELECT id, tenant_id, policy_id, resource_type, scheduled_for,
                   status as "status: RetentionJobStatus",
                   records_processed, records_deleted, error_message,
                   started_at, completed_at, created_at
            FROM data_retention_jobs WHERE status = 'scheduled'
            ORDER BY scheduled_for ASC
            "#
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(jobs)
    }

    pub async fn get_pending_jobs_for_policy(&self, policy_id: Uuid) -> SecurityResult<Vec<DataRetentionJob>> {
        let jobs = sqlx::query_as!(
            DataRetentionJob,
            r#"
            SELECT id, tenant_id, policy_id, resource_type, scheduled_for,
                   status as "status: RetentionJobStatus",
                   records_processed, records_deleted, error_message,
                   started_at, completed_at, created_at
            FROM data_retention_jobs 
            WHERE policy_id = $1 AND status IN ('scheduled', 'running')
            ORDER BY scheduled_for ASC
            "#,
            policy_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(jobs)
    }

    pub async fn get_tenant_scheduled_jobs(&self, tenant_id: &str) -> SecurityResult<Vec<DataRetentionJob>> {
        let jobs = sqlx::query_as!(
            DataRetentionJob,
            r#"
            SELECT id, tenant_id, policy_id, resource_type, scheduled_for,
                   status as "status: RetentionJobStatus",
                   records_processed, records_deleted, error_message,
                   started_at, completed_at, created_at
            FROM data_retention_jobs 
            WHERE tenant_id = $1 AND status = 'scheduled'
            ORDER BY scheduled_for ASC
            "#,
            tenant_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(jobs)
    }

    pub async fn count_records_to_delete(&self, tenant_id: &str) -> SecurityResult<i64> {
        // This would query actual data tables to count records eligible for deletion
        // For now, return a mock count
        Ok(0)
    }

    pub async fn get_next_cleanup_time(&self, tenant_id: &str) -> SecurityResult<Option<DateTime<Utc>>> {
        let next_time = sqlx::query_scalar!(
            "SELECT MIN(scheduled_for) FROM data_retention_jobs WHERE tenant_id = $1 AND status = 'scheduled'",
            tenant_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(next_time.flatten())
    }
}

// Security Scanning Repository
#[derive(Clone)]
pub struct ScanningRepository {
    pool: Arc<PgPool>,
}

impl ScanningRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_scan(&self, scan: SecurityScan) -> SecurityResult<SecurityScan> {
        sqlx::query!(
            r#"
            INSERT INTO security_scans (
                id, tenant_id, scan_type, target, status, severity_threshold,
                vulnerabilities_found, critical_count, high_count, medium_count, low_count,
                scan_results, remediation_suggestions, started_at, completed_at, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
            "#,
            scan.id,
            scan.tenant_id,
            scan.scan_type as ScanType,
            scan.target,
            scan.status as ScanStatus,
            scan.severity_threshold,
            scan.vulnerabilities_found,
            scan.critical_count,
            scan.high_count,
            scan.medium_count,
            scan.low_count,
            scan.scan_results,
            scan.remediation_suggestions,
            scan.started_at,
            scan.completed_at,
            scan.created_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(scan)
    }

    pub async fn get_scan(&self, scan_id: Uuid) -> SecurityResult<Option<SecurityScan>> {
        let scan = sqlx::query_as!(
            SecurityScan,
            r#"
            SELECT id, tenant_id, scan_type as "scan_type: ScanType",
                   target, status as "status: ScanStatus", severity_threshold,
                   vulnerabilities_found, critical_count, high_count, medium_count, low_count,
                   scan_results, remediation_suggestions, started_at, completed_at, created_at
            FROM security_scans WHERE id = $1
            "#,
            scan_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(scan)
    }

    pub async fn update_scan(&self, scan: SecurityScan) -> SecurityResult<SecurityScan> {
        sqlx::query!(
            r#"
            UPDATE security_scans SET
                status = $2, vulnerabilities_found = $3, critical_count = $4,
                high_count = $5, medium_count = $6, low_count = $7,
                scan_results = $8, remediation_suggestions = $9,
                started_at = $10, completed_at = $11
            WHERE id = $1
            "#,
            scan.id,
            scan.status as ScanStatus,
            scan.vulnerabilities_found,
            scan.critical_count,
            scan.high_count,
            scan.medium_count,
            scan.low_count,
            scan.scan_results,
            scan.remediation_suggestions,
            scan.started_at,
            scan.completed_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(scan)
    }

    pub async fn save_vulnerability(&self, scan_id: Uuid, vulnerability: &Vulnerability) -> SecurityResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO vulnerabilities (
                id, scan_id, cve_id, title, description, severity, cvss_score,
                affected_component, fixed_version, references, discovered_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            Uuid::new_v4(),
            scan_id,
            vulnerability.cve_id,
            vulnerability.title,
            vulnerability.description,
            vulnerability.severity as crate::models::VulnerabilitySeverity,
            vulnerability.cvss_score,
            vulnerability.affected_component,
            vulnerability.fixed_version,
            &vulnerability.references,
            vulnerability.discovered_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_scan_vulnerabilities(&self, scan_id: Uuid) -> SecurityResult<Vec<Vulnerability>> {
        let vulnerabilities = sqlx::query!(
            r#"
            SELECT id, cve_id, title, description, 
                   severity as "severity: crate::models::VulnerabilitySeverity",
                   cvss_score, affected_component, fixed_version, references, discovered_at
            FROM vulnerabilities WHERE scan_id = $1
            ORDER BY severity DESC, cvss_score DESC
            "#,
            scan_id
        )
        .fetch_all(&*self.pool)
        .await?;

        let mut result = Vec::new();
        for row in vulnerabilities {
            result.push(Vulnerability {
                id: row.id.to_string(),
                cve_id: row.cve_id,
                title: row.title,
                description: row.description,
                severity: row.severity,
                cvss_score: row.cvss_score,
                affected_component: row.affected_component,
                fixed_version: row.fixed_version,
                references: row.references,
                discovered_at: row.discovered_at,
            });
        }

        Ok(result)
    }

    pub async fn get_tenant_scans(
        &self,
        tenant_id: &str,
        scan_type: Option<ScanType>,
        status: Option<ScanStatus>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<Vec<SecurityScan>> {
        let offset = (page - 1) * page_size;

        let mut query = sqlx::QueryBuilder::new(
            r#"
            SELECT id, tenant_id, scan_type, target, status, severity_threshold,
                   vulnerabilities_found, critical_count, high_count, medium_count, low_count,
                   scan_results, remediation_suggestions, started_at, completed_at, created_at
            FROM security_scans WHERE tenant_id = 
            "#
        );
        query.push_bind(tenant_id);

        if let Some(stype) = scan_type {
            query.push(" AND scan_type = ").push_bind(stype);
        }
        if let Some(stat) = status {
            query.push(" AND status = ").push_bind(stat);
        }

        query.push(" ORDER BY created_at DESC LIMIT ").push_bind(page_size);
        query.push(" OFFSET ").push_bind(offset);

        let scans = query
            .build_query_as::<SecurityScan>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(scans)
    }
}

// Zero Trust Repository
#[derive(Clone)]
pub struct ZeroTrustRepository {
    pool: Arc<PgPool>,
}

impl ZeroTrustRepository {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    pub async fn create_policy(&self, policy: ZeroTrustPolicy) -> SecurityResult<ZeroTrustPolicy> {
        sqlx::query!(
            r#"
            INSERT INTO zero_trust_policies (
                id, tenant_id, name, description, policy_type, conditions,
                actions, enabled, priority, created_by, created_at, updated_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#,
            policy.id,
            policy.tenant_id,
            policy.name,
            policy.description,
            policy.policy_type as ZeroTrustPolicyType,
            policy.conditions,
            policy.actions,
            policy.enabled,
            policy.priority,
            policy.created_by,
            policy.created_at,
            policy.updated_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(policy)
    }

    pub async fn get_policy(&self, policy_id: Uuid) -> SecurityResult<Option<ZeroTrustPolicy>> {
        let policy = sqlx::query_as!(
            ZeroTrustPolicy,
            r#"
            SELECT id, tenant_id, name, description,
                   policy_type as "policy_type: ZeroTrustPolicyType",
                   conditions, actions, enabled, priority, created_by, created_at, updated_at
            FROM zero_trust_policies WHERE id = $1
            "#,
            policy_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(policy)
    }

    pub async fn update_policy(&self, policy: ZeroTrustPolicy) -> SecurityResult<ZeroTrustPolicy> {
        sqlx::query!(
            r#"
            UPDATE zero_trust_policies SET
                name = $2, description = $3, conditions = $4, actions = $5,
                enabled = $6, priority = $7, updated_at = $8
            WHERE id = $1
            "#,
            policy.id,
            policy.name,
            policy.description,
            policy.conditions,
            policy.actions,
            policy.enabled,
            policy.priority,
            policy.updated_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(policy)
    }

    pub async fn delete_policy(&self, policy_id: Uuid) -> SecurityResult<()> {
        sqlx::query!("DELETE FROM zero_trust_policies WHERE id = $1", policy_id)
            .execute(&*self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_tenant_policies(&self, tenant_id: &str) -> SecurityResult<Vec<ZeroTrustPolicy>> {
        let policies = sqlx::query_as!(
            ZeroTrustPolicy,
            r#"
            SELECT id, tenant_id, name, description,
                   policy_type as "policy_type: ZeroTrustPolicyType",
                   conditions, actions, enabled, priority, created_by, created_at, updated_at
            FROM zero_trust_policies WHERE tenant_id = $1
            ORDER BY priority DESC, created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(policies)
    }

    pub async fn get_active_policies(&self, tenant_id: &str) -> SecurityResult<Vec<ZeroTrustPolicy>> {
        let policies = sqlx::query_as!(
            ZeroTrustPolicy,
            r#"
            SELECT id, tenant_id, name, description,
                   policy_type as "policy_type: ZeroTrustPolicyType",
                   conditions, actions, enabled, priority, created_by, created_at, updated_at
            FROM zero_trust_policies WHERE tenant_id = $1 AND enabled = true
            ORDER BY priority DESC, created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(policies)
    }

    pub async fn get_network_policies(&self, tenant_id: &str) -> SecurityResult<Vec<ZeroTrustPolicy>> {
        let policies = sqlx::query_as!(
            ZeroTrustPolicy,
            r#"
            SELECT id, tenant_id, name, description,
                   policy_type as "policy_type: ZeroTrustPolicyType",
                   conditions, actions, enabled, priority, created_by, created_at, updated_at
            FROM zero_trust_policies 
            WHERE tenant_id = $1 AND policy_type = 'networkaccess' AND enabled = true
            ORDER BY priority DESC, created_at DESC
            "#,
            tenant_id
        )
        .fetch_all(&*self.pool)
        .await?;

        Ok(policies)
    }

    pub async fn create_security_event(&self, event: SecurityEvent) -> SecurityResult<SecurityEvent> {
        sqlx::query!(
            r#"
            INSERT INTO security_events (
                id, tenant_id, event_type, severity, source_ip, user_id, device_id,
                resource, description, details, status, resolved_at, resolved_by, created_at
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
            event.id,
            event.tenant_id,
            event.event_type as SecurityEventType,
            event.severity as SecurityEventSeverity,
            event.source_ip,
            event.user_id,
            event.device_id,
            event.resource,
            event.description,
            event.details,
            event.status as SecurityEventStatus,
            event.resolved_at,
            event.resolved_by,
            event.created_at
        )
        .execute(&*self.pool)
        .await?;

        Ok(event)
    }

    pub async fn get_security_event(&self, event_id: Uuid) -> SecurityResult<Option<SecurityEvent>> {
        let event = sqlx::query_as!(
            SecurityEvent,
            r#"
            SELECT id, tenant_id, event_type as "event_type: SecurityEventType",
                   severity as "severity: SecurityEventSeverity",
                   source_ip, user_id, device_id, resource, description, details,
                   status as "status: SecurityEventStatus",
                   resolved_at, resolved_by, created_at
            FROM security_events WHERE id = $1
            "#,
            event_id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(event)
    }

    pub async fn update_security_event(&self, event: SecurityEvent) -> SecurityResult<SecurityEvent> {
        sqlx::query!(
            r#"
            UPDATE security_events SET
                status = $2, resolved_at = $3, resolved_by = $4, details = $5
            WHERE id = $1
            "#,
            event.id,
            event.status as SecurityEventStatus,
            event.resolved_at,
            event.resolved_by,
            event.details
        )
        .execute(&*self.pool)
        .await?;

        Ok(event)
    }

    pub async fn get_security_events(
        &self,
        tenant_id: &str,
        event_type: Option<SecurityEventType>,
        severity: Option<SecurityEventSeverity>,
        status: Option<SecurityEventStatus>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<Vec<SecurityEvent>> {
        let offset = (page - 1) * page_size;

        let mut query = sqlx::QueryBuilder::new(
            r#"
            SELECT id, tenant_id, event_type, severity, source_ip, user_id, device_id,
                   resource, description, details, status, resolved_at, resolved_by, created_at
            FROM security_events WHERE tenant_id = 
            "#
        );
        query.push_bind(tenant_id);

        if let Some(etype) = event_type {
            query.push(" AND event_type = ").push_bind(etype);
        }
        if let Some(sev) = severity {
            query.push(" AND severity = ").push_bind(sev);
        }
        if let Some(stat) = status {
            query.push(" AND status = ").push_bind(stat);
        }
        if let Some(start) = start_date {
            query.push(" AND created_at >= ").push_bind(start);
        }
        if let Some(end) = end_date {
            query.push(" AND created_at <= ").push_bind(end);
        }

        query.push(" ORDER BY created_at DESC LIMIT ").push_bind(page_size);
        query.push(" OFFSET ").push_bind(offset);

        let events = query
            .build_query_as::<SecurityEvent>()
            .fetch_all(&*self.pool)
            .await?;

        Ok(events)
    }

    // Mock device status methods (would be implemented with actual device tracking)
    pub async fn get_device_status(&self, tenant_id: &str, device_id: &str) -> SecurityResult<Option<crate::zero_trust::DeviceStatus>> {
        // This would query a device registry table
        // For now, return None to indicate unknown device
        Ok(None)
    }
}