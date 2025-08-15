use chrono::{DateTime, Utc, Duration};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    billing::{BillingService, PaymentResult},
    error::{LicenseError, Result},
    models::*,
    repositories::{LicenseRepository, QuotaRepository, BillingRepository, ComplianceRepository},
};

// Activity request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct ProvisionLicenseRequest {
    pub tenant_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub billing_cycle: BillingCycle,
    pub customer_email: String,
    pub customer_name: String,
    pub payment_method: String, // "stripe", "paypal", "manual"
    pub features: Vec<String>,
    pub custom_quotas: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProvisionLicenseResult {
    pub license_id: Uuid,
    pub license_key: String,
    pub customer_id: Option<String>,
    pub subscription_id: Option<String>,
    pub status: LicenseStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CheckQuotaRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
    pub requested_amount: i64,
    pub operation_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnforceQuotaRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
    pub amount: i64,
    pub operation_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenewLicenseRequest {
    pub license_id: Uuid,
    pub payment_method: Option<String>,
    pub new_billing_cycle: Option<BillingCycle>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RenewLicenseResult {
    pub license_id: Uuid,
    pub new_expires_at: Option<DateTime<Utc>>,
    pub payment_result: Option<PaymentResult>,
    pub invoice_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessPaymentRequest {
    pub tenant_id: Uuid,
    pub amount: Decimal,
    pub currency: String,
    pub payment_method: String,
    pub customer_id: String,
    pub invoice_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateComplianceReportRequest {
    pub tenant_id: Uuid,
    pub report_period_start: DateTime<Utc>,
    pub report_period_end: DateTime<Utc>,
    pub include_recommendations: bool,
}

// License Activities
#[derive(Clone)]
pub struct LicenseActivities {
    license_repo: LicenseRepository,
    quota_repo: QuotaRepository,
    billing_repo: BillingRepository,
    compliance_repo: ComplianceRepository,
    billing_service: BillingService,
}

impl LicenseActivities {
    pub fn new(
        license_repo: LicenseRepository,
        quota_repo: QuotaRepository,
        billing_repo: BillingRepository,
        compliance_repo: ComplianceRepository,
        billing_service: BillingService,
    ) -> Self {
        Self {
            license_repo,
            quota_repo,
            billing_repo,
            compliance_repo,
            billing_service,
        }
    }

    // License provisioning activity
    pub async fn provision_license(&self, request: ProvisionLicenseRequest) -> Result<ProvisionLicenseResult> {
        tracing::info!("Provisioning license for tenant: {}", request.tenant_id);

        // Create customer in payment provider
        let customer_id = if request.payment_method == "stripe" {
            Some(self.billing_service.create_customer(
                request.tenant_id,
                &request.customer_email,
                &request.customer_name,
            ).await?)
        } else {
            None
        };

        // Create license
        let license_request = CreateLicenseRequest {
            tenant_id: request.tenant_id,
            subscription_tier: request.subscription_tier.clone(),
            billing_cycle: request.billing_cycle.clone(),
            base_price: self.get_tier_price(&request.subscription_tier, &request.billing_cycle),
            currency: "USD".to_string(),
            features: request.features,
            custom_quotas: request.custom_quotas,
            auto_renew: true,
        };

        let license = self.license_repo.create(license_request).await?;

        // Initialize tenant quotas based on subscription tier
        self.quota_repo.initialize_tenant_quotas(request.tenant_id, request.subscription_tier).await?;

        // Create subscription if using payment provider
        let subscription_id = if let Some(ref customer_id) = customer_id {
            let price_id = self.get_price_id(&request.subscription_tier, &request.billing_cycle);
            Some(self.billing_service.create_subscription(
                customer_id,
                &price_id,
                request.billing_cycle,
            ).await?)
        } else {
            None
        };

        // Update license with payment provider IDs
        if customer_id.is_some() || subscription_id.is_some() {
            let update_request = UpdateLicenseRequest {
                subscription_tier: None,
                status: Some(LicenseStatus::Active),
                base_price: None,
                expires_at: None,
                auto_renew: None,
                features: None,
                custom_quotas: None,
            };
            self.license_repo.update(license.id, update_request).await?;
        }

        // Log compliance event
        let compliance_log = ComplianceLog {
            id: Uuid::new_v4(),
            tenant_id: request.tenant_id,
            event_type: "license_provisioned".to_string(),
            event_category: "license".to_string(),
            severity: "info".to_string(),
            description: format!("License provisioned for tier: {:?}", request.subscription_tier),
            details: Some(serde_json::json!({
                "license_id": license.id,
                "subscription_tier": request.subscription_tier,
                "billing_cycle": request.billing_cycle,
                "customer_id": customer_id,
                "subscription_id": subscription_id
            })),
            user_id: None,
            resource_id: Some(license.id),
            ip_address: None,
            resolved: true,
            resolved_at: Some(Utc::now()),
            resolved_by: None,
            resolution_notes: None,
            created_at: Utc::now(),
        };
        self.compliance_repo.log_compliance_event(compliance_log).await?;

        Ok(ProvisionLicenseResult {
            license_id: license.id,
            license_key: license.license_key,
            customer_id,
            subscription_id,
            status: LicenseStatus::Active,
        })
    }

    // Quota checking activity
    pub async fn check_quota(&self, request: CheckQuotaRequest) -> Result<QuotaCheckResult> {
        let quota = self.quota_repo.get_tenant_quota(request.tenant_id, &request.quota_name).await?
            .ok_or_else(|| LicenseError::QuotaNotFound { quota_name: request.quota_name.clone() })?;

        let definition = self.quota_repo.get_quota_definition_by_name(&request.quota_name).await?
            .ok_or_else(|| LicenseError::QuotaNotFound { quota_name: request.quota_name.clone() })?;

        let would_exceed = if quota.quota_limit >= 0 {
            quota.current_usage + request.requested_amount > quota.quota_limit
        } else {
            false // Unlimited quota
        };

        let allowed = !would_exceed || !definition.enforce_hard_limit;
        let remaining = quota.remaining();
        let warning_threshold_reached = quota.is_warning_threshold_reached(definition.warning_threshold_percent);

        // Log quota check if it would exceed
        if would_exceed {
            let compliance_log = ComplianceLog {
                id: Uuid::new_v4(),
                tenant_id: request.tenant_id,
                event_type: "quota_check_exceeded".to_string(),
                event_category: "quota".to_string(),
                severity: if allowed { "warning".to_string() } else { "error".to_string() },
                description: format!("Quota check for {} would exceed limit", request.quota_name),
                details: Some(serde_json::json!({
                    "quota_name": request.quota_name,
                    "current_usage": quota.current_usage,
                    "quota_limit": quota.quota_limit,
                    "requested_amount": request.requested_amount,
                    "allowed": allowed
                })),
                user_id: request.user_id,
                resource_id: request.resource_id,
                ip_address: None,
                resolved: false,
                resolved_at: None,
                resolved_by: None,
                resolution_notes: None,
                created_at: Utc::now(),
            };
            self.compliance_repo.log_compliance_event(compliance_log).await?;
        }

        Ok(QuotaCheckResult {
            allowed,
            current_usage: quota.current_usage,
            quota_limit: quota.quota_limit,
            remaining,
            warning_threshold_reached,
            quota_name: request.quota_name,
        })
    }

    // Quota enforcement activity
    pub async fn enforce_quota(&self, request: EnforceQuotaRequest) -> Result<QuotaCheckResult> {
        // First check if the quota allows this usage
        let check_request = CheckQuotaRequest {
            tenant_id: request.tenant_id,
            quota_name: request.quota_name.clone(),
            requested_amount: request.amount,
            operation_type: request.operation_type.clone(),
            resource_id: request.resource_id,
            user_id: request.user_id,
        };

        let check_result = self.check_quota(check_request).await?;

        if !check_result.allowed {
            return Err(LicenseError::QuotaExceeded {
                quota_name: request.quota_name,
                current_usage: check_result.current_usage,
                quota_limit: check_result.quota_limit,
            });
        }

        // Update quota usage
        let updated_quota = self.quota_repo.update_quota_usage(
            request.tenant_id,
            &request.quota_name,
            request.amount,
        ).await?;

        // Log usage
        let usage_request = QuotaUsageRequest {
            tenant_id: request.tenant_id,
            quota_name: request.quota_name.clone(),
            amount: request.amount,
            operation_type: request.operation_type,
            resource_id: request.resource_id,
            user_id: request.user_id,
            metadata: request.metadata,
        };
        self.quota_repo.log_usage(usage_request).await?;

        Ok(QuotaCheckResult {
            allowed: true,
            current_usage: updated_quota.current_usage,
            quota_limit: updated_quota.quota_limit,
            remaining: updated_quota.remaining(),
            warning_threshold_reached: check_result.warning_threshold_reached,
            quota_name: request.quota_name,
        })
    }

    // License renewal activity
    pub async fn renew_license(&self, request: RenewLicenseRequest) -> Result<RenewLicenseResult> {
        let license = self.license_repo.get_by_id(request.license_id).await?
            .ok_or_else(|| LicenseError::LicenseNotFound(request.license_id.to_string()))?;

        // Calculate new expiration date
        let new_expires_at = match request.new_billing_cycle.unwrap_or(license.billing_cycle.clone()) {
            BillingCycle::Monthly => Some(Utc::now() + Duration::days(30)),
            BillingCycle::Yearly => Some(Utc::now() + Duration::days(365)),
            BillingCycle::OneTime => None, // One-time licenses don't expire
            BillingCycle::UsageBased => Some(Utc::now() + Duration::days(30)), // Default to monthly
        };

        // Process payment if required
        let payment_result = if let Some(payment_method) = request.payment_method {
            if let Some(customer_id) = &license.stripe_customer_id {
                Some(self.billing_service.process_payment(
                    license.base_price,
                    &license.currency,
                    customer_id,
                ).await?)
            } else {
                None
            }
        } else {
            None
        };

        // Update license
        let update_request = UpdateLicenseRequest {
            subscription_tier: None,
            status: Some(LicenseStatus::Active),
            base_price: None,
            expires_at: new_expires_at,
            auto_renew: None,
            features: None,
            custom_quotas: None,
        };
        self.license_repo.update(request.license_id, update_request).await?;

        // Log compliance event
        let compliance_log = ComplianceLog {
            id: Uuid::new_v4(),
            tenant_id: license.tenant_id,
            event_type: "license_renewed".to_string(),
            event_category: "license".to_string(),
            severity: "info".to_string(),
            description: "License renewed successfully".to_string(),
            details: Some(serde_json::json!({
                "license_id": request.license_id,
                "new_expires_at": new_expires_at,
                "payment_processed": payment_result.is_some()
            })),
            user_id: None,
            resource_id: Some(request.license_id),
            ip_address: None,
            resolved: true,
            resolved_at: Some(Utc::now()),
            resolved_by: None,
            resolution_notes: None,
            created_at: Utc::now(),
        };
        self.compliance_repo.log_compliance_event(compliance_log).await?;

        Ok(RenewLicenseResult {
            license_id: request.license_id,
            new_expires_at,
            payment_result,
            invoice_id: None, // TODO: Generate invoice
        })
    }

    // Payment processing activity
    pub async fn process_payment(&self, request: ProcessPaymentRequest) -> Result<PaymentResult> {
        let payment_result = self.billing_service.process_payment(
            request.amount,
            &request.currency,
            &request.customer_id,
        ).await?;

        // Log compliance event
        let compliance_log = ComplianceLog {
            id: Uuid::new_v4(),
            tenant_id: request.tenant_id,
            event_type: "payment_processed".to_string(),
            event_category: "billing".to_string(),
            severity: "info".to_string(),
            description: format!("Payment processed: {} {}", request.amount, request.currency),
            details: Some(serde_json::json!({
                "payment_id": payment_result.payment_id,
                "amount": request.amount,
                "currency": request.currency,
                "status": payment_result.status,
                "payment_method": request.payment_method
            })),
            user_id: None,
            resource_id: None,
            ip_address: None,
            resolved: true,
            resolved_at: Some(Utc::now()),
            resolved_by: None,
            resolution_notes: None,
            created_at: Utc::now(),
        };
        self.compliance_repo.log_compliance_event(compliance_log).await?;

        Ok(payment_result)
    }

    // Compliance reporting activity
    pub async fn generate_compliance_report(&self, request: GenerateComplianceReportRequest) -> Result<ComplianceReport> {
        // Get license status
        let license = self.license_repo.get_by_tenant_id(request.tenant_id).await?
            .ok_or_else(|| LicenseError::LicenseNotFound(request.tenant_id.to_string()))?;

        // Get compliance logs for the period
        let compliance_logs = self.compliance_repo.get_compliance_logs(
            request.tenant_id,
            request.report_period_start,
            request.report_period_end,
        ).await?;

        // Analyze quota violations
        let mut quota_violations = Vec::new();
        let mut billing_issues = Vec::new();

        for log in &compliance_logs {
            match log.event_category.as_str() {
                "quota" => {
                    if log.event_type.contains("exceeded") {
                        quota_violations.push(QuotaViolation {
                            quota_name: log.details.as_ref()
                                .and_then(|d| d.get("quota_name"))
                                .and_then(|n| n.as_str())
                                .unwrap_or("unknown")
                                .to_string(),
                            violation_count: 1, // TODO: Aggregate violations
                            last_violation: log.created_at,
                            severity: log.severity.clone(),
                        });
                    }
                }
                "billing" => {
                    if log.event_type.contains("failed") || log.event_type.contains("error") {
                        billing_issues.push(BillingIssue {
                            issue_type: log.event_type.clone(),
                            description: log.description.clone(),
                            amount: log.details.as_ref()
                                .and_then(|d| d.get("amount"))
                                .and_then(|a| a.as_str())
                                .and_then(|s| s.parse().ok()),
                            occurred_at: log.created_at,
                            resolved: log.resolved,
                        });
                    }
                }
                _ => {}
            }
        }

        // Calculate compliance score
        let total_events = compliance_logs.len() as f64;
        let resolved_events = compliance_logs.iter().filter(|log| log.resolved).count() as f64;
        let compliance_score = if total_events > 0.0 {
            (resolved_events / total_events) * 100.0
        } else {
            100.0
        };

        // Generate recommendations
        let mut recommendations = Vec::new();
        if !quota_violations.is_empty() {
            recommendations.push("Consider upgrading subscription tier to increase quotas".to_string());
        }
        if !billing_issues.is_empty() {
            recommendations.push("Review payment methods and billing configuration".to_string());
        }
        if license.is_expired() {
            recommendations.push("Renew license to maintain service access".to_string());
        }

        Ok(ComplianceReport {
            tenant_id: request.tenant_id,
            report_period_start: request.report_period_start,
            report_period_end: request.report_period_end,
            license_status: license.status,
            quota_violations,
            billing_issues,
            compliance_score,
            recommendations,
        })
    }

    // Helper methods
    fn get_tier_price(&self, tier: &SubscriptionTier, cycle: &BillingCycle) -> Decimal {
        use rust_decimal_macros::dec;
        
        match (tier, cycle) {
            (SubscriptionTier::Free, _) => dec!(0.00),
            (SubscriptionTier::Professional, BillingCycle::Monthly) => dec!(29.00),
            (SubscriptionTier::Professional, BillingCycle::Yearly) => dec!(290.00),
            (SubscriptionTier::Enterprise, BillingCycle::Monthly) => dec!(99.00),
            (SubscriptionTier::Enterprise, BillingCycle::Yearly) => dec!(990.00),
            (SubscriptionTier::Custom, _) => dec!(0.00), // Custom pricing
            _ => dec!(0.00),
        }
    }

    fn get_price_id(&self, tier: &SubscriptionTier, cycle: &BillingCycle) -> String {
        match (tier, cycle) {
            (SubscriptionTier::Professional, BillingCycle::Monthly) => "price_professional_monthly".to_string(),
            (SubscriptionTier::Professional, BillingCycle::Yearly) => "price_professional_yearly".to_string(),
            (SubscriptionTier::Enterprise, BillingCycle::Monthly) => "price_enterprise_monthly".to_string(),
            (SubscriptionTier::Enterprise, BillingCycle::Yearly) => "price_enterprise_yearly".to_string(),
            _ => "price_default".to_string(),
        }
    }
}