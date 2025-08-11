use axum::{
    routing::{get, post, put},
    Router,
    middleware,
};

use crate::{
    handlers::{auth, users, health},
    middleware::{
        auth::auth_middleware,
        tenant::tenant_context_middleware,
        rate_limit::rate_limit_middleware,
        logging::{cors_middleware, security_headers_middleware, request_logging_middleware},
    },
    AppState,
};

pub fn create_routes(state: AppState) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(health::health_check))
        .route("/auth/register", post(auth::register))
        .route("/auth/login", post(auth::login))
        .route("/auth/password-reset", post(auth::request_password_reset));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/auth/profile", get(users::get_user_profile))
        .route("/auth/profile", put(users::update_user_profile))
        .route("/auth/password", put(users::change_password))
        .route("/users/:user_id", get(users::get_user_by_id))
        .layer(middleware::from_fn_with_state(state.clone(), tenant_context_middleware))
        .layer(middleware::from_fn_with_state(state.clone(), auth_middleware));

    // Combine all routes with common middleware
    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(middleware::from_fn_with_state(state.clone(), rate_limit_middleware))
        .layer(middleware::from_fn(request_logging_middleware))
        .layer(middleware::from_fn(security_headers_middleware))
        .layer(middleware::from_fn(cors_middleware))
        .with_state(state)
}

// API versioning routes
pub fn create_versioned_routes(state: AppState) -> Router {
    let v1_routes = create_routes(state.clone())
        .nest("/api/v1", create_routes(state.clone()));

    Router::new()
        .merge(v1_routes)
        .fallback(not_found_handler)
}

async fn not_found_handler() -> (axum::http::StatusCode, axum::Json<serde_json::Value>) {
    (
        axum::http::StatusCode::NOT_FOUND,
        axum::Json(serde_json::json!({
            "error": {
                "code": "NOT_FOUND",
                "message": "The requested endpoint was not found"
            }
        })),
    )
}