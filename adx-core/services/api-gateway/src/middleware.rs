use axum::{
    extract::{Request, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::sync::Arc;
use tracing::{debug, warn, error, info};
use uuid::Uuid;

use adx_shared::{JwtClaims, TenantContext, UserContext};
use crate::error::{ApiGatewayError, ApiResult};
use crate::rate_limiter::{RateLimiter, check_rate_limit_middleware};

/// Shared state for middleware
#[derive(Clone)]
pub struct MiddlewareState {
    pub rate_limiter: Arc<RateLimiter>,
    pub jwt_secret: String,
    pub require_auth: bool,
}

/// Request context extracted from middleware
#[derive(Debug, Clone)]
pub struct RequestContext {
    pub request_id: String,
    pub tenant_context: Option<TenantContext>,
    pub user_context: Option<UserContext>,
    pub jwt_claims: Option<JwtClaims>,
}

impl RequestContext {
    pub fn new() -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            tenant_context: None,
            user_context: None,
            jwt_claims: None,
        }
    }

    pub fn with_request_id(request_id: String) -> Self {
        Self {
            request_id,
            tenant_context: None,
            user_context: None,
            jwt_claims: None,
        }
    }
}

/// Request ID middleware - adds unique request ID to all requests
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Extract or generate request ID
    let request_id = request
        .headers()
        .get("X-Request-ID")
        .and_then(|h| h.to_str().ok())
        .map(|s| s.to_string())
        .unwrap_or_else(|| Uuid::new_v4().to_string());

    // Create request context
    let context = RequestContext::with_request_id(request_id.clone());
    
    // Insert context into request extensions
    request.extensions_mut().insert(context);

    // Call next middleware/handler
    let mut response = next.run(request).await;

    // Add request ID to response headers
    response.headers_mut().insert(
        "X-Request-ID",
        HeaderValue::from_str(&request_id).unwrap_or_else(|_| HeaderValue::from_static("invalid")),
    );

    response
}

/// Authentication middleware - validates JWT tokens and extracts user context
pub async fn auth_middleware(
    State(state): State<MiddlewareState>,
    mut request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    
    // Skip authentication for health checks and public endpoints
    if is_public_endpoint(path) {
        return next.run(request).await;
    }

    // Extract authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok());

    let jwt_claims = if let Some(auth_header) = auth_header {
        // Extract and validate JWT token
        let token = match extract_bearer_token(auth_header) {
            Ok(token) => token,
            Err(e) => return e.into_response(),
        };
        let claims = match validate_jwt_token(&token, &state.jwt_secret) {
            Ok(claims) => claims,
            Err(e) => return e.into_response(),
        };
        Some(claims)
    } else if state.require_auth {
        return ApiGatewayError::AuthenticationRequired.into_response();
    } else {
        None
    };

    // Update request context with authentication info
    let mut updated_context = request.extensions().get::<RequestContext>().cloned()
        .unwrap_or_else(RequestContext::new);
    
    {
        if let Some(ref claims) = jwt_claims {
            updated_context.jwt_claims = Some(claims.clone());
            updated_context.user_context = Some(UserContext {
                user_id: claims.sub.clone(),
                email: claims.user_email.clone(),
                display_name: None, // Should be loaded from DB
                roles: claims.user_roles.clone(),
                permissions: claims.permissions.clone(),
                quotas: claims.quotas.clone(),
                preferences: Default::default(),
                last_login: None,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });
            updated_context.tenant_context = Some(TenantContext {
                tenant_id: claims.tenant_id.clone(),
                tenant_name: claims.tenant_name.clone(),
                subscription_tier: adx_shared::SubscriptionTier::Professional, // Default, should be loaded from DB
                features: claims.features.clone(),
                quotas: adx_shared::TenantQuotas::default(), // Should be loaded from claims or DB
                settings: Default::default(),
                is_active: true,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            });
        }
    }
    
    debug!(
        path = path,
        user_id = jwt_claims.as_ref().map(|c| c.sub.as_str()),
        tenant_id = jwt_claims.as_ref().map(|c| c.tenant_id.as_str()),
        "Authentication middleware processed"
    );

    // Insert updated context back into request
    request.extensions_mut().insert(updated_context);

    next.run(request).await
}

/// Rate limiting middleware
pub async fn rate_limiting_middleware(
    State(state): State<MiddlewareState>,
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    
    // Skip rate limiting for health checks
    if is_health_endpoint(path) {
        return next.run(request).await;
    }

    // Get request context
    let context = request.extensions().get::<RequestContext>();
    
    let (tenant_id, user_id) = if let Some(context) = context {
        let tenant_id = context.tenant_context
            .as_ref()
            .map(|t| t.tenant_id.as_str())
            .unwrap_or("anonymous");
        let user_id = context.user_context
            .as_ref()
            .map(|u| u.user_id.as_str())
            .unwrap_or("anonymous");
        (tenant_id, user_id)
    } else {
        ("anonymous", "anonymous")
    };

    // Check rate limits
    if let Err(e) = check_rate_limit_middleware(&state.rate_limiter, tenant_id, user_id, path).await {
        return e.into_response();
    }

    debug!(
        path = path,
        tenant_id = tenant_id,
        user_id = user_id,
        "Rate limiting middleware passed"
    );

    next.run(request).await
}

/// Tenant context middleware - validates tenant access and injects tenant context
pub async fn tenant_middleware(
    request: Request,
    next: Next,
) -> Response {
    let path = request.uri().path();
    
    // Skip tenant validation for public endpoints
    if is_public_endpoint(path) {
        return next.run(request).await;
    }

    // Get request context
    let context = request.extensions().get::<RequestContext>().cloned();
    
    if let Some(context) = context {
        if let Some(tenant_context) = &context.tenant_context {
            // Validate tenant is active and user has access
            match is_tenant_active(&tenant_context.tenant_id).await {
                Ok(is_active) => {
                    if !is_active {
                        return ApiGatewayError::TenantAccessDenied {
                            reason: "Tenant is not active".to_string(),
                        }.into_response();
                    }
                }
                Err(e) => return e.into_response(),
            }

            debug!(
                path = path,
                tenant_id = %tenant_context.tenant_id,
                tenant_name = %tenant_context.tenant_name,
                "Tenant middleware validated"
            );
        }
    }

    next.run(request).await
}

/// CORS middleware
pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;

    // Add CORS headers
    let headers = response.headers_mut();
    headers.insert(
        "Access-Control-Allow-Origin",
        HeaderValue::from_static("*"), // Configure appropriately for production
    );
    headers.insert(
        "Access-Control-Allow-Methods",
        HeaderValue::from_static("GET, POST, PUT, DELETE, OPTIONS"),
    );
    headers.insert(
        "Access-Control-Allow-Headers",
        HeaderValue::from_static("Content-Type, Authorization, X-Tenant-ID, X-Request-ID"),
    );
    headers.insert(
        "Access-Control-Expose-Headers",
        HeaderValue::from_static("X-Request-ID, X-Rate-Limit-Remaining"),
    );

    response
}

/// Logging middleware
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_string();
    let start_time = std::time::Instant::now();

    // Get request context for logging
    let request_id = request.extensions()
        .get::<RequestContext>()
        .map(|c| c.request_id.clone())
        .unwrap_or_else(|| "unknown".to_string());

    info!(
        method = %method,
        path = %path,
        request_id = %request_id,
        "Request started"
    );

    let response = next.run(request).await;
    let duration = start_time.elapsed();

    info!(
        method = %method,
        path = %path,
        request_id = %request_id,
        status = %response.status(),
        duration_ms = duration.as_millis(),
        "Request completed"
    );

    response
}

/// Helper functions

fn is_public_endpoint(path: &str) -> bool {
    matches!(path, 
        "/health" | 
        "/metrics" | 
        "/api/v1/health" |
        "/api/v1/auth/login" |
        "/api/v1/auth/register" |
        "/api/v1/auth/refresh"
    )
}

fn is_health_endpoint(path: &str) -> bool {
    matches!(path, "/health" | "/api/v1/health" | "/metrics")
}

fn extract_bearer_token(auth_header: &str) -> ApiResult<String> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Ok(token.to_string())
    } else {
        Err(ApiGatewayError::InvalidToken {
            message: "Authorization header must start with 'Bearer '".to_string(),
        })
    }
}

fn validate_jwt_token(token: &str, secret: &str) -> ApiResult<JwtClaims> {
    use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};

    let key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    match decode::<JwtClaims>(token, &key, &validation) {
        Ok(token_data) => {
            // Check if token is expired
            let now = chrono::Utc::now().timestamp();
            if token_data.claims.exp < now {
                return Err(ApiGatewayError::InvalidToken {
                    message: "Token has expired".to_string(),
                });
            }

            Ok(token_data.claims)
        }
        Err(e) => Err(ApiGatewayError::InvalidToken {
            message: format!("Invalid JWT token: {}", e),
        }),
    }
}

async fn is_tenant_active(tenant_id: &str) -> ApiResult<bool> {
    // For now, assume all tenants are active
    // This should be replaced with actual tenant validation logic
    debug!(tenant_id = tenant_id, "Checking tenant status (simulated)");
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_public_endpoint_detection() {
        assert!(is_public_endpoint("/health"));
        assert!(is_public_endpoint("/api/v1/health"));
        assert!(is_public_endpoint("/api/v1/auth/login"));
        assert!(!is_public_endpoint("/api/v1/users"));
        assert!(!is_public_endpoint("/api/v1/workflows/test"));
    }

    #[test]
    fn test_bearer_token_extraction() {
        let valid_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9";
        let result = extract_bearer_token(valid_header);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9");

        let invalid_header = "Basic dXNlcjpwYXNz";
        let result = extract_bearer_token(invalid_header);
        assert!(result.is_err());
    }

    #[test]
    fn test_health_endpoint_detection() {
        assert!(is_health_endpoint("/health"));
        assert!(is_health_endpoint("/metrics"));
        assert!(!is_health_endpoint("/api/v1/users"));
    }
}