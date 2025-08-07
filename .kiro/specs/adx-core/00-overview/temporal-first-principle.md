# ADX CORE - Temporal-First Principle

## Core Architectural Principle

**"If it's more complex than a simple CRUD operation, it MUST be a Temporal workflow."**

This principle is the foundation of ADX CORE's architecture and must be applied consistently across all modules, designs, and implementations.

## What This Means

### ✅ USE Temporal Workflows For:
- **Multi-step processes** (user registration, file upload, tenant provisioning)
- **Long-running operations** (data migration, bulk processing, monitoring)
- **Operations requiring retry logic** (external API calls, payment processing)
- **Operations requiring timeouts** (email verification, approval processes)
- **Operations requiring rollback** (plugin updates, tenant upgrades)
- **Operations requiring state persistence** (form wizards, complex workflows)
- **Operations requiring monitoring** (resource usage, health checks)
- **Error recovery scenarios** (service failures, data corruption)
- **Async operations** (background tasks, scheduled jobs)
- **Complex business logic** (approval workflows, multi-tenant operations)

### ❌ DON'T Use Temporal Workflows For:
- **Simple CRUD operations** (get user, update record, delete item)
- **Immediate responses** (health checks, simple queries)
- **Operations that never fail** (reading configuration, static data)
- **Pure calculations** (data transformations, validations)
- **Simple API endpoints** (basic REST operations)

## Module-by-Module Application

### 1. Authentication Service ✅
- `user_registration_workflow` - Complete registration with email verification
- `password_reset_workflow` - Password reset with timeout and cleanup
- `mfa_setup_workflow` - Multi-factor authentication setup
- `sso_authentication_workflow` - SSO login with external providers

### 2. File Service ✅
- `file_upload_workflow` - Upload with virus scan, validation, AI processing
- `file_sharing_workflow` - Share creation with notifications and expiration
- `file_migration_workflow` - Provider migration with rollback
- `file_processing_workflow` - Parallel processing (thumbnails, AI analysis)

### 3. Tenant Service ✅
- `tenant_provisioning_workflow` - Complete tenant setup with infrastructure
- `tenant_monitoring_workflow` - Continuous resource monitoring with alerts
- `tenant_upgrade_workflow` - Plan upgrades with payment processing
- `tenant_termination_workflow` - Complete cleanup with data export

### 4. Plugin System ✅
- `plugin_installation_workflow` - Installation with dependencies and security
- `plugin_update_workflow` - Updates with automatic rollback on failure
- `plugin_monitoring_workflow` - Continuous health and performance monitoring
- `plugin_removal_workflow` - Complete cleanup with dependency checking

### 5. API Gateway ✅
- `api_request_workflow` - Complex request processing with auth and validation
- `multi_service_orchestration_workflow` - Parallel service calls with aggregation
- `async_request_workflow` - Long-running async operations
- `error_recovery_workflow` - Intelligent error recovery and circuit breaking

### 6. Frontend ✅
- `frontend_file_upload_workflow` - Reliable file uploads with progress
- `data_sync_workflow` - Client-server data synchronization
- `form_wizard_workflow` - Multi-step forms with state persistence
- `bulk_operation_workflow` - Batch processing with progress tracking

### 7. Workflow Service ✅
- Already Temporal-native - provides templates and AI enhancement

## Implementation Guidelines

### Workflow Design Patterns
1. **Sequential Steps** - Use `await` for dependent operations
2. **Parallel Execution** - Use `temporal_sdk::join!` for independent operations
3. **Conditional Logic** - Use `temporal_sdk::select!` for timeouts and signals
4. **Error Handling** - Let Temporal handle retries, use compensation for rollback
5. **State Management** - Store state in workflow variables, not external systems
6. **Communication** - Use signals for external events, activities for operations

### Activity Design Patterns
1. **Idempotent** - Activities should be safe to retry
2. **Focused** - Each activity should do one thing well
3. **Stateless** - Activities should not maintain state between calls
4. **Error-aware** - Activities should handle and report errors clearly
5. **Timeout-aware** - Activities should complete within reasonable time limits

### Code Examples

#### ✅ Correct: Complex Operation as Workflow
```rust
#[workflow]
pub async fn user_registration_workflow(
    registration_data: UserRegistrationData
) -> WorkflowResult<User> {
    // Step 1: Validate registration data
    validate_registration_activity(registration_data.clone()).await?;
    
    // Step 2: Create user account
    let user = create_user_activity(registration_data).await?;
    
    // Step 3: Send verification email
    send_verification_email_activity(user.email.clone()).await?;
    
    // Step 4: Wait for email verification (with timeout)
    let verified = temporal_sdk::select! {
        _ = wait_for_email_verification_signal(user.id) => true,
        _ = temporal_sdk::sleep(Duration::from_hours(24)) => false,
    };
    
    if verified {
        activate_user_activity(user.id).await?;
    } else {
        cleanup_unverified_user_activity(user.id).await?;
    }
    
    Ok(user)
}
```

#### ❌ Incorrect: Simple CRUD as Workflow
```rust
// DON'T DO THIS - Simple operations don't need workflows
#[workflow]
pub async fn get_user_workflow(user_id: UserId) -> WorkflowResult<User> {
    let user = get_user_activity(user_id).await?;
    Ok(user)
}

// DO THIS INSTEAD - Direct service call
pub async fn get_user(user_id: UserId) -> Result<User, Error> {
    user_repository.get_user_by_id(user_id).await
}
```

## Benefits of Temporal-First Approach

### 1. Simplified Development
- **No custom orchestration code** - just Temporal workflows
- **Built-in error handling** - no custom retry logic needed
- **Visual debugging** - Temporal UI shows exactly what happened
- **Easy testing** - individual activities and workflows are testable

### 2. Operational Excellence
- **Automatic retries** for transient failures
- **State persistence** across service restarts
- **Timeout handling** for long-running operations
- **Workflow history** for audit and debugging

### 3. Scalability
- **Horizontal scaling** with Temporal workers
- **Resource isolation** through worker pools
- **Load balancing** handled by Temporal
- **Auto-scaling** based on queue depth

### 4. Reliability
- **Exactly-once execution** guarantees
- **Durable state** survives failures
- **Automatic recovery** from crashes
- **Consistent error handling** across all operations

## Enforcement

### Code Review Checklist
- [ ] Complex operations implemented as Temporal workflows
- [ ] Simple CRUD operations NOT implemented as workflows
- [ ] Workflows use proper error handling patterns
- [ ] Activities are idempotent and focused
- [ ] Parallel operations use `temporal_sdk::join!`
- [ ] Timeouts use `temporal_sdk::select!`

### Architecture Review Questions
1. Is this operation complex enough to warrant a workflow?
2. Does this operation need retry logic or error recovery?
3. Could this operation benefit from state persistence?
4. Does this operation need to be monitored or audited?
5. Is this operation part of a larger multi-step process?

If the answer to any of these is "yes", it should be a Temporal workflow.

## Documentation Requirements

### Requirements Documents
Every requirements document MUST include at least one requirement that explicitly states when Temporal workflows will be used for that module's operations.

**Template:**
```markdown
### REQ-XXX-001: Temporal-First [Module] Operations
**User Story:** As a user, I want reliable [module] operations, so that [complex operations] are handled with Temporal's durability.

**Acceptance Criteria:**
1. WHEN [complex operation] occurs THEN the system SHALL use Temporal workflows for [specific process]
2. WHEN [operation] fails THEN the system SHALL use Temporal's retry mechanisms for recovery
3. WHEN [long-running operation] is needed THEN the system SHALL use Temporal workflows for state persistence
4. WHEN [multi-step process] is required THEN the system SHALL use Temporal workflows instead of custom logic
5. WHEN [error recovery] is needed THEN the system SHALL use Temporal's built-in error handling
```

### Design Documents
Every design document MUST include:
1. **Temporal Workflows section** - List all workflows for the module
2. **Activity definitions** - Define all activities used by workflows
3. **Workflow patterns** - Show how complex operations are orchestrated
4. **Error handling** - Explain how Temporal handles failures and retries

## Conclusion

The **Temporal-First Principle** is not optional - it's a core architectural requirement that ensures ADX CORE is:
- **Simple to develop** (no custom orchestration)
- **Easy to maintain** (visual workflow debugging)
- **Highly reliable** (built-in error handling)
- **Operationally excellent** (automatic monitoring and recovery)

By consistently applying this principle, we eliminate custom complexity while gaining enterprise-grade reliability and operational excellence.