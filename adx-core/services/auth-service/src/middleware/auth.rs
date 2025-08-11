use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};

use adx_shared::{
    auth::JwtClaims,
    Error,
};
use crate::AppState;

/// Authentication middleware that validates JWT tokens
pub async fn auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Extract authorization header
    let auth_header = match headers.get("authorization") {
        Some(header) => match header.to_str() {
            Ok(header_str) => header_str,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        },
        None => return Err(StatusCode::UNAUTHORIZED),
    };

    // Extract bearer token
    let token = match extract_bearer_token(auth_header) {
        Ok(token) => token,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Validate token
    let claims = match state.jwt_manager.validate_token(&token) {
        Ok(claims) => claims,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Check token expiration
    let now = chrono::Utc::now().timestamp();
    if claims.exp < now {
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Add claims to request extensions
    request.extensions_mut().insert(claims);

    // Continue to next middleware/handler
    Ok(next.run(request).await)
}

/// Optional authentication middleware that doesn't fail if no token is provided
pub async fn optional_auth_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> std::result::Result<Response, StatusCode> {
    let headers = request.headers();
    
    // Try to extract authorization header
    if let Some(auth_header) = headers.get("authorization") {
        if let Ok(header_str) = auth_header.to_str() {
            if let Ok(token) = extract_bearer_token(header_str) {
                if let Ok(claims) = state.jwt_manager.validate_token(&token) {
                    // Check token expiration
                    let now = chrono::Utc::now().timestamp();
                    if claims.exp >= now {
                        // Add claims to request extensions
                        request.extensions_mut().insert(claims);
                    }
                }
            }
        }
    }

    // Continue to next middleware/handler regardless of auth status
    Ok(next.run(request).await)
}

/// Permission-based authorization middleware
pub fn require_permission(required_permission: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Get claims from request extensions
            let claims = match request.extensions().get::<JwtClaims>() {
                Some(claims) => claims,
                None => return Err(StatusCode::UNAUTHORIZED),
            };

            // Check if user has required permission
            if !has_permission(claims, required_permission) {
                return Err(StatusCode::FORBIDDEN);
            }

            // Continue to next middleware/handler
            Ok(next.run(request).await)
        })
    }
}

/// Role-based authorization middleware
pub fn require_role(required_role: &'static str) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        Box::pin(async move {
            // Get claims from request extensions
            let claims = match request.extensions().get::<JwtClaims>() {
                Some(claims) => claims,
                None => return Err(StatusCode::UNAUTHORIZED),
            };

            // Check if user has required role
            if !claims.user_roles.contains(&required_role.to_string()) {
                return Err(StatusCode::FORBIDDEN);
            }

            // Continue to next middleware/handler
            Ok(next.run(request).await)
        })
    }
}

// Helper functions
fn extract_bearer_token(auth_header: &str) -> std::result::Result<String, Error> {
    if let Some(token) = auth_header.strip_prefix("Bearer ") {
        Ok(token.to_string())
    } else {
        Err(Error::Authentication("Invalid authorization header format".to_string()))
    }
}

fn has_permission(claims: &JwtClaims, required_permission: &str) -> bool {
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
    // This would be loaded from a database or configuration
    match role {
        "admin" => Some(vec!["*".to_string()]),
        "user" => Some(vec![
            "tenant:read".to_string(),
            "user:read".to_string(),
            "user:write".to_string(),
            "file:read".to_string(),
            "file:write".to_string(),
        ]),
        "viewer" => Some(vec![
            "tenant:read".to_string(),
            "user:read".to_string(),
            "file:read".to_string(),
        ]),
        _ => None,
    }
}

fn matches_wildcard_permission(permission: &str, required_parts: &[&str]) -> bool {
    if permission == "*" {
        return true;
    }
    
    let perm_parts: Vec<&str> = permission.split(':').collect();
    if perm_parts.len() != required_parts.len() {
        return false;
    }
    
    for (perm_part, req_part) in perm_parts.iter().zip(required_parts.iter()) {
        if perm_part != &"*" && perm_part != req_part {
            return false;
        }
    }
    
    true
}