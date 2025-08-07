use adx_shared::{init_tracing, RequestContext, StandardWorkflowInput, WorkflowContext};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

mod workflows;
use workflows::*;

#[derive(Clone)]
pub struct AppState {
    // Add Temporal client and other state here
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub steps: Vec<WorkflowStep>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: Uuid,
    pub name: String,
    pub step_type: String,
    pub config: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub status: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartWorkflowRequest {
    pub workflow_id: Uuid,
    pub input_data: serde_json::Value,
}

#[tokio::main]
async fn main() {
    init_tracing();

    let app_state = AppState {};

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/v1/workflows", get(list_workflows))
        .route("/api/v1/workflows", post(create_workflow))
        .route("/api/v1/workflows/:workflow_id", get(get_workflow))
        .route(
            "/api/v1/workflows/:workflow_id/execute",
            post(start_workflow),
        )
        .route(
            "/api/v1/executions/:execution_id",
            get(get_execution_status),
        )
        .route("/api/v1/processes/execute", post(execute_business_process))
        .route("/api/v1/approvals/create", post(create_approval_request))
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8084").await.unwrap();
    tracing::info!(
        "Workflow Service listening on {}",
        listener.local_addr().unwrap()
    );
    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "Workflow Service OK"
}

async fn list_workflows(
    State(_state): State<AppState>,
) -> Result<Json<Vec<WorkflowDefinition>>, StatusCode> {
    // TODO: Query database for workflows
    tracing::info!("Listing workflows");

    // Mock response
    let workflows = vec![WorkflowDefinition {
        id: Uuid::new_v4(),
        name: "Data Processing Pipeline".to_string(),
        description: "Process uploaded files and extract insights".to_string(),
        steps: vec![],
        created_at: Utc::now(),
    }];

    Ok(Json(workflows))
}

async fn create_workflow(
    State(_state): State<AppState>,
    Json(workflow): Json<WorkflowDefinition>,
) -> Result<Json<WorkflowDefinition>, StatusCode> {
    // TODO: Save workflow to database
    // TODO: Register with Temporal

    tracing::info!("Creating workflow: {}", workflow.name);
    Ok(Json(workflow))
}

async fn get_workflow(
    State(_state): State<AppState>,
    Path(workflow_id): Path<Uuid>,
) -> Result<Json<WorkflowDefinition>, StatusCode> {
    // TODO: Query database for specific workflow

    tracing::info!("Getting workflow: {}", workflow_id);

    // Mock response
    let workflow = WorkflowDefinition {
        id: workflow_id,
        name: "Example Workflow".to_string(),
        description: "An example workflow".to_string(),
        steps: vec![],
        created_at: Utc::now(),
    };

    Ok(Json(workflow))
}

async fn start_workflow(
    State(_state): State<AppState>,
    Path(workflow_id): Path<Uuid>,
    Json(request): Json<StartWorkflowRequest>,
) -> Result<Json<WorkflowExecution>, StatusCode> {
    // TODO: Start workflow execution with Temporal

    tracing::info!("Starting workflow execution: {}", workflow_id);

    // Mock response
    let execution = WorkflowExecution {
        id: Uuid::new_v4(),
        workflow_id,
        status: "running".to_string(),
        started_at: Utc::now(),
        completed_at: None,
    };

    Ok(Json(execution))
}

async fn get_execution_status(
    State(_state): State<AppState>,
    Path(execution_id): Path<Uuid>,
) -> Result<Json<WorkflowExecution>, StatusCode> {
    // TODO: Query Temporal for execution status

    tracing::info!("Getting execution status: {}", execution_id);

    // Mock response
    let execution = WorkflowExecution {
        id: execution_id,
        workflow_id: Uuid::new_v4(),
        status: "completed".to_string(),
        started_at: Utc::now(),
        completed_at: Some(Utc::now()),
    };

    Ok(Json(execution))
}

// Execute business process using Temporal workflow
pub async fn execute_business_process(
    State(_state): State<AppState>,
    Json(request): Json<BusinessProcessRequest>,
) -> Result<Json<BusinessProcessResponse>, StatusCode> {
    tracing::info!("Executing business process: {}", request.process_name);

    // Create workflow context (in real implementation, extract from auth headers)
    let context = WorkflowContext::new(
        request.tenant_id.unwrap_or_else(|| Uuid::new_v4()),
        request.user_id,
    );

    // Create workflow input following Temporal-First Principle
    let process_data = BusinessProcessData {
        process_name: request.process_name.clone(),
        process_type: request.process_type,
        input_data: request.input_data,
        configuration: request.configuration,
        notifications: request.notifications,
    };

    let workflow_input = StandardWorkflowInput {
        context,
        data: process_data,
    };

    // Execute Temporal workflow for business process
    match business_process_workflow(workflow_input).await {
        Ok(result) => {
            tracing::info!(
                "Business process workflow completed: {:?}",
                result.result.process_id
            );
            Ok(Json(BusinessProcessResponse {
                process_id: result.result.process_id,
                status: format!("{:?}", result.result.execution_status),
                steps_completed: result.result.steps_completed,
                total_steps: result.result.total_steps,
                message: format!("Process '{}' executed successfully", request.process_name),
            }))
        }
        Err(error) => {
            tracing::error!("Business process workflow failed: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Create approval request using Temporal workflow
pub async fn create_approval_request(
    State(_state): State<AppState>,
    Json(request): Json<ApprovalRequestRequest>,
) -> Result<Json<ApprovalRequestResponse>, StatusCode> {
    tracing::info!("Creating approval request: {}", request.request_type);

    // Create workflow context (in real implementation, extract from auth headers)
    let context = WorkflowContext::new(
        request.tenant_id.unwrap_or_else(|| Uuid::new_v4()),
        request.user_id,
    );

    // Create workflow input following Temporal-First Principle
    let approval_data = ApprovalRequestData {
        request_type: request.request_type.clone(),
        request_data: request.request_data,
        approvers: request.approvers,
        timeout_hours: request.timeout_hours,
        escalation_enabled: request.escalation_enabled,
    };

    let workflow_input = StandardWorkflowInput {
        context,
        data: approval_data,
    };

    // Execute Temporal workflow for approval
    match approval_workflow(workflow_input).await {
        Ok(result) => {
            tracing::info!(
                "Approval workflow completed: {:?}",
                result.result.approval_id
            );
            Ok(Json(ApprovalRequestResponse {
                approval_id: result.result.approval_id,
                status: format!("{:?}", result.result.final_status),
                message: format!(
                    "Approval request '{}' created successfully",
                    request.request_type
                ),
            }))
        }
        Err(error) => {
            tracing::error!("Approval workflow failed: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Request/Response types for new endpoints
#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessProcessRequest {
    pub process_name: String,
    pub process_type: ProcessType,
    pub input_data: serde_json::Value,
    pub configuration: ProcessConfiguration,
    pub notifications: Vec<NotificationConfig>,
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BusinessProcessResponse {
    pub process_id: Uuid,
    pub status: String,
    pub steps_completed: i32,
    pub total_steps: i32,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalRequestRequest {
    pub request_type: String,
    pub request_data: serde_json::Value,
    pub approvers: Vec<ApprovalLevel>,
    pub timeout_hours: i32,
    pub escalation_enabled: bool,
    pub tenant_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalRequestResponse {
    pub approval_id: Uuid,
    pub status: String,
    pub message: String,
}
