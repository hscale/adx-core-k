use crate::{
    error::{SecurityError, SecurityResult},
    models::{
        DataRetentionPolicy, DataRetentionJob, DeletionMethod, RetentionJobStatus,
        DataRetentionSummary
    },
    repositories::RetentionRepository,
    audit::AuditService,
};
use chrono::{DateTime, Utc, Duration};
use serde_json::Value;
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct DataRetentionService {
    repository: Arc<RetentionRepository>,
    audit_service: Arc<AuditService>,
}

impl DataRetentionService {
    pub fn new(
        repository: Arc<RetentionRepository>,
        audit_service: Arc<AuditService>,
    ) -> Self {
        Self {
            repository,
            audit_service,
        }
    }

    /// Create a new data retention policy
    pub async fn create_policy(
        &self,
        tenant_id: &str,
        resource_type: &str,
        retention_period_days: i32,
        deletion_method: DeletionMethod,
        created_by: &str,
    ) -> SecurityResult<DataRetentionPolicy> {
        // Validate inputs
        self.validate_policy_inputs(tenant_id, resource_type, retention_period_days)?;

        // Check for existing policy
        if let Some(_existing) = self.repository.get_policy_by_resource(tenant_id, resource_type).await? {
            return Err(SecurityError::Conflict(
                format!("Retention policy already exists for resource type: {}", resource_type)
            ));
        }

        let policy = DataRetentionPolicy {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            resource_type: resource_type.to_string(),
            retention_period_days,
            deletion_method,
            enabled: true,
            created_by: created_by.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_policy = self.repository.create_policy(policy).await?;

        // Log policy creation
        self.audit_service.log_compliance_event(
            tenant_id,
            "DATA_RETENTION",
            "policy_created",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "policy_id": created_policy.id,
                "resource_type": resource_type,
                "retention_period_days": retention_period_days,
                "deletion_method": deletion_method,
                "created_by": created_by
            }),
        ).await?;

        Ok(created_policy)
    }

    /// Update an existing data retention policy
    pub async fn update_policy(
        &self,
        policy_id: Uuid,
        retention_period_days: Option<i32>,
        deletion_method: Option<DeletionMethod>,
        enabled: Option<bool>,
    ) -> SecurityResult<DataRetentionPolicy> {
        let mut policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| SecurityError::NotFound("Retention policy not found".to_string()))?;

        let original_policy = policy.clone();

        // Update fields
        if let Some(period) = retention_period_days {
            if period < 1 {
                return Err(SecurityError::Validation("Retention period must be at least 1 day".to_string()));
            }
            policy.retention_period_days = period;
        }

        if let Some(method) = deletion_method {
            policy.deletion_method = method;
        }

        if let Some(enabled_flag) = enabled {
            policy.enabled = enabled_flag;
        }

        policy.updated_at = Utc::now();

        let updated_policy = self.repository.update_policy(policy).await?;

        // Log policy update
        self.audit_service.log_compliance_event(
            &updated_policy.tenant_id,
            "DATA_RETENTION",
            "policy_updated",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "policy_id": policy_id,
                "original": original_policy,
                "updated": updated_policy
            }),
        ).await?;

        Ok(updated_policy)
    }

    /// Delete a data retention policy
    pub async fn delete_policy(&self, policy_id: Uuid) -> SecurityResult<()> {
        let policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| SecurityError::NotFound("Retention policy not found".to_string()))?;

        // Check for pending jobs
        let pending_jobs = self.repository.get_pending_jobs_for_policy(policy_id).await?;
        if !pending_jobs.is_empty() {
            return Err(SecurityError::Conflict(
                "Cannot delete policy with pending retention jobs".to_string()
            ));
        }

        self.repository.delete_policy(policy_id).await?;

        // Log policy deletion
        self.audit_service.log_compliance_event(
            &policy.tenant_id,
            "DATA_RETENTION",
            "policy_deleted",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "policy_id": policy_id,
                "resource_type": policy.resource_type
            }),
        ).await?;

        Ok(())
    }

    /// Get all retention policies for a tenant
    pub async fn get_tenant_policies(&self, tenant_id: &str) -> SecurityResult<Vec<DataRetentionPolicy>> {
        self.repository.get_tenant_policies(tenant_id).await
    }

    /// Get retention policy by resource type
    pub async fn get_policy_by_resource(
        &self,
        tenant_id: &str,
        resource_type: &str,
    ) -> SecurityResult<Option<DataRetentionPolicy>> {
        self.repository.get_policy_by_resource(tenant_id, resource_type).await
    }

    /// Schedule retention jobs for all active policies
    pub async fn schedule_retention_jobs(&self) -> SecurityResult<Vec<Uuid>> {
        let active_policies = self.repository.get_active_policies().await?;
        let mut scheduled_jobs = Vec::new();

        for policy in active_policies {
            match self.schedule_job_for_policy(&policy).await {
                Ok(job_id) => {
                    scheduled_jobs.push(job_id);
                    info!(
                        policy_id = %policy.id,
                        job_id = %job_id,
                        "Scheduled retention job"
                    );
                }
                Err(e) => {
                    error!(
                        policy_id = %policy.id,
                        error = %e,
                        "Failed to schedule retention job"
                    );
                }
            }
        }

        Ok(scheduled_jobs)
    }

    /// Execute pending retention jobs
    pub async fn execute_pending_jobs(&self) -> SecurityResult<Vec<Uuid>> {
        let pending_jobs = self.repository.get_pending_jobs().await?;
        let mut executed_jobs = Vec::new();

        for job in pending_jobs {
            if job.scheduled_for <= Utc::now() {
                match self.execute_retention_job(job).await {
                    Ok(_) => {
                        executed_jobs.push(job.id);
                    }
                    Err(e) => {
                        error!(
                            job_id = %job.id,
                            error = %e,
                            "Failed to execute retention job"
                        );
                        
                        // Mark job as failed
                        let mut failed_job = job;
                        failed_job.status = RetentionJobStatus::Failed;
                        failed_job.error_message = Some(e.to_string());
                        
                        if let Err(update_err) = self.repository.update_job(failed_job).await {
                            error!(error = %update_err, "Failed to update failed job status");
                        }
                    }
                }
            }
        }

        Ok(executed_jobs)
    }

    /// Get retention job status
    pub async fn get_job_status(&self, job_id: Uuid) -> SecurityResult<DataRetentionJob> {
        self.repository.get_job(job_id).await?
            .ok_or_else(|| SecurityError::NotFound("Retention job not found".to_string()))
    }

    /// Get retention summary for a tenant
    pub async fn get_retention_summary(&self, tenant_id: &str) -> SecurityResult<DataRetentionSummary> {
        let policies = self.repository.get_tenant_policies(tenant_id).await?;
        let scheduled_jobs = self.repository.get_tenant_scheduled_jobs(tenant_id).await?;
        let records_to_delete = self.repository.count_records_to_delete(tenant_id).await?;
        let next_cleanup = self.repository.get_next_cleanup_time(tenant_id).await?;

        let total_policies = policies.len() as i32;
        let active_policies = policies.iter().filter(|p| p.enabled).count() as i32;
        let scheduled_jobs_count = scheduled_jobs.len() as i32;

        Ok(DataRetentionSummary {
            total_policies,
            active_policies,
            scheduled_jobs: scheduled_jobs_count,
            records_to_delete,
            next_cleanup,
        })
    }

    /// Cancel a scheduled retention job
    pub async fn cancel_job(&self, job_id: Uuid) -> SecurityResult<()> {
        let mut job = self.repository.get_job(job_id).await?
            .ok_or_else(|| SecurityError::NotFound("Retention job not found".to_string()))?;

        if job.status != RetentionJobStatus::Scheduled {
            return Err(SecurityError::Validation("Can only cancel scheduled jobs".to_string()));
        }

        job.status = RetentionJobStatus::Cancelled;
        self.repository.update_job(job.clone()).await?;

        // Log job cancellation
        self.audit_service.log_compliance_event(
            &job.tenant_id,
            "DATA_RETENTION",
            "job_cancelled",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "job_id": job_id,
                "policy_id": job.policy_id,
                "resource_type": job.resource_type
            }),
        ).await?;

        Ok(())
    }

    /// Apply retention policy to specific records
    pub async fn apply_policy_to_records(
        &self,
        tenant_id: &str,
        resource_type: &str,
        record_ids: Vec<String>,
    ) -> SecurityResult<i64> {
        let policy = self.repository.get_policy_by_resource(tenant_id, resource_type).await?
            .ok_or_else(|| SecurityError::NotFound("No retention policy found for resource type".to_string()))?;

        if !policy.enabled {
            return Err(SecurityError::Validation("Retention policy is disabled".to_string()));
        }

        let cutoff_date = Utc::now() - Duration::days(policy.retention_period_days as i64);
        let processed_count = self.process_records_for_deletion(
            tenant_id,
            resource_type,
            &record_ids,
            cutoff_date,
            &policy.deletion_method,
        ).await?;

        // Log the operation
        self.audit_service.log_compliance_event(
            tenant_id,
            "DATA_RETENTION",
            "policy_applied",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "policy_id": policy.id,
                "resource_type": resource_type,
                "records_processed": processed_count,
                "deletion_method": policy.deletion_method,
                "cutoff_date": cutoff_date
            }),
        ).await?;

        Ok(processed_count)
    }

    // Private helper methods

    fn validate_policy_inputs(
        &self,
        tenant_id: &str,
        resource_type: &str,
        retention_period_days: i32,
    ) -> SecurityResult<()> {
        if tenant_id.is_empty() {
            return Err(SecurityError::Validation("Tenant ID is required".to_string()));
        }
        if resource_type.is_empty() {
            return Err(SecurityError::Validation("Resource type is required".to_string()));
        }
        if retention_period_days < 1 {
            return Err(SecurityError::Validation("Retention period must be at least 1 day".to_string()));
        }
        if retention_period_days > 36500 { // 100 years
            return Err(SecurityError::Validation("Retention period cannot exceed 100 years".to_string()));
        }
        Ok(())
    }

    async fn schedule_job_for_policy(&self, policy: &DataRetentionPolicy) -> SecurityResult<Uuid> {
        // Check if there's already a scheduled job for this policy
        let existing_jobs = self.repository.get_pending_jobs_for_policy(policy.id).await?;
        if !existing_jobs.is_empty() {
            return Ok(existing_jobs[0].id);
        }

        // Calculate when the job should run (next day at midnight)
        let scheduled_for = (Utc::now() + Duration::days(1))
            .date_naive()
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        let job = DataRetentionJob {
            id: Uuid::new_v4(),
            tenant_id: policy.tenant_id.clone(),
            policy_id: policy.id,
            resource_type: policy.resource_type.clone(),
            scheduled_for,
            status: RetentionJobStatus::Scheduled,
            records_processed: 0,
            records_deleted: 0,
            error_message: None,
            started_at: None,
            completed_at: None,
            created_at: Utc::now(),
        };

        let created_job = self.repository.create_job(job).await?;
        Ok(created_job.id)
    }

    async fn execute_retention_job(&self, mut job: DataRetentionJob) -> SecurityResult<()> {
        // Mark job as running
        job.status = RetentionJobStatus::Running;
        job.started_at = Some(Utc::now());
        self.repository.update_job(job.clone()).await?;

        // Get the policy
        let policy = self.repository.get_policy(job.policy_id).await?
            .ok_or_else(|| SecurityError::DataRetention("Policy not found for job".to_string()))?;

        // Calculate cutoff date
        let cutoff_date = Utc::now() - Duration::days(policy.retention_period_days as i64);

        // Process records for deletion
        let (processed_count, deleted_count) = self.execute_deletion_for_resource(
            &job.tenant_id,
            &job.resource_type,
            cutoff_date,
            &policy.deletion_method,
        ).await?;

        // Update job with results
        job.status = RetentionJobStatus::Completed;
        job.records_processed = processed_count;
        job.records_deleted = deleted_count;
        job.completed_at = Some(Utc::now());
        self.repository.update_job(job.clone()).await?;

        // Log job completion
        self.audit_service.log_compliance_event(
            &job.tenant_id,
            "DATA_RETENTION",
            "job_completed",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "job_id": job.id,
                "policy_id": job.policy_id,
                "resource_type": job.resource_type,
                "records_processed": processed_count,
                "records_deleted": deleted_count,
                "cutoff_date": cutoff_date,
                "deletion_method": policy.deletion_method
            }),
        ).await?;

        Ok(())
    }

    async fn execute_deletion_for_resource(
        &self,
        tenant_id: &str,
        resource_type: &str,
        cutoff_date: DateTime<Utc>,
        deletion_method: &DeletionMethod,
    ) -> SecurityResult<(i64, i64)> {
        // This would integrate with the specific service to perform the deletion
        // For now, we'll simulate the process
        
        info!(
            tenant_id = %tenant_id,
            resource_type = %resource_type,
            cutoff_date = %cutoff_date,
            deletion_method = ?deletion_method,
            "Executing retention deletion"
        );

        // Simulate finding records to delete
        let records_to_process = self.find_records_for_deletion(tenant_id, resource_type, cutoff_date).await?;
        let processed_count = records_to_process.len() as i64;

        // Simulate deletion based on method
        let deleted_count = match deletion_method {
            DeletionMethod::SoftDelete => {
                self.soft_delete_records(tenant_id, resource_type, &records_to_process).await?
            }
            DeletionMethod::HardDelete => {
                self.hard_delete_records(tenant_id, resource_type, &records_to_process).await?
            }
            DeletionMethod::Anonymize => {
                self.anonymize_records(tenant_id, resource_type, &records_to_process).await?
            }
            DeletionMethod::Archive => {
                self.archive_records(tenant_id, resource_type, &records_to_process).await?
            }
        };

        Ok((processed_count, deleted_count))
    }

    async fn process_records_for_deletion(
        &self,
        tenant_id: &str,
        resource_type: &str,
        record_ids: &[String],
        cutoff_date: DateTime<Utc>,
        deletion_method: &DeletionMethod,
    ) -> SecurityResult<i64> {
        // Filter records that are older than cutoff date
        let eligible_records = self.filter_eligible_records(tenant_id, resource_type, record_ids, cutoff_date).await?;
        
        match deletion_method {
            DeletionMethod::SoftDelete => {
                self.soft_delete_records(tenant_id, resource_type, &eligible_records).await
            }
            DeletionMethod::HardDelete => {
                self.hard_delete_records(tenant_id, resource_type, &eligible_records).await
            }
            DeletionMethod::Anonymize => {
                self.anonymize_records(tenant_id, resource_type, &eligible_records).await
            }
            DeletionMethod::Archive => {
                self.archive_records(tenant_id, resource_type, &eligible_records).await
            }
        }
    }

    async fn find_records_for_deletion(
        &self,
        tenant_id: &str,
        resource_type: &str,
        cutoff_date: DateTime<Utc>,
    ) -> SecurityResult<Vec<String>> {
        // This would query the specific service to find records older than cutoff_date
        // For now, we'll return a mock list
        info!(
            tenant_id = %tenant_id,
            resource_type = %resource_type,
            cutoff_date = %cutoff_date,
            "Finding records for deletion"
        );
        
        // Simulate finding some records
        Ok(vec!["record1".to_string(), "record2".to_string(), "record3".to_string()])
    }

    async fn filter_eligible_records(
        &self,
        tenant_id: &str,
        resource_type: &str,
        record_ids: &[String],
        cutoff_date: DateTime<Utc>,
    ) -> SecurityResult<Vec<String>> {
        // This would check which records are actually eligible for deletion
        // For now, we'll return all provided records
        info!(
            tenant_id = %tenant_id,
            resource_type = %resource_type,
            record_count = %record_ids.len(),
            cutoff_date = %cutoff_date,
            "Filtering eligible records"
        );
        
        Ok(record_ids.to_vec())
    }

    async fn soft_delete_records(
        &self,
        tenant_id: &str,
        resource_type: &str,
        record_ids: &[String],
    ) -> SecurityResult<i64> {
        info!(
            tenant_id = %tenant_id,
            resource_type = %resource_type,
            record_count = %record_ids.len(),
            "Soft deleting records"
        );
        
        // This would mark records as deleted without actually removing them
        Ok(record_ids.len() as i64)
    }

    async fn hard_delete_records(
        &self,
        tenant_id: &str,
        resource_type: &str,
        record_ids: &[String],
    ) -> SecurityResult<i64> {
        info!(
            tenant_id = %tenant_id,
            resource_type = %resource_type,
            record_count = %record_ids.len(),
            "Hard deleting records"
        );
        
        // This would permanently remove records
        Ok(record_ids.len() as i64)
    }

    async fn anonymize_records(
        &self,
        tenant_id: &str,
        resource_type: &str,
        record_ids: &[String],
    ) -> SecurityResult<i64> {
        info!(
            tenant_id = %tenant_id,
            resource_type = %resource_type,
            record_count = %record_ids.len(),
            "Anonymizing records"
        );
        
        // This would remove or hash personally identifiable information
        Ok(record_ids.len() as i64)
    }

    async fn archive_records(
        &self,
        tenant_id: &str,
        resource_type: &str,
        record_ids: &[String],
    ) -> SecurityResult<i64> {
        info!(
            tenant_id = %tenant_id,
            resource_type = %resource_type,
            record_count = %record_ids.len(),
            "Archiving records"
        );
        
        // This would move records to long-term storage
        Ok(record_ids.len() as i64)
    }
}