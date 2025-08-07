//! # ADX CORE Shared Library
//!
//! Common types, traits, and utilities for all ADX CORE services.
//! Implements enterprise patterns and Temporal-First architecture.

pub mod database;
pub mod events;
pub mod observability;
pub mod temporal;
pub mod types;

// Re-export commonly used types
pub use database::DatabaseManager;
pub use events::{Event, EventBus, EventSubscription, InMemoryEventBus};
pub use observability::init_tracing;
pub use temporal::{
    StandardWorkflowInput, StandardWorkflowOutput, ValidationResult, WorkflowContext,
    WorkflowStatus,
};
pub use types::*;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// SERVICE TRAITS
// ============================================================================

/// Base service trait that all ADX services must implement
#[async_trait]
pub trait BaseService: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Service name for identification
    fn service_name() -> &'static str;

    /// Health check for the service
    async fn health_check(&self) -> Result<ServiceHealthStatus, Self::Error>;

    /// Get service metrics for monitoring
    async fn get_metrics(&self) -> Result<ServiceMetrics, Self::Error>;

    /// Graceful shutdown
    async fn shutdown(&self) -> Result<(), Self::Error>;
}

/// Repository trait for data access with tenant isolation
#[async_trait]
pub trait Repository<T, ID>: Send + Sync {
    type Error: std::error::Error + Send + Sync + 'static;

    /// Create entity with tenant isolation
    async fn create(&self, tenant_id: TenantId, entity: T) -> Result<T, Self::Error>;

    /// Get entity by ID with tenant isolation
    async fn get_by_id(&self, tenant_id: TenantId, id: ID) -> Result<Option<T>, Self::Error>;

    /// List entities with pagination and tenant isolation
    async fn list(
        &self,
        tenant_id: TenantId,
        filters: RepositoryFilters,
        pagination: Pagination,
    ) -> Result<PaginatedResult<T>, Self::Error>;

    /// Update entity with tenant isolation
    async fn update(&self, tenant_id: TenantId, id: ID, entity: T) -> Result<T, Self::Error>;

    /// Delete entity with tenant isolation
    async fn delete(&self, tenant_id: TenantId, id: ID) -> Result<(), Self::Error>;

    /// Health check for repository
    async fn health_check(&self) -> Result<(), Self::Error>;
}

// ============================================================================
// COMMON RESPONSE TYPES
// ============================================================================

/// Standard API response wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub metadata: ResponseMetadata,
}

/// Response metadata for tracing and versioning
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub correlation_id: Uuid,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub version: String,
}

/// Paginated response for list operations
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_count: u64,
    pub page: u32,
    pub page_size: u32,
    pub has_more: bool,
}

/// Pagination parameters
#[derive(Debug, Deserialize)]
pub struct Pagination {
    pub page: u32,
    pub page_size: u32,
    pub sort_by: Option<String>,
    pub sort_order: Option<SortOrder>,
}

impl Default for Pagination {
    fn default() -> Self {
        Self {
            page: 1,
            page_size: 20,
            sort_by: None,
            sort_order: Some(SortOrder::Asc),
        }
    }
}

#[derive(Debug, Deserialize)]
pub enum SortOrder {
    Asc,
    Desc,
}

/// Generic filters for repository queries
#[derive(Debug, Default)]
pub struct RepositoryFilters {
    pub filters: std::collections::HashMap<String, serde_json::Value>,
}

// ============================================================================
// SERVICE MONITORING
// ============================================================================

/// Service health status
#[derive(Debug, Serialize)]
pub struct ServiceHealthStatus {
    pub healthy: bool,
    pub checks: Vec<HealthCheck>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Individual health check result
#[derive(Debug, Serialize)]
pub struct HealthCheck {
    pub name: String,
    pub healthy: bool,
    pub duration_ms: u64,
    pub message: Option<String>,
}

/// Service metrics for monitoring
#[derive(Debug, Serialize)]
pub struct ServiceMetrics {
    pub requests_per_second: f64,
    pub average_response_time_ms: f64,
    pub error_rate: f64,
    pub active_connections: u64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub custom_metrics: std::collections::HashMap<String, f64>,
}

// ============================================================================
// ERROR HANDLING
// ============================================================================

/// Common error types across services
#[derive(thiserror::Error, Debug)]
pub enum CommonError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Permission denied: {action} on {resource}")]
    PermissionDenied { action: String, resource: String },

    #[error("Resource not found: {resource_type} with id {id}")]
    NotFound { resource_type: String, id: String },

    #[error("Tenant access denied: {tenant_id}")]
    TenantAccessDenied { tenant_id: TenantId },

    #[error("Rate limit exceeded for {resource}")]
    RateLimitExceeded { resource: String },

    #[error("External service error: {service} - {message}")]
    ExternalService { service: String, message: String },

    #[error("Temporal workflow error: {0}")]
    Workflow(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Internal server error")]
    Internal,
}

impl CommonError {
    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            CommonError::Validation { .. } => 400,
            CommonError::PermissionDenied { .. } => 403,
            CommonError::TenantAccessDenied { .. } => 403,
            CommonError::NotFound { .. } => 404,
            CommonError::RateLimitExceeded { .. } => 429,
            CommonError::Database(_) => 500,
            CommonError::ExternalService { .. } => 502,
            CommonError::Workflow(_) => 500,
            CommonError::Configuration(_) => 500,
            CommonError::Internal => 500,
        }
    }
}

// ============================================================================
// CONFIGURATION
// ============================================================================

/// Service configuration
#[derive(Debug, Clone)]
pub struct ServiceConfig {
    pub database_url: String,
    pub temporal_server_url: String,
    pub redis_url: String,
    pub jwt_secret: String,
    pub max_connections: u32,
    pub connection_timeout_seconds: u64,
    pub log_level: String,
    pub metrics_enabled: bool,
    pub tracing_enabled: bool,
}

impl ServiceConfig {
    /// Load configuration from environment
    pub fn from_env() -> Result<Self, CommonError> {
        Ok(Self {
            database_url: std::env::var("DATABASE_URL").unwrap_or_else(|_| {
                "postgresql://adx_user:dev_password@localhost:5432/adx_core".to_string()
            }),
            temporal_server_url: std::env::var("TEMPORAL_SERVER_URL")
                .unwrap_or_else(|_| "temporal://localhost:7233".to_string()),
            redis_url: std::env::var("REDIS_URL")
                .unwrap_or_else(|_| "redis://localhost:6379".to_string()),
            jwt_secret: std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "dev_secret_key".to_string()),
            max_connections: std::env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .map_err(|_| CommonError::Configuration("Invalid MAX_CONNECTIONS".to_string()))?,
            connection_timeout_seconds: std::env::var("CONNECTION_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()
                .map_err(|_| {
                    CommonError::Configuration("Invalid CONNECTION_TIMEOUT_SECONDS".to_string())
                })?,
            log_level: std::env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            metrics_enabled: std::env::var("METRICS_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
            tracing_enabled: std::env::var("TRACING_ENABLED")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}
