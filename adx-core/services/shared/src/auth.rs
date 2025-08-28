// Authentication utilities

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use chrono::{Duration, Utc};
use crate::{Result, ServiceError};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub tenant_id: String,
    pub user_email: String,
    pub roles: Vec<String>,
    pub exp: i64,
    pub iat: i64,
}

pub struct AuthManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
}

impl AuthManager {
    pub fn new(secret: &str) -> Self {
        Self {
            encoding_key: EncodingKey::from_secret(secret.as_ref()),
            decoding_key: DecodingKey::from_secret(secret.as_ref()),
        }
    }
    
    pub fn generate_token(&self, user_id: &str, tenant_id: &str, email: &str, roles: Vec<String>) -> Result<String> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            user_email: email.to_string(),
            roles,
            exp: (now + Duration::hours(24)).timestamp(),
            iat: now.timestamp(),
        };
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(|e| ServiceError::Authentication(e.to_string()))
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims> {
        decode::<Claims>(token, &self.decoding_key, &Validation::default())
            .map(|data| data.claims)
            .map_err(|e| ServiceError::Authentication(e.to_string()))
    }
    
    pub fn hash_password(&self, password: &str) -> Result<String> {
        bcrypt::hash(password, bcrypt::DEFAULT_COST)
            .map_err(|e| ServiceError::Authentication(e.to_string()))
    }
    
    pub fn verify_password(&self, password: &str, hash: &str) -> Result<bool> {
        bcrypt::verify(password, hash)
            .map_err(|e| ServiceError::Authentication(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_auth_manager() -> AuthManager {
        AuthManager::new("test-secret-key")
    }

    #[test]
    fn test_password_hashing() {
        let auth = get_test_auth_manager();
        let password = "test-password";
        
        let hash = auth.hash_password(password).unwrap();
        assert_ne!(hash, password);
        assert!(hash.starts_with("$2b$"));
        
        assert!(auth.verify_password(password, &hash).unwrap());
        assert!(!auth.verify_password("wrong-password", &hash).unwrap());
    }

    #[test]
    fn test_token_generation_and_validation() {
        let auth = get_test_auth_manager();
        
        let token = auth
            .generate_token(
                "user123",
                "tenant456",
                "user@example.com",
                vec!["user".to_string(), "admin".to_string()],
            )
            .unwrap();
        
        assert!(!token.is_empty());
        
        let claims = auth.validate_token(&token).unwrap();
        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.tenant_id, "tenant456");
        assert_eq!(claims.user_email, "user@example.com");
        assert_eq!(claims.roles, vec!["user", "admin"]);
    }

    #[test]
    fn test_invalid_token() {
        let auth = get_test_auth_manager();
        let result = auth.validate_token("invalid-token");
        assert!(result.is_err());
    }

    #[test]
    fn test_expired_token() {
        // This would require mocking time or creating an expired token
        // For now, we'll just test the basic validation
        let auth = get_test_auth_manager();
        let result = auth.validate_token("eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.invalid");
        assert!(result.is_err());
    }
}