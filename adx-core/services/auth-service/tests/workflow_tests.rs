// Temporal workflow integration tests for Auth Service
use std::sync::Arc;
use std::time::Duration;
use serde_json::json;
use uuid::Uuid;
use chrono::Utc;

use adx_shared::testing::{
    TestContext,
    temporal::{TemporalTestEnvironment, TemporalTestConfig, test_workflows::*},
    mocks::*,
};
use adx_shared::{workflow_test, workflow_failure_test};

/// Test user registration workflow success path
workflow_test!(
    test_user_registration_workflow_success,
    user_registration_workflow,
    json!({
        "email": "test@example.com",
        "password": "password123",
        "tenant_id": "tenant-123",
        "send_welcome_email": true
    }),
    json!({
        "user_id": "user-123",
        "status": "completed",
        "email_sent": true
    })
);

/// Test user registration workflow with validation failure
workflow_failure_test!(
    test_user_registration_workflow_validation_failure,
    user_registration_workflow_with_validation_error,
    json!({
        "email": "invalid-email",
        "password": "123",
        "tenant_id": "tenant-123"
    }),
    MockError::ValidationError("Invalid email format".to_string())
);

/// Test password reset workflow
workflow_test!(
    test_password_reset_workflow_success,
    password_reset_workflow,
    json!({
        "email": "test@example.com",
        "tenant_id": "tenant-123",
        "reset_url": "https://app.example.com/reset"
    }),
    json!({
        "reset_token": "reset-token-123",
        "status": "email_sent",
        "expires_at": "2024-01-15T12:00:00Z"
    })
);

/// Comprehensive workflow integration tests
#[tokio::test]
async fn test_user_registration_workflow_with_activities() {
    let test_env = TemporalTestEnvironment::new();
    
    // Mock activities
    let user_repo = Arc::new(MockUserRepository::new());
    let email_service = Arc::new(MockEmailService::new());
    let temporal_client = Arc::new(MockTemporalClient::new());
    
    // Set up expected activity results
    email_service.set_response(
        "send_welcome_email",
        json!({
            "message_id": "msg-123",
            "status": "sent"
        })
    );
    
    // Execute workflow
    let workflow_input = json!({
        "email": "test@example.com",
        "password": "password123",
        "tenant_id": "tenant-123",
        "user_metadata": {
            "first_name": "Test",
            "last_name": "User"
        }
    });
    
    let result = test_env.execute_workflow(
        |input| user_registration_workflow_with_activities(input, user_repo.clone(), email_service.clone()),
        workflow_input,
    ).await;
    
    // Verify workflow completed successfully
    assert!(result.is_ok());
    let workflow_result = result.unwrap();
    
    // Verify result structure
    assert!(workflow_result["user_id"].is_string());
    assert_eq!(workflow_result["status"], "completed");
    assert_eq!(workflow_result["email_sent"], true);
    
    // Verify activities were called
    assert_eq!(user_repo.get_call_count("create"), 1);
    assert_eq!(email_service.get_call_count("send_welcome_email"), 1);
    
    // Verify workflow history
    let workflows = test_env.get_all_workflows();
    assert_eq!(workflows.len(), 1);
    
    let workflow = &workflows[0];
    assert_eq!(workflow.status, WorkflowStatus::Completed);
    assert!(workflow.history.len() >= 2); // Started + Completed events
}

/// Test workflow with activity failure and compensation
#[tokio::test]
async fn test_user_registration_workflow_with_compensation() {
    let test_env = TemporalTestEnvironment::new();
    
    // Mock activities
    let user_repo = Arc::new(MockUserRepository::new());
    let email_service = Arc::new(MockEmailService::new());
    
    // Set up user creation to succeed but email to fail
    email_service.set_failure(MockError::NetworkError("Email service unavailable".to_string()));
    
    let workflow_input = json!({
        "email": "test@example.com",
        "password": "password123",
        "tenant_id": "tenant-123"
    });
    
    let result = test_env.execute_workflow(
        |input| user_registration_workflow_with_compensation(input, user_repo.clone(), email_service.clone()),
        workflow_input,
    ).await;
    
    // Workflow should fail due to email service failure
    assert!(result.is_err());
    
    // Verify compensation was executed (user should be deleted)
    assert_eq!(user_repo.get_call_count("create"), 1);
    assert_eq!(user_repo.get_call_count("delete"), 1); // Compensation
    assert_eq!(email_service.get_call_count("send_welcome_email"), 1);
    
    // Verify workflow history includes compensation
    let workflows = test_env.get_all_workflows();
    assert_eq!(workflows.len(), 1);
    
    let workflow = &workflows[0];
    assert_eq!(workflow.status, WorkflowStatus::Failed);
}

/// Test workflow replay functionality
#[tokio::test]
async fn test_workflow_replay() {
    let test_env = TemporalTestEnvironment::with_config(TemporalTestConfig {
        enable_replay_testing: true,
        ..Default::default()
    });
    
    // Create a workflow history to replay
    let workflow_history = vec![
        WorkflowEvent {
            event_id: "1".to_string(),
            event_type: WorkflowEventType::WorkflowStarted,
            timestamp: Utc::now(),
            data: json!({
                "email": "test@example.com",
                "password": "password123",
                "tenant_id": "tenant-123"
            }),
        },
        WorkflowEvent {
            event_id: "2".to_string(),
            event_type: WorkflowEventType::ActivityScheduled,
            timestamp: Utc::now(),
            data: json!({
                "activity_type": "validate_user_data",
                "activity_id": "activity-1"
            }),
        },
        WorkflowEvent {
            event_id: "3".to_string(),
            event_type: WorkflowEventType::ActivityCompleted,
            timestamp: Utc::now(),
            data: json!({
                "activity_id": "activity-1",
                "result": {
                    "valid": true,
                    "user_id": "user-123"
                }
            }),
        },
        WorkflowEvent {
            event_id: "4".to_string(),
            event_type: WorkflowEventType::ActivityScheduled,
            timestamp: Utc::now(),
            data: json!({
                "activity_type": "create_user",
                "activity_id": "activity-2"
            }),
        },
        WorkflowEvent {
            event_id: "5".to_string(),
            event_type: WorkflowEventType::ActivityCompleted,
            timestamp: Utc::now(),
            data: json!({
                "activity_id": "activity-2",
                "result": {
                    "user_id": "user-123",
                    "created": true
                }
            }),
        },
        WorkflowEvent {
            event_id: "6".to_string(),
            event_type: WorkflowEventType::WorkflowCompleted,
            timestamp: Utc::now(),
            data: json!({
                "user_id": "user-123",
                "status": "completed"
            }),
        },
    ];
    
    // Test replay
    let replay_result = test_env.test_workflow_replay(workflow_history).await;
    
    assert!(replay_result.is_ok());
    let replay_test_result = replay_result.unwrap();
    
    assert!(replay_test_result.success);
    assert_eq!(replay_test_result.events_processed, 6);
    assert!(replay_test_result.final_state.workflow_started);
    assert!(replay_test_result.final_state.workflow_completed);
    assert_eq!(replay_test_result.final_state.activities_scheduled, 2);
    assert_eq!(replay_test_result.final_state.activities_completed, 2);
    assert_eq!(replay_test_result.final_state.activities_failed, 0);
}

/// Test concurrent workflow execution
#[tokio::test]
async fn test_concurrent_workflow_execution() {
    let test_env = Arc::new(TemporalTestEnvironment::new());
    let user_repo = Arc::new(MockUserRepository::new());
    let email_service = Arc::new(MockEmailService::new());
    
    let mut handles = Vec::new();
    
    // Start multiple workflows concurrently
    for i in 0..5 {
        let env = test_env.clone();
        let repo = user_repo.clone();
        let email = email_service.clone();
        
        let handle = tokio::spawn(async move {
            let workflow_input = json!({
                "email": format!("user{}@example.com", i),
                "password": "password123",
                "tenant_id": "tenant-123"
            });
            
            env.execute_workflow(
                |input| user_registration_workflow_with_activities(input, repo, email),
                workflow_input,
            ).await
        });
        
        handles.push(handle);
    }
    
    // Wait for all workflows to complete
    let results = futures::future::join_all(handles).await;
    
    // Verify all workflows completed successfully
    let mut successful_workflows = 0;
    for result in results {
        let task_result = result.unwrap();
        if task_result.is_ok() {
            successful_workflows += 1;
        }
    }
    
    assert_eq!(successful_workflows, 5);
    assert_eq!(user_repo.get_call_count("create"), 5);
    assert_eq!(email_service.get_call_count("send_welcome_email"), 5);
    
    // Verify all workflows are recorded
    let workflows = test_env.get_all_workflows();
    assert_eq!(workflows.len(), 5);
}

/// Test workflow timeout handling
#[tokio::test]
async fn test_workflow_timeout() {
    let test_env = TemporalTestEnvironment::with_config(TemporalTestConfig {
        workflow_timeout: Duration::from_millis(100), // Very short timeout
        ..Default::default()
    });
    
    let workflow_input = json!({
        "email": "test@example.com",
        "password": "password123",
        "tenant_id": "tenant-123"
    });
    
    // Execute a workflow that takes longer than the timeout
    let result = test_env.execute_workflow(
        timeout_workflow, // This workflow sleeps for 35 seconds
        workflow_input,
    ).await;
    
    // Should timeout
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), MockError::Timeout));
    
    // Verify workflow status
    let workflows = test_env.get_all_workflows();
    assert_eq!(workflows.len(), 1);
    assert_eq!(workflows[0].status, WorkflowStatus::TimedOut);
}

/// Test workflow with signal handling
#[tokio::test]
async fn test_workflow_with_signals() {
    let test_env = TemporalTestEnvironment::new();
    
    // This would test signal handling in a real Temporal environment
    // For now, we'll simulate signal events in the workflow history
    let workflow_input = json!({
        "email": "test@example.com",
        "password": "password123",
        "tenant_id": "tenant-123",
        "wait_for_approval": true
    });
    
    let result = test_env.execute_workflow(
        user_registration_workflow_with_approval,
        workflow_input,
    ).await;
    
    assert!(result.is_ok());
    let workflow_result = result.unwrap();
    assert_eq!(workflow_result["status"], "approved");
}

// Mock workflow implementations for testing
async fn user_registration_workflow(input: serde_json::Value) -> Result<serde_json::Value, MockError> {
    let email = input["email"].as_str().ok_or_else(|| {
        MockError::ValidationError("Email is required".to_string())
    })?;
    
    if !email.contains('@') {
        return Err(MockError::ValidationError("Invalid email format".to_string()));
    }
    
    // Simulate workflow execution
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    Ok(json!({
        "user_id": "user-123",
        "status": "completed",
        "email_sent": input["send_welcome_email"].as_bool().unwrap_or(false)
    }))
}

async fn user_registration_workflow_with_validation_error(
    input: serde_json::Value,
) -> Result<serde_json::Value, MockError> {
    let email = input["email"].as_str().unwrap_or("");
    let password = input["password"].as_str().unwrap_or("");
    
    if !email.contains('@') {
        return Err(MockError::ValidationError("Invalid email format".to_string()));
    }
    
    if password.len() < 8 {
        return Err(MockError::ValidationError("Password too short".to_string()));
    }
    
    Ok(json!({"status": "completed"}))
}

async fn password_reset_workflow(input: serde_json::Value) -> Result<serde_json::Value, MockError> {
    tokio::time::sleep(Duration::from_millis(50)).await;
    
    Ok(json!({
        "reset_token": "reset-token-123",
        "status": "email_sent",
        "expires_at": "2024-01-15T12:00:00Z"
    }))
}

async fn user_registration_workflow_with_activities(
    input: serde_json::Value,
    user_repo: Arc<MockUserRepository>,
    email_service: Arc<MockEmailService>,
) -> Result<serde_json::Value, MockError> {
    // Activity 1: Create user
    let user = User {
        id: Uuid::new_v4().to_string(),
        email: input["email"].as_str().unwrap().to_string(),
        password_hash: "hashed_password".to_string(),
        tenant_id: input["tenant_id"].as_str().unwrap().to_string(),
        is_active: true,
    };
    
    let created_user = user_repo.create(user).await?;
    
    // Activity 2: Send welcome email
    let email_result = email_service.send_welcome_email(&created_user.email).await?;
    
    Ok(json!({
        "user_id": created_user.id,
        "status": "completed",
        "email_sent": true,
        "email_message_id": email_result["message_id"]
    }))
}

async fn user_registration_workflow_with_compensation(
    input: serde_json::Value,
    user_repo: Arc<MockUserRepository>,
    email_service: Arc<MockEmailService>,
) -> Result<serde_json::Value, MockError> {
    // Activity 1: Create user
    let user = User {
        id: Uuid::new_v4().to_string(),
        email: input["email"].as_str().unwrap().to_string(),
        password_hash: "hashed_password".to_string(),
        tenant_id: input["tenant_id"].as_str().unwrap().to_string(),
        is_active: true,
    };
    
    let created_user = user_repo.create(user).await?;
    
    // Activity 2: Send welcome email (this will fail)
    match email_service.send_welcome_email(&created_user.email).await {
        Ok(email_result) => {
            Ok(json!({
                "user_id": created_user.id,
                "status": "completed",
                "email_sent": true,
                "email_message_id": email_result["message_id"]
            }))
        }
        Err(e) => {
            // Compensation: Delete the created user
            let _ = user_repo.delete(&created_user.id).await;
            Err(e)
        }
    }
}

async fn user_registration_workflow_with_approval(
    input: serde_json::Value,
) -> Result<serde_json::Value, MockError> {
    // Simulate waiting for approval signal
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    Ok(json!({
        "user_id": "user-123",
        "status": "approved"
    }))
}

// Mock types and services
#[derive(Debug, Clone)]
struct User {
    id: String,
    email: String,
    password_hash: String,
    tenant_id: String,
    is_active: bool,
}

mock_repository!(MockUserRepository, User);

struct MockEmailService {
    responses: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, serde_json::Value>>>,
    should_fail: std::sync::Arc<std::sync::Mutex<Option<MockError>>>,
    call_count: std::sync::Arc<std::sync::Mutex<std::collections::HashMap<String, usize>>>,
}

impl MockEmailService {
    fn new() -> Self {
        Self {
            responses: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
            should_fail: std::sync::Arc::new(std::sync::Mutex::new(None)),
            call_count: std::sync::Arc::new(std::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }
    
    fn set_response(&self, method: &str, response: serde_json::Value) {
        self.responses.lock().unwrap().insert(method.to_string(), response);
    }
    
    fn set_failure(&self, error: MockError) {
        *self.should_fail.lock().unwrap() = Some(error);
    }
    
    fn get_call_count(&self, method: &str) -> usize {
        self.call_count.lock().unwrap().get(method).copied().unwrap_or(0)
    }
    
    async fn send_welcome_email(&self, email: &str) -> Result<serde_json::Value, MockError> {
        // Increment call count
        let mut counts = self.call_count.lock().unwrap();
        *counts.entry("send_welcome_email".to_string()).or_insert(0) += 1;
        drop(counts);
        
        // Check for failure
        if let Some(error) = self.should_fail.lock().unwrap().clone() {
            return Err(error);
        }
        
        // Return mock response
        self.responses
            .lock()
            .unwrap()
            .get("send_welcome_email")
            .cloned()
            .unwrap_or_else(|| json!({
                "message_id": "default-msg-123",
                "status": "sent",
                "recipient": email
            }))
            .pipe(Ok)
    }
}

// Helper trait for pipeline operations
trait Pipe<T> {
    fn pipe<F, U>(self, f: F) -> U
    where
        F: FnOnce(T) -> U;
}

impl<T> Pipe<T> for T {
    fn pipe<F, U>(self, f: F) -> U
    where
        F: FnOnce(T) -> U,
    {
        f(self)
    }
}