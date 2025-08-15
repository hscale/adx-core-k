use crate::{
    error::{SecurityError, SecurityResult},
    models::{
        ZeroTrustPolicy, ZeroTrustPolicyType, SecurityEvent, SecurityEventType,
        SecurityEventSeverity, SecurityEventStatus
    },
    repositories::ZeroTrustRepository,
    audit::AuditService,
};
use chrono::{DateTime, Utc};
use serde_json::Value;
use std::{collections::HashMap, sync::Arc, net::IpAddr};
use tracing::{info, warn, error};
use uuid::Uuid;

#[derive(Clone)]
pub struct ZeroTrustService {
    repository: Arc<ZeroTrustRepository>,
    audit_service: Arc<AuditService>,
    verify_all_requests: bool,
    certificate_validation: bool,
    mutual_tls: bool,
    network_segmentation: bool,
    device_verification: bool,
}

impl ZeroTrustService {
    pub fn new(
        repository: Arc<ZeroTrustRepository>,
        audit_service: Arc<AuditService>,
        verify_all_requests: bool,
        certificate_validation: bool,
        mutual_tls: bool,
        network_segmentation: bool,
        device_verification: bool,
    ) -> Self {
        Self {
            repository,
            audit_service,
            verify_all_requests,
            certificate_validation,
            mutual_tls,
            network_segmentation,
            device_verification,
        }
    }

    /// Create a new zero trust policy
    pub async fn create_policy(
        &self,
        tenant_id: &str,
        name: &str,
        description: &str,
        policy_type: ZeroTrustPolicyType,
        conditions: Value,
        actions: Value,
        priority: i32,
        created_by: &str,
    ) -> SecurityResult<ZeroTrustPolicy> {
        // Validate policy inputs
        self.validate_policy_inputs(tenant_id, name, &conditions, &actions)?;

        let policy = ZeroTrustPolicy {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            policy_type,
            conditions,
            actions,
            enabled: true,
            priority,
            created_by: created_by.to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let created_policy = self.repository.create_policy(policy).await?;

        // Log policy creation
        self.audit_service.log_security_event(
            tenant_id,
            "zero_trust_policy_created",
            "INFO",
            &format!("Zero trust policy '{}' created", name),
            serde_json::json!({
                "policy_id": created_policy.id,
                "policy_name": name,
                "policy_type": policy_type,
                "priority": priority,
                "created_by": created_by
            }),
        ).await?;

        Ok(created_policy)
    }

    /// Update an existing zero trust policy
    pub async fn update_policy(
        &self,
        policy_id: Uuid,
        name: Option<&str>,
        description: Option<&str>,
        conditions: Option<Value>,
        actions: Option<Value>,
        enabled: Option<bool>,
        priority: Option<i32>,
    ) -> SecurityResult<ZeroTrustPolicy> {
        let mut policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| SecurityError::NotFound("Zero trust policy not found".to_string()))?;

        let original_policy = policy.clone();

        // Update fields
        if let Some(n) = name {
            policy.name = n.to_string();
        }
        if let Some(d) = description {
            policy.description = d.to_string();
        }
        if let Some(c) = conditions {
            self.validate_conditions(&c)?;
            policy.conditions = c;
        }
        if let Some(a) = actions {
            self.validate_actions(&a)?;
            policy.actions = a;
        }
        if let Some(e) = enabled {
            policy.enabled = e;
        }
        if let Some(p) = priority {
            policy.priority = p;
        }

        policy.updated_at = Utc::now();

        let updated_policy = self.repository.update_policy(policy).await?;

        // Log policy update
        self.audit_service.log_security_event(
            &updated_policy.tenant_id,
            "zero_trust_policy_updated",
            "INFO",
            &format!("Zero trust policy '{}' updated", updated_policy.name),
            serde_json::json!({
                "policy_id": policy_id,
                "original": original_policy,
                "updated": updated_policy
            }),
        ).await?;

        Ok(updated_policy)
    }

    /// Delete a zero trust policy
    pub async fn delete_policy(&self, policy_id: Uuid) -> SecurityResult<()> {
        let policy = self.repository.get_policy(policy_id).await?
            .ok_or_else(|| SecurityError::NotFound("Zero trust policy not found".to_string()))?;

        self.repository.delete_policy(policy_id).await?;

        // Log policy deletion
        self.audit_service.log_security_event(
            &policy.tenant_id,
            "zero_trust_policy_deleted",
            "INFO",
            &format!("Zero trust policy '{}' deleted", policy.name),
            serde_json::json!({
                "policy_id": policy_id,
                "policy_name": policy.name
            }),
        ).await?;

        Ok(())
    }

    /// Get all policies for a tenant
    pub async fn get_tenant_policies(&self, tenant_id: &str) -> SecurityResult<Vec<ZeroTrustPolicy>> {
        self.repository.get_tenant_policies(tenant_id).await
    }

    /// Evaluate access request against zero trust policies
    pub async fn evaluate_access_request(
        &self,
        tenant_id: &str,
        user_id: &str,
        device_id: Option<&str>,
        source_ip: IpAddr,
        resource: &str,
        action: &str,
        context: &HashMap<String, Value>,
    ) -> SecurityResult<AccessDecision> {
        // Get applicable policies
        let policies = self.repository.get_active_policies(tenant_id).await?;

        // Sort policies by priority (higher priority first)
        let mut sorted_policies = policies;
        sorted_policies.sort_by(|a, b| b.priority.cmp(&a.priority));

        let mut decision = AccessDecision::default();
        let mut applied_policies = Vec::new();

        // Evaluate each policy
        for policy in sorted_policies {
            if self.policy_applies(&policy, resource, action, context).await? {
                let policy_decision = self.evaluate_policy(
                    &policy,
                    user_id,
                    device_id,
                    source_ip,
                    resource,
                    action,
                    context,
                ).await?;

                applied_policies.push(policy.id);

                // Apply policy decision
                match policy_decision.action {
                    PolicyAction::Allow => {
                        decision.allowed = true;
                        decision.conditions.extend(policy_decision.conditions);
                    }
                    PolicyAction::Deny => {
                        decision.allowed = false;
                        decision.reason = Some(policy_decision.reason);
                        break; // Deny takes precedence
                    }
                    PolicyAction::RequireAdditionalAuth => {
                        decision.requires_additional_auth = true;
                        decision.auth_methods.extend(policy_decision.auth_methods);
                    }
                    PolicyAction::Monitor => {
                        decision.monitor = true;
                    }
                }
            }
        }

        decision.applied_policies = applied_policies;

        // Log access evaluation
        self.audit_service.log_security_event(
            tenant_id,
            "zero_trust_access_evaluated",
            if decision.allowed { "INFO" } else { "WARNING" },
            &format!("Access request evaluated for user {} to resource {}", user_id, resource),
            serde_json::json!({
                "user_id": user_id,
                "device_id": device_id,
                "source_ip": source_ip.to_string(),
                "resource": resource,
                "action": action,
                "decision": decision,
                "applied_policies": applied_policies
            }),
        ).await?;

        Ok(decision)
    }

    /// Verify device trust status
    pub async fn verify_device_trust(
        &self,
        tenant_id: &str,
        device_id: &str,
        device_info: &DeviceInfo,
    ) -> SecurityResult<DeviceTrustStatus> {
        if !self.device_verification {
            return Ok(DeviceTrustStatus::Trusted);
        }

        // Check device registration
        let device_status = self.repository.get_device_status(tenant_id, device_id).await?;

        let trust_status = match device_status {
            Some(status) => {
                // Verify device attributes
                if self.verify_device_attributes(&status, device_info).await? {
                    DeviceTrustStatus::Trusted
                } else {
                    DeviceTrustStatus::Suspicious
                }
            }
            None => DeviceTrustStatus::Unknown,
        };

        // Log device verification
        self.audit_service.log_security_event(
            tenant_id,
            "device_trust_verified",
            match trust_status {
                DeviceTrustStatus::Trusted => "INFO",
                DeviceTrustStatus::Suspicious => "WARNING",
                DeviceTrustStatus::Unknown => "WARNING",
                DeviceTrustStatus::Blocked => "ERROR",
            },
            &format!("Device trust verification for device {}", device_id),
            serde_json::json!({
                "device_id": device_id,
                "trust_status": trust_status,
                "device_info": device_info
            }),
        ).await?;

        Ok(trust_status)
    }

    /// Validate network access based on segmentation rules
    pub async fn validate_network_access(
        &self,
        tenant_id: &str,
        source_ip: IpAddr,
        destination: &str,
        port: u16,
        protocol: &str,
    ) -> SecurityResult<NetworkAccessDecision> {
        if !self.network_segmentation {
            return Ok(NetworkAccessDecision::Allow);
        }

        // Get network policies
        let network_policies = self.repository.get_network_policies(tenant_id).await?;

        // Evaluate network access
        for policy in network_policies {
            if self.network_policy_matches(&policy, source_ip, destination, port, protocol).await? {
                let decision = self.evaluate_network_policy(&policy, source_ip, destination, port, protocol).await?;
                
                // Log network access decision
                self.audit_service.log_security_event(
                    tenant_id,
                    "network_access_evaluated",
                    match decision {
                        NetworkAccessDecision::Allow => "INFO",
                        NetworkAccessDecision::Deny => "WARNING",
                        NetworkAccessDecision::Monitor => "INFO",
                    },
                    &format!("Network access from {} to {}:{}", source_ip, destination, port),
                    serde_json::json!({
                        "source_ip": source_ip.to_string(),
                        "destination": destination,
                        "port": port,
                        "protocol": protocol,
                        "decision": decision,
                        "policy_id": policy.id
                    }),
                ).await?;

                return Ok(decision);
            }
        }

        // Default deny for network segmentation
        Ok(NetworkAccessDecision::Deny)
    }

    /// Create a security event
    pub async fn create_security_event(
        &self,
        tenant_id: &str,
        event_type: SecurityEventType,
        severity: SecurityEventSeverity,
        source_ip: Option<IpAddr>,
        user_id: Option<&str>,
        device_id: Option<&str>,
        resource: Option<&str>,
        description: &str,
        details: Value,
    ) -> SecurityResult<Uuid> {
        let event = SecurityEvent {
            id: Uuid::new_v4(),
            tenant_id: tenant_id.to_string(),
            event_type,
            severity,
            source_ip: source_ip.map(|ip| ip.to_string()),
            user_id: user_id.map(|s| s.to_string()),
            device_id: device_id.map(|s| s.to_string()),
            resource: resource.map(|s| s.to_string()),
            description: description.to_string(),
            details,
            status: SecurityEventStatus::Open,
            resolved_at: None,
            resolved_by: None,
            created_at: Utc::now(),
        };

        let created_event = self.repository.create_security_event(event).await?;

        // Log security event creation
        self.audit_service.log_security_event(
            tenant_id,
            "security_event_created",
            match severity {
                SecurityEventSeverity::Critical => "CRITICAL",
                SecurityEventSeverity::High => "HIGH",
                SecurityEventSeverity::Medium => "MEDIUM",
                SecurityEventSeverity::Low => "LOW",
                SecurityEventSeverity::Info => "INFO",
            },
            description,
            serde_json::json!({
                "event_id": created_event.id,
                "event_type": event_type,
                "severity": severity,
                "source_ip": source_ip.map(|ip| ip.to_string()),
                "user_id": user_id,
                "device_id": device_id,
                "resource": resource
            }),
        ).await?;

        Ok(created_event.id)
    }

    /// Get security events for a tenant
    pub async fn get_security_events(
        &self,
        tenant_id: &str,
        event_type: Option<SecurityEventType>,
        severity: Option<SecurityEventSeverity>,
        status: Option<SecurityEventStatus>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
        page: i32,
        page_size: i32,
    ) -> SecurityResult<Vec<SecurityEvent>> {
        self.repository.get_security_events(
            tenant_id,
            event_type,
            severity,
            status,
            start_date,
            end_date,
            page,
            page_size,
        ).await
    }

    /// Resolve a security event
    pub async fn resolve_security_event(
        &self,
        event_id: Uuid,
        resolved_by: &str,
        resolution_notes: Option<&str>,
    ) -> SecurityResult<()> {
        let mut event = self.repository.get_security_event(event_id).await?
            .ok_or_else(|| SecurityError::NotFound("Security event not found".to_string()))?;

        event.status = SecurityEventStatus::Resolved;
        event.resolved_at = Some(Utc::now());
        event.resolved_by = Some(resolved_by.to_string());

        if let Some(notes) = resolution_notes {
            let mut details = event.details.as_object().unwrap_or(&serde_json::Map::new()).clone();
            details.insert("resolution_notes".to_string(), Value::String(notes.to_string()));
            event.details = Value::Object(details);
        }

        self.repository.update_security_event(event.clone()).await?;

        // Log event resolution
        self.audit_service.log_security_event(
            &event.tenant_id,
            "security_event_resolved",
            "INFO",
            &format!("Security event {} resolved", event_id),
            serde_json::json!({
                "event_id": event_id,
                "resolved_by": resolved_by,
                "resolution_notes": resolution_notes
            }),
        ).await?;

        Ok(())
    }

    // Private helper methods

    fn validate_policy_inputs(
        &self,
        tenant_id: &str,
        name: &str,
        conditions: &Value,
        actions: &Value,
    ) -> SecurityResult<()> {
        if tenant_id.is_empty() {
            return Err(SecurityError::Validation("Tenant ID is required".to_string()));
        }
        if name.is_empty() {
            return Err(SecurityError::Validation("Policy name is required".to_string()));
        }
        self.validate_conditions(conditions)?;
        self.validate_actions(actions)?;
        Ok(())
    }

    fn validate_conditions(&self, conditions: &Value) -> SecurityResult<()> {
        // Validate that conditions is a proper JSON object with required fields
        if !conditions.is_object() {
            return Err(SecurityError::Validation("Conditions must be a JSON object".to_string()));
        }
        Ok(())
    }

    fn validate_actions(&self, actions: &Value) -> SecurityResult<()> {
        // Validate that actions is a proper JSON object with required fields
        if !actions.is_object() {
            return Err(SecurityError::Validation("Actions must be a JSON object".to_string()));
        }
        Ok(())
    }

    async fn policy_applies(
        &self,
        policy: &ZeroTrustPolicy,
        resource: &str,
        action: &str,
        context: &HashMap<String, Value>,
    ) -> SecurityResult<bool> {
        // Check if policy conditions match the request
        let conditions = policy.conditions.as_object()
            .ok_or_else(|| SecurityError::ZeroTrust("Invalid policy conditions".to_string()))?;

        // Check resource pattern
        if let Some(resource_pattern) = conditions.get("resource_pattern") {
            if let Some(pattern) = resource_pattern.as_str() {
                if !self.matches_pattern(resource, pattern) {
                    return Ok(false);
                }
            }
        }

        // Check action pattern
        if let Some(action_pattern) = conditions.get("action_pattern") {
            if let Some(pattern) = action_pattern.as_str() {
                if !self.matches_pattern(action, pattern) {
                    return Ok(false);
                }
            }
        }

        // Check context conditions
        if let Some(context_conditions) = conditions.get("context") {
            if let Some(ctx_obj) = context_conditions.as_object() {
                for (key, expected_value) in ctx_obj {
                    if let Some(actual_value) = context.get(key) {
                        if actual_value != expected_value {
                            return Ok(false);
                        }
                    } else {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    async fn evaluate_policy(
        &self,
        policy: &ZeroTrustPolicy,
        user_id: &str,
        device_id: Option<&str>,
        source_ip: IpAddr,
        resource: &str,
        action: &str,
        context: &HashMap<String, Value>,
    ) -> SecurityResult<PolicyDecision> {
        let actions = policy.actions.as_object()
            .ok_or_else(|| SecurityError::ZeroTrust("Invalid policy actions".to_string()))?;

        let mut decision = PolicyDecision::default();

        // Determine primary action
        if let Some(action_type) = actions.get("action") {
            if let Some(action_str) = action_type.as_str() {
                decision.action = match action_str {
                    "allow" => PolicyAction::Allow,
                    "deny" => PolicyAction::Deny,
                    "require_auth" => PolicyAction::RequireAdditionalAuth,
                    "monitor" => PolicyAction::Monitor,
                    _ => PolicyAction::Deny,
                };
            }
        }

        // Set reason for deny actions
        if matches!(decision.action, PolicyAction::Deny) {
            decision.reason = actions.get("reason")
                .and_then(|r| r.as_str())
                .unwrap_or("Access denied by zero trust policy")
                .to_string();
        }

        // Set additional conditions
        if let Some(conditions) = actions.get("conditions") {
            if let Some(cond_array) = conditions.as_array() {
                for condition in cond_array {
                    if let Some(cond_str) = condition.as_str() {
                        decision.conditions.push(cond_str.to_string());
                    }
                }
            }
        }

        // Set required auth methods
        if let Some(auth_methods) = actions.get("auth_methods") {
            if let Some(methods_array) = auth_methods.as_array() {
                for method in methods_array {
                    if let Some(method_str) = method.as_str() {
                        decision.auth_methods.push(method_str.to_string());
                    }
                }
            }
        }

        Ok(decision)
    }

    async fn verify_device_attributes(
        &self,
        stored_status: &DeviceStatus,
        current_info: &DeviceInfo,
    ) -> SecurityResult<bool> {
        // Compare device attributes to detect changes
        Ok(stored_status.os_version == current_info.os_version
            && stored_status.browser_version == current_info.browser_version
            && stored_status.screen_resolution == current_info.screen_resolution)
    }

    async fn network_policy_matches(
        &self,
        policy: &ZeroTrustPolicy,
        source_ip: IpAddr,
        destination: &str,
        port: u16,
        protocol: &str,
    ) -> SecurityResult<bool> {
        // Check if network policy matches the request
        let conditions = policy.conditions.as_object()
            .ok_or_else(|| SecurityError::ZeroTrust("Invalid network policy conditions".to_string()))?;

        // Check source IP range
        if let Some(source_range) = conditions.get("source_ip_range") {
            if let Some(range_str) = source_range.as_str() {
                if !self.ip_in_range(source_ip, range_str)? {
                    return Ok(false);
                }
            }
        }

        // Check destination pattern
        if let Some(dest_pattern) = conditions.get("destination_pattern") {
            if let Some(pattern) = dest_pattern.as_str() {
                if !self.matches_pattern(destination, pattern) {
                    return Ok(false);
                }
            }
        }

        // Check port range
        if let Some(port_range) = conditions.get("port_range") {
            if let Some(range_str) = port_range.as_str() {
                if !self.port_in_range(port, range_str)? {
                    return Ok(false);
                }
            }
        }

        // Check protocol
        if let Some(proto_pattern) = conditions.get("protocol") {
            if let Some(pattern) = proto_pattern.as_str() {
                if !self.matches_pattern(protocol, pattern) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    async fn evaluate_network_policy(
        &self,
        policy: &ZeroTrustPolicy,
        source_ip: IpAddr,
        destination: &str,
        port: u16,
        protocol: &str,
    ) -> SecurityResult<NetworkAccessDecision> {
        let actions = policy.actions.as_object()
            .ok_or_else(|| SecurityError::ZeroTrust("Invalid network policy actions".to_string()))?;

        if let Some(action_type) = actions.get("action") {
            if let Some(action_str) = action_type.as_str() {
                return Ok(match action_str {
                    "allow" => NetworkAccessDecision::Allow,
                    "deny" => NetworkAccessDecision::Deny,
                    "monitor" => NetworkAccessDecision::Monitor,
                    _ => NetworkAccessDecision::Deny,
                });
            }
        }

        Ok(NetworkAccessDecision::Deny)
    }

    fn matches_pattern(&self, value: &str, pattern: &str) -> bool {
        // Simple pattern matching with wildcards
        if pattern == "*" {
            return true;
        }
        
        if pattern.contains('*') {
            // Convert glob pattern to regex
            let regex_pattern = pattern.replace('*', ".*");
            if let Ok(regex) = regex::Regex::new(&regex_pattern) {
                return regex.is_match(value);
            }
        }
        
        value == pattern
    }

    fn ip_in_range(&self, ip: IpAddr, range: &str) -> SecurityResult<bool> {
        // Parse CIDR range and check if IP is in range
        if let Ok(network) = ipnetwork::IpNetwork::from_str(range) {
            Ok(network.contains(ip))
        } else {
            Err(SecurityError::Validation(format!("Invalid IP range: {}", range)))
        }
    }

    fn port_in_range(&self, port: u16, range: &str) -> SecurityResult<bool> {
        // Parse port range (e.g., "80-443" or "22")
        if range.contains('-') {
            let parts: Vec<&str> = range.split('-').collect();
            if parts.len() == 2 {
                if let (Ok(start), Ok(end)) = (parts[0].parse::<u16>(), parts[1].parse::<u16>()) {
                    return Ok(port >= start && port <= end);
                }
            }
        } else if let Ok(single_port) = range.parse::<u16>() {
            return Ok(port == single_port);
        }
        
        Err(SecurityError::Validation(format!("Invalid port range: {}", range)))
    }
}

// Supporting types and enums

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AccessDecision {
    pub allowed: bool,
    pub reason: Option<String>,
    pub requires_additional_auth: bool,
    pub auth_methods: Vec<String>,
    pub conditions: Vec<String>,
    pub monitor: bool,
    pub applied_policies: Vec<Uuid>,
}

impl Default for AccessDecision {
    fn default() -> Self {
        Self {
            allowed: false,
            reason: None,
            requires_additional_auth: false,
            auth_methods: Vec::new(),
            conditions: Vec::new(),
            monitor: false,
            applied_policies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PolicyDecision {
    pub action: PolicyAction,
    pub reason: String,
    pub conditions: Vec<String>,
    pub auth_methods: Vec<String>,
}

impl Default for PolicyDecision {
    fn default() -> Self {
        Self {
            action: PolicyAction::Deny,
            reason: "Default deny".to_string(),
            conditions: Vec::new(),
            auth_methods: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    RequireAdditionalAuth,
    Monitor,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum DeviceTrustStatus {
    Trusted,
    Suspicious,
    Unknown,
    Blocked,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NetworkAccessDecision {
    Allow,
    Deny,
    Monitor,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceInfo {
    pub os_version: String,
    pub browser_version: String,
    pub screen_resolution: String,
    pub timezone: String,
    pub language: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceStatus {
    pub device_id: String,
    pub tenant_id: String,
    pub os_version: String,
    pub browser_version: String,
    pub screen_resolution: String,
    pub last_seen: DateTime<Utc>,
    pub trust_score: f32,
}

// External crate imports for IP and regex functionality
use regex;
use ipnetwork;
use std::str::FromStr;