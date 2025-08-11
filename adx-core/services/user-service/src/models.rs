use chrono::{DateTime, Utc, NaiveDate, NaiveTime};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;

// Core user model (from base users table)
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    #[serde(skip_serializing)]
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "snake_case")]
pub enum UserStatus {
    Active,
    Inactive,
    Suspended,
    PendingVerification,
}

// Extended user profile model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub timezone: String,
    pub language: String,
    pub date_format: String,
    pub time_format: String,
    pub phone_number: Option<String>,
    pub phone_verified_at: Option<DateTime<Utc>>,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub manager_id: Option<Uuid>,
    pub hire_date: Option<NaiveDate>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User preferences model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserPreference {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub preference_category: String,
    pub preference_key: String,
    pub preference_value: serde_json::Value,
    pub is_inherited: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User notification settings model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserNotificationSetting {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub notification_type: String,
    pub event_category: String,
    pub event_name: String,
    pub is_enabled: bool,
    pub delivery_schedule: String,
    pub quiet_hours_start: Option<NaiveTime>,
    pub quiet_hours_end: Option<NaiveTime>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User activity log model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserActivityLog {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub activity_type: String,
    pub activity_description: Option<String>,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub metadata: serde_json::Value,
    pub ip_address: Option<std::net::IpAddr>,
    pub user_agent: Option<String>,
    pub session_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

// User team model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserTeam {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub team_name: String,
    pub team_description: Option<String>,
    pub team_lead_id: Option<Uuid>,
    pub is_active: bool,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User team membership model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserTeamMembership {
    pub id: Uuid,
    pub team_id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub role: String,
    pub joined_at: DateTime<Utc>,
    pub left_at: Option<DateTime<Utc>>,
    pub is_active: bool,
}

// User skill model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSkill {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub skill_name: String,
    pub skill_category: Option<String>,
    pub proficiency_level: Option<i32>,
    pub years_experience: Option<i32>,
    pub is_certified: bool,
    pub certification_name: Option<String>,
    pub certification_date: Option<NaiveDate>,
    pub verified_by: Option<Uuid>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User connection model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserConnection {
    pub id: Uuid,
    pub requester_id: Uuid,
    pub requestee_id: Uuid,
    pub tenant_id: Uuid,
    pub connection_type: String,
    pub status: ConnectionStatus,
    pub message: Option<String>,
    pub requested_at: DateTime<Utc>,
    pub responded_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "varchar", rename_all = "snake_case")]
pub enum ConnectionStatus {
    Pending,
    Accepted,
    Declined,
    Blocked,
}

// User saved search model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserSavedSearch {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub search_name: String,
    pub search_type: String,
    pub search_query: serde_json::Value,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// User bookmark model
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct UserBookmark {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub bookmark_name: String,
    pub bookmark_type: String,
    pub resource_id: Option<Uuid>,
    pub external_url: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Combined user data for API responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserWithProfile {
    #[serde(flatten)]
    pub user: User,
    pub profile: Option<UserProfile>,
}

// Request/Response DTOs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Option<Vec<String>>,
    pub profile: Option<CreateUserProfileRequest>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub phone_number: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub manager_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub status: Option<UserStatus>,
    pub roles: Option<Vec<String>>,
    pub permissions: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub avatar_url: Option<String>,
    pub cover_image_url: Option<String>,
    pub location: Option<String>,
    pub website_url: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub phone_number: Option<String>,
    pub birth_date: Option<NaiveDate>,
    pub gender: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub manager_id: Option<Uuid>,
    pub hire_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferenceRequest {
    pub preference_category: String,
    pub preferences: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchRequest {
    pub query: Option<String>,
    pub department: Option<String>,
    pub role: Option<String>,
    pub skills: Option<Vec<String>>,
    pub team_id: Option<Uuid>,
    pub status: Option<UserStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSearchResponse {
    pub users: Vec<UserWithProfile>,
    pub total_count: i64,
    pub has_more: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDirectoryEntry {
    pub id: Uuid,
    pub display_name: String,
    pub email: String,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub avatar_url: Option<String>,
    pub status: UserStatus,
    pub last_login_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserDirectoryResponse {
    pub entries: Vec<UserDirectoryEntry>,
    pub total_count: i64,
    pub departments: Vec<String>,
    pub roles: Vec<String>,
}