use crate::error::{WhiteLabelError, WhiteLabelResult};
use crate::types::*;
use crate::workflows::*;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::collections::HashMap;
use std::sync::Arc;
use crate::temporal_mock::TemporalClient;
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: Arc<PgPool>,
    pub temporal_client: Arc<TemporalClient>,
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        // Domain management routes
        .route("/domains", post(create_custom_domain))
        .route("/domains", get(list_custom_domains))
        .route("/domains/:domain_id", get(get_custom_domain))
        .route("/domains/:domain_id", delete(delete_custom_domain))
        .route("/domains/:domain_id/verify", post(verify_custom_domain))
        .route("/domains/:domain_id/ssl", post(provision_ssl_certificate))
        
        // Branding management routes
        .route("/branding", post(create_white_label_branding))
        .route("/branding", get(get_white_label_branding))
        .route("/branding", put(update_white_label_branding))
        .route("/branding", delete(delete_white_label_branding))
        .route("/branding/preview", get(get_branding_preview))
        .route("/branding/rollback", post(rollback_branding))
        
        // Reseller management routes
        .route("/resellers", post(create_reseller))
        .route("/resellers", get(list_resellers))
        .route("/resellers/:reseller_id", get(get_reseller))
        .route("/resellers/:reseller_id", put(update_reseller))
        .route("/resellers/:reseller_id", delete(delete_reseller))
        .route("/resellers/:reseller_id/hierarchy", get(get_reseller_hierarchy))
        
        // Asset management routes
        .route("/assets", post(upload_branding_asset))
        .route("/assets", get(list_branding_assets))
        .route("/assets/:asset_id", get(get_branding_asset))
        .route("/assets/:asset_id", delete(delete_branding_asset))
        
        // Workflow status routes
        .route("/workflows/:operation_id/status", get(get_workflow_status))
        .route("/workflows/:operation_id/cancel", post(cancel_workflow))
        
        // Health check
        .route("/health", get(health_check))
}

// Domain management handlers
#[derive(Debug, Deserialize)]
pub struct CreateCustomDomainRequest {
    pub domain: String,
    pub ssl_enabled: bool,
    pub auto_redirect: bool,
    pub dns_provider: Option<String>,
}

pub async fn create_custom_domain(
    State(state): State<AppState>,
    Json(request): Json<CreateCustomDomainRequest>,
) -> WhiteLabelResult<Json<WorkflowResponse>> {
    let tenant_id = "default_tenant".to_string(); // This would come from auth context
    
    let workflow_request = CustomDomainSetupRequest {
        tenant_id,
        domain: request.domain,
        ssl_enabled: request.ssl_enabled,
        auto_redirect: request.auto_redirect,
        dns_provider: request.dns_provider,
    };

    let operation_id = Uuid::new_v4().to_string();
    
    // Start workflow
    let _handle = state
        .temporal_client
        .start_workflow(
            "custom_domain_setup_workflow",
            operation_id.clone(),
            "white-label-task-queue",
            workflow_request,
        )
        .await
        .map_err(|e| WhiteLabelError::Temporal(e.to_string()))?;

    Ok(Json(WorkflowResponse::Started {
        operation_id,
        status_url: format!("/api/white-label/workflows/{}/status", operation_id),
        estimated_duration_seconds: Some(300), // 5 minutes
    }))
}

#[derive(Debug, Deserialize)]
pub struct ListDomainsQuery {
    pub tenant_id: Option<String>,
    pub status: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn list_custom_domains(
    State(state): State<AppState>,
    Query(query): Query<ListDomainsQuery>,
) -> WhiteLabelResult<Json<Vec<CustomDomain>>> {
    let tenant_id = query.tenant_id.unwrap_or_else(|| "default_tenant".to_string());
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);

    let mut sql = "SELECT id, tenant_id, domain, status, verification_token, ssl_certificate_id, created_at, verified_at, expires_at FROM custom_domains WHERE tenant_id = $1".to_string();
    let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Postgres> + Send + Sync>> = vec![Box::new(tenant_id)];
    let mut param_count = 1;

    if let Some(status) = query.status {
        param_count += 1;
        sql.push_str(&format!(" AND status = ${}", param_count));
        params.push(Box::new(status));
    }

    sql.push_str(&format!(" ORDER BY created_at DESC LIMIT ${} OFFSET ${}", param_count + 1, param_count + 2));
    params.push(Box::new(limit));
    params.push(Box::new(offset));

    let domains = sqlx::query_as::<_, crate::models::CustomDomainModel>(&sql)
        .bind(&params[0])
        .fetch_all(&*state.db_pool)
        .await?
        .into_iter()
        .map(CustomDomain::from)
        .collect();

    Ok(Json(domains))
}

pub async fn get_custom_domain(
    State(state): State<AppState>,
    Path(domain_id): Path<Uuid>,
) -> WhiteLabelResult<Json<CustomDomain>> {
    let domain = sqlx::query_as::<_, crate::models::CustomDomainModel>(
        "SELECT id, tenant_id, domain, status, verification_token, ssl_certificate_id, created_at, verified_at, expires_at FROM custom_domains WHERE id = $1"
    )
    .bind(domain_id)
    .fetch_optional(&*state.db_pool)
    .await?
    .ok_or_else(|| WhiteLabelError::NotFound("Domain not found".to_string()))?;

    Ok(Json(CustomDomain::from(domain)))
}

pub async fn delete_custom_domain(
    State(state): State<AppState>,
    Path(domain_id): Path<Uuid>,
) -> WhiteLabelResult<StatusCode> {
    let result = sqlx::query!("DELETE FROM custom_domains WHERE id = $1", domain_id)
        .execute(&*state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(WhiteLabelError::NotFound("Domain not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn verify_custom_domain(
    State(state): State<AppState>,
    Path(domain_id): Path<Uuid>,
) -> WhiteLabelResult<Json<DomainVerificationResult>> {
    // Get domain details
    let domain = sqlx::query!("SELECT domain FROM custom_domains WHERE id = $1", domain_id)
        .fetch_optional(&*state.db_pool)
        .await?
        .ok_or_else(|| WhiteLabelError::NotFound("Domain not found".to_string()))?;

    // This would trigger DNS verification
    // For now, return a mock result
    Ok(Json(DomainVerificationResult {
        verified: true,
        verification_method: "DNS".to_string(),
        dns_records_found: vec![],
        error_message: None,
    }))
}

pub async fn provision_ssl_certificate(
    State(state): State<AppState>,
    Path(domain_id): Path<Uuid>,
) -> WhiteLabelResult<Json<SslCertificateResult>> {
    // Get domain details
    let domain = sqlx::query!("SELECT domain FROM custom_domains WHERE id = $1", domain_id)
        .fetch_optional(&*state.db_pool)
        .await?
        .ok_or_else(|| WhiteLabelError::NotFound("Domain not found".to_string()))?;

    // This would trigger SSL certificate provisioning
    // For now, return a mock result
    Ok(Json(SslCertificateResult {
        certificate_id: Uuid::new_v4().to_string(),
        certificate_arn: None,
        status: SslStatus::Issued,
        expires_at: chrono::Utc::now() + chrono::Duration::days(90),
        auto_renewal: true,
    }))
}

// Branding management handlers
pub async fn create_white_label_branding(
    State(state): State<AppState>,
    Json(request): Json<WhiteLabelBrandingRequest>,
) -> WhiteLabelResult<Json<WorkflowResponse>> {
    let operation_id = Uuid::new_v4().to_string();
    
    // Start workflow
    let _handle = state
        .temporal_client
        .start_workflow(
            "white_label_branding_workflow",
            operation_id.clone(),
            "white-label-task-queue",
            request,
        )
        .await
        .map_err(|e| WhiteLabelError::Temporal(e.to_string()))?;

    Ok(Json(WorkflowResponse::Started {
        operation_id,
        status_url: format!("/api/white-label/workflows/{}/status", operation_id),
        estimated_duration_seconds: Some(180), // 3 minutes
    }))
}

pub async fn get_white_label_branding(
    State(state): State<AppState>,
) -> WhiteLabelResult<Json<WhiteLabelBranding>> {
    let tenant_id = "default_tenant".to_string(); // This would come from auth context
    
    let branding = sqlx::query_as::<_, crate::models::WhiteLabelBrandingModel>(
        "SELECT id, tenant_id, brand_name, logo_url, favicon_url, primary_color, secondary_color, accent_color, font_family, custom_css, email_templates, created_at, updated_at FROM white_label_branding WHERE tenant_id = $1"
    )
    .bind(tenant_id)
    .fetch_optional(&*state.db_pool)
    .await?
    .ok_or_else(|| WhiteLabelError::NotFound("Branding not found".to_string()))?;

    Ok(Json(WhiteLabelBranding::from(branding)))
}

pub async fn update_white_label_branding(
    State(state): State<AppState>,
    Json(request): Json<WhiteLabelBrandingRequest>,
) -> WhiteLabelResult<Json<WorkflowResponse>> {
    let operation_id = Uuid::new_v4().to_string();
    
    // Start workflow
    let _handle = state
        .temporal_client
        .start_workflow(
            "white_label_branding_workflow",
            operation_id.clone(),
            "white-label-task-queue",
            request,
        )
        .await
        .map_err(|e| WhiteLabelError::Temporal(e.to_string()))?;

    Ok(Json(WorkflowResponse::Started {
        operation_id,
        status_url: format!("/api/white-label/workflows/{}/status", operation_id),
        estimated_duration_seconds: Some(180), // 3 minutes
    }))
}

pub async fn delete_white_label_branding(
    State(state): State<AppState>,
) -> WhiteLabelResult<StatusCode> {
    let tenant_id = "default_tenant".to_string(); // This would come from auth context
    
    let result = sqlx::query!("DELETE FROM white_label_branding WHERE tenant_id = $1", tenant_id)
        .execute(&*state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(WhiteLabelError::NotFound("Branding not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_branding_preview(
    State(state): State<AppState>,
) -> WhiteLabelResult<Json<BrandingPreviewResponse>> {
    let tenant_id = "default_tenant".to_string(); // This would come from auth context
    
    let preview_url = format!("https://preview.adxcore.com/branding/{}", tenant_id);
    
    Ok(Json(BrandingPreviewResponse { preview_url }))
}

#[derive(Debug, Serialize)]
pub struct BrandingPreviewResponse {
    pub preview_url: String,
}

#[derive(Debug, Deserialize)]
pub struct RollbackBrandingRequest {
    pub backup_id: Uuid,
}

pub async fn rollback_branding(
    State(state): State<AppState>,
    Json(request): Json<RollbackBrandingRequest>,
) -> WhiteLabelResult<StatusCode> {
    let tenant_id = "default_tenant".to_string(); // This would come from auth context
    
    // This would trigger a rollback workflow
    // For now, just return success
    Ok(StatusCode::OK)
}

// Reseller management handlers
pub async fn create_reseller(
    State(state): State<AppState>,
    Json(request): Json<ResellerSetupRequest>,
) -> WhiteLabelResult<Json<WorkflowResponse>> {
    let operation_id = Uuid::new_v4().to_string();
    
    // Start workflow
    let _handle = state
        .temporal_client
        .start_workflow(
            "reseller_setup_workflow",
            operation_id.clone(),
            "white-label-task-queue",
            request,
        )
        .await
        .map_err(|e| WhiteLabelError::Temporal(e.to_string()))?;

    Ok(Json(WorkflowResponse::Started {
        operation_id,
        status_url: format!("/api/white-label/workflows/{}/status", operation_id),
        estimated_duration_seconds: Some(240), // 4 minutes
    }))
}

pub async fn list_resellers(
    State(state): State<AppState>,
) -> WhiteLabelResult<Json<Vec<ResellerHierarchy>>> {
    let resellers = sqlx::query_as::<_, crate::models::ResellerHierarchyModel>(
        "SELECT id, parent_reseller_id, tenant_id, reseller_name, reseller_type, commission_rate, revenue_share_model, support_contact, branding_overrides, allowed_features, created_at, updated_at FROM reseller_hierarchies ORDER BY created_at DESC"
    )
    .fetch_all(&*state.db_pool)
    .await?
    .into_iter()
    .map(ResellerHierarchy::from)
    .collect();

    Ok(Json(resellers))
}

pub async fn get_reseller(
    State(state): State<AppState>,
    Path(reseller_id): Path<Uuid>,
) -> WhiteLabelResult<Json<ResellerHierarchy>> {
    let reseller = sqlx::query_as::<_, crate::models::ResellerHierarchyModel>(
        "SELECT id, parent_reseller_id, tenant_id, reseller_name, reseller_type, commission_rate, revenue_share_model, support_contact, branding_overrides, allowed_features, created_at, updated_at FROM reseller_hierarchies WHERE id = $1"
    )
    .bind(reseller_id)
    .fetch_optional(&*state.db_pool)
    .await?
    .ok_or_else(|| WhiteLabelError::NotFound("Reseller not found".to_string()))?;

    Ok(Json(ResellerHierarchy::from(reseller)))
}

pub async fn update_reseller(
    State(state): State<AppState>,
    Path(reseller_id): Path<Uuid>,
    Json(request): Json<ResellerSetupRequest>,
) -> WhiteLabelResult<Json<WorkflowResponse>> {
    let operation_id = Uuid::new_v4().to_string();
    
    // This would trigger an update workflow
    // For now, return a mock response
    Ok(Json(WorkflowResponse::Started {
        operation_id,
        status_url: format!("/api/white-label/workflows/{}/status", operation_id),
        estimated_duration_seconds: Some(120), // 2 minutes
    }))
}

pub async fn delete_reseller(
    State(state): State<AppState>,
    Path(reseller_id): Path<Uuid>,
) -> WhiteLabelResult<StatusCode> {
    let result = sqlx::query!("DELETE FROM reseller_hierarchies WHERE id = $1", reseller_id)
        .execute(&*state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(WhiteLabelError::NotFound("Reseller not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Serialize)]
pub struct ResellerHierarchyResponse {
    pub reseller: ResellerHierarchy,
    pub children: Vec<ResellerHierarchy>,
    pub hierarchy_level: u32,
}

pub async fn get_reseller_hierarchy(
    State(state): State<AppState>,
    Path(reseller_id): Path<Uuid>,
) -> WhiteLabelResult<Json<ResellerHierarchyResponse>> {
    // Get the reseller
    let reseller = sqlx::query_as::<_, crate::models::ResellerHierarchyModel>(
        "SELECT id, parent_reseller_id, tenant_id, reseller_name, reseller_type, commission_rate, revenue_share_model, support_contact, branding_overrides, allowed_features, created_at, updated_at FROM reseller_hierarchies WHERE id = $1"
    )
    .bind(reseller_id)
    .fetch_optional(&*state.db_pool)
    .await?
    .ok_or_else(|| WhiteLabelError::NotFound("Reseller not found".to_string()))?;

    // Get children
    let children = sqlx::query_as::<_, crate::models::ResellerHierarchyModel>(
        "SELECT id, parent_reseller_id, tenant_id, reseller_name, reseller_type, commission_rate, revenue_share_model, support_contact, branding_overrides, allowed_features, created_at, updated_at FROM reseller_hierarchies WHERE parent_reseller_id = $1"
    )
    .bind(reseller_id)
    .fetch_all(&*state.db_pool)
    .await?
    .into_iter()
    .map(ResellerHierarchy::from)
    .collect();

    Ok(Json(ResellerHierarchyResponse {
        reseller: ResellerHierarchy::from(reseller),
        children,
        hierarchy_level: 1, // This would be calculated properly
    }))
}

// Asset management handlers
#[derive(Debug, Deserialize)]
pub struct UploadAssetRequest {
    pub asset_type: AssetType,
    pub filename: String,
    pub file_data: Vec<u8>,
}

pub async fn upload_branding_asset(
    State(state): State<AppState>,
    Json(request): Json<UploadAssetRequest>,
) -> WhiteLabelResult<Json<BrandingAsset>> {
    let tenant_id = "default_tenant".to_string(); // This would come from auth context
    
    // This would process the asset upload
    // For now, return a mock response
    Ok(Json(BrandingAsset {
        id: Uuid::new_v4(),
        tenant_id,
        asset_type: request.asset_type,
        original_filename: request.filename,
        file_path: "assets/mock/path.png".to_string(),
        file_size: request.file_data.len() as u64,
        mime_type: "image/png".to_string(),
        dimensions: Some(AssetDimensions {
            width: 256,
            height: 256,
        }),
        checksum: "mock_checksum".to_string(),
        created_at: chrono::Utc::now(),
    }))
}

pub async fn list_branding_assets(
    State(state): State<AppState>,
) -> WhiteLabelResult<Json<Vec<BrandingAsset>>> {
    let tenant_id = "default_tenant".to_string(); // This would come from auth context
    
    let assets = sqlx::query_as::<_, crate::models::BrandingAssetModel>(
        "SELECT id, tenant_id, asset_type, original_filename, file_path, file_size, mime_type, dimensions_width, dimensions_height, checksum, created_at FROM branding_assets WHERE tenant_id = $1 ORDER BY created_at DESC"
    )
    .bind(tenant_id)
    .fetch_all(&*state.db_pool)
    .await?
    .into_iter()
    .map(BrandingAsset::from)
    .collect();

    Ok(Json(assets))
}

pub async fn get_branding_asset(
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
) -> WhiteLabelResult<Json<BrandingAsset>> {
    let asset = sqlx::query_as::<_, crate::models::BrandingAssetModel>(
        "SELECT id, tenant_id, asset_type, original_filename, file_path, file_size, mime_type, dimensions_width, dimensions_height, checksum, created_at FROM branding_assets WHERE id = $1"
    )
    .bind(asset_id)
    .fetch_optional(&*state.db_pool)
    .await?
    .ok_or_else(|| WhiteLabelError::NotFound("Asset not found".to_string()))?;

    Ok(Json(BrandingAsset::from(asset)))
}

pub async fn delete_branding_asset(
    State(state): State<AppState>,
    Path(asset_id): Path<Uuid>,
) -> WhiteLabelResult<StatusCode> {
    let result = sqlx::query!("DELETE FROM branding_assets WHERE id = $1", asset_id)
        .execute(&*state.db_pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(WhiteLabelError::NotFound("Asset not found".to_string()));
    }

    Ok(StatusCode::NO_CONTENT)
}

// Workflow status handlers
#[derive(Debug, Serialize)]
pub struct WorkflowStatusResponse {
    pub operation_id: String,
    pub status: String,
    pub progress: Option<WorkflowProgress>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub completed_steps: u32,
    pub total_steps: u32,
    pub percentage: f32,
}

pub async fn get_workflow_status(
    State(state): State<AppState>,
    Path(operation_id): Path<String>,
) -> WhiteLabelResult<Json<WorkflowStatusResponse>> {
    // This would query the Temporal client for workflow status
    // For now, return a mock response
    Ok(Json(WorkflowStatusResponse {
        operation_id,
        status: "running".to_string(),
        progress: Some(WorkflowProgress {
            current_step: "Processing assets".to_string(),
            completed_steps: 2,
            total_steps: 5,
            percentage: 40.0,
        }),
        result: None,
        error: None,
    }))
}

pub async fn cancel_workflow(
    State(state): State<AppState>,
    Path(operation_id): Path<String>,
) -> WhiteLabelResult<StatusCode> {
    // This would cancel the workflow
    // For now, just return success
    Ok(StatusCode::OK)
}

// Utility handlers
pub async fn health_check() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "white-label-service",
        "timestamp": chrono::Utc::now()
    }))
}

#[derive(Debug, Serialize)]
pub enum WorkflowResponse {
    Started {
        operation_id: String,
        status_url: String,
        estimated_duration_seconds: Option<u64>,
    },
    Completed {
        result: serde_json::Value,
    },
}