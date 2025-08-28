use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

pub async fn handle_error() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(json!({
            "error": {
                "code": "NOT_FOUND",
                "message": "The requested resource was not found"
            },
            "timestamp": chrono::Utc::now().to_rfc3339()
        })),
    )
        .into_response()
}