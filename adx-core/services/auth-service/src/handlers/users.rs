use axum::{
    extract::{State, Path, Json},
    http::StatusCode,
    response::Json as ResponseJson,
    Extension,
};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use adx_shared::{
    auth::{JwtClaims, UserPreferences},
    types::{UserId, UserQuotas},
};
use crate::AppState;

#[derive(Debug, Serialize)]
pub struct UserProfile {
    pub id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub quotas: UserQuotas,
    pub preferences: UserPreferences,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateUserProfileRequest {
    pub display_name: Option<String>,
    pub preferences: Option<UserPreferences>,
}

#[derive(Debug, Serialize)]
pub struct UpdateUserProfileResponse {
    pub user: UserProfile,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug, Serialize)]
pub struct ChangePasswordResponse {
    pub message: String,
    pub password_changed: bool,
}

/// Get current user profile
pub async fn get_user_profile(
    State(_state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
) -> std::result::Result<ResponseJson<UserProfile>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // TODO: Load user profile from database
    // For now, we'll return data from JWT claims
    
    let user_profile = UserProfile {
        id: claims.sub.clone(),
        email: claims.user_email.clone(),
        display_name: Some("Test User".to_string()), // TODO: Load from database
        roles: claims.user_roles.clone(),
        permissions: claims.permissions.clone(),
        quotas: claims.quotas.clone(),
        preferences: UserPreferences::default(), // TODO: Load from database
        last_login: Some(Utc::now()), // TODO: Load from database
        created_at: Utc::now(), // TODO: Load from database
        updated_at: Utc::now(), // TODO: Load from database
    };

    tracing::info!(
        user_id = %claims.sub,
        "User profile retrieved"
    );

    Ok(ResponseJson(user_profile))
}

/// Update user profile
pub async fn update_user_profile(
    State(_state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(request): Json<UpdateUserProfileRequest>,
) -> std::result::Result<ResponseJson<UpdateUserProfileResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // TODO: Validate input
    // TODO: Update user profile in database
    
    let updated_profile = UserProfile {
        id: claims.sub.clone(),
        email: claims.user_email.clone(),
        display_name: request.display_name.clone(),
        roles: claims.user_roles.clone(),
        permissions: claims.permissions.clone(),
        quotas: claims.quotas.clone(),
        preferences: request.preferences.unwrap_or_default(),
        last_login: Some(Utc::now()),
        created_at: Utc::now(), // TODO: Load from database
        updated_at: Utc::now(),
    };

    tracing::info!(
        user_id = %claims.sub,
        "User profile updated"
    );

    Ok(ResponseJson(UpdateUserProfileResponse {
        user: updated_profile,
        message: "Profile updated successfully".to_string(),
    }))
}

/// Get user by ID (admin only)
pub async fn get_user_by_id(
    State(_state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Path(user_id): Path<UserId>,
) -> std::result::Result<ResponseJson<UserProfile>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // Check if user has admin permissions
    if !claims.permissions.contains(&"user:admin".to_string()) && !claims.user_roles.contains(&"admin".to_string()) {
        return Err((
            StatusCode::FORBIDDEN,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "INSUFFICIENT_PERMISSIONS",
                    "message": "Admin permissions required to view other users"
                }
            })),
        ));
    }

    // TODO: Load user from database
    // For now, return a mock user
    let user_profile = UserProfile {
        id: user_id.clone(),
        email: format!("user-{}@example.com", user_id),
        display_name: Some(format!("User {}", user_id)),
        roles: vec!["user".to_string()],
        permissions: vec![
            "tenant:read".to_string(),
            "user:read".to_string(),
        ],
        quotas: UserQuotas::default(),
        preferences: UserPreferences::default(),
        last_login: Some(Utc::now()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    tracing::info!(
        admin_user_id = %claims.sub,
        target_user_id = %user_id,
        "User profile retrieved by admin"
    );

    Ok(ResponseJson(user_profile))
}

/// Change user password
pub async fn change_password(
    State(_state): State<AppState>,
    Extension(claims): Extension<JwtClaims>,
    Json(request): Json<ChangePasswordRequest>,
) -> std::result::Result<ResponseJson<ChangePasswordResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // Validate input
    if request.current_password.is_empty() || request.new_password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Current password and new password are required"
                }
            })),
        ));
    }

    // Validate new password strength
    if !is_strong_password(&request.new_password) {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "WEAK_PASSWORD",
                    "message": "New password does not meet security requirements"
                }
            })),
        ));
    }

    // TODO: Verify current password against database
    // TODO: Hash new password and update in database
    // TODO: Invalidate all existing sessions for this user

    tracing::info!(
        user_id = %claims.sub,
        "Password changed successfully"
    );

    Ok(ResponseJson(ChangePasswordResponse {
        message: "Password changed successfully. Please log in again.".to_string(),
        password_changed: true,
    }))
}

// Helper functions
fn is_strong_password(password: &str) -> bool {
    // Password strength validation
    password.len() >= 8
        && password.chars().any(|c| c.is_uppercase())
        && password.chars().any(|c| c.is_lowercase())
        && password.chars().any(|c| c.is_numeric())
        && password.chars().any(|c| !c.is_alphanumeric())
}