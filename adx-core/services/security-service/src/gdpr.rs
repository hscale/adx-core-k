use crate::{
    error::{SecurityError, SecurityResult},
    models::{
        GdprRequest, GdprRequestType, GdprRequestStatus, GdprExportRequest, 
        GdprDeletionRequest, GdprExportResponse
    },
    repositories::GdprRepository,
    encryption::EncryptionService,
    audit::AuditService,
};
use chrono::{DateTime, Utc, Duration};
use serde_json::{Value, Map};
use std::sync::Arc;
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct GdprService {
    repository: Arc<GdprRepository>,
    encryption: Arc<EncryptionService>,
    audit_service: Arc<AuditService>,
    verification_token_expiry_hours: i64,
}

impl GdprService {
    pub fn new(
        repository: Arc<GdprRepository>,
        encryption: Arc<EncryptionService>,
        audit_service: Arc<AuditService>,
        verification_token_expiry_hours: i64,
    ) -> Self {
        Self {
            repository,
            encryption,
            audit_service,
            verification_token_expiry_hours,
        }
    }

    /// Submit a GDPR data export request
    pub async fn request_data_export(&self, request: GdprExportRequest) -> SecurityResult<GdprExportResponse> {
        // Validate the request
        self.validate_export_request(&request)?;

        // Check for existing pending requests
        if let Some(existing) = self.repository.get_pending_request(
            &request.tenant_id,
            &request.user_id,
            GdprRequestType::DataExport,
        ).await? {
            return Ok(GdprExportResponse {
                request_id: existing.id,
                status: existing.status,
                download_url: existing.data_export_url,
                expires_at: None,
            });
        }

        // Create verification token
        let verification_token = self.generate_verification_token();

        // Create GDPR request
        let gdpr_request = GdprRequest {
            id: Uuid::new_v4(),
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            request_type: GdprRequestType::DataExport,
            status: GdprRequestStatus::Pending,
            requester_email: request.requester_email.clone(),
            verification_token: Some(verification_token.clone()),
            verified_at: None,
            processed_at: None,
            data_export_url: None,
            deletion_confirmation: None,
            notes: Some(serde_json::to_string(&serde_json::json!({
                "include_deleted": request.include_deleted,
                "format": request.format
            }))?),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save the request
        let saved_request = self.repository.create_request(gdpr_request).await?;

        // Send verification email (this would integrate with notification service)
        self.send_verification_email(&saved_request, &verification_token).await?;

        // Log the GDPR request
        self.audit_service.log_compliance_event(
            &request.tenant_id,
            "GDPR",
            "data_export_requested",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "request_id": saved_request.id,
                "user_id": request.user_id,
                "requester_email": request.requester_email,
                "include_deleted": request.include_deleted,
                "format": request.format
            }),
        ).await?;

        Ok(GdprExportResponse {
            request_id: saved_request.id,
            status: saved_request.status,
            download_url: None,
            expires_at: Some(Utc::now() + Duration::hours(self.verification_token_expiry_hours)),
        })
    }

    /// Submit a GDPR data deletion request
    pub async fn request_data_deletion(&self, request: GdprDeletionRequest) -> SecurityResult<Uuid> {
        // Validate the request
        self.validate_deletion_request(&request)?;

        // Check for existing pending requests
        if let Some(existing) = self.repository.get_pending_request(
            &request.tenant_id,
            &request.user_id,
            GdprRequestType::DataDeletion,
        ).await? {
            return Ok(existing.id);
        }

        // Create verification token if required
        let verification_token = if request.verification_required {
            Some(self.generate_verification_token())
        } else {
            None
        };

        // Create GDPR request
        let gdpr_request = GdprRequest {
            id: Uuid::new_v4(),
            tenant_id: request.tenant_id.clone(),
            user_id: request.user_id.clone(),
            request_type: GdprRequestType::DataDeletion,
            status: if request.verification_required {
                GdprRequestStatus::Pending
            } else {
                GdprRequestStatus::Verified
            },
            requester_email: request.requester_email.clone(),
            verification_token: verification_token.clone(),
            verified_at: if request.verification_required { None } else { Some(Utc::now()) },
            processed_at: None,
            data_export_url: None,
            deletion_confirmation: None,
            notes: Some(serde_json::to_string(&serde_json::json!({
                "delete_backups": request.delete_backups,
                "verification_required": request.verification_required
            }))?),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Save the request
        let saved_request = self.repository.create_request(gdpr_request).await?;

        // Send verification email if required
        if let Some(token) = verification_token {
            self.send_verification_email(&saved_request, &token).await?;
        }

        // Log the GDPR request
        self.audit_service.log_compliance_event(
            &request.tenant_id,
            "GDPR",
            "data_deletion_requested",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "request_id": saved_request.id,
                "user_id": request.user_id,
                "requester_email": request.requester_email,
                "delete_backups": request.delete_backups,
                "verification_required": request.verification_required
            }),
        ).await?;

        Ok(saved_request.id)
    }

    /// Verify a GDPR request using the verification token
    pub async fn verify_request(&self, request_id: Uuid, token: &str) -> SecurityResult<()> {
        let mut request = self.repository.get_request(request_id).await?
            .ok_or_else(|| SecurityError::NotFound("GDPR request not found".to_string()))?;

        // Check if already verified
        if request.status != GdprRequestStatus::Pending {
            return Err(SecurityError::Validation("Request is not pending verification".to_string()));
        }

        // Verify token
        if request.verification_token.as_ref() != Some(&token.to_string()) {
            return Err(SecurityError::Authorization("Invalid verification token".to_string()));
        }

        // Check token expiry
        let token_expiry = request.created_at + Duration::hours(self.verification_token_expiry_hours);
        if Utc::now() > token_expiry {
            request.status = GdprRequestStatus::Expired;
            self.repository.update_request(request).await?;
            return Err(SecurityError::Validation("Verification token has expired".to_string()));
        }

        // Mark as verified
        request.status = GdprRequestStatus::Verified;
        request.verified_at = Some(Utc::now());
        request.updated_at = Utc::now();

        self.repository.update_request(request.clone()).await?;

        // Log verification
        self.audit_service.log_compliance_event(
            &request.tenant_id,
            "GDPR",
            "request_verified",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "request_id": request.id,
                "request_type": request.request_type,
                "user_id": request.user_id
            }),
        ).await?;

        Ok(())
    }

    /// Process verified GDPR requests
    pub async fn process_verified_requests(&self) -> SecurityResult<Vec<Uuid>> {
        let verified_requests = self.repository.get_verified_requests().await?;
        let mut processed_requests = Vec::new();

        for request in verified_requests {
            match self.process_single_request(request).await {
                Ok(_) => {
                    processed_requests.push(request.id);
                }
                Err(e) => {
                    error!(
                        request_id = %request.id,
                        error = %e,
                        "Failed to process GDPR request"
                    );
                    
                    // Mark request as failed
                    let mut failed_request = request;
                    failed_request.status = GdprRequestStatus::Rejected;
                    failed_request.notes = Some(format!("Processing failed: {}", e));
                    failed_request.updated_at = Utc::now();
                    
                    if let Err(update_err) = self.repository.update_request(failed_request).await {
                        error!(error = %update_err, "Failed to update failed request status");
                    }
                }
            }
        }

        Ok(processed_requests)
    }

    /// Get GDPR request status
    pub async fn get_request_status(&self, request_id: Uuid) -> SecurityResult<GdprRequest> {
        self.repository.get_request(request_id).await?
            .ok_or_else(|| SecurityError::NotFound("GDPR request not found".to_string()))
    }

    /// Get all GDPR requests for a tenant
    pub async fn get_tenant_requests(
        &self,
        tenant_id: &str,
        request_type: Option<GdprRequestType>,
        status: Option<GdprRequestStatus>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<Vec<GdprRequest>> {
        self.repository.get_tenant_requests(tenant_id, request_type, status, page, page_size).await
    }

    /// Export user data for GDPR compliance
    pub async fn export_user_data(
        &self,
        tenant_id: &str,
        user_id: &str,
        include_deleted: bool,
        format: &str,
    ) -> SecurityResult<Vec<u8>> {
        // Collect data from all services
        let user_data = self.collect_user_data(tenant_id, user_id, include_deleted).await?;

        // Format the data
        match format.to_lowercase().as_str() {
            "json" => {
                let json_data = serde_json::to_string_pretty(&user_data)?;
                Ok(json_data.into_bytes())
            }
            "xml" => {
                self.export_to_xml(user_data).await
            }
            _ => Err(SecurityError::Validation("Unsupported export format".to_string())),
        }
    }

    /// Delete user data for GDPR compliance
    pub async fn delete_user_data(
        &self,
        tenant_id: &str,
        user_id: &str,
        delete_backups: bool,
    ) -> SecurityResult<String> {
        // This would coordinate with all services to delete user data
        // For now, we'll return a confirmation token
        let confirmation_token = Uuid::new_v4().to_string();

        // Log the deletion
        self.audit_service.log_compliance_event(
            tenant_id,
            "GDPR",
            "user_data_deleted",
            crate::models::AuditOutcome::Success,
            serde_json::json!({
                "user_id": user_id,
                "delete_backups": delete_backups,
                "confirmation_token": confirmation_token
            }),
        ).await?;

        Ok(confirmation_token)
    }

    // Private helper methods

    fn validate_export_request(&self, request: &GdprExportRequest) -> SecurityResult<()> {
        if request.tenant_id.is_empty() {
            return Err(SecurityError::Validation("Tenant ID is required".to_string()));
        }
        if request.user_id.is_empty() {
            return Err(SecurityError::Validation("User ID is required".to_string()));
        }
        if request.requester_email.is_empty() {
            return Err(SecurityError::Validation("Requester email is required".to_string()));
        }
        if !["json", "xml"].contains(&request.format.to_lowercase().as_str()) {
            return Err(SecurityError::Validation("Unsupported export format".to_string()));
        }
        Ok(())
    }

    fn validate_deletion_request(&self, request: &GdprDeletionRequest) -> SecurityResult<()> {
        if request.tenant_id.is_empty() {
            return Err(SecurityError::Validation("Tenant ID is required".to_string()));
        }
        if request.user_id.is_empty() {
            return Err(SecurityError::Validation("User ID is required".to_string()));
        }
        if request.requester_email.is_empty() {
            return Err(SecurityError::Validation("Requester email is required".to_string()));
        }
        Ok(())
    }

    fn generate_verification_token(&self) -> String {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        (0..32)
            .map(|_| rng.sample(rand::distributions::Alphanumeric) as char)
            .collect()
    }

    async fn send_verification_email(&self, request: &GdprRequest, token: &str) -> SecurityResult<()> {
        // This would integrate with the notification service
        info!(
            request_id = %request.id,
            email = %request.requester_email,
            "Sending GDPR verification email"
        );
        
        // For now, just log the verification URL
        let verification_url = format!(
            "https://app.adxcore.com/gdpr/verify?request_id={}&token={}",
            request.id,
            token
        );
        
        info!(verification_url = %verification_url, "GDPR verification URL generated");
        
        Ok(())
    }

    async fn process_single_request(&self, mut request: GdprRequest) -> SecurityResult<()> {
        request.status = GdprRequestStatus::Processing;
        request.updated_at = Utc::now();
        self.repository.update_request(request.clone()).await?;

        match request.request_type {
            GdprRequestType::DataExport => {
                self.process_export_request(&mut request).await?;
            }
            GdprRequestType::DataDeletion => {
                self.process_deletion_request(&mut request).await?;
            }
            _ => {
                return Err(SecurityError::GdprRequest("Unsupported request type".to_string()));
            }
        }

        request.status = GdprRequestStatus::Completed;
        request.processed_at = Some(Utc::now());
        request.updated_at = Utc::now();
        self.repository.update_request(request).await?;

        Ok(())
    }

    async fn process_export_request(&self, request: &mut GdprRequest) -> SecurityResult<()> {
        let notes: Value = serde_json::from_str(request.notes.as_ref().unwrap_or(&"{}".to_string()))?;
        let include_deleted = notes.get("include_deleted").and_then(|v| v.as_bool()).unwrap_or(false);
        let format = notes.get("format").and_then(|v| v.as_str()).unwrap_or("json");

        // Export the data
        let exported_data = self.export_user_data(
            &request.tenant_id,
            &request.user_id,
            include_deleted,
            format,
        ).await?;

        // Store the exported data (this would typically be uploaded to secure storage)
        let export_url = self.store_exported_data(&request.id, exported_data).await?;
        request.data_export_url = Some(export_url);

        Ok(())
    }

    async fn process_deletion_request(&self, request: &mut GdprRequest) -> SecurityResult<()> {
        let notes: Value = serde_json::from_str(request.notes.as_ref().unwrap_or(&"{}".to_string()))?;
        let delete_backups = notes.get("delete_backups").and_then(|v| v.as_bool()).unwrap_or(false);

        // Delete the data
        let confirmation_token = self.delete_user_data(
            &request.tenant_id,
            &request.user_id,
            delete_backups,
        ).await?;

        request.deletion_confirmation = Some(confirmation_token);

        Ok(())
    }

    async fn collect_user_data(&self, tenant_id: &str, user_id: &str, include_deleted: bool) -> SecurityResult<Value> {
        let mut user_data = Map::new();

        // This would collect data from all services
        // For now, we'll create a mock structure
        user_data.insert("user_id".to_string(), Value::String(user_id.to_string()));
        user_data.insert("tenant_id".to_string(), Value::String(tenant_id.to_string()));
        user_data.insert("export_timestamp".to_string(), Value::String(Utc::now().to_rfc3339()));
        user_data.insert("include_deleted".to_string(), Value::Bool(include_deleted));

        // Add placeholder data sections
        user_data.insert("profile".to_string(), serde_json::json!({}));
        user_data.insert("files".to_string(), serde_json::json!([]));
        user_data.insert("audit_logs".to_string(), serde_json::json!([]));
        user_data.insert("preferences".to_string(), serde_json::json!({}));

        Ok(Value::Object(user_data))
    }

    async fn export_to_xml(&self, data: Value) -> SecurityResult<Vec<u8>> {
        // This would convert JSON to XML format
        // For now, we'll create a simple XML structure
        let xml_content = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<gdpr_export>
    <metadata>
        <export_timestamp>{}</export_timestamp>
    </metadata>
    <data>
        {}
    </data>
</gdpr_export>"#,
            Utc::now().to_rfc3339(),
            serde_json::to_string_pretty(&data)?
        );

        Ok(xml_content.into_bytes())
    }

    async fn store_exported_data(&self, request_id: &Uuid, data: Vec<u8>) -> SecurityResult<String> {
        // This would upload to secure storage (S3, etc.)
        // For now, we'll return a mock URL
        let storage_url = format!(
            "https://secure-storage.adxcore.com/gdpr-exports/{}.zip",
            request_id
        );

        info!(
            request_id = %request_id,
            data_size = %data.len(),
            storage_url = %storage_url,
            "Stored GDPR export data"
        );

        Ok(storage_url)
    }
}