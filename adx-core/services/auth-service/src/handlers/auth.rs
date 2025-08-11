use axum::{
    extract::{State, Json},
    http::StatusCode,
    response::Json as ResponseJson,
};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use uuid::Uuid;
use bcrypt::{hash, verify, DEFAULT_COST};
use std::collections::HashMap;

use adx_shared::{
    auth::JwtClaims,
    types::{UserId, TenantId, UserQuotas, SubscriptionTier},
};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
    pub display_name: Option<String>,
    pub tenant_name: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub tenant_id: Option<TenantId>,
    pub device_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PasswordResetRequest {
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user: UserInfo,
    pub tenant: TenantInfo,
}

#[derive(Debug, Serialize)]
pub struct UserInfo {
    pub id: UserId,
    pub email: String,
    pub display_name: Option<String>,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct TenantInfo {
    pub id: TenantId,
    pub name: String,
    pub subscription_tier: SubscriptionTier,
    pub features: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    pub user_id: UserId,
    pub tenant_id: Option<TenantId>,
    pub message: String,
    pub verification_required: bool,
}

#[derive(Debug, Serialize)]
pub struct PasswordResetResponse {
    pub message: String,
    pub reset_token_sent: bool,
}

/// Register a new user
pub async fn register(
    State(_state): State<AppState>,
    Json(request): Json<RegisterRequest>,
) -> std::result::Result<ResponseJson<RegisterResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // Validate input
    if request.email.is_empty() || request.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Email and password are required",
                    "validation_errors": [
                        {
                            "field": "email",
                            "code": "REQUIRED",
                            "message": "Email is required"
                        },
                        {
                            "field": "password",
                            "code": "REQUIRED",
                            "message": "Password is required"
                        }
                    ]
                }
            })),
        ));
    }

    // Validate email format
    if !is_valid_email(&request.email) {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Invalid email format",
                    "validation_errors": [
                        {
                            "field": "email",
                            "code": "INVALID_FORMAT",
                            "message": "Email address format is invalid",
                            "rejected_value": request.email
                        }
                    ]
                }
            })),
        ));
    }

    // Validate password strength
    if !is_strong_password(&request.password) {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Password does not meet security requirements",
                    "validation_errors": [
                        {
                            "field": "password",
                            "code": "WEAK_PASSWORD",
                            "message": "Password must be at least 8 characters long and contain uppercase, lowercase, number, and special character"
                        }
                    ]
                }
            })),
        ));
    }

    // Check if user already exists
    // TODO: This would query the database in a real implementation
    // For now, we'll simulate the check
    
    // Hash password
    let _password_hash = match hash(&request.password, DEFAULT_COST) {
        Ok(hash) => hash,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": {
                        "code": "INTERNAL_ERROR",
                        "message": "Failed to process password"
                    }
                })),
            ));
        }
    };

    // Generate user ID
    let user_id = Uuid::new_v4().to_string();
    
    // TODO: Save user to database
    // TODO: Create default tenant if tenant_name is provided
    // TODO: Send verification email
    
    tracing::info!(
        user_id = %user_id,
        email = %request.email,
        "User registration initiated"
    );

    Ok(ResponseJson(RegisterResponse {
        user_id,
        tenant_id: None, // Would be set if tenant was created
        message: "Registration successful. Please check your email for verification.".to_string(),
        verification_required: true,
    }))
}

/// Login user
pub async fn login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> std::result::Result<ResponseJson<AuthResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // Validate input
    if request.email.is_empty() || request.password.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Email and password are required"
                }
            })),
        ));
    }

    // TODO: Query user from database
    // For now, we'll simulate a user lookup
    let user_id = Uuid::new_v4().to_string();
    let stored_password_hash = hash("password123", DEFAULT_COST).unwrap(); // Simulated stored hash
    
    // Verify password
    let password_valid = match verify(&request.password, &stored_password_hash) {
        Ok(valid) => valid,
        Err(_) => false,
    };

    if !password_valid {
        return Err((
            StatusCode::UNAUTHORIZED,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "INVALID_CREDENTIALS",
                    "message": "Invalid email or password"
                }
            })),
        ));
    }

    // TODO: Load user's tenants and determine active tenant
    let tenant_id = request.tenant_id.unwrap_or_else(|| "default-tenant".to_string());
    
    // Generate session ID
    let session_id = Uuid::new_v4().to_string();
    
    // Create JWT claims
    let now = Utc::now();
    let expires_in = 3600; // 1 hour
    let exp = now + Duration::seconds(expires_in);
    
    let claims = JwtClaims {
        sub: user_id.clone(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
        iss: "adx-core-auth".to_string(),
        aud: "adx-core".to_string(),
        tenant_id: tenant_id.clone(),
        tenant_name: "Default Tenant".to_string(),
        user_email: request.email.clone(),
        user_roles: vec!["user".to_string()],
        permissions: vec![
            "tenant:read".to_string(),
            "user:read".to_string(),
            "file:read".to_string(),
            "file:write".to_string(),
        ],
        features: vec!["basic_features".to_string()],
        quotas: UserQuotas::default(),
        session_id: session_id.clone(),
        device_id: request.device_id.clone(),
        ip_address: "127.0.0.1".to_string(), // TODO: Extract from request
        available_tenants: vec![tenant_id.clone()],
        tenant_roles: {
            let mut roles = HashMap::new();
            roles.insert(tenant_id.clone(), vec!["user".to_string()]);
            roles
        },
    };

    // Generate JWT token
    let token = match state.jwt_manager.generate_token(&claims) {
        Ok(token) => token,
        Err(_) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                ResponseJson(serde_json::json!({
                    "error": {
                        "code": "TOKEN_GENERATION_FAILED",
                        "message": "Failed to generate authentication token"
                    }
                })),
            ));
        }
    };

    // TODO: Generate refresh token
    let refresh_token = Uuid::new_v4().to_string();
    
    // TODO: Store session in database/Redis
    
    tracing::info!(
        user_id = %user_id,
        tenant_id = %tenant_id,
        session_id = %session_id,
        "User login successful"
    );

    Ok(ResponseJson(AuthResponse {
        token,
        refresh_token,
        expires_in,
        user: UserInfo {
            id: user_id,
            email: request.email,
            display_name: Some("Test User".to_string()),
            roles: vec!["user".to_string()],
            permissions: vec![
                "tenant:read".to_string(),
                "user:read".to_string(),
                "file:read".to_string(),
                "file:write".to_string(),
            ],
        },
        tenant: TenantInfo {
            id: tenant_id,
            name: "Default Tenant".to_string(),
            subscription_tier: SubscriptionTier::Professional,
            features: vec!["basic_features".to_string()],
        },
    }))
}

/// Request password reset
pub async fn request_password_reset(
    State(_state): State<AppState>,
    Json(request): Json<PasswordResetRequest>,
) -> std::result::Result<ResponseJson<PasswordResetResponse>, (StatusCode, ResponseJson<serde_json::Value>)> {
    // Validate email format
    if !is_valid_email(&request.email) {
        return Err((
            StatusCode::BAD_REQUEST,
            ResponseJson(serde_json::json!({
                "error": {
                    "code": "VALIDATION_FAILED",
                    "message": "Invalid email format"
                }
            })),
        ));
    }

    // TODO: Check if user exists in database
    // TODO: Generate password reset token
    // TODO: Send password reset email
    
    tracing::info!(
        email = %request.email,
        "Password reset requested"
    );

    // Always return success to prevent email enumeration
    Ok(ResponseJson(PasswordResetResponse {
        message: "If an account with this email exists, a password reset link has been sent.".to_string(),
        reset_token_sent: true,
    }))
}

// Helper functions
fn is_valid_email(email: &str) -> bool {
    // Basic email validation - in production, use a proper email validation library
    email.contains('@') && email.contains('.') && email.len() > 5
}

fn is_strong_password(password: &str) -> bool {
    // Password strength validation
    password.len() >= 8
        && password.chars().any(|c| c.is_uppercase())
        && password.chars().any(|c| c.is_lowercase())
        && password.chars().any(|c| c.is_numeric())
        && password.chars().any(|c| !c.is_alphanumeric())
}