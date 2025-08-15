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
    
    // Check role-based permissions (simplified - in production this would query a permissions service)
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
            "file:*".to_string(),
            "workflow:*".to_string(),
        ]),
        "user" => Some(vec![
            "file:read".to_string(),
            "file:write".to_string(),
            "workflow:execute".to_string(),
        ]),
        "viewer" => Some(vec![
            "file:read".to_string(),
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

// Middleware to check specific permissions
pub async fn require_permission(
    permission: &'static str,
) -> impl Fn(Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |request: Request, next: Next| {
        let required_permission = permission;
        Box::pin(async move {
            let claims = request
                .extensions()
                .get::<Claims>()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            if !has_permission(claims, required_permission) {
                warn!(
                    "User {} lacks permission: {}",
                    claims.sub, required_permission
                );
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(request).await)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jsonwebtoken::{encode, EncodingKey, Header};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_claims() -> Claims {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Claims {
            sub: "user123".to_string(),
            exp: now + 3600, // 1 hour from now
            iat: now,
            iss: "adx-core".to_string(),
            aud: "file-bff".to_string(),
            tenant_id: "tenant123".to_string(),
            tenant_name: "Test Tenant".to_string(),
            user_email: "test@example.com".to_string(),
            user_roles: vec!["user".to_string()],
            permissions: vec!["file:read".to_string(), "file:write".to_string()],
            features: vec!["basic".to_string()],
            session_id: "session123".to_string(),
            device_id: None,
            ip_address: "127.0.0.1".to_string(),
            available_tenants: vec!["tenant123".to_string()],
            tenant_roles: HashMap::new(),
        }
    }

    fn create_test_token(claims: &Claims) -> String {
        let jwt_secret = "test-secret";
        let encoding_key = EncodingKey::from_secret(jwt_secret.as_ref());
        
        std::env::set_var("JWT_SECRET", jwt_secret);
        
        encode(&Header::default(), claims, &encoding_key).unwrap()
    }

    #[test]
    fn test_validate_jwt_token() {
        let claims = create_test_claims();
        let token = create_test_token(&claims);
        
        let decoded_claims = validate_jwt_token(&token).unwrap();
        assert_eq!(decoded_claims.sub, claims.sub);
        assert_eq!(decoded_claims.user_email, claims.user_email);
    }

    #[test]
    fn test_has_permission() {
        let claims = create_test_claims();
        
        assert!(has_permission(&claims, "file:read"));
        assert!(has_permission(&claims, "file:write"));
        assert!(!has_permission(&claims, "file:delete"));
    }

    #[test]
    fn test_wildcard_permissions() {
        let mut claims = create_test_claims();
        claims.permissions = vec!["file:*".to_string()];
        
        assert!(has_permission(&claims, "file:read"));
        assert!(has_permission(&claims, "file:write"));
        assert!(has_permission(&claims, "file:delete"));
        assert!(!has_permission(&claims, "workflow:execute"));
    }

    #[test]
    fn test_role_based_permissions() {
        let mut claims = create_test_claims();
        claims.user_roles = vec!["admin".to_string()];
        claims.permissions = vec![]; // No direct permissions
        
        assert!(has_permission(&claims, "file:read"));
        assert!(has_permission(&claims, "file:delete"));
        assert!(has_permission(&claims, "workflow:execute"));
    }
}