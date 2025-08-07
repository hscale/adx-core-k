use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type TenantId = Uuid;
pub type UserId = Uuid;
pub type CorrelationId = Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestContext {
    pub correlation_id: CorrelationId,
    pub tenant_id: TenantId,
    pub user_id: Option<UserId>,
    pub trace_id: String,
    pub timestamp: DateTime<Utc>,
}

impl RequestContext {
    pub fn new(tenant_id: TenantId, user_id: Option<UserId>) -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            tenant_id,
            user_id,
            trace_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub data: T,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub correlation_id: CorrelationId,
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

impl ResponseMetadata {
    pub fn success() -> Self {
        Self {
            correlation_id: Uuid::new_v4(),
            timestamp: Utc::now(),
            version: "1.0.0".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiError {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub correlation_id: CorrelationId,
    pub timestamp: DateTime<Utc>,
}
