// RBAC Service - Fast permission checks and simple operations
// Simple CRUD operations that don't require Temporal workflows

use crate::rbac::types::*;
use crate::rbac::workflows::*;
use adx_shared::{DatabaseManager, TenantId, UserId};
use chrono::{DateTime, Timelike, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// High-performance RBAC service for fast permission checks
/// Complex operations delegate to Temporal workflows
#[derive(Debug)]
pub struct RbacService {
    // Database connection for direct queries
    db: Arc<DatabaseManager>,
    // In-memory cache for fast permission lookups
    permission_cache: Arc<RwLock<HashMap<String, PermissionCheckResponse>>>,
    // Role hierarchy cache
    role_cache: Arc<RwLock<HashMap<Uuid, Role>>>,
}

impl RbacService {
    pub fn new(db: Arc<DatabaseManager>) -> Self {
        Self {
            db,
            permission_cache: Arc::new(RwLock::new(HashMap::new())),
            role_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    // ========================================================================
    // FAST PERMISSION CHECKS (< 10ms target)
    // ========================================================================

    /// Fast permission check - core security function
    /// This is called on every API request, so it must be extremely fast
    pub async fn check_permission(
        &self,
        request: PermissionCheckRequest,
    ) -> Result<PermissionCheckResponse, RbacError> {
        let start_time = std::time::Instant::now();

        // Create cache key
        let cache_key = format!(
            "{}:{}:{}:{}",
            request.user_id, request.tenant_id, request.resource, request.action
        );

        // Check cache first (most permissions are repeated)
        {
            let cache = self.permission_cache.read().await;
            if let Some(cached_response) = cache.get(&cache_key) {
                // Update duration for cache hit
                let mut response = cached_response.clone();
                response.check_duration_ms = start_time.elapsed().as_millis() as u32;
                return Ok(response);
            }
        }

        // Cache miss - perform full permission check
        let result = self.perform_permission_check(&request).await?;

        // Cache the result (with TTL in production)
        {
            let mut cache = self.permission_cache.write().await;
            cache.insert(cache_key, result.clone());
        }

        let mut response = result;
        response.check_duration_ms = start_time.elapsed().as_millis() as u32;

        Ok(response)
    }

    /// Performs the actual permission check against database
    async fn perform_permission_check(
        &self,
        request: &PermissionCheckRequest,
    ) -> Result<PermissionCheckResponse, RbacError> {
        // Step 1: Get user roles for the tenant
        let user_roles = self
            .get_user_roles_internal(request.user_id, request.tenant_id)
            .await?;

        if user_roles.is_empty() {
            return Ok(PermissionCheckResponse {
                allowed: false,
                reason: Some("User has no roles in this tenant".to_string()),
                matched_permissions: vec![],
                check_duration_ms: 0, // Will be set by caller
            });
        }

        // Step 2: Get all permissions for these roles
        let mut all_permissions = Vec::new();
        for role in &user_roles {
            all_permissions.extend(role.permissions.clone());
        }

        // Step 3: Check if any permission grants access
        let mut matched_permissions = Vec::new();
        for permission in &all_permissions {
            if self
                .permission_matches(permission, &request.resource, &request.action)
                .await?
            {
                // Check permission conditions
                if self
                    .evaluate_permission_conditions(
                        permission,
                        request.user_id,
                        request.tenant_id,
                        &request.context,
                    )
                    .await?
                {
                    matched_permissions.push(permission.clone());
                }
            }
        }

        let allowed = !matched_permissions.is_empty();
        let reason = if !allowed {
            Some(format!(
                "No matching permissions found for action '{}' on resource '{}'",
                request.action, request.resource
            ))
        } else {
            None
        };

        Ok(PermissionCheckResponse {
            allowed,
            reason,
            matched_permissions,
            check_duration_ms: 0, // Will be set by caller
        })
    }

    /// Checks if a permission matches the requested resource and action
    async fn permission_matches(
        &self,
        permission: &Permission,
        resource: &str,
        action: &str,
    ) -> Result<bool, RbacError> {
        // Exact match
        if permission.resource == resource && permission.action == action {
            return Ok(true);
        }

        // Wildcard matching
        if permission.resource == "*" || permission.action == "*" {
            return Ok(true);
        }

        // Pattern matching (e.g., "files:*", "*:read")
        if permission.resource.ends_with("*") {
            let prefix = &permission.resource[..permission.resource.len() - 1];
            if resource.starts_with(prefix) && permission.action == action {
                return Ok(true);
            }
        }

        if permission.action.ends_with("*") {
            let prefix = &permission.action[..permission.action.len() - 1];
            if action.starts_with(prefix) && permission.resource == resource {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Evaluates permission conditions (e.g., resource ownership)
    async fn evaluate_permission_conditions(
        &self,
        permission: &Permission,
        user_id: UserId,
        tenant_id: TenantId,
        context: &Option<serde_json::Value>,
    ) -> Result<bool, RbacError> {
        for condition in &permission.conditions {
            match condition {
                PermissionCondition::TenantOwner => {
                    // Check if user owns the tenant
                    if !self.is_tenant_owner(user_id, tenant_id).await? {
                        return Ok(false);
                    }
                }
                PermissionCondition::ResourceOwner => {
                    // Check if user owns the specific resource
                    if let Some(context) = context {
                        if let Some(resource_owner_id) = context.get("owner_id") {
                            if resource_owner_id.as_str() != Some(&user_id.to_string()) {
                                return Ok(false);
                            }
                        }
                    }
                }
                PermissionCondition::DepartmentMember(department) => {
                    // Check department membership
                    if !self
                        .is_department_member(user_id, tenant_id, department)
                        .await?
                    {
                        return Ok(false);
                    }
                }
                PermissionCondition::CustomCondition(condition_code) => {
                    // Evaluate custom business logic
                    if !self
                        .evaluate_custom_condition(condition_code, user_id, tenant_id, context)
                        .await?
                    {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    // ========================================================================
    // USER ROLES QUERIES (Fast lookups)
    // ========================================================================

    /// Gets all roles for a user in a specific tenant
    pub async fn get_user_roles(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<UserRolesResponse, RbacError> {
        let roles = self.get_user_roles_internal(user_id, tenant_id).await?;

        // Collect all effective permissions
        let mut effective_permissions = Vec::new();
        for role in &roles {
            effective_permissions.extend(role.permissions.clone());
        }

        // Remove duplicates
        effective_permissions.sort_by(|a, b| a.id.cmp(&b.id));
        effective_permissions.dedup_by(|a, b| a.id == b.id);

        Ok(UserRolesResponse {
            user_id,
            roles,
            effective_permissions,
            tenant_id,
        })
    }

    /// Internal method to get user roles with caching
    async fn get_user_roles_internal(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<Vec<Role>, RbacError> {
        // TODO: In production, this would query the database:
        // 1. Get user_roles records for user_id and tenant_id where is_active=true
        // 2. Join with roles table to get full role details
        // 3. Join with role_permissions to get permissions
        // 4. Build role hierarchy if needed

        // For now, return mock data
        let mock_role = Role {
            id: Uuid::new_v4(),
            name: "User".to_string(),
            description: "Standard user role".to_string(),
            tenant_id,
            parent_role_id: None,
            permissions: vec![Permission {
                id: Uuid::new_v4(),
                resource: "files".to_string(),
                action: "read".to_string(),
                conditions: vec![PermissionCondition::ResourceOwner],
                created_at: Utc::now(),
            }],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        Ok(vec![mock_role])
    }

    /// Lists all permissions for a specific role
    pub async fn get_role_permissions(
        &self,
        role_id: Uuid,
        tenant_id: TenantId,
    ) -> Result<Vec<Permission>, RbacError> {
        // Check cache first
        {
            let cache = self.role_cache.read().await;
            if let Some(role) = cache.get(&role_id) {
                if role.tenant_id == tenant_id {
                    return Ok(role.permissions.clone());
                }
            }
        }

        // TODO: Query database for role permissions
        // For now, return empty permissions
        Ok(vec![])
    }

    // ========================================================================
    // HELPER METHODS
    // ========================================================================

    /// Checks if user is tenant owner
    async fn is_tenant_owner(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
    ) -> Result<bool, RbacError> {
        // TODO: Query tenant ownership
        Ok(false) // Placeholder
    }

    /// Checks if user is member of department
    async fn is_department_member(
        &self,
        user_id: UserId,
        tenant_id: TenantId,
        department: &str,
    ) -> Result<bool, RbacError> {
        // TODO: Query department membership
        Ok(false) // Placeholder
    }

    /// Evaluates custom permission condition
    async fn evaluate_custom_condition(
        &self,
        condition_code: &str,
        user_id: UserId,
        tenant_id: TenantId,
        context: &Option<serde_json::Value>,
    ) -> Result<bool, RbacError> {
        // TODO: Implement custom condition evaluation engine
        // This could be a simple script engine or business rules engine
        match condition_code {
            "business_hours" => {
                // Check if current time is during business hours
                let now = Utc::now();
                let hour = now.hour();
                Ok(hour >= 9 && hour <= 17)
            }
            "ip_whitelist" => {
                // Check if request comes from whitelisted IP
                // TODO: Get IP from context
                Ok(true) // Placeholder
            }
            _ => {
                tracing::warn!("Unknown custom condition: {}", condition_code);
                Ok(false)
            }
        }
    }

    // ========================================================================
    // CACHE MANAGEMENT
    // ========================================================================

    /// Invalidates permission cache for a user
    pub async fn invalidate_user_cache(&self, user_id: UserId) {
        let mut cache = self.permission_cache.write().await;
        cache.retain(|key, _| !key.starts_with(&user_id.to_string()));
    }

    /// Invalidates permission cache for a tenant
    pub async fn invalidate_tenant_cache(&self, tenant_id: TenantId) {
        let mut cache = self.permission_cache.write().await;
        cache.retain(|key, _| !key.contains(&tenant_id.to_string()));
    }

    /// Clears all caches (for cache refresh)
    pub async fn clear_all_caches(&self) {
        let mut permission_cache = self.permission_cache.write().await;
        let mut role_cache = self.role_cache.write().await;
        permission_cache.clear();
        role_cache.clear();
    }

    // ========================================================================
    // WORKFLOW INTEGRATION
    // ========================================================================

    /// Triggers role assignment workflow for complex operations
    pub async fn assign_role_workflow(
        &self,
        input: RoleAssignmentInput,
    ) -> Result<String, RbacError> {
        // Start the workflow (simplified version for now)
        let workflow_id = format!("role_assignment_{}", Uuid::new_v4());

        // Execute the workflow
        let _result = role_assignment_workflow(input)
            .await
            .map_err(|e| RbacError::WorkflowError(e.to_string()))?;

        tracing::info!("Started role assignment workflow: {}", workflow_id);
        Ok(workflow_id)
    }

    /// Triggers permission audit workflow
    pub async fn audit_permissions_workflow(
        &self,
        input: PermissionAuditInput,
    ) -> Result<String, RbacError> {
        let workflow_id = format!("permission_audit_{}", Uuid::new_v4());

        // Execute the workflow
        let _result = permission_audit_workflow(input)
            .await
            .map_err(|e| RbacError::WorkflowError(e.to_string()))?;

        tracing::info!("Started permission audit workflow: {}", workflow_id);
        Ok(workflow_id)
    }

    /// Triggers access review workflow
    pub async fn access_review_workflow(
        &self,
        input: AccessReviewInput,
    ) -> Result<String, RbacError> {
        let workflow_id = format!("access_review_{}", Uuid::new_v4());

        // Execute the workflow
        let _result = access_review_workflow(input)
            .await
            .map_err(|e| RbacError::WorkflowError(e.to_string()))?;

        tracing::info!("Started access review workflow: {}", workflow_id);
        Ok(workflow_id)
    }

    /// Triggers security incident workflow
    pub async fn security_incident_workflow(
        &self,
        input: SecurityIncidentInput,
    ) -> Result<String, RbacError> {
        let workflow_id = format!("security_incident_{}", Uuid::new_v4());

        // Execute the workflow
        let _result = security_incident_workflow(input)
            .await
            .map_err(|e| RbacError::WorkflowError(e.to_string()))?;

        tracing::info!("Started security incident workflow: {}", workflow_id);
        Ok(workflow_id)
    }
}
