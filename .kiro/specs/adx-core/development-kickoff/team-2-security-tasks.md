# Team 2: Identity & Security - Immediate Development Tasks

## 🔒 Mission: Build Security Foundation

**Status**: START IMMEDIATELY - CRITICAL PATH
**Timeline**: Week 1-2 (Security must be ready for other teams)
**Team Size**: 6 developers

## Day 1 Tasks - START NOW

### Authentication Service (2 developers)

#### Task 2.1: JWT Token Service
```rust
// File: services/auth-service/src/jwt.rs
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{Utc, Duration};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,           // User ID
    pub tenant_id: Uuid,     // Tenant ID
    pub roles: Vec<String>,  // User roles
    pub permissions: Vec<String>, // User permissions
    pub exp: i64,           // Expiration
    pub iat: i64,           // Issued at
    pub jti: Uuid,          // JWT ID for revocation
}

pub struct JwtService {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
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
    
    pub fn generate_access_token(&self, user_id: Uuid, tenant_id: Uuid, roles: Vec<String>) -> Result<String, JwtError> {
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
        
        encode(&Header::default(), &claims, &self.encoding_key)
            .map_err(JwtError::from)
    }
    
    pub fn validate_token(&self, token: &str) -> Result<Claims, JwtError> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map(|data| data.claims)
            .map_err(JwtError::from)
    }
}
```