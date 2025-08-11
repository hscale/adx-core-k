# ADX Core Temporal Workflow Versioning and Migration Strategy

## Overview

This document outlines the comprehensive strategy for managing workflow versions and migrations in ADX Core's Temporal-first architecture. The versioning system ensures backward compatibility, safe deployments, and seamless evolution of business processes.

## Versioning Principles

### 1. Semantic Versioning
ADX Core workflows follow semantic versioning (SemVer) with the format `MAJOR.MINOR.PATCH[-PRERELEASE]`:

- **MAJOR**: Breaking changes that are incompatible with previous versions
- **MINOR**: Backward-compatible functionality additions
- **PATCH**: Backward-compatible bug fixes
- **PRERELEASE**: Alpha, beta, or release candidate versions

### 2. Compatibility Rules
- **Same Major Version**: All minor and patch versions within the same major version must be backward compatible
- **Cross-Major Compatibility**: Major version changes may introduce breaking changes
- **Deprecation Policy**: Versions are deprecated before removal with at least one major version notice

### 3. Version Lifecycle
1. **Development**: Pre-release versions (alpha, beta, rc)
2. **Active**: Current production versions
3. **Deprecated**: Versions marked for future removal
4. **End-of-Life**: Versions no longer supported

## Workflow Version Management

### Version Registration
```rust
// Register a new workflow version
let mut version_manager = AdxWorkflowVersionManager::new();

// Register new version
version_manager.register_version(
    "user_onboarding_workflow",
    WorkflowVersion::new(1, 2, 0),
    false, // Not default yet
)?;

// Set as default after validation
version_manager.register_version(
    "user_onboarding_workflow",
    WorkflowVersion::new(1, 2, 0),
    true, // Set as default
)?;
```

### Version Selection Strategy
1. **Explicit Version**: Client specifies exact version
2. **Default Version**: Use registered default version
3. **Latest Compatible**: Use latest version compatible with client requirements
4. **Fallback**: Use previous stable version if current version fails

### Workflow Execution Versioning
```rust
// Execute workflow with specific version
let workflow_request = WorkflowBuilder::new("user_onboarding_workflow")
    .version(WorkflowVersion::new(1, 2, 0))
    .input(onboarding_data)
    .user_context(user_context)
    .tenant_context(tenant_context)
    .build()?;

// Execute workflow with default version
let workflow_request = WorkflowBuilder::new("user_onboarding_workflow")
    // No version specified - uses default
    .input(onboarding_data)
    .user_context(user_context)
    .tenant_context(tenant_context)
    .build()?;
```

## Migration Strategies

### 1. Backward Compatible Changes (Minor/Patch Versions)

#### Adding New Activities
```rust
// Version 1.0.0 - Original workflow
#[workflow]
pub async fn user_onboarding_workflow_v1_0_0(
    request: UserOnboardingRequest,
) -> Result<UserOnboardingResult, WorkflowError> {
    // Step 1: Create user
    let user = call_activity(
        AuthServiceActivities::create_user,
        request.user_data,
    ).await?;
    
    // Step 2: Send welcome email
    call_activity(
        NotificationActivities::send_welcome_email,
        WelcomeEmailRequest { user_id: user.id },
    ).await?;
    
    Ok(UserOnboardingResult { user_id: user.id })
}

// Version 1.1.0 - Added profile setup (backward compatible)
#[workflow]
pub async fn user_onboarding_workflow_v1_1_0(
    request: UserOnboardingRequest,
) -> Result<UserOnboardingResult, WorkflowError> {
    // Step 1: Create user (unchanged)
    let user = call_activity(
        AuthServiceActivities::create_user,
        request.user_data,
    ).await?;
    
    // Step 2: Setup user profile (NEW - optional)
    if request.setup_profile.unwrap_or(false) {
        call_activity(
            UserServiceActivities::setup_user_profile,
            SetupProfileRequest { user_id: user.id.clone() },
        ).await?;
    }
    
    // Step 3: Send welcome email (unchanged)
    call_activity(
        NotificationActivities::send_welcome_email,
        WelcomeEmailRequest { user_id: user.id.clone() },
    ).await?;
    
    Ok(UserOnboardingResult { user_id: user.id })
}
```

#### Modifying Activity Parameters
```rust
// Version 1.1.0 - Original activity input
#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
}

// Version 1.2.0 - Extended activity input (backward compatible)
#[derive(Serialize, Deserialize)]
pub struct CreateUserRequest {
    pub email: String,
    pub password: String,
    // New optional fields
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub phone: Option<String>,
}
```

### 2. Breaking Changes (Major Versions)

#### Workflow Structure Changes
```rust
// Version 1.x.x - Sequential workflow
#[workflow]
pub async fn file_processing_workflow_v1(
    request: FileProcessingRequest,
) -> Result<FileProcessingResult, WorkflowError> {
    // Sequential processing
    let validated = call_activity(validate_file, request.file_id).await?;
    let scanned = call_activity(virus_scan, validated.file_id).await?;
    let processed = call_activity(process_file, scanned.file_id).await?;
    
    Ok(FileProcessingResult { file_id: processed.file_id })
}

// Version 2.0.0 - Parallel processing (breaking change)
#[workflow]
pub async fn file_processing_workflow_v2(
    request: FileProcessingRequest,
) -> Result<FileProcessingResult, WorkflowError> {
    // Parallel processing - different execution model
    let validation_future = call_activity(validate_file, request.file_id.clone());
    let scan_future = call_activity(virus_scan, request.file_id.clone());
    
    let (validated, scanned) = tokio::try_join!(validation_future, scan_future)?;
    
    let processed = call_activity(
        process_file, 
        ProcessFileRequest {
            file_id: request.file_id,
            validation_result: validated,
            scan_result: scanned,
        }
    ).await?;
    
    Ok(FileProcessingResult { file_id: processed.file_id })
}
```

### 3. Migration Execution Strategies

#### Blue-Green Deployment
```bash
# Deploy new version alongside old version
kubectl apply -f deployment-v2.yaml

# Route percentage of traffic to new version
kubectl patch service workflow-service -p '{"spec":{"selector":{"version":"v2"}}}'

# Monitor metrics and gradually increase traffic
# If issues occur, rollback immediately
kubectl patch service workflow-service -p '{"spec":{"selector":{"version":"v1"}}}'
```

#### Canary Deployment
```rust
// Route workflows based on tenant or user criteria
pub async fn route_workflow_version(
    workflow_type: &str,
    tenant_context: &TenantContext,
    user_context: &UserContext,
) -> WorkflowVersion {
    // Canary deployment logic
    if is_canary_tenant(&tenant_context.tenant_id) {
        WorkflowVersion::new(2, 0, 0) // New version
    } else if is_beta_user(&user_context.user_id) {
        WorkflowVersion::new(1, 9, 0) // Beta version
    } else {
        WorkflowVersion::new(1, 8, 0) // Stable version
    }
}
```

## Version Compatibility Matrix

| Workflow Version | Client Version | Compatible | Notes |
|------------------|----------------|------------|-------|
| 1.0.0 | 1.0.x | ✅ | Full compatibility |
| 1.1.0 | 1.0.x | ✅ | Backward compatible |
| 1.2.0 | 1.0.x | ✅ | New optional features |
| 2.0.0 | 1.x.x | ❌ | Breaking changes |
| 2.0.0 | 2.0.x | ✅ | Full compatibility |
| 2.1.0 | 2.0.x | ✅ | Backward compatible |

## Migration Testing Strategy

### 1. Replay Testing
```rust
#[tokio::test]
async fn test_workflow_version_compatibility() {
    let test_env = TestWorkflowEnvironment::new().await;
    
    // Load historical workflow execution from v1.0.0
    let v1_history = load_workflow_history("user_onboarding_v1_0_0_history.json");
    
    // Replay with v1.1.0 implementation
    let replay_result = test_env.replay_workflow(
        user_onboarding_workflow_v1_1_0,
        v1_history,
    ).await;
    
    // Should succeed (backward compatible)
    assert!(replay_result.is_ok());
}
```

### 2. Cross-Version Integration Testing
```rust
#[tokio::test]
async fn test_cross_version_activity_compatibility() {
    let test_env = IntegrationTestEnvironment::new().await;
    
    // Start workflow with v1.0.0
    let workflow_handle = test_env.start_workflow(
        "user_onboarding_workflow",
        WorkflowVersion::new(1, 0, 0),
        onboarding_request,
    ).await?;
    
    // Deploy v1.1.0 activities
    test_env.deploy_activities_version(WorkflowVersion::new(1, 1, 0)).await?;
    
    // Workflow should complete successfully
    let result = workflow_handle.get_result().await?;
    assert!(result.is_ok());
}
```

### 3. Load Testing with Multiple Versions
```rust
#[tokio::test]
async fn test_multi_version_load() {
    let test_env = LoadTestEnvironment::new().await;
    
    // Run workflows with different versions concurrently
    let mut handles = Vec::new();
    
    for i in 0..100 {
        let version = if i % 3 == 0 {
            WorkflowVersion::new(1, 0, 0) // 33% v1.0.0
        } else if i % 3 == 1 {
            WorkflowVersion::new(1, 1, 0) // 33% v1.1.0
        } else {
            WorkflowVersion::new(1, 2, 0) // 33% v1.2.0
        };
        
        let handle = test_env.start_workflow(
            "user_onboarding_workflow",
            version,
            create_test_request(i),
        ).await?;
        
        handles.push(handle);
    }
    
    // Wait for all workflows to complete
    let results = futures::future::try_join_all(handles).await?;
    
    // Verify all versions completed successfully
    for result in results {
        assert!(result.is_ok());
    }
}
```

## Monitoring and Observability

### Version Metrics
```rust
// Prometheus metrics for workflow versions
lazy_static! {
    static ref WORKFLOW_VERSION_EXECUTIONS: Counter = Counter::new(
        "workflow_version_executions_total",
        "Total workflow executions by version"
    ).unwrap();
    
    static ref WORKFLOW_VERSION_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new("workflow_version_duration_seconds", "Workflow duration by version")
            .buckets(vec![1.0, 5.0, 10.0, 30.0, 60.0, 300.0])
    ).unwrap();
    
    static ref WORKFLOW_VERSION_ERRORS: Counter = Counter::new(
        "workflow_version_errors_total",
        "Workflow errors by version"
    ).unwrap();
}

// Record metrics during workflow execution
pub fn record_workflow_metrics(
    workflow_type: &str,
    version: &WorkflowVersion,
    duration: Duration,
    success: bool,
) {
    let labels = &[
        ("workflow_type", workflow_type),
        ("version", &version.to_string()),
    ];
    
    WORKFLOW_VERSION_EXECUTIONS.with_label_values(labels).inc();
    WORKFLOW_VERSION_DURATION.with_label_values(labels).observe(duration.as_secs_f64());
    
    if !success {
        WORKFLOW_VERSION_ERRORS.with_label_values(labels).inc();
    }
}
```

### Version Health Checks
```rust
pub async fn check_workflow_version_health(
    version_manager: &AdxWorkflowVersionManager,
) -> VersionHealthReport {
    let mut report = VersionHealthReport::new();
    
    for workflow_type in CORE_WORKFLOW_TYPES {
        let versions = version_manager.registry().get_versions(workflow_type);
        
        if let Some(versions) = versions {
            for version in versions {
                let health = check_version_health(workflow_type, version).await;
                report.add_version_health(workflow_type, version.clone(), health);
            }
        }
    }
    
    report
}
```

## Rollback Procedures

### 1. Immediate Rollback
```bash
# Emergency rollback script
#!/bin/bash
set -e

WORKFLOW_TYPE=$1
OLD_VERSION=$2
NEW_VERSION=$3

echo "Rolling back $WORKFLOW_TYPE from $NEW_VERSION to $OLD_VERSION"

# Stop new workflow executions
kubectl patch deployment workflow-service -p '{"spec":{"replicas":0}}'

# Wait for current workflows to complete (with timeout)
timeout 300 kubectl wait --for=condition=available=false deployment/workflow-service

# Deploy old version
kubectl set image deployment/workflow-service workflow-service=adx-core/workflow-service:$OLD_VERSION

# Scale back up
kubectl patch deployment workflow-service -p '{"spec":{"replicas":3}}'

# Verify rollback
kubectl wait --for=condition=available deployment/workflow-service

echo "Rollback completed successfully"
```

### 2. Gradual Rollback
```rust
// Gradually reduce traffic to new version
pub async fn gradual_rollback(
    workflow_type: &str,
    from_version: WorkflowVersion,
    to_version: WorkflowVersion,
    rollback_percentage: u8,
) -> Result<(), TemporalError> {
    let mut current_percentage = 100;
    
    while current_percentage > rollback_percentage {
        current_percentage -= 10;
        
        // Update routing rules
        update_version_routing(
            workflow_type,
            &from_version,
            current_percentage,
        ).await?;
        
        // Wait and monitor
        tokio::time::sleep(Duration::from_secs(60)).await;
        
        // Check health metrics
        let health = check_version_health(workflow_type, &from_version).await;
        if !health.is_healthy() {
            // Accelerate rollback if issues detected
            current_percentage = rollback_percentage;
        }
    }
    
    Ok(())
}
```

## Best Practices

### 1. Development Guidelines
- Always increment version numbers for any workflow changes
- Test backward compatibility with replay tests
- Document breaking changes in migration guides
- Use feature flags for gradual rollouts

### 2. Deployment Guidelines
- Deploy new versions alongside old versions
- Use canary deployments for major version changes
- Monitor metrics during version transitions
- Have rollback procedures ready

### 3. Monitoring Guidelines
- Track version-specific metrics
- Set up alerts for version health issues
- Monitor workflow execution success rates by version
- Track version adoption rates

### 4. Documentation Guidelines
- Maintain version compatibility matrices
- Document migration procedures for each version
- Provide examples for version-specific features
- Keep deprecation notices up to date

## Conclusion

This versioning and migration strategy ensures that ADX Core can evolve its Temporal workflows safely and reliably. By following semantic versioning principles, implementing comprehensive testing, and maintaining proper monitoring, we can deliver new features while maintaining system stability and backward compatibility.

The strategy supports both gradual evolution through minor versions and major architectural changes through major versions, providing flexibility for different types of improvements while minimizing risk to production systems.