use async_trait::async_trait;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::models::*;
use adx_shared::types::{TenantId, UserId};

#[async_trait]
pub trait TenantRepository: Send + Sync {
    async fn create(&self, tenant: &Tenant) -> Result<Tenant>;
    async fn find_by_id(&self, id: &TenantId) -> Result<Option<Tenant>>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>>;
    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>>;
    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Tenant>>;
    async fn update(&self, tenant: &Tenant) -> Result<Tenant>;
    async fn delete(&self, id: &TenantId) -> Result<()>;
    async fn count(&self) -> Result<u64>;
}

#[async_trait]
pub trait TenantMembershipRepository: Send + Sync {
    async fn create(&self, membership: &TenantMembership) -> Result<TenantMembership>;
    async fn find_by_id(&self, id: &str) -> Result<Option<TenantMembership>>;
    async fn find_by_tenant_and_user(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<Option<TenantMembership>>;
    async fn list_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<TenantMembership>>;
    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<TenantMembership>>;
    async fn update(&self, membership: &TenantMembership) -> Result<TenantMembership>;
    async fn delete(&self, id: &str) -> Result<()>;
}

// Mock implementations for development
pub struct MockTenantRepository {
    tenants: Arc<Mutex<HashMap<String, Tenant>>>,
}

impl MockTenantRepository {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn generate_slug(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

#[async_trait]
impl TenantRepository for MockTenantRepository {
    async fn create(&self, tenant: &Tenant) -> Result<Tenant> {
        let mut new_tenant = tenant.clone();
        if new_tenant.id.is_empty() {
            new_tenant.id = Uuid::new_v4().to_string();
        }
        if new_tenant.slug.is_empty() {
            new_tenant.slug = Self::generate_slug(&new_tenant.name);
        }
        new_tenant.created_at = Utc::now();
        new_tenant.updated_at = Utc::now();

        let mut tenants = self.tenants.lock().unwrap();
        tenants.insert(new_tenant.id.clone(), new_tenant.clone());

        Ok(new_tenant)
    }

    async fn find_by_id(&self, id: &TenantId) -> Result<Option<Tenant>> {
        let tenants = self.tenants.lock().unwrap();
        Ok(tenants.get(id).cloned())
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>> {
        let tenants = self.tenants.lock().unwrap();
        Ok(tenants.values().find(|t| t.slug == slug).cloned())
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>> {
        let tenants = self.tenants.lock().unwrap();
        Ok(tenants.values().find(|t| t.name == name).cloned())
    }

    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Tenant>> {
        let tenants = self.tenants.lock().unwrap();
        let mut tenant_list: Vec<_> = tenants.values().cloned().collect();
        tenant_list.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        let offset = offset.unwrap_or(0) as usize;
        let limit = limit.unwrap_or(50) as usize;

        Ok(tenant_list.into_iter().skip(offset).take(limit).collect())
    }

    async fn update(&self, tenant: &Tenant) -> Result<Tenant> {
        let mut updated_tenant = tenant.clone();
        updated_tenant.updated_at = Utc::now();

        let mut tenants = self.tenants.lock().unwrap();
        tenants.insert(updated_tenant.id.clone(), updated_tenant.clone());

        Ok(updated_tenant)
    }

    async fn delete(&self, id: &TenantId) -> Result<()> {
        let mut tenants = self.tenants.lock().unwrap();
        tenants.remove(id);
        Ok(())
    }

    async fn count(&self) -> Result<u64> {
        let tenants = self.tenants.lock().unwrap();
        Ok(tenants.len() as u64)
    }
}

pub struct MockTenantMembershipRepository {
    memberships: Arc<Mutex<HashMap<String, TenantMembership>>>,
}

impl MockTenantMembershipRepository {
    pub fn new() -> Self {
        Self {
            memberships: Arc<Mutex::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl TenantMembershipRepository for MockTenantMembershipRepository {
    async fn create(&self, membership: &TenantMembership) -> Result<TenantMembership> {
        let mut new_membership = membership.clone();
        if new_membership.id.is_empty() {
            new_membership.id = Uuid::new_v4().to_string();
        }
        new_membership.created_at = Utc::now();
        new_membership.updated_at = Utc::now();

        let mut memberships = self.memberships.lock().unwrap();
        memberships.insert(new_membership.id.clone(), new_membership.clone());

        Ok(new_membership)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<TenantMembership>> {
        let memberships = self.memberships.lock().unwrap();
        Ok(memberships.get(id).cloned())
    }

    async fn find_by_tenant_and_user(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<Option<TenantMembership>> {
        let memberships = self.memberships.lock().unwrap();
        Ok(memberships.values()
            .find(|m| m.tenant_id == *tenant_id && m.user_id == *user_id)
            .cloned())
    }

    async fn list_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<TenantMembership>> {
        let memberships = self.memberships.lock().unwrap();
        let mut tenant_memberships: Vec<_> = memberships.values()
            .filter(|m| m.tenant_id == *tenant_id)
            .cloned()
            .collect();
        tenant_memberships.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(tenant_memberships)
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<TenantMembership>> {
        let memberships = self.memberships.lock().unwrap();
        let mut user_memberships: Vec<_> = memberships.values()
            .filter(|m| m.user_id == *user_id)
            .cloned()
            .collect();
        user_memberships.sort_by(|a, b| b.created_at.cmp(&a.created_at));
        Ok(user_memberships)
    }

    async fn update(&self, membership: &TenantMembership) -> Result<TenantMembership> {
        let mut updated_membership = membership.clone();
        updated_membership.updated_at = Utc::now();

        let mut memberships = self.memberships.lock().unwrap();
        memberships.insert(updated_membership.id.clone(), updated_membership.clone());

        Ok(updated_membership)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        let mut memberships = self.memberships.lock().unwrap();
        memberships.remove(id);
        Ok(())
    }
}