use chrono::{DateTime, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Error, Result, TenantId, UserId, SubscriptionTier, TenantQuotas, UserQuotas};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    // Standard claims
    pub sub: UserId,           // User ID
    pub exp: i64,              // Expiration time
    pub iat: i64,              // Issued at
    pub iss: String,           // Issuer
    pub aud: String,           // Audience
    
    // ADX Core specific claims
    pub tenant_id: TenantId,   // Current tenant
    pub tenant_name: String,   // Tenant display name
    pub user_email: String,    // User email
    pub user_roles: Vec<String>, // User roles in current tenant
    pub permissions: Vec<String>, // Specific permissions
    pub features: Vec<String>, // Available features
    pub quotas: UserQuotas,    // User-specific quotas
    
    // Session information
    pub session_id: String,    // Session identifier
    pub device_id: Option<String>, // Device identifier
    pub ip_address: String,    // Client IP address
    
    // Multi-tenant support
    pub available_tenants: Vec<String>, // Tenants user has access to
    pub tenant_roles: HashMap<String, Vec<String>>, // Roles per tenant
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub subscription_tier: SubscriptionTier,
    pub features: Vec<String>,
    pub quotas: TenantQuotas,
    pub settings: TenantSettings,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantSettings {
    pub timezone: String,
    pub locale: String,
    pub theme: String,
    pub custom_domain: Option<String>,
    pub branding: Option<TenantBranding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantBranding {
    pub logo_url: Option<String>,
    pub primary_color: Option<String>,
    pub secondary_color: Option<String>,
    pub custom_css: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: UserId,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub language: String,
    pub timezone: String,
    pub theme: String,
    pub notifications: NotificationPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_enabled: bool,
    pub push_enabled: bool,
    pub workflow_updates: bool,
    pub security_alerts: bool,
}

impl Default for TenantSettings {
    fn default() -> Self {
        Self {
            timezone: "UTC".to_string(),
            locale: "en-US".to_string(),
            theme: "light".to_string(),
            custom_domain: None,
            branding: None,
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            theme: "light".to_string(),
            notifications: NotificationPreferences {
                email_enabled: true,
                push_enabled: true,
                workflow_updates: true,
                security_alerts: true,
            },
        }
    }
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    pub fn new(secret: &str) -> Self {
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let decoding_key = DecodingKey::from_secret(secret.as_ref());
        let validation = Validation::default();
        
        Self {
            encoding_key,
            decoding_key,
            validation,
        }
    }
    
    pub fn generate_token(&self, claims: &JwtClaims) -> Result<String> {
        encode(&Header::default(), claims, &self.encoding_key)
            .map_err(|e| Error::Authentication(format!("Failed to generate token: {}", e)))
    }
    
    pub fn validate_token(&self, token: &str) -> Result<JwtClaims> {
        decode::<JwtClaims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(|e| Error::Authentication(format!("Invalid token: {}", e)))
    }
    
    pub fn extract_bearer_token(auth_header: &str) -> Result<String> {
        if let Some(token) = auth_header.strip_prefix("Bearer ") {
            Ok(token.to_string())
        } else {
            Err(Error::Authentication("Invalid authorization header format".to_string()))
        }
    }
}

// Permission checking utilities
pub fn has_permission(claims: &JwtClaims, required_permission: &str) -> bool {
    // Check direct permissions
    if claims.permissions.contains(&required_permission.to_string()) {
        return true;
    }
    
    // Check role-based permissions (would be implemented with a role permission mapping)
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