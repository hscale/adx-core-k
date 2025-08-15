use axum::{
    extract::{Path, Query, Request, State},
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::{
    middleware::{
        auth::{has_permission, Claims},
        error_handler::{BffError, BffResult},
        tenant::{get_tenant_context, get_tenant_id},
    },
    types::{ApiResponse, ResponseMeta, TenantContext},
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard/:user_id", get(get_user_dashboard))
        .route("/profile-summary/:user_id", get(get_user_profile_summary))
        .route("/activity-summary/:user_id", get(get_user_activity_summary))
        .route("/workflow-summary/:user_id", get(get_user_workflow_summary))
        .route("/tenant-users-overview", get(get_tenant_users_overview))
        .route("/user-management-dashboard", get(get_user_management_dashboard))
}

#[derive(Debug, Deserialize)]
struct DashboardQuery {
    include_activity: Option<bool>,
    include_workflows: Option<bool>,
    include_preferences: Option<bool>,
    activity_days: Option<u32>,
}

#[derive(Debug, Serialize)]
struct UserDashboardData {
    user: serde_json::Value,
    profile: serde_json::Value,
    preferences: serde_json::Value,
    recent_activity: Option<serde_json::Value>,
    active_workflows: Option<Vec<serde_json::Value>>,
    workflow_stats: Option<WorkflowStats>,
    tenant_info: serde_json::Value,
    quick_actions: Vec<QuickAction>,
}

#[derive(Debug, Serialize)]
struct WorkflowStats {
    total_workflows: u32,
    running_workflows: u32,
    completed_workflows: u32,
    failed_workflows: u32,
    success_rate: f32,
}

#[derive(Debug, Serialize)]
struct QuickAction {
    id: String,
    title: String,
    description: String,
    action_type: String,
    url: String,
    icon: String,
    enabled: bool,
}

#[derive(Debug, Serialize)]
struct UserProfileSummary {
    user: serde_json::Value,
    profile: serde_json::Value,
    last_login: Option<String>,
    session_count: u32,
    account_age_days: u32,
    profile_completeness: f32,
    verification_status: VerificationStatus,
}

#[derive(Debug, Serialize)]
struct VerificationStatus {
    email_verified: bool,
    phone_verified: bool,
    identity_verified: bool,
    mfa_enabled: bool,
}

#[derive(Debug, Serialize)]
struct UserActivitySummary {
    total_activities: u32,
    activities_today: u32,
    activities_this_week: u32,
    activities_this_month: u32,
    most_common_activities: Vec<ActivityCount>,
    activity_timeline: Vec<ActivityTimelineItem>,
    peak_activity_hours: Vec<u32>,
}

#[derive(Debug, Serialize)]
struct ActivityCount {
    activity_type: String,
    count: u32,
    percentage: f32,
}

#[derive(Debug, Serialize)]
struct ActivityTimelineItem {
    date: String,
    count: u32,
}

#[derive(Debug, Serialize)]
struct UserWorkflowSummary {
    total_workflows: u32,
    running_workflows: Vec<serde_json::Value>,
    recent_completed: Vec<serde_json::Value>,
    recent_failed: Vec<serde_json::Value>,
    workflow_types: Vec<WorkflowTypeCount>,
    avg_execution_time: Option<f64>,
    success_rate: f32,
}

#[derive(Debug, Serialize)]
struct WorkflowTypeCount {
    workflow_type: String,
    count: u32,
    success_rate: f32,
    avg_duration: Option<f64>,
}

#[derive(Debug, Serialize)]
struct TenantUsersOverview {
    total_users: u32,
    active_users: u32,
    new_users_today: u32,
    new_users_this_week: u32,
    new_users_this_month: u32,
    user_roles: Vec<RoleCount>,
    user_activity_trend: Vec<ActivityTimelineItem>,
    top_active_users: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct RoleCount {
    role: String,
    count: u32,
    percentage: f32,
}

#[derive(Debug, Serialize)]
struct UserManagementDashboard {
    tenant_overview: TenantUsersOverview,
    recent_registrations: Vec<serde_json::Value>,
    pending_invitations: Vec<serde_json::Value>,
    user_sessions: SessionStats,
    security_alerts: Vec<SecurityAlert>,
    system_health: SystemHealthStatus,
}

#[derive(Debug, Serialize)]
struct SessionStats {
    total_active_sessions: u32,
    unique_users_online: u32,
    avg_session_duration: f64,
    sessions_by_device: HashMap<String, u32>,
}

#[derive(Debug, Serialize)]
struct SecurityAlert {
    id: String,
    alert_type: String,
    severity: String,
    message: String,
    user_id: Option<String>,
    timestamp: String,
    resolved: bool,
}

#[derive(Debug, Serialize)]
struct SystemHealthStatus {
    overall_status: String,
    user_service_status: String,
    auth_service_status: String,
    workflow_service_status: String,
    cache_hit_rate: f32,
    avg_response_time: f64,
}

// Get comprehensive user dashboard data
async fn get_user_dashboard(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Query(query): Query<DashboardQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions (users can access their own dashboard, or need user:read permission)
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to access user dashboard"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Try to get cached dashboard data first
    if let Ok(Some(cached_dashboard)) = state.redis.get_cached_user_dashboard(&user_id).await {
        debug!("Returning cached user dashboard: {}", user_id);
        return Ok(Json(ApiResponse {
            data: cached_dashboard,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Fetch data from multiple sources in parallel
    let user_future = state.api_client.get_user(&user_id, tenant_id, &auth_token);
    let profile_future = state.api_client.get_user_profile(&user_id, tenant_id, &auth_token);
    let preferences_future = state.api_client.get_user_preferences(&user_id, tenant_id, &auth_token);

    let (user_result, profile_result, preferences_result) = tokio::try_join!(
        user_future,
        profile_future,
        preferences_future
    )?;

    // Optionally fetch activity data
    let recent_activity = if query.include_activity.unwrap_or(true) {
        let activity_days = query.activity_days.unwrap_or(7);
        let mut params = HashMap::new();
        params.insert("limit".to_string(), "10".to_string());
        params.insert("days".to_string(), activity_days.to_string());
        
        match state.api_client.get_user_activity(&user_id, tenant_id, &auth_token, Some(params)).await {
            Ok(activity) => Some(activity),
            Err(e) => {
                error!("Failed to fetch user activity: {}", e);
                None
            }
        }
    } else {
        None
    };

    // Optionally fetch workflow data
    let (active_workflows, workflow_stats) = if query.include_workflows.unwrap_or(true) {
        match state.temporal_client.list_workflows(None, None, Some(10), None).await {
            Ok(workflows) => {
                let user_workflows: Vec<_> = workflows.into_iter()
                    .filter(|w| {
                        w.input.get("user_id")
                            .and_then(|v| v.as_str())
                            .map(|id| id == user_id)
                            .unwrap_or(false)
                    })
                    .collect();

                let running_count = user_workflows.iter()
                    .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Running))
                    .count() as u32;
                
                let completed_count = user_workflows.iter()
                    .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Completed))
                    .count() as u32;
                
                let failed_count = user_workflows.iter()
                    .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Failed))
                    .count() as u32;

                let total_count = user_workflows.len() as u32;
                let success_rate = if total_count > 0 {
                    (completed_count as f32 / total_count as f32) * 100.0
                } else {
                    0.0
                };

                let active_workflows: Vec<serde_json::Value> = user_workflows.into_iter()
                    .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Running))
                    .map(|w| serde_json::to_value(w).unwrap_or_default())
                    .collect();

                let stats = WorkflowStats {
                    total_workflows: total_count,
                    running_workflows: running_count,
                    completed_workflows: completed_count,
                    failed_workflows: failed_count,
                    success_rate,
                };

                (Some(active_workflows), Some(stats))
            }
            Err(e) => {
                error!("Failed to fetch user workflows: {}", e);
                (None, None)
            }
        }
    } else {
        (None, None)
    };

    // Generate quick actions based on user permissions and context
    let quick_actions = generate_quick_actions(claims, tenant_context);

    // Build dashboard data
    let dashboard_data = UserDashboardData {
        user: user_result,
        profile: profile_result,
        preferences: preferences_result,
        recent_activity,
        active_workflows,
        workflow_stats,
        tenant_info: serde_json::to_value(tenant_context)?,
        quick_actions,
    };

    // Cache the dashboard data
    let dashboard_json = serde_json::to_value(&dashboard_data)?;
    if let Err(e) = state.redis.cache_user_dashboard(&user_id, &dashboard_json, Some(300)).await {
        error!("Failed to cache user dashboard: {}", e);
    }

    info!("Generated user dashboard for user: {} in tenant: {}", user_id, tenant_id);

    Ok(Json(ApiResponse {
        data: serde_json::to_value(dashboard_data)?,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get user profile summary
async fn get_user_profile_summary(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<UserProfileSummary>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to access user profile summary"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Fetch user and profile data
    let (user_result, profile_result, sessions_result) = tokio::try_join!(
        state.api_client.get_user(&user_id, tenant_id, &auth_token),
        state.api_client.get_user_profile(&user_id, tenant_id, &auth_token),
        state.api_client.get_user_sessions(&user_id, tenant_id, &auth_token)
    )?;

    // Calculate profile completeness and other metrics
    let profile_completeness = calculate_profile_completeness(&profile_result);
    let account_age_days = calculate_account_age(&user_result);
    let session_count = sessions_result.as_array().map(|s| s.len() as u32).unwrap_or(0);

    let summary = UserProfileSummary {
        user: user_result,
        profile: profile_result,
        last_login: None, // Would be extracted from user data
        session_count,
        account_age_days,
        profile_completeness,
        verification_status: VerificationStatus {
            email_verified: true, // Mock data
            phone_verified: false,
            identity_verified: false,
            mfa_enabled: true,
        },
    };

    Ok(Json(ApiResponse {
        data: summary,
        meta: None,
    }))
}

// Get user activity summary
async fn get_user_activity_summary(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<UserActivitySummary>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "user:read") {
        return Err(BffError::authorization("Insufficient permissions to access user activity summary"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Fetch activity data with different time ranges
    let mut params = HashMap::new();
    params.insert("limit".to_string(), "100".to_string());
    
    let activity_result = state.api_client.get_user_activity(&user_id, tenant_id, &auth_token, Some(params)).await?;

    // Process activity data to generate summary
    let summary = UserActivitySummary {
        total_activities: 156, // Mock data - would be calculated from activity_result
        activities_today: 12,
        activities_this_week: 45,
        activities_this_month: 156,
        most_common_activities: vec![
            ActivityCount {
                activity_type: "login".to_string(),
                count: 45,
                percentage: 28.8,
            },
            ActivityCount {
                activity_type: "profile_update".to_string(),
                count: 23,
                percentage: 14.7,
            },
            ActivityCount {
                activity_type: "file_upload".to_string(),
                count: 18,
                percentage: 11.5,
            },
        ],
        activity_timeline: vec![
            ActivityTimelineItem {
                date: "2024-01-15".to_string(),
                count: 12,
            },
            ActivityTimelineItem {
                date: "2024-01-14".to_string(),
                count: 8,
            },
        ],
        peak_activity_hours: vec![9, 10, 14, 15, 16], // Hours of day with most activity
    };

    Ok(Json(ApiResponse {
        data: summary,
        meta: None,
    }))
}

// Get user workflow summary
async fn get_user_workflow_summary(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<UserWorkflowSummary>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if claims.sub != user_id && !has_permission(claims, "workflow:read") {
        return Err(BffError::authorization("Insufficient permissions to access user workflow summary"));
    }

    // Fetch user workflows
    let workflows = state.temporal_client.list_workflows(None, None, Some(50), None).await?;
    
    let user_workflows: Vec<_> = workflows.into_iter()
        .filter(|w| {
            w.input.get("user_id")
                .and_then(|v| v.as_str())
                .map(|id| id == user_id)
                .unwrap_or(false)
        })
        .collect();

    // Process workflow data
    let running_workflows: Vec<serde_json::Value> = user_workflows.iter()
        .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Running))
        .map(|w| serde_json::to_value(w).unwrap_or_default())
        .collect();

    let recent_completed: Vec<serde_json::Value> = user_workflows.iter()
        .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Completed))
        .take(5)
        .map(|w| serde_json::to_value(w).unwrap_or_default())
        .collect();

    let recent_failed: Vec<serde_json::Value> = user_workflows.iter()
        .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Failed))
        .take(5)
        .map(|w| serde_json::to_value(w).unwrap_or_default())
        .collect();

    // Calculate workflow type statistics
    let mut workflow_type_counts: HashMap<String, (u32, u32, Vec<u64>)> = HashMap::new();
    for workflow in &user_workflows {
        let entry = workflow_type_counts.entry(workflow.workflow_type.clone()).or_insert((0, 0, Vec::new()));
        entry.0 += 1; // total count
        if matches!(workflow.status, crate::types::WorkflowStatus::Completed) {
            entry.1 += 1; // success count
        }
        if let Some(duration) = workflow.execution_time_ms {
            entry.2.push(duration);
        }
    }

    let workflow_types: Vec<WorkflowTypeCount> = workflow_type_counts.into_iter()
        .map(|(workflow_type, (total, success, durations))| {
            let success_rate = if total > 0 { (success as f32 / total as f32) * 100.0 } else { 0.0 };
            let avg_duration = if !durations.is_empty() {
                Some(durations.iter().sum::<u64>() as f64 / durations.len() as f64)
            } else {
                None
            };
            
            WorkflowTypeCount {
                workflow_type,
                count: total,
                success_rate,
                avg_duration,
            }
        })
        .collect();

    let total_workflows = user_workflows.len() as u32;
    let completed_count = user_workflows.iter()
        .filter(|w| matches!(w.status, crate::types::WorkflowStatus::Completed))
        .count() as u32;
    
    let success_rate = if total_workflows > 0 {
        (completed_count as f32 / total_workflows as f32) * 100.0
    } else {
        0.0
    };

    let avg_execution_time = {
        let durations: Vec<u64> = user_workflows.iter()
            .filter_map(|w| w.execution_time_ms)
            .collect();
        
        if !durations.is_empty() {
            Some(durations.iter().sum::<u64>() as f64 / durations.len() as f64)
        } else {
            None
        }
    };

    let summary = UserWorkflowSummary {
        total_workflows,
        running_workflows,
        recent_completed,
        recent_failed,
        workflow_types,
        avg_execution_time,
        success_rate,
    };

    Ok(Json(ApiResponse {
        data: summary,
        meta: None,
    }))
}

// Get tenant users overview (admin only)
async fn get_tenant_users_overview(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<TenantUsersOverview>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "user:admin") {
        return Err(BffError::authorization("Insufficient permissions to access tenant users overview"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let auth_token = extract_auth_token(&request)?;

    // Fetch users list
    let users_result = state.api_client.list_users(tenant_id, &auth_token, None).await?;

    // Process users data to generate overview
    let overview = TenantUsersOverview {
        total_users: 1250, // Mock data - would be calculated from users_result
        active_users: 1100,
        new_users_today: 15,
        new_users_this_week: 87,
        new_users_this_month: 342,
        user_roles: vec![
            RoleCount {
                role: "user".to_string(),
                count: 1000,
                percentage: 80.0,
            },
            RoleCount {
                role: "admin".to_string(),
                count: 200,
                percentage: 16.0,
            },
            RoleCount {
                role: "viewer".to_string(),
                count: 50,
                percentage: 4.0,
            },
        ],
        user_activity_trend: vec![
            ActivityTimelineItem {
                date: "2024-01-15".to_string(),
                count: 1100,
            },
            ActivityTimelineItem {
                date: "2024-01-14".to_string(),
                count: 1050,
            },
        ],
        top_active_users: vec![], // Would be populated from actual data
    };

    Ok(Json(ApiResponse {
        data: overview,
        meta: None,
    }))
}

// Get user management dashboard (admin only)
async fn get_user_management_dashboard(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<UserManagementDashboard>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "user:admin") {
        return Err(BffError::authorization("Insufficient permissions to access user management dashboard"));
    }

    // This would aggregate data from multiple sources
    let dashboard = UserManagementDashboard {
        tenant_overview: TenantUsersOverview {
            total_users: 1250,
            active_users: 1100,
            new_users_today: 15,
            new_users_this_week: 87,
            new_users_this_month: 342,
            user_roles: vec![],
            user_activity_trend: vec![],
            top_active_users: vec![],
        },
        recent_registrations: vec![],
        pending_invitations: vec![],
        user_sessions: SessionStats {
            total_active_sessions: 456,
            unique_users_online: 234,
            avg_session_duration: 3600.0, // 1 hour
            sessions_by_device: {
                let mut devices = HashMap::new();
                devices.insert("web".to_string(), 300);
                devices.insert("mobile".to_string(), 120);
                devices.insert("desktop".to_string(), 36);
                devices
            },
        },
        security_alerts: vec![],
        system_health: SystemHealthStatus {
            overall_status: "healthy".to_string(),
            user_service_status: "healthy".to_string(),
            auth_service_status: "healthy".to_string(),
            workflow_service_status: "healthy".to_string(),
            cache_hit_rate: 85.5,
            avg_response_time: 120.0,
        },
    };

    Ok(Json(ApiResponse {
        data: dashboard,
        meta: None,
    }))
}

// Helper functions
fn extract_auth_token(request: &Request) -> BffResult<String> {
    let auth_header = request.headers()
        .get("authorization")
        .ok_or_else(|| BffError::authentication("Missing authorization header"))?;
    
    let auth_str = auth_header.to_str()
        .map_err(|_| BffError::authentication("Invalid authorization header"))?;
    
    if auth_str.starts_with("Bearer ") {
        Ok(auth_str[7..].to_string())
    } else {
        Err(BffError::authentication("Invalid authorization header format"))
    }
}

fn generate_quick_actions(claims: &Claims, tenant_context: &TenantContext) -> Vec<QuickAction> {
    let mut actions = vec![
        QuickAction {
            id: "update_profile".to_string(),
            title: "Update Profile".to_string(),
            description: "Update your profile information".to_string(),
            action_type: "navigation".to_string(),
            url: "/profile/edit".to_string(),
            icon: "user-edit".to_string(),
            enabled: true,
        },
        QuickAction {
            id: "change_password".to_string(),
            title: "Change Password".to_string(),
            description: "Update your account password".to_string(),
            action_type: "navigation".to_string(),
            url: "/security/password".to_string(),
            icon: "lock".to_string(),
            enabled: true,
        },
    ];

    // Add admin-specific actions
    if has_permission(claims, "user:admin") {
        actions.push(QuickAction {
            id: "manage_users".to_string(),
            title: "Manage Users".to_string(),
            description: "Manage tenant users".to_string(),
            action_type: "navigation".to_string(),
            url: "/admin/users".to_string(),
            icon: "users".to_string(),
            enabled: true,
        });
    }

    // Add workflow actions if user has permissions
    if has_permission(claims, "workflow:execute") {
        actions.push(QuickAction {
            id: "export_data".to_string(),
            title: "Export My Data".to_string(),
            description: "Export your account data".to_string(),
            action_type: "workflow".to_string(),
            url: "/workflows/user-data-export".to_string(),
            icon: "download".to_string(),
            enabled: true,
        });
    }

    actions
}

fn calculate_profile_completeness(profile: &serde_json::Value) -> f32 {
    // Mock calculation - would check various profile fields
    let mut completeness = 0.0;
    let total_fields = 10.0;
    
    if profile.get("display_name").and_then(|v| v.as_str()).is_some() {
        completeness += 1.0;
    }
    if profile.get("bio").and_then(|v| v.as_str()).is_some() {
        completeness += 1.0;
    }
    if profile.get("location").and_then(|v| v.as_str()).is_some() {
        completeness += 1.0;
    }
    // ... check other fields
    
    (completeness / total_fields) * 100.0
}

fn calculate_account_age(user: &serde_json::Value) -> u32 {
    // Mock calculation - would parse created_at date
    30 // days
}