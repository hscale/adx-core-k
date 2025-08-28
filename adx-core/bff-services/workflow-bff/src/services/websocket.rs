use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

#[derive(Clone)]
pub struct WebSocketService {
    connections: Arc<RwLock<HashMap<String, broadcast::Sender<String>>>>,
}

impl WebSocketService {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_connection(&self, user_id: String) -> (String, broadcast::Receiver<String>) {
        let connection_id = Uuid::new_v4().to_string();
        let (tx, rx) = broadcast::channel(100);
        
        self.connections.write().await.insert(connection_id.clone(), tx);
        
        tracing::info!("WebSocket connection added for user: {} (connection: {})", user_id, connection_id);
        
        (connection_id, rx)
    }

    pub async fn remove_connection(&self, connection_id: &str) {
        self.connections.write().await.remove(connection_id);
        tracing::info!("WebSocket connection removed: {}", connection_id);
    }

    pub async fn broadcast_workflow_update(&self, workflow_id: &str, status: &str, progress: Option<f32>) {
        let message = serde_json::json!({
            "type": "workflow_update",
            "workflow_id": workflow_id,
            "status": status,
            "progress": progress,
            "timestamp": chrono::Utc::now().to_rfc3339()
        }).to_string();

        let connections = self.connections.read().await;
        for (connection_id, sender) in connections.iter() {
            if let Err(_) = sender.send(message.clone()) {
                tracing::warn!("Failed to send message to connection: {}", connection_id);
            }
        }
    }

    pub async fn send_to_user(&self, user_id: &str, message: &str) {
        // In a real implementation, we'd track which connections belong to which users
        // For now, broadcast to all connections
        let connections = self.connections.read().await;
        for (connection_id, sender) in connections.iter() {
            if let Err(_) = sender.send(message.to_string()) {
                tracing::warn!("Failed to send message to user {} (connection: {})", user_id, connection_id);
            }
        }
    }
}