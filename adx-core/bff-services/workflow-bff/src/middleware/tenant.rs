use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};

use crate::AppState;
use crate::middleware::auth::Claims;

pub async fn tenant_middleware(
    State(_state): State<AppState>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get tenant ID from JWT claims (set by auth middleware)
    let claims = request.extensions().get::<Claims>();
    
    let tenant_id = match claims {
        Some(claims) => claims.tenant_id.clone(),
        None => {
            // Fallback to header if no claims (shouldn't happen after auth middleware)
            headers
                .get("x-tenant-id")
                .and_then(|header| header.to_str().ok())
                .unwrap_or("default")
                .to_string()
        }
    };

    // Add tenant ID to request extensions for use in handlers
    request.extensions_mut().insert(TenantContext { tenant_id });
    
    Ok(next.run(request).await)
}

#[derive(Debug, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
}