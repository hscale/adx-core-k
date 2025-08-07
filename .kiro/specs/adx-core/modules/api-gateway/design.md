# API Gateway - Temporal-First Design

## Overview

The API Gateway uses Temporal workflows for all complex request processing, eliminating custom orchestration logic in favor of proven workflow patterns.

```
┌─────────────────────────────────────────────────────────────┐
│              Temporal-First API Gateway                    │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Request        │   Processing    │    Response             │
│  Workflows      │   Activities    │    Workflows            │
│                 │                 │                         │
│ • Auth Flow     │ • Route Lookup  │ • Response Transform   │
│ • Rate Limit    │ • Load Balance  │ • Error Handling       │
│ • Validation    │ • Service Call  │ • Retry Logic          │
│ • Orchestration │ • Data Transform│ • Circuit Breaking     │
└─────────────────┴─────────────────┴─────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Temporal    │    │   Redis       │    │   Backend     │
│   Workflows   │    │   (Cache)     │    │   Services    │
└───────────────┘    └───────────────┘    └───────────────┘
```

## Core Temporal Workflows

### 1. Request Processing Workflow
```rust
#[workflow]
pub async fn api_request_workflow(
    request: IncomingRequest,
) -> WorkflowResult<APIResponse> {
    // Step 1: Authentication and authorization
    let auth_result = authenticate_request_activity(request.clone()).await?;
    
    if !auth_result.is_authenticated {
        return Ok(APIResponse::unauthorized());
    }
    
    // Step 2: Rate limiting check
    let rate_limit_result = check_rate_limit_activity(
        auth_result.user_id,
        auth_result.tenant_id,
        request.endpoint.clone(),
    ).await?;
    
    if rate_limit_result.is_exceeded {
        return Ok(APIResponse::rate_limited(rate_limit_result.retry_after));
    }
    
    // Step 3: Request validation
    validate_request_activity(request.clone()).await?;
    
    // Step 4: Route to appropriate service
    let service_response = route_to_service_activity(
        request.clone(),
        auth_result.context,
    ).await?;
    
    // Step 5: Transform response
    let final_response = transform_response_activity(
        service_response,
        request.response_format,
    ).await?;
    
    // Step 6: Log request for analytics
    log_request_activity(request, final_response.clone()).await?;
    
    Ok(final_response)
}
```

### 2. Complex Multi-Service Workflow
```rust
#[workflow]
pub async fn multi_service_orchestration_workflow(
    request: ComplexRequest,
    auth_context: AuthContext,
) -> WorkflowResult<OrchestrationResponse> {
    // Step 1: Validate complex request
    validate_complex_request_activity(request.clone()).await?;
    
    // Step 2: Call multiple services in parallel
    let (user_data, tenant_data, permissions) = temporal_sdk::join!(
        call_user_service_activity(request.user_id, auth_context.clone()),
        call_tenant_service_activity(request.tenant_id, auth_context.clone()),
        call_auth_service_activity(request.user_id, request.permissions_needed.clone())
    );
    
    // Step 3: Aggregate results
    let aggregated_data = aggregate_service_responses_activity(
        user_data?,
        tenant_data?,
        permissions?,
    ).await?;
    
    // Step 4: Apply business logic
    let processed_data = apply_business_logic_activity(
        aggregated_data,
        request.business_rules,
    ).await?;
    
    // Step 5: Format final response
    let response = format_orchestration_response_activity(
        processed_data,
        request.response_format,
    ).await?;
    
    Ok(response)
}
```

### 3. Async Request Processing Workflow
```rust
#[workflow]
pub async fn async_request_workflow(
    request: AsyncRequest,
    auth_context: AuthContext,
) -> WorkflowResult<AsyncResponse> {
    // Step 1: Create job record
    let job = create_async_job_activity(request.clone(), auth_context.clone()).await?;
    
    // Step 2: Send immediate response to client
    send_async_response_activity(
        request.callback_url.clone(),
        AsyncResponse::accepted(job.id.clone()),
    ).await?;
    
    // Step 3: Process request (potentially long-running)
    let processing_result = process_async_request_activity(
        request.clone(),
        auth_context,
    ).await?;
    
    // Step 4: Update job status
    update_job_status_activity(
        job.id.clone(),
        JobStatus::Completed,
        Some(processing_result.clone()),
    ).await?;
    
    // Step 5: Send completion notification
    if let Some(callback_url) = request.callback_url {
        send_completion_notification_activity(
            callback_url,
            job.id,
            processing_result,
        ).await?;
    }
    
    Ok(AsyncResponse::completed(processing_result))
}
```

### 4. Error Recovery Workflow
```rust
#[workflow]
pub async fn error_recovery_workflow(
    failed_request: FailedRequest,
    error_context: ErrorContext,
) -> WorkflowResult<RecoveryResult> {
    // Step 1: Analyze error
    let error_analysis = analyze_error_activity(
        failed_request.clone(),
        error_context.clone(),
    ).await?;
    
    // Step 2: Determine recovery strategy
    let recovery_strategy = determine_recovery_strategy_activity(
        error_analysis.clone(),
    ).await?;
    
    // Step 3: Execute recovery based on strategy
    let recovery_result = match recovery_strategy.strategy_type {
        RecoveryType::Retry => {
            // Use Temporal's built-in retry with exponential backoff
            temporal_sdk::retry_with_policy(
                recovery_strategy.retry_policy,
                || async {
                    retry_original_request_activity(failed_request.clone()).await
                }
            ).await?
        },
        RecoveryType::Fallback => {
            execute_fallback_activity(
                failed_request.clone(),
                recovery_strategy.fallback_config,
            ).await?
        },
        RecoveryType::CircuitBreaker => {
            activate_circuit_breaker_activity(
                error_context.service_name,
                recovery_strategy.circuit_config,
            ).await?
        },
    };
    
    // Step 4: Log recovery outcome
    log_recovery_outcome_activity(
        failed_request,
        recovery_strategy,
        recovery_result.clone(),
    ).await?;
    
    Ok(recovery_result)
}
```

## Simple Gateway Activities

### Authentication Activities
```rust
#[activity]
pub async fn authenticate_request_activity(
    request: IncomingRequest,
) -> Result<AuthResult, ActivityError> {
    let auth_service = get_auth_service().await?;
    
    // Extract token from request
    let token = extract_auth_token(&request)?;
    
    // Validate token
    let validation_result = auth_service.validate_token(&token).await?;
    
    if !validation_result.is_valid {
        return Ok(AuthResult {
            is_authenticated: false,
            user_id: None,
            tenant_id: None,
            context: None,
        });
    }
    
    Ok(AuthResult {
        is_authenticated: true,
        user_id: Some(validation_result.user_id),
        tenant_id: Some(validation_result.tenant_id),
        context: Some(validation_result.context),
    })
}

#[activity]
pub async fn check_rate_limit_activity(
    user_id: UserId,
    tenant_id: TenantId,
    endpoint: String,
) -> Result<RateLimitResult, ActivityError> {
    let rate_limiter = get_rate_limiter().await?;
    
    // Check user-level rate limit
    let user_limit = rate_limiter.check_user_limit(user_id, &endpoint).await?;
    if user_limit.is_exceeded {
        return Ok(RateLimitResult {
            is_exceeded: true,
            retry_after: user_limit.retry_after,
            limit_type: LimitType::User,
        });
    }
    
    // Check tenant-level rate limit
    let tenant_limit = rate_limiter.check_tenant_limit(tenant_id, &endpoint).await?;
    if tenant_limit.is_exceeded {
        return Ok(RateLimitResult {
            is_exceeded: true,
            retry_after: tenant_limit.retry_after,
            limit_type: LimitType::Tenant,
        });
    }
    
    Ok(RateLimitResult {
        is_exceeded: false,
        retry_after: None,
        limit_type: LimitType::None,
    })
}
```

### Service Routing Activities
```rust
#[activity]
pub async fn route_to_service_activity(
    request: IncomingRequest,
    auth_context: AuthContext,
) -> Result<ServiceResponse, ActivityError> {
    let service_registry = get_service_registry().await?;
    
    // Find target service
    let service_config = service_registry
        .find_service_for_endpoint(&request.endpoint)
        .ok_or(ActivityError::ServiceNotFound)?;
    
    // Load balance across healthy instances
    let service_instance = service_registry
        .get_healthy_instance(&service_config.name)
        .await?;
    
    // Make service call with retry
    let client = get_http_client();
    let service_request = build_service_request(request, auth_context)?;
    
    let response = client
        .request(service_request)
        .send()
        .await
        .map_err(|e| ActivityError::ServiceCallFailed(e.to_string()))?;
    
    Ok(ServiceResponse {
        status_code: response.status().as_u16(),
        headers: extract_headers(&response),
        body: response.text().await?,
        service_name: service_config.name,
        instance_id: service_instance.id,
    })
}

#[activity]
pub async fn call_user_service_activity(
    user_id: UserId,
    auth_context: AuthContext,
) -> Result<UserData, ActivityError> {
    let user_service = get_user_service().await?;
    
    let user_data = user_service
        .get_user(user_id, auth_context)
        .await
        .map_err(|e| ActivityError::UserServiceError(e.to_string()))?;
    
    Ok(user_data)
}

#[activity]
pub async fn aggregate_service_responses_activity(
    user_data: UserData,
    tenant_data: TenantData,
    permissions: PermissionData,
) -> Result<AggregatedData, ActivityError> {
    // Simple aggregation logic
    let aggregated = AggregatedData {
        user: user_data,
        tenant: tenant_data,
        permissions,
        aggregated_at: Utc::now(),
    };
    
    Ok(aggregated)
}
```

## Database Schema (Simplified)

```sql
-- API Gateway request logs
CREATE TABLE api_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id VARCHAR(255) UNIQUE NOT NULL,
    temporal_workflow_id VARCHAR(255),
    
    -- Request details
    method VARCHAR(10) NOT NULL,
    endpoint VARCHAR(500) NOT NULL,
    user_id UUID,
    tenant_id UUID,
    
    -- Response details
    status_code INTEGER,
    response_time_ms INTEGER,
    error_message TEXT,
    
    -- Metadata
    user_agent TEXT,
    ip_address INET,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    INDEX idx_api_requests_tenant_date (tenant_id, created_at),
    INDEX idx_api_requests_endpoint (endpoint),
    INDEX idx_api_requests_user (user_id, created_at)
);

-- Rate limiting counters (Redis-backed)
CREATE TABLE rate_limit_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    endpoint_pattern VARCHAR(500) NOT NULL,
    requests_per_minute INTEGER NOT NULL,
    requests_per_hour INTEGER NOT NULL,
    requests_per_day INTEGER NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    UNIQUE(tenant_id, endpoint_pattern)
);
```

## Key Benefits of Temporal-First API Gateway

### 1. Simplified Complex Request Processing
- **Multi-service orchestration** becomes a simple workflow
- **Async request handling** uses Temporal's durability
- **Error recovery** leverages Temporal's retry mechanisms
- **Request validation** can be complex workflows

### 2. Built-in Reliability
- **Automatic retries** for failed service calls
- **State persistence** across gateway restarts
- **Timeout handling** for long-running requests
- **Circuit breaker** patterns using Temporal workflows

### 3. Easy Monitoring and Debugging
- **Request tracing** through Temporal workflow history
- **Visual debugging** using Temporal UI
- **Step-by-step execution** makes troubleshooting easy
- **Replay capability** for testing and analysis

### 4. Scalable and Maintainable
- **Horizontal scaling** with Temporal workers
- **Version management** for gateway logic updates
- **Clear separation** between routing and business logic
- **Easy testing** of individual activities and workflows

This Temporal-first approach makes the API Gateway **simple, reliable, and maintainable** while handling all the complexity of request processing through proven workflow patterns.