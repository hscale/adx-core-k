use axum::{
    extract::{Path, State, Extension},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde_json::{json, Value};

use crate::{AppState, middleware::{auth::Claims, tenant::TenantContext}};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/:user_id", get(get_user))
        .route("/:user_id/profile", get(get_user_profile))
        .route("/:user_id/dashboard", get(get_user_dashboard))
}

async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Extension(claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Check if user is requesting their own data or has admin role
    if user_id != claims.sub && !claims.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Try to get from cache first
    if let Ok(Some(cached_user)) = state.redis.get_cached_user(&user_id).await {
        return Ok(Json(cached_user));
    }

    // Get from API Gateway
    let token = ""; // In real implementation, extract from request
    match state.api_client.get_user(&user_id, token).await {
        Ok(user_data) => {
            // Cache the result
            let _ = state.redis.cache_user(&user_id, &user_data, 300).await;
            Ok(Json(user_data))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Extension(claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Check permissions
    if user_id != claims.sub && !claims.roles.contains(&"admin".to_string()) {
        return Err(StatusCode::FORBIDDEN);
    }

    // Try cache first
    if let Ok(Some(cached_profile)) = state.redis.get_cached_user_profile(&user_id).await {
        return Ok(Json(cached_profile));
    }

    // Get from API Gateway
    let token = ""; // In real implementation, extract from request
    match state.api_client.get_user_profile(&user_id, token).await {
        Ok(profile_data) => {
            // Cache the result
            let _ = state.redis.cache_user_profile(&user_id, &profile_data, 600).await;
            Ok(Json(profile_data))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_user_dashboard(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Extension(claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    // Check permissions
    if user_id != claims.sub {
        return Err(StatusCode::FORBIDDEN);
    }

    // Try cache first
    if let Ok(Some(cached_dashboard)) = state.redis.get_aggregated_dashboard(&user_id).await {
        return Ok(Json(cached_dashboard));
    }

    // Aggregate data from multiple sources
    let token = ""; // In real implementation, extract from request
    
    let user_data = state.api_client.get_user(&user_id, token).await.ok();
    let profile_data = state.api_client.get_user_profile(&user_id, token).await.ok();
    let tenants_data = state.api_client.get_user_tenants(&user_id, token).await.ok();
    let activity_data = state.api_client.get_user_activity(&user_id, token).await.ok();
    let workflows_data = state.temporal_client.get_user_workflows(&user_id).await.ok();

    let dashboard = json!({
        "user": user_data,
        "profile": profile_data,
        "tenants": tenants_data,
        "recent_activity": activity_data,
        "workflows": workflows_data,
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    // Cache the aggregated result
    let _ = state.redis.cache_aggregated_dashboard(&user_id, &dashboard, 300).await;

    Ok(Json(dashboard))
}