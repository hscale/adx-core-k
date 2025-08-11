use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use std::time::Instant;
use uuid::Uuid;

use adx_shared::auth::JwtClaims;

/// Request logging middleware
pub async fn request_logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let start = Instant::now();
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = Uuid::new_v4().to_string();
    
    // Extract user info if available
    let user_info = request.extensions().get::<JwtClaims>()
        .map(|claims| format!("user:{} tenant:{}", claims.sub, claims.tenant_id))
        .unwrap_or_else(|| "anonymous".to_string());
    
    // Extract matched path if available
    let path = uri.path();
    
    tracing::info!(
        request_id = %request_id,
        method = %method,
        path = %path,
        user_info = %user_info,
        "Request started"
    );
    
    // Add request ID to request extensions
    let mut request = request;
    request.extensions_mut().insert(RequestId(request_id.clone()));
    
    // Process request
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    // Log response
    if status.is_server_error() {
        tracing::error!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            user_info = %user_info,
            "Request completed"
        );
    } else if status.is_client_error() {
        tracing::warn!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            user_info = %user_info,
            "Request completed"
        );
    } else {
        tracing::info!(
            request_id = %request_id,
            method = %method,
            path = %path,
            status = %status.as_u16(),
            duration_ms = %duration.as_millis(),
            user_info = %user_info,
            "Request completed"
        );
    }
    
    // Add request ID to response headers
    let mut response = response;
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.parse().unwrap(),
    );
    
    response
}

/// CORS middleware
pub async fn cors_middleware(
    request: Request,
    next: Next,
) -> Response {
    let origin = request.headers().get("origin")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("*")
        .to_string();
    
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    headers.insert("Access-Control-Allow-Origin", origin.parse().unwrap());
    headers.insert("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS".parse().unwrap());
    headers.insert("Access-Control-Allow-Headers", "Content-Type, Authorization, X-Tenant-ID, X-Request-ID".parse().unwrap());
    headers.insert("Access-Control-Expose-Headers", "X-Request-ID, X-RateLimit-Remaining, X-RateLimit-Reset".parse().unwrap());
    headers.insert("Access-Control-Max-Age", "86400".parse().unwrap());
    
    response
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Response {
    let mut response = next.run(request).await;
    
    let headers = response.headers_mut();
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    headers.insert("Content-Security-Policy", "default-src 'self'".parse().unwrap());
    
    response
}

#[derive(Debug, Clone)]
pub struct RequestId(pub String);

impl RequestId {
    pub fn get(&self) -> &str {
        &self.0
    }
}