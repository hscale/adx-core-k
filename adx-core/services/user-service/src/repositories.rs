use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use std::collections::HashMap;
use adx_shared::{Result, Error, TenantContext};
use crate::models::*;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, tenant_id: Uuid, user_id: Uuid) -> Result<Option<User>>;
    async fn find_by_email(&self, tenant_id: Uuid, email: &str) -> Result<Option<User>>;
    async fn create(&self, tenant_id: Uuid, user: CreateUserRequest) -> Result<User>;
    async fn update(&self, tenant_id: Uuid, user_id: Uuid, updates: UpdateUserRequest) -> Result<User>;
    async fn delete(&self, tenant_id: Uuid, user_id: Uuid) -> Result<()>;
    async fn list(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<User>>;
    async fn search(&self, tenant_id: Uuid, request: UserSearchRequest) -> Result<UserSearchResponse>;
    async fn get_directory(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<UserDirectoryResponse>;
}

#[async_trait]
pub trait UserProfileRepository: Send + Sync {
    async fn find_by_user_id(&self, tenant_id: Uuid, user_id: Uuid) -> Result<Option<UserProfile>>;
    async fn create(&self, tenant_id: Uuid, user_id: Uuid, profile: CreateUserProfileRequest) -> Result<UserProfile>;
    async fn update(&self, tenant_id: Uuid, user_id: Uuid, updates: UpdateUserProfileRequest) -> Result<UserProfile>;
    async fn delete(&self, tenant_id: Uuid, user_id: Uuid) -> Result<()>;
}

#[async_trait]
pub trait UserPreferenceRepository: Send + Sync {
    async fn get_preferences(&self, tenant_id: Uuid, user_id: Uuid, category: Option<&str>) -> Result<Vec<UserPreference>>;
    async fn find_by_category(&self, tenant_id: Uuid, user_id: Uuid, category: &str) -> Result<Vec<UserPreference>>;
    async fn set_preference(&self, tenant_id: Uuid, user_id: Uuid, category: &str, key: &str, value: serde_json::Value) -> Result<UserPreference>;
    async fn set_preferences(&self, tenant_id: Uuid, user_id: Uuid, request: UserPreferenceRequest) -> Result<Vec<UserPreference>>;
    async fn delete_preference(&self, tenant_id: Uuid, user_id: Uuid, category: &str, key: &str) -> Result<()>;
}

#[async_trait]
pub trait UserActivityRepository: Send + Sync {
    async fn log_activity(&self, activity: UserActivityLog) -> Result<()>;
    async fn get_user_activity(&self, tenant_id: Uuid, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<UserActivityLog>>;
}

// PostgreSQL implementations
pub struct PostgresUserRepository {
    pool: PgPool,
}

impl PostgresUserRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    async fn set_tenant_context(&self, tenant_id: Uuid) -> Result<()> {
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&self.pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, tenant_id: Uuid, user_id: Uuid) -> Result<Option<User>> {
        self.set_tenant_context(tenant_id).await?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, password_hash, first_name, last_name,
                   status as "status: UserStatus", roles, permissions, preferences,
                   last_login_at, email_verified_at, created_at, updated_at
            FROM users 
            WHERE id = $1 AND tenant_id = $2
            "#,
            user_id,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(user)
    }
    
    async fn find_by_email(&self, tenant_id: Uuid, email: &str) -> Result<Option<User>> {
        self.set_tenant_context(tenant_id).await?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, password_hash, first_name, last_name,
                   status as "status: UserStatus", roles, permissions, preferences,
                   last_login_at, email_verified_at, created_at, updated_at
            FROM users 
            WHERE email = $1 AND tenant_id = $2
            "#,
            email,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(user)
    }
    
    async fn create(&self, tenant_id: Uuid, request: CreateUserRequest) -> Result<User> {
        self.set_tenant_context(tenant_id).await?;
        
        // Hash password
        let password_hash = bcrypt::hash(&request.password, bcrypt::DEFAULT_COST)
            .map_err(|e| Error::Internal(format!("Failed to hash password: {}", e)))?;
        
        let roles = request.roles.unwrap_or_else(|| vec!["user".to_string()]);
        
        let user = sqlx::query_as!(
            User,
            r#"
            INSERT INTO users (tenant_id, email, password_hash, first_name, last_name, roles, status)
            VALUES ($1, $2, $3, $4, $5, $6, 'pending_verification')
            RETURNING id, tenant_id, email, password_hash, first_name, last_name,
                      status as "status: UserStatus", roles, permissions, preferences,
                      last_login_at, email_verified_at, created_at, updated_at
            "#,
            tenant_id,
            request.email,
            password_hash,
            request.first_name,
            request.last_name,
            &roles
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(user)
    }
    
    async fn update(&self, tenant_id: Uuid, user_id: Uuid, updates: UpdateUserRequest) -> Result<User> {
        self.set_tenant_context(tenant_id).await?;
        
        let user = sqlx::query_as!(
            User,
            r#"
            UPDATE users 
            SET first_name = COALESCE($3, first_name),
                last_name = COALESCE($4, last_name),
                status = COALESCE($5, status),
                roles = COALESCE($6, roles),
                permissions = COALESCE($7, permissions),
                updated_at = NOW()
            WHERE id = $1 AND tenant_id = $2
            RETURNING id, tenant_id, email, password_hash, first_name, last_name,
                      status as "status: UserStatus", roles, permissions, preferences,
                      last_login_at, email_verified_at, created_at, updated_at
            "#,
            user_id,
            tenant_id,
            updates.first_name,
            updates.last_name,
            updates.status.map(|s| s as i32),
            updates.roles.as_deref(),
            updates.permissions.as_deref()
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(user)
    }
    
    async fn delete(&self, tenant_id: Uuid, user_id: Uuid) -> Result<()> {
        self.set_tenant_context(tenant_id).await?;
        
        let result = sqlx::query!(
            "DELETE FROM users WHERE id = $1 AND tenant_id = $2",
            user_id,
            tenant_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        if result.rows_affected() == 0 {
            return Err(Error::NotFound("User not found".to_string()));
        }
        
        Ok(())
    }
    
    async fn list(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<User>> {
        self.set_tenant_context(tenant_id).await?;
        
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, password_hash, first_name, last_name,
                   status as "status: UserStatus", roles, permissions, preferences,
                   last_login_at, email_verified_at, created_at, updated_at
            FROM users 
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(users)
    }
    
    async fn search(&self, tenant_id: Uuid, request: UserSearchRequest) -> Result<UserSearchResponse> {
        self.set_tenant_context(tenant_id).await?;
        
        // Simplified search implementation
        let limit = request.limit.unwrap_or(50).min(100);
        let offset = request.offset.unwrap_or(0);
        
        let users = sqlx::query_as!(
            User,
            r#"
            SELECT id, tenant_id, email, password_hash, first_name, last_name,
                   status as "status: UserStatus", roles, permissions, preferences,
                   last_login_at, email_verified_at, created_at, updated_at
            FROM users 
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        let user_with_profiles: Vec<UserWithProfile> = users
            .into_iter()
            .map(|user| UserWithProfile { user, profile: None })
            .collect();
        
        // Get total count
        let total_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?
        .unwrap_or(0);
        
        Ok(UserSearchResponse {
            users: user_with_profiles,
            total_count,
            has_more: (offset + limit) < total_count,
        })
    }
    
    async fn get_directory(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<UserDirectoryResponse> {
        self.set_tenant_context(tenant_id).await?;
        
        let entries = sqlx::query!(
            r#"
            SELECT u.id, 
                   COALESCE(p.display_name, CONCAT(u.first_name, ' ', u.last_name)) as display_name,
                   u.email, p.job_title, p.department, p.avatar_url,
                   u.status as "status: UserStatus", u.last_login_at
            FROM users u
            LEFT JOIN user_profiles p ON u.id = p.user_id AND u.tenant_id = p.tenant_id
            WHERE u.tenant_id = $1 AND u.status = 'active'
            ORDER BY display_name
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        let directory_entries: Vec<UserDirectoryEntry> = entries
            .into_iter()
            .map(|row| UserDirectoryEntry {
                id: row.id,
                display_name: row.display_name.unwrap_or_else(|| "Unknown User".to_string()),
                email: row.email,
                job_title: row.job_title,
                department: row.department,
                avatar_url: row.avatar_url,
                status: row.status,
                last_login_at: row.last_login_at,
            })
            .collect();
        
        // Get total count
        let total_count = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM users WHERE tenant_id = $1 AND status = 'active'",
            tenant_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?
        .unwrap_or(0);
        
        // Get departments and roles for filtering
        let departments = sqlx::query_scalar!(
            "SELECT DISTINCT department FROM user_profiles WHERE tenant_id = $1 AND department IS NOT NULL",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        let roles = sqlx::query!(
            "SELECT DISTINCT unnest(roles) as role FROM users WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?
        .into_iter()
        .map(|row| row.role.unwrap_or_default())
        .collect();
        
        Ok(UserDirectoryResponse {
            entries: directory_entries,
            total_count,
            departments,
            roles,
        })
    }
}

pub struct PostgresUserProfileRepository {
    pool: PgPool,
}

impl PostgresUserProfileRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    async fn set_tenant_context(&self, tenant_id: Uuid) -> Result<()> {
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&self.pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[async_trait]
impl UserProfileRepository for PostgresUserProfileRepository {
    async fn find_by_user_id(&self, tenant_id: Uuid, user_id: Uuid) -> Result<Option<UserProfile>> {
        self.set_tenant_context(tenant_id).await?;
        
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            SELECT id, user_id, tenant_id, display_name, bio, avatar_url, cover_image_url,
                   location, website_url, timezone, language, date_format, time_format,
                   phone_number, phone_verified_at, birth_date, gender, job_title,
                   department, manager_id, hire_date, created_at, updated_at
            FROM user_profiles 
            WHERE user_id = $1 AND tenant_id = $2
            "#,
            user_id,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(profile)
    }
    
    async fn create(&self, tenant_id: Uuid, user_id: Uuid, request: CreateUserProfileRequest) -> Result<UserProfile> {
        self.set_tenant_context(tenant_id).await?;
        
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            INSERT INTO user_profiles (
                user_id, tenant_id, display_name, bio, location, website_url,
                timezone, language, phone_number, job_title, department, manager_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING id, user_id, tenant_id, display_name, bio, avatar_url, cover_image_url,
                      location, website_url, timezone, language, date_format, time_format,
                      phone_number, phone_verified_at, birth_date, gender, job_title,
                      department, manager_id, hire_date, created_at, updated_at
            "#,
            user_id,
            tenant_id,
            request.display_name,
            request.bio,
            request.location,
            request.website_url,
            request.timezone.unwrap_or_else(|| "UTC".to_string()),
            request.language.unwrap_or_else(|| "en".to_string()),
            request.phone_number,
            request.job_title,
            request.department,
            request.manager_id
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(profile)
    }
    
    async fn update(&self, tenant_id: Uuid, user_id: Uuid, updates: UpdateUserProfileRequest) -> Result<UserProfile> {
        self.set_tenant_context(tenant_id).await?;
        
        let profile = sqlx::query_as!(
            UserProfile,
            r#"
            UPDATE user_profiles 
            SET display_name = COALESCE($3, display_name),
                bio = COALESCE($4, bio),
                avatar_url = COALESCE($5, avatar_url),
                cover_image_url = COALESCE($6, cover_image_url),
                location = COALESCE($7, location),
                website_url = COALESCE($8, website_url),
                timezone = COALESCE($9, timezone),
                language = COALESCE($10, language),
                date_format = COALESCE($11, date_format),
                time_format = COALESCE($12, time_format),
                phone_number = COALESCE($13, phone_number),
                birth_date = COALESCE($14, birth_date),
                gender = COALESCE($15, gender),
                job_title = COALESCE($16, job_title),
                department = COALESCE($17, department),
                manager_id = COALESCE($18, manager_id),
                hire_date = COALESCE($19, hire_date),
                updated_at = NOW()
            WHERE user_id = $1 AND tenant_id = $2
            RETURNING id, user_id, tenant_id, display_name, bio, avatar_url, cover_image_url,
                      location, website_url, timezone, language, date_format, time_format,
                      phone_number, phone_verified_at, birth_date, gender, job_title,
                      department, manager_id, hire_date, created_at, updated_at
            "#,
            user_id,
            tenant_id,
            updates.display_name,
            updates.bio,
            updates.avatar_url,
            updates.cover_image_url,
            updates.location,
            updates.website_url,
            updates.timezone,
            updates.language,
            updates.date_format,
            updates.time_format,
            updates.phone_number,
            updates.birth_date,
            updates.gender,
            updates.job_title,
            updates.department,
            updates.manager_id,
            updates.hire_date
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(profile)
    }
    
    async fn delete(&self, tenant_id: Uuid, user_id: Uuid) -> Result<()> {
        self.set_tenant_context(tenant_id).await?;
        
        let result = sqlx::query!(
            "DELETE FROM user_profiles WHERE user_id = $1 AND tenant_id = $2",
            user_id,
            tenant_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        if result.rows_affected() == 0 {
            return Err(Error::NotFound("User profile not found".to_string()));
        }
        
        Ok(())
    }
}

pub struct PostgresUserPreferenceRepository {
    pool: PgPool,
}

impl PostgresUserPreferenceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    async fn set_tenant_context(&self, tenant_id: Uuid) -> Result<()> {
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&self.pool)
            .await
            .map_err(Error::Database)?;
        Ok(())
    }
}

#[async_trait]
impl UserPreferenceRepository for PostgresUserPreferenceRepository {
    async fn get_preferences(&self, tenant_id: Uuid, user_id: Uuid, category: Option<&str>) -> Result<Vec<UserPreference>> {
        self.set_tenant_context(tenant_id).await?;
        
        let preferences = if let Some(cat) = category {
            sqlx::query_as!(
                UserPreference,
                r#"
                SELECT id, user_id, tenant_id, preference_category, preference_key,
                       preference_value, is_inherited, created_at, updated_at
                FROM user_preferences 
                WHERE user_id = $1 AND tenant_id = $2 AND preference_category = $3
                ORDER BY preference_key
                "#,
                user_id,
                tenant_id,
                cat
            )
            .fetch_all(&self.pool)
            .await
        } else {
            sqlx::query_as!(
                UserPreference,
                r#"
                SELECT id, user_id, tenant_id, preference_category, preference_key,
                       preference_value, is_inherited, created_at, updated_at
                FROM user_preferences 
                WHERE user_id = $1 AND tenant_id = $2
                ORDER BY preference_category, preference_key
                "#,
                user_id,
                tenant_id
            )
            .fetch_all(&self.pool)
            .await
        };
        
        preferences.map_err(Error::Database)
    }
    
    async fn find_by_category(&self, tenant_id: Uuid, user_id: Uuid, category: &str) -> Result<Vec<UserPreference>> {
        self.get_preferences(tenant_id, user_id, Some(category)).await
    }
    
    async fn set_preference(&self, tenant_id: Uuid, user_id: Uuid, category: &str, key: &str, value: serde_json::Value) -> Result<UserPreference> {
        self.set_tenant_context(tenant_id).await?;
        
        let preference = sqlx::query_as!(
            UserPreference,
            r#"
            INSERT INTO user_preferences (user_id, tenant_id, preference_category, preference_key, preference_value)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id, tenant_id, preference_category, preference_key)
            DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = NOW()
            RETURNING id, user_id, tenant_id, preference_category, preference_key,
                      preference_value, is_inherited, created_at, updated_at
            "#,
            user_id,
            tenant_id,
            category,
            key,
            value
        )
        .fetch_one(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(preference)
    }
    
    async fn set_preferences(&self, tenant_id: Uuid, user_id: Uuid, request: UserPreferenceRequest) -> Result<Vec<UserPreference>> {
        let mut preferences = Vec::new();
        
        for (key, value) in request.preferences {
            let preference = self.set_preference(tenant_id, user_id, &request.preference_category, &key, value).await?;
            preferences.push(preference);
        }
        
        Ok(preferences)
    }
    
    async fn delete_preference(&self, tenant_id: Uuid, user_id: Uuid, category: &str, key: &str) -> Result<()> {
        self.set_tenant_context(tenant_id).await?;
        
        let result = sqlx::query!(
            "DELETE FROM user_preferences WHERE user_id = $1 AND tenant_id = $2 AND preference_category = $3 AND preference_key = $4",
            user_id,
            tenant_id,
            category,
            key
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        if result.rows_affected() == 0 {
            return Err(Error::NotFound("Preference not found".to_string()));
        }
        
        Ok(())
    }
}

pub struct PostgresUserActivityRepository {
    pool: PgPool,
}

impl PostgresUserActivityRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserActivityRepository for PostgresUserActivityRepository {
    async fn log_activity(&self, activity: UserActivityLog) -> Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO user_activity_log (
                user_id, tenant_id, activity_type, activity_description,
                resource_type, resource_id, metadata, ip_address, user_agent, session_id
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
            activity.user_id,
            activity.tenant_id,
            activity.activity_type,
            activity.activity_description,
            activity.resource_type,
            activity.resource_id,
            activity.metadata,
            activity.ip_address,
            activity.user_agent,
            activity.session_id
        )
        .execute(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(())
    }
    
    async fn get_user_activity(&self, tenant_id: Uuid, user_id: Uuid, limit: i64, offset: i64) -> Result<Vec<UserActivityLog>> {
        sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
            .bind(tenant_id)
            .execute(&self.pool)
            .await
            .map_err(Error::Database)?;
        
        let activities = sqlx::query_as!(
            UserActivityLog,
            r#"
            SELECT id, user_id, tenant_id, activity_type, activity_description,
                   resource_type, resource_id, metadata, ip_address, user_agent,
                   session_id, created_at
            FROM user_activity_log 
            WHERE user_id = $1 AND tenant_id = $2
            ORDER BY created_at DESC
            LIMIT $3 OFFSET $4
            "#,
            user_id,
            tenant_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await
        .map_err(Error::Database)?;
        
        Ok(activities)
    }
}