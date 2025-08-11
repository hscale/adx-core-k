use std::sync::Arc;
use anyhow::{Result, anyhow};
use chrono::Utc;

use crate::models::*;
use crate::repository_traits::{TenantRepository, TenantMembershipRepository};
use adx_shared::types::{TenantId, UserId};

pub struct TenantService {
    tenant_repo: Arc<dyn TenantRepository>,
    membership_repo: Arc<dyn TenantMembershipRepository>,
}

impl TenantService {
    pub fn new(
        tenant_repo: Arc<dyn TenantRepository>,
        membership_repo: Arc<dyn TenantMembershipRepository>,
    ) -> Self {
        Self {
            tenant_repo,
            membership_repo,
        }
    }

    // Tenant CRUD operations
    pub async fn create_tenant(&self, request: CreateTenantRequest) -> Result<Tenant> {
        // Check if tenant name already exists
        if let Some(_) = self.tenant_repo.find_by_name(&request.name).await? {
            return Err(anyhow!("Tenant with name '{}' already exists", request.name));
        }

        let tenant = Tenant {
            id: String::new(), // Will be generated in repository
            name: request.name,
            slug: String::new(), // Will be generated in repository
            admin_email: request.admin_email,
            subscription_tier: request.subscription_tier.unwrap_or_default(),
            isolation_level: request.isolation_level.unwrap_or_default(),
            quotas: Default::default(),
            features: request.features.unwrap_or_default(),
            settings: request.settings.unwrap_or_default(),
            status: TenantStatus::Active,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.tenant_repo.create(&tenant).await
    }

    pub async fn get_tenant(&self, id: &TenantId) -> Result<Option<Tenant>> {
        self.tenant_repo.find_by_id(id).await
    }

    pub async fn get_tenant_by_slug(&self, slug: &str) -> Result<Option<Tenant>> {
        self.tenant_repo.find_by_slug(slug).await
    }

    pub async fn list_tenants(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Tenant>> {
        self.tenant_repo.list(limit, offset).await
    }

    pub async fn update_tenant(&self, id: &TenantId, request: UpdateTenantRequest) -> Result<Tenant> {
        let mut tenant = self.tenant_repo.find_by_id(id).await?
            .ok_or_else(|| anyhow!("Tenant not found"))?;

        if let Some(name) = request.name {
            // Check if new name conflicts with existing tenant
            if let Some(existing) = self.tenant_repo.find_by_name(&name).await? {
                if existing.id != tenant.id {
                    return Err(anyhow!("Tenant with name '{}' already exists", name));
                }
            }
            tenant.name = name;
        }

        if let Some(subscription_tier) = request.subscription_tier {
            tenant.subscription_tier = subscription_tier;
        }

        if let Some(quotas) = request.quotas {
            tenant.quotas = quotas;
        }

        if let Some(features) = request.features {
            tenant.features = features;
        }

        if let Some(settings) = request.settings {
            tenant.settings = settings;
        }

        if let Some(status) = request.status {
            tenant.status = status;
        }

        self.tenant_repo.update(&tenant).await
    }

    pub async fn delete_tenant(&self, id: &TenantId) -> Result<()> {
        // Check if tenant exists
        if self.tenant_repo.find_by_id(id).await?.is_none() {
            return Err(anyhow!("Tenant not found"));
        }

        // TODO: In a real implementation, we would need to handle cascading deletes
        // and cleanup of tenant data, which should be done through a workflow
        self.tenant_repo.delete(id).await
    }

    // Tenant membership operations
    pub async fn create_membership(&self, tenant_id: &TenantId, request: CreateMembershipRequest) -> Result<TenantMembership> {
        // Verify tenant exists
        if self.tenant_repo.find_by_id(tenant_id).await?.is_none() {
            return Err(anyhow!("Tenant not found"));
        }

        // Check if membership already exists
        if let Some(_) = self.membership_repo.find_by_tenant_and_user(tenant_id, &request.user_id).await? {
            return Err(anyhow!("User is already a member of this tenant"));
        }

        let membership = TenantMembership {
            id: String::new(), // Will be generated in repository
            tenant_id: tenant_id.clone(),
            user_id: request.user_id,
            role: request.role,
            permissions: request.permissions.unwrap_or_default(),
            status: MembershipStatus::Active,
            invited_by: None, // TODO: Get from request context
            invited_at: Utc::now(),
            joined_at: Some(Utc::now()),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.membership_repo.create(&membership).await
    }

    pub async fn get_membership(&self, id: &str) -> Result<Option<TenantMembership>> {
        self.membership_repo.find_by_id(id).await
    }

    pub async fn list_tenant_members(&self, tenant_id: &TenantId) -> Result<Vec<TenantMembership>> {
        self.membership_repo.list_by_tenant(tenant_id).await
    }

    pub async fn list_user_memberships(&self, user_id: &UserId) -> Result<Vec<TenantMembership>> {
        self.membership_repo.list_by_user(user_id).await
    }

    pub async fn update_membership(&self, id: &str, request: UpdateMembershipRequest) -> Result<TenantMembership> {
        let mut membership = self.membership_repo.find_by_id(id).await?
            .ok_or_else(|| anyhow!("Membership not found"))?;

        if let Some(role) = request.role {
            membership.role = role;
        }

        if let Some(permissions) = request.permissions {
            membership.permissions = permissions;
        }

        if let Some(status) = request.status {
            membership.status = status;
        }

        self.membership_repo.update(&membership).await
    }

    pub async fn delete_membership(&self, id: &str) -> Result<()> {
        // Check if membership exists
        if self.membership_repo.find_by_id(id).await?.is_none() {
            return Err(anyhow!("Membership not found"));
        }

        self.membership_repo.delete(id).await
    }

    // Tenant switching operations
    pub async fn switch_tenant(&self, user_id: &UserId, request: SwitchTenantRequest) -> Result<SwitchTenantResponse> {
        // Verify user has access to target tenant
        let membership = self.membership_repo
            .find_by_tenant_and_user(&request.target_tenant_id, user_id)
            .await?
            .ok_or_else(|| anyhow!("User does not have access to target tenant"))?;

        if membership.status != MembershipStatus::Active {
            return Err(anyhow!("User membership is not active"));
        }

        // Get tenant information
        let tenant = self.tenant_repo
            .find_by_id(&request.target_tenant_id)
            .await?
            .ok_or_else(|| anyhow!("Target tenant not found"))?;

        if tenant.status != TenantStatus::Active {
            return Err(anyhow!("Target tenant is not active"));
        }

        // Build tenant context
        let tenant_context = TenantContext {
            tenant_id: tenant.id.clone(),
            tenant_name: tenant.name.clone(),
            tenant_slug: tenant.slug.clone(),
            subscription_tier: tenant.subscription_tier.clone(),
            features: tenant.features.clone(),
            quotas: tenant.quotas.clone(),
            settings: tenant.settings.clone(),
            user_role: membership.role.clone(),
            user_permissions: membership.permissions.clone(),
        };

        Ok(SwitchTenantResponse {
            success: true,
            new_tenant_id: request.target_tenant_id,
            new_session_id: None, // TODO: Generate new session ID
            tenant_context,
        })
    }

    pub async fn get_tenant_context(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<TenantContext> {
        // Get tenant information
        let tenant = self.tenant_repo
            .find_by_id(tenant_id)
            .await?
            .ok_or_else(|| anyhow!("Tenant not found"))?;

        // Get user membership
        let membership = self.membership_repo
            .find_by_tenant_and_user(tenant_id, user_id)
            .await?
            .ok_or_else(|| anyhow!("User does not have access to tenant"))?;

        Ok(TenantContext {
            tenant_id: tenant.id,
            tenant_name: tenant.name,
            tenant_slug: tenant.slug,
            subscription_tier: tenant.subscription_tier,
            features: tenant.features,
            quotas: tenant.quotas,
            settings: tenant.settings,
            user_role: membership.role,
            user_permissions: membership.permissions,
        })
    }

    // Validation helpers
    pub async fn validate_tenant_access(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<bool> {
        match self.membership_repo.find_by_tenant_and_user(tenant_id, user_id).await? {
            Some(membership) => Ok(membership.status == MembershipStatus::Active),
            None => Ok(false),
        }
    }

    pub async fn validate_tenant_permission(&self, tenant_id: &TenantId, user_id: &UserId, permission: &str) -> Result<bool> {
        match self.membership_repo.find_by_tenant_and_user(tenant_id, user_id).await? {
            Some(membership) => {
                if membership.status != MembershipStatus::Active {
                    return Ok(false);
                }

                // Check direct permissions
                if membership.permissions.contains(&permission.to_string()) {
                    return Ok(true);
                }

                // Check role-based permissions
                match membership.role {
                    TenantRole::Owner => Ok(true), // Owners have all permissions
                    TenantRole::Admin => {
                        // Admins have most permissions except owner-specific ones
                        Ok(!permission.starts_with("owner:"))
                    }
                    TenantRole::Member => {
                        // Members have basic permissions
                        Ok(permission.starts_with("tenant:read") || permission.starts_with("user:read"))
                    }
                    TenantRole::Guest => {
                        // Guests have very limited permissions
                        Ok(permission == "tenant:read")
                    }
                }
            }
            None => Ok(false),
        }
    }
}