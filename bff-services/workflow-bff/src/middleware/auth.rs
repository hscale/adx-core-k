use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

use crate::{types::UserContext, AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,           // User ID
    pub exp: i64,              // Expiration time
    pub iat: i64,              // Issued at
    pub iss: String,           // Issuer
    pub aud: String,           // Audience
    
    // ADX Core specific claims
    pub tenant_id: String,     // Current tenant
    pub tenant_name: String,   // Tenant display name
    pub user_email: String,    // User email
    pub user_roles: Vec<String>, // User roles in current tenant
    pub permissions: Vec<String>, // Specific permissions
    pub features: Vec<String>, // Available features
    
    // Session information
    pub session_id: String,    // Session identifier
    pub device_id: Option<String>, // Device identifier
    pub ip_address: String,    // Client IP address
    
    // Multi-tenant support
    pub available_tenants: Vec<String>, // Tenants user has access to
    pub tenant_roles: HashMap<String, Vec<String>>, // Roles per tenant
}

pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Skip auth for health check
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    let headers = request.headers();
    
    // Extract JWT token from Authorization header
    let token = extract_token_from_headers(headers)
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Validate and decode JWT token
    let claims = validate_jwt_token(&token)
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    // Create user context from claims
    let user_context = UserContext {
        user_id: claims.sub.clone(),
        email: claims.user_email.clone(),
        roles: claims.user_roles.clone(),
        permissions: claims.permissions.clone(),
    };

    // Add user context to request extensions
    request.extensions_mut().insert(user_context);
    request.extensions_mut().insert(claims);

    Ok(next.run(request).await)
}

fn extract_token_from_headers(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    
    if auth_str.starts_with("Bearer ") {
        Some(auth_str[7..].to_string())
    } else {
        None
    }
}

fn validate_jwt_token(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    // In production, this should use a proper JWT secret or public key
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "your-secret-key".to_string());

    let decoding_key = DecodingKey::from_secret(jwt_secret.as_ref());
    let validation = Validation::new(Algorithm::HS256);

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    
    debug!("JWT token validated for user: {}", token_data.claims.sub);
    
    Ok(token_data.claims)
}

// Helper function to check if user has specific permission
pub fn has_permission(claims: &Claims, required_permission: &str) -> bool {
    // Check direct permissions
    if claims.permissions.contains(&required_permission.to_string()) {
        return true;
    }
    
    // Check role-based permissions
    for role in &claims.user_roles {
        if let Some(role_permissions) = get_role_permissions(role) {
            if role_permissions.contains(&required_permission.to_string()) {
                return true;
            }
        }
    }
    
    // Check wildcard permissions
    let permission_parts: Vec<&str> = required_permission.split(':').collect();
    for perm in &claims.permissions {
        if matches_wildcard_permission(perm, &permission_parts) {
            return true;
        }
    }
    
    false
}

fn get_role_permissions(role: &str) -> Option<Vec<String>> {
    // In production, this would query a permissions service or database
    match role {
        "admin" => Some(vec![
            "workflow:*".to_string(),
            "monitoring:*".to_string(),
        ]),
        "user" => Some(vec![
            "workflow:read".to_string(),
            "workflow:execute".to_string(),
            "monitoring:read".to_string(),
        ]),
        "viewer" => Some(vec![
            "workflow:read".to_string(),
            "monitoring:read".to_string(),
        ]),
        _ => None,
    }
}

fn matches_wildcard_permission(permission: &str, required_parts: &[&str]) -> bool {
    if permission.ends_with('*') {
        let perm_prefix = &permission[..permission.len() - 1];
        let required_prefix = required_parts.join(":");
        required_prefix.starts_with(perm_prefix)
    } else {
        false
    }
}