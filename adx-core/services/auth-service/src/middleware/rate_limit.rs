use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

use adx_shared::{
    auth::JwtClaims,
    types::TenantId,
};
use crate::AppState;

#[derive(Debug, Clone)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub burst_limit: u32,
}

impl Default for RateLimits {
    fn default() -> Self {
        Self {
            requests_per_minute: 60,
            requests_per_hour: 1000,
            burst_limit: 10,
        }
    }
}

#[derive(Debug)]
pub struct RateLimitEntry {
    pub count: u32,
    pub window_start: Instant,
    pub burst_count: u32,
    pub last_request: Instant,
}

#[derive(Clone)]
pub struct RateLimiter {
    // In-memory rate limiting - in production, use Redis
    minute_windows: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    hour_windows: Arc<RwLock<HashMap<String, RateLimitEntry>>>,
    default_limits: RateLimits,
    tenant_limits: Arc<RwLock<HashMap<TenantId, RateLimits>>>,
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            minute_windows: Arc::new(RwLock::new(HashMap::new())),
            hour_windows: Arc::new(RwLock::new(HashMap::new())),
            default_limits: RateLimits::default(),
            tenant_limits: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn check_rate_limit(
        &self,
        key: &str,
        limits: &RateLimits,
    ) -> std::result::Result<RateLimitResult, RateLimitError> {
        let now = Instant::now();

        // Check minute window
        let minute_result = self.check_window(
            &self.minute_windows,
            key,
            limits.requests_per_minute,
            Duration::from_secs(60),
            now,
        )?;

        if !minute_result.allowed {
            return Ok(minute_result);
        }

        // Check hour window
        let hour_result = self.check_window(
            &self.hour_windows,
            key,
            limits.requests_per_hour,
            Duration::from_secs(3600),
            now,
        )?;

        if !hour_result.allowed {
            return Ok(hour_result);
        }

        // Check burst limit
        let burst_result = self.check_burst_limit(key, limits.burst_limit, now)?;

        Ok(burst_result)
    }

    fn check_window(
        &self,
        windows: &Arc<RwLock<HashMap<String, RateLimitEntry>>>,
        key: &str,
        limit: u32,
        window_duration: Duration,
        now: Instant,
    ) -> std::result::Result<RateLimitResult, RateLimitError> {
        let mut windows_guard = windows.write().unwrap();

        let entry = windows_guard.entry(key.to_string()).or_insert(RateLimitEntry {
            count: 0,
            window_start: now,
            burst_count: 0,
            last_request: now,
        });

        // Reset window if expired
        if now.duration_since(entry.window_start) >= window_duration {
            entry.count = 0;
            entry.window_start = now;
        }

        // Check if limit exceeded
        if entry.count >= limit {
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_time: entry.window_start + window_duration,
                retry_after: (entry.window_start + window_duration).saturating_duration_since(now),
            });
        }

        // Increment counter
        entry.count += 1;
        entry.last_request = now;

        Ok(RateLimitResult {
            allowed: true,
            remaining: limit - entry.count,
            reset_time: entry.window_start + window_duration,
            retry_after: Duration::from_secs(0),
        })
    }

    fn check_burst_limit(
        &self,
        key: &str,
        burst_limit: u32,
        now: Instant,
    ) -> std::result::Result<RateLimitResult, RateLimitError> {
        let mut windows_guard = self.minute_windows.write().unwrap();

        let entry = windows_guard.get_mut(key).unwrap();

        // Reset burst counter if more than 1 second since last request
        if now.duration_since(entry.last_request) >= Duration::from_secs(1) {
            entry.burst_count = 0;
        }

        // Check burst limit
        if entry.burst_count >= burst_limit {
            return Ok(RateLimitResult {
                allowed: false,
                remaining: 0,
                reset_time: entry.last_request + Duration::from_secs(1),
                retry_after: Duration::from_secs(1),
            });
        }

        // Increment burst counter
        entry.burst_count += 1;

        Ok(RateLimitResult {
            allowed: true,
            remaining: burst_limit - entry.burst_count,
            reset_time: entry.last_request + Duration::from_secs(1),
            retry_after: Duration::from_secs(0),
        })
    }

    pub fn get_tenant_limits(&self, tenant_id: &str) -> RateLimits {
        self.tenant_limits
            .read()
            .unwrap()
            .get(tenant_id)
            .cloned()
            .unwrap_or_else(|| self.default_limits.clone())
    }
}

#[derive(Debug)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub remaining: u32,
    pub reset_time: Instant,
    pub retry_after: Duration,
}

#[derive(Debug, thiserror::Error)]
pub enum RateLimitError {
    #[error("Lock error")]
    LockError,
    #[error("Entry not found")]
    EntryNotFound,
    #[error("Internal error: {0}")]
    Internal(String),
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    // Create rate limiter key based on user/tenant/IP
    let rate_limit_key = create_rate_limit_key(&request);
    
    // Get rate limits (tenant-specific or default)
    let limits = get_rate_limits_for_request(&request);
    
    // Check rate limit
    match state.rate_limiter.check_rate_limit(&rate_limit_key, &limits) {
        Ok(result) => {
            if !result.allowed {
                // Add rate limit headers
                let mut response = Response::new(axum::body::Body::from(
                    serde_json::json!({
                        "error": {
                            "code": "RATE_LIMIT_EXCEEDED",
                            "message": "Rate limit exceeded",
                            "retry_after": result.retry_after.as_secs()
                        }
                    }).to_string()
                ));
                
                *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
                response.headers_mut().insert(
                    "X-RateLimit-Remaining",
                    result.remaining.to_string().parse().unwrap(),
                );
                response.headers_mut().insert(
                    "X-RateLimit-Reset",
                    result.reset_time.elapsed().as_secs().to_string().parse().unwrap(),
                );
                response.headers_mut().insert(
                    "Retry-After",
                    result.retry_after.as_secs().to_string().parse().unwrap(),
                );
                
                return Ok(response);
            }
            
            // Add rate limit headers to successful response
            let mut response = next.run(request).await;
            response.headers_mut().insert(
                "X-RateLimit-Remaining",
                result.remaining.to_string().parse().unwrap(),
            );
            
            Ok(response)
        }
        Err(_) => {
            // Log error and allow request to proceed
            tracing::warn!("Rate limiting error, allowing request");
            Ok(next.run(request).await)
        }
    }
}

fn create_rate_limit_key(request: &Request) -> String {
    // Try to get user ID from JWT claims
    if let Some(claims) = request.extensions().get::<JwtClaims>() {
        return format!("user:{}:tenant:{}", claims.sub, claims.tenant_id);
    }
    
    // Fall back to IP address
    let ip = extract_client_ip(request);
    format!("ip:{}", ip)
}

fn get_rate_limits_for_request(request: &Request) -> RateLimits {
    // Get tenant-specific limits if available
    if let Some(_claims) = request.extensions().get::<JwtClaims>() {
        // TODO: Load tenant-specific rate limits from database
        // For now, return default limits
        return RateLimits::default();
    }
    
    // Return more restrictive limits for unauthenticated requests
    RateLimits {
        requests_per_minute: 20,
        requests_per_hour: 100,
        burst_limit: 5,
    }
}

fn extract_client_ip(request: &Request) -> String {
    // Try various headers for client IP
    let headers = request.headers();
    
    if let Some(forwarded_for) = headers.get("X-Forwarded-For") {
        if let Ok(ip_str) = forwarded_for.to_str() {
            // Take the first IP in the chain
            if let Some(first_ip) = ip_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    if let Some(real_ip) = headers.get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Fall back to connection info (not available in this context)
    "unknown".to_string()
}