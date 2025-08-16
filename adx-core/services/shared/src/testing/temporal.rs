// Temporal testing utilities for ADX CORE
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use tokio::time::sleep;

use super::mocks::{MockError, WorkflowStatus};

/// Test environment for Temporal workflows
pub struct TemporalTestEnvironment {
    workflows: Arc<Mutex<HashMap<String, WorkflowExecution>>>,
    activities: Arc<Mutex<HashMap<String, ActivityExecution>>>,
    config: TemporalTestConfig,
}

#[derive(Debug, Clone)]
pub struct TemporalTestConfig {
    pub default_timeout: Duration,
    pub activity_timeout: Duration,
    pub workflow_timeout: Duration,
    pub enable_replay_testing: bool,
}

impl Default for TemporalTestConfig {
    fn default() -> Self {
        Self {
            default_timeout: Duration::from_secs(30),
            activity_timeout: Duration::from_secs(10),
            workflow_timeout: Duration::from_secs(60),
            enable_replay_testing: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WorkflowExecution {
    pub workflow_id: String,
    pub workflow_type: String,
    pub status: WorkflowStatus,
    pub input: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub activities: Vec<ActivityExecution>,
    pub history: Vec<WorkflowEvent>,
}

#[derive(Debug, Clone)]
pub struct ActivityExecution {
    pub activity_id: String,
    pub activity_type: String,
    pub status: ActivityStatus,
    pub input: serde_json::Value,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub retry_count: u32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ActivityStatus {
    Scheduled,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowEvent {
    pub event_id: String,
    pub event_type: WorkflowEventType,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowEventType {
    WorkflowStarted,
    WorkflowCompleted,
    WorkflowFailed,
    WorkflowCancelled,
    ActivityScheduled,
    ActivityStarted,
    ActivityCompleted,
    ActivityFailed,
    ActivityRetried,
    TimerStarted,
    TimerFired,
    SignalReceived,
}

impl TemporalTestEnvironment {
    pub fn new() -> Self {
        Self::with_config(TemporalTestConfig::default())
    }
    
    pub fn with_config(config: TemporalTestConfig) -> Self {
        Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            activities: Arc::new(Mutex::new(HashMap::new())),
            config,
        }
    }
    
    /// Execute a workflow in the test environment
    pub async fn execute_workflow<T, R>(
        &self,
        workflow_fn: impl Fn(T) -> R + Send + 'static,
        input: T,
    ) -> Result<serde_json::Value, MockError>
    where
        T: Serialize + Send + 'static,
        R: std::future::Future<Output = Result<serde_json::Value, MockError>> + Send + 'static,
    {
        let workflow_id = Uuid::new_v4().to_string();
        let workflow_type = std::any::type_name::<T>().to_string();
        
        // Create workflow execution record
        let execution = WorkflowExecution {
            workflow_id: workflow_id.clone(),
            workflow_type: workflow_type.clone(),
            status: WorkflowStatus::Running,
            input: serde_json::to_value(&input).unwrap(),
            result: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
            activities: Vec::new(),
            history: vec![WorkflowEvent {
                event_id: Uuid::new_v4().to_string(),
                event_type: WorkflowEventType::WorkflowStarted,
                timestamp: Utc::now(),
                data: serde_json::to_value(&input).unwrap(),
            }],
        };
        
        self.workflows.lock().unwrap().insert(workflow_id.clone(), execution);
        
        // Execute the workflow function
        let start_time = Instant::now();
        let result = tokio::time::timeout(
            self.config.workflow_timeout,
            workflow_fn(input),
        ).await;
        
        let mut workflows = self.workflows.lock().unwrap();
        let execution = workflows.get_mut(&workflow_id).unwrap();
        
        match result {
            Ok(Ok(workflow_result)) => {
                execution.status = WorkflowStatus::Completed;
                execution.result = Some(workflow_result.clone());
                execution.completed_at = Some(Utc::now());
                execution.history.push(WorkflowEvent {
                    event_id: Uuid::new_v4().to_string(),
                    event_type: WorkflowEventType::WorkflowCompleted,
                    timestamp: Utc::now(),
                    data: workflow_result.clone(),
                });
                Ok(workflow_result)
            }
            Ok(Err(error)) => {
                execution.status = WorkflowStatus::Failed;
                execution.error = Some(error.to_string());
                execution.completed_at = Some(Utc::now());
                execution.history.push(WorkflowEvent {
                    event_id: Uuid::new_v4().to_string(),
                    event_type: WorkflowEventType::WorkflowFailed,
                    timestamp: Utc::now(),
                    data: serde_json::json!({ "error": error.to_string() }),
                });
                Err(error)
            }
            Err(_) => {
                execution.status = WorkflowStatus::TimedOut;
                execution.error = Some("Workflow timed out".to_string());
                execution.completed_at = Some(Utc::now());
                Err(MockError::Timeout)
            }
        }
    }
    
    /// Execute an activity in the test environment
    pub async fn execute_activity<T, R>(
        &self,
        activity_fn: impl Fn(T) -> R + Send + 'static,
        input: T,
    ) -> Result<serde_json::Value, MockError>
    where
        T: Serialize + Send + 'static,
        R: std::future::Future<Output = Result<serde_json::Value, MockError>> + Send + 'static,
    {
        let activity_id = Uuid::new_v4().to_string();
        let activity_type = std::any::type_name::<T>().to_string();
        
        // Create activity execution record
        let execution = ActivityExecution {
            activity_id: activity_id.clone(),
            activity_type: activity_type.clone(),
            status: ActivityStatus::Running,
            input: serde_json::to_value(&input).unwrap(),
            result: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
            retry_count: 0,
        };
        
        self.activities.lock().unwrap().insert(activity_id.clone(), execution);
        
        // Execute the activity function
        let result = tokio::time::timeout(
            self.config.activity_timeout,
            activity_fn(input),
        ).await;
        
        let mut activities = self.activities.lock().unwrap();
        let execution = activities.get_mut(&activity_id).unwrap();
        
        match result {
            Ok(Ok(activity_result)) => {
                execution.status = ActivityStatus::Completed;
                execution.result = Some(activity_result.clone());
                execution.completed_at = Some(Utc::now());
                Ok(activity_result)
            }
            Ok(Err(error)) => {
                execution.status = ActivityStatus::Failed;
                execution.error = Some(error.to_string());
                execution.completed_at = Some(Utc::now());
                Err(error)
            }
            Err(_) => {
                execution.status = ActivityStatus::TimedOut;
                execution.error = Some("Activity timed out".to_string());
                execution.completed_at = Some(Utc::now());
                Err(MockError::Timeout)
            }
        }
    }
    
    /// Test workflow replay functionality
    pub async fn test_workflow_replay(
        &self,
        workflow_history: Vec<WorkflowEvent>,
    ) -> Result<ReplayTestResult, MockError> {
        if !self.config.enable_replay_testing {
            return Err(MockError::ValidationError("Replay testing is disabled".to_string()));
        }
        
        let replay_start = Instant::now();
        
        // Simulate replay by processing events in order
        let mut replay_state = ReplayState::new();
        
        for event in &workflow_history {
            match event.event_type {
                WorkflowEventType::WorkflowStarted => {
                    replay_state.workflow_started = true;
                }
                WorkflowEventType::ActivityScheduled => {
                    replay_state.activities_scheduled += 1;
                }
                WorkflowEventType::ActivityCompleted => {
                    replay_state.activities_completed += 1;
                }
                WorkflowEventType::ActivityFailed => {
                    replay_state.activities_failed += 1;
                }
                WorkflowEventType::WorkflowCompleted => {
                    replay_state.workflow_completed = true;
                }
                WorkflowEventType::WorkflowFailed => {
                    replay_state.workflow_failed = true;
                }
                _ => {}
            }
        }
        
        let replay_duration = replay_start.elapsed();
        
        Ok(ReplayTestResult {
            success: true,
            events_processed: workflow_history.len(),
            replay_duration,
            final_state: replay_state,
            errors: Vec::new(),
        })
    }
    
    /// Get workflow execution history
    pub fn get_workflow_history(&self, workflow_id: &str) -> Option<Vec<WorkflowEvent>> {
        self.workflows
            .lock()
            .unwrap()
            .get(workflow_id)
            .map(|execution| execution.history.clone())
    }
    
    /// Get all workflow executions
    pub fn get_all_workflows(&self) -> Vec<WorkflowExecution> {
        self.workflows.lock().unwrap().values().cloned().collect()
    }
    
    /// Get all activity executions
    pub fn get_all_activities(&self) -> Vec<ActivityExecution> {
        self.activities.lock().unwrap().values().cloned().collect()
    }
    
    /// Clear all execution history
    pub fn clear_history(&self) {
        self.workflows.lock().unwrap().clear();
        self.activities.lock().unwrap().clear();
    }
}

#[derive(Debug, Clone)]
pub struct ReplayState {
    pub workflow_started: bool,
    pub workflow_completed: bool,
    pub workflow_failed: bool,
    pub activities_scheduled: usize,
    pub activities_completed: usize,
    pub activities_failed: usize,
}

impl ReplayState {
    pub fn new() -> Self {
        Self {
            workflow_started: false,
            workflow_completed: false,
            workflow_failed: false,
            activities_scheduled: 0,
            activities_completed: 0,
            activities_failed: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ReplayTestResult {
    pub success: bool,
    pub events_processed: usize,
    pub replay_duration: Duration,
    pub final_state: ReplayState,
    pub errors: Vec<String>,
}

/// Utility functions for creating test workflows
pub mod test_workflows {
    use super::*;
    
    /// Simple test workflow that completes successfully
    pub async fn simple_success_workflow(
        input: serde_json::Value,
    ) -> Result<serde_json::Value, MockError> {
        sleep(Duration::from_millis(100)).await;
        Ok(serde_json::json!({
            "status": "completed",
            "input": input,
            "timestamp": Utc::now()
        }))
    }
    
    /// Test workflow that fails
    pub async fn failing_workflow(
        _input: serde_json::Value,
    ) -> Result<serde_json::Value, MockError> {
        sleep(Duration::from_millis(50)).await;
        Err(MockError::ValidationError("Test workflow failure".to_string()))
    }
    
    /// Test workflow with activities
    pub async fn workflow_with_activities(
        input: serde_json::Value,
    ) -> Result<serde_json::Value, MockError> {
        // Simulate activity execution
        sleep(Duration::from_millis(100)).await;
        
        let activity_result = serde_json::json!({
            "activity_completed": true,
            "input": input
        });
        
        sleep(Duration::from_millis(100)).await;
        
        Ok(serde_json::json!({
            "workflow_completed": true,
            "activity_result": activity_result,
            "timestamp": Utc::now()
        }))
    }
    
    /// Test workflow that times out
    pub async fn timeout_workflow(
        _input: serde_json::Value,
    ) -> Result<serde_json::Value, MockError> {
        // Sleep longer than the default timeout
        sleep(Duration::from_secs(35)).await;
        Ok(serde_json::json!({"status": "should_not_reach_here"}))
    }
}

/// Macros for workflow testing
#[macro_export]
macro_rules! workflow_test {
    ($name:ident, $workflow_fn:expr, $input:expr, $expected_result:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_env = TemporalTestEnvironment::new();
            let result = test_env.execute_workflow($workflow_fn, $input).await;
            
            match result {
                Ok(actual) => {
                    assert_eq!(actual, $expected_result, "Workflow result mismatch");
                }
                Err(e) => panic!("Workflow failed unexpectedly: {:?}", e),
            }
        }
    };
}

#[macro_export]
macro_rules! workflow_failure_test {
    ($name:ident, $workflow_fn:expr, $input:expr, $expected_error:expr) => {
        #[tokio::test]
        async fn $name() {
            let test_env = TemporalTestEnvironment::new();
            let result = test_env.execute_workflow($workflow_fn, $input).await;
            
            match result {
                Ok(actual) => panic!("Expected workflow to fail, but got result: {:?}", actual),
                Err(actual_error) => {
                    assert_eq!(
                        std::mem::discriminant(&actual_error),
                        std::mem::discriminant(&$expected_error),
                        "Error type mismatch"
                    );
                }
            }
        }
    };
}