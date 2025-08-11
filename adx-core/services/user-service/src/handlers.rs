use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    Extension,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use adx_shared::{TenantContext, UserContext, Result};
use crate::{
    models::*,
    repositories::*,
    workflows::*,
    validation::{UserValidator, validate_create_user_request, validate_update_user_request, validate_update_user_profile_request},
};

// Handler state
#[derive(Clone)]
pub struct UserServiceState {
    pub user_repo: Arc<dyn UserRepository>,
    pub profile_repo: Arc<dyn UserProfileRepository>,
    pub preference_repo: Arc<dyn UserPreferenceRepository>,
    pub activity_repo: Arc<dyn UserActivityRepository>,
    pub validator: Arc<UserValidator>,
}

// Query parameters for listing users
#[derive(Debug, Deserialize)]
pub struct ListUsersQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Query parameters for user search
#[derive(Debug, Deserialize)]
pub struct SearchUsersQuery {
    pub q: Option<String>,
    pub department: Option<String>,
    pub role: Option<String>,
    pub skills: Option<String>, // Comma-separated
    pub team_id: Option<Uuid>,
    pub status: Option<UserStatus>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Query parameters for user directory
#[derive(Debug, Deserialize)]
pub struct DirectoryQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

// Response wrapper for API responses
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }
    
    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

// Helper function to parse tenant ID
fn parse_tenant_id(tenant_context: &TenantContext) -> Result<Uuid> {
    Uuid::parse_str(&tenant_context.tenant_id)
        .map_err(|_| adx_shared::Error::Validation("Invalid tenant ID format".to_string()))
}

// Helper function to parse user ID
fn parse_user_id(user_context: &UserContext) -> Result<Uuid> {
    Uuid::parse_str(&user_context.user_id)
        .map_err(|_| adx_shared::Error::Validation("Invalid user ID format".to_string()))
}

// Helper function to create a mock workflow context for simulation
fn create_mock_workflow_context(tenant_id: &str, workflow_type: &str) -> adx_shared::temporal::WorkflowContext {
    adx_shared::temporal::WorkflowContext {
        workflow_id: format!("{}-{}", workflow_type, uuid::Uuid::new_v4()),
        run_id: uuid::Uuid::new_v4().to_string(),
        workflow_type: workflow_type.to_string(),
        version: adx_shared::temporal::WorkflowVersion::new(1, 0, 0),
        task_queue: "user-service".to_string(),
        namespace: "default".to_string(),
        user_context: adx_shared::temporal::workflow::UserContext {
            user_id: "system".to_string(),
            email: "system@adxcore.com".to_string(),
            roles: vec!["system".to_string()],
            permissions: vec!["workflow:execute".to_string()],
            session_id: None,
            device_info: None,
        },
        tenant_context: adx_shared::temporal::workflow::TenantContext {
            tenant_id: tenant_id.to_string(),
            tenant_name: "Default".to_string(),
            subscription_tier: adx_shared::temporal::workflow::SubscriptionTier::Professional,
            features: vec![],
            quotas: adx_shared::temporal::workflow::TenantQuotas {
                max_users: 1000,
                max_storage_gb: 100,
                max_api_calls_per_hour: 10000,
                max_concurrent_workflows: 50,
                max_file_upload_size_mb: 100,
            },
            settings: adx_shared::temporal::workflow::TenantSettings {
                default_language: "en".to_string(),
                timezone: "UTC".to_string(),
                date_format: "YYYY-MM-DD".to_string(),
                currency: "USD".to_string(),
                branding: None,
            },
            isolation_level: adx_shared::temporal::workflow::TenantIsolationLevel::Schema,
        },
        metadata: adx_shared::temporal::workflow::WorkflowMetadata {
            start_time: chrono::Utc::now(),
            timeout: std::time::Duration::from_secs(3600),
            retry_policy: None,
            parent_workflow_id: None,
            correlation_id: None,
            business_process: Some("user_management".to_string()),
            priority: adx_shared::temporal::workflow::WorkflowPriority::Normal,
            tags: vec!["user".to_string()],
        },
        search_attributes: std::collections::HashMap::new(),
    }
}

// User CRUD handlers
pub async fn create_user(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Extension(user_context): Extension<UserContext>,
    Json(request): Json<CreateUserRequest>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let creator_uuid = parse_user_id(&user_context)?;
    
    // Validate request
    if let Err(e) = validate_create_user_request(&state.validator, &request) {
        return Ok(Json(ApiResponse::error(e.to_string())));
    }
    
    // Check if user already exists
    if let Ok(Some(_)) = state.user_repo.find_by_email(tenant_uuid, &request.email).await {
        return Ok(Json(ApiResponse::error("User with this email already exists".to_string())));
    }
    
    // Create user
    match state.user_repo.create(tenant_uuid, request).await {
        Ok(user) => {
            // Log activity
            let activity = UserActivityLog {
                id: Uuid::new_v4(),
                user_id: user.id,
                tenant_id: tenant_uuid,
                activity_type: "user_created".to_string(),
                activity_description: Some("User account created".to_string()),
                resource_type: Some("user".to_string()),
                resource_id: Some(user.id),
                metadata: serde_json::json!({"created_by": creator_uuid}),
                ip_address: None,
                user_agent: None,
                session_id: None,
                created_at: chrono::Utc::now(),
            };
            
            let _ = state.activity_repo.log_activity(activity).await;
            
            Ok(Json(ApiResponse::success(user)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn get_user(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserWithProfile>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
        
    match state.user_repo.find_by_id(tenant_uuid, user_id).await {
        Ok(Some(user)) => {
            // Get user profile
            let profile = state.profile_repo.find_by_user_id(tenant_uuid, user_id).await.ok().flatten();
            
            Ok(Json(ApiResponse::success(UserWithProfile { user, profile })))
        }
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn update_user(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Extension(user_context): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<User>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let updater_uuid = parse_user_id(&user_context)?;
    
    // Validate request
    if let Err(e) = validate_update_user_request(&state.validator, &request) {
        return Ok(Json(ApiResponse::error(e.to_string())));
    }
    
    // Check if user exists
    if state.user_repo.find_by_id(tenant_uuid, user_id).await?.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Update user
    match state.user_repo.update(tenant_uuid, user_id, request).await {
        Ok(user) => {
            // Log activity
            let activity = UserActivityLog {
                id: Uuid::new_v4(),
                user_id,
                tenant_id: tenant_uuid,
                activity_type: "user_updated".to_string(),
                activity_description: Some("User account updated".to_string()),
                resource_type: Some("user".to_string()),
                resource_id: Some(user_id),
                metadata: serde_json::json!({"updated_by": updater_uuid}),
                ip_address: None,
                user_agent: None,
                session_id: None,
                created_at: chrono::Utc::now(),
            };
            
            let _ = state.activity_repo.log_activity(activity).await;
            
            Ok(Json(ApiResponse::success(user)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn delete_user(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Extension(user_context): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let deleter_uuid = parse_user_id(&user_context)?;
    
    // Check if user exists
    if state.user_repo.find_by_id(tenant_uuid, user_id).await?.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Delete user
    match state.user_repo.delete(tenant_uuid, user_id).await {
        Ok(_) => {
            // Log activity
            let activity = UserActivityLog {
                id: Uuid::new_v4(),
                user_id,
                tenant_id: tenant_uuid,
                activity_type: "user_deleted".to_string(),
                activity_description: Some("User account deleted".to_string()),
                resource_type: Some("user".to_string()),
                resource_id: Some(user_id),
                metadata: serde_json::json!({"deleted_by": deleter_uuid}),
                ip_address: None,
                user_agent: None,
                session_id: None,
                created_at: chrono::Utc::now(),
            };
            
            let _ = state.activity_repo.log_activity(activity).await;
            
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            tracing::error!("Failed to delete user: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn list_users(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);
    
    match state.user_repo.list(tenant_uuid, limit, offset).await {
        Ok(users) => Ok(Json(ApiResponse::success(users))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// User profile handlers
pub async fn get_user_profile(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<UserProfile>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    
    match state.profile_repo.find_by_user_id(tenant_uuid, user_id).await {
        Ok(Some(profile)) => Ok(Json(ApiResponse::success(profile))),
        Ok(None) => Err(StatusCode::NOT_FOUND),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn create_user_profile(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Extension(user_context): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<CreateUserProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let creator_uuid = parse_user_id(&user_context)?;
    
    // Validate request
    if let Err(e) = crate::validation::validate_create_user_profile_request(&state.validator, &request) {
        return Ok(Json(ApiResponse::error(e.to_string())));
    }
    
    // Check if user exists
    if state.user_repo.find_by_id(tenant_uuid, user_id).await?.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Check if profile already exists
    if state.profile_repo.find_by_user_id(tenant_uuid, user_id).await?.is_some() {
        return Ok(Json(ApiResponse::error("User profile already exists".to_string())));
    }
    
    // Create profile
    match state.profile_repo.create(tenant_uuid, user_id, request).await {
        Ok(profile) => {
            // Log activity
            let activity = UserActivityLog {
                id: Uuid::new_v4(),
                user_id,
                tenant_id: tenant_uuid,
                activity_type: "profile_created".to_string(),
                activity_description: Some("User profile created".to_string()),
                resource_type: Some("user_profile".to_string()),
                resource_id: Some(profile.id),
                metadata: serde_json::json!({"created_by": creator_uuid}),
                ip_address: None,
                user_agent: None,
                session_id: None,
                created_at: chrono::Utc::now(),
            };
            
            let _ = state.activity_repo.log_activity(activity).await;
            
            Ok(Json(ApiResponse::success(profile)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn update_user_profile(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Extension(user_context): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UpdateUserProfileRequest>,
) -> Result<Json<ApiResponse<UserProfile>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let updater_uuid = parse_user_id(&user_context)?;
    
    // Validate request
    if let Err(e) = validate_update_user_profile_request(&state.validator, &request) {
        return Ok(Json(ApiResponse::error(e.to_string())));
    }
    
    // Check if profile exists
    if state.profile_repo.find_by_user_id(tenant_uuid, user_id).await?.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    // Update profile
    match state.profile_repo.update(tenant_uuid, user_id, request).await {
        Ok(profile) => {
            // Log activity
            let activity = UserActivityLog {
                id: Uuid::new_v4(),
                user_id,
                tenant_id: tenant_uuid,
                activity_type: "profile_updated".to_string(),
                activity_description: Some("User profile updated".to_string()),
                resource_type: Some("user_profile".to_string()),
                resource_id: Some(profile.id),
                metadata: serde_json::json!({"updated_by": updater_uuid}),
                ip_address: None,
                user_agent: None,
                session_id: None,
                created_at: chrono::Utc::now(),
            };
            
            let _ = state.activity_repo.log_activity(activity).await;
            
            Ok(Json(ApiResponse::success(profile)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// User preferences handlers
pub async fn get_user_preferences(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(user_id): Path<Uuid>,
    Query(category): Query<Option<String>>,
) -> Result<Json<ApiResponse<Vec<UserPreference>>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    
    match state.preference_repo.get_preferences(tenant_uuid, user_id, category.as_deref()).await {
        Ok(preferences) => Ok(Json(ApiResponse::success(preferences))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn set_user_preferences(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Extension(user_context): Extension<UserContext>,
    Path(user_id): Path<Uuid>,
    Json(request): Json<UserPreferenceRequest>,
) -> Result<Json<ApiResponse<Vec<UserPreference>>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let updater_uuid = parse_user_id(&user_context)?;
    
    // Check if user exists
    if state.user_repo.find_by_id(tenant_uuid, user_id).await?.is_none() {
        return Err(StatusCode::NOT_FOUND);
    }
    
    match state.preference_repo.set_preferences(tenant_uuid, user_id, request).await {
        Ok(preferences) => {
            // Log activity
            let activity = UserActivityLog {
                id: Uuid::new_v4(),
                user_id,
                tenant_id: tenant_uuid,
                activity_type: "preferences_updated".to_string(),
                activity_description: Some("User preferences updated".to_string()),
                resource_type: Some("user_preferences".to_string()),
                resource_id: Some(user_id),
                metadata: serde_json::json!({"updated_by": updater_uuid}),
                ip_address: None,
                user_agent: None,
                session_id: None,
                created_at: chrono::Utc::now(),
            };
            
            let _ = state.activity_repo.log_activity(activity).await;
            
            Ok(Json(ApiResponse::success(preferences)))
        }
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// User search and directory handlers
pub async fn search_users(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(query): Query<SearchUsersQuery>,
) -> Result<Json<ApiResponse<UserSearchResponse>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let skills = query.skills.map(|s| s.split(',').map(|skill| skill.trim().to_string()).collect());
    
    let search_request = UserSearchRequest {
        query: query.q,
        department: query.department,
        role: query.role,
        skills,
        team_id: query.team_id,
        status: query.status,
        limit: query.limit,
        offset: query.offset,
    };
    
    match state.user_repo.search(tenant_uuid, search_request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn get_user_directory(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Query(query): Query<DirectoryQuery>,
) -> Result<Json<ApiResponse<UserDirectoryResponse>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);
    
    match state.user_repo.get_directory(tenant_uuid, limit, offset).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// User activity handlers
pub async fn get_user_activity(
    State(state): State<UserServiceState>,
    Extension(tenant_context): Extension<TenantContext>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<Vec<UserActivityLog>>>, StatusCode> {
    let tenant_uuid = parse_tenant_id(&tenant_context)?;
    let limit = query.limit.unwrap_or(50).min(100);
    let offset = query.offset.unwrap_or(0);
    
    match state.activity_repo.get_user_activity(tenant_uuid, user_id, limit, offset).await {
        Ok(activities) => Ok(Json(ApiResponse::success(activities))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

// Health check handler
pub async fn health_check() -> Json<ApiResponse<&'static str>> {
    Json(ApiResponse::success("User Service is healthy"))
}

// Workflow endpoint handlers
pub async fn start_user_profile_sync_workflow(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<UserProfileSyncWorkflowRequest>,
) -> Result<Json<ApiResponse<UserProfileSyncWorkflowResponse>>, StatusCode> {
    tracing::info!("Starting user profile sync workflow for user {}", request.user_id);
    
    // In a real implementation, this would start a Temporal workflow
    // For now, we'll simulate the workflow execution
    let workflow_context = create_mock_workflow_context(&tenant_context.tenant_id, "user_profile_sync_workflow");
    
    match user_profile_sync_workflow(workflow_context, request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn start_user_preference_migration_workflow(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<UserPreferenceMigrationWorkflowRequest>,
) -> Result<Json<ApiResponse<UserPreferenceMigrationWorkflowResponse>>, StatusCode> {
    tracing::info!("Starting user preference migration workflow for user {}", request.user_id);
    
    let workflow_context = create_mock_workflow_context(&tenant_context.tenant_id, "user_preference_migration_workflow");
    
    match user_preference_migration_workflow(workflow_context, request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn start_user_data_export_workflow(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<UserDataExportWorkflowRequest>,
) -> Result<Json<ApiResponse<UserDataExportWorkflowResponse>>, StatusCode> {
    tracing::info!("Starting user data export workflow for user {}", request.user_id);
    
    let workflow_context = create_mock_workflow_context(&tenant_context.tenant_id, "user_data_export_workflow");
    
    match user_data_export_workflow(workflow_context, request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn start_user_deactivation_workflow(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<UserDeactivationWorkflowRequest>,
) -> Result<Json<ApiResponse<UserDeactivationWorkflowResponse>>, StatusCode> {
    tracing::info!("Starting user deactivation workflow for user {}", request.user_id);
    
    let workflow_context = create_mock_workflow_context(&tenant_context.tenant_id, "user_deactivation_workflow");
    
    match user_deactivation_workflow(workflow_context, request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn start_user_reactivation_workflow(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<UserReactivationWorkflowRequest>,
) -> Result<Json<ApiResponse<UserReactivationWorkflowResponse>>, StatusCode> {
    tracing::info!("Starting user reactivation workflow for user {}", request.user_id);
    
    let workflow_context = create_mock_workflow_context(&tenant_context.tenant_id, "user_reactivation_workflow");
    
    match user_reactivation_workflow(workflow_context, request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}

pub async fn start_bulk_user_operation_workflow(
    Extension(tenant_context): Extension<TenantContext>,
    Json(request): Json<BulkUserOperationWorkflowRequest>,
) -> Result<Json<ApiResponse<BulkUserOperationWorkflowResponse>>, StatusCode> {
    tracing::info!("Starting bulk user operation workflow with {} operations", request.user_operations.len());
    
    let workflow_context = create_mock_workflow_context(&tenant_context.tenant_id, "bulk_user_operation_workflow");
    
    match bulk_user_operation_workflow(workflow_context, request).await {
        Ok(response) => Ok(Json(ApiResponse::success(response))),
        Err(e) => Ok(Json(ApiResponse::error(e.to_string()))),
    }
}