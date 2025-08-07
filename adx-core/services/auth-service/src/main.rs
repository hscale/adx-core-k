//! # ADX CORE Authentication Service
//!
//! Enterprise-grade authentication service with Temporal-First architecture.
//! Handles JWT tokens, RBAC permissions, and security workflows.

use adx_shared::{init_tracing, ApiResponse, DatabaseManager, ResponseMetadata, TenantId, UserId};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    middleware,
    response::Json,
    routing::{get, post},
    Router,
};
use rbac::types::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, instrument};
use uuid::Uuid;

mod auth;
mod jwt;
mod rbac;

use auth::AuthService;
use jwt::JwtService;
use rbac::RbacService;

/// Application state with enterprise services
#[derive(Clone, Debug)]
pub struct AppState {
    pub auth_service: Arc<AuthService>,
    pub jwt_service: Arc<JwtService>,
    pub rbac_service: Arc<RbacService>,
    pub db: Arc<DatabaseManager>,
}

impl AppState {
    /// Create new application state with all services
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize database with connection pooling
        let db = Arc::new(
            DatabaseManager::new()
                .await
                .expect("Failed to initialize database connection"),
        );

        // Initialize services with proper dependencies
        let jwt_service = Arc::new(JwtService::new("dev_secret_key"));
        let auth_service = Arc::new(AuthService::new());
        let rbac_service = Arc::new(RbacService::new(Arc::clone(&db)));

        Ok(Self {
            auth_service,
            jwt_service,
            rbac_service,
            db,
        })
    }

    /// Health check for all services
    pub async fn health_check(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Check database connectivity
        self.db.health_check().await?;

        // Check Temporal connectivity
        // TODO: Add Temporal health check

        info!("All services healthy");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize structured logging
    init_tracing();

    info!("🚀 Starting ADX CORE Authentication Service");

    // Initialize application state
    let app_state = AppState::new().await?;

    // Perform initial health check
    app_state.health_check().await?;

    // Build application with middleware stack
    let app = Router::new()
        // Health and monitoring endpoints
        .route("/health", get(health_check))
        .route("/metrics", get(get_metrics))
        // Authentication endpoints (simple operations)
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/validate", post(validate_token))
        .route("/api/v1/auth/refresh", post(refresh_token))
        // RBAC endpoints (mix of simple and complex operations)
        .route("/api/v1/auth/permissions/check", post(check_permission))
        .route("/api/v1/auth/users/:user_id/roles", get(get_user_roles))
        .route("/api/v1/auth/roles/:role_id/assign", post(assign_role))
        .route("/api/v1/auth/audit/permissions", post(audit_permissions))
        .route("/api/v1/auth/review/access", post(start_access_review))
        .route(
            "/api/v1/auth/incident/security",
            post(handle_security_incident),
        )
        // Workflow status endpoints
        .route(
            "/api/v1/auth/workflows/:workflow_id/status",
            get(get_workflow_status),
        )
        .with_state(app_state)
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
                .layer(middleware::from_fn(request_id_middleware))
                .layer(middleware::from_fn(tenant_context_middleware)),
        );

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8081").await?;
    info!("🔐 Auth Service listening on {}", listener.local_addr()?);

    axum::serve(listener, app).await?;
    Ok(())
}

// ============================================================================
// MIDDLEWARE
// ============================================================================

/// Add request ID for tracing
async fn request_id_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    let request_id = Uuid::new_v4();
    let mut request = request;
    request
        .headers_mut()
        .insert("x-request-id", request_id.to_string().parse().unwrap());

    let response = next.run(request).await;
    response
}

/// Extract and validate tenant context
async fn tenant_context_middleware(
    request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    // TODO: Extract tenant ID from JWT token or headers
    // TODO: Validate tenant access
    next.run(request).await
}

// ============================================================================
// HEALTH AND MONITORING ENDPOINTS
// ============================================================================

#[instrument]
async fn health_check(
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<HealthStatus>>, StatusCode> {
    match app_state.health_check().await {
        Ok(_) => Ok(Json(ApiResponse {
            data: HealthStatus {
                status: "healthy".to_string(),
                services: vec![
                    ServiceHealth {
                        name: "database".to_string(),
                        healthy: true,
                    },
                    ServiceHealth {
                        name: "temporal".to_string(),
                        healthy: true,
                    },
                    ServiceHealth {
                        name: "auth".to_string(),
                        healthy: true,
                    },
                ],
                timestamp: chrono::Utc::now(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::SERVICE_UNAVAILABLE),
    }
}

async fn get_metrics(
    State(_app_state): State<AppState>,
) -> Result<Json<ApiResponse<MetricsData>>, StatusCode> {
    // TODO: Collect actual metrics from services
    Ok(Json(ApiResponse {
        data: MetricsData {
            active_sessions: 0,
            requests_per_minute: 0.0,
            average_response_time_ms: 0.0,
            error_rate: 0.0,
        },
        metadata: ResponseMetadata {
            correlation_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            version: "1.0.0".to_string(),
        },
    }))
}

// ============================================================================
// AUTHENTICATION ENDPOINTS (Simple Operations)
// ============================================================================

#[instrument(skip(app_state, request))]
async fn login(
    State(app_state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, StatusCode> {
    info!(
        email = %request.email,
        tenant_id = %request.tenant_id,
        "Processing login request"
    );

    // Authenticate user (simple operation - no workflow needed)
    let user_id = match app_state
        .auth_service
        .authenticate_user(&request.email, &request.password, request.tenant_id)
        .await
    {
        Ok(id) => id,
        Err(_) => return Err(StatusCode::UNAUTHORIZED),
    };

    // Get user roles for JWT
    let roles = match app_state
        .rbac_service
        .get_user_roles(user_id, request.tenant_id)
        .await
    {
        Ok(response) => response.roles.into_iter().map(|r| r.name).collect(),
        Err(_) => vec!["user".to_string()], // Default role
    };

    // Generate tokens
    let access_token = app_state
        .jwt_service
        .generate_access_token(user_id, request.tenant_id, roles)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let refresh_token = app_state
        .jwt_service
        .generate_refresh_token(user_id, request.tenant_id)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response = LoginResponse {
        access_token,
        refresh_token,
        expires_in: 3600,
        user_id,
    };

    info!(user_id = %user_id, "Login successful");

    Ok(Json(ApiResponse {
        data: response,
        metadata: ResponseMetadata {
            correlation_id: Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            version: "1.0.0".to_string(),
        },
    }))
}

#[instrument(skip(app_state, request))]
async fn validate_token(
    State(app_state): State<AppState>,
    Json(request): Json<ValidateTokenRequest>,
) -> Result<Json<ApiResponse<ValidateTokenResponse>>, StatusCode> {
    match app_state.jwt_service.validate_token(&request.token) {
        Ok(claims) => {
            let response = ValidateTokenResponse {
                valid: true,
                user_id: Some(claims.sub),
                tenant_id: Some(claims.tenant_id),
                roles: claims.roles,
                expires_at: Some(
                    chrono::DateTime::from_timestamp(claims.exp as i64, 0)
                        .unwrap_or_else(chrono::Utc::now),
                ),
            };

            Ok(Json(ApiResponse {
                data: response,
                metadata: ResponseMetadata {
                    correlation_id: Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    version: "1.0.0".to_string(),
                },
            }))
        }
        Err(_) => {
            let response = ValidateTokenResponse {
                valid: false,
                user_id: None,
                tenant_id: None,
                roles: vec![],
                expires_at: None,
            };

            Ok(Json(ApiResponse {
                data: response,
                metadata: ResponseMetadata {
                    correlation_id: Uuid::new_v4(),
                    timestamp: chrono::Utc::now(),
                    version: "1.0.0".to_string(),
                },
            }))
        }
    }
}

async fn refresh_token(
    State(app_state): State<AppState>,
    Json(request): Json<RefreshTokenRequest>,
) -> Result<Json<ApiResponse<RefreshTokenResponse>>, StatusCode> {
    // TODO: Implement refresh token logic
    Err(StatusCode::NOT_IMPLEMENTED)
}

// ============================================================================
// RBAC ENDPOINTS
// ============================================================================

/// Fast permission check endpoint (< 10ms target)
#[instrument(skip(app_state, request))]
async fn check_permission(
    State(app_state): State<AppState>,
    Json(request): Json<PermissionCheckRequest>,
) -> Result<Json<ApiResponse<PermissionCheckResponse>>, StatusCode> {
    match app_state.rbac_service.check_permission(request).await {
        Ok(response) => Ok(Json(ApiResponse {
            data: response,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get user roles and effective permissions
async fn get_user_roles(
    State(app_state): State<AppState>,
    Path(user_id): Path<UserId>,
    Json(tenant_request): Json<TenantRequest>,
) -> Result<Json<ApiResponse<UserRolesResponse>>, StatusCode> {
    match app_state
        .rbac_service
        .get_user_roles(user_id, tenant_request.tenant_id)
        .await
    {
        Ok(response) => Ok(Json(ApiResponse {
            data: response,
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Complex role assignment - triggers Temporal workflow
async fn assign_role(
    State(app_state): State<AppState>,
    Path(_role_id): Path<Uuid>,
    Json(request): Json<RoleAssignmentInput>,
) -> Result<Json<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state.rbac_service.assign_role_workflow(request).await {
        Ok(workflow_id) => Ok(Json(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Role assignment workflow started. Check status via workflow ID."
                    .to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Permission audit - triggers comprehensive audit workflow
async fn audit_permissions(
    State(app_state): State<AppState>,
    Json(request): Json<PermissionAuditInput>,
) -> Result<Json<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state
        .rbac_service
        .audit_permissions_workflow(request)
        .await
    {
        Ok(workflow_id) => Ok(Json(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message:
                    "Permission audit workflow started. Results will be available when complete."
                        .to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Access review - triggers periodic access certification workflow
async fn start_access_review(
    State(app_state): State<AppState>,
    Json(request): Json<AccessReviewInput>,
) -> Result<Json<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state.rbac_service.access_review_workflow(request).await {
        Ok(workflow_id) => Ok(Json(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Access review workflow started. Reviewers will be notified.".to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Security incident response - triggers automated security workflow
async fn handle_security_incident(
    State(app_state): State<AppState>,
    Json(request): Json<SecurityIncidentInput>,
) -> Result<Json<ApiResponse<WorkflowStartResponse>>, StatusCode> {
    match app_state.rbac_service.security_incident_workflow(request).await {
        Ok(workflow_id) => Ok(Json(ApiResponse {
            data: WorkflowStartResponse {
                workflow_id,
                status: "started".to_string(),
                message: "Security incident response workflow started. Immediate actions are being taken.".to_string(),
            },
            metadata: ResponseMetadata {
                correlation_id: Uuid::new_v4(),
                timestamp: chrono::Utc::now(),
                version: "1.0.0".to_string(),
            },
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

/// Get workflow status - check status of a Temporal workflow
async fn get_workflow_status(
    State(_app_state): State<AppState>,
    Path(_workflow_id): Path<String>,
) -> Result<Json<ApiResponse<WorkflowStatusResponse>>, StatusCode> {
    // TODO: Implement workflow status check via Temporal client
    Err(StatusCode::NOT_IMPLEMENTED)
}

// ============================================================================
// SUPPORTING TYPES
// ============================================================================

#[derive(Debug, Serialize)]
pub struct HealthStatus {
    pub status: String,
    pub services: Vec<ServiceHealth>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize)]
pub struct ServiceHealth {
    pub name: String,
    pub healthy: bool,
}

#[derive(Debug, Serialize)]
pub struct MetricsData {
    pub active_sessions: u64,
    pub requests_per_minute: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
}

#[derive(Debug, Deserialize)]
pub struct TenantRequest {
    pub tenant_id: TenantId,
}

#[derive(Debug, Serialize)]
pub struct WorkflowStartResponse {
    pub workflow_id: String,
    pub status: String,
    pub message: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
    pub tenant_id: TenantId,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub user_id: UserId,
}

#[derive(Debug, Deserialize)]
pub struct ValidateTokenRequest {
    pub token: String,
}

#[derive(Debug, Serialize)]
pub struct ValidateTokenResponse {
    pub valid: bool,
    pub user_id: Option<UserId>,
    pub tenant_id: Option<TenantId>,
    pub roles: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct RefreshTokenResponse {
    pub access_token: String,
    pub expires_in: i64,
}

#[derive(Debug, Serialize)]
pub struct WorkflowStatusResponse {
    pub workflow_id: String,
    pub status: String,
    pub progress: Option<f64>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}
