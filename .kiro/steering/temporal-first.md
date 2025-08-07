# Temporal-First Backend Architecture

## Core Principles

ADX CORE follows a "Temporal-first" architecture where Temporal workflows are the PRIMARY mechanism for implementing all multi-step business operations. This approach provides:

- **Reliability**: Automatic retry, timeout, and error handling for distributed operations
- **Observability**: Complete visibility into business process execution through Temporal UI
- **Maintainability**: Clear separation between business logic (workflows) and infrastructure concerns
- **Scalability**: Horizontal scaling of workflow workers independent of HTTP services

## Architecture Decision Rules

### When to Use Workflows vs Direct Endpoints

**Use Temporal Workflows for:**
- Operations involving multiple microservices
- Long-running operations (>5 seconds expected duration)
- Operations requiring rollback/compensation logic
- Complex business processes with multiple steps
- Operations that need progress tracking
- Operations with complex retry/timeout requirements

**Use Direct Endpoints for:**
- Single-service CRUD operations
- Simple data retrieval
- Operations that complete in <1 second
- Health checks and status endpoints
- Static data serving

### Service Implementation Pattern

Each backend service MUST implement a dual-mode pattern:

```rust
// main.rs - Service entry point with mode selection
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let mode = args.get(1).map(|s| s.as_str()).unwrap_or("server");
    
    match mode {
        "server" => {
            // Start HTTP server for direct endpoints
            start_http_server().await
        }
        "worker" => {
            // Start Temporal workflow worker
            start_workflow_worker().await
        }
        _ => {
            eprintln!("Usage: {} [server|worker]", args[0]);
            std::process::exit(1);
        }
    }
}
```

**HTTP Server Mode:**
- Handles direct endpoint requests
- Provides simple CRUD operations
- Routes complex operations to workflow initiation
- Maintains backward compatibility

**Workflow Worker Mode:**
- Executes Temporal activities and workflows
- Implements business logic as activities
- Provides compensation logic for rollbacks
- Scales independently based on workflow load

## Workflow Design Patterns

### Activity Implementation Pattern

```rust
#[async_trait]
pub trait ServiceActivities {
    // Core business operations as activities
    async fn create_entity(&self, request: CreateEntityRequest) -> Result<Entity, ActivityError>;
    async fn update_entity(&self, id: &str, updates: EntityUpdates) -> Result<Entity, ActivityError>;
    async fn delete_entity(&self, id: &str) -> Result<(), ActivityError>;
    
    // Compensation activities for rollbacks
    async fn rollback_entity_creation(&self, entity_id: &str) -> Result<(), ActivityError>;
    async fn restore_entity_state(&self, entity_id: &str, previous_state: EntityState) -> Result<(), ActivityError>;
}

// Activity error handling with retry policies
#[derive(Debug, Serialize, Deserialize)]
pub enum ActivityError {
    DatabaseError(String),      // Retryable
    NetworkError(String),       // Retryable
    ValidationError(String),    // Non-retryable
    AuthorizationError(String), // Non-retryable
    ExternalServiceError(String), // Retryable with circuit breaker
}

impl ActivityError {
    pub fn retry_policy(&self) -> RetryPolicy {
        match self {
            ActivityError::DatabaseError(_) => RetryPolicy::exponential_backoff(3, Duration::seconds(1)),
            ActivityError::NetworkError(_) => RetryPolicy::exponential_backoff(5, Duration::seconds(2)),
            ActivityError::ExternalServiceError(_) => RetryPolicy::exponential_backoff(3, Duration::seconds(5)),
            _ => RetryPolicy::no_retry(),
        }
    }
}
```

### Workflow Implementation Pattern

```rust
#[workflow]
pub async fn business_process_workflow(
    request: BusinessProcessRequest,
    context: WorkflowContext,
) -> Result<BusinessProcessResult, WorkflowError> {
    // Step 1: Validate preconditions
    let validation = call_activity(
        ServiceActivities::validate_request,
        ValidationRequest {
            request: request.clone(),
            user_context: context.user_context.clone(),
            tenant_context: context.tenant_context.clone(),
        },
    ).await?;
    
    if !validation.is_valid {
        return Err(WorkflowError::ValidationFailed(validation.errors));
    }
    
    // Step 2: Execute main business logic with compensation tracking
    let mut compensation_activities = Vec::new();
    
    let entity = call_activity(
        ServiceActivities::create_entity,
        CreateEntityRequest::from(request.clone()),
    ).await.map_err(|e| {
        WorkflowError::ActivityFailed("create_entity".to_string(), e)
    })?;
    
    // Track compensation for rollback
    compensation_activities.push(CompensationActivity::new(
        "rollback_entity_creation",
        entity.id.clone(),
    ));
    
    // Step 3: Execute dependent operations
    let related_data = call_activity(
        RelatedServiceActivities::create_related_data,
        CreateRelatedDataRequest {
            entity_id: entity.id.clone(),
            data: request.related_data,
        },
    ).await.map_err(|e| {
        // Execute compensation for previous steps
        spawn_compensation_workflow(compensation_activities);
        WorkflowError::ActivityFailed("create_related_data".to_string(), e)
    })?;
    
    // Step 4: Finalize and return result
    Ok(BusinessProcessResult {
        entity_id: entity.id,
        related_data_id: related_data.id,
        completed_at: Utc::now(),
    })
}
```

### Cross-Service Workflow Pattern

```rust
// Cross-service workflows coordinate multiple services
#[workflow]
pub async fn tenant_switch_workflow(
    user_id: String,
    from_tenant_id: String,
    to_tenant_id: String,
) -> Result<TenantSwitchResult, WorkflowError> {
    // Coordinate activities across multiple services
    let auth_result = call_activity(
        AuthServiceActivities::validate_tenant_access,
        ValidateTenantAccessRequest {
            user_id: user_id.clone(),
            tenant_id: to_tenant_id.clone(),
        },
    ).await?;
    
    let user_result = call_activity(
        UserServiceActivities::update_user_tenant,
        UpdateUserTenantRequest {
            user_id: user_id.clone(),
            new_tenant_id: to_tenant_id.clone(),
        },
    ).await?;
    
    let session_result = call_activity(
        AuthServiceActivities::create_tenant_session,
        CreateTenantSessionRequest {
            user_id: user_id.clone(),
            tenant_id: to_tenant_id.clone(),
        },
    ).await?;
    
    Ok(TenantSwitchResult {
        success: true,
        new_session_id: session_result.session_id,
        user_context: user_result.updated_user,
    })
}
```

## API Gateway Integration

The API Gateway serves as the entry point for both direct operations and workflow initiation:

```rust
// API Gateway workflow endpoints
impl ApiGateway {
    pub async fn handle_workflow_request(
        &self,
        path: &str,
        request: WorkflowRequest,
    ) -> Result<WorkflowResponse, ApiError> {
        let workflow_type = self.determine_workflow_type(path)?;
        
        // Start workflow execution
        let workflow_id = format!("{}-{}-{}", 
            workflow_type, 
            request.user_context.user_id, 
            Uuid::new_v4()
        );
        
        let handle = self.temporal_client
            .start_workflow(
                workflow_type,
                workflow_id.clone(),
                self.get_task_queue(&workflow_type),
                request,
            )
            .await?;
        
        // For quick workflows, wait for completion
        if self.is_synchronous_workflow(&workflow_type) {
            let result = handle.get_result().await?;
            return Ok(WorkflowResponse::Completed(result));
        }
        
        // For long-running workflows, return operation ID
        Ok(WorkflowResponse::Started {
            operation_id: workflow_id,
            status_url: format!("/api/workflows/{}/status", workflow_id),
            stream_url: format!("/api/workflows/{}/stream", workflow_id),
        })
    }
    
    pub async fn get_workflow_status(
        &self,
        operation_id: &str,
    ) -> Result<WorkflowStatusResponse, ApiError> {
        let execution_info = self.temporal_client
            .get_workflow_execution_info(operation_id)
            .await?;
        
        Ok(WorkflowStatusResponse {
            operation_id: operation_id.to_string(),
            status: execution_info.status,
            progress: execution_info.progress,
            result: execution_info.result,
            error: execution_info.error,
        })
    }
}
```

## BFF Integration Strategy

BFF services act as Temporal workflow clients for frontend optimization:

```rust
// BFF as workflow client pattern
pub struct ServiceBFF {
    temporal_client: Arc<TemporalClient>,
    api_gateway_client: Arc<ApiGatewayClient>,
    cache: Arc<RedisClient>,
}

impl ServiceBFF {
    // Aggregate multiple workflows for complex UI needs
    pub async fn get_dashboard_data(
        &self,
        user_id: &str,
    ) -> Result<DashboardData, BFFError> {
        // Check cache first
        if let Some(cached) = self.get_cached_data(user_id).await? {
            return Ok(cached);
        }
        
        // Start multiple workflows in parallel
        let workflows = vec![
            self.start_workflow("get_user_profile", user_id),
            self.start_workflow("get_recent_activity", user_id),
            self.start_workflow("get_tenant_data", user_id),
        ];
        
        let results = futures::future::try_join_all(workflows).await?;
        
        let dashboard_data = DashboardData::from_workflow_results(results);
        
        // Cache the aggregated result
        self.cache_data(user_id, &dashboard_data).await?;
        
        Ok(dashboard_data)
    }
    
    // Direct workflow initiation from BFF
    pub async fn initiate_complex_operation(
        &self,
        operation_request: ComplexOperationRequest,
    ) -> Result<OperationResponse, BFFError> {
        let workflow_id = format!("complex-op-{}-{}", 
            operation_request.user_id, 
            Uuid::new_v4()
        );
        
        let handle = self.temporal_client
            .start_workflow(
                "complex_operation_workflow",
                workflow_id.clone(),
                "complex-operations-queue",
                operation_request,
            )
            .await?;
        
        Ok(OperationResponse::InProgress {
            operation_id: workflow_id,
            estimated_duration: Duration::minutes(5),
            progress_url: format!("/api/operations/{}/progress", workflow_id),
        })
    }
}
```

## Development and Testing Patterns

### Workflow Testing

```rust
#[cfg(test)]
mod workflow_tests {
    use super::*;
    use temporal_sdk_core_test_utils::TestWorkflowEnvironment;
    
    #[tokio::test]
    async fn test_business_process_workflow() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock activities
        let mock_activities = MockServiceActivities::new()
            .expect_validate_request()
            .returning(|_| Ok(ValidationResult { is_valid: true, errors: vec![] }))
            .expect_create_entity()
            .returning(|_| Ok(Entity::default()));
        
        // Execute workflow
        let result = test_env.execute_workflow(
            business_process_workflow,
            BusinessProcessRequest::default(),
        ).await;
        
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_workflow_compensation() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock failure scenario
        let mock_activities = MockServiceActivities::new()
            .expect_create_entity()
            .returning(|_| Ok(Entity::default()))
            .expect_create_related_data()
            .returning(|_| Err(ActivityError::ExternalServiceError("Service down".to_string())))
            .expect_rollback_entity_creation()
            .returning(|_| Ok(()));
        
        let result = test_env.execute_workflow(
            business_process_workflow,
            BusinessProcessRequest::default(),
        ).await;
        
        assert!(result.is_err());
        // Verify compensation was executed
        mock_activities.verify_rollback_entity_creation_called();
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_end_to_end_workflow_execution() {
    let test_env = IntegrationTestEnvironment::new().await;
    
    // Test complete workflow execution through API Gateway
    let response = test_env.api_client
        .post("/api/workflows/business-process")
        .json(&BusinessProcessRequest::default())
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let workflow_response: WorkflowResponse = response.json().await.unwrap();
    
    match workflow_response {
        WorkflowResponse::Started { operation_id, .. } => {
            // Poll for completion
            let final_result = test_env.poll_workflow_completion(&operation_id).await;
            assert!(final_result.is_ok());
        }
        WorkflowResponse::Completed(result) => {
            assert!(result.is_ok());
        }
    }
}
```

## Monitoring and Observability

### Workflow Metrics

```rust
// Prometheus metrics for workflows
lazy_static! {
    static ref WORKFLOW_EXECUTIONS: Counter = Counter::new(
        "workflow_executions_total",
        "Total workflow executions by type and status"
    ).unwrap();
    
    static ref WORKFLOW_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("workflow_duration_seconds", "Workflow execution duration")
            .buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0])
    ).unwrap();
    
    static ref ACTIVITY_RETRIES: Counter = Counter::new(
        "activity_retries_total",
        "Total activity retries by activity type"
    ).unwrap();
}
```

### Structured Logging

```rust
// Workflow context logging
pub fn log_workflow_execution(
    workflow_id: &str,
    workflow_type: &str,
    user_context: &UserContext,
    event: WorkflowEvent,
) {
    info!(
        workflow_id = %workflow_id,
        workflow_type = %workflow_type,
        user_id = %user_context.user_id,
        tenant_id = %user_context.tenant_id,
        event = ?event,
        "Workflow event"
    );
}
```

This Temporal-first approach ensures that all complex business operations are reliable, observable, and maintainable while providing the flexibility to optimize with BFF services where needed.