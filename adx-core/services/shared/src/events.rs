use crate::types::TenantId;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub id: Uuid,
    pub event_type: String,
    pub tenant_id: Option<TenantId>,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub event_data: serde_json::Value,
    pub metadata: EventMetadata,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    pub correlation_id: Uuid,
    pub causation_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub source_service: String,
    pub version: String,
}

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn publish(&self, event: Event) -> Result<(), EventBusError>;
    async fn subscribe(&self, subscription: EventSubscription) -> Result<(), EventBusError>;
    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError>;
}

#[derive(Clone)]
pub struct EventSubscription {
    pub id: String,
    pub event_types: Vec<String>,
    pub handler: Arc<dyn EventHandler>,
}

impl std::fmt::Debug for EventSubscription {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventSubscription")
            .field("id", &self.id)
            .field("event_types", &self.event_types)
            .field("handler", &"<EventHandler>")
            .finish()
    }
}

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &Event) -> Result<(), EventHandlerError>;
}

#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Transport error: {0}")]
    Transport(String),
    #[error("Subscription error: {0}")]
    Subscription(String),
}

#[derive(Debug, thiserror::Error)]
pub enum EventHandlerError {
    #[error("Handler error: {0}")]
    Handler(String),
    #[error("Retry needed: {0}")]
    Retry(String),
}

pub struct InMemoryEventBus {
    subscriptions: Arc<RwLock<HashMap<String, EventSubscription>>>,
}

impl InMemoryEventBus {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for InMemoryEventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EventBus for InMemoryEventBus {
    async fn publish(&self, event: Event) -> Result<(), EventBusError> {
        let subscriptions = self.subscriptions.read().await;

        for subscription in subscriptions.values() {
            if subscription.event_types.contains(&event.event_type) {
                let handler = subscription.handler.clone();
                let event_clone = event.clone();
                tokio::spawn(async move {
                    if let Err(e) = handler.handle(&event_clone).await {
                        tracing::error!("Event handler error: {}", e);
                    }
                });
            }
        }

        Ok(())
    }

    async fn subscribe(&self, subscription: EventSubscription) -> Result<(), EventBusError> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(subscription.id.clone(), subscription);
        Ok(())
    }

    async fn unsubscribe(&self, subscription_id: &str) -> Result<(), EventBusError> {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(subscription_id);
        Ok(())
    }
}
