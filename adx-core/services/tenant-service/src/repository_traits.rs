use async_trait::async_trait;
use anyhow::Result;

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