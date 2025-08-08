pub mod config;
pub mod database;
pub mod error;
pub mod temporal;
pub mod types;
pub mod auth;
pub mod logging;
pub mod health;

// Re-export commonly used types
pub use error::{Error, Result};
pub use types::*;
pub use temporal::{TemporalClient, WorkflowContext};
pub use auth::{JwtClaims, TenantContext, UserContext};
pub use database::{DatabasePool, Repository};