use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use adx_shared::{
    database::{DatabasePool, Repository},
    types::{TenantId, UserId, SubscriptionTier},
    Error, Result,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub tenant_id: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: UserStatus,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub preferences: serde_json::Value,
    pub last_login_at: Option<DateTime<Utc>>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
}

impl std::fmt::Display for UserStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserStatus::Active => write!(f, "active"),
            UserStatus::Inactive => write!(f, "inactive"),
            UserStatus::Suspended => write!(f, "suspended"),
            UserStatus::PendingVerification => write!(f, "pending_verification"),
        }
    }
}

impl std::str::FromStr for UserStatus {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "active" => Ok(UserStatus::Active),
            "inactive" => Ok(UserStatus::Inactive),
            "suspended" => Ok(UserStatus::Suspended),
            "pending_verification" => Ok(UserStatus::PendingVerification),
            _ => Err(Error::Validation(format!("Invalid user status: {}", s))),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub tenant_id: String,
    pub email: String,
    pub password_hash: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub roles: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
    pub preferences: Option<serde_json::Value>,
    pub email_verified_at: Option<DateTime<Utc>>,
}

pub struct UserRepository {
    pool: DatabasePool,
    tenant_id: TenantId,
}

impl UserRepository {
    pub fn new(pool: DatabasePool, tenant_id: TenantId) -> Self {
        Self { pool, tenant_id }
    }

    /// Find user by email within the current tenant
    pub async fn find_by_email(&self, email: &str) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
            SELECT id, tenant_id, email, password_hash, first_name, last_name,
                   status as "status: UserStatus", roles, permissions, preferences,
                   last_login_at, email_verified_at, created_at, updated_at
            FROM users 
            WHERE tenant_id = $1 AND email = $2
            "#,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            email
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(User {
                id: row.id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                email: row.email,
                password_hash: row.password_hash,
                first_name: row.first_name,
                last_name: row.last_name,
                status: row.status,
                roles: row.roles.unwrap_or_default(),
                permissions: row.permissions.unwrap_or_default(),
                preferences: row.preferences.unwrap_or_else(|| serde_json::json!({})),
                last_login_at: row.last_login_at,
                email_verified_at: row.email_verified_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    /// Update user's last login timestamp
    pub async fn update_last_login(&self, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET last_login_at = NOW(), updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Verify user's email
    pub async fn verify_email(&self, user_id: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET email_verified_at = NOW(), status = 'active', updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// Update user password
    pub async fn update_password(&self, user_id: &str, password_hash: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET password_hash = $3, updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(user_id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            password_hash
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        Ok(())
    }

    /// List users with pagination and filtering
    pub async fn list_with_filters(
        &self,
        status: Option<UserStatus>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> Result<Vec<User>> {
        let limit = limit.unwrap_or(50).min(100) as i64;
        let offset = offset.unwrap_or(0) as i64;

        let rows = match status {
            Some(status) => {
                sqlx::query!(
                    r#"
                    SELECT id, tenant_id, email, password_hash, first_name, last_name,
                           status as "status: UserStatus", roles, permissions, preferences,
                           last_login_at, email_verified_at, created_at, updated_at
                    FROM users 
                    WHERE tenant_id = $1 AND status = $2
                    ORDER BY created_at DESC
                    LIMIT $3 OFFSET $4
                    "#,
                    Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
                    status.to_string(),
                    limit,
                    offset
                )
                .fetch_all(&*self.pool)
                .await
                .map_err(|e| Error::Database(e.to_string()))?
            }
            None => {
                sqlx::query!(
                    r#"
                    SELECT id, tenant_id, email, password_hash, first_name, last_name,
                           status as "status: UserStatus", roles, permissions, preferences,
                           last_login_at, email_verified_at, created_at, updated_at
                    FROM users 
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
                .map_err(|e| Error::Database(e.to_string()))?
            }
        };

        let users = rows
            .into_iter()
            .map(|row| User {
                id: row.id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                email: row.email,
                password_hash: row.password_hash,
                first_name: row.first_name,
                last_name: row.last_name,
                status: row.status,
                roles: row.roles.unwrap_or_default(),
                permissions: row.permissions.unwrap_or_default(),
                preferences: row.preferences.unwrap_or_else(|| serde_json::json!({})),
                last_login_at: row.last_login_at,
                email_verified_at: row.email_verified_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect();

        Ok(users)
    }

    /// Count total users in tenant
    pub async fn count(&self, status: Option<UserStatus>) -> Result<i64> {
        let count = match status {
            Some(status) => {
                sqlx::query!(
                    "SELECT COUNT(*) as count FROM users WHERE tenant_id = $1 AND status = $2",
                    Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
                    status.to_string()
                )
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| Error::Database(e.to_string()))?
                .count
                .unwrap_or(0)
            }
            None => {
                sqlx::query!(
                    "SELECT COUNT(*) as count FROM users WHERE tenant_id = $1",
                    Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
                )
                .fetch_one(&*self.pool)
                .await
                .map_err(|e| Error::Database(e.to_string()))?
                .count
                .unwrap_or(0)
            }
        };

        Ok(count)
    }
}

#[async_trait]
impl Repository<User> for UserRepository {
    async fn create(&self, request: User) -> Result<User> {
        let user_id = Uuid::new_v4();
        let tenant_uuid = Uuid::parse_str(&self.tenant_id)
            .map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?;

        sqlx::query!(
            r#"
            INSERT INTO users (id, tenant_id, email, password_hash, first_name, last_name, 
                             status, roles, permissions, preferences)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            user_id,
            tenant_uuid,
            request.email,
            request.password_hash,
            request.first_name,
            request.last_name,
            request.status.to_string(),
            &request.roles,
            &request.permissions,
            request.preferences
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Fetch the created user
        self.find_by_id(&user_id.to_string()).await?
            .ok_or_else(|| Error::Internal("Failed to fetch created user".to_string()))
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<User>> {
        let row = sqlx::query!(
            r#"
            SELECT id, tenant_id, email, password_hash, first_name, last_name,
                   status as "status: UserStatus", roles, permissions, preferences,
                   last_login_at, email_verified_at, created_at, updated_at
            FROM users 
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        match row {
            Some(row) => Ok(Some(User {
                id: row.id.to_string(),
                tenant_id: row.tenant_id.to_string(),
                email: row.email,
                password_hash: row.password_hash,
                first_name: row.first_name,
                last_name: row.last_name,
                status: row.status,
                roles: row.roles.unwrap_or_default(),
                permissions: row.permissions.unwrap_or_default(),
                preferences: row.preferences.unwrap_or_else(|| serde_json::json!({})),
                last_login_at: row.last_login_at,
                email_verified_at: row.email_verified_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })),
            None => Ok(None),
        }
    }

    async fn update(&self, user: User) -> Result<User> {
        sqlx::query!(
            r#"
            UPDATE users 
            SET first_name = $3, last_name = $4, status = $5, roles = $6, 
                permissions = $7, preferences = $8, updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            "#,
            Uuid::parse_str(&user.id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?,
            user.first_name,
            user.last_name,
            user.status.to_string(),
            &user.roles,
            &user.permissions,
            user.preferences
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        // Fetch the updated user
        self.find_by_id(&user.id).await?
            .ok_or_else(|| Error::Internal("Failed to fetch updated user".to_string()))
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1 AND tenant_id = $2",
            Uuid::parse_str(id).map_err(|e| Error::Validation(format!("Invalid user ID: {}", e)))?,
            Uuid::parse_str(&self.tenant_id).map_err(|e| Error::Validation(format!("Invalid tenant ID: {}", e)))?
        )
        .execute(&*self.pool)
        .await
        .map_err(|e| Error::Database(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(Error::NotFound("User not found".to_string()));
        }

        Ok(())
    }

    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<User>> {
        self.list_with_filters(None, limit, offset).await
    }
}

// Tests disabled for now due to SQLx macro compilation issues
// TODO: Re-enable tests once DATABASE_URL is properly configured
// #[cfg(test)]
// mod tests { ... }