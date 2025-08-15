use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, put},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::{
    middleware::{
        auth::{has_permission, Claims},
        error_handler::{BffError, BffResult},
        tenant::{get_tenant_context},
    },
    types::{
        ApiResponse, PaginationParams, ResponseMeta, User,
        CreateUserRequest, UpdateUserRequest, UpdateUserProfileRequest, UpdateUserPreferencesRequest,
    },
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_users))
        .route("/:user_id", get(get_user).delete(delete_user))
        .route("/:user_id/profile", get(get_user_profile))
        .route("/:user_id/preferences", get(get_user_preferences))
        .route("/:user_id/activity", get(get_user_activity))
        .route("/:user_id/sessions", get(get_user_sessions))
        .route("/:user_id/sessions/:session_id", delete(revoke_user_session))
        .route("/stats", get(get_user_stats))
        .route("/search", get(search_users))
}

#[derive(Debug, Deserialize, Serialize)]
struct UserListQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    search: Option<String>,
    role: Option<String>,
    is_active: Option<bool>,
    created_after: Option<String>,
    created_before: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct UserActivityQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    activity_type: Option<String>,
    from_date: Option<String>,
    to_date: Option<String>,
}

#[derive(Debug, Deserialize)]
struct UserSearchQuery {
    q: String,
    #[serde(flatten)]
    pagination: PaginationParams,
}

// List users with caching and filtering
async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserListQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to list users"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Create cache key based on query parameters
    let params_hash = create_params_hash(&query)?;
    
    // Try to get from cache first
    if let Ok(Some(cached_users)) = state.redis.get_cached_user_list(tenant_id, &params_hash).await {
        debug!("Returning cached user list for tenant: {}", tenant_id);
        return Ok(Json(ApiResponse {
            data: cached_users,
            meta: Some(ResponseMeta {
                total: None,
                page: query.pagination.page,
                per_page: query.pagination.per_page,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Build query parameters for API call
    let mut params = HashMap::new();
    if let Some(page) = query.pagination.page {
        params.insert("page".to_string(), page.to_string());
    }
    if let Some(per_page) = query.pagination.per_page {
        params.insert("per_page".to_string(), per_page.to_string());
    }
    if let Some(search) = &query.search {
        params.insert("search".to_string(), search.clone());
    }
    if let Some(role) = &query.role {
        params.insert("role".to_string(), role.clone());
    }
    if let Some(is_active) = query.is_active {
        params.insert("is_active".to_string(), is_active.to_string());
    }
    if let Some(created_after) = &query.created_after {
        params.insert("created_after".to_string(), created_after.clone());
    }
    if let Some(created_before) = &query.created_before {
        params.insert("created_before".to_string(), created_before.clone());
    }

    // Fetch from API Gateway
    let users = state.api_client.list_users(tenant_id, &auth_token, Some(params)).await?;

    // Cache the result
    if let Err(e) = state.redis.cache_user_list(tenant_id, &params_hash, &users, Some(300)).await {
        error!("Failed to cache user list: {}", e);
    }

    info!("Listed users for tenant: {} (query: {:?})", tenant_id, query);

    Ok(Json(ApiResponse {
        data: users,
        meta: Some(ResponseMeta {
            total: None,
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get single user with caching
async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions (users can read their own data, or need user:read permission)
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to read user"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Try cache first
    if let Ok(Some(cached_user)) = state.redis.get_cached_user(&user_id).await {
        debug!("Returning cached user: {}", user_id);
        return Ok(Json(ApiResponse {
            data: serde_json::to_value(cached_user)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(600),
            }),
        }));
    }

    // Fetch from API Gateway
    let user_data = state.api_client.get_user(&user_id, tenant_id, &auth_token).await?;

    // Cache the result if it's a valid user object
    if let Ok(user) = serde_json::from_value::<User>(user_data.clone()) {
        if let Err(e) = state.redis.cache_user(&user_id, &user, Some(600)).await {
            error!("Failed to cache user: {}", e);
        }
    }

    info!("Retrieved user: {} for tenant: {}", user_id, tenant_id);

    Ok(Json(ApiResponse {
        data: user_data,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Create user
async fn create_user(
    State(state): State<AppState>,
    Json(create_request): Json<CreateUserRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "user:write") {
        return Err(BffError::authorization("Insufficient permissions to create user"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Convert to JSON for API call
    let user_data = serde_json::to_value(&create_request)?;

    // Create user via API Gateway
    let created_user = state.api_client.create_user(tenant_id, &auth_token, &user_data).await?;

    info!("Created user for tenant: {} (email: {})", tenant_id, create_request.email);

    Ok(Json(ApiResponse {
        data: created_user,
        meta: None,
    }))
}

// Update user
async fn update_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(update_request): Json<UpdateUserRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions (users can update their own data, or need user:write permission)
    if claims.sub != user_id && !has_permission(claims, "user:write") {
        return Err(BffError::authorization("Insufficient permissions to update user"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Convert to JSON for API call
    let user_data = serde_json::to_value(&update_request)?;

    // Update user via API Gateway
    let updated_user = state.api_client.update_user(&user_id, tenant_id, &auth_token, &user_data).await?;

    // Invalidate cache
    if let Err(e) = state.redis.invalidate_user_cache(&user_id).await {
        error!("Failed to invalidate user cache: {}", e);
    }

    info!("Updated user: {} for tenant: {}", user_id, tenant_id);

    Ok(Json(ApiResponse {
        data: updated_user,
        meta: None,
    }))
}

// Delete user
async fn delete_user(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "user:delete") {
        return Err(BffError::authorization("Insufficient permissions to delete user"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Delete user via API Gateway
    state.api_client.delete_user(&user_id, tenant_id, &auth_token).await?;

    // Invalidate cache
    if let Err(e) = state.redis.invalidate_user_cache(&user_id).await {
        error!("Failed to invalidate user cache: {}", e);
    }

    info!("Deleted user: {} for tenant: {}", user_id, tenant_id);

    Ok(StatusCode::NO_CONTENT)
}

// Get user profile
async fn get_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to read user profile"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Try cache first
    if let Ok(Some(cached_profile)) = state.redis.get_cached_user_profile(&user_id).await {
        debug!("Returning cached user profile: {}", user_id);
        return Ok(Json(ApiResponse {
            data: serde_json::to_value(cached_profile)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(600),
            }),
        }));
    }

    // Fetch from API Gateway
    let profile_data = state.api_client.get_user_profile(&user_id, tenant_id, &auth_token).await?;

    // Cache the result
    if let Ok(profile) = serde_json::from_value(profile_data.clone()) {
        if let Err(e) = state.redis.cache_user_profile(&user_id, &profile, Some(600)).await {
            error!("Failed to cache user profile: {}", e);
        }
    }

    Ok(Json(ApiResponse {
        data: profile_data,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Update user profile
async fn update_user_profile(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(profile_request): Json<UpdateUserProfileRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:write") {
        return Err(BffError::authorization("Insufficient permissions to update user profile"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Convert to JSON for API call
    let profile_data = serde_json::to_value(&profile_request)?;

    // Update profile via API Gateway
    let updated_profile = state.api_client.update_user_profile(&user_id, tenant_id, &auth_token, &profile_data).await?;

    // Invalidate cache
    if let Err(e) = state.redis.invalidate_user_cache(&user_id).await {
        error!("Failed to invalidate user cache: {}", e);
    }

    Ok(Json(ApiResponse {
        data: updated_profile,
        meta: None,
    }))
}

// Get user preferences
async fn get_user_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to read user preferences"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Try cache first
    if let Ok(Some(cached_preferences)) = state.redis.get_cached_user_preferences(&user_id).await {
        debug!("Returning cached user preferences: {}", user_id);
        return Ok(Json(ApiResponse {
            data: serde_json::to_value(cached_preferences)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(600),
            }),
        }));
    }

    // Fetch from API Gateway
    let preferences_data = state.api_client.get_user_preferences(&user_id, tenant_id, &auth_token).await?;

    // Cache the result
    if let Ok(preferences) = serde_json::from_value(preferences_data.clone()) {
        if let Err(e) = state.redis.cache_user_preferences(&user_id, &preferences, Some(600)).await {
            error!("Failed to cache user preferences: {}", e);
        }
    }

    Ok(Json(ApiResponse {
        data: preferences_data,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Update user preferences
async fn update_user_preferences(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(preferences_request): Json<UpdateUserPreferencesRequest>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:write") {
        return Err(BffError::authorization("Insufficient permissions to update user preferences"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Convert to JSON for API call
    let preferences_data = serde_json::to_value(&preferences_request)?;

    // Update preferences via API Gateway
    let updated_preferences = state.api_client.update_user_preferences(&user_id, tenant_id, &auth_token, &preferences_data).await?;

    // Invalidate cache
    if let Err(e) = state.redis.invalidate_user_cache(&user_id).await {
        error!("Failed to invalidate user cache: {}", e);
    }

    Ok(Json(ApiResponse {
        data: updated_preferences,
        meta: None,
    }))
}

// Get user activity
async fn get_user_activity(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<UserActivityQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to read user activity"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Create cache key based on query parameters
    let params_hash = create_params_hash(&query)?;
    
    // Try cache first
    if let Ok(Some(cached_activity)) = state.redis.get_cached_user_activity(&user_id, &params_hash).await {
        debug!("Returning cached user activity: {}", user_id);
        return Ok(Json(ApiResponse {
            data: cached_activity,
            meta: Some(ResponseMeta {
                total: None,
                page: query.pagination.page,
                per_page: query.pagination.per_page,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Build query parameters for API call
    let mut params = HashMap::new();
    if let Some(page) = query.pagination.page {
        params.insert("page".to_string(), page.to_string());
    }
    if let Some(per_page) = query.pagination.per_page {
        params.insert("per_page".to_string(), per_page.to_string());
    }
    if let Some(activity_type) = &query.activity_type {
        params.insert("activity_type".to_string(), activity_type.clone());
    }
    if let Some(from_date) = &query.from_date {
        params.insert("from_date".to_string(), from_date.clone());
    }
    if let Some(to_date) = &query.to_date {
        params.insert("to_date".to_string(), to_date.clone());
    }

    // Fetch from API Gateway
    let activity_data = state.api_client.get_user_activity(&user_id, tenant_id, &auth_token, Some(params)).await?;

    // Cache the result
    if let Err(e) = state.redis.cache_user_activity(&user_id, &params_hash, &activity_data, Some(300)).await {
        error!("Failed to cache user activity: {}", e);
    }

    Ok(Json(ApiResponse {
        data: activity_data,
        meta: Some(ResponseMeta {
            total: None,
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get user sessions
async fn get_user_sessions(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to read user sessions"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Try cache first
    if let Ok(Some(cached_sessions)) = state.redis.get_cached_user_sessions(&user_id).await {
        debug!("Returning cached user sessions: {}", user_id);
        return Ok(Json(ApiResponse {
            data: cached_sessions,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Fetch from API Gateway
    let sessions_data = state.api_client.get_user_sessions(&user_id, tenant_id, &auth_token).await?;

    // Cache the result
    if let Err(e) = state.redis.cache_user_sessions(&user_id, &sessions_data, Some(300)).await {
        error!("Failed to cache user sessions: {}", e);
    }

    Ok(Json(ApiResponse {
        data: sessions_data,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Revoke user session
async fn revoke_user_session(
    State(state): State<AppState>,
    Path((user_id, session_id)): Path<(String, String)>,
    request: Request,
) -> BffResult<StatusCode> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:write") {
        return Err(BffError::authorization("Insufficient permissions to revoke user session"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Revoke session via API Gateway
    state.api_client.revoke_user_session(&user_id, &session_id, tenant_id, &auth_token).await?;

    // Invalidate sessions cache
    let sessions_key = format!("user_sessions:{}", user_id);
    if let Err(e) = state.redis.delete(&sessions_key).await {
        error!("Failed to invalidate user sessions cache: {}", e);
    }

    info!("Revoked session: {} for user: {}", session_id, user_id);

    Ok(StatusCode::NO_CONTENT)
}

// Get user stats (admin only)
async fn get_user_stats(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "user:admin") {
        return Err(BffError::authorization("Insufficient permissions to view user stats"));
    }

    // Mock user stats (in real implementation, this would aggregate from multiple sources)
    let stats = serde_json::json!({
        "total_users": 1250,
        "active_users": 1100,
        "new_users_today": 15,
        "new_users_this_week": 87,
        "new_users_this_month": 342,
        "user_activity_summary": {
            "login": 2340,
            "profile_update": 156,
            "password_change": 23,
            "session_created": 2340
        }
    });

    Ok(Json(ApiResponse {
        data: stats,
        meta: None,
    }))
}

// Search users
async fn search_users(
    State(state): State<AppState>,
    Query(query): Query<UserSearchQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to search users"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Build query parameters for API call
    let mut params = HashMap::new();
    params.insert("search".to_string(), query.q.clone());
    if let Some(page) = query.pagination.page {
        params.insert("page".to_string(), page.to_string());
    }
    if let Some(per_page) = query.pagination.per_page {
        params.insert("per_page".to_string(), per_page.to_string());
    }

    // Search via API Gateway
    let search_results = state.api_client.list_users(tenant_id, &auth_token, Some(params)).await?;

    Ok(Json(ApiResponse {
        data: search_results,
        meta: Some(ResponseMeta {
            total: None,
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Helper functions
fn extract_auth_token(request: &Request) -> BffResult<String> {
    let auth_header = request.headers()
        .get("authorization")
        .ok_or_else(|| BffError::authentication("Missing authorization header"))?;
    
    let auth_str = auth_header.to_str()
        .map_err(|_| BffError::authentication("Invalid authorization header"))?;
    
    if auth_str.starts_with("Bearer ") {
        Ok(auth_str[7..].to_string())
    } else {
        Err(BffError::authentication("Invalid authorization header format"))
    }
}

fn create_params_hash<T: Serialize>(params: &T) -> BffResult<String> {
    let params_json = serde_json::to_string(params)?;
    let hash = format!("{:x}", md5::compute(params_json.as_bytes()));
    Ok(hash)
}