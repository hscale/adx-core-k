pub mod user;
pub mod workflow;

pub use user::*;
pub use workflow::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: String,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub tenant_name: String,
    pub subscription_tier: String,
    pub features: Vec<String>,
    pub quotas: HashMap<String, u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub meta: Option<ResponseMeta>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMeta {
    pub total: Option<u64>,
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub cached: Option<bool>,
    pub cache_ttl: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserDashboardData {
    pub user_info: serde_json::Value,
    pub recent_activity: serde_json::Value,
    pub workflow_status: serde_json::Value,
    pub quick_actions: Vec<QuickAction>,
    pub notifications: Vec<Notification>,
    pub stats: UserStats,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuickAction {
    pub id: String,
    pub title: String,
    pub description: String,
    pub action_type: String,
    pub url: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Notification {
    pub id: String,
    pub title: String,
    pub message: String,
    pub notification_type: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub is_read: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserStats {
    pub total_workflows: u32,
    pub completed_workflows: u32,
    pub active_workflows: u32,
    pub files_uploaded: u32,
    pub account_age_days: u32,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(20),
            sort_by: None,
            sort_order: Some("asc".to_string()),
        }
    }
}