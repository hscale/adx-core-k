use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use adx_shared::{
    auth::{JwtClaims, TenantContext},
    types::{TenantId, SubscriptionTier, TenantQuotas},
};
use crate::AppState;

/// Tenant context middleware that loads and validates tenant information
pub async fn tenant_context_middleware(
    State(_state): State<AppState>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Get JWT claims from request extensions (should be set by auth middleware)
    let claims = match request.extensions().get::<JwtClaims>() {
        Some(claims) => claims,
        None => {
            // Try to extract tenant ID from headers or path if no auth
            let tenant_id = extract_tenant_id_from_request(&request)?;
            return load_tenant_context_without_auth(tenant_id, request, next).await;
        }
    };

    // Load tenant context
    let tenant_context = match load_tenant_context(&claims.tenant_id).await {
        Ok(context) => context,
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };

    // Validate tenant is active
    if !tenant_context.is_active {
        return Err(StatusCode::FORBIDDEN);
    }

    // Add tenant context to request extensions
    request.extensions_mut().insert(tenant_context);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

async fn load_tenant_context_without_auth(
    tenant_id: TenantId,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Load tenant context for unauthenticated requests
    let tenant_context = match load_tenant_context(&tenant_id).await {
        Ok(context) => context,
        Err(_) => return Err(StatusCode::NOT_FOUND),
    };

    // Add tenant context to request extensions
    request.extensions_mut().insert(tenant_context);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

fn extract_tenant_id_from_request(request: &Request) -> std::result::Result<TenantId, StatusCode> {
    let headers = request.headers();
    
    // Priority order for tenant ID extraction:
    // 1. X-Tenant-ID header
    if let Some(header_value) = headers.get("X-Tenant-ID") {
        if let Ok(tenant_id) = header_value.to_str() {
            return Ok(tenant_id.to_string());
        }
    }
    
    // 2. Host header (subdomain extraction)
    if let Some(host) = headers.get("Host") {
        if let Ok(host_str) = host.to_str() {
            if let Some(subdomain) = extract_subdomain(host_str) {
                return Ok(subdomain);
            }
        }
    }
    
    // 3. Path prefix (/tenant/{id}/...)
    if let Some(path_tenant) = extract_tenant_from_path(request.uri().path()) {
        return Ok(path_tenant);
    }
    
    Err(StatusCode::BAD_REQUEST)
}

fn extract_subdomain(host: &str) -> Option<String> {
    // Extract subdomain from host like "tenant.adxcore.com"
    let parts: Vec<&str> = host.split('.').collect();
    if parts.len() >= 3 && parts[1] == "adxcore" && parts[2] == "com" {
        Some(parts[0].to_string())
    } else {
        None
    }
}

fn extract_tenant_from_path(path: &str) -> Option<String> {
    // Extract tenant ID from path like "/tenant/{id}/..."
    if path.starts_with("/tenant/") {
        let parts: Vec<&str> = path.split('/').collect();
        if parts.len() >= 3 {
            Some(parts[2].to_string())
        } else {
            None
        }
    } else {
        None
    }
}

async fn load_tenant_context(tenant_id: &str) -> std::result::Result<TenantContext, TenantError> {
    // TODO: Load tenant from database
    // For now, return a mock tenant context
    Ok(TenantContext {
        tenant_id: tenant_id.to_string(),
        tenant_name: format!("Tenant {}", tenant_id),
        subscription_tier: SubscriptionTier::Professional,
        features: vec![
            "basic_features".to_string(),
            "advanced_analytics".to_string(),
        ],
        quotas: TenantQuotas::default(),
        settings: adx_shared::auth::TenantSettings::default(),
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    })
}

#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("Tenant not found")]
    NotFound,
    #[error("Tenant is suspended")]
    Suspended,
    #[error("Tenant subscription expired")]
    Expired,
    #[error("Database error: {0}")]
    Database(String),
    #[error("Internal error: {0}")]
    Internal(String),
}