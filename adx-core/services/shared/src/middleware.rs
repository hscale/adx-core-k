use axum::{
    extract::Request,
    http::{HeaderMap, HeaderValue},
    middleware::Next,
    response::Response,
};
use uuid::Uuid;

// Request ID middleware - adds a unique request ID to each request
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Response {
    // Generate a unique request ID
    let request_id = Uuid::new_v4().to_string();
    
    // Add request ID to request headers
    request.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    // Process the request
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap(),
    );
    
    response
}

// Logging middleware - logs request and response information
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let request_id = request
        .headers()
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown")
        .to_string();
    
    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        "Processing request"
    );
    
    let start = std::time::Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed();
    
    tracing::info!(
        request_id = %request_id,
        method = %method,
        uri = %uri,
        status = %response.status(),
        duration_ms = %duration.as_millis(),
        "Request completed"
    );
    
    response
}