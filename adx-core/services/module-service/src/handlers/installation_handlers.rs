use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use std::sync::Arc;

use crate::server::AppState;
use crate::error::ModuleServiceError;
use crate::types::{InstallModuleRequest, InstallModuleResult};
use crate::models::ModuleInstallationResponse;

#[derive(Debug, Deserialize)]
pub struct ListInstallationsQuery {
    pub tenant_id: String,
}

pub async fn list_installations(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListInstallationsQuery>,
) -> Result<Json<Vec<ModuleInstallationResponse>>, ModuleServiceError> {
    let installations = state.installation_repo.list_installations(&query.tenant_id).await?;
    
    let responses: Vec<ModuleInstallationResponse> = installations
        .into_iter()
        .map(|installation| ModuleInstallationResponse {
            installation_id: installation.id,
            module_id: installation.module_id,
            version: installation.version,
            status: serde_json::from_str(&format!("\"{}\"", installation.status)).unwrap_or(crate::types::ModuleStatus::Available),
            configuration: installation.configuration_json.map(|json| {
                serde_json::from_value(json).unwrap_or_default()
            }),
            installed_at: installation.installed_at,
        })
        .collect();

    Ok(Json(responses))
}

pub async fn get_installation(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<Json<ModuleInstallationResponse>, ModuleServiceError> {
    let installation = state.installation_repo.get_installation_by_id(&id)
        .await?
        .ok_or_else(|| ModuleServiceError::ModuleNotFound(id))?;

    let response = ModuleInstallationResponse {
        installation_id: installation.id,
        module_id: installation.module_id,
        version: installation.version,
        status: serde_json::from_str(&format!("\"{}\"", installation.status)).unwrap_or(crate::types::ModuleStatus::Available),
        configuration: installation.configuration_json.map(|json| {
            serde_json::from_value(json).unwrap_or_default()
        }),
        installed_at: installation.installed_at,
    };

    Ok(Json(response))
}

pub async fn install_module(
    State(state): State<Arc<AppState>>,
    Json(request): Json<InstallModuleRequest>,
) -> Result<Json<InstallModuleResult>, ModuleServiceError> {
    let result = state.module_manager.install_module(request).await?;
    Ok(Json(result))
}

pub async fn uninstall_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ModuleServiceError> {
    // Get installation to find module_id and tenant_id
    let installation = state.installation_repo.get_installation_by_id(&id)
        .await?
        .ok_or_else(|| ModuleServiceError::ModuleNotFound(id))?;

    let uninstall_request = crate::types::UninstallModuleRequest {
        module_id: installation.module_id,
        tenant_id: installation.tenant_id,
        cleanup_data: true,
        force_uninstall: false,
    };

    let _result = state.module_manager.uninstall_module(uninstall_request).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn activate_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ModuleServiceError> {
    let installation = state.installation_repo.get_installation_by_id(&id)
        .await?
        .ok_or_else(|| ModuleServiceError::ModuleNotFound(id))?;

    state.module_manager.activate_module(&installation.module_id, &installation.tenant_id).await?;
    Ok(StatusCode::OK)
}

pub async fn deactivate_module(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
) -> Result<StatusCode, ModuleServiceError> {
    let installation = state.installation_repo.get_installation_by_id(&id)
        .await?
        .ok_or_else(|| ModuleServiceError::ModuleNotFound(id))?;

    state.module_manager.deactivate_module(&installation.module_id, &installation.tenant_id).await?;
    Ok(StatusCode::OK)
}