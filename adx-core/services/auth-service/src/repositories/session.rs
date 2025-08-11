use async_trait::async_trait;
use chrono::{DateTime, Utc, Duration};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use adx_shared::{
    database::{DatabasePool, Repository},
    types::{TenantId, UserId, SessionId},
    Error, Result,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub session_token: String,
    pub refresh_token: String,
    pub status: SessionStatus,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub expires_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    Active,
    Expired,
    Revoked,
}

impl std::fmt::Display for SessionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionStatus::Active => write!(f, "active"),
            SessionStatus::Expired => write!(f, "expired"),
            SessionStatus::Revoked => write!(f, "revoked"),
        }
    }
}

impl std::str::FromStr for SessionStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(SessionStatus::Active),
            "expired" => Ok(SessionStatus::Expired),
            "revoked" => Ok(SessionStatus::Revoked),
            _ => Err(Error::Validation(format!("Invalid session status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSessionRequest {
    pub user_id: String,
    pub session_token: String,
    pub refresh_token: String,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub device_id: Option<String>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSessionRequest {
    pub status: Option<SessionStatus>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

pub struct SessionRepository {
    pool: DatabasePool,
    tenant_id: TenantId,
}

impl SessionRepository {
    pub fn new(pool: DatabasePool, tenant_id: TenantId) -> Self {
        Self { pool, tenant_id }
    }

    /// Find session by session token
    pub async fn find_by_session_token(&self, session_token: &str) -> Result<Option<UserSession>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, session_token, refresh_token,
                   status as "status: SessionStatus", ip_address, user_agent, device_id,
                   expires_at, last_activity_at, created_at
            FROM user_sessions 
            WHERE tenant_id = $1 AND session_token = $2
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            session_token
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(UserSession {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                session_token: row.session_token,
                refresh_token: row.refresh_token,
                status: row.status,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                device_id: row.device_id,
                expires_at: row.expires_at,
                last_activity_at: row.last_activity_at,
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    /// Find session by refresh token
    pub async fn find_by_refresh_token(&self, refresh_token: &str) -> Result<Option<UserSession>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, session_token, refresh_token,
                   status as "status: SessionStatus", ip_address, user_agent, device_id,
                   expires_at, last_activity_at, created_at
            FROM user_sessions 
            WHERE tenant_id = $1 AND refresh_token = $2
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            refresh_token
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(UserSession {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                session_token: row.session_token,
                refresh_token: row.refresh_token,
                status: row.status,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                device_id: row.device_id,
                expires_at: row.expires_at,
                last_activity_at: row.last_activity_at,
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    /// Find active sessions for a user
    pub async fn find_active_sessions_for_user(&self, user_id: &str) -> Result<Vec<UserSession>> {
        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, session_token, refresh_token,
                   status as "status: SessionStatus", ip_address, user_agent, device_id,
                   expires_at, last_activity_at, created_at
            FROM user_sessions 
            WHERE tenant_id = $1 AND user_id = $2 AND status = 'active' AND expires_at > NOW()
            ORDER BY last_activity_at DESC
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        let sessions = rows
            .into_iter()
            .map(|row| UserSession {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                session_token: row.session_token,
                refresh_token: row.refresh_token,
                status: row.status,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                device_id: row.device_id,
                expires_at: row.expires_at,
                last_activity_at: row.last_activity_at,
                created_at: row.created_at,
            })
            .collect();

        Ok(sessions)
    }

    /// Update session activity timestamp
    pub async fn update_activity(&self, session_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE user_sessions 
            SET last_activity_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(session_id).map_err(|e| Error::Validation(format!("Invalid session ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Revoke session
    pub async fn revoke_session(&self, session_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE user_sessions 
            SET status = 'revoked'
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(session_id).map_err(|e| Error::Validation(format!("Invalid session ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Revoke all sessions for a user
    pub async fn revoke_all_user_sessions(&self, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE user_sessions 
            SET status = 'revoked'
            WHERE user_id = $1 AND tenant_id = $2 AND status = 'active'
            "#,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Clean up expired sessions
    pub async fn cleanup_expired_sessions(&self) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            UPDATE user_sessions 
            SET status = 'expired'
            WHERE tenant_id = $1 AND status = 'active' AND expires_at <= NOW()
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Delete old sessions (older than specified days)
    pub async fn delete_old_sessions(&self, days_old: i64) -> Result<u64> {
        let result = sqlx::query!(
            r#"
            DELETE FROM user_sessions 
            WHERE tenant_id = $1 AND created_at < NOW() - INTERVAL '%s days'
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            days_old
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(result.rows_affected())
    }

    /// Validate session and check if it's active and not expired
    pub async fn validate_session(&self, session_token: &str) -> Result<Option<UserSession>> {
        let session = self.find_by_session_token(session_token).await?;
        
        match session {
            Some(session) => {
                if session.status == SessionStatus::Active && session.expires_at > Utc::now() {
                    // Update activity timestamp
                    self.update_activity(&session.id).await?;
                    Ok(Some(session))
                } else {
                    // Mark as expired if needed
                    if session.status == SessionStatus::Active && session.expires_at <= Utc::now() {
                        self.revoke_session(&session.id).await?;
                    }
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    /// Create session from request
    pub async fn create_from_request(&self, request: CreateSessionRequest) -> Result<UserSession> {
        let session_id = Uuid::new_v4();
        let tenant_uuid = Uuid::parse_str(&self.tenant_id)
            .map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?;
        let user_uuid = Uuid::parse_str(&request.user_id)
            .map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO user_sessions (id, user_id, tenant_id, session_token, refresh_token,
                                     status, ip_address, user_agent, device_id, expires_at)
            VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9)
            "#,
            session_id,
            user_uuid,
            tenant_uuid,
            request.session_token,
            request.refresh_token,
            request.ip_address,
            request.user_agent,
            request.device_id,
            request.expires_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Fetch the created session
        self.find_by_id(&session_id.to_string()).await?
            .ok_or_else(|| Error::Internal("Failed to fetch created session".to_string()))
    }
}

#[async_trait]
impl Repository<UserSession> for SessionRepository {
    async fn create(&self, session: UserSession) -> Result<UserSession> {
        let session_id = Uuid::new_v4();
        let tenant_uuid = Uuid::parse_str(&self.tenant_id)
            .map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?;
        let user_uuid = Uuid::parse_str(&session.user_id)
            .map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO user_sessions (id, user_id, tenant_id, session_token, refresh_token,
                                     status, ip_address, user_agent, device_id, expires_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            session_id,
            user_uuid,
            tenant_uuid,
            session.session_token,
            session.refresh_token,
            session.status.to_string(),
            session.ip_address,
            session.user_agent,
            session.device_id,
            session.expires_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Fetch the created session
        self.find_by_id(&session_id.to_string()).await?
            .ok_or_else(|| Error::Internal("Failed to fetch created session".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<UserSession>> {
        let row = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, session_token, refresh_token,
                   status as "status: SessionStatus", ip_address, user_agent, device_id,
                   expires_at, last_activity_at, created_at
            FROM user_sessions 
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(id).map_err(|e| Error::Validation(format!("Invalid session ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(UserSession {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                session_token: row.session_token,
                refresh_token: row.refresh_token,
                status: row.status,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                device_id: row.device_id,
                expires_at: row.expires_at,
                last_activity_at: row.last_activity_at,
                created_at: row.created_at,
            })),
            None => Ok(None),
        }
    }

    async fn update(&self, session: UserSession) -> Result<UserSession> {
        sqlx::query!(
            r#"
            UPDATE user_sessions 
            SET status = $3, expires_at = $4, last_activity_at = $5
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(&session.id).map_err(|e| Error::Validation(format!("Invalid session ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            session.status.to_string(),
            session.expires_at,
            session.last_activity_at
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Fetch the updated session
        self.find_by_id(&session.id).await?
            .ok_or_else(|| Error::Internal("Failed to fetch updated session".to_string()))
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM user_sessions WHERE id = $1 AND tenant_id = $2",
            Uuid::parse_str(id).map_err(|e| Error::Validation(format!("Invalid session ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Session not found".to_string()));
        }

        Ok(())
    }

    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<UserSession>> {
        let limit = limit.unwrap_or(50).min(100) as i64;
        let offset = offset.unwrap_or(0) as i64;

        let rows = sqlx::query!(
            r#"
            SELECT id, user_id, tenant_id, session_token, refresh_token,
                   status as "status: SessionStatus", ip_address, user_agent, device_id,
                   expires_at, last_activity_at, created_at
            FROM user_sessions 
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            limit,
            offset
        )
        .fetch_all(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        let sessions = rows
            .into_iter()
            .map(|row| UserSession {
                id: row.id.to_string(),
                user_id: row.user_id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                session_token: row.session_token,
                refresh_token: row.refresh_token,
                status: row.status,
                ip_address: row.ip_address,
                user_agent: row.user_agent,
                device_id: row.device_id,
                expires_at: row.expires_at,
                last_activity_at: row.last_activity_at,
                created_at: row.created_at,
            })
            .collect();

        Ok(sessions)
    }
}

// Tests disabled for now due to SQLx macro compilation issues
// TODO: Re-enable tests once DATABASE_URL is properly configured
// #[cfg(test)]
// mod tests { ... }