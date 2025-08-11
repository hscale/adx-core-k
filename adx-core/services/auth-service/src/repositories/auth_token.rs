use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use adx_shared::{
    database::{DatabasePool, Repository},
    types::{TenantId, UserId},
    Error, Result,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordResetToken {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
    pub used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailVerificationToken {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub token: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatePasswordResetTokenRequest {
    pub user_id: String,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateEmailVerificationTokenRequest {
    pub user_id: String,
    pub token: String,
    pub email: String,
    pub expires_at: DateTime<Utc>,
}

pub struct AuthTokenRepository {
    pool: DatabasePool,
    tenant_id: TenantId,
}

impl AuthTokenRepository {
    pub fn new(pool: DatabasePool, tenant_id: TenantId) -> Self {
        Self { pool, tenant_id }
    }

    // Password Reset Token Methods

    /// Create a password reset token
    pub async fn create_password_reset_token(
        &self,
        request: CreatePasswordResetTokenRequest,
    ) -> Result<PasswordResetToken> {
        let token_id = Uuid::new_v4();
        let tenant_uuid = Uuid::parse_str(&self.tenant_id)
            .map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?;
        let user_uuid = Uuid::parse_str(&request.user_id)
            .map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO password_reset_tokens (id, user_id, tenant_id, token, expires_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            token_id,
            user_uuid,
            tenant_uuid,
            request.token,
            request.expires_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Fetch the created token
        self.find_password_reset_token_by_id(&token_id.to_string()).await?
            .ok_or_else(|| Error::Internal("Failed to fetch created password reset token".to_string()))
    }

    /// Find password reset token by token string
    pub async fn find_password_reset_token_by_token(&self, token: &str) -> Result<Option<PasswordResetToken>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, token, expires_at, used_at, created_at
            FROM password_reset_tokens 
            WHERE tenant_id = $1 AND token = $2
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            token
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(PasswordResetToken {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                token: row.token,
                expires_at: row.expires_at,
                used_at: row.used_at,
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    /// Find password reset token by ID
    pub async fn find_password_reset_token_by_id(&self, id: &str) -> Result<Option<PasswordResetToken>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, token, expires_at, used_at, created_at
            FROM password_reset_tokens 
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(id).map_err(|e| Error::Validation(format!("Invalid token ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(PasswordResetToken {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                token: row.token,
                expires_at: row.expires_at,
                used_at: row.used_at,
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    /// Validate and use password reset token
    pub async fn validate_and_use_password_reset_token(&self, token: &str) -> Result<Option<PasswordResetToken>> {
        let reset_token = self.find_password_reset_token_by_token(token).await?;
        
        match reset_token {
            Some(token_data) => {
                // Check if token is expired or already used
                if token_data.expires_at <= Utc::now() {
                    return Err(Error::Validation("Password reset token has expired".to_string()));
                }
                
                if token_data.used_at.is_some() {
                    return Err(Error::Validation("Password reset token has already been used".to_string()));
                }

                // Mark token as used
                sqlx::query!(
                    r#"
                    UPDATE password_reset_tokens 
                    SET used_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                    Uuid::parse_str(&token_data.id).map_err(|e| Error::Validation(format!("Invalid token ID: {}", e)))?,
                    Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
                )
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                Ok(Some(token_data))
            }
            None => Ok(None),
        }
    }

    /// Invalidate all password reset tokens for a user
    pub async fn invalidate_user_password_reset_tokens(&self, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE password_reset_tokens 
            SET used_at = NOW()
            WHERE user_id = $1 AND tenant_id = $2 AND used_at IS NULL
            "#,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    // Email Verification Token Methods

    /// Create an email verification token
    pub async fn create_email_verification_token(
        &self,
        request: CreateEmailVerificationTokenRequest,
    ) -> Result<EmailVerificationToken> {
        let token_id = Uuid::new_v4();
        let tenant_uuid = Uuid::parse_str(&self.tenant_id)
            .map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?;
        let user_uuid = Uuid::parse_str(&request.user_id)
            .map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO email_verification_tokens (id, user_id, tenant_id, token, email, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            token_id,
            user_uuid,
            tenant_uuid,
            request.token,
            request.email,
            request.expires_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Fetch the created token
        self.find_email_verification_token_by_id(&token_id.to_string()).await?
            .ok_or_else(|| Error::Internal("Failed to fetch created email verification token".to_string()))
    }

    /// Find email verification token by token string
    pub async fn find_email_verification_token_by_token(&self, token: &str) -> Result<Option<EmailVerificationToken>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, token, email, expires_at, verified_at, created_at
            FROM email_verification_tokens 
            WHERE tenant_id = $1 AND token = $2
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            token
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(EmailVerificationToken {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                token: row.token,
                email: row.email,
                expires_at: row.expires_at,
                verified_at: row.verified_at,
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    /// Find email verification token by ID
    pub async fn find_email_verification_token_by_id(&self, id: &str) -> Result<Option<EmailVerificationToken>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, token, email, expires_at, verified_at, created_at
            FROM email_verification_tokens 
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(id).map_err(|e| Error::Validation(format!("Invalid token ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(EmailVerificationToken {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                token: row.token,
                email: row.email,
                expires_at: row.expires_at,
                verified_at: row.verified_at,
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    /// Validate and use email verification token
    pub async fn validate_and_use_email_verification_token(&self, token: &str) -> Result<Option<EmailVerificationToken>> {
        let verification_token = self.find_email_verification_token_by_token(token).await?;
        
        match verification_token {
            Some(token_data) => {
                // Check if token is expired or already used
                if token_data.expires_at <= Utc::now() {
                    return Err(Error::Validation("Email verification token has expired".to_string()));
                }
                
                if token_data.verified_at.is_some() {
                    return Err(Error::Validation("Email verification token has already been used".to_string()));
                }

                // Mark token as verified
                sqlx::query!(
                    r#"
                    UPDATE email_verification_tokens 
                    SET verified_at = NOW()
                    WHERE id = $1 AND tenant_id = $2
                    "#,
                    Uuid::parse_str(&token_data.id).map_err(|e| Error::Validation(format!("Invalid token ID: {}", e)))?,
                    Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
                )
                .execute(&*self.pool)
                .await
                .map_err(|e| Error::Database(e.to_string()))?;

                Ok(Some(token_data))
            }
            None => Ok(None),
        }
    }

    /// Find pending email verification tokens for a user
    pub async fn find_pending_email_verification_tokens(&self, user_id: &str) -> Result<Vec<EmailVerificationToken>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, token, email, expires_at, verified_at, created_at
            FROM email_verification_tokens 
            WHERE tenant_id = $1 AND user_id = $2 AND verified_at IS NULL AND expires_at > NOW()
            ORDER BY created_at DESC
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        let tokens = rows
            .into_iter()
            .map(|row| EmailVerificationToken {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                token: row.token,
                email: row.email,
                expires_at: row.expires_at,
                verified_at: row.verified_at,
                created_at: row.created_at,
            })
            .collect();

        Ok(tokens)
    }

    /// Invalidate all email verification tokens for a user
    pub async fn invalidate_user_email_verification_tokens(&self, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE email_verification_tokens 
            SET verified_at = NOW()
            WHERE user_id = $1 AND tenant_id = $2 AND verified_at IS NULL
            "#,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Clean up expired tokens
    pub async fn cleanup_expired_tokens(&self) -> Result<(u64, u64)> {
        // Clean up expired password reset tokens
        let password_reset_result = sqlx::query!(
            r#"
            DELETE FROM password_reset_tokens 
            WHERE tenant_id = $1 AND expires_at <= NOW()
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Clean up expired email verification tokens
        let email_verification_result = sqlx::query!(
            r#"
            DELETE FROM email_verification_tokens 
            WHERE tenant_id = $1 AND expires_at <= NOW()
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok((password_reset_result.rows_affected(), email_verification_result.rows_affected()))
    }
}

// Tests disabled for now due to SQLx macro compilation issues
// TODO: Re-enable tests once DATABASE_URL is properly configured
// #[cfg(test)]
// mod tests { ... }