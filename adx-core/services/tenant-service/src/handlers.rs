use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;

use crate::models::*;
use crate::services::TenantService;
use adx_shared::types::{TenantId, UserId, PaginatedResponse, PaginationInfo};

pub type TenantServiceState = Arc<TenantService>;

#[derive(Debug, Deserialize)]
pub struct ListTenantsQuery {
    pub page: Option<u32>,
    pub limit: Option<u32>,
}

// Tenant handlers
pub async fn create_tenant(
    State(service): State<TenantServiceState>,
    Json(request): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<Tenant>), (StatusCode, Json<serde_json::Value>)> {
    match service.create_tenant(request).await {
        Ok(tenant) => Ok((StatusCode::CREATED, Json(tenant))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {
                    "code": "TENANT_CREATION_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn get_tenant(
    State(service): State<TenantServiceState>,
    Path(id): Path<TenantId>,
) -> Result<Json<Tenant>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_tenant(&id).await {
        Ok(Some(tenant)) => Ok(Json(tenant)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": {
                    "code": "TENANT_NOT_FOUND",
                    "message": "Tenant not found"
                }
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn get_tenant_by_slug(
    State(service): State<TenantServiceState>,
    Path(slug): Path<String>,
) -> Result<Json<Tenant>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_tenant_by_slug(&slug).await {
        Ok(Some(tenant)) => Ok(Json(tenant)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": {
                    "code": "TENANT_NOT_FOUND",
                    "message": "Tenant not found"
                }
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn list_tenants(
    State(service): State<TenantServiceState>,
    Query(params): Query<ListTenantsQuery>,
) -> Result<Json<PaginatedResponse<Tenant>>, (StatusCode, Json<serde_json::Value>)> {
    let page = params.page.unwrap_or(1);
    let limit = params.limit.unwrap_or(50);
    let offset = (page - 1) * limit;

    match service.list_tenants(Some(limit), Some(offset)).await {
        Ok(tenants) => {
            // For simplicity, we're not implementing total count here
            // In a real implementation, you'd want to get the total count
            let pagination = PaginationInfo {
                page,
                limit,
                total: tenants.len() as u64, // This is incorrect but simplified
                total_pages: 1, // This should be calculated properly
                has_next: tenants.len() == limit as usize,
                has_prev: page > 1,
            };

            Ok(Json(PaginatedResponse {
                data: tenants,
                pagination,
            }))
        }
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn update_tenant(
    State(service): State<TenantServiceState>,
    Path(id): Path<TenantId>,
    Json(request): Json<UpdateTenantRequest>,
) -> Result<Json<Tenant>, (StatusCode, Json<serde_json::Value>)> {
    match service.update_tenant(&id, request).await {
        Ok(tenant) => Ok(Json(tenant)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            
            Err((
                status,
                Json(serde_json::json!({
                    "error": {
                        "code": "TENANT_UPDATE_FAILED",
                        "message": e.to_string()
                    }
                })),
            ))
        }
    }
}

pub async fn delete_tenant(
    State(service): State<TenantServiceState>,
    Path(id): Path<TenantId>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match service.delete_tenant(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(serde_json::json!({
                    "error": {
                        "code": "TENANT_DELETE_FAILED",
                        "message": e.to_string()
                    }
                })),
            ))
        }
    }
}

// Membership handlers
pub async fn create_membership(
    State(service): State<TenantServiceState>,
    Path(tenant_id): Path<TenantId>,
    Json(request): Json<CreateMembershipRequest>,
) -> Result<(StatusCode, Json<TenantMembership>), (StatusCode, Json<serde_json::Value>)> {
    match service.create_membership(&tenant_id, request).await {
        Ok(membership) => Ok((StatusCode::CREATED, Json(membership))),
        Err(e) => Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": {
                    "code": "MEMBERSHIP_CREATION_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn get_membership(
    State(service): State<TenantServiceState>,
    Path(id): Path<String>,
) -> Result<Json<TenantMembership>, (StatusCode, Json<serde_json::Value>)> {
    match service.get_membership(&id).await {
        Ok(Some(membership)) => Ok(Json(membership)),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": {
                    "code": "MEMBERSHIP_NOT_FOUND",
                    "message": "Membership not found"
                }
            })),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn list_tenant_members(
    State(service): State<TenantServiceState>,
    Path(tenant_id): Path<TenantId>,
) -> Result<Json<Vec<TenantMembership>>, (StatusCode, Json<serde_json::Value>)> {
    match service.list_tenant_members(&tenant_id).await {
        Ok(memberships) => Ok(Json(memberships)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn list_user_memberships(
    State(service): State<TenantServiceState>,
    Path(user_id): Path<UserId>,
) -> Result<Json<Vec<TenantMembership>>, (StatusCode, Json<serde_json::Value>)> {
    match service.list_user_memberships(&user_id).await {
        Ok(memberships) => Ok(Json(memberships)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "INTERNAL_ERROR",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn update_membership(
    State(service): State<TenantServiceState>,
    Path(id): Path<String>,
    Json(request): Json<UpdateMembershipRequest>,
) -> Result<Json<TenantMembership>, (StatusCode, Json<serde_json::Value>)> {
    match service.update_membership(&id, request).await {
        Ok(membership) => Ok(Json(membership)),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_REQUEST
            };
            
            Err((
                status,
                Json(serde_json::json!({
                    "error": {
                        "code": "MEMBERSHIP_UPDATE_FAILED",
                        "message": e.to_string()
                    }
                })),
            ))
        }
    }
}

pub async fn delete_membership(
    State(service): State<TenantServiceState>,
    Path(id): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    match service.delete_membership(&id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => {
            let status = if e.to_string().contains("not found") {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(serde_json::json!({
                    "error": {
                        "code": "MEMBERSHIP_DELETE_FAILED",
                        "message": e.to_string()
                    }
                })),
            ))
        }
    }
}

// Tenant switching handlers
pub async fn switch_tenant(
    State(service): State<TenantServiceState>,
    // TODO: Extract user_id from JWT token in middleware
    Json(request): Json<SwitchTenantRequest>,
) -> Result<Json<SwitchTenantResponse>, (StatusCode, Json<serde_json::Value>)> {
    // For now, we'll use a placeholder user_id
    // In a real implementation, this would come from the authenticated user context
    let user_id = "placeholder-user-id".to_string();
    
    match service.switch_tenant(&user_id, request).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => {
            let status = if e.to_string().contains("not found") || e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::BAD_REQUEST
            };
            
            Err((
                status,
                Json(serde_json::json!({
                    "error": {
                        "code": "TENANT_SWITCH_FAILED",
                        "message": e.to_string()
                    }
                })),
            ))
        }
    }
}

pub async fn get_tenant_context(
    State(service): State<TenantServiceState>,
    Path(tenant_id): Path<TenantId>,
    // TODO: Extract user_id from JWT token in middleware
) -> Result<Json<TenantContext>, (StatusCode, Json<serde_json::Value>)> {
    // For now, we'll use a placeholder user_id
    // In a real implementation, this would come from the authenticated user context
    let user_id = "placeholder-user-id".to_string();
    
    match service.get_tenant_context(&tenant_id, &user_id).await {
        Ok(context) => Ok(Json(context)),
        Err(e) => {
            let status = if e.to_string().contains("not found") || e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(serde_json::json!({
                    "error": {
                        "code": "TENANT_CONTEXT_FAILED",
                        "message": e.to_string()
                    }
                })),
            ))
        }
    }
}

pub async fn get_current_tenant_context(
    State(service): State<TenantServiceState>,
    // TODO: Extract user_id and tenant_id from JWT token in middleware
) -> Result<Json<TenantContext>, (StatusCode, Json<serde_json::Value>)> {
    // For now, we'll use placeholder values
    // In a real implementation, these would come from the authenticated user context
    let user_id = "placeholder-user-id".to_string();
    let tenant_id = "placeholder-tenant-id".to_string();
    
    match service.get_tenant_context(&tenant_id, &user_id).await {
        Ok(context) => Ok(Json(context)),
        Err(e) => {
            let status = if e.to_string().contains("not found") || e.to_string().contains("does not have access") {
                StatusCode::FORBIDDEN
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            
            Err((
                status,
                Json(serde_json::json!({
                    "error": {
                        "code": "TENANT_CONTEXT_FAILED",
                        "message": e.to_string()
                    }
                })),
            ))
        }
    }
}

// Tenant validation and access control handlers
pub async fn validate_tenant_access(
    State(service): State<TenantServiceState>,
    Path((tenant_id, user_id)): Path<(TenantId, UserId)>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    match service.validate_tenant_access(&tenant_id, &user_id).await {
        Ok(has_access) => Ok(Json(serde_json::json!({
            "has_access": has_access,
            "tenant_id": tenant_id,
            "user_id": user_id
        }))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": {
                    "code": "ACCESS_VALIDATION_FAILED",
                    "message": e.to_string()
                }
            })),
        )),
    }
}

pub async fn get_user_tenant_permissions(
    State(service): State<TenantServiceState>,
    Path((tenant_id, user_id)): Path<(TenantId, UserId)>,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let permission = params.get("permission").cloned().unwrap_or_default();
    
    if permission.is_empty() {
        // Return all permissions for the user in the tenant
        match service.get_tenant_context(&tenant_id, &user_id).await {
            Ok(context) => Ok(Json(serde_json::json!({
                "tenant_id": tenant_id,
                "user_id": user_id,
                "role": context.user_role,
                "permissions": context.user_permissions
            }))),
            Err(e) => {
                let status = if e.to_string().contains("not found") || e.to_string().contains("does not have access") {
                    StatusCode::FORBIDDEN
                } else {
                    StatusCode::INTERNAL_SERVER_ERROR
                };
                
                Err((
                    status,
                    Json(serde_json::json!({
                        "error": {
                            "code": "PERMISSIONS_FAILED",
                            "message": e.to_string()
                        }
                    })),
                ))
            }
        }
    } else {
        // Check specific permission
        match service.validate_tenant_permission(&tenant_id, &user_id, &permission).await {
            Ok(has_permission) => Ok(Json(serde_json::json!({
                "tenant_id": tenant_id,
                "user_id": user_id,
                "permission": permission,
                "has_permission": has_permission
            }))),
            Err(e) => Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": {
                        "code": "PERMISSION_CHECK_FAILED",
                        "message": e.to_string()
                    }
                })),
            )),
        }
    }
}