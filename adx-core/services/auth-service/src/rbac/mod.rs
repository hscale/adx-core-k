// RBAC Module - Role-Based Access Control
// Following Temporal-First principle: Complex operations are workflows, simple operations are direct API calls

pub mod activities;
pub mod service;
pub mod types;
pub mod workflows;

pub use service::RbacService;
pub use types::*;
