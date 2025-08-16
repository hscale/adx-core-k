// Mock implementations for testing ADX CORE services
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Mock database repository trait
#[async_trait]
pub trait MockRepository<T>: Send + Sync {
    async fn create(&self, entity: T) -> Result<T, MockError>;
    async fn find_by_id(&self, id: &str) -> Result<Option<T>, MockError>;
    async fn update(&self, id: &str, entity: T) -> Result<T, MockError>;
    async fn delete(&self, id: &str) -> Result<(), MockError>;
    async fn list(&self) -> Result<Vec<T>, MockError>;
}

/// Mock error type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MockError {
    NotFound(String),
    ValidationError(String),
    DatabaseError(String),
    NetworkError(String),
    Timeout,
}

impl std::fmt::Display for MockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MockError::NotFound(msg) => write!(f, "Not found: {}", msg),
            MockError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            MockError::DatabaseError(msg) => write!(f, "Database error: {}", msg),
            MockError::NetworkError(msg) => write!(f, "Network error: {}", msg),
            MockError::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for MockError {}

/// In-memory mock repository implementation
pub struct InMemoryMockRepository<T> {
    data: Arc<Mutex<HashMap<String, T>>>,
    should_fail: Arc<Mutex<Option<MockError>>>,
    call_count: Arc<Mutex<HashMap<String, usize>>>,
}

impl<T> InMemoryMockRepository<T>
where
    T: Clone + Send + Sync + 'static,
{
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(HashMap::new())),
            should_fail: Arc::new(Mutex::new(None)),
            call_count: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Configure the mock to fail with a specific error
    pub fn set_failure(&self, error: MockError) {
        *self.should_fail.lock().unwrap() = Some(error);
    }
    
    /// Clear any configured failure
    pub fn clear_failure(&self) {
        *self.should_fail.lock().unwrap() = None;
    }
    
    /// Get the number of times a method was called
    pub fn get_call_count(&self, method: &str) -> usize {
        self.call_count.lock().unwrap().get(method).copied().unwrap_or(0)
    }
    
    /// Reset all call counts
    pub fn reset_call_counts(&self) {
        self.call_count.lock().unwrap().clear();
    }
    
    /// Increment call count for a method
    fn increment_call_count(&self, method: &str) {
        let mut counts = self.call_count.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
    
    /// Check if should fail and return error
    fn check_failure(&self) -> Result<(), MockError> {
        if let Some(error) = self.should_fail.lock().unwrap().clone() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<T> MockRepository<T> for InMemoryMockRepository<T>
where
    T: Clone + Send + Sync + 'static,
{
    async fn create(&self, entity: T) -> Result<T, MockError> {
        self.increment_call_count("create");
        self.check_failure()?;
        
        let id = Uuid::new_v4().to_string();
        self.data.lock().unwrap().insert(id, entity.clone());
        Ok(entity)
    }
    
    async fn find_by_id(&self, id: &str) -> Result<Option<T>, MockError> {
        self.increment_call_count("find_by_id");
        self.check_failure()?;
        
        Ok(self.data.lock().unwrap().get(id).cloned())
    }
    
    async fn update(&self, id: &str, entity: T) -> Result<T, MockError> {
        self.increment_call_count("update");
        self.check_failure()?;
        
        let mut data = self.data.lock().unwrap();
        if data.contains_key(id) {
            data.insert(id.to_string(), entity.clone());
            Ok(entity)
        } else {
            Err(MockError::NotFound(format!("Entity with id {} not found", id)))
        }
    }
    
    async fn delete(&self, id: &str) -> Result<(), MockError> {
        self.increment_call_count("delete");
        self.check_failure()?;
        
        let mut data = self.data.lock().unwrap();
        if data.remove(id).is_some() {
            Ok(())
        } else {
            Err(MockError::NotFound(format!("Entity with id {} not found", id)))
        }
    }
    
    async fn list(&self) -> Result<Vec<T>, MockError> {
        self.increment_call_count("list");
        self.check_failure()?;
        
        Ok(self.data.lock().unwrap().values().cloned().collect())
    }
}

/// Mock Temporal client for testing workflows
pub struct MockTemporalClient {
    workflows: Arc<Mutex<HashMap<String, MockWorkflowExecution>>>,
    should_fail: Arc<Mutex<Option<MockError>>>,
    call_count: Arc<Mutex<HashMap<String, usize>>>,
}

#[derive(Debug, Clone)]
pub struct MockWorkflowExecution {
    pub workflow_id: String,
    pub workflow_type: String,
    pub status: WorkflowStatus,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowStatus {
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

impl MockTemporalClient {
    pub fn new() -> Self {
        Self {
            workflows: Arc::new(Mutex::new(HashMap::new())),
            should_fail: Arc::new(Mutex::new(None)),
            call_count: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Start a mock workflow
    pub async fn start_workflow(
        &self,
        workflow_type: &str,
        workflow_id: &str,
        input: serde_json::Value,
    ) -> Result<MockWorkflowHandle, MockError> {
        self.increment_call_count("start_workflow");
        self.check_failure()?;
        
        let execution = MockWorkflowExecution {
            workflow_id: workflow_id.to_string(),
            workflow_type: workflow_type.to_string(),
            status: WorkflowStatus::Running,
            result: None,
            error: None,
            started_at: Utc::now(),
            completed_at: None,
        };
        
        self.workflows.lock().unwrap().insert(workflow_id.to_string(), execution);
        
        Ok(MockWorkflowHandle {
            workflow_id: workflow_id.to_string(),
            client: Arc::new(self.clone()),
        })
    }
    
    /// Get workflow execution info
    pub async fn get_workflow_execution(
        &self,
        workflow_id: &str,
    ) -> Result<MockWorkflowExecution, MockError> {
        self.increment_call_count("get_workflow_execution");
        self.check_failure()?;
        
        self.workflows
            .lock()
            .unwrap()
            .get(workflow_id)
            .cloned()
            .ok_or_else(|| MockError::NotFound(format!("Workflow {} not found", workflow_id)))
    }
    
    /// Complete a workflow with result
    pub async fn complete_workflow(
        &self,
        workflow_id: &str,
        result: serde_json::Value,
    ) -> Result<(), MockError> {
        let mut workflows = self.workflows.lock().unwrap();
        if let Some(execution) = workflows.get_mut(workflow_id) {
            execution.status = WorkflowStatus::Completed;
            execution.result = Some(result);
            execution.completed_at = Some(Utc::now());
            Ok(())
        } else {
            Err(MockError::NotFound(format!("Workflow {} not found", workflow_id)))
        }
    }
    
    /// Fail a workflow with error
    pub async fn fail_workflow(
        &self,
        workflow_id: &str,
        error: &str,
    ) -> Result<(), MockError> {
        let mut workflows = self.workflows.lock().unwrap();
        if let Some(execution) = workflows.get_mut(workflow_id) {
            execution.status = WorkflowStatus::Failed;
            execution.error = Some(error.to_string());
            execution.completed_at = Some(Utc::now());
            Ok(())
        } else {
            Err(MockError::NotFound(format!("Workflow {} not found", workflow_id)))
        }
    }
    
    /// Set failure mode
    pub fn set_failure(&self, error: MockError) {
        *self.should_fail.lock().unwrap() = Some(error);
    }
    
    /// Clear failure mode
    pub fn clear_failure(&self) {
        *self.should_fail.lock().unwrap() = None;
    }
    
    /// Get call count for method
    pub fn get_call_count(&self, method: &str) -> usize {
        self.call_count.lock().unwrap().get(method).copied().unwrap_or(0)
    }
    
    fn increment_call_count(&self, method: &str) {
        let mut counts = self.call_count.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
    
    fn check_failure(&self) -> Result<(), MockError> {
        if let Some(error) = self.should_fail.lock().unwrap().clone() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

impl Clone for MockTemporalClient {
    fn clone(&self) -> Self {
        Self {
            workflows: self.workflows.clone(),
            should_fail: self.should_fail.clone(),
            call_count: self.call_count.clone(),
        }
    }
}

/// Mock workflow handle
pub struct MockWorkflowHandle {
    workflow_id: String,
    client: Arc<MockTemporalClient>,
}

impl MockWorkflowHandle {
    /// Get workflow result (blocks until completed)
    pub async fn get_result(&self) -> Result<serde_json::Value, MockError> {
        // In a real implementation, this would poll until completion
        // For testing, we'll just check the current status
        let execution = self.client.get_workflow_execution(&self.workflow_id).await?;
        
        match execution.status {
            WorkflowStatus::Completed => {
                execution.result.ok_or_else(|| {
                    MockError::DatabaseError("Workflow completed but no result found".to_string())
                })
            }
            WorkflowStatus::Failed => {
                Err(MockError::DatabaseError(
                    execution.error.unwrap_or_else(|| "Workflow failed".to_string())
                ))
            }
            _ => Err(MockError::DatabaseError("Workflow not completed".to_string())),
        }
    }
    
    /// Cancel the workflow
    pub async fn cancel(&self) -> Result<(), MockError> {
        let mut workflows = self.client.workflows.lock().unwrap();
        if let Some(execution) = workflows.get_mut(&self.workflow_id) {
            execution.status = WorkflowStatus::Cancelled;
            execution.completed_at = Some(Utc::now());
            Ok(())
        } else {
            Err(MockError::NotFound(format!("Workflow {} not found", self.workflow_id)))
        }
    }
}

/// Mock HTTP client for testing external API calls
pub struct MockHttpClient {
    responses: Arc<Mutex<HashMap<String, MockHttpResponse>>>,
    should_fail: Arc<Mutex<Option<MockError>>>,
    call_count: Arc<Mutex<HashMap<String, usize>>>,
}

#[derive(Debug, Clone)]
pub struct MockHttpResponse {
    pub status: u16,
    pub body: String,
    pub headers: HashMap<String, String>,
}

impl MockHttpClient {
    pub fn new() -> Self {
        Self {
            responses: Arc::new(Mutex::new(HashMap::new())),
            should_fail: Arc::new(Mutex::new(None)),
            call_count: Arc::new(Mutex::new(HashMap::new())),
        }
    }
    
    /// Set a mock response for a URL
    pub fn set_response(&self, url: &str, response: MockHttpResponse) {
        self.responses.lock().unwrap().insert(url.to_string(), response);
    }
    
    /// Make a mock HTTP request
    pub async fn get(&self, url: &str) -> Result<MockHttpResponse, MockError> {
        self.increment_call_count("get");
        self.check_failure()?;
        
        self.responses
            .lock()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| MockError::NotFound(format!("No mock response for URL: {}", url)))
    }
    
    /// Make a mock HTTP POST request
    pub async fn post(&self, url: &str, _body: &str) -> Result<MockHttpResponse, MockError> {
        self.increment_call_count("post");
        self.check_failure()?;
        
        self.responses
            .lock()
            .unwrap()
            .get(url)
            .cloned()
            .ok_or_else(|| MockError::NotFound(format!("No mock response for URL: {}", url)))
    }
    
    /// Set failure mode
    pub fn set_failure(&self, error: MockError) {
        *self.should_fail.lock().unwrap() = Some(error);
    }
    
    /// Clear failure mode
    pub fn clear_failure(&self) {
        *self.should_fail.lock().unwrap() = None;
    }
    
    /// Get call count
    pub fn get_call_count(&self, method: &str) -> usize {
        self.call_count.lock().unwrap().get(method).copied().unwrap_or(0)
    }
    
    fn increment_call_count(&self, method: &str) {
        let mut counts = self.call_count.lock().unwrap();
        *counts.entry(method.to_string()).or_insert(0) += 1;
    }
    
    fn check_failure(&self) -> Result<(), MockError> {
        if let Some(error) = self.should_fail.lock().unwrap().clone() {
            Err(error)
        } else {
            Ok(())
        }
    }
}

/// Utility macros for creating mocks
#[macro_export]
macro_rules! mock_repository {
    ($name:ident, $entity_type:ty) => {
        pub struct $name {
            inner: InMemoryMockRepository<$entity_type>,
        }
        
        impl $name {
            pub fn new() -> Self {
                Self {
                    inner: InMemoryMockRepository::new(),
                }
            }
            
            pub fn set_failure(&self, error: MockError) {
                self.inner.set_failure(error);
            }
            
            pub fn clear_failure(&self) {
                self.inner.clear_failure();
            }
            
            pub fn get_call_count(&self, method: &str) -> usize {
                self.inner.get_call_count(method)
            }
        }
        
        #[async_trait]
        impl MockRepository<$entity_type> for $name {
            async fn create(&self, entity: $entity_type) -> Result<$entity_type, MockError> {
                self.inner.create(entity).await
            }
            
            async fn find_by_id(&self, id: &str) -> Result<Option<$entity_type>, MockError> {
                self.inner.find_by_id(id).await
            }
            
            async fn update(&self, id: &str, entity: $entity_type) -> Result<$entity_type, MockError> {
                self.inner.update(id, entity).await
            }
            
            async fn delete(&self, id: &str) -> Result<(), MockError> {
                self.inner.delete(id).await
            }
            
            async fn list(&self) -> Result<Vec<$entity_type>, MockError> {
                self.inner.list().await
            }
        }
    };
}