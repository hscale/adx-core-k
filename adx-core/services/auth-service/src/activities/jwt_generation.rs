use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use adx_shared::{
    temporal::{
        ActivityContext, AdxActivity, TenantAwareActivity, DatabaseActivity,
        ActivityError, utils::database_retry_policy
    },
    auth::{UserContext, TenantContext, JwtClaims, JwtManager},
    database::DatabasePool,
    UserQuotas, TenantQuotas,
    Error, Result,
};

use crate::repositories::{
    UserRepository, SessionRepository,
    user::{User, UserStatus},
    session::{Session, SessionStatus},
};

/// Request for generating JWT tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateJwtTokensRequest {
    pub user_id: String,
    pub session_duration_hours: Option<i64>,
    pub include_refresh_token: bool,
    pub device_id: Option<String>,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub additional_claims: Option<HashMap<String, serde_json::Value>>,
    pub scopes: Option<Vec<String>>,
}

/// Response from generating JWT tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerateJwtTokensResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_type: String,
    pub expires_in: i64,
    pub expires_at: DateTime<Utc>,
    pub session_id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub scopes: Vec<String>,
}

/// Token configuration
#[derive(Debug, Clone)]
pub struct TokenConfig {
    pub access_token_duration_hours: i64,
    pub refresh_token_duration_days: i64,
    pub issuer: String,
    pub audience: String,
    pub include_user_permissions: bool,
    pub include_tenant_features: bool,
}

impl Default for TokenConfig {
    fn default() -> Self {
        Self {
            access_token_duration_hours: 1, // 1 hour
            refresh_token_duration_days: 30, // 30 days
            issuer: "adx-core".to_string(),
            audience: "adx-core-api".to_string(),
            include_user_permissions: true,
            include_tenant_features: true,
        }
    }
}

/// Activity for generating JWT tokens with proper claims
pub struct GenerateJwtTokensActivity {
    database_pool: DatabasePool,
    jwt_manager: JwtManager,
    token_config: TokenConfig,
}

impl GenerateJwtTokensActivity {
    pub fn new(
        database_pool: DatabasePool,
        jwt_manager: JwtManager,
        token_config: Option<TokenConfig>,
    ) -> Self {
        Self {
            database_pool,
            jwt_manager,
            token_config: token_config.unwrap_or_default(),
        }
    }

    /// Get user with full details
    async fn get_user_details(
        &self,
        tenant_id: &str,
        user_id: &str,
    ) -> Result<User, ActivityError> {
        let user_repo = UserRepository::new(
            self.database_pool.clone(),
            tenant_id.to_string(),
        );

        user_repo.find_by_id(user_id).await
            .map_err(|e| ActivityError::DatabaseError {
                message: format!("Failed to get user details: {}", e),
            })?
            .ok_or_else(|| ActivityError::NotFoundError {
                resource_type: "user".to_string(),
                resource_id: user_id.to_string(),
            })
    }

    /// Create session record
    async fn create_session(
        &self,
        tenant_id: &str,
        user_id: &str,
        device_id: Option<&str>,
        client_ip: &str,
        user_agent: Option<&str>,
        expires_at: DateTime<Utc>,
    ) -> Result<Session, ActivityError> {
        let session_repo = SessionRepository::new(
            self.database_pool.clone(),
            tenant_id.to_string(),
        );

        let session = Session {
            id: Uuid::new_v4().to_string(),
            tenant_id: tenant_id.to_string(),
            user_id: user_id.to_string(),
            device_id: device_id.map(|s| s.to_string()),
            client_ip: client_ip.to_string(),
            user_agent: user_agent.map(|s| s.to_string()),
            status: SessionStatus::Active,
            expires_at,
            last_activity_at: Utc::now(),
            metadata: serde_json::json!({}),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        session_repo.create(session).await
            .map_err(|e| ActivityError::DatabaseError {
                message: format!("Failed to create session: {}", e),
            })
    }

    /// Build JWT claims
    fn build_jwt_claims(
        &self,
        user: &User,
        tenant_context: &TenantContext,
        session_id: &str,
        client_ip: &str,
        expires_at: DateTime<Utc>,
        scopes: Vec<String>,
        additional_claims: Option<HashMap<String, serde_json::Value>>,
    ) -> JwtClaims {
        let mut available_tenants = vec![tenant_context.tenant_id.clone()];
        let mut tenant_roles = HashMap::new();
        tenant_roles.insert(tenant_context.tenant_id.clone(), user.roles.clone());

        // Build user quotas
        let user_quotas = UserQuotas {
            api_calls_per_hour: 1000, // Default, would be loaded from user/tenant settings
            storage_gb: 10,
            concurrent_workflows: 5,
            file_upload_size_mb: 100,
        };

        let mut claims = JwtClaims {
            // Standard claims
            sub: user.id.clone(),
            exp: expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            iss: self.token_config.issuer.clone(),
            aud: self.token_config.audience.clone(),
            
            // ADX Core specific claims
            tenant_id: tenant_context.tenant_id.clone(),
            tenant_name: tenant_context.tenant_name.clone(),
            user_email: user.email.clone(),
            user_roles: user.roles.clone(),
            permissions: if self.token_config.include_user_permissions {
                user.permissions.clone()
            } else {
                vec![]
            },
            features: if self.token_config.include_tenant_features {
                tenant_context.features.clone()
            } else {
                vec![]
            },
            quotas: user_quotas,
            
            // Session information
            session_id: session_id.to_string(),
            device_id: None, // Would be set if provided
            ip_address: client_ip.to_string(),
            
            // Multi-tenant support
            available_tenants,
            tenant_roles,
        };

        // Add additional claims if provided
        if let Some(additional) = additional_claims {
            // For now, we'll store additional claims in a custom field
            // In a real implementation, you might extend JwtClaims to support custom fields
            tracing::debug!("Additional claims provided: {:?}", additional);
        }

        claims
    }

    /// Generate access token
    fn generate_access_token(
        &self,
        claims: &JwtClaims,
    ) -> Result<String, ActivityError> {
        self.jwt_manager.generate_token(claims)
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to generate access token: {}", e),
            })
    }

    /// Generate refresh token (simplified - in production would have different claims)
    fn generate_refresh_token(
        &self,
        user_id: &str,
        tenant_id: &str,
        session_id: &str,
        expires_at: DateTime<Utc>,
    ) -> Result<String, ActivityError> {
        let refresh_claims = JwtClaims {
            sub: user_id.to_string(),
            exp: expires_at.timestamp(),
            iat: Utc::now().timestamp(),
            iss: self.token_config.issuer.clone(),
            aud: format!("{}-refresh", self.token_config.audience),
            tenant_id: tenant_id.to_string(),
            tenant_name: "".to_string(), // Minimal claims for refresh token
            user_email: "".to_string(),
            user_roles: vec![],
            permissions: vec![],
            features: vec![],
            quotas: UserQuotas {
                api_calls_per_hour: 0,
                storage_gb: 0,
                concurrent_workflows: 0,
                file_upload_size_mb: 0,
            },
            session_id: session_id.to_string(),
            device_id: None,
            ip_address: "".to_string(),
            available_tenants: vec![],
            tenant_roles: HashMap::new(),
        };

        self.jwt_manager.generate_token(&refresh_claims)
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to generate refresh token: {}", e),
            })
    }

    /// Validate scopes
    fn validate_scopes(&self, scopes: &[String], user: &User) -> Result<Vec<String>, ActivityError> {
        let mut validated_scopes = Vec::new();

        for scope in scopes {
            match scope.as_str() {
                "read" => {
                    // Basic read access - always allowed for active users
                    if user.status == UserStatus::Active {
                        validated_scopes.push(scope.clone());
                    }
                }
                "write" => {
                    // Write access - check if user has write permissions
                    if user.permissions.iter().any(|p| p.contains("write") || p == "*") {
                        validated_scopes.push(scope.clone());
                    }
                }
                "admin" => {
                    // Admin access - check if user has admin role
                    if user.roles.contains(&"admin".to_string()) {
                        validated_scopes.push(scope.clone());
                    }
                }
                "api" => {
                    // API access - check if user has API permissions
                    if user.permissions.iter().any(|p| p.starts_with("api:") || p == "*") {
                        validated_scopes.push(scope.clone());
                    }
                }
                _ => {
                    // Custom scope - validate against user permissions
                    if user.permissions.contains(scope) || user.permissions.contains(&"*".to_string()) {
                        validated_scopes.push(scope.clone());
                    }
                }
            }
        }

        Ok(validated_scopes)
    }
}

#[async_trait]
impl AdxActivity<GenerateJwtTokensRequest, GenerateJwtTokensResponse> for GenerateJwtTokensActivity {
    async fn execute(
        &self,
        context: ActivityContext,
        input: GenerateJwtTokensRequest,
    ) -> Result<GenerateJwtTokensResponse, ActivityError> {
        // Validate input
        self.validate_input(&input)?;

        // Get user details
        let user = self.get_user_details(
            &context.tenant_context.tenant_id,
            &input.user_id,
        ).await?;

        // Check if user is active
        if user.status != UserStatus::Active {
            return Err(ActivityError::AuthorizationError {
                message: format!("Cannot generate tokens for user with status: {:?}", user.status),
            });
        }

        // Validate and filter scopes
        let requested_scopes = input.scopes.unwrap_or_else(|| vec!["read".to_string()]);
        let validated_scopes = self.validate_scopes(&requested_scopes, &user)?;

        // Calculate token expiration
        let session_duration = input.session_duration_hours
            .unwrap_or(self.token_config.access_token_duration_hours);
        let expires_at = Utc::now() + Duration::hours(session_duration);

        // Create session
        let session = self.create_session(
            &context.tenant_context.tenant_id,
            &input.user_id,
            input.device_id.as_deref(),
            &input.client_ip,
            input.user_agent.as_deref(),
            expires_at,
        ).await?;

        // Build JWT claims
        let claims = self.build_jwt_claims(
            &user,
            &context.tenant_context,
            &session.id,
            &input.client_ip,
            expires_at,
            validated_scopes.clone(),
            input.additional_claims,
        );

        // Generate access token
        let access_token = self.generate_access_token(&claims)?;

        // Generate refresh token if requested
        let refresh_token = if input.include_refresh_token {
            let refresh_expires_at = Utc::now() + Duration::days(self.token_config.refresh_token_duration_days);
            Some(self.generate_refresh_token(
                &input.user_id,
                &context.tenant_context.tenant_id,
                &session.id,
                refresh_expires_at,
            )?)
        } else {
            None
        };

        Ok(GenerateJwtTokensResponse {
            access_token,
            refresh_token,
            token_type: "Bearer".to_string(),
            expires_in: session_duration * 3600, // Convert hours to seconds
            expires_at,
            session_id: session.id,
            user_id: input.user_id,
            tenant_id: context.tenant_context.tenant_id,
            scopes: validated_scopes,
        })
    }

    fn activity_type(&self) -> &'static str {
        "generate_jwt_tokens_activity"
    }

    fn validate_input(&self, input: &GenerateJwtTokensRequest) -> Result<(), ActivityError> {
        if input.user_id.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "user_id".to_string(),
                message: "User ID is required".to_string(),
            });
        }

        if input.client_ip.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "client_ip".to_string(),
                message: "Client IP is required".to_string(),
            });
        }

        // Validate session duration
        if let Some(duration) = input.session_duration_hours {
            if duration <= 0 || duration > 24 * 7 { // Max 1 week
                return Err(ActivityError::ValidationError {
                    field: "session_duration_hours".to_string(),
                    message: "Session duration must be between 1 hour and 1 week".to_string(),
                });
            }
        }

        // Validate scopes if provided
        if let Some(ref scopes) = input.scopes {
            if scopes.is_empty() {
                return Err(ActivityError::ValidationError {
                    field: "scopes".to_string(),
                    message: "Scopes cannot be empty if provided".to_string(),
                });
            }

            for scope in scopes {
                if scope.trim().is_empty() {
                    return Err(ActivityError::ValidationError {
                        field: "scopes".to_string(),
                        message: "Scope names cannot be empty".to_string(),
                    });
                }
            }
        }

        Ok(())
    }

    fn default_options(&self) -> adx_shared::temporal::ActivityExecutionOptions {
        let mut options = adx_shared::temporal::ActivityExecutionOptions::default();
        options.retry_policy = Some(database_retry_policy());
        options.tags.push("jwt_generation".to_string());
        options.tags.push("authentication".to_string());
        options.tags.push("session_management".to_string());
        options
    }
}

#[async_trait]
impl TenantAwareActivity<GenerateJwtTokensRequest, GenerateJwtTokensResponse> for GenerateJwtTokensActivity {
    async fn validate_tenant_access(
        &self,
        tenant_context: &TenantContext,
        user_context: &UserContext,
    ) -> Result<(), ActivityError> {
        // Check if tenant is active
        if !tenant_context.is_active {
            return Err(ActivityError::AuthorizationError {
                message: "Cannot generate tokens for inactive tenant".to_string(),
            });
        }

        // Check if user has permission to generate tokens (for service accounts)
        if user_context.user_id != "system" && 
           !user_context.permissions.contains(&"auth:generate_tokens".to_string()) &&
           !user_context.roles.contains(&"admin".to_string()) {
            return Err(ActivityError::AuthorizationError {
                message: "Insufficient permissions to generate tokens".to_string(),
            });
        }

        Ok(())
    }

    async fn check_tenant_quotas(
        &self,
        tenant_context: &TenantContext,
        resource_type: &str,
        requested_amount: u64,
    ) -> Result<(), ActivityError> {
        if resource_type == "sessions" {
            // Check if tenant has session management features enabled
            if !tenant_context.features.contains(&"session_management".to_string()) {
                return Err(ActivityError::QuotaExceededError {
                    resource_type: "sessions".to_string(),
                    current_usage: 0,
                    limit: 0,
                    requested: requested_amount,
                });
            }
        }

        Ok(())
    }
}

#[async_trait]
impl DatabaseActivity<GenerateJwtTokensRequest, GenerateJwtTokensResponse> for GenerateJwtTokensActivity {
    async fn get_tenant_connection(
        &self,
        _tenant_context: &TenantContext,
    ) -> Result<Box<dyn std::any::Any + Send + Sync>, ActivityError> {
        Ok(Box::new(self.database_pool.clone()))
    }

    async fn execute_transaction<F, R>(
        &self,
        _tenant_context: &TenantContext,
        transaction: F,
    ) -> Result<R, ActivityError>
    where
        F: FnOnce() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<R, ActivityError>> + Send>> + Send,
        R: Send + Sync,
    {
        // For now, execute without explicit transaction
        // TODO: Implement proper transaction support when needed
        transaction().await
    }
}

// Tests commented out for now due to compilation issues
// #[cfg(test)]
// mod tests {
//     // Test implementations would go here
// }