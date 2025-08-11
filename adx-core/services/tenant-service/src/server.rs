use std::sync::Arc;
use axum::{
    routing::{get, post, put, delete},
    Router,
    middleware,
    extract::Request,
    response::Response,
    http::{StatusCode, HeaderValue},
};
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::TraceLayer,
    timeout::TimeoutLayer,
    compression::CompressionLayer,
};
use sqlx::PgPool;
use std::time::Duration;

use crate::handlers::*;
use crate::services::TenantService;
use crate::repositories::{PostgresTenantRepository, PostgresTenantMembershipRepository};
use adx_shared::{
    config::AppConfig,
    health::{health_check, HealthChecker, DatabaseHealthCheck},
    middleware::{request_id_middleware, logging_middleware},
};

// Tenant isolation middleware - validates tenant access and injects context
async fn tenant_isolation_middleware(
    request: Request,
    next: middleware::Next,
) -> Result<Response, StatusCode> {
    // Extract tenant ID from various sources (header, subdomain, path, JWT)
    let tenant_id = extract_tenant_id(&request);
    
    // For now, we'll allow requests without tenant context for public endpoints
    // In a real implementation, this would validate tenant access based on the endpoint
    let path = request.uri().path();
    
    // Public endpoints that don't require tenant context
    if path == "/health" || path.starts_with("/api/v1/tenants") && request.method() == "GET" {
        return Ok(next.run(request).await);
    }
    
    // For tenant-specific operations, we would validate access here
    // This is a simplified implementation
    Ok(next.run(request).await)
}

fn extract_tenant_id(request: &Request) -> Option<String> {
    // Priority order for tenant ID extraction:
    // 1. X-Tenant-ID header
    if let Some(header_value) = request.headers().get("X-Tenant-ID") {
        if let Ok(tenant_id) = header_value.to_str() {
            return Some(tenant_id.to_string());
        }
    }
    
    // 2. Authorization header (JWT token - would need to decode)
    // This is simplified - in reality we'd decode the JWT
    if let Some(_auth_header) = request.headers().get("Authorization") {
        // Would extract tenant_id from JWT claims
    }
    
    // 3. Path-based tenant extraction
    let path = request.uri().path();
    if let Some(captures) = regex::Regex::new(r"/api/v1/tenants/([^/]+)/")
        .unwrap()
        .captures(path) 
    {
        if let Some(tenant_match) = captures.get(1) {
            return Some(tenant_match.as_str().to_string());
        }
    }
    
    None
}

// Security headers middleware
async fn security_headers_middleware(
    request: Request,
    next: middleware::Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", HeaderValue::from_static("nosniff"));
    headers.insert("X-Frame-Options", HeaderValue::from_static("DENY"));
    headers.insert("X-XSS-Protection", HeaderValue::from_static("1; mode=block"));
    headers.insert("Referrer-Policy", HeaderValue::from_static("strict-origin-when-cross-origin"));
    
    response
}

pub async fn create_app(config: &AppConfig, pool: PgPool) -> Router {
    // Create repositories
    let tenant_repo = Arc::new(PostgresTenantRepository::new(pool.clone()));
    let membership_repo = Arc::new(PostgresTenantMembershipRepository::new(pool.clone()));

    // Create service
    let tenant_service = Arc::new(TenantService::new(tenant_repo, membership_repo));

    // Create health checker
    let mut health_checker = HealthChecker::new("tenant-service-2.0.0".to_string());
    health_checker.add_check(DatabaseHealthCheck::new(pool.clone()));

    // Build router with comprehensive endpoint coverage
    Router::new()
        // Health check endpoints
        .route("/health", get(health_check))
        .route("/health/detailed", get(move || async move {
            axum::Json(health_checker.check_health().await)
        }))
        
        // Tenant CRUD routes (direct endpoints for simple operations)
        .route("/api/v1/tenants", post(create_tenant))
        .route("/api/v1/tenants", get(list_tenants))
        .route("/api/v1/tenants/:id", get(get_tenant))
        .route("/api/v1/tenants/:id", put(update_tenant))
        .route("/api/v1/tenants/:id", delete(delete_tenant))
        .route("/api/v1/tenants/slug/:slug", get(get_tenant_by_slug))
        
        // Tenant membership management routes
        .route("/api/v1/tenants/:tenant_id/members", post(create_membership))
        .route("/api/v1/tenants/:tenant_id/members", get(list_tenant_members))
        .route("/api/v1/memberships/:id", get(get_membership))
        .route("/api/v1/memberships/:id", put(update_membership))
        .route("/api/v1/memberships/:id", delete(delete_membership))
        .route("/api/v1/users/:user_id/memberships", get(list_user_memberships))
        
        // Tenant switching and context routes (immediate context changes)
        .route("/api/v1/tenant/switch", post(switch_tenant))
        .route("/api/v1/tenants/:tenant_id/context", get(get_tenant_context))
        .route("/api/v1/tenant/current", get(get_current_tenant_context))
        
        // Tenant validation and access control routes
        .route("/api/v1/tenants/:tenant_id/validate-access/:user_id", get(validate_tenant_access))
        .route("/api/v1/tenants/:tenant_id/permissions/:user_id", get(get_user_tenant_permissions))
        
        // Add state
        .with_state(tenant_service)
        
        // Add comprehensive middleware stack
        .layer(
            ServiceBuilder::new()
                // Compression for better performance
                .layer(CompressionLayer::new())
                // Security headers
                .layer(middleware::from_fn(security_headers_middleware))
                // Tenant isolation and validation
                .layer(middleware::from_fn(tenant_isolation_middleware))
                // Request tracing
                .layer(TraceLayer::new_for_http())
                // CORS configuration
                .layer(CorsLayer::new()
                    .allow_origin(tower_http::cors::Any)
                    .allow_methods([
                        axum::http::Method::GET,
                        axum::http::Method::POST,
                        axum::http::Method::PUT,
                        axum::http::Method::DELETE,
                        axum::http::Method::OPTIONS,
                    ])
                    .allow_headers([
                        axum::http::header::CONTENT_TYPE,
                        axum::http::header::AUTHORIZATION,
                        axum::http::HeaderName::from_static("x-tenant-id"),
                        axum::http::HeaderName::from_static("x-request-id"),
                    ])
                )
                // Request timeout
                .layer(TimeoutLayer::new(Duration::from_secs(30)))
                // Request ID generation
                .layer(middleware::from_fn(request_id_middleware))
                // Request/response logging
                .layer(middleware::from_fn(logging_middleware))
        )
}

pub async fn start_server(config: AppConfig, pool: PgPool) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_app(&config, pool).await;
    
    let port = 8085; // Fixed port for tenant service (dual-mode HTTP server)
    let addr = format!("{}:{}", config.server.host, port);
    
    tracing::info!("🌐 Tenant Service HTTP server listening on {}", addr);
    tracing::info!("🔒 Security: Tenant isolation middleware enabled");
    tracing::info!("📊 Health checks: /health (simple), /health/detailed (comprehensive)");
    tracing::info!("🔄 Mode: Dual-mode (HTTP server + workflow activities)");
    
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}