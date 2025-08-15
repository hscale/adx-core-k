use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: bool,
    pub last_login: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub tenant_id: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
    pub preferences: UserPreferences,
    pub profile: UserProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub theme: String, // "light", "dark", "auto"
    pub notifications: NotificationPreferences,
    pub dashboard: DashboardPreferences,
    pub privacy: PrivacyPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationPreferences {
    pub email_notifications: bool,
    pub push_notifications: bool,
    pub workflow_updates: bool,
    pub security_alerts: bool,
    pub marketing_emails: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardPreferences {
    pub default_view: String,
    pub widgets: Vec<String>,
    pub refresh_interval: u32, // seconds
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreferences {
    pub profile_visibility: String, // "public", "team", "private"
    pub activity_visibility: String,
    pub allow_analytics: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub roles: Vec<String>,
    pub send_invitation: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub avatar_url: Option<String>,
    pub is_active: Option<bool>,
    pub roles: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserProfileRequest {
    pub display_name: Option<String>,
    pub bio: Option<String>,
    pub location: Option<String>,
    pub timezone: Option<String>,
    pub language: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUserPreferencesRequest {
    pub theme: Option<String>,
    pub notifications: Option<NotificationPreferences>,
    pub dashboard: Option<DashboardPreferences>,
    pub privacy: Option<PrivacyPreferences>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserListResponse {
    pub users: Vec<User>,
    pub total: u64,
    pub page: u32,
    pub per_page: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserActivity {
    pub id: String,
    pub user_id: String,
    pub activity_type: String,
    pub description: String,
    pub metadata: HashMap<String, serde_json::Value>,
    pub timestamp: DateTime<Utc>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserSession {
    pub id: String,
    pub user_id: String,
    pub tenant_id: String,
    pub device_id: Option<String>,
    pub ip_address: String,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub total_users: u64,
    pub active_users: u64,
    pub new_users_today: u64,
    pub new_users_this_week: u64,
    pub new_users_this_month: u64,
    pub user_activity_summary: HashMap<String, u64>,
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            theme: "auto".to_string(),
            notifications: NotificationPreferences::default(),
            dashboard: DashboardPreferences::default(),
            privacy: PrivacyPreferences::default(),
        }
    }
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            email_notifications: true,
            push_notifications: true,
            workflow_updates: true,
            security_alerts: true,
            marketing_emails: false,
        }
    }
}

impl Default for DashboardPreferences {
    fn default() -> Self {
        Self {
            default_view: "overview".to_string(),
            widgets: vec![
                "recent_activity".to_string(),
                "workflow_status".to_string(),
                "quick_actions".to_string(),
            ],
            refresh_interval: 30,
        }
    }
}

impl Default for PrivacyPreferences {
    fn default() -> Self {
        Self {
            profile_visibility: "team".to_string(),
            activity_visibility: "team".to_string(),
            allow_analytics: true,
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            display_name: None,
            bio: None,
            location: None,
            timezone: None,
            language: Some("en".to_string()),
            phone: None,
            company: None,
            job_title: None,
        }
    }
}