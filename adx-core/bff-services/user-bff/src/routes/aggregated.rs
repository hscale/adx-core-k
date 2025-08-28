use axum::{
    extract::{State, Extension, Query},
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::{AppState, middleware::{auth::Claims, tenant::TenantContext}};

#[derive(Debug, Deserialize)]
struct DashboardQuery {
    include: Option<String>, // comma-separated list: profile,tenants,activity,workflows
}

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/dashboard", get(get_aggregated_dashboard))
        .route("/user-summary", get(get_user_summary))
}

async fn get_aggregated_dashboard(
    State(state): State<AppState>,
    Query(query): Query<DashboardQuery>,
    Extension(claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &claims.sub;
    
    // Parse what to include
    let include_items: Vec<&str> = query.include
        .as_deref()
        .unwrap_or("profile,tenants,activity,workflows")
        .split(',')
        .collect();

    // Check cache first
    let cache_key = format!("dashboard:{}:{}", user_id, include_items.join(","));
    
    let mut dashboard = json!({
        "user_id": user_id,
        "generated_at": chrono::Utc::now().to_rfc3339()
    });

    let token = ""; // In real implementation, extract from request

    // Fetch requested data in parallel
    let mut tasks = Vec::new();

    if include_items.contains(&"profile") {
        let api_client = state.api_client.clone();
        let user_id = user_id.clone();
        let token = token.to_string();
        tasks.push(tokio::spawn(async move {
            ("profile", api_client.get_user_profile(&user_id, &token).await)
        }));
    }

    if include_items.contains(&"tenants") {
        let api_client = state.api_client.clone();
        let user_id = user_id.clone();
        let token = token.to_string();
        tasks.push(tokio::spawn(async move {
            ("tenants", api_client.get_user_tenants(&user_id, &token).await)
        }));
    }

    if include_items.contains(&"activity") {
        let api_client = state.api_client.clone();
        let user_id = user_id.clone();
        let token = token.to_string();
        tasks.push(tokio::spawn(async move {
            ("activity", api_client.get_user_activity(&user_id, &token).await)
        }));
    }

    if include_items.contains(&"workflows") {
        let temporal_client = state.temporal_client.clone();
        let user_id = user_id.clone();
        tasks.push(tokio::spawn(async move {
            ("workflows", temporal_client.get_user_workflows(&user_id).await.map(|w| json!(w)))
        }));
    }

    // Wait for all tasks to complete
    for task in tasks {
        if let Ok((key, result)) = task.await {
            match result {
                Ok(data) => {
                    dashboard[key] = data;
                }
                Err(_) => {
                    dashboard[key] = json!(null);
                }
            }
        }
    }

    // Cache the result
    let _ = state.redis.cache_aggregated_dashboard(user_id, &dashboard, 300).await;

    Ok(Json(dashboard))
}

async fn get_user_summary(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Extension(_tenant): Extension<TenantContext>,
) -> Result<Json<Value>, StatusCode> {
    let user_id = &claims.sub;
    
    // Get basic user info and create a summary
    let token = ""; // In real implementation, extract from request
    
    let user_data = state.api_client.get_user(user_id, token).await.ok();
    let workflows = state.temporal_client.get_user_workflows(user_id).await.ok();
    
    let workflow_summary = workflows.as_ref().map(|w| {
        let total = w.len();
        let completed = w.iter().filter(|wf| wf.status == "COMPLETED").count();
        let running = w.iter().filter(|wf| wf.status == "RUNNING").count();
        let failed = w.iter().filter(|wf| wf.status == "FAILED").count();
        
        json!({
            "total": total,
            "completed": completed,
            "running": running,
            "failed": failed
        })
    });

    let summary = json!({
        "user_id": user_id,
        "user": user_data,
        "workflow_summary": workflow_summary,
        "last_updated": chrono::Utc::now().to_rfc3339()
    });

    Ok(Json(summary))
}