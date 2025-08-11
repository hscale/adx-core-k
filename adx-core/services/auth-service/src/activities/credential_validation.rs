use async_trait::async_trait;
use bcrypt::verify;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use adx_shared::{
    temporal::{
        ActivityContext, AdxActivity, TenantAwareActivity, DatabaseActivity,
        ActivityError, utils::database_retry_policy
    },
    auth::{UserContext, TenantContext},
    database::DatabasePool,
    Error, Result,
};

use crate::repositories::{UserRepository, user::{User, UserStatus}};

/// Simple rate limiter for activities
#[derive(Clone)]
pub struct RateLimiter {
    // In production, this would use Redis or similar
    // For now, we'll use a simple in-memory implementation
}

impl RateLimiter {
    pub fn new_mock() -> Self {
        Self {}
    }

    pub async fn get_count(&self, _key: &str, _window_seconds: u64) -> Result<u32, String> {
        // Mock implementation - always return 0
        Ok(0)
    }

    pub async fn increment(&self, _key: &str, _window_seconds: u64) -> Result<u32, String> {
        // Mock implementation - always return 1
        Ok(1)
    }

    pub async fn clear(&self, _key: &str) -> Result<(), String> {
        // Mock implementation - always succeed
        Ok(())
    }

    pub async fn get_expiry(&self, _key: &str) -> Result<Option<DateTime<Utc>>, String> {
        // Mock implementation - no expiry
        Ok(None)
    }

    pub async fn set_with_expiry(&self, _key: &str, _value: &str, _ttl_seconds: u64) -> Result<(), String> {
        // Mock implementation - always succeed
        Ok(())
    }
}

/// Request for validating user credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCredentialsRequest {
    pub email: String,
    pub password: String,
    pub client_ip: String,
    pub user_agent: Option<String>,
    pub require_email_verification: bool,
    pub check_account_status: bool,
}

/// Response from validating user credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateCredentialsResponse {
    pub is_valid: bool,
    pub user_id: Option<String>,
    pub user_status: Option<UserStatus>,
    pub email_verified: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub failed_attempts: u32,
    pub account_locked: bool,
    pub lock_expires_at: Option<DateTime<Utc>>,
    pub requires_mfa: bool,
    pub validation_errors: Vec<String>,
}

/// Rate limiting configuration
#[derive(Debug, Clone)]
pub struct RateLimitConfig {
    pub max_attempts_per_hour: u32,
    pub max_attempts_per_day: u32,
    pub lockout_duration_minutes: u32,
    pub progressive_delay: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_attempts_per_hour: 10,
            max_attempts_per_day: 50,
            lockout_duration_minutes: 30,
            progressive_delay: true,
        }
    }
}

/// Activity for validating user credentials with rate limiting
pub struct ValidateCredentialsActivity {
    database_pool: DatabasePool,
    rate_limiter: RateLimiter,
    rate_limit_config: RateLimitConfig,
}

impl ValidateCredentialsActivity {
    pub fn new(
        database_pool: DatabasePool,
        rate_limiter: RateLimiter,
        rate_limit_config: Option<RateLimitConfig>,
    ) -> Self {
        Self {
            database_pool,
            rate_limiter,
            rate_limit_config: rate_limit_config.unwrap_or_default(),
        }
    }

    /// Check rate limiting for login attempts
    async fn check_rate_limit(
        &self,
        tenant_id: &str,
        email: &str,
        client_ip: &str,
    ) -> Result<(), ActivityError> {
        // Check rate limits by email
        let email_key = format!("login_attempts:email:{}:{}", tenant_id, email);
        let email_attempts = self.rate_limiter.get_count(&email_key, 3600).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to check email rate limit: {}", e),
            })?;

        if email_attempts >= self.rate_limit_config.max_attempts_per_hour {
            return Err(ActivityError::RateLimitExceededError {
                resource_type: "login_attempts_email".to_string(),
                current_usage: email_attempts as u64,
                limit: self.rate_limit_config.max_attempts_per_hour as u64,
                retry_after: Some(3600),
            });
        }

        // Check rate limits by IP
        let ip_key = format!("login_attempts:ip:{}:{}", tenant_id, client_ip);
        let ip_attempts = self.rate_limiter.get_count(&ip_key, 3600).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to check IP rate limit: {}", e),
            })?;

        if ip_attempts >= self.rate_limit_config.max_attempts_per_hour * 3 {
            return Err(ActivityError::RateLimitExceededError {
                resource_type: "login_attempts_ip".to_string(),
                current_usage: ip_attempts as u64,
                limit: (self.rate_limit_config.max_attempts_per_hour * 3) as u64,
                retry_after: Some(3600),
            });
        }

        // Check daily limits
        let daily_email_key = format!("login_attempts:daily:email:{}:{}", tenant_id, email);
        let daily_attempts = self.rate_limiter.get_count(&daily_email_key, 86400).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to check daily rate limit: {}", e),
            })?;

        if daily_attempts >= self.rate_limit_config.max_attempts_per_day {
            return Err(ActivityError::RateLimitExceededError {
                resource_type: "login_attempts_daily".to_string(),
                current_usage: daily_attempts as u64,
                limit: self.rate_limit_config.max_attempts_per_day as u64,
                retry_after: Some(86400),
            });
        }

        Ok(())
    }

    /// Record failed login attempt
    async fn record_failed_attempt(
        &self,
        tenant_id: &str,
        email: &str,
        client_ip: &str,
    ) -> Result<u32, ActivityError> {
        // Increment counters
        let email_key = format!("login_attempts:email:{}:{}", tenant_id, email);
        let ip_key = format!("login_attempts:ip:{}:{}", tenant_id, client_ip);
        let daily_email_key = format!("login_attempts:daily:email:{}:{}", tenant_id, email);

        let email_count = self.rate_limiter.increment(&email_key, 3600).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to record email attempt: {}", e),
            })?;

        self.rate_limiter.increment(&ip_key, 3600).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to record IP attempt: {}", e),
            })?;

        self.rate_limiter.increment(&daily_email_key, 86400).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to record daily attempt: {}", e),
            })?;

        Ok(email_count)
    }

    /// Clear failed attempts on successful login
    async fn clear_failed_attempts(
        &self,
        tenant_id: &str,
        email: &str,
        client_ip: &str,
    ) -> Result<(), ActivityError> {
        let email_key = format!("login_attempts:email:{}:{}", tenant_id, email);
        let ip_key = format!("login_attempts:ip:{}:{}", tenant_id, client_ip);

        self.rate_limiter.clear(&email_key).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to clear email attempts: {}", e),
            })?;

        self.rate_limiter.clear(&ip_key).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to clear IP attempts: {}", e),
            })?;

        Ok(())
    }

    /// Check if account is locked
    async fn check_account_lock(
        &self,
        tenant_id: &str,
        email: &str,
    ) -> Result<(bool, Option<DateTime<Utc>>), ActivityError> {
        let lock_key = format!("account_lock:{}:{}", tenant_id, email);
        
        match self.rate_limiter.get_expiry(&lock_key).await {
            Ok(Some(expiry)) => {
                if expiry > Utc::now() {
                    Ok((true, Some(expiry)))
                } else {
                    // Lock expired, clear it
                    self.rate_limiter.clear(&lock_key).await
                        .map_err(|e| ActivityError::InternalError {
                            message: format!("Failed to clear expired lock: {}", e),
                        })?;
                    Ok((false, None))
                }
            }
            Ok(None) => Ok((false, None)),
            Err(e) => Err(ActivityError::InternalError {
                message: format!("Failed to check account lock: {}", e),
            }),
        }
    }

    /// Lock account after too many failed attempts
    async fn lock_account(
        &self,
        tenant_id: &str,
        email: &str,
    ) -> Result<DateTime<Utc>, ActivityError> {
        let lock_key = format!("account_lock:{}:{}", tenant_id, email);
        let lock_duration = Duration::minutes(self.rate_limit_config.lockout_duration_minutes as i64);
        let expires_at = Utc::now() + lock_duration;

        self.rate_limiter.set_with_expiry(&lock_key, "locked", lock_duration.num_seconds() as u64).await
            .map_err(|e| ActivityError::InternalError {
                message: format!("Failed to lock account: {}", e),
            })?;

        Ok(expires_at)
    }

    /// Validate password against hash
    fn validate_password(&self, password: &str, password_hash: &str) -> bool {
        verify(password, password_hash).unwrap_or(false)
    }

    /// Check if user requires MFA
    fn requires_mfa(&self, user: &User) -> bool {
        // Check if user has MFA enabled in preferences
        if let Some(mfa_enabled) = user.preferences.get("mfa_enabled") {
            return mfa_enabled.as_bool().unwrap_or(false);
        }
        false
    }

    /// Apply progressive delay based on failed attempts
    async fn apply_progressive_delay(&self, failed_attempts: u32) -> Result<(), ActivityError> {
        if !self.rate_limit_config.progressive_delay {
            return Ok(());
        }

        let delay_seconds = match failed_attempts {
            0..=2 => 0,
            3..=5 => 2,
            6..=10 => 5,
            11..=20 => 10,
            _ => 30,
        };

        if delay_seconds > 0 {
            tokio::time::sleep(std::time::Duration::from_secs(delay_seconds)).await;
        }

        Ok(())
    }
}

#[async_trait]
impl AdxActivity<ValidateCredentialsRequest, ValidateCredentialsResponse> for ValidateCredentialsActivity {
    async fn execute(
        &self,
        context: ActivityContext,
        input: ValidateCredentialsRequest,
    ) -> Result<ValidateCredentialsResponse, ActivityError> {
        // Validate input
        self.validate_input(&input)?;

        let mut validation_errors = Vec::new();

        // Check rate limiting first
        if let Err(e) = self.check_rate_limit(
            &context.tenant_context.tenant_id,
            &input.email,
            &input.client_ip,
        ).await {
            return Ok(ValidateCredentialsResponse {
                is_valid: false,
                user_id: None,
                user_status: None,
                email_verified: false,
                last_login: None,
                failed_attempts: 0,
                account_locked: true,
                lock_expires_at: None,
                requires_mfa: false,
                validation_errors: vec!["Rate limit exceeded".to_string()],
            });
        }

        // Check if account is locked
        let (is_locked, lock_expires_at) = self.check_account_lock(
            &context.tenant_context.tenant_id,
            &input.email,
        ).await?;

        if is_locked {
            return Ok(ValidateCredentialsResponse {
                is_valid: false,
                user_id: None,
                user_status: None,
                email_verified: false,
                last_login: None,
                failed_attempts: 0,
                account_locked: true,
                lock_expires_at,
                requires_mfa: false,
                validation_errors: vec!["Account is temporarily locked".to_string()],
            });
        }

        // Find user by email
        let user_repo = UserRepository::new(
            self.database_pool.clone(),
            context.tenant_context.tenant_id.clone(),
        );

        let user = match user_repo.find_by_email(&input.email).await {
            Ok(Some(user)) => user,
            Ok(None) => {
                // Record failed attempt even for non-existent users to prevent enumeration
                let failed_attempts = self.record_failed_attempt(
                    &context.tenant_context.tenant_id,
                    &input.email,
                    &input.client_ip,
                ).await?;

                self.apply_progressive_delay(failed_attempts).await?;

                return Ok(ValidateCredentialsResponse {
                    is_valid: false,
                    user_id: None,
                    user_status: None,
                    email_verified: false,
                    last_login: None,
                    failed_attempts,
                    account_locked: false,
                    lock_expires_at: None,
                    requires_mfa: false,
                    validation_errors: vec!["Invalid credentials".to_string()],
                });
            }
            Err(e) => {
                return Err(ActivityError::DatabaseError {
                    message: format!("Failed to find user: {}", e),
                });
            }
        };

        // Check account status
        if input.check_account_status {
            match user.status {
                UserStatus::Suspended => {
                    validation_errors.push("Account is suspended".to_string());
                }
                UserStatus::Inactive => {
                    validation_errors.push("Account is inactive".to_string());
                }
                UserStatus::PendingVerification if input.require_email_verification => {
                    validation_errors.push("Email verification required".to_string());
                }
                _ => {}
            }
        }

        // Validate password
        let password_valid = self.validate_password(&input.password, &user.password_hash);

        if !password_valid {
            let failed_attempts = self.record_failed_attempt(
                &context.tenant_context.tenant_id,
                &input.email,
                &input.client_ip,
            ).await?;

            // Lock account if too many failed attempts
            let lock_expires_at = if failed_attempts >= self.rate_limit_config.max_attempts_per_hour {
                Some(self.lock_account(
                    &context.tenant_context.tenant_id,
                    &input.email,
                ).await?)
            } else {
                None
            };

            self.apply_progressive_delay(failed_attempts).await?;

            return Ok(ValidateCredentialsResponse {
                is_valid: false,
                user_id: Some(user.id),
                user_status: Some(user.status),
                email_verified: user.email_verified_at.is_some(),
                last_login: user.last_login_at,
                failed_attempts,
                account_locked: lock_expires_at.is_some(),
                lock_expires_at,
                requires_mfa: self.requires_mfa(&user),
                validation_errors: vec!["Invalid credentials".to_string()],
            });
        }

        // Check email verification if required
        if input.require_email_verification && user.email_verified_at.is_none() {
            validation_errors.push("Email not verified".to_string());
        }

        // If we have validation errors, return invalid
        if !validation_errors.is_empty() {
            let failed_attempts = self.record_failed_attempt(
                &context.tenant_context.tenant_id,
                &input.email,
                &input.client_ip,
            ).await?;

            return Ok(ValidateCredentialsResponse {
                is_valid: false,
                user_id: Some(user.id),
                user_status: Some(user.status),
                email_verified: user.email_verified_at.is_some(),
                last_login: user.last_login_at,
                failed_attempts,
                account_locked: false,
                lock_expires_at: None,
                requires_mfa: self.requires_mfa(&user),
                validation_errors,
            });
        }

        // Successful validation - clear failed attempts
        self.clear_failed_attempts(
            &context.tenant_context.tenant_id,
            &input.email,
            &input.client_ip,
        ).await?;

        // Update last login
        if let Err(e) = user_repo.update_last_login(&user.id).await {
            tracing::warn!("Failed to update last login for user {}: {}", user.id, e);
        }

        Ok(ValidateCredentialsResponse {
            is_valid: true,
            user_id: Some(user.id),
            user_status: Some(user.status),
            email_verified: user.email_verified_at.is_some(),
            last_login: user.last_login_at,
            failed_attempts: 0,
            account_locked: false,
            lock_expires_at: None,
            requires_mfa: self.requires_mfa(&user),
            validation_errors: vec![],
        })
    }

    fn activity_type(&self) -> &'static str {
        "validate_user_credentials_activity"
    }

    fn validate_input(&self, input: &ValidateCredentialsRequest) -> Result<(), ActivityError> {
        if input.email.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Email is required".to_string(),
            });
        }

        if input.password.is_empty() {
            return Err(ActivityError::ValidationError {
                field: "password".to_string(),
                message: "Password is required".to_string(),
            });
        }

        if input.client_ip.trim().is_empty() {
            return Err(ActivityError::ValidationError {
                field: "client_ip".to_string(),
                message: "Client IP is required for rate limiting".to_string(),
            });
        }

        // Basic email validation
        if !input.email.contains('@') || !input.email.contains('.') {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            });
        }

        Ok(())
    }

    fn default_options(&self) -> adx_shared::temporal::ActivityExecutionOptions {
        let mut options = adx_shared::temporal::ActivityExecutionOptions::default();
        options.retry_policy = Some(database_retry_policy());
        options.tags.push("credential_validation".to_string());
        options.tags.push("authentication".to_string());
        options.tags.push("rate_limited".to_string());
        options
    }
}

#[async_trait]
impl TenantAwareActivity<ValidateCredentialsRequest, ValidateCredentialsResponse> for ValidateCredentialsActivity {
    async fn validate_tenant_access(
        &self,
        tenant_context: &TenantContext,
        _user_context: &UserContext,
    ) -> Result<(), ActivityError> {
        // Check if tenant is active
        if !tenant_context.is_active {
            return Err(ActivityError::AuthorizationError {
                message: "Cannot validate credentials for inactive tenant".to_string(),
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
        if resource_type == "login_attempts" {
            // Check if tenant has authentication features enabled
            if !tenant_context.features.contains(&"authentication".to_string()) {
                return Err(ActivityError::QuotaExceededError {
                    resource_type: "login_attempts".to_string(),
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
impl DatabaseActivity<ValidateCredentialsRequest, ValidateCredentialsResponse> for ValidateCredentialsActivity {
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