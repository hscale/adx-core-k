# 🔄 Temporal Workflow Patterns - Enterprise Grade

## Core Workflow Philosophy

> **"Make it durable, make it reliable, make it observable. If it's complex business logic, it's a workflow."**

These patterns ensure bulletproof business process execution using Temporal's battle-tested orchestration.

## 🎯 Workflow Design Patterns

### 1. Standard Workflow Template
```rust
use temporal_sdk::{WfContext, WorkflowResult, ActivityOptions};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};

/// Standard workflow template for all ADX Core workflows
/// 
/// Every workflow should follow this pattern for consistency and reliability.
#[workflow]
pub async fn standard_business_workflow(
    ctx: &mut WfContext,
    input: WorkflowInput,
) -> WorkflowResult<WorkflowOutput> {
    // Step 1: Input validation and logging
    tracing::info!(
        workflow_id = %ctx.info().workflow_execution.workflow_id,
        workflow_type = %ctx.info().workflow_type.name,
        input = ?input,
        "Workflow started"
    );
    
    // Step 2: Validate tenant access (SECURITY CRITICAL)
    let validation_result = ctx.activity(
        ActivityOptions {
            activity_type: validate_tenant_access_activity.clone(),
            start_to_close_timeout: Some(Duration::seconds(30)),
            retry_policy: Some(RetryPolicy {
                initial_interval: Duration::seconds(1),
                maximum_interval: Duration::seconds(10),
                maximum_attempts: 3,
                ..Default::default()
            }),
        },
        TenantValidationInput {
            user_id: input.user_id,
            tenant_id: input.tenant_id,
            resource_type: input.resource_type.clone(),
        }
    ).await?;
    
    if !validation_result.is_valid {
        tracing::warn!(
            user_id = %input.user_id,
            tenant_id = %input.tenant_id,
            reason = %validation_result.reason,
            "Tenant validation failed"
        );
        return Err(WorkflowError::TenantValidationFailed(validation_result.reason));
    }
    
    // Step 3: Main business logic with proper error handling
    let business_result = match input.operation_type {
        OperationType::Create => {
            execute_create_operation(ctx, input).await?
        },
        OperationType::Update => {
            execute_update_operation(ctx, input).await?
        },
        OperationType::Delete => {
            execute_delete_operation(ctx, input).await?
        },
    };
    
    // Step 4: Audit logging (required for compliance)
    ctx.activity(
        ActivityOptions {
            activity_type: audit_log_activity.clone(),
            start_to_close_timeout: Some(Duration::seconds(10)),
            retry_policy: Some(RetryPolicy::default()),
        },
        AuditLogEntry {
            user_id: input.user_id,
            tenant_id: input.tenant_id,
            operation: input.operation_type,
            resource_id: business_result.resource_id,
            timestamp: ctx.now(),
            metadata: business_result.metadata.clone(),
        }
    ).await?;
    
    // Step 5: Post-processing workflows (if needed)
    if business_result.requires_post_processing {
        ctx.start_child_workflow(
            post_processing_workflow,
            PostProcessingInput {
                resource_id: business_result.resource_id,
                processing_type: business_result.post_processing_type,
            }
        ).await?;
    }
    
    tracing::info!(
        workflow_id = %ctx.info().workflow_execution.workflow_id,
        result = ?business_result,
        "Workflow completed successfully"
    );
    
    Ok(WorkflowOutput {
        resource_id: business_result.resource_id,
        status: WorkflowStatus::Completed,
        metadata: business_result.metadata,
        completed_at: ctx.now(),
    })
}
```

### 2. Saga Pattern for Distributed Transactions
```rust
/// Saga pattern for coordinating distributed transactions
/// Use this for operations that span multiple services
#[workflow]
pub async fn distributed_transaction_saga(
    ctx: &mut WfContext,
    input: SagaInput,
) -> WorkflowResult<SagaOutput> {
    let mut completed_steps = Vec::new();
    let mut compensation_needed = false;
    
    // Execute all saga steps
    for (step_index, step) in input.steps.iter().enumerate() {
        match execute_saga_step(ctx, step.clone()).await {
            Ok(result) => {
                completed_steps.push(SagaStepResult {
                    step_index,
                    status: SagaStepStatus::Completed,
                    result: Some(result),
                    error: None,
                });
            },
            Err(error) => {
                tracing::warn!(
                    step_index = step_index,
                    step_type = %step.step_type,
                    error = %error,
                    "Saga step failed, initiating compensation"
                );
                
                completed_steps.push(SagaStepResult {
                    step_index,
                    status: SagaStepStatus::Failed,
                    result: None,
                    error: Some(error.to_string()),
                });
                
                compensation_needed = true;
                break;
            }
        }
    }
    
    // Execute compensation if needed
    if compensation_needed {
        tracing::info!("Executing compensation for failed saga");
        
        // Compensate in reverse order
        for step_result in completed_steps.iter().rev() {
            if step_result.status == SagaStepStatus::Completed {
                let compensation_step = input.steps[step_result.step_index].compensation.clone();
                
                if let Err(comp_error) = execute_compensation_step(ctx, compensation_step).await {
                    tracing::error!(
                        step_index = step_result.step_index,
                        error = %comp_error,
                        "Compensation step failed - manual intervention required"
                    );
                    
                    // Send alert for manual intervention
                    ctx.activity(
                        send_manual_intervention_alert_activity,
                        ManualInterventionAlert {
                            saga_id: input.saga_id,
                            failed_compensation_step: step_result.step_index,
                            error_details: comp_error.to_string(),
                        }
                    ).await?;
                }
            }
        }
        
        return Ok(SagaOutput {
            status: SagaStatus::Compensated,
            completed_steps,
            error_message: Some("Saga compensated due to step failure".to_string()),
        });
    }
    
    Ok(SagaOutput {
        status: SagaStatus::Completed,
        completed_steps,
        error_message: None,
    })
}

// Helper function for executing individual saga steps
async fn execute_saga_step(
    ctx: &mut WfContext,
    step: SagaStep,
) -> WorkflowResult<serde_json::Value> {
    match step.step_type {
        SagaStepType::CreateUser => {
            ctx.activity(create_user_activity, step.input).await
        },
        SagaStepType::CreateTenant => {
            ctx.activity(create_tenant_activity, step.input).await
        },
        SagaStepType::SendNotification => {
            ctx.activity(send_notification_activity, step.input).await
        },
        SagaStepType::UpdateDatabase => {
            ctx.activity(update_database_activity, step.input).await
        },
    }
}
```

### 3. Long-Running Process Pattern
```rust
/// Long-running process with checkpoints and progress tracking
#[workflow]
pub async fn long_running_process_workflow(
    ctx: &mut WfContext,
    input: LongRunningProcessInput,
) -> WorkflowResult<LongRunningProcessOutput> {
    let process_id = Uuid::new_v4();
    let mut progress = ProcessProgress::new(process_id);
    
    // Initialize process tracking
    ctx.activity(
        initialize_process_tracking_activity,
        ProcessTrackingInit {
            process_id,
            total_items: input.items.len(),
            user_id: input.user_id,
            tenant_id: input.tenant_id,
        }
    ).await?;
    
    let mut processed_items = Vec::new();
    let mut failed_items = Vec::new();
    
    // Process items in batches to prevent workflow timeout
    const BATCH_SIZE: usize = 10;
    const CHECKPOINT_INTERVAL: usize = 50;
    
    for (batch_index, item_batch) in input.items.chunks(BATCH_SIZE).enumerate() {
        // Process batch
        let batch_results = ctx.activity(
            ActivityOptions {
                activity_type: process_item_batch_activity.clone(),
                start_to_close_timeout: Some(Duration::minutes(5)),
                heartbeat_timeout: Some(Duration::seconds(30)),
                retry_policy: Some(RetryPolicy {
                    initial_interval: Duration::seconds(10),
                    maximum_interval: Duration::minutes(5),
                    maximum_attempts: 3,
                    ..Default::default()
                }),
            },
            ItemBatch {
                items: item_batch.to_vec(),
                batch_index,
                process_id,
            }
        ).await?;
        
        // Update progress
        for result in batch_results {
            match result.status {
                ItemProcessingStatus::Success => {
                    processed_items.push(result.item_id);
                    progress.increment_success();
                },
                ItemProcessingStatus::Failed => {
                    failed_items.push((result.item_id, result.error_message));
                    progress.increment_failure();
                },
            }
        }
        
        // Checkpoint progress every N batches
        if batch_index % (CHECKPOINT_INTERVAL / BATCH_SIZE) == 0 {
            ctx.activity(
                update_process_progress_activity,
                progress.clone()
            ).await?;
            
            // Send progress update to user
            ctx.activity(
                send_progress_notification_activity,
                ProgressNotification {
                    user_id: input.user_id,
                    process_id,
                    current_progress: progress.percentage(),
                    estimated_completion: progress.estimated_completion(),
                }
            ).await?;
        }
        
        // Yield control to prevent workflow timeout
        if batch_index % 10 == 0 {
            ctx.timer(Duration::milliseconds(100)).await?;
        }
    }
    
    // Final progress update
    progress.mark_completed();
    ctx.activity(
        finalize_process_tracking_activity,
        ProcessCompletion {
            process_id,
            total_processed: processed_items.len(),
            total_failed: failed_items.len(),
            completion_time: ctx.now(),
        }
    ).await?;
    
    // Send completion notification
    ctx.activity(
        send_completion_notification_activity,
        CompletionNotification {
            user_id: input.user_id,
            process_id,
            success_count: processed_items.len(),
            failure_count: failed_items.len(),
            failed_items: failed_items.clone(),
        }
    ).await?;
    
    Ok(LongRunningProcessOutput {
        process_id,
        status: if failed_items.is_empty() {
            ProcessStatus::CompletedSuccessfully
        } else {
            ProcessStatus::CompletedWithErrors
        },
        processed_items,
        failed_items,
        total_duration: progress.total_duration(),
    })
}
```

### 4. Event-Driven Workflow Pattern
```rust
/// Event-driven workflow that reacts to external events
#[workflow]
pub async fn event_driven_workflow(
    ctx: &mut WfContext,
    input: EventWorkflowInput,
) -> WorkflowResult<EventWorkflowOutput> {
    let mut workflow_state = WorkflowState::new(input.workflow_id);
    let mut event_count = 0;
    
    // Set up event listeners
    let timeout_duration = Duration::hours(24); // 24-hour timeout
    let mut timeout = ctx.timer(timeout_duration);
    
    loop {
        let event_result = ctx.select! {
            // Wait for external events
            event = ctx.receive_signal::<WorkflowEvent>("workflow_event") => {
                Some(event)
            },
            
            // Handle timeout
            _ = &mut timeout => {
                tracing::warn!(
                    workflow_id = %input.workflow_id,
                    event_count = event_count,
                    "Workflow timed out waiting for events"
                );
                None
            }
        };
        
        match event_result {
            Some(event) => {
                event_count += 1;
                
                // Process the event
                let processing_result = ctx.activity(
                    process_workflow_event_activity,
                    EventProcessingInput {
                        event: event.clone(),
                        current_state: workflow_state.clone(),
                        workflow_id: input.workflow_id,
                    }
                ).await?;
                
                // Update workflow state
                workflow_state = processing_result.new_state;
                
                // Check if workflow should complete
                if processing_result.should_complete {
                    tracing::info!(
                        workflow_id = %input.workflow_id,
                        event_count = event_count,
                        final_state = ?workflow_state,
                        "Workflow completed due to completion event"
                    );
                    
                    return Ok(EventWorkflowOutput {
                        workflow_id: input.workflow_id,
                        final_state: workflow_state,
                        event_count,
                        completion_reason: CompletionReason::EventTriggered,
                    });
                }
                
                // Continue processing if needed
                if processing_result.requires_followup {
                    ctx.activity(
                        execute_followup_action_activity,
                        processing_result.followup_action
                    ).await?;
                }
                
                // Reset timeout if we're still active
                timeout = ctx.timer(timeout_duration);
            },
            None => {
                // Timeout occurred
                return Ok(EventWorkflowOutput {
                    workflow_id: input.workflow_id,
                    final_state: workflow_state,
                    event_count,
                    completion_reason: CompletionReason::Timeout,
                });
            }
        }
    }
}
```

## 🎯 Activity Design Patterns

### 1. Standard Activity Template
```rust
/// Standard activity template for all ADX Core activities
/// 
/// Every activity should follow this pattern for consistency and reliability.
#[activity]
pub async fn standard_business_activity(
    input: ActivityInput,
) -> Result<ActivityOutput, ActivityError> {
    let activity_start = Instant::now();
    let activity_id = Uuid::new_v4();
    
    tracing::info!(
        activity_id = %activity_id,
        activity_type = "standard_business_activity",
        input = ?input,
        "Activity started"
    );
    
    // Step 1: Input validation
    validate_activity_input(&input)?;
    
    // Step 2: Tenant isolation check (SECURITY CRITICAL)
    validate_tenant_access(input.user_id, input.tenant_id, &input.resource_id).await?;
    
    // Step 3: Rate limiting check
    check_activity_rate_limit(input.user_id, input.tenant_id, "standard_business_activity").await?;
    
    // Step 4: Main business logic with error handling
    let result = execute_business_logic(&input).await.map_err(|e| {
        tracing::error!(
            activity_id = %activity_id,
            error = %e,
            "Business logic execution failed"
        );
        ActivityError::BusinessLogicError(e.to_string())
    })?;
    
    // Step 5: Audit logging (if required)
    if input.requires_audit {
        audit_activity_execution(
            activity_id,
            input.user_id,
            input.tenant_id,
            "standard_business_activity",
            &result
        ).await?;
    }
    
    let activity_duration = activity_start.elapsed();
    
    tracing::info!(
        activity_id = %activity_id,
        duration_ms = activity_duration.as_millis(),
        result = ?result,
        "Activity completed successfully"
    );
    
    // Performance monitoring
    metrics::histogram!("activity_duration_ms", activity_duration.as_millis() as f64)
        .with_tag("activity_type", "standard_business_activity");
    
    Ok(ActivityOutput {
        activity_id,
        result,
        execution_time: activity_duration,
        metadata: generate_activity_metadata(&input, &result),
    })
}

// Helper functions for standard activity pattern
fn validate_activity_input(input: &ActivityInput) -> Result<(), ActivityError> {
    if input.user_id.is_nil() {
        return Err(ActivityError::InvalidInput("user_id cannot be nil".to_string()));
    }
    
    if input.tenant_id.is_nil() {
        return Err(ActivityError::InvalidInput("tenant_id cannot be nil".to_string()));
    }
    
    // Add more validation as needed
    Ok(())
}

async fn execute_business_logic(input: &ActivityInput) -> Result<BusinessResult, BusinessError> {
    // Implement your specific business logic here
    // This is just a placeholder
    
    match input.operation_type {
        OperationType::Create => {
            // Create operation logic
            create_resource(input).await
        },
        OperationType::Update => {
            // Update operation logic
            update_resource(input).await
        },
        OperationType::Delete => {
            // Delete operation logic
            delete_resource(input).await
        },
    }
}
```

### 2. Database Activity Pattern
```rust
/// Database activity with transaction support and retry logic
#[activity]
pub async fn database_transaction_activity(
    input: DatabaseTransactionInput,
) -> Result<DatabaseTransactionOutput, ActivityError> {
    let activity_start = Instant::now();
    
    // Get database connection from pool
    let db = get_database_connection().await?;
    
    // Start transaction
    let mut transaction = db.begin().await
        .map_err(|e| ActivityError::DatabaseError(e.to_string()))?;
    
    let mut operations_completed = Vec::new();
    
    // Execute all operations in transaction
    for (operation_index, operation) in input.operations.iter().enumerate() {
        match execute_database_operation(&mut transaction, operation).await {
            Ok(result) => {
                operations_completed.push(DatabaseOperationResult {
                    operation_index,
                    status: OperationStatus::Success,
                    result: Some(result),
                    error: None,
                });
            },
            Err(error) => {
                // Rollback transaction on any failure
                transaction.rollback().await
                    .map_err(|e| ActivityError::DatabaseError(format!("Rollback failed: {}", e)))?;
                
                tracing::error!(
                    operation_index = operation_index,
                    operation_type = %operation.operation_type,
                    error = %error,
                    "Database operation failed, transaction rolled back"
                );
                
                return Err(ActivityError::DatabaseError(error.to_string()));
            }
        }
    }
    
    // Commit transaction if all operations succeeded
    transaction.commit().await
        .map_err(|e| ActivityError::DatabaseError(format!("Commit failed: {}", e)))?;
    
    let activity_duration = activity_start.elapsed();
    
    // Performance monitoring
    metrics::histogram!("db_transaction_duration_ms", activity_duration.as_millis() as f64)
        .with_tag("operation_count", input.operations.len().to_string());
    
    Ok(DatabaseTransactionOutput {
        operations_completed,
        transaction_id: input.transaction_id,
        execution_time: activity_duration,
    })
}
```

### 3. External API Activity Pattern
```rust
/// External API activity with retry and circuit breaker
#[activity]
pub async fn external_api_activity(
    input: ExternalApiInput,
) -> Result<ExternalApiOutput, ActivityError> {
    let activity_start = Instant::now();
    let client = get_http_client();
    
    // Implement circuit breaker pattern
    let circuit_breaker = get_circuit_breaker(&input.service_name).await;
    
    if circuit_breaker.is_open() {
        return Err(ActivityError::CircuitBreakerOpen(input.service_name.clone()));
    }
    
    // Build request
    let request = client
        .request(input.method.clone(), &input.url)
        .headers(input.headers.clone())
        .timeout(Duration::from_secs(30));
    
    let request = if let Some(body) = input.body {
        request.json(&body)
    } else {
        request
    };
    
    // Execute request with retry logic
    let mut last_error = None;
    
    for attempt in 1..=3 {
        match request.try_clone().unwrap().send().await {
            Ok(response) => {
                let status = response.status();
                let response_body = response.text().await
                    .map_err(|e| ActivityError::ExternalApiError(e.to_string()))?;
                
                // Record success in circuit breaker
                circuit_breaker.record_success().await;
                
                let activity_duration = activity_start.elapsed();
                
                // Performance monitoring
                metrics::histogram!("external_api_duration_ms", activity_duration.as_millis() as f64)
                    .with_tag("service", &input.service_name)
                    .with_tag("status", status.as_u16().to_string());
                
                return Ok(ExternalApiOutput {
                    status_code: status.as_u16(),
                    body: response_body,
                    execution_time: activity_duration,
                });
            },
            Err(error) => {
                last_error = Some(error);
                
                tracing::warn!(
                    attempt = attempt,
                    service = %input.service_name,
                    url = %input.url,
                    error = %last_error.as_ref().unwrap(),
                    "External API call failed, retrying..."
                );
                
                // Exponential backoff
                if attempt < 3 {
                    let delay = Duration::from_millis(100 * (2_u64.pow(attempt - 1)));
                    tokio::time::sleep(delay).await;
                }
            }
        }
    }
    
    // All retries failed
    let error = last_error.unwrap();
    
    // Record failure in circuit breaker
    circuit_breaker.record_failure().await;
    
    Err(ActivityError::ExternalApiError(error.to_string()))
}
```

## 🚨 Error Handling Patterns

### 1. Workflow Error Recovery
```rust
/// Workflow with comprehensive error handling and recovery
#[workflow]
pub async fn resilient_workflow(
    ctx: &mut WfContext,
    input: ResilientWorkflowInput,
) -> WorkflowResult<ResilientWorkflowOutput> {
    let mut recovery_attempts = 0;
    const MAX_RECOVERY_ATTEMPTS: u32 = 3;
    
    loop {
        match execute_main_workflow_logic(ctx, input.clone()).await {
            Ok(result) => {
                return Ok(result);
            },
            Err(error) => {
                recovery_attempts += 1;
                
                tracing::warn!(
                    workflow_id = %ctx.info().workflow_execution.workflow_id,
                    error = %error,
                    recovery_attempt = recovery_attempts,
                    "Workflow execution failed, attempting recovery"
                );
                
                // Determine if error is recoverable
                let recovery_strategy = determine_recovery_strategy(&error);
                
                match recovery_strategy {
                    RecoveryStrategy::Retry => {
                        if recovery_attempts <= MAX_RECOVERY_ATTEMPTS {
                            // Wait before retry with exponential backoff
                            let delay = Duration::seconds(2_i64.pow(recovery_attempts - 1));
                            ctx.timer(delay).await?;
                            continue;
                        } else {
                            // Max retries exceeded, fail workflow
                            return Err(WorkflowError::MaxRetriesExceeded {
                                original_error: error.to_string(),
                                attempts: recovery_attempts,
                            });
                        }
                    },
                    RecoveryStrategy::Compensate => {
                        // Execute compensation workflow
                        ctx.start_child_workflow(
                            compensation_workflow,
                            CompensationInput {
                                original_input: input.clone(),
                                failure_reason: error.to_string(),
                            }
                        ).await?;
                        
                        return Ok(ResilientWorkflowOutput {
                            status: WorkflowStatus::Compensated,
                            error_message: Some(error.to_string()),
                            recovery_attempts,
                        });
                    },
                    RecoveryStrategy::Escalate => {
                        // Send alert to operations team
                        ctx.activity(
                            send_escalation_alert_activity,
                            EscalationAlert {
                                workflow_id: ctx.info().workflow_execution.workflow_id.clone(),
                                error_details: error.to_string(),
                                escalation_level: EscalationLevel::Critical,
                            }
                        ).await?;
                        
                        return Err(WorkflowError::EscalatedError(error.to_string()));
                    }
                }
            }
        }
    }
}

fn determine_recovery_strategy(error: &WorkflowError) -> RecoveryStrategy {
    match error {
        WorkflowError::ActivityError(ActivityError::DatabaseError(_)) => RecoveryStrategy::Retry,
        WorkflowError::ActivityError(ActivityError::ExternalApiError(_)) => RecoveryStrategy::Retry,
        WorkflowError::TenantValidationFailed(_) => RecoveryStrategy::Escalate,
        WorkflowError::InvalidInput(_) => RecoveryStrategy::Escalate,
        _ => RecoveryStrategy::Compensate,
    }
}
```

---

**Remember: Every business process should be a workflow. Make it durable, observable, and reliable!** 🔄
