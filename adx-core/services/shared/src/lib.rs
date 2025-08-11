pub mod config;
pub mod database;
pub mod error;
pub mod temporal;
pub mod types;
pub mod auth;
pub mod logging;
pub mod health;
pub mod middleware;

// Re-export commonly used types
pub use error::{Error, Result};
pub use types::*;
pub use temporal::{
    // AdxTemporalClient,  // Commented out due to SDK compatibility
    TemporalConfig, TemporalError, WorkflowError, ActivityError,
    WorkflowContext, ActivityContext, WorkflowVersion, RetryPolicy, AdxWorkflowVersionManager
};
pub use auth::{JwtClaims, TenantContext, UserContext};
pub use database::{DatabasePool, Repository};