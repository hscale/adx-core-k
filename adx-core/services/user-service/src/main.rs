use adx_shared::{init_tracing, ApiResponse, ResponseMetadata, TenantId, UserId};
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone)]
pub struct AppState {
    // Simplified for now - database will be added later
}

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub tenant_id: TenantId,
    pub email: String,
    pub profile: serde_json::Value,
    pub is_active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let app_state = AppState {};

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/users", get(list_users))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8082").await.unwrap();
    tracing::info!(
        "User Service listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "User Service OK"
}

async fn list_users(_state: State<AppState>) -> Result<Json<ApiResponse<Vec<User>>>, StatusCode> {
    // Mock implementation for now - will be replaced with actual database queries
    let mock_users = vec![User {
        id: Uuid::new_v4(),
        tenant_id: Uuid::new_v4(),
        email: "admin@example.com".to_string(),
        profile: serde_json::json!({"name": "Admin User"}),
        is_active: true,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }];

    let response = ApiResponse {
        data: mock_users,
        metadata: ResponseMetadata {
            correlation_id: uuid::Uuid::new_v4(),
            timestamp: chrono::Utc::now(),
            version: "1.0.0".to_string(),
        },
    };

    Ok(Json(response))
}
