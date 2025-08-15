use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

use crate::server::AppState;
use crate::error::ModuleServiceError;
use crate::types::{Module, ModuleSearchRequest, ModuleSearchResponse};
use crate::models::{CreateModuleRequest, UpdateModuleRequest, ModuleListResponse};

#[derive(Debug, Deserialize)]
pub struct ListModulesQuery {
    pub tenant_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub category: Option<String>,
    pub status: Option<String>,
}

pub async fn list_modules(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListModulesQuery>,
) -> Result<Json<ModuleListResponse>, ModuleServiceError> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(20).min(100); // Max 100 per page

    let modules = state.module_repo.list_modules(
        query.tenant_id.as_deref(),
        page,
        page_size,
    ).await?;

    // Get total count for pagination
    let total_count = modules.len() as u64; // Simplified - would be a separate query

    Ok(Json(ModuleListResponse {
        modules,
        total_count,
        page,
        page_size,
    }))
}

pub async fn get_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<Module>, ModuleServiceError> {
    let module = state.module_repo.get_module_by_id(&id)
        .await?
        .ok_or_else(|| ModuleServiceError::ModuleNotFound(id))?;

    Ok(Json(module))
}

pub async fn create_module(
    State(state): State<Arc<AppState>>,
    Json(request): Json<CreateModuleRequest>,
) -> Result<Json<Module>, ModuleServiceError> {
    // Validate request
    // request.validate().map_err(|e| ModuleServiceError::ModuleValidationError(format!("{:?}", e)))?;

    // Create module from request
    let module = Module {
        id: uuid::Uuid::new_v4().to_string(),
        name: request.name,
        version: "1.0.0".to_string(), // Would come from manifest
        description: request.description,
        author: request.author,
        category: request.category,
        manifest: request.manifest,
        status: crate::types::ModuleStatus::Available,
        tenant_id: None, // Global module
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let created_module = state.module_repo.create_module(&module).await?;

    Ok(Json(created_module))
}

pub async fn update_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(request): Json<UpdateModuleRequest>,
) -> Result<Json<Module>, ModuleServiceError> {
    // Get existing module
    let mut module = state.module_repo.get_module_by_id(&id)
        .await?
        .ok_or_else(|| ModuleServiceError::ModuleNotFound(id.clone()))?;

    // Update fields
    if let Some(description) = request.description {
        module.description = description;
    }
    if let Some(category) = request.category {
        module.category = category;
    }
    if let Some(manifest) = request.manifest {
        module.manifest = manifest;
    }

    module.updated_at = chrono::Utc::now();

    let updated_module = state.module_repo.update_module(&id, &module).await?;

    Ok(Json(updated_module))
}

pub async fn delete_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ModuleServiceError> {
    // Check if module exists
    let _module = state.module_repo.get_module_by_id(&id)
        .await?
        .ok_or_else(|| ModuleServiceError::ModuleNotFound(id.clone()))?;

    // Check if module has active installations
    // This would be a more complex check in production

    state.module_repo.delete_module(&id).await?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn search_modules(
    State(state): State<Arc<AppState>>,
    Json(request): Json<ModuleSearchRequest>,
) -> Result<Json<ModuleSearchResponse>, ModuleServiceError> {
    // Validate request
    if request.page_size > 100 {
        return Err(ModuleServiceError::ModuleValidationError(
            "Page size cannot exceed 100".to_string()
        ));
    }

    let response = state.module_repo.search_modules(&request).await?;

    Ok(Json(response))
}