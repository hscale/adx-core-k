use adx_shared::{TenantId, UserId};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: UserId,              // User ID
    pub tenant_id: TenantId,      // Tenant ID
    pub roles: Vec<String>,       // User roles
    pub permissions: Vec<String>, // User permissions
    pub exp: i64,                 // Expiration
    pub iat: i64,                 // Issued at
    pub jti: Uuid,                // JWT ID for revocation
}

#[derive(Debug, thiserror::Error)]
pub enum JwtError {
    #[error("JWT encoding error: {0}")]
    Encoding(#[from] jsonwebtoken::errors::Error),
    #[error("JWT validation error: {0}")]
    Validation(String),
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl std::fmt::Debug for JwtService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("JwtService")
            .field("validation", &self.validation)
            .finish_non_exhaustive()
    }
}

impl JwtService {
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

    pub fn generate_access_token(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        roles: Vec<String>,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            tenant_id,
            roles,
            permissions: vec![], // Will be populated by authorization service
            exp: (now + Duration::hours(1)).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(JwtError::from)
    }

    pub fn generate_refresh_token(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<String, JwtError> {
        let now = Utc::now();
        let claims = Claims {
            sub: user_id,
            tenant_id,
            roles: vec![], // Refresh tokens don't need roles
            permissions: vec![],
            exp: (now + Duration::days(30)).timestamp(),
            iat: now.timestamp(),
            jti: Uuid::new_v4(),
        };

        encode(&Header::default(), &claims, &self.encoding_key).map_err(JwtError::from)
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(JwtError::from)
    }
}
