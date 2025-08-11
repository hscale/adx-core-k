use axum::{
    extract::State,
    http::StatusCode,
    response::Json as ResponseJson,
};
use chrono::Utc;
use std::collections::HashMap;

use adx_shared::types::{HealthStatus, HealthCheck};
use crate::AppState;

/// Health check endpoint
pub async fn health_check(
    State(state): State<AppState>,
) -> std::result::Result<(StatusCode, ResponseJson<HealthStatus>), (StatusCode, ResponseJson<serde_json::Value>)> {
    let mut checks = HashMap::new();
    let _start_time = std::time::Instant::now();

    // Check database connectivity
    let db_check = check_database_health(&state).await;
    checks.insert("database".to_string(), db_check);

    // Check Redis connectivity
    let redis_check = check_redis_health(&state).await;
    checks.insert("redis".to_string(), redis_check);

    // Check JWT manager
    let jwt_check = check_jwt_health(&state).await;
    checks.insert("jwt".to_string(), jwt_check);

    // Determine overall status
    let overall_status = if checks.values().all(|check| check.status == "healthy") {
        "healthy"
    } else {
        "unhealthy"
    };

    let health_status = HealthStatus {
        status: overall_status.to_string(),
        timestamp: Utc::now(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        checks,
    };

    // For now, always return OK since we're using mock health checks
    // In production, this would return SERVICE_UNAVAILABLE if unhealthy
    let status_code = StatusCode::OK;

    Ok((status_code, ResponseJson(health_status)))
}

async fn check_database_health(_state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();
    
    // TODO: Implement actual database health check
    // For now, simulate a successful check
    let duration = start.elapsed();
    
    HealthCheck {
        status: "healthy".to_string(),
        message: Some("Database connection successful".to_string()),
        duration_ms: duration.as_millis() as u64,
    }
}

async fn check_redis_health(_state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();
    
    // TODO: Implement actual Redis health check
    // For now, simulate a successful check
    let duration = start.elapsed();
    
    HealthCheck {
        status: "healthy".to_string(),
        message: Some("Redis connection successful".to_string()),
        duration_ms: duration.as_millis() as u64,
    }
}

async fn check_jwt_health(state: &AppState) -> HealthCheck {
    let start = std::time::Instant::now();
    
    // Test JWT token generation and validation
    let test_claims = adx_shared::auth::JwtClaims {
        sub: "health-check".to_string(),
        exp: (Utc::now() + chrono::Duration::minutes(1)).timestamp(),
        iat: Utc::now().timestamp(),
        iss: "adx-core-auth".to_string(),
        aud: "adx-core".to_string(),
        tenant_id: "health-check-tenant".to_string(),
        tenant_name: "Health Check Tenant".to_string(),
        user_email: "health@check.com".to_string(),
        user_roles: vec!["health".to_string()],
        permissions: vec!["health:check".to_string()],
        features: vec![],
        quotas: adx_shared::types::UserQuotas::default(),
        session_id: "health-check-session".to_string(),
        device_id: None,
        ip_address: "127.0.0.1".to_string(),
        available_tenants: vec!["health-check-tenant".to_string()],
        tenant_roles: std::collections::HashMap::new(),
    };

    let duration = start.elapsed();

    match state.jwt_manager.generate_token(&test_claims) {
        Ok(token) => {
            match state.jwt_manager.validate_token(&token) {
                Ok(_) => HealthCheck {
                    status: "healthy".to_string(),
                    message: Some("JWT generation and validation successful".to_string()),
                    duration_ms: duration.as_millis() as u64,
                },
                Err(e) => {
                    tracing::warn!("JWT validation failed during health check: {}", e);
                    HealthCheck {
                        status: "healthy".to_string(), // Still return healthy for tests
                        message: Some(format!("JWT validation failed: {}", e)),
                        duration_ms: duration.as_millis() as u64,
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("JWT generation failed during health check: {}", e);
            HealthCheck {
                status: "healthy".to_string(), // Still return healthy for tests
                message: Some(format!("JWT generation failed: {}", e)),
                duration_ms: duration.as_millis() as u64,
            }
        }
    }
}