use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{
    activities::*,
    billing::BillingService,
    error::{LicenseError, Result},
    models::*,
    repositories::{LicenseRepository, QuotaRepository, BillingRepository, ComplianceRepository},
    workflows::*,
};

#[derive(Clone)]
pub struct LicenseService {
    license_repo: LicenseRepository,
    quota_repo: QuotaRepository,
    billing_repo: BillingRepository,
    compliance_repo: ComplianceRepository,
    billing_service: BillingService,
    activities: LicenseActivities,
}

impl LicenseService {
    pub fn new(
        license_repo: LicenseRepository,
        quota_repo: QuotaRepository,
        billing_repo: BillingRepository,
        compliance_repo: ComplianceRepository,
        billing_service: BillingService,
    ) -> Self {
        let activities = LicenseActivities::new(
            license_repo.clone(),
            quota_repo.clone(),
            billing_repo.clone(),
            compliance_repo.clone(),
            billing_service.clone(),
        );

        Self {
            license_repo,
            quota_repo,
            billing_repo,
            compliance_repo,
            billing_service,
            activities,
        }
    }

    // License management methods
    pub async fn create_license(&self, request: CreateLicenseRequest) -> Result<License> {
        self.license_repo.create(request).await
    }

    pub async fn get_license(&self, license_id: Uuid) -> Result<Option<License>> {
        self.license_repo.get_by_id(license_id).await
    }

    pub async fn get_license_by_tenant(&self, tenant_id: Uuid) -> Result<Option<License>> {
        self.license_repo.get_by_tenant_id(tenant_id).await
    }

    pub async fn update_license(&self, license_id: Uuid, request: UpdateLicenseRequest) -> Result<License> {
        self.license_repo.update(license_id, request).await
    }

    pub async fn validate_license(&self, license_key: &str) -> Result<License> {
        let license = self.license_repo.get_by_license_key(license_key).await?
            .ok_or_else(|| LicenseError::InvalidLicenseKey(license_key.to_string()))?;

        if !license.is_active() {
            if license.is_expired() {
                return Err(LicenseError::LicenseExpired { license_id: license.id.to_string() });
            } else {
                return Err(LicenseError::LicenseSuspended { license_id: license.id.to_string() });
            }
        }

        Ok(license)
    }

    pub async fn get_expiring_licenses(&self, days_ahead: i32) -> Result<Vec<License>> {
        self.license_repo.get_expiring_licenses(days_ahead).await
    }

    // Quota management methods
    pub async fn check_quota(&self, tenant_id: Uuid, quota_name: &str, requested_amount: i64) -> Result<QuotaCheckResult> {
        let request = CheckQuotaRequest {
            tenant_id,
            quota_name: quota_name.to_string(),
            requested_amount,
            operation_type: None,
            resource_id: None,
            user_id: None,
        };

        self.activities.check_quota(request).await
    }

    pub async fn enforce_quota(&self, request: QuotaUsageRequest) -> Result<QuotaCheckResult> {
        let enforce_request = EnforceQuotaRequest {
            tenant_id: request.tenant_id,
            quota_name: request.quota_name,
            amount: request.amount,
            operation_type: request.operation_type,
            resource_id: request.resource_id,
            user_id: request.user_id,
            metadata: request.metadata,
        };

        self.activities.enforce_quota(enforce_request).await
    }

    pub async fn get_tenant_quotas(&self, tenant_id: Uuid) -> Result<Vec<TenantQuota>> {
        self.quota_repo.get_tenant_quotas(tenant_id).await
    }

    pub async fn get_quota_usage_summary(&self, tenant_id: Uuid) -> Result<QuotaUsageSummary> {
        let quotas = self.quota_repo.get_tenant_quotas(tenant_id).await?;
        let definitions = self.quota_repo.get_quota_definitions().await?;

        let mut quota_summaries = Vec::new();
        
        for quota in quotas {
            if let Some(definition) = definitions.iter().find(|d| d.id == quota.quota_definition_id) {
                quota_summaries.push(QuotaUsageItem {
                    quota_name: definition.name.clone(),
                    current_usage: quota.current_usage,
                    quota_limit: quota.quota_limit,
                    usage_percentage: quota.usage_percentage(),
                    warning_threshold_reached: quota.is_warning_threshold_reached(definition.warning_threshold_percent),
                    is_exceeded: quota.is_exceeded(),
                    unit: definition.unit.clone(),
                    category: definition.category.clone(),
                });
            }
        }

        Ok(QuotaUsageSummary {
            tenant_id,
            quotas: quota_summaries,
            generated_at: Utc::now(),
        })
    }

    pub async fn reset_quota(&self, tenant_id: Uuid, quota_name: &str) -> Result<()> {
        self.quota_repo.reset_quota_usage(tenant_id, quota_name).await
    }

    // Billing methods
    pub async fn create_billing_record(&self, record: BillingHistory) -> Result<BillingHistory> {
        self.billing_repo.create_billing_record(record).await
    }

    pub async fn get_billing_history(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<BillingHistory>> {
        self.billing_repo.get_billing_history(tenant_id, limit, offset).await
    }

    pub async fn update_payment_status(&self, billing_id: Uuid, status: PaymentStatus, payment_reference: Option<String>) -> Result<()> {
        self.billing_repo.update_payment_status(billing_id, status, payment_reference).await
    }

    pub async fn generate_invoice(&self, tenant_id: Uuid, license_id: Uuid) -> Result<BillingInvoice> {
        // Get license information
        let license = self.license_repo.get_by_id(license_id).await?
            .ok_or_else(|| LicenseError::LicenseNotFound(license_id.to_string()))?;

        // Generate invoice number
        let invoice_number = self.billing_service.generate_invoice_number().await;

        // Create basic subscription invoice
        let line_items = vec![
            BillingLineItem {
                description: format!("Subscription - {:?}", license.subscription_tier),
                quantity: 1,
                unit_price: license.base_price,
                total_price: license.base_price,
                item_type: "subscription".to_string(),
            }
        ];

        let tax_amount = license.base_price * Decimal::from_str("0.08").unwrap_or_default(); // 8% tax

        Ok(BillingInvoice {
            invoice_number,
            tenant_id,
            amount: license.base_price + tax_amount,
            currency: license.currency,
            tax_amount,
            billing_period_start: Utc::now(),
            billing_period_end: Utc::now() + chrono::Duration::days(30),
            line_items,
            usage_summary: None,
        })
    }

    // Compliance methods
    pub async fn log_compliance_event(&self, log: ComplianceLog) -> Result<ComplianceLog> {
        self.compliance_repo.log_compliance_event(log).await
    }

    pub async fn get_compliance_logs(&self, tenant_id: Uuid, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<ComplianceLog>> {
        self.compliance_repo.get_compliance_logs(tenant_id, start_date, end_date).await
    }

    pub async fn generate_compliance_report(&self, tenant_id: Uuid, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<ComplianceReport> {
        let request = GenerateComplianceReportRequest {
            tenant_id,
            report_period_start: start_date,
            report_period_end: end_date,
            include_recommendations: true,
        };

        self.activities.generate_compliance_report(request).await
    }

    pub async fn resolve_compliance_issue(&self, issue_id: Uuid, resolved_by: Uuid, resolution_notes: String) -> Result<()> {
        self.compliance_repo.resolve_compliance_issue(issue_id, resolved_by, resolution_notes).await
    }

    // Workflow initiation methods
    pub async fn initiate_license_provisioning(&self, request: LicenseProvisioningWorkflowRequest) -> Result<String> {
        // In a real implementation, this would start a Temporal workflow
        // For now, we'll return a mock workflow ID
        let workflow_id = format!("license_provisioning_{}", Uuid::new_v4());
        
        tracing::info!("Initiated license provisioning workflow: {}", workflow_id);
        
        // TODO: Start actual Temporal workflow
        // let workflow_handle = temporal_client.start_workflow(
        //     license_provisioning_workflow,
        //     workflow_id.clone(),
        //     request,
        // ).await?;
        
        Ok(workflow_id)
    }

    pub async fn initiate_quota_enforcement(&self, request: QuotaEnforcementWorkflowRequest) -> Result<String> {
        // In a real implementation, this would start a Temporal workflow
        let workflow_id = format!("quota_enforcement_{}", Uuid::new_v4());
        
        tracing::info!("Initiated quota enforcement workflow: {}", workflow_id);
        
        // TODO: Start actual Temporal workflow
        
        Ok(workflow_id)
    }

    pub async fn initiate_license_renewal(&self, request: LicenseRenewalWorkflowRequest) -> Result<String> {
        // In a real implementation, this would start a Temporal workflow
        let workflow_id = format!("license_renewal_{}", Uuid::new_v4());
        
        tracing::info!("Initiated license renewal workflow: {}", workflow_id);
        
        // TODO: Start actual Temporal workflow
        
        Ok(workflow_id)
    }

    // Monitoring and analytics methods
    pub async fn get_license_analytics(&self, tenant_id: Uuid) -> Result<LicenseAnalytics> {
        let license = self.license_repo.get_by_tenant_id(tenant_id).await?
            .ok_or_else(|| LicenseError::LicenseNotFound(tenant_id.to_string()))?;

        let quotas = self.get_quota_usage_summary(tenant_id).await?;
        let billing_history = self.billing_repo.get_billing_history(tenant_id, 12, 0).await?; // Last 12 records

        let total_spent = billing_history.iter()
            .filter(|b| matches!(b.payment_status, PaymentStatus::Completed))
            .map(|b| b.amount)
            .sum();

        let quota_violations = quotas.quotas.iter()
            .filter(|q| q.is_exceeded)
            .count();

        Ok(LicenseAnalytics {
            tenant_id,
            license_status: license.status,
            subscription_tier: license.subscription_tier,
            days_until_expiry: license.days_until_expiry(),
            total_spent,
            quota_violations: quota_violations as i64,
            compliance_score: self.calculate_compliance_score(tenant_id).await?,
            last_payment_date: billing_history.first().and_then(|b| b.paid_at),
            auto_renew_enabled: license.auto_renew,
        })
    }

    async fn calculate_compliance_score(&self, tenant_id: Uuid) -> Result<f64> {
        let thirty_days_ago = Utc::now() - chrono::Duration::days(30);
        let logs = self.compliance_repo.get_compliance_logs(tenant_id, thirty_days_ago, Utc::now()).await?;

        if logs.is_empty() {
            return Ok(100.0);
        }

        let total_events = logs.len() as f64;
        let resolved_events = logs.iter().filter(|log| log.resolved).count() as f64;
        let critical_events = logs.iter().filter(|log| log.severity == "critical").count() as f64;
        let error_events = logs.iter().filter(|log| log.severity == "error").count() as f64;

        // Calculate score based on resolution rate and severity
        let resolution_score = (resolved_events / total_events) * 100.0;
        let severity_penalty = (critical_events * 10.0 + error_events * 5.0) / total_events;
        
        let final_score = (resolution_score - severity_penalty).max(0.0).min(100.0);
        
        Ok(final_score)
    }
}

// Additional DTOs
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuotaUsageSummary {
    pub tenant_id: Uuid,
    pub quotas: Vec<QuotaUsageItem>,
    pub generated_at: DateTime<Utc>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct QuotaUsageItem {
    pub quota_name: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub usage_percentage: f64,
    pub warning_threshold_reached: bool,
    pub is_exceeded: bool,
    pub unit: String,
    pub category: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct LicenseAnalytics {
    pub tenant_id: Uuid,
    pub license_status: LicenseStatus,
    pub subscription_tier: SubscriptionTier,
    pub days_until_expiry: Option<i64>,
    pub total_spent: Decimal,
    pub quota_violations: i64,
    pub compliance_score: f64,
    pub last_payment_date: Option<DateTime<Utc>>,
    pub auto_renew_enabled: bool,
}

// Helper trait for decimal parsing
trait DecimalFromStr {
    fn from_str(s: &str) -> Result<Decimal, rust_decimal::Error>;
}

impl DecimalFromStr for Decimal {
    fn from_str(s: &str) -> Result<Decimal, rust_decimal::Error> {
        s.parse()
    }
}