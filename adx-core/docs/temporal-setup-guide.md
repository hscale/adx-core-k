# ADX Core Temporal Setup Guide

## Overview

This guide provides comprehensive instructions for setting up and working with Temporal in the ADX Core development environment. ADX Core uses Temporal as the primary workflow orchestration engine for all complex business operations.

## Quick Start

### 1. Automated Setup (Recommended)

The fastest way to get started is using the automated setup script:

```bash
# Navigate to ADX Core directory
cd adx-core

# Run the Temporal development setup script
./scripts/temporal-dev-setup.sh
```

This script will:
- Start PostgreSQL database for Temporal
- Start Temporal server
- Start Temporal Web UI
- Create development namespaces
- Verify the setup

### 2. Manual Setup

If you prefer manual setup or need to customize the configuration:

```bash
# Start Temporal infrastructure
docker-compose -f infrastructure/docker/docker-compose.temporal.yml up -d

# Wait for services to be ready (may take 30-60 seconds)
docker-compose -f infrastructure/docker/docker-compose.temporal.yml logs -f

# Setup namespaces
./scripts/setup-temporal-namespaces.sh
```

## Services and Ports

After setup, the following services will be available:

| Service | URL/Port | Description |
|---------|----------|-------------|
| Temporal Server | `localhost:7233` | gRPC endpoint for Temporal clients |
| Temporal Web UI | `http://localhost:8088` | Web interface for monitoring workflows |
| PostgreSQL | `localhost:5432` | Database for Temporal (user: `temporal`, password: `temporal`) |

## Namespaces

ADX Core uses three Temporal namespaces for different environments:

| Namespace | Retention | Purpose |
|-----------|-----------|---------|
| `adx-core-development` | 72 hours | Local development and testing |
| `adx-core-staging` | 7 days | Integration testing and pre-production |
| `adx-core-production` | 1 year | Production workloads |

## Development Workflow

### 1. Using the Temporal Client

```rust
use adx_shared::temporal::{AdxTemporalClient, TemporalConfig};

// Create a client for development
let config = TemporalConfig::development();
let client = AdxTemporalClient::new(config).await?;

// Start a workflow
let workflow_handle = client.start_workflow(
    "user_onboarding_workflow",
    "user-123-onboarding".to_string(),
    "user-onboarding-queue",
    UserOnboardingRequest {
        user_id: "user-123".to_string(),
        email: "user@example.com".to_string(),
        tenant_id: "tenant-456".to_string(),
    },
).await?;

// Get the result
let result = workflow_handle.get_result().await?;
```

### 2. Implementing Workflows

```rust
use adx_shared::temporal::{WorkflowContext, WorkflowError};

#[temporal::workflow]
pub async fn user_onboarding_workflow(
    request: UserOnboardingRequest,
) -> Result<UserOnboardingResult, WorkflowError> {
    // Step 1: Validate user data
    let validation = call_activity(
        UserActivities::validate_user_data,
        request.clone(),
    ).await?;
    
    // Step 2: Create user account
    let user = call_activity(
        AuthActivities::create_user_account,
        CreateUserRequest {
            email: request.email,
            tenant_id: request.tenant_id,
        },
    ).await?;
    
    // Step 3: Send welcome email
    call_activity(
        NotificationActivities::send_welcome_email,
        WelcomeEmailRequest {
            user_id: user.id.clone(),
            email: user.email,
        },
    ).await?;
    
    Ok(UserOnboardingResult {
        user_id: user.id,
        success: true,
    })
}
```

### 3. Implementing Activities

```rust
use adx_shared::temporal::{ActivityContext, ActivityError, AdxActivity};

pub struct UserValidationActivity {
    user_repository: Arc<dyn UserRepository>,
}

#[async_trait]
impl AdxActivity<UserOnboardingRequest, ValidationResult> for UserValidationActivity {
    async fn execute(
        &self,
        context: ActivityContext,
        input: UserOnboardingRequest,
    ) -> Result<ValidationResult, ActivityError> {
        // Validate email format
        if !is_valid_email(&input.email) {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "Invalid email format".to_string(),
            });
        }
        
        // Check if user already exists
        let existing_user = self.user_repository
            .find_by_email(&input.email)
            .await
            .map_err(|e| ActivityError::DatabaseError {
                message: format!("Database error: {}", e),
            })?;
        
        if existing_user.is_some() {
            return Err(ActivityError::ValidationError {
                field: "email".to_string(),
                message: "User already exists".to_string(),
            });
        }
        
        Ok(ValidationResult {
            is_valid: true,
            user_id: generate_user_id(),
        })
    }
    
    fn activity_type(&self) -> &'static str {
        "validate_user_data"
    }
}
```

## Configuration

### Environment Variables

Set these environment variables for different environments:

```bash
# Development
export ADX_ENVIRONMENT=development
export TEMPORAL_SERVER_ADDRESS=localhost:7233
export TEMPORAL_NAMESPACE=adx-core-development

# Staging
export ADX_ENVIRONMENT=staging
export TEMPORAL_SERVER_ADDRESS=temporal-staging:7233
export TEMPORAL_NAMESPACE=adx-core-staging

# Production
export ADX_ENVIRONMENT=production
export TEMPORAL_SERVER_ADDRESS=temporal-prod:7233
export TEMPORAL_NAMESPACE=adx-core-production
```

### Service Configuration

Each service should include Temporal configuration in its `config.toml`:

```toml
[temporal]
server_address = "localhost:7233"
namespace = "adx-core-development"
client_identity = "auth-service"

[temporal.connection]
connect_timeout = "10s"
keep_alive_timeout = "30s"
max_concurrent_connections = 100

[temporal.worker]
max_concurrent_workflow_tasks = 100
max_concurrent_activity_tasks = 200
task_queues = ["auth-service-queue", "user-management-queue"]

[temporal.workflow]
default_execution_timeout = "1h"
default_run_timeout = "30m"
default_task_timeout = "10s"
```

## Monitoring and Debugging

### 1. Temporal Web UI

Access the Temporal Web UI at `http://localhost:8088` to:
- Monitor workflow executions
- View workflow history and events
- Debug failed workflows
- Inspect activity details
- Search workflows by attributes

### 2. Command Line Tools

Use the Temporal CLI for advanced operations:

```bash
# Access Temporal CLI
docker exec -it temporal tctl

# List workflows
tctl workflow list --namespace adx-core-development

# Describe a workflow
tctl workflow describe --workflow_id user-123-onboarding --namespace adx-core-development

# Show workflow history
tctl workflow show --workflow_id user-123-onboarding --namespace adx-core-development

# Cancel a workflow
tctl workflow cancel --workflow_id user-123-onboarding --namespace adx-core-development
```

### 3. Logging and Metrics

ADX Core provides structured logging for Temporal operations:

```rust
use tracing::{info, warn, error};

// Log workflow start
info!(
    workflow_id = %workflow_id,
    workflow_type = %workflow_type,
    user_id = %user_context.user_id,
    tenant_id = %tenant_context.tenant_id,
    "Starting workflow execution"
);

// Log activity execution
info!(
    activity_type = %activity_type,
    workflow_id = %context.workflow_id,
    attempt = context.attempt,
    "Executing activity"
);
```

## Testing

### 1. Unit Testing Workflows

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use temporal_sdk_core_test_utils::TestWorkflowEnvironment;
    
    #[tokio::test]
    async fn test_user_onboarding_workflow() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock activities
        let mock_activities = MockUserActivities::new()
            .expect_validate_user_data()
            .returning(|_| Ok(ValidationResult { is_valid: true, user_id: "user-123".to_string() }))
            .expect_create_user_account()
            .returning(|_| Ok(User { id: "user-123".to_string(), email: "test@example.com".to_string() }));
        
        // Execute workflow
        let result = test_env.execute_workflow(
            user_onboarding_workflow,
            UserOnboardingRequest {
                user_id: "user-123".to_string(),
                email: "test@example.com".to_string(),
                tenant_id: "tenant-456".to_string(),
            },
        ).await;
        
        assert!(result.is_ok());
        let workflow_result = result.unwrap();
        assert_eq!(workflow_result.user_id, "user-123");
        assert!(workflow_result.success);
    }
}
```

### 2. Integration Testing

```rust
#[tokio::test]
async fn test_workflow_integration() {
    let test_env = IntegrationTestEnvironment::new().await;
    
    // Start workflow through API Gateway
    let response = test_env.api_client
        .post("/api/v1/workflows/user-onboarding")
        .json(&UserOnboardingRequest {
            user_id: "integration-test-user".to_string(),
            email: "integration@test.com".to_string(),
            tenant_id: "test-tenant".to_string(),
        })
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 202); // Accepted
    
    let workflow_response: WorkflowApiResponse<UserOnboardingResult> = response.json().await.unwrap();
    
    match workflow_response {
        WorkflowApiResponse::Asynchronous { operation_id, .. } => {
            // Poll for completion
            let final_result = test_env.poll_workflow_completion(&operation_id).await;
            assert!(final_result.is_ok());
        }
        WorkflowApiResponse::Synchronous { data, .. } => {
            assert!(data.success);
        }
    }
}
```

## Troubleshooting

### Common Issues

#### 1. Temporal Server Not Starting

```bash
# Check container logs
docker-compose -f infrastructure/docker/docker-compose.temporal.yml logs temporal

# Common solutions:
# - Ensure PostgreSQL is running and accessible
# - Check port conflicts (7233, 8088)
# - Verify Docker has enough resources allocated
```

#### 2. Namespace Not Found

```bash
# List available namespaces
docker exec temporal tctl namespace list

# Create missing namespace
docker exec temporal tctl namespace register --namespace adx-core-development --retention 72h
```

#### 3. Workflow Execution Failures

```bash
# Check workflow details in Web UI
open http://localhost:8088

# Or use CLI
docker exec temporal tctl workflow show --workflow_id <workflow-id> --namespace adx-core-development
```

#### 4. Activity Timeouts

Check activity timeout configurations:

```rust
// Increase activity timeout
let options = ActivityExecutionOptions {
    start_to_close_timeout: Some(Duration::from_secs(600)), // 10 minutes
    heartbeat_timeout: Some(Duration::from_secs(60)), // 1 minute
    ..Default::default()
};
```

### Performance Tuning

#### 1. Worker Configuration

```rust
let worker_config = WorkerConfig {
    max_concurrent_workflow_tasks: 200,
    max_concurrent_activity_tasks: 500,
    enable_sticky_execution: true,
    sticky_schedule_to_start_timeout: Duration::from_secs(5),
    ..Default::default()
};
```

#### 2. Connection Pooling

```rust
let connection_config = ConnectionConfig {
    max_concurrent_connections: 200,
    keep_alive_timeout: Duration::from_secs(60),
    keep_alive_interval: Duration::from_secs(30),
    ..Default::default()
};
```

## Production Deployment

### 1. Infrastructure Requirements

- **Temporal Server**: Clustered deployment with load balancing
- **Database**: PostgreSQL with high availability and backup
- **Monitoring**: Prometheus metrics and Grafana dashboards
- **Security**: TLS encryption and authentication

### 2. Configuration

```rust
let production_config = TemporalConfig {
    server_address: "temporal-cluster.internal:7233".to_string(),
    namespace: "adx-core-production".to_string(),
    connection: ConnectionConfig {
        enable_tls: true,
        tls: Some(TlsConfig {
            server_name: "temporal-cluster.internal".to_string(),
            client_cert_path: Some("/etc/temporal/certs/client.crt".to_string()),
            client_key_path: Some("/etc/temporal/certs/client.key".to_string()),
            ca_cert_path: Some("/etc/temporal/certs/ca.crt".to_string()),
            insecure_skip_verify: false,
        }),
        max_concurrent_connections: 500,
        ..Default::default()
    },
    worker: WorkerConfig {
        max_concurrent_workflow_tasks: 500,
        max_concurrent_activity_tasks: 1000,
        ..Default::default()
    },
    ..Default::default()
};
```

## Best Practices

### 1. Workflow Design
- Keep workflows deterministic
- Use activities for non-deterministic operations
- Implement proper error handling and compensation
- Use appropriate timeouts and retry policies

### 2. Activity Implementation
- Make activities idempotent
- Use heartbeats for long-running activities
- Implement proper error classification
- Handle tenant isolation correctly

### 3. Testing
- Write comprehensive unit tests for workflows
- Use replay testing for workflow evolution
- Test error scenarios and compensation logic
- Perform load testing with realistic scenarios

### 4. Monitoring
- Monitor workflow success rates and durations
- Set up alerts for workflow failures
- Track activity performance metrics
- Monitor resource usage and scaling needs

## Additional Resources

- [Temporal Documentation](https://docs.temporal.io/)
- [ADX Core Workflow Versioning Strategy](./temporal-workflow-versioning-strategy.md)
- [ADX Core Architecture Overview](../README.md)
- [Development Guidelines](./development-guidelines.md)

## Support

For issues with Temporal setup or development:

1. Check the troubleshooting section above
2. Review Temporal Web UI for workflow details
3. Check container logs for infrastructure issues
4. Consult the ADX Core development team

Remember: Temporal is the backbone of ADX Core's reliability and observability. Proper setup and understanding of Temporal concepts is crucial for successful development and operation of the platform.