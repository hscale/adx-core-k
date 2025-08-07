# ADX CORE Integration Specifications

## Integration Architecture Overview

This document defines the comprehensive integration patterns, protocols, and standards for all inter-module communication within ADX CORE, ensuring seamless system coherence and reliable data flow.

## Integration Patterns

### 1. Synchronous Integration (Request-Response)

#### REST API Standards
```yaml
api_standards:
  versioning:
    strategy: "URL path versioning"
    format: "/api/v{major}/{resource}"
    example: "/api/v1/users"
  
  headers:
    required:
      - "Authorization: Bearer {jwt_token}"
      - "X-Tenant-ID: {tenant_uuid}"
      - "X-Correlation-ID: {correlation_uuid}"
    optional:
      - "X-User-ID: {user_uuid}"
      - "X-Request-ID: {request_uuid}"
  
  response_format:
    success:
      status_codes: [200, 201, 202, 204]
      body_structure:
        data: "actual response data"
        metadata:
          correlation_id: "uuid"
          timestamp: "iso8601"
          version: "api version"
    
    error:
      status_codes: [400, 401, 403, 404, 409, 422, 429, 500, 502, 503]
      body_structure:
        error:
          code: "ERROR_CODE"
          message: "Human readable message"
          details: "Additional context"
          correlation_id: "uuid"
          timestamp: "iso8601"
```

#### API Contract Example
```rust
// Unified API response structure
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub version: String,
    pub tenant_id: TenantId,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub correlation_id: Uuid,
    pub timestamp: DateTime<Utc>,
}

// Service-to-service API client
pub struct ServiceClient {
    base_url: String,
    service_name: String,
    http_client: reqwest::Client,
    auth_provider: Arc<dyn ServiceAuthProvider>,
}

impl ServiceClient {
    pub async fn call<Req, Resp>(
        &self,
        method: Method,
        path: &str,
        request: Option<&Req>,
        context: &RequestContext,
    ) -> Result<ApiResponse<Resp>, ApiError>
    where
        Req: Serialize,
        Resp: DeserializeOwned,
    {
        let token = self.auth_provider.get_service_token().await?;
        
        let mut request_builder = self.http_client
            .request(method, &format!("{}{}", self.base_url, path))
            .header("Authorization", format!("Bearer {}", token))
            .header("X-Tenant-ID", context.tenant_id.to_string())
            .header("X-Correlation-ID", context.correlation_id.to_string())
            .header("X-Service-Name", &self.service_name)
            .timeout(Duration::from_secs(30));
        
        if let Some(body) = request {
            request_builder = request_builder.json(body);
        }
        
        let response = request_builder.send().await?;
        
        if response.status().is_success() {
            let api_response: ApiResponse<Resp> = response.json().await?;
            Ok(api_response)
        } else {
            let error: ApiError = response.json().await?;
            Err(error)
        }
    }
}
```

### 2. Asynchronous Integration (Event-Driven)

#### Event Schema Standards
```rust
// Unified event envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope {
    pub id: Uuid,
    pub event_type: String,
    pub event_version: String,
    pub source_service: String,
    pub tenant_id: Option<TenantId>,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub sequence_number: i64,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub metadata: EventMetadata,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub user_id: Option<Uuid>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub trace_id: String,
    pub span_id: String,
}

// Event type registry
pub struct EventTypeRegistry {
    types: HashMap<String, EventTypeDefinition>,
}

#[derive(Debug, Clone)]
pub struct EventTypeDefinition {
    pub event_type: String,
    pub version: String,
    pub schema: serde_json::Value,
    pub source_service: String,
    pub description: String,
}
```

#### Event Bus Implementation
```rust
// Unified event bus interface
#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: EventEnvelope) -> Result<(), EventBusError>;
    async fn publish_batch(&self, events: Vec<EventEnvelope>) -> Result<(), EventBusError>;
    async fn subscribe(&self, subscription: EventSubscription) -> Result<(), EventBusError>;
    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError>;
}

#[derive(Debug, Clone)]
pub struct EventSubscription {
    pub id: String,
    pub service_name: String,
    pub event_types: Vec<String>,
    pub filter: Option<EventFilter>,
    pub handler: Arc<dyn EventHandler>,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone)]
pub struct EventFilter {
    pub tenant_ids: Option<Vec<TenantId>>,
    pub aggregate_types: Option<Vec<String>>,
    pub custom_filters: HashMap<String, serde_json::Value>,
}

// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &EventEnvelope) -> Result<(), EventHandlerError>;
    fn can_handle(&self, event_type: &str) -> bool;
}
```

### 3. Workflow Integration (Temporal)

#### Workflow Communication Patterns
```rust
// Inter-service workflow communication
#[workflow]
pub async fn cross_service_workflow(
    input: CrossServiceWorkflowInput,
) -> WorkflowResult<CrossServiceWorkflowOutput> {
    // 1. Call external service activity
    let external_result = call_external_service_activity(
        input.service_name.clone(),
        input.operation.clone(),
        input.payload.clone(),
    ).await?;
    
    // 2. Process result locally
    let processed_result = process_external_result_activity(
        external_result,
        input.processing_options.clone(),
    ).await?;
    
    // 3. Notify other services via event
    publish_workflow_event_activity(WorkflowEvent {
        workflow_id: workflow_id(),
        event_type: "cross_service_workflow.completed".to_string(),
        tenant_id: input.tenant_id,
        result: processed_result.clone(),
    }).await?;
    
    Ok(CrossServiceWorkflowOutput {
        result: processed_result,
        external_service_response: external_result,
    })
}

// External service activity
#[activity]
pub async fn call_external_service_activity(
    service_name: String,
    operation: String,
    payload: serde_json::Value,
) -> Result<serde_json::Value, ActivityError> {
    let service_client = get_service_client(&service_name).await?;
    
    let response = service_client
        .call(
            Method::POST,
            &format!("/api/v1/{}", operation),
            Some(&payload),
            &get_current_context(),
        )
        .await
        .map_err(|e| ActivityError::ExternalServiceError(e.to_string()))?;
    
    Ok(response.data)
}
```

#### Workflow Event Integration
```rust
// Workflow-triggered events
pub struct WorkflowEventPublisher {
    event_bus: Arc<dyn EventBus>,
    workflow_context: WorkflowContext,
}

impl WorkflowEventPublisher {
    pub async fn publish_workflow_event(
        &self,
        event_type: &str,
        payload: serde_json::Value,
    ) -> Result<(), WorkflowError> {
        let event = EventEnvelope {
            id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            event_version: "1.0.0".to_string(),
            source_service: "workflow-service".to_string(),
            tenant_id: self.workflow_context.tenant_id,
            aggregate_id: self.workflow_context.workflow_id,
            aggregate_type: "workflow".to_string(),
            sequence_number: self.workflow_context.sequence_number,
            timestamp: Utc::now(),
            correlation_id: self.workflow_context.correlation_id,
            causation_id: Some(self.workflow_context.workflow_id),
            metadata: EventMetadata {
                user_id: self.workflow_context.user_id,
                session_id: None,
                ip_address: None,
                user_agent: None,
                trace_id: self.workflow_context.trace_id.clone(),
                span_id: self.workflow_context.span_id.clone(),
            },
            payload,
        };
        
        self.event_bus.publish(event).await
            .map_err(WorkflowError::from)
    }
}
```

## Service Integration Contracts

### User Service Integration
```yaml
user_service:
  provides:
    apis:
      - path: "/api/v1/users"
        methods: [GET, POST, PUT, DELETE]
        description: "User CRUD operations"
      - path: "/api/v1/users/{id}/profile"
        methods: [GET, PUT]
        description: "User profile management"
    
    events:
      - type: "user.created"
        version: "1.0.0"
        schema:
          user_id: "uuid"
          tenant_id: "uuid"
          email: "string"
          profile: "object"
      - type: "user.updated"
        version: "1.0.0"
        schema:
          user_id: "uuid"
          tenant_id: "uuid"
          changes: "object"
  
  consumes:
    events:
      - type: "tenant.created"
        handler: "create_default_admin_user"
      - type: "auth.login_failed"
        handler: "track_failed_login_attempts"
```

### File Service Integration
```yaml
file_service:
  provides:
    apis:
      - path: "/api/v1/files"
        methods: [GET, POST]
        description: "File upload and listing"
      - path: "/api/v1/files/{id}"
        methods: [GET, PUT, DELETE]
        description: "File operations"
      - path: "/api/v1/files/{id}/share"
        methods: [POST, DELETE]
        description: "File sharing"
    
    events:
      - type: "file.uploaded"
        version: "1.0.0"
        schema:
          file_id: "uuid"
          tenant_id: "uuid"
          user_id: "uuid"
          filename: "string"
          size: "integer"
          mime_type: "string"
      - type: "file.shared"
        version: "1.0.0"
        schema:
          file_id: "uuid"
          tenant_id: "uuid"
          shared_with: "array"
          permissions: "object"
  
  consumes:
    events:
      - type: "user.deleted"
        handler: "cleanup_user_files"
      - type: "tenant.deleted"
        handler: "cleanup_tenant_files"
```

### Workflow Service Integration
```yaml
workflow_service:
  provides:
    apis:
      - path: "/api/v1/workflows"
        methods: [GET, POST]
        description: "Workflow management"
      - path: "/api/v1/workflows/{id}/execute"
        methods: [POST]
        description: "Workflow execution"
      - path: "/api/v1/workflows/{id}/status"
        methods: [GET]
        description: "Workflow status"
    
    events:
      - type: "workflow.started"
        version: "1.0.0"
        schema:
          workflow_id: "uuid"
          tenant_id: "uuid"
          workflow_type: "string"
          input: "object"
      - type: "workflow.completed"
        version: "1.0.0"
        schema:
          workflow_id: "uuid"
          tenant_id: "uuid"
          result: "object"
          duration: "integer"
  
  consumes:
    events:
      - type: "file.uploaded"
        handler: "trigger_file_processing_workflow"
      - type: "user.created"
        handler: "trigger_user_onboarding_workflow"
```

## Data Integration Patterns

### Shared Data Models
```rust
// Common data structures across services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: TenantId,
    pub tenant_name: String,
    pub settings: TenantSettings,
    pub limits: TenantLimits,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserContext {
    pub user_id: Uuid,
    pub tenant_id: TenantId,
    pub email: String,
    pub roles: Vec<String>,
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub correlation_id: Uuid,
    pub tenant_id: TenantId,
    pub user_id: Option<Uuid>,
    pub trace_id: String,
    pub span_id: String,
    pub timestamp: DateTime<Utc>,
}
```

### Data Consistency Patterns
```rust
// Eventual consistency with compensation
pub struct SagaOrchestrator {
    steps: Vec<SagaStep>,
    compensation_steps: Vec<CompensationStep>,
}

#[async_trait]
pub trait SagaStep: Send + Sync {
    async fn execute(&self, context: &SagaContext) -> Result<SagaStepResult, SagaError>;
    async fn compensate(&self, context: &SagaContext) -> Result<(), SagaError>;
}

// Example: User creation saga
pub struct CreateUserSaga {
    user_service: Arc<dyn UserService>,
    auth_service: Arc<dyn AuthService>,
    notification_service: Arc<dyn NotificationService>,
}

impl CreateUserSaga {
    pub async fn execute(&self, input: CreateUserInput) -> Result<CreateUserOutput, SagaError> {
        let mut saga_context = SagaContext::new(input.correlation_id);
        
        // Step 1: Create user record
        let user = self.user_service
            .create_user(input.user_data.clone())
            .await
            .map_err(|e| SagaError::StepFailed("create_user".to_string(), e.into()))?;
        saga_context.add_compensation(Box::new(DeleteUserCompensation { user_id: user.id }));
        
        // Step 2: Create auth credentials
        let auth_result = self.auth_service
            .create_credentials(user.id, input.credentials.clone())
            .await
            .map_err(|e| {
                // Compensate previous steps
                self.compensate_saga(&saga_context).await;
                SagaError::StepFailed("create_credentials".to_string(), e.into())
            })?;
        saga_context.add_compensation(Box::new(DeleteCredentialsCompensation { user_id: user.id }));
        
        // Step 3: Send welcome notification
        self.notification_service
            .send_welcome_email(user.email.clone())
            .await
            .map_err(|e| {
                // Compensate previous steps
                self.compensate_saga(&saga_context).await;
                SagaError::StepFailed("send_welcome_email".to_string(), e.into())
            })?;
        
        Ok(CreateUserOutput {
            user_id: user.id,
            auth_token: auth_result.token,
        })
    }
}
```

## Integration Testing Framework

### Contract Testing
```rust
// API contract testing
#[cfg(test)]
mod contract_tests {
    use super::*;
    use wiremock::{MockServer, Mock, ResponseTemplate};
    
    #[tokio::test]
    async fn test_user_service_contract() {
        let mock_server = MockServer::start().await;
        
        // Setup mock responses
        Mock::given(method("GET"))
            .and(path("/api/v1/users/123"))
            .respond_with(ResponseTemplate::new(200)
                .set_body_json(json!({
                    "data": {
                        "id": "123",
                        "email": "test@example.com",
                        "tenant_id": "tenant-123"
                    },
                    "metadata": {
                        "correlation_id": "corr-123",
                        "timestamp": "2024-01-01T00:00:00Z",
                        "version": "1.0.0"
                    }
                })))
            .mount(&mock_server)
            .await;
        
        // Test service client
        let client = ServiceClient::new(&mock_server.uri(), "test-service");
        let response = client.get_user("123").await.unwrap();
        
        assert_eq!(response.data.id, "123");
        assert_eq!(response.data.email, "test@example.com");
    }
}
```

### Event Integration Testing
```rust
// Event flow testing
#[cfg(test)]
mod event_integration_tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_creation_event_flow() {
        let event_bus = InMemoryEventBus::new();
        let user_service = UserService::new(event_bus.clone());
        let notification_service = NotificationService::new(event_bus.clone());
        
        // Subscribe to user.created events
        let notification_handler = WelcomeEmailHandler::new();
        event_bus.subscribe(EventSubscription {
            id: "welcome-email".to_string(),
            service_name: "notification-service".to_string(),
            event_types: vec!["user.created".to_string()],
            filter: None,
            handler: Arc::new(notification_handler),
            retry_policy: RetryPolicy::default(),
        }).await.unwrap();
        
        // Create user (should trigger event)
        let user = user_service.create_user(CreateUserRequest {
            email: "test@example.com".to_string(),
            tenant_id: TenantId::new_v4(),
        }).await.unwrap();
        
        // Wait for event processing
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        // Verify welcome email was sent
        let sent_emails = notification_service.get_sent_emails().await;
        assert_eq!(sent_emails.len(), 1);
        assert_eq!(sent_emails[0].recipient, "test@example.com");
    }
}
```

## Integration Monitoring

### Health Checks
```rust
// Service health check interface
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check(&self) -> HealthStatus;
    fn name(&self) -> &str;
}

pub struct IntegrationHealthChecker {
    checks: Vec<Box<dyn HealthCheck>>,
}

impl IntegrationHealthChecker {
    pub async fn check_all(&self) -> SystemHealthStatus {
        let mut results = Vec::new();
        
        for check in &self.checks {
            let result = check.check().await;
            results.push((check.name().to_string(), result));
        }
        
        SystemHealthStatus {
            overall: if results.iter().all(|(_, status)| status.is_healthy()) {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            },
            services: results.into_iter().collect(),
            timestamp: Utc::now(),
        }
    }
}

// Example: Database connectivity check
pub struct DatabaseHealthCheck {
    pool: Arc<DatabasePool>,
}

#[async_trait]
impl HealthCheck for DatabaseHealthCheck {
    async fn check(&self) -> HealthStatus {
        match self.pool.get().await {
            Ok(conn) => {
                match sqlx::query("SELECT 1").execute(&*conn).await {
                    Ok(_) => HealthStatus::Healthy,
                    Err(e) => HealthStatus::Unhealthy(format!("Query failed: {}", e)),
                }
            }
            Err(e) => HealthStatus::Unhealthy(format!("Connection failed: {}", e)),
        }
    }
    
    fn name(&self) -> &str {
        "database"
    }
}
```

### Integration Metrics
```rust
// Integration performance metrics
pub struct IntegrationMetrics {
    request_duration: Histogram,
    request_count: Counter,
    error_count: Counter,
    circuit_breaker_state: Gauge,
}

impl IntegrationMetrics {
    pub fn record_request(&self, service: &str, operation: &str, duration: Duration, success: bool) {
        let labels = &[
            ("service", service),
            ("operation", operation),
            ("status", if success { "success" } else { "error" }),
        ];
        
        self.request_duration.observe(duration.as_secs_f64(), labels);
        self.request_count.increment(labels);
        
        if !success {
            self.error_count.increment(labels);
        }
    }
}
```

This comprehensive integration specification ensures all ADX CORE services communicate reliably and consistently, enabling seamless system operation and maintainability.