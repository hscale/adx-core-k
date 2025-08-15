use crate::{
    error::{SecurityError, SecurityResult},
    models::{AuditLog, AuditEventCategory, AuditOutcome, CreateAuditLogRequest, AuditLogResponse},
    repositories::AuditRepository,
    encryption::EncryptionService,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct AuditService {
    repository: Arc<AuditRepository>,
    encryption: Arc<EncryptionService>,
    batch_buffer: Arc<RwLock<Vec<AuditLog>>>,
    batch_size: usize,
    encryption_enabled: bool,
}

impl AuditService {
    pub fn new(
        repository: Arc<AuditRepository>,
        encryption: Arc<EncryptionService>,
        batch_size: usize,
        encryption_enabled: bool,
    ) -> Self {
        Self {
            repository,
            encryption,
            batch_buffer: Arc::new(RwLock::new(Vec::new())),
            batch_size,
            encryption_enabled,
        }
    }

    /// Log an audit event
    pub async fn log_event(&self, request: CreateAuditLogRequest) -> SecurityResult<Uuid> {
        let audit_log = self.create_audit_log(request).await?;
        
        // Add to batch buffer
        let mut buffer = self.batch_buffer.write().await;
        buffer.push(audit_log.clone());
        
        // Flush if batch is full
        if buffer.len() >= self.batch_size {
            let logs_to_flush = buffer.drain(..).collect();
            drop(buffer); // Release lock before async operation
            
            self.flush_batch(logs_to_flush).await?;
        }
        
        Ok(audit_log.id)
    }

    /// Log authentication event
    pub async fn log_authentication(
        &self,
        tenant_id: &str,
        user_id: Option<&str>,
        action: &str,
        outcome: AuditOutcome,
        ip_address: Option<&str>,
        user_agent: Option<&str>,
        details: Value,
    ) -> SecurityResult<Uuid> {
        self.log_event(CreateAuditLogRequest {
            tenant_id: tenant_id.to_string(),
            user_id: user_id.map(|s| s.to_string()),
            session_id: None,
            event_type: "authentication".to_string(),
            event_category: AuditEventCategory::Authentication,
            resource_type: "user".to_string(),
            resource_id: user_id.map(|s| s.to_string()),
            action: action.to_string(),
            outcome,
            ip_address: ip_address.map(|s| s.to_string()),
            user_agent: user_agent.map(|s| s.to_string()),
            request_id: None,
            details,
        }).await
    }

    /// Log data access event
    pub async fn log_data_access(
        &self,
        tenant_id: &str,
        user_id: &str,
        resource_type: &str,
        resource_id: &str,
        action: &str,
        outcome: AuditOutcome,
        details: Value,
    ) -> SecurityResult<Uuid> {
        self.log_event(CreateAuditLogRequest {
            tenant_id: tenant_id.to_string(),
            user_id: Some(user_id.to_string()),
            session_id: None,
            event_type: "data_access".to_string(),
            event_category: AuditEventCategory::DataAccess,
            resource_type: resource_type.to_string(),
            resource_id: Some(resource_id.to_string()),
            action: action.to_string(),
            outcome,
            ip_address: None,
            user_agent: None,
            request_id: None,
            details,
        }).await
    }

    /// Log data modification event
    pub async fn log_data_modification(
        &self,
        tenant_id: &str,
        user_id: &str,
        resource_type: &str,
        resource_id: &str,
        action: &str,
        outcome: AuditOutcome,
        before_state: Option<Value>,
        after_state: Option<Value>,
    ) -> SecurityResult<Uuid> {
        let mut details = serde_json::Map::new();
        if let Some(before) = before_state {
            details.insert("before_state".to_string(), before);
        }
        if let Some(after) = after_state {
            details.insert("after_state".to_string(), after);
        }

        self.log_event(CreateAuditLogRequest {
            tenant_id: tenant_id.to_string(),
            user_id: Some(user_id.to_string()),
            session_id: None,
            event_type: "data_modification".to_string(),
            event_category: AuditEventCategory::DataModification,
            resource_type: resource_type.to_string(),
            resource_id: Some(resource_id.to_string()),
            action: action.to_string(),
            outcome,
            ip_address: None,
            user_agent: None,
            request_id: None,
            details: Value::Object(details),
        }).await
    }

    /// Log security event
    pub async fn log_security_event(
        &self,
        tenant_id: &str,
        event_type: &str,
        severity: &str,
        description: &str,
        details: Value,
    ) -> SecurityResult<Uuid> {
        let mut event_details = serde_json::Map::new();
        event_details.insert("severity".to_string(), Value::String(severity.to_string()));
        event_details.insert("description".to_string(), Value::String(description.to_string()));
        event_details.insert("additional_details".to_string(), details);

        self.log_event(CreateAuditLogRequest {
            tenant_id: tenant_id.to_string(),
            user_id: None,
            session_id: None,
            event_type: event_type.to_string(),
            event_category: AuditEventCategory::Security,
            resource_type: "system".to_string(),
            resource_id: None,
            action: "security_event".to_string(),
            outcome: AuditOutcome::Warning,
            ip_address: None,
            user_agent: None,
            request_id: None,
            details: Value::Object(event_details),
        }).await
    }

    /// Log compliance event
    pub async fn log_compliance_event(
        &self,
        tenant_id: &str,
        compliance_type: &str,
        action: &str,
        outcome: AuditOutcome,
        details: Value,
    ) -> SecurityResult<Uuid> {
        let mut event_details = serde_json::Map::new();
        event_details.insert("compliance_type".to_string(), Value::String(compliance_type.to_string()));
        event_details.insert("additional_details".to_string(), details);

        self.log_event(CreateAuditLogRequest {
            tenant_id: tenant_id.to_string(),
            user_id: None,
            session_id: None,
            event_type: "compliance".to_string(),
            event_category: AuditEventCategory::Compliance,
            resource_type: "compliance".to_string(),
            resource_id: None,
            action: action.to_string(),
            outcome,
            ip_address: None,
            user_agent: None,
            request_id: None,
            details: Value::Object(event_details),
        }).await
    }

    /// Get audit logs with filtering
    pub async fn get_audit_logs(
        &self,
        tenant_id: &str,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        event_category: Option<AuditEventCategory>,
        user_id: Option<&str>,
        resource_type: Option<&str>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<AuditLogResponse> {
        let logs = self.repository.get_audit_logs(
            tenant_id,
            start_date,
            end_date,
            event_category,
            user_id,
            resource_type,
            page,
            page_size,
        ).await?;

        let total_count = self.repository.count_audit_logs(
            tenant_id,
            start_date,
            end_date,
            event_category,
            user_id,
            resource_type,
        ).await?;

        // Decrypt sensitive details if encryption is enabled
        let decrypted_logs = if self.encryption_enabled {
            self.decrypt_audit_logs(logs).await?
        } else {
            logs
        };

        Ok(AuditLogResponse {
            logs: decrypted_logs,
            total_count,
            page,
            page_size,
        })
    }

    /// Export audit logs for compliance
    pub async fn export_audit_logs(
        &self,
        tenant_id: &str,
        start_date: DateTime<Utc>,
        end_date: DateTime<Utc>,
        format: &str,
    ) -> SecurityResult<Vec<u8>> {
        let logs = self.repository.get_all_audit_logs(tenant_id, start_date, end_date).await?;
        
        // Decrypt if needed
        let decrypted_logs = if self.encryption_enabled {
            self.decrypt_audit_logs(logs).await?
        } else {
            logs
        };

        match format.to_lowercase().as_str() {
            "json" => {
                let json_data = serde_json::to_string_pretty(&decrypted_logs)?;
                Ok(json_data.into_bytes())
            }
            "csv" => {
                self.export_to_csv(decrypted_logs).await
            }
            _ => Err(SecurityError::Validation("Unsupported export format".to_string())),
        }
    }

    /// Flush pending audit logs
    pub async fn flush_pending(&self) -> SecurityResult<()> {
        let mut buffer = self.batch_buffer.write().await;
        if !buffer.is_empty() {
            let logs_to_flush = buffer.drain(..).collect();
            drop(buffer);
            
            self.flush_batch(logs_to_flush).await?;
        }
        Ok(())
    }

    /// Clean up old audit logs based on retention policy
    pub async fn cleanup_old_logs(&self, tenant_id: &str, retention_days: i32) -> SecurityResult<i64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);
        let deleted_count = self.repository.delete_old_logs(tenant_id, cutoff_date).await?;
        
        info!(
            tenant_id = %tenant_id,
            retention_days = %retention_days,
            deleted_count = %deleted_count,
            "Cleaned up old audit logs"
        );

        Ok(deleted_count)
    }

    // Private helper methods

    async fn create_audit_log(&self, request: CreateAuditLogRequest) -> SecurityResult<AuditLog> {
        let mut details = request.details;
        
        // Encrypt sensitive details if encryption is enabled
        if self.encryption_enabled {
            details = self.encrypt_audit_details(details).await?;
        }

        // Calculate risk score based on event characteristics
        let risk_score = self.calculate_risk_score(&request);

        Ok(AuditLog {
            id: Uuid::new_v4(),
            tenant_id: request.tenant_id,
            user_id: request.user_id,
            session_id: request.session_id,
            event_type: request.event_type,
            event_category: request.event_category,
            resource_type: request.resource_type,
            resource_id: request.resource_id,
            action: request.action,
            outcome: request.outcome,
            ip_address: request.ip_address,
            user_agent: request.user_agent,
            request_id: request.request_id,
            details,
            risk_score: Some(risk_score),
            created_at: Utc::now(),
        })
    }

    async fn flush_batch(&self, logs: Vec<AuditLog>) -> SecurityResult<()> {
        match self.repository.batch_insert_logs(logs.clone()).await {
            Ok(_) => {
                info!(count = %logs.len(), "Successfully flushed audit logs batch");
            }
            Err(e) => {
                error!(error = %e, count = %logs.len(), "Failed to flush audit logs batch");
                // In a production system, you might want to implement retry logic
                // or write to a backup storage system
                return Err(SecurityError::AuditLog(format!("Batch flush failed: {}", e)));
            }
        }
        Ok(())
    }

    async fn encrypt_audit_details(&self, details: Value) -> SecurityResult<Value> {
        let details_str = serde_json::to_string(&details)?;
        let encrypted_data = self.encryption.encrypt_data(details_str.as_bytes()).await
            .map_err(|e| SecurityError::Encryption(e.to_string()))?;
        
        Ok(serde_json::json!({
            "encrypted": true,
            "data": base64::encode(encrypted_data)
        }))
    }

    async fn decrypt_audit_logs(&self, logs: Vec<AuditLog>) -> SecurityResult<Vec<AuditLog>> {
        let mut decrypted_logs = Vec::new();
        
        for mut log in logs {
            if let Some(encrypted_obj) = log.details.as_object() {
                if encrypted_obj.get("encrypted").and_then(|v| v.as_bool()).unwrap_or(false) {
                    if let Some(encrypted_data) = encrypted_obj.get("data").and_then(|v| v.as_str()) {
                        match base64::decode(encrypted_data) {
                            Ok(data) => {
                                match self.encryption.decrypt_data(&data).await {
                                    Ok(decrypted_bytes) => {
                                        match String::from_utf8(decrypted_bytes) {
                                            Ok(decrypted_str) => {
                                                match serde_json::from_str(&decrypted_str) {
                                                    Ok(decrypted_details) => {
                                                        log.details = decrypted_details;
                                                    }
                                                    Err(e) => {
                                                        warn!(error = %e, "Failed to parse decrypted audit details");
                                                    }
                                                }
                                            }
                                            Err(e) => {
                                                warn!(error = %e, "Failed to convert decrypted data to string");
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        warn!(error = %e, "Failed to decrypt audit details");
                                    }
                                }
                            }
                            Err(e) => {
                                warn!(error = %e, "Failed to decode base64 encrypted data");
                            }
                        }
                    }
                }
            }
            decrypted_logs.push(log);
        }
        
        Ok(decrypted_logs)
    }

    async fn export_to_csv(&self, logs: Vec<AuditLog>) -> SecurityResult<Vec<u8>> {
        let mut csv_content = String::new();
        
        // CSV header
        csv_content.push_str("id,tenant_id,user_id,event_type,event_category,resource_type,resource_id,action,outcome,ip_address,user_agent,risk_score,created_at,details\n");
        
        // CSV rows
        for log in logs {
            let details_str = serde_json::to_string(&log.details)
                .unwrap_or_else(|_| "{}".to_string())
                .replace('"', '""'); // Escape quotes for CSV
            
            csv_content.push_str(&format!(
                "{},{},{},{},{:?},{},{},{},{:?},{},{},{},{},{}\n",
                log.id,
                log.tenant_id,
                log.user_id.unwrap_or_default(),
                log.event_type,
                log.event_category,
                log.resource_type,
                log.resource_id.unwrap_or_default(),
                log.action,
                log.outcome,
                log.ip_address.unwrap_or_default(),
                log.user_agent.unwrap_or_default(),
                log.risk_score.unwrap_or(0),
                log.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                format!("\"{}\"", details_str)
            ));
        }
        
        Ok(csv_content.into_bytes())
    }

    fn calculate_risk_score(&self, request: &CreateAuditLogRequest) -> i32 {
        let mut score = 0;

        // Base score by event category
        score += match request.event_category {
            AuditEventCategory::Security => 80,
            AuditEventCategory::Authentication => 60,
            AuditEventCategory::Authorization => 50,
            AuditEventCategory::DataModification => 40,
            AuditEventCategory::Administrative => 30,
            AuditEventCategory::DataAccess => 20,
            AuditEventCategory::Privacy => 70,
            AuditEventCategory::Compliance => 50,
            _ => 10,
        };

        // Adjust based on outcome
        score += match request.outcome {
            AuditOutcome::Failure => 30,
            AuditOutcome::Error => 25,
            AuditOutcome::Warning => 15,
            AuditOutcome::Success => 0,
        };

        // Adjust based on action type
        if request.action.contains("delete") || request.action.contains("remove") {
            score += 20;
        } else if request.action.contains("create") || request.action.contains("add") {
            score += 10;
        } else if request.action.contains("modify") || request.action.contains("update") {
            score += 5;
        }

        // Cap the score at 100
        score.min(100)
    }
}

// Base64 encoding/decoding utilities
mod base64 {
    use base64::{Engine as _, engine::general_purpose};

    pub fn encode<T: AsRef<[u8]>>(input: T) -> String {
        general_purpose::STANDARD.encode(input)
    }

    pub fn decode<T: AsRef<[u8]>>(input: T) -> Result<Vec<u8>, base64::DecodeError> {
        general_purpose::STANDARD.decode(input)
    }
}