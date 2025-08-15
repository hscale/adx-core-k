// Mock Temporal SDK for compilation until actual SDK is available
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;

// Mock Temporal Client
#[derive(Clone)]
pub struct Client;

impl Client {
    pub async fn new(_server_url: &str) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self)
    }

    pub async fn start_workflow<T: Serialize>(
        &self,
        _workflow_type: &str,
        _workflow_id: String,
        _task_queue: &str,
        _input: T,
    ) -> Result<WorkflowHandle, Box<dyn std::error::Error>> {
        Ok(WorkflowHandle)
    }
}

// Mock Workflow Handle
pub struct WorkflowHandle;

impl WorkflowHandle {
    pub async fn get_result<T: for<'de> Deserialize<'de>>(&self) -> Result<T, Box<dyn std::error::Error>> {
        Err("Mock implementation".into())
    }
}

// Mock Worker
pub struct Worker;

impl Worker {
    pub async fn register_workflow<F, Fut, T, R>(&mut self, _workflow_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(WfContext, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, crate::error::WhiteLabelError>> + Send + 'static,
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
    {
        Ok(())
    }

    pub async fn register_activity<F, Fut, T, R>(&mut self, _activity_fn: F) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, crate::error::WhiteLabelError>> + Send + 'static,
        T: for<'de> Deserialize<'de> + Send + 'static,
        R: Serialize + Send + 'static,
    {
        Ok(())
    }

    pub async fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Mock implementation - would run indefinitely in real implementation
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        Ok(())
    }
}

// Mock WorkerBuilder
pub struct WorkerBuilder;

impl WorkerBuilder {
    pub fn default() -> Self {
        Self
    }

    pub fn task_queue(self, _task_queue: &str) -> Self {
        self
    }

    pub async fn build(self) -> Result<Worker, Box<dyn std::error::Error>> {
        Ok(Worker)
    }
}

// Mock Workflow Context
pub struct WfContext;

impl WfContext {
    pub async fn activity<T: Serialize, R: for<'de> Deserialize<'de>>(
        &self,
        _options: ActivityOptions,
    ) -> ActivityBuilder<T, R> {
        ActivityBuilder::new()
    }

    pub async fn sleep(&self, _duration: chrono::Duration) {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

// Mock Activity Options
#[derive(Default)]
pub struct ActivityOptions;

// Mock Activity Builder
pub struct ActivityBuilder<T, R> {
    _phantom: std::marker::PhantomData<(T, R)>,
}

impl<T, R> ActivityBuilder<T, R>
where
    T: Serialize,
    R: for<'de> Deserialize<'de>,
{
    fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }

    pub async fn call<F, Fut>(
        self,
        _activity_fn: F,
        _input: T,
    ) -> Result<R, Box<dyn std::error::Error>>
    where
        F: Fn(T) -> Fut,
        Fut: Future<Output = Result<R, crate::error::WhiteLabelError>>,
    {
        Err("Mock implementation".into())
    }
}

// Mock workflow macro
pub use temporal_mock_macros::*;

mod temporal_mock_macros {
    pub use temporal_mock_macros_impl::*;
}

mod temporal_mock_macros_impl {
    // Mock workflow attribute macro
    pub fn workflow(_attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        item
    }

    // Mock activity attribute macro  
    pub fn activity(_attr: proc_macro2::TokenStream, item: proc_macro2::TokenStream) -> proc_macro2::TokenStream {
        item
    }
}

// Re-export for convenience
pub use self::Client as TemporalClient;