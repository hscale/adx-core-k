use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::{middleware::auth::Claims, types::TenantContext, AppState};

pub async fn tenant_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip tenant validation for health check
    if request.uri().path() == "/health" {
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

    // Load tenant context (in production, this would query a tenant service)
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
    // Priority order for tenant ID extraction:
    
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
        // Extract subdomain from host like "tenant.adxcore.com"
        let subdomain = parts[0];
        if subdomain != "www" && subdomain != "api" {
            return Some(subdomain.to_string());
        }
    }
    None
}

fn extract_tenant_from_path(path: &str) -> Option<String> {
    // Extract tenant ID from path like "/tenant/{id}/..."
    let parts: Vec<&str> = path.split('/').collect();
    if parts.len() >= 3 && parts[1] == "tenant" {
        return Some(parts[2].to_string());
    }
    None
}

fn validate_tenant_access(claims: &Claims, tenant_id: &str) -> bool {
    // Check if user has access to the requested tenant
    claims.available_tenants.contains(&tenant_id.to_string()) || 
    claims.tenant_id == tenant_id
}

async fn load_tenant_context(tenant_id: &str, state: &AppState) -> Result<TenantContext, anyhow::Error> {
    // Try to get tenant context from cache first
    if let Ok(Some(cached_context)) = state.redis.get_cached_tenant_context(tenant_id).await {
        debug!("Loaded tenant context from cache for: {}", tenant_id);
        return Ok(cached_context);
    }

    // In production, this would call the tenant service
    // For now, we'll create a mock tenant context
    let tenant_context = create_mock_tenant_context(tenant_id);

    // Cache the tenant context
    if let Err(e) = state.redis.cache_tenant_context(tenant_id, &tenant_context, Some(300)).await {
        warn!("Failed to cache tenant context: {}", e);
    }

    Ok(tenant_context)
}

fn create_mock_tenant_context(tenant_id: &str) -> TenantContext {
    let mut quotas = HashMap::new();
    quotas.insert("storage_gb".to_string(), 100);
    quotas.insert("api_calls_per_hour".to_string(), 10000);
    quotas.insert("concurrent_uploads".to_string(), 10);

    TenantContext {
        tenant_id: tenant_id.to_string(),
        tenant_name: format!("Tenant {}", tenant_id),
        subscription_tier: "professional".to_string(),
        features: vec![
            "file_upload".to_string(),
            "file_sharing".to_string(),
            "virus_scanning".to_string(),
            "thumbnail_generation".to_string(),
        ],
        quotas,
    }
}

fn is_tenant_active(tenant_context: &TenantContext) -> bool {
    // In production, this would check tenant status, subscription, etc.
    // For now, we'll assume all tenants are active
    !tenant_context.tenant_id.is_empty()
}

// Extension methods for Redis service to handle tenant context caching
impl crate::services::RedisService {
    pub async fn cache_tenant_context(
        &self,
        tenant_id: &str,
        context: &TenantContext,
        ttl_seconds: Option<u64>,
    ) -> Result<(), anyhow::Error> {
        let key = format!("tenant:context:{}", tenant_id);
        self.set(&key, context, ttl_seconds).await
    }

    pub async fn get_cached_tenant_context(
        &self,
        tenant_id: &str,
    ) -> Result<Option<TenantContext>, anyhow::Error> {
        let key = format!("tenant:context:{}", tenant_id);
        self.get(&key).await
    }

    pub async fn invalidate_tenant_context(&self, tenant_id: &str) -> Result<(), anyhow::Error> {
        let key = format!("tenant:context:{}", tenant_id);
        self.delete(&key).await
    }
}

// Helper function to get tenant context from request
pub fn get_tenant_context(request: &Request) -> Option<&TenantContext> {
    request.extensions().get::<TenantContext>()
}

// Helper function to get tenant ID from request
pub fn get_tenant_id(request: &Request) -> Option<String> {
    get_tenant_context(request).map(|ctx| ctx.tenant_id.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::HeaderValue;

    #[test]
    fn test_extract_subdomain() {
        assert_eq!(extract_subdomain("tenant1.adxcore.com"), Some("tenant1".to_string()));
        assert_eq!(extract_subdomain("www.adxcore.com"), None);
        assert_eq!(extract_subdomain("api.adxcore.com"), None);
        assert_eq!(extract_subdomain("adxcore.com"), None);
    }

    #[test]
    fn test_extract_tenant_from_path() {
        assert_eq!(extract_tenant_from_path("/tenant/tenant1/files"), Some("tenant1".to_string()));
        assert_eq!(extract_tenant_from_path("/api/files"), None);
        assert_eq!(extract_tenant_from_path("/tenant"), None);
    }

    #[test]
    fn test_validate_tenant_access() {
        let mut claims = crate::middleware::auth::Claims {
            sub: "user123".to_string(),
            exp: 0,
            iat: 0,
            iss: "adx-core".to_string(),
            aud: "file-bff".to_string(),
            tenant_id: "tenant1".to_string(),
            tenant_name: "Tenant 1".to_string(),
            user_email: "test@example.com".to_string(),
            user_roles: vec![],
            permissions: vec![],
            features: vec![],
            session_id: "session123".to_string(),
            device_id: None,
            ip_address: "127.0.0.1".to_string(),
            available_tenants: vec!["tenant1".to_string(), "tenant2".to_string()],
            tenant_roles: HashMap::new(),
        };

        assert!(validate_tenant_access(&claims, "tenant1"));
        assert!(validate_tenant_access(&claims, "tenant2"));
        assert!(!validate_tenant_access(&claims, "tenant3"));
    }

    #[test]
    fn test_create_mock_tenant_context() {
        let context = create_mock_tenant_context("test-tenant");
        
        assert_eq!(context.tenant_id, "test-tenant");
        assert_eq!(context.tenant_name, "Tenant test-tenant");
        assert_eq!(context.subscription_tier, "professional");
        assert!(context.features.contains(&"file_upload".to_string()));
        assert!(context.quotas.contains_key("storage_gb"));
    }

    #[test]
    fn test_is_tenant_active() {
        let context = create_mock_tenant_context("test-tenant");
        assert!(is_tenant_active(&context));

        let inactive_context = TenantContext {
            tenant_id: "".to_string(),
            tenant_name: "".to_string(),
            subscription_tier: "".to_string(),
            features: vec![],
            quotas: HashMap::new(),
        };
        assert!(!is_tenant_active(&inactive_context));
    }
}