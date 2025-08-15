use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::{middleware::auth::Claims, types::TenantContext, AppState};

pub async fn tenant_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip tenant validation for health check and WebSocket upgrade
    let path = request.uri().path();
    if path == "/health" || path.starts_with("/ws") {
        return Ok(next.run(request).await);
    }

    let headers = request.headers();
    
    // Extract tenant ID from various sources
    let tenant_id = extract_tenant_id(headers, &request)
        .ok_or(StatusCode::BAD_REQUEST)?;

    // Get user claims from auth middleware
    let claims = request
        .extensions()
        .get::<Claims>()
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate user has access to the requested tenant
    if !validate_tenant_access(claims, &tenant_id) {
        warn!(
            "User {} attempted to access unauthorized tenant: {}",
            claims.sub, tenant_id
        );
        return Err(StatusCode::FORBIDDEN);
    }

    // Load tenant context
    let tenant_context = load_tenant_context(&tenant_id, &state).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Validate tenant is active
    if !is_tenant_active(&tenant_context) {
        warn!("Attempted access to inactive tenant: {}", tenant_id);
        return Err(StatusCode::FORBIDDEN);
    }

    // Add tenant context to request extensions
    request.extensions_mut().insert(tenant_context);

    debug!("Tenant context validated for tenant: {}", tenant_id);

    Ok(next.run(request).await)
}

fn extract_tenant_id(headers: &HeaderMap, request: &Request) -> Option<String> {
    // 1. X-Tenant-ID header
    if let Some(header_value) = headers.get("X-Tenant-ID") {
        if let Ok(tenant_id) = header_value.to_str() {
            return Some(tenant_id.to_string());
        }
    }
    
    // 2. Subdomain (tenant.adxcore.com)
    if let Some(host) = headers.get("Host") {
        if let Ok(host_str) = host.to_str() {
            if let Some(subdomain) = extract_subdomain(host_str) {
                return Some(subdomain);
            }
        }
    }
    
    // 3. Path prefix (/tenant/{id}/...)
    if let Some(path_tenant) = extract_tenant_from_path(request.uri().path()) {
        return Some(path_tenant);
    }
    
    None
}

fn extract_subdomain(host: &str) -> Option<String> {
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 3 {
        let subdomain = parts[0];
        if subdomain != "www" && subdomain != "api" && subdomain != "workflow-bff" {
            return Some(subdomain.to_string());
        }
    }
    None
}

fn extract_tenant_from_path(path: &str) -> Option<String> {
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 && parts[1] == "tenant" {
        return Some(parts[2].to_string());
    }
    None
}

fn validate_tenant_access(claims: &Claims, tenant_id: &str) -> bool {
    claims.available_tenants.contains(&tenant_id.to_string()) || 
    claims.tenant_id == tenant_id
}

async fn load_tenant_context(tenant_id: &str, state: &AppState) -> Result<TenantContext, anyhow::Error> {
    // Try to get tenant context from cache first
    if let Ok(Some(cached_context)) = state.redis.get_cached_tenant_context(tenant_id).await {
        debug!("Loaded tenant context from cache for: {}", tenant_id);
        return Ok(cached_context);
    }

    // Create mock tenant context for workflow BFF
    let tenant_context = create_mock_tenant_context(tenant_id);

    // Cache the tenant context
    if let Err(e) = state.redis.cache_tenant_context(tenant_id, &tenant_context, Some(300)).await {
        warn!("Failed to cache tenant context: {}", e);
    }

    Ok(tenant_context)
}

fn create_mock_tenant_context(tenant_id: &str) -> TenantContext {
    let mut quotas = HashMap::new();
    quotas.insert("max_workflows".to_string(), 1000);
    quotas.insert("concurrent_workflows".to_string(), 50);
    quotas.insert("workflow_history_retention_days".to_string(), 90);
    quotas.insert("max_workflow_duration_hours".to_string(), 24);

    TenantContext {
        tenant_id: tenant_id.to_string(),
        tenant_name: format!("Tenant {}", tenant_id),
        subscription_tier: "professional".to_string(),
        features: vec![
            "workflow_monitoring".to_string(),
            "advanced_analytics".to_string(),
            "real_time_updates".to_string(),
            "workflow_templates".to_string(),
            "workflow_scheduling".to_string(),
            "custom_dashboards".to_string(),
        ],
        quotas,
    }
}

fn is_tenant_active(tenant_context: &TenantContext) -> bool {
    !tenant_context.tenant_id.is_empty()
}

// Helper function to get tenant context from request
pub fn get_tenant_context(request: &Request) -> Option<&TenantContext> {
    request.extensions().get::<TenantContext>()
}

// Helper function to get tenant ID from request
pub fn get_tenant_id(request: &Request) -> Option<String> {
    get_tenant_context(request).map(|ctx| ctx.tenant_id.clone())
}