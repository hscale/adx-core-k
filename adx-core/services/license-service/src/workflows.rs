use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    activities::*,
    error::{LicenseError, Result},
    models::*,
};

// Workflow request/response types
#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseProvisioningWorkflowRequest {
    pub tenant_id: Uuid,
    pub subscription_tier: SubscriptionTier,
    pub billing_cycle: BillingCycle,
    pub customer_email: String,
    pub customer_name: String,
    pub payment_method: String,
    pub features: Vec<String>,
    pub custom_quotas: Option<serde_json::Value>,
    pub setup_billing: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseProvisioningWorkflowResult {
    pub license_id: Uuid,
    pub license_key: String,
    pub customer_id: Option<String>,
    pub subscription_id: Option<String>,
    pub quotas_initialized: bool,
    pub billing_setup: bool,
    pub status: LicenseStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaEnforcementWorkflowRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
    pub requested_amount: i64,
    pub operation_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
    pub real_time_monitoring: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuotaEnforcementWorkflowResult {
    pub allowed: bool,
    pub quota_enforced: bool,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub warning_sent: bool,
    pub compliance_logged: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseRenewalWorkflowRequest {
    pub license_id: Uuid,
    pub payment_method: Option<String>,
    pub new_billing_cycle: Option<BillingCycle>,
    pub auto_renewal: bool,
    pub send_notifications: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LicenseRenewalWorkflowResult {
    pub license_id: Uuid,
    pub renewal_successful: bool,
    pub new_expires_at: Option<DateTime<Utc>>,
    pub payment_processed: bool,
    pub invoice_generated: bool,
    pub notifications_sent: bool,
}

// Workflow implementations using shared temporal abstractions
use adx_shared::{WorkflowContext, ActivityContext, WorkflowError, ActivityError};

/// License Provisioning Workflow
/// 
/// This workflow handles the complete license provisioning process including:
/// - Customer creation in payment providers
/// - License creation and activation
/// - Quota initialization
/// - Billing setup
/// - Compliance logging
pub async fn license_provisioning_workflow(
    request: LicenseProvisioningWorkflowRequest,
    _context: WorkflowContext,
) -> Result<LicenseProvisioningWorkflowResult> {
    tracing::info!("Starting license provisioning workflow for tenant: {}", request.tenant_id);

    // Step 1: Provision the license
    let provision_request = ProvisionLicenseRequest {
        tenant_id: request.tenant_id,
        subscription_tier: request.subscription_tier.clone(),
        billing_cycle: request.billing_cycle.clone(),
        customer_email: request.customer_email.clone(),
        customer_name: request.customer_name.clone(),
        payment_method: request.payment_method.clone(),
        features: request.features.clone(),
        custom_quotas: request.custom_quotas.clone(),
    };

    // Execute provision license activity
    let provision_result = execute_activity(
        "provision_license",
        provision_request,
        ActivityContext::default(),
    ).await.map_err(|e| LicenseError::WorkflowError(e))?;

    // Step 2: Set up billing if requested
    let billing_setup = if request.setup_billing && provision_result.customer_id.is_some() {
        // Create initial invoice for setup
        let invoice_request = ProcessPaymentRequest {
            tenant_id: request.tenant_id,
            amount: get_setup_fee(&request.subscription_tier),
            currency: "USD".to_string(),
            payment_method: request.payment_method.clone(),
            customer_id: provision_result.customer_id.clone().unwrap(),
            invoice_id: None,
        };

        match execute_activity(
            "process_payment",
            invoice_request,
            ActivityContext::default(),
        ).await {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Billing setup failed: {:?}", e);
                false
            }
        }
    } else {
        true // No billing setup required
    };

    // Step 3: Send welcome notification (if billing setup successful)
    if billing_setup {
        let notification_request = SendWelcomeNotificationRequest {
            tenant_id: request.tenant_id,
            customer_email: request.customer_email,
            license_key: provision_result.license_key.clone(),
            subscription_tier: request.subscription_tier.clone(),
        };

        // Execute notification activity (non-critical)
        let _ = execute_activity(
            "send_welcome_notification",
            notification_request,
            ActivityContext::default(),
        ).await;
    }

    Ok(LicenseProvisioningWorkflowResult {
        license_id: provision_result.license_id,
        license_key: provision_result.license_key,
        customer_id: provision_result.customer_id,
        subscription_id: provision_result.subscription_id,
        quotas_initialized: true,
        billing_setup,
        status: provision_result.status,
    })
}

/// Quota Enforcement Workflow
/// 
/// This workflow handles real-time quota enforcement including:
/// - Quota checking and validation
/// - Usage tracking and logging
/// - Warning notifications
/// - Compliance monitoring
pub async fn quota_enforcement_workflow(
    request: QuotaEnforcementWorkflowRequest,
    _context: WorkflowContext,
) -> Result<QuotaEnforcementWorkflowResult> {
    tracing::info!("Starting quota enforcement workflow for tenant: {} quota: {}", 
        request.tenant_id, request.quota_name);

    // Step 1: Check quota availability
    let check_request = CheckQuotaRequest {
        tenant_id: request.tenant_id,
        quota_name: request.quota_name.clone(),
        requested_amount: request.requested_amount,
        operation_type: request.operation_type.clone(),
        resource_id: request.resource_id,
        user_id: request.user_id,
    };

    let check_result = execute_activity(
        "check_quota",
        check_request,
        ActivityContext::default(),
    ).await.map_err(|e| LicenseError::WorkflowError(e))?;

    if !check_result.allowed {
        // Quota exceeded - log compliance event and return
        let compliance_request = LogComplianceEventRequest {
            tenant_id: request.tenant_id,
            event_type: "quota_exceeded".to_string(),
            event_category: "quota".to_string(),
            severity: "error".to_string(),
            description: format!("Quota exceeded for {}", request.quota_name),
            details: Some(serde_json::json!({
                "quota_name": request.quota_name,
                "requested_amount": request.requested_amount,
                "current_usage": check_result.current_usage,
                "quota_limit": check_result.quota_limit
            })),
            user_id: request.user_id,
            resource_id: request.resource_id,
        };

        let _ = execute_activity(
            "log_compliance_event",
            compliance_request,
            ActivityContext::default(),
        ).await;

        return Ok(QuotaEnforcementWorkflowResult {
            allowed: false,
            quota_enforced: false,
            current_usage: check_result.current_usage,
            quota_limit: check_result.quota_limit,
            warning_sent: false,
            compliance_logged: true,
        });
    }

    // Step 2: Enforce quota (update usage)
    let enforce_request = EnforceQuotaRequest {
        tenant_id: request.tenant_id,
        quota_name: request.quota_name.clone(),
        amount: request.requested_amount,
        operation_type: request.operation_type.clone(),
        resource_id: request.resource_id,
        user_id: request.user_id,
        metadata: request.metadata.clone(),
    };

    let enforce_result = execute_activity(
        "enforce_quota",
        enforce_request,
        ActivityContext::default(),
    ).await.map_err(|e| LicenseError::WorkflowError(e))?;

    // Step 3: Send warning notification if threshold reached
    let warning_sent = if enforce_result.warning_threshold_reached {
        let warning_request = SendQuotaWarningRequest {
            tenant_id: request.tenant_id,
            quota_name: request.quota_name.clone(),
            current_usage: enforce_result.current_usage,
            quota_limit: enforce_result.quota_limit,
            usage_percentage: (enforce_result.current_usage as f64 / enforce_result.quota_limit as f64) * 100.0,
        };

        match execute_activity(
            "send_quota_warning",
            warning_request,
            ActivityContext::default(),
        ).await {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Failed to send quota warning: {:?}", e);
                false
            }
        }
    } else {
        false
    };

    // Step 4: Real-time monitoring update (if enabled)
    if request.real_time_monitoring {
        let monitoring_request = UpdateQuotaMonitoringRequest {
            tenant_id: request.tenant_id,
            quota_name: request.quota_name.clone(),
            current_usage: enforce_result.current_usage,
            quota_limit: enforce_result.quota_limit,
            timestamp: Utc::now(),
        };

        let _ = execute_activity(
            "update_quota_monitoring",
            monitoring_request,
            ActivityContext::default(),
        ).await;
    }

    Ok(QuotaEnforcementWorkflowResult {
        allowed: true,
        quota_enforced: true,
        current_usage: enforce_result.current_usage,
        quota_limit: enforce_result.quota_limit,
        warning_sent,
        compliance_logged: false,
    })
}

/// License Renewal Workflow
/// 
/// This workflow handles license renewal including:
/// - Payment processing
/// - License extension
/// - Invoice generation
/// - Notification sending
pub async fn license_renewal_workflow(
    request: LicenseRenewalWorkflowRequest,
    _context: WorkflowContext,
) -> Result<LicenseRenewalWorkflowResult> {
    tracing::info!("Starting license renewal workflow for license: {}", request.license_id);

    // Step 1: Validate license and get current state
    let license_info = execute_activity(
        "get_license_info",
        GetLicenseInfoRequest {
            license_id: request.license_id,
        },
        ActivityContext::default(),
    ).await.map_err(|e| LicenseError::WorkflowError(e))?;

    // Step 2: Process payment if required
    let payment_processed = if let Some(payment_method) = &request.payment_method {
        if let Some(customer_id) = &license_info.customer_id {
            let payment_request = ProcessPaymentRequest {
                tenant_id: license_info.tenant_id,
                amount: license_info.renewal_amount,
                currency: license_info.currency.clone(),
                payment_method: payment_method.clone(),
                customer_id: customer_id.clone(),
                invoice_id: None,
            };

            match execute_activity(
                "process_payment",
                payment_request,
                ActivityContext::default(),
            ).await {
                Ok(_) => true,
                Err(e) => {
                    tracing::error!("Payment processing failed: {:?}", e);
                    
                    // Log payment failure
                    let compliance_request = LogComplianceEventRequest {
                        tenant_id: license_info.tenant_id,
                        event_type: "payment_failed".to_string(),
                        event_category: "billing".to_string(),
                        severity: "error".to_string(),
                        description: "License renewal payment failed".to_string(),
                        details: Some(serde_json::json!({
                            "license_id": request.license_id,
                            "payment_method": payment_method,
                            "amount": license_info.renewal_amount,
                            "error": format!("{:?}", e)
                        })),
                        user_id: None,
                        resource_id: Some(request.license_id),
                    };

                    let _ = execute_activity(
                        "log_compliance_event",
                        compliance_request,
                        ActivityContext::default(),
                    ).await;

                    return Ok(LicenseRenewalWorkflowResult {
                        license_id: request.license_id,
                        renewal_successful: false,
                        new_expires_at: None,
                        payment_processed: false,
                        invoice_generated: false,
                        notifications_sent: false,
                    });
                }
            }
        } else {
            false
        }
    } else {
        true // No payment required
    };

    // Step 3: Renew license if payment successful
    let renewal_result = if payment_processed {
        let renew_request = RenewLicenseRequest {
            license_id: request.license_id,
            payment_method: request.payment_method.clone(),
            new_billing_cycle: request.new_billing_cycle.clone(),
        };

        execute_activity(
            "renew_license",
            renew_request,
            ActivityContext::default(),
        ).await.map_err(|e| LicenseError::WorkflowError(e))?
    } else {
        return Ok(LicenseRenewalWorkflowResult {
            license_id: request.license_id,
            renewal_successful: false,
            new_expires_at: None,
            payment_processed: false,
            invoice_generated: false,
            notifications_sent: false,
        });
    };

    // Step 4: Generate invoice
    let invoice_generated = if payment_processed {
        let invoice_request = GenerateInvoiceRequest {
            tenant_id: license_info.tenant_id,
            license_id: request.license_id,
            amount: license_info.renewal_amount,
            currency: license_info.currency.clone(),
            billing_period_start: Utc::now(),
            billing_period_end: renewal_result.new_expires_at.unwrap_or_else(|| Utc::now() + Duration::days(30)),
            line_items: vec![BillingLineItem {
                description: format!("License Renewal - {:?}", license_info.subscription_tier),
                quantity: 1,
                unit_price: license_info.renewal_amount,
                total_price: license_info.renewal_amount,
                item_type: "subscription".to_string(),
            }],
        };

        match execute_activity(
            "generate_invoice",
            invoice_request,
            ActivityContext::default(),
        ).await {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Invoice generation failed: {:?}", e);
                false
            }
        }
    } else {
        false
    };

    // Step 5: Send notifications if requested
    let notifications_sent = if request.send_notifications && renewal_result.new_expires_at.is_some() {
        let notification_request = SendRenewalNotificationRequest {
            tenant_id: license_info.tenant_id,
            license_id: request.license_id,
            customer_email: license_info.customer_email,
            new_expires_at: renewal_result.new_expires_at.unwrap(),
            amount_paid: if payment_processed { Some(license_info.renewal_amount) } else { None },
        };

        match execute_activity(
            "send_renewal_notification",
            notification_request,
            ActivityContext::default(),
        ).await {
            Ok(_) => true,
            Err(e) => {
                tracing::warn!("Notification sending failed: {:?}", e);
                false
            }
        }
    } else {
        false
    };

    Ok(LicenseRenewalWorkflowResult {
        license_id: request.license_id,
        renewal_successful: renewal_result.new_expires_at.is_some(),
        new_expires_at: renewal_result.new_expires_at,
        payment_processed,
        invoice_generated,
        notifications_sent,
    })
}

// Helper functions and additional request types
#[derive(Debug, Serialize, Deserialize)]
pub struct SendWelcomeNotificationRequest {
    pub tenant_id: Uuid,
    pub customer_email: String,
    pub license_key: String,
    pub subscription_tier: SubscriptionTier,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendQuotaWarningRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub usage_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateQuotaMonitoringRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
    pub current_usage: i64,
    pub quota_limit: i64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LogComplianceEventRequest {
    pub tenant_id: Uuid,
    pub event_type: String,
    pub event_category: String,
    pub severity: String,
    pub description: String,
    pub details: Option<serde_json::Value>,
    pub user_id: Option<Uuid>,
    pub resource_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLicenseInfoRequest {
    pub license_id: Uuid,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetLicenseInfoResult {
    pub tenant_id: Uuid,
    pub customer_id: Option<String>,
    pub customer_email: String,
    pub subscription_tier: SubscriptionTier,
    pub renewal_amount: rust_decimal::Decimal,
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerateInvoiceRequest {
    pub tenant_id: Uuid,
    pub license_id: Uuid,
    pub amount: rust_decimal::Decimal,
    pub currency: String,
    pub billing_period_start: DateTime<Utc>,
    pub billing_period_end: DateTime<Utc>,
    pub line_items: Vec<BillingLineItem>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SendRenewalNotificationRequest {
    pub tenant_id: Uuid,
    pub license_id: Uuid,
    pub customer_email: String,
    pub new_expires_at: DateTime<Utc>,
    pub amount_paid: Option<rust_decimal::Decimal>,
}

// Helper functions
fn get_setup_fee(tier: &SubscriptionTier) -> rust_decimal::Decimal {
    use rust_decimal_macros::dec;
    
    match tier {
        SubscriptionTier::Free => dec!(0.00),
        SubscriptionTier::Professional => dec!(0.00), // No setup fee
        SubscriptionTier::Enterprise => dec!(99.00),
        SubscriptionTier::Custom => dec!(0.00),
    }
}

// Mock activity execution function (replace with actual Temporal SDK calls)
async fn execute_activity<T, R>(
    _activity_name: &str,
    _request: T,
    _context: ActivityContext,
) -> std::result::Result<R, WorkflowError>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    // This is a placeholder - in real implementation, this would use Temporal SDK
    // to execute the actual activity
    Err(WorkflowError::ActivityFailed("Mock implementation".to_string()))
}