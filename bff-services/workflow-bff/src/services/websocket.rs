use anyhow::Result;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    response::Response,
};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::Arc,
};
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{
    middleware::error_handler::{BffError, BffResult},
    types::WorkflowExecution,
    AppState,
};

#[derive(Clone)]
pub struct WebSocketService {
    // Broadcast channel for workflow updates
    workflow_sender: broadcast::Sender<WorkflowUpdate>,
    // Active connections
    connections: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowUpdate {
    pub workflow_id: String,
    pub update_type: WorkflowUpdateType,
    pub data: serde_json::Value,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowUpdateType {
    StatusChange,
    ProgressUpdate,
    ActivityUpdate,
    Error,
    Completed,
    Cancelled,
    Terminated,
}

#[derive(Debug, Clone)]
struct ConnectionInfo {
    user_id: String,
    tenant_id: String,
    subscribed_workflows: Vec<String>,
    connected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
struct WebSocketMessage {
    message_type: String,
    data: serde_json::Value,
    timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
struct SubscribeMessage {
    workflow_id: String,
}

#[derive(Debug, Deserialize)]
struct UnsubscribeMessage {
    workflow_id: String,
}

impl WebSocketService {
    pub fn new() -> Self {
        let (workflow_sender, _) = broadcast::channel(1000);
        
        Self {
            workflow_sender,
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // Handle WebSocket upgrade
    pub async fn handle_websocket_upgrade(
        &self,
        ws: WebSocketUpgrade,
        user_id: String,
        tenant_id: String,
        state: AppState,
    ) -> Response {
        let service = self.clone();
        
        ws.on_upgrade(move |socket| {
            service.handle_websocket_connection(socket, user_id, tenant_id, state)
        })
    }

    // Handle individual WebSocket connection
    async fn handle_websocket_connection(
        &self,
        socket: WebSocket,
        user_id: String,
        tenant_id: String,
        state: AppState,
    ) {
        let connection_id = Uuid::new_v4().to_string();
        
        info!("WebSocket connection established: {} (user: {}, tenant: {})", 
              connection_id, user_id, tenant_id);

        // Store connection info
        {
            let mut connections = self.connections.write().await;
            connections.insert(connection_id.clone(), ConnectionInfo {
                user_id: user_id.clone(),
                tenant_id: tenant_id.clone(),
                subscribed_workflows: Vec::new(),
                connected_at: chrono::Utc::now(),
            });
        }

        // Store session in Redis
        if let Err(e) = state.redis.store_websocket_session(&connection_id, &user_id, &tenant_id, Some(3600)).await {
            error!("Failed to store WebSocket session: {}", e);
        }

        let (mut sender, mut receiver) = socket.split();
        let mut workflow_receiver = self.workflow_sender.subscribe();

        // Send welcome message
        let welcome_message = WebSocketMessage {
            message_type: "welcome".to_string(),
            data: serde_json::json!({
                "connection_id": connection_id,
                "server_time": chrono::Utc::now(),
                "supported_messages": ["subscribe", "unsubscribe", "ping"]
            }),
            timestamp: chrono::Utc::now(),
        };

        if let Ok(welcome_json) = serde_json::to_string(&welcome_message) {
            if let Err(e) = sender.send(Message::Text(welcome_json)).await {
                error!("Failed to send welcome message: {}", e);
                return;
            }
        }

        // Handle incoming messages and workflow updates concurrently
        let connections_clone = self.connections.clone();
        let state_clone = state.clone();
        let connection_id_clone = connection_id.clone();

        tokio::select! {
            // Handle incoming WebSocket messages
            _ = async {
                while let Some(msg) = receiver.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Err(e) = self.handle_client_message(
                                &text, 
                                &connection_id, 
                                &state_clone
                            ).await {
                                error!("Error handling client message: {}", e);
                            }
                        }
                        Ok(Message::Ping(data)) => {
                            if let Err(e) = sender.send(Message::Pong(data)).await {
                                error!("Failed to send pong: {}", e);
                                break;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            debug!("WebSocket connection closed by client: {}", connection_id);
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            } => {},
            
            // Handle workflow updates broadcast
            _ = async {
                while let Ok(update) = workflow_receiver.recv().await {
                    // Check if this connection is subscribed to this workflow
                    let should_send = {
                        let connections = connections_clone.read().await;
                        if let Some(conn_info) = connections.get(&connection_id) {
                            conn_info.subscribed_workflows.contains(&update.workflow_id) &&
                            conn_info.tenant_id == tenant_id // Ensure tenant isolation
                        } else {
                            false
                        }
                    };

                    if should_send {
                        let message = WebSocketMessage {
                            message_type: "workflow_update".to_string(),
                            data: serde_json::to_value(&update).unwrap_or_default(),
                            timestamp: chrono::Utc::now(),
                        };

                        if let Ok(message_json) = serde_json::to_string(&message) {
                            if let Err(e) = sender.send(Message::Text(message_json)).await {
                                error!("Failed to send workflow update: {}", e);
                                break;
                            }
                        }
                    }
                }
            } => {}
        }

        // Cleanup connection
        self.cleanup_connection(&connection_id_clone, &state).await;
    }

    // Handle client messages
    async fn handle_client_message(
        &self,
        message: &str,
        connection_id: &str,
        state: &AppState,
    ) -> BffResult<()> {
        let parsed_message: serde_json::Value = serde_json::from_str(message)
            .map_err(|e| BffError::validation(format!("Invalid JSON message: {}", e)))?;

        let message_type = parsed_message.get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| BffError::validation("Missing message type"))?;

        match message_type {
            "subscribe" => {
                let subscribe_msg: SubscribeMessage = serde_json::from_value(
                    parsed_message.get("data").cloned().unwrap_or_default()
                ).map_err(|e| BffError::validation(format!("Invalid subscribe message: {}", e)))?;

                self.subscribe_to_workflow(connection_id, &subscribe_msg.workflow_id, state).await?;
            }
            "unsubscribe" => {
                let unsubscribe_msg: UnsubscribeMessage = serde_json::from_value(
                    parsed_message.get("data").cloned().unwrap_or_default()
                ).map_err(|e| BffError::validation(format!("Invalid unsubscribe message: {}", e)))?;

                self.unsubscribe_from_workflow(connection_id, &unsubscribe_msg.workflow_id, state).await?;
            }
            "ping" => {
                // Ping is handled automatically by the WebSocket protocol
                debug!("Received ping from connection: {}", connection_id);
            }
            _ => {
                warn!("Unknown message type: {} from connection: {}", message_type, connection_id);
            }
        }

        Ok(())
    }

    // Subscribe connection to workflow updates
    async fn subscribe_to_workflow(
        &self,
        connection_id: &str,
        workflow_id: &str,
        state: &AppState,
    ) -> BffResult<()> {
        // Update connection info
        {
            let mut connections = self.connections.write().await;
            if let Some(conn_info) = connections.get_mut(connection_id) {
                if !conn_info.subscribed_workflows.contains(&workflow_id.to_string()) {
                    conn_info.subscribed_workflows.push(workflow_id.to_string());
                }
            }
        }

        // Store subscription in Redis for persistence
        if let Err(e) = state.redis.subscribe_to_workflow(workflow_id, connection_id, Some(3600)).await {
            error!("Failed to store workflow subscription: {}", e);
        }

        info!("Connection {} subscribed to workflow: {}", connection_id, workflow_id);
        Ok(())
    }

    // Unsubscribe connection from workflow updates
    async fn unsubscribe_from_workflow(
        &self,
        connection_id: &str,
        workflow_id: &str,
        state: &AppState,
    ) -> BffResult<()> {
        // Update connection info
        {
            let mut connections = self.connections.write().await;
            if let Some(conn_info) = connections.get_mut(connection_id) {
                conn_info.subscribed_workflows.retain(|id| id != workflow_id);
            }
        }

        // Remove subscription from Redis
        if let Err(e) = state.redis.unsubscribe_from_workflow(workflow_id, connection_id).await {
            error!("Failed to remove workflow subscription: {}", e);
        }

        info!("Connection {} unsubscribed from workflow: {}", connection_id, workflow_id);
        Ok(())
    }

    // Broadcast workflow update to all subscribed connections
    pub async fn broadcast_workflow_update(&self, update: WorkflowUpdate) -> BffResult<()> {
        debug!("Broadcasting workflow update: {} (type: {:?})", update.workflow_id, update.update_type);

        if let Err(e) = self.workflow_sender.send(update) {
            // This is expected if there are no active receivers
            debug!("No active WebSocket connections to receive workflow update: {}", e);
        }

        Ok(())
    }

    // Send workflow status update
    pub async fn send_workflow_status_update(
        &self,
        workflow_execution: &WorkflowExecution,
    ) -> BffResult<()> {
        let update = WorkflowUpdate {
            workflow_id: workflow_execution.workflow_id.clone(),
            update_type: match workflow_execution.status {
                crate::types::WorkflowStatus::Completed => WorkflowUpdateType::Completed,
                crate::types::WorkflowStatus::Failed => WorkflowUpdateType::Error,
                crate::types::WorkflowStatus::Cancelled => WorkflowUpdateType::Cancelled,
                crate::types::WorkflowStatus::Terminated => WorkflowUpdateType::Terminated,
                _ => WorkflowUpdateType::StatusChange,
            },
            data: serde_json::to_value(workflow_execution)?,
            timestamp: chrono::Utc::now(),
        };

        self.broadcast_workflow_update(update).await
    }

    // Send workflow progress update
    pub async fn send_workflow_progress_update(
        &self,
        workflow_id: &str,
        progress: &crate::types::WorkflowProgress,
    ) -> BffResult<()> {
        let update = WorkflowUpdate {
            workflow_id: workflow_id.to_string(),
            update_type: WorkflowUpdateType::ProgressUpdate,
            data: serde_json::to_value(progress)?,
            timestamp: chrono::Utc::now(),
        };

        self.broadcast_workflow_update(update).await
    }

    // Send activity update
    pub async fn send_activity_update(
        &self,
        workflow_id: &str,
        activity: &crate::types::ActivityProgress,
    ) -> BffResult<()> {
        let update = WorkflowUpdate {
            workflow_id: workflow_id.to_string(),
            update_type: WorkflowUpdateType::ActivityUpdate,
            data: serde_json::to_value(activity)?,
            timestamp: chrono::Utc::now(),
        };

        self.broadcast_workflow_update(update).await
    }

    // Get connection statistics
    pub async fn get_connection_stats(&self) -> ConnectionStats {
        let connections = self.connections.read().await;
        
        let mut tenant_counts: HashMap<String, u32> = HashMap::new();
        let mut user_counts: HashMap<String, u32> = HashMap::new();
        
        for conn_info in connections.values() {
            *tenant_counts.entry(conn_info.tenant_id.clone()).or_insert(0) += 1;
            *user_counts.entry(conn_info.user_id.clone()).or_insert(0) += 1;
        }

        ConnectionStats {
            total_connections: connections.len() as u32,
            connections_by_tenant: tenant_counts,
            connections_by_user: user_counts,
            oldest_connection: connections.values()
                .map(|c| c.connected_at)
                .min(),
        }
    }

    // Cleanup connection
    async fn cleanup_connection(&self, connection_id: &str, state: &AppState) {
        // Remove from active connections
        let subscribed_workflows = {
            let mut connections = self.connections.write().await;
            connections.remove(connection_id)
                .map(|conn| conn.subscribed_workflows)
                .unwrap_or_default()
        };

        // Remove all workflow subscriptions
        for workflow_id in subscribed_workflows {
            if let Err(e) = state.redis.unsubscribe_from_workflow(&workflow_id, connection_id).await {
                error!("Failed to cleanup workflow subscription: {}", e);
            }
        }

        // Remove session from Redis
        if let Err(e) = state.redis.remove_websocket_session(connection_id).await {
            error!("Failed to cleanup WebSocket session: {}", e);
        }

        info!("Cleaned up WebSocket connection: {}", connection_id);
    }

    // Health check
    pub async fn health_check(&self) -> BffResult<()> {
        let connections = self.connections.read().await;
        debug!("WebSocket service health check: {} active connections", connections.len());
        Ok(())
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionStats {
    pub total_connections: u32,
    pub connections_by_tenant: HashMap<String, u32>,
    pub connections_by_user: HashMap<String, u32>,
    pub oldest_connection: Option<chrono::DateTime<chrono::Utc>>,
}

impl Default for WebSocketService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_websocket_service_creation() {
        let service = WebSocketService::new();
        let stats = service.get_connection_stats().await;
        assert_eq!(stats.total_connections, 0);
    }

    #[tokio::test]
    async fn test_workflow_update_broadcast() {
        let service = WebSocketService::new();
        
        let update = WorkflowUpdate {
            workflow_id: "test-workflow-123".to_string(),
            update_type: WorkflowUpdateType::StatusChange,
            data: serde_json::json!({"status": "running"}),
            timestamp: chrono::Utc::now(),
        };

        // This should not fail even with no active connections
        let result = service.broadcast_workflow_update(update).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_connection_stats() {
        let service = WebSocketService::new();
        
        // Add mock connection
        {
            let mut connections = service.connections.write().await;
            connections.insert("conn1".to_string(), ConnectionInfo {
                user_id: "user123".to_string(),
                tenant_id: "tenant456".to_string(),
                subscribed_workflows: vec!["workflow1".to_string()],
                connected_at: chrono::Utc::now(),
            });
        }

        let stats = service.get_connection_stats().await;
        assert_eq!(stats.total_connections, 1);
        assert_eq!(stats.connections_by_tenant.get("tenant456"), Some(&1));
        assert_eq!(stats.connections_by_user.get("user123"), Some(&1));
    }
}