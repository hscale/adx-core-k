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
    types::{
        ApiResponse, PaginationParams, ResponseMeta, TenantContext,
        WorkflowMetrics, SystemHealth,
    },
    AppState,
};

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .route("/health", get(get_system_health))
        .route("/metrics", get(get_workflow_metrics))
        .route("/metrics/tenant/:tenant_id", get(get_tenant_workflow_metrics))
        .route("/metrics/workflow/:workflow_id", get(get_workflow_specific_metrics))
        .route("/performance", get(get_performance_metrics))
        .route("/alerts", get(get_workflow_alerts))
        .route("/capacity", get(get_capacity_metrics))
        .route("/trends", get(get_workflow_trends))
}

#[derive(Debug, Deserialize, Serialize)]
struct MetricsQuery {
    time_range: Option<String>, // e.g., "1h", "24h", "7d", "30d"
    granularity: Option<String>, // e.g., "minute", "hour", "day"
    workflow_type: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct PerformanceQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    sort_by: Option<String>, // e.g., "execution_time", "retry_count", "failure_rate"
    order: Option<String>, // "asc" or "desc"
}

#[derive(Debug, Deserialize, Serialize)]
struct AlertsQuery {
    #[serde(flatten)]
    pagination: PaginationParams,
    severity: Option<String>, // "low", "medium", "high", "critical"
    status: Option<String>, // "active", "resolved", "acknowledged"
}

// Get system health with caching
async fn get_system_health(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;

    // Check permissions
    if !has_permission(claims, "monitoring:read") {
        return Err(BffError::authorization("Insufficient permissions to view system health"));
    }

    // Try cache first
    if let Ok(Some(cached_health)) = state.redis.get_cached_system_health().await {
        debug!("Returning cached system health");
        return Ok(Json(ApiResponse {
            data: serde_json::to_value(cached_health)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(60),
            }),
        }));
    }

    // Check various system components
    let temporal_health = check_temporal_health(&state).await;
    let redis_health = check_redis_health(&state).await;
    let api_gateway_health = check_api_gateway_health(&state).await;

    let system_health = SystemHealth {
        overall_status: if temporal_health && redis_health && api_gateway_health {
            "healthy".to_string()
        } else {
            "degraded".to_string()
        },
        timestamp: chrono::Utc::now(),
        components: {
            let mut components = HashMap::new();
            components.insert("temporal".to_string(), serde_json::json!({
                "status": if temporal_health { "healthy" } else { "unhealthy" },
                "response_time_ms": 45,
                "last_check": chrono::Utc::now()
            }));
            components.insert("redis".to_string(), serde_json::json!({
                "status": if redis_health { "healthy" } else { "unhealthy" },
                "response_time_ms": 12,
                "last_check": chrono::Utc::now()
            }));
            components.insert("api_gateway".to_string(), serde_json::json!({
                "status": if api_gateway_health { "healthy" } else { "unhealthy" },
                "response_time_ms": 89,
                "last_check": chrono::Utc::now()
            }));
            components
        },
        metrics: serde_json::json!({
            "active_workflows": 234,
            "queued_workflows": 12,
            "failed_workflows_last_hour": 3,
            "average_workflow_duration_ms": 45000,
            "system_load": 0.65,
            "memory_usage_percent": 72.3,
            "cpu_usage_percent": 45.8
        }),
    };

    // Cache the result
    if let Err(e) = state.redis.cache_system_health(&system_health, Some(60)).await {
        error!("Failed to cache system health: {}", e);
    }

    Ok(Json(ApiResponse {
        data: serde_json::to_value(&system_health)?,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get workflow metrics with caching
async fn get_workflow_metrics(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "monitoring:read") {
        return Err(BffError::authorization("Insufficient permissions to view workflow metrics"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Create cache key based on query parameters
    let params_hash = create_params_hash(&query)?;
    
    // Try cache first
    if let Ok(Some(cached_metrics)) = state.redis.get_cached_workflow_metrics(tenant_id, &params_hash).await {
        debug!("Returning cached workflow metrics for tenant: {}", tenant_id);
        return Ok(Json(ApiResponse {
            data: serde_json::to_value(cached_metrics)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Generate metrics based on query parameters
    let time_range = query.time_range.as_deref().unwrap_or("24h");
    let granularity = query.granularity.as_deref().unwrap_or("hour");

    let metrics = WorkflowMetrics {
        tenant_id: tenant_id.clone(),
        time_range: time_range.to_string(),
        granularity: granularity.to_string(),
        total_executions: 1456,
        successful_executions: 1378,
        failed_executions: 78,
        average_duration_ms: 42000,
        median_duration_ms: 35000,
        p95_duration_ms: 89000,
        p99_duration_ms: 156000,
        throughput_per_hour: 60.7,
        error_rate: 5.4,
        retry_rate: 12.8,
        workflow_types: {
            let mut types = HashMap::new();
            types.insert("user_onboarding".to_string(), serde_json::json!({
                "count": 456,
                "success_rate": 98.2,
                "avg_duration_ms": 35000
            }));
            types.insert("data_processing".to_string(), serde_json::json!({
                "count": 678,
                "success_rate": 92.1,
                "avg_duration_ms": 67000
            }));
            types.insert("file_processing".to_string(), serde_json::json!({
                "count": 322,
                "success_rate": 89.4,
                "avg_duration_ms": 23000
            }));
            types
        },
        time_series: generate_time_series_data(time_range, granularity),
        generated_at: chrono::Utc::now(),
    };

    // Cache the result
    if let Err(e) = state.redis.cache_workflow_metrics(tenant_id, &params_hash, &metrics, Some(300)).await {
        error!("Failed to cache workflow metrics: {}", e);
    }

    info!("Generated workflow metrics for tenant: {} (time_range: {})", tenant_id, time_range);

    Ok(Json(ApiResponse {
        data: serde_json::to_value(&metrics)?,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get tenant-specific workflow metrics
async fn get_tenant_workflow_metrics(
    State(state): State<AppState>,
    Path(target_tenant_id): Path<String>,
    Query(query): Query<MetricsQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;

    // Check permissions (admin only for cross-tenant metrics)
    if !has_permission(claims, "monitoring:admin") {
        return Err(BffError::authorization("Insufficient permissions to view cross-tenant metrics"));
    }

    // Create cache key based on query parameters
    let params_hash = create_params_hash(&query)?;
    
    // Try cache first
    if let Ok(Some(cached_metrics)) = state.redis.get_cached_workflow_metrics(&target_tenant_id, &params_hash).await {
        debug!("Returning cached workflow metrics for tenant: {}", target_tenant_id);
        return Ok(Json(ApiResponse {
            data: serde_json::to_value(cached_metrics)?,
            meta: Some(ResponseMeta {
                total: None,
                page: None,
                per_page: None,
                cached: Some(true),
                cache_ttl: Some(300),
            }),
        }));
    }

    // Generate tenant-specific metrics
    let time_range = query.time_range.as_deref().unwrap_or("24h");
    let granularity = query.granularity.as_deref().unwrap_or("hour");

    let metrics = WorkflowMetrics {
        tenant_id: target_tenant_id.clone(),
        time_range: time_range.to_string(),
        granularity: granularity.to_string(),
        total_executions: 892,
        successful_executions: 845,
        failed_executions: 47,
        average_duration_ms: 38000,
        median_duration_ms: 32000,
        p95_duration_ms: 78000,
        p99_duration_ms: 134000,
        throughput_per_hour: 37.2,
        error_rate: 5.3,
        retry_rate: 11.2,
        workflow_types: HashMap::new(),
        time_series: generate_time_series_data(time_range, granularity),
        generated_at: chrono::Utc::now(),
    };

    // Cache the result
    if let Err(e) = state.redis.cache_workflow_metrics(&target_tenant_id, &params_hash, &metrics, Some(300)).await {
        error!("Failed to cache tenant workflow metrics: {}", e);
    }

    Ok(Json(ApiResponse {
        data: serde_json::to_value(&metrics)?,
        meta: Some(ResponseMeta {
            total: None,
            page: None,
            per_page: None,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get workflow-specific metrics
async fn get_workflow_specific_metrics(
    State(state): State<AppState>,
    Path(workflow_id): Path<String>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "monitoring:read") {
        return Err(BffError::authorization("Insufficient permissions to view workflow metrics"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Generate workflow-specific metrics
    let metrics = serde_json::json!({
        "workflow_id": workflow_id,
        "tenant_id": tenant_id,
        "execution_count": 1,
        "current_status": "running",
        "start_time": chrono::Utc::now() - chrono::Duration::minutes(15),
        "duration_ms": 900000, // 15 minutes
        "activities_completed": 8,
        "activities_total": 12,
        "retry_count": 1,
        "resource_usage": {
            "cpu_time_ms": 45000,
            "memory_peak_mb": 256,
            "network_bytes_sent": 1024000,
            "network_bytes_received": 2048000
        },
        "performance_metrics": {
            "activities_per_second": 0.53,
            "average_activity_duration_ms": 1875,
            "queue_time_ms": 120,
            "processing_time_ms": 899880
        },
        "error_details": null,
        "generated_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse {
        data: metrics,
        meta: None,
    }))
}

// Get performance metrics
async fn get_performance_metrics(
    State(state): State<AppState>,
    Query(query): Query<PerformanceQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "monitoring:read") {
        return Err(BffError::authorization("Insufficient permissions to view performance metrics"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Generate performance metrics
    let sort_by = query.sort_by.as_deref().unwrap_or("execution_time");
    let order = query.order.as_deref().unwrap_or("desc");

    let performance_data = serde_json::json!({
        "tenant_id": tenant_id,
        "sort_by": sort_by,
        "order": order,
        "slowest_workflows": [
            {
                "workflow_id": "workflow-123",
                "workflow_type": "data_processing",
                "execution_time_ms": 156000,
                "start_time": chrono::Utc::now() - chrono::Duration::hours(2),
                "status": "completed"
            },
            {
                "workflow_id": "workflow-456",
                "workflow_type": "file_processing",
                "execution_time_ms": 134000,
                "start_time": chrono::Utc::now() - chrono::Duration::hours(1),
                "status": "completed"
            }
        ],
        "most_retried_workflows": [
            {
                "workflow_id": "workflow-789",
                "workflow_type": "user_onboarding",
                "retry_count": 5,
                "last_retry": chrono::Utc::now() - chrono::Duration::minutes(30),
                "status": "running"
            }
        ],
        "highest_failure_rate_types": [
            {
                "workflow_type": "external_integration",
                "failure_rate": 15.6,
                "total_executions": 234,
                "failed_executions": 37
            }
        ],
        "resource_intensive_workflows": [
            {
                "workflow_id": "workflow-heavy-1",
                "workflow_type": "batch_processing",
                "cpu_time_ms": 89000,
                "memory_peak_mb": 1024,
                "execution_time_ms": 234000
            }
        ],
        "generated_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse {
        data: performance_data,
        meta: Some(ResponseMeta {
            total: None,
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get workflow alerts
async fn get_workflow_alerts(
    State(state): State<AppState>,
    Query(query): Query<AlertsQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "monitoring:read") {
        return Err(BffError::authorization("Insufficient permissions to view workflow alerts"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Generate alerts based on query parameters
    let severity_filter = query.severity.as_deref();
    let status_filter = query.status.as_deref();

    let alerts = serde_json::json!({
        "tenant_id": tenant_id,
        "filters": {
            "severity": severity_filter,
            "status": status_filter
        },
        "alerts": [
            {
                "id": "alert-001",
                "title": "High Workflow Failure Rate",
                "description": "Workflow failure rate exceeded 10% threshold",
                "severity": "high",
                "status": "active",
                "workflow_type": "data_processing",
                "threshold": 10.0,
                "current_value": 15.6,
                "created_at": chrono::Utc::now() - chrono::Duration::hours(2),
                "updated_at": chrono::Utc::now() - chrono::Duration::minutes(30),
                "actions": [
                    "Check external service connectivity",
                    "Review workflow configuration",
                    "Scale up worker capacity"
                ]
            },
            {
                "id": "alert-002",
                "title": "Workflow Queue Backlog",
                "description": "Workflow queue depth exceeded normal capacity",
                "severity": "medium",
                "status": "acknowledged",
                "workflow_type": "all",
                "threshold": 50,
                "current_value": 78,
                "created_at": chrono::Utc::now() - chrono::Duration::hours(1),
                "updated_at": chrono::Utc::now() - chrono::Duration::minutes(15),
                "actions": [
                    "Scale up worker instances",
                    "Review workflow priorities",
                    "Check resource constraints"
                ]
            },
            {
                "id": "alert-003",
                "title": "Long Running Workflow",
                "description": "Workflow exceeded maximum execution time",
                "severity": "low",
                "status": "resolved",
                "workflow_type": "file_processing",
                "workflow_id": "workflow-long-123",
                "threshold": 3600000, // 1 hour in ms
                "current_value": 4200000, // 70 minutes
                "created_at": chrono::Utc::now() - chrono::Duration::hours(3),
                "updated_at": chrono::Utc::now() - chrono::Duration::minutes(45),
                "resolution": "Workflow completed successfully after optimization"
            }
        ],
        "summary": {
            "total_alerts": 3,
            "active_alerts": 1,
            "acknowledged_alerts": 1,
            "resolved_alerts": 1,
            "critical_alerts": 0,
            "high_alerts": 1,
            "medium_alerts": 1,
            "low_alerts": 1
        },
        "generated_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse {
        data: alerts,
        meta: Some(ResponseMeta {
            total: Some(3),
            page: query.pagination.page,
            per_page: query.pagination.per_page,
            cached: Some(false),
            cache_ttl: None,
        }),
    }))
}

// Get capacity metrics
async fn get_capacity_metrics(
    State(state): State<AppState>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "monitoring:read") {
        return Err(BffError::authorization("Insufficient permissions to view capacity metrics"));
    }

    let tenant_id = &tenant_context.tenant_id;

    // Generate capacity metrics
    let capacity_data = serde_json::json!({
        "tenant_id": tenant_id,
        "current_capacity": {
            "active_workers": 12,
            "max_workers": 20,
            "worker_utilization": 60.0,
            "queue_depth": 45,
            "max_queue_depth": 1000,
            "queue_utilization": 4.5
        },
        "resource_usage": {
            "cpu_usage_percent": 45.8,
            "memory_usage_percent": 72.3,
            "disk_usage_percent": 34.2,
            "network_usage_mbps": 12.4
        },
        "scaling_recommendations": [
            {
                "component": "workflow_workers",
                "current": 12,
                "recommended": 16,
                "reason": "Queue depth trending upward",
                "priority": "medium"
            },
            {
                "component": "memory",
                "current": "8GB",
                "recommended": "12GB",
                "reason": "Memory usage consistently above 70%",
                "priority": "low"
            }
        ],
        "capacity_trends": {
            "worker_utilization_trend": "increasing",
            "queue_depth_trend": "stable",
            "resource_usage_trend": "stable"
        },
        "forecasts": {
            "next_24h": {
                "expected_workflows": 1440,
                "expected_peak_workers": 18,
                "expected_peak_queue": 89
            },
            "next_7d": {
                "expected_workflows": 10080,
                "scaling_events": 3,
                "maintenance_windows": 1
            }
        },
        "generated_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse {
        data: capacity_data,
        meta: None,
    }))
}

// Get workflow trends
async fn get_workflow_trends(
    State(state): State<AppState>,
    Query(query): Query<MetricsQuery>,
    request: Request,
) -> BffResult<Json<ApiResponse<serde_json::Value>>> {
    let claims = request.extensions().get::<Claims>()
        .ok_or_else(|| BffError::authentication("Missing authentication claims"))?;
    
    let tenant_context = get_tenant_context(&request)
        .ok_or_else(|| BffError::tenant_validation("Missing tenant context"))?;

    // Check permissions
    if !has_permission(claims, "monitoring:read") {
        return Err(BffError::authorization("Insufficient permissions to view workflow trends"));
    }

    let tenant_id = &tenant_context.tenant_id;
    let time_range = query.time_range.as_deref().unwrap_or("7d");

    // Generate trend data
    let trends_data = serde_json::json!({
        "tenant_id": tenant_id,
        "time_range": time_range,
        "execution_trends": {
            "total_executions": {
                "current_period": 5420,
                "previous_period": 4890,
                "change_percent": 10.8,
                "trend": "increasing"
            },
            "success_rate": {
                "current_period": 94.5,
                "previous_period": 92.1,
                "change_percent": 2.6,
                "trend": "improving"
            },
            "average_duration": {
                "current_period": 42000,
                "previous_period": 45000,
                "change_percent": -6.7,
                "trend": "improving"
            }
        },
        "workflow_type_trends": {
            "user_onboarding": {
                "executions": 1234,
                "change_percent": 15.2,
                "trend": "increasing",
                "success_rate": 98.2
            },
            "data_processing": {
                "executions": 2156,
                "change_percent": 8.7,
                "trend": "increasing",
                "success_rate": 92.1
            },
            "file_processing": {
                "executions": 1890,
                "change_percent": -2.3,
                "trend": "decreasing",
                "success_rate": 89.4
            }
        },
        "performance_trends": {
            "p95_duration": {
                "current": 89000,
                "previous": 95000,
                "change_percent": -6.3,
                "trend": "improving"
            },
            "retry_rate": {
                "current": 12.8,
                "previous": 15.2,
                "change_percent": -15.8,
                "trend": "improving"
            },
            "throughput": {
                "current": 60.7,
                "previous": 54.3,
                "change_percent": 11.8,
                "trend": "increasing"
            }
        },
        "anomalies": [
            {
                "type": "execution_spike",
                "detected_at": chrono::Utc::now() - chrono::Duration::hours(6),
                "description": "Unusual spike in user_onboarding workflows",
                "severity": "low",
                "impact": "Temporary queue backlog"
            }
        ],
        "predictions": {
            "next_24h": {
                "expected_executions": 720,
                "confidence": 85.2
            },
            "next_7d": {
                "expected_executions": 5040,
                "confidence": 78.9
            }
        },
        "generated_at": chrono::Utc::now()
    });

    Ok(Json(ApiResponse {
        data: trends_data,
        meta: None,
    }))
}

// Helper functions
async fn check_temporal_health(state: &AppState) -> bool {
    match state.temporal_client.health_check().await {
        Ok(_) => true,
        Err(e) => {
            error!("Temporal health check failed: {}", e);
            false
        }
    }
}

async fn check_redis_health(state: &AppState) -> bool {
    match state.redis.health_check().await {
        Ok(_) => true,
        Err(e) => {
            error!("Redis health check failed: {}", e);
            false
        }
    }
}

async fn check_api_gateway_health(state: &AppState) -> bool {
    // In a real implementation, this would ping the API Gateway
    // For now, we'll assume it's healthy
    true
}

fn generate_time_series_data(time_range: &str, granularity: &str) -> Vec<serde_json::Value> {
    // Generate mock time series data based on time range and granularity
    let points = match time_range {
        "1h" => 60,
        "24h" => 24,
        "7d" => 7,
        "30d" => 30,
        _ => 24,
    };

    (0..points).map(|i| {
        let timestamp = chrono::Utc::now() - chrono::Duration::hours(points - i);
        serde_json::json!({
            "timestamp": timestamp,
            "executions": 50 + (i * 2),
            "successes": 47 + (i * 2),
            "failures": 3,
            "avg_duration_ms": 40000 + (i * 1000),
            "throughput": 2.1 + (i as f64 * 0.1)
        })
    }).collect()
}

fn create_params_hash<T: serde::Serialize>(params: &T) -> BffResult<String> {
    let params_json = serde_json::to_string(params)?;
    let hash = format!("{:x}", md5::compute(params_json.as_bytes()));
    Ok(hash)
}