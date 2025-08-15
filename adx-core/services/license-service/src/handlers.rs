use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post, put, delete},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    error::{LicenseError, Result},
    models::*,
    services::LicenseService,
    workflows::*,
};

// Request/Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowResponse {
    pub workflow_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
}

// Application state
#[derive(Clone)]
pub struct AppState {
    pub license_service: LicenseService,
}

// Create router with all routes
pub fn create_router(state: AppState) -> Router {
    Router::new()
        // License management routes
        .route("/licenses", post(create_license_handler))
        .route("/licenses/:id", get(get_license_handler))
        .route("/licenses/:id", put(update_license_handler))
        .route("/licenses/tenant/:tenant_id", get(get_license_by_tenant_handler))
        .route("/licenses/validate/:license_key", get(validate_license_handler))
        .route("/licenses/expiring", get(get_expiring_licenses_handler))
        
        // Quota management routes
        .route("/quotas/tenant/:tenant_id", get(get_tenant_quotas_handler))
        .route("/quotas/tenant/:tenant_id/summary", get(get_quota_usage_summary_handler))
        .route("/quotas/check", post(check_quota_handler))
        .route("/quotas/enforce", post(enforce_quota_handler))
        .route("/quotas/reset", post(reset_quota_handler))
        
        // Billing routes
        .route("/billing/tenant/:tenant_id", get(get_billing_history_handler))
        .route("/billing/invoice", post(generate_invoice_handler))
        .route("/billing/:id/status", put(update_payment_status_handler))
        
        // Compliance routes
        .route("/compliance/tenant/:tenant_id/logs", get(get_compliance_logs_handler))
        .route("/compliance/tenant/:tenant_id/report", get(generate_compliance_report_handler))
        .route("/compliance/:id/resolve", post(resolve_compliance_issue_handler))
        
        // Workflow routes
        .route("/workflows/provision-license", post(provision_license_workflow_handler))
        .route("/workflows/enforce-quota", post(enforce_quota_workflow_handler))
        .route("/workflows/renew-license", post(renew_license_workflow_handler))
        
        // Analytics routes
        .route("/analytics/tenant/:tenant_id", get(get_license_analytics_handler))
        
        // Health check
        .route("/health", get(health_check_handler))
        
        .with_state(state)
}

// License handlers
async fn create_license_handler(
    State(state): State<AppState>,
    Json(request): Json<CreateLicenseRequest>,
) -> Result<Json<ApiResponse<License>>, StatusCode> {
    match state.license_service.create_license(request).await {
        Ok(license) => Ok(Json(ApiResponse {
            success: true,
            data: Some(license),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to create license: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_license_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<License>>, StatusCode> {
    match state.license_service.get_license(id).await {
        Ok(Some(license)) => Ok(Json(ApiResponse {
            success: true,
            data: Some(license),
            error: None,
            timestamp: Utc::now(),
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get license: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_license_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdateLicenseRequest>,
) -> Result<Json<ApiResponse<License>>, StatusCode> {
    match state.license_service.update_license(id, request).await {
        Ok(license) => Ok(Json(ApiResponse {
            success: true,
            data: Some(license),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to update license: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_license_by_tenant_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<ApiResponse<License>>, StatusCode> {
    match state.license_service.get_license_by_tenant(tenant_id).await {
        Ok(Some(license)) => Ok(Json(ApiResponse {
            success: true,
            data: Some(license),
            error: None,
            timestamp: Utc::now(),
        })),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => {
            tracing::error!("Failed to get license by tenant: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn validate_license_handler(
    State(state): State<AppState>,
    Path(license_key): Path<String>,
) -> Result<Json<ApiResponse<License>>, StatusCode> {
    match state.license_service.validate_license(&license_key).await {
        Ok(license) => Ok(Json(ApiResponse {
            success: true,
            data: Some(license),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(LicenseError::InvalidLicenseKey(_)) => Err(StatusCode::UNAUTHORIZED),
        Err(LicenseError::LicenseExpired { .. }) => Err(StatusCode::FORBIDDEN),
        Err(LicenseError::LicenseSuspended { .. }) => Err(StatusCode::FORBIDDEN),
        Err(e) => {
            tracing::error!("Failed to validate license: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_expiring_licenses_handler(
    State(state): State<AppState>,
    Query(query): Query<ExpiringLicensesQuery>,
) -> Result<Json<ApiResponse<Vec<License>>>, StatusCode> {
    let days_ahead = query.days_ahead.unwrap_or(30);
    
    match state.license_service.get_expiring_licenses(days_ahead).await {
        Ok(licenses) => Ok(Json(ApiResponse {
            success: true,
            data: Some(licenses),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to get expiring licenses: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Quota handlers
async fn get_tenant_quotas_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Vec<TenantQuota>>>, StatusCode> {
    match state.license_service.get_tenant_quotas(tenant_id).await {
        Ok(quotas) => Ok(Json(ApiResponse {
            success: true,
            data: Some(quotas),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to get tenant quotas: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_quota_usage_summary_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::services::QuotaUsageSummary>>, StatusCode> {
    match state.license_service.get_quota_usage_summary(tenant_id).await {
        Ok(summary) => Ok(Json(ApiResponse {
            success: true,
            data: Some(summary),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to get quota usage summary: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn check_quota_handler(
    State(state): State<AppState>,
    Json(request): Json<CheckQuotaApiRequest>,
) -> Result<Json<ApiResponse<QuotaCheckResult>>, StatusCode> {
    match state.license_service.check_quota(
        request.tenant_id,
        &request.quota_name,
        request.requested_amount,
    ).await {
        Ok(result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(result),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to check quota: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn enforce_quota_handler(
    State(state): State<AppState>,
    Json(request): Json<QuotaUsageRequest>,
) -> Result<Json<ApiResponse<QuotaCheckResult>>, StatusCode> {
    match state.license_service.enforce_quota(request).await {
        Ok(result) => Ok(Json(ApiResponse {
            success: true,
            data: Some(result),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(LicenseError::QuotaExceeded { .. }) => Err(StatusCode::TOO_MANY_REQUESTS),
        Err(e) => {
            tracing::error!("Failed to enforce quota: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn reset_quota_handler(
    State(state): State<AppState>,
    Json(request): Json<ResetQuotaRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.license_service.reset_quota(request.tenant_id, &request.quota_name).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to reset quota: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Billing handlers
async fn get_billing_history_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<Vec<BillingHistory>>>, StatusCode> {
    let limit = query.limit.unwrap_or(50);
    let offset = query.offset.unwrap_or(0);
    
    match state.license_service.get_billing_history(tenant_id, limit, offset).await {
        Ok(history) => Ok(Json(ApiResponse {
            success: true,
            data: Some(history),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to get billing history: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn generate_invoice_handler(
    State(state): State<AppState>,
    Json(request): Json<GenerateInvoiceApiRequest>,
) -> Result<Json<ApiResponse<BillingInvoice>>, StatusCode> {
    match state.license_service.generate_invoice(request.tenant_id, request.license_id).await {
        Ok(invoice) => Ok(Json(ApiResponse {
            success: true,
            data: Some(invoice),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to generate invoice: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_payment_status_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<UpdatePaymentStatusRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.license_service.update_payment_status(id, request.status, request.payment_reference).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to update payment status: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Compliance handlers
async fn get_compliance_logs_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ApiResponse<Vec<ComplianceLog>>>, StatusCode> {
    let start_date = query.start_date.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end_date = query.end_date.unwrap_or_else(Utc::now);
    
    match state.license_service.get_compliance_logs(tenant_id, start_date, end_date).await {
        Ok(logs) => Ok(Json(ApiResponse {
            success: true,
            data: Some(logs),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to get compliance logs: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn generate_compliance_report_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
    Query(query): Query<DateRangeQuery>,
) -> Result<Json<ApiResponse<ComplianceReport>>, StatusCode> {
    let start_date = query.start_date.unwrap_or_else(|| Utc::now() - chrono::Duration::days(30));
    let end_date = query.end_date.unwrap_or_else(Utc::now);
    
    match state.license_service.generate_compliance_report(tenant_id, start_date, end_date).await {
        Ok(report) => Ok(Json(ApiResponse {
            success: true,
            data: Some(report),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to generate compliance report: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn resolve_compliance_issue_handler(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<ResolveComplianceIssueRequest>,
) -> Result<Json<ApiResponse<()>>, StatusCode> {
    match state.license_service.resolve_compliance_issue(id, request.resolved_by, request.resolution_notes).await {
        Ok(_) => Ok(Json(ApiResponse {
            success: true,
            data: Some(()),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to resolve compliance issue: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Workflow handlers
async fn provision_license_workflow_handler(
    State(state): State<AppState>,
    Json(request): Json<LicenseProvisioningWorkflowRequest>,
) -> Result<Json<ApiResponse<WorkflowResponse>>, StatusCode> {
    match state.license_service.initiate_license_provisioning(request).await {
        Ok(workflow_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(WorkflowResponse {
                workflow_id,
                status: "started".to_string(),
                message: "License provisioning workflow initiated".to_string(),
            }),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to start license provisioning workflow: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn enforce_quota_workflow_handler(
    State(state): State<AppState>,
    Json(request): Json<QuotaEnforcementWorkflowRequest>,
) -> Result<Json<ApiResponse<WorkflowResponse>>, StatusCode> {
    match state.license_service.initiate_quota_enforcement(request).await {
        Ok(workflow_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(WorkflowResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Quota enforcement workflow initiated".to_string(),
            }),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to start quota enforcement workflow: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn renew_license_workflow_handler(
    State(state): State<AppState>,
    Json(request): Json<LicenseRenewalWorkflowRequest>,
) -> Result<Json<ApiResponse<WorkflowResponse>>, StatusCode> {
    match state.license_service.initiate_license_renewal(request).await {
        Ok(workflow_id) => Ok(Json(ApiResponse {
            success: true,
            data: Some(WorkflowResponse {
                workflow_id,
                status: "started".to_string(),
                message: "License renewal workflow initiated".to_string(),
            }),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to start license renewal workflow: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Analytics handler
async fn get_license_analytics_handler(
    State(state): State<AppState>,
    Path(tenant_id): Path<Uuid>,
) -> Result<Json<ApiResponse<crate::services::LicenseAnalytics>>, StatusCode> {
    match state.license_service.get_license_analytics(tenant_id).await {
        Ok(analytics) => Ok(Json(ApiResponse {
            success: true,
            data: Some(analytics),
            error: None,
            timestamp: Utc::now(),
        })),
        Err(e) => {
            tracing::error!("Failed to get license analytics: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Health check handler
async fn health_check_handler() -> Json<ApiResponse<String>> {
    Json(ApiResponse {
        success: true,
        data: Some("License service is healthy".to_string()),
        error: None,
        timestamp: Utc::now(),
    })
}

// Additional request DTOs
#[derive(Debug, Deserialize)]
pub struct ExpiringLicensesQuery {
    pub days_ahead: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CheckQuotaApiRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
    pub requested_amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct ResetQuotaRequest {
    pub tenant_id: Uuid,
    pub quota_name: String,
}

#[derive(Debug, Deserialize)]
pub struct GenerateInvoiceApiRequest {
    pub tenant_id: Uuid,
    pub license_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdatePaymentStatusRequest {
    pub status: PaymentStatus,
    pub payment_reference: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResolveComplianceIssueRequest {
    pub resolved_by: Uuid,
    pub resolution_notes: String,
}