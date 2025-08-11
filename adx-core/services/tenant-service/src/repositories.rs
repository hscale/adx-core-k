use async_trait::async_trait;
use sqlx::PgPool;
use chrono::Utc;
use uuid::Uuid;
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

pub struct PostgresTenantRepository {
    pool: PgPool,
}

impl PostgresTenantRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
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
impl TenantRepository for PostgresTenantRepository {
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

        let settings_json = serde_json::to_value(&new_tenant.settings)?;
        let quotas_json = serde_json::to_value(&new_tenant.quotas)?;

        sqlx::query(
            r#"
            INSERT INTO tenants (
                id, name, slug, admin_email, subscription_tier, isolation_level,
                quotas, features, settings, status, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            "#
        )
        .bind(&new_tenant.id)
        .bind(&new_tenant.name)
        .bind(&new_tenant.slug)
        .bind(&new_tenant.admin_email)
        .bind(serde_json::to_string(&new_tenant.subscription_tier)?)
        .bind(serde_json::to_string(&new_tenant.isolation_level)?)
        .bind(quotas_json)
        .bind(&new_tenant.features)
        .bind(settings_json)
        .bind(serde_json::to_string(&new_tenant.status)?)
        .bind(new_tenant.created_at)
        .bind(new_tenant.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(new_tenant)
    }

    async fn find_by_id(&self, id: &TenantId) -> Result<Option<Tenant>> {
        let row = sqlx::query!(
            "SELECT * FROM tenants WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let tenant = Tenant {
                id: row.id,
                name: row.name,
                slug: row.slug,
                admin_email: row.admin_email,
                subscription_tier: serde_json::from_str(&row.subscription_tier)?,
                isolation_level: serde_json::from_str(&row.isolation_level)?,
                quotas: serde_json::from_value(row.quotas)?,
                features: row.features,
                settings: serde_json::from_value(row.settings)?,
                status: serde_json::from_str(&row.status)?,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Tenant>> {
        let row = sqlx::query!(
            "SELECT * FROM tenants WHERE slug = $1",
            slug
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let tenant = Tenant {
                id: row.id,
                name: row.name,
                slug: row.slug,
                admin_email: row.admin_email,
                subscription_tier: serde_json::from_str(&row.subscription_tier)?,
                isolation_level: serde_json::from_str(&row.isolation_level)?,
                quotas: serde_json::from_value(row.quotas)?,
                features: row.features,
                settings: serde_json::from_value(row.settings)?,
                status: serde_json::from_str(&row.status)?,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }

    async fn find_by_name(&self, name: &str) -> Result<Option<Tenant>> {
        let row = sqlx::query!(
            "SELECT * FROM tenants WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let tenant = Tenant {
                id: row.id,
                name: row.name,
                slug: row.slug,
                admin_email: row.admin_email,
                subscription_tier: serde_json::from_str(&row.subscription_tier)?,
                isolation_level: serde_json::from_str(&row.isolation_level)?,
                quotas: serde_json::from_value(row.quotas)?,
                features: row.features,
                settings: serde_json::from_value(row.settings)?,
                status: serde_json::from_str(&row.status)?,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            Ok(Some(tenant))
        } else {
            Ok(None)
        }
    }

    async fn list(&self, limit: Option<u32>, offset: Option<u32>) -> Result<Vec<Tenant>> {
        let limit = limit.unwrap_or(50) as i64;
        let offset = offset.unwrap_or(0) as i64;

        let rows = sqlx::query!(
            "SELECT * FROM tenants ORDER BY created_at DESC LIMIT $1 OFFSET $2",
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        let mut tenants = Vec::new();
        for row in rows {
            let tenant = Tenant {
                id: row.id,
                name: row.name,
                slug: row.slug,
                admin_email: row.admin_email,
                subscription_tier: serde_json::from_str(&row.subscription_tier)?,
                isolation_level: serde_json::from_str(&row.isolation_level)?,
                quotas: serde_json::from_value(row.quotas)?,
                features: row.features,
                settings: serde_json::from_value(row.settings)?,
                status: serde_json::from_str(&row.status)?,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            tenants.push(tenant);
        }

        Ok(tenants)
    }

    async fn update(&self, tenant: &Tenant) -> Result<Tenant> {
        let mut updated_tenant = tenant.clone();
        updated_tenant.updated_at = Utc::now();

        let settings_json = serde_json::to_value(&updated_tenant.settings)?;
        let quotas_json = serde_json::to_value(&updated_tenant.quotas)?;

        sqlx::query!(
            r#"
            UPDATE tenants SET
                name = $2,
                slug = $3,
                admin_email = $4,
                subscription_tier = $5,
                isolation_level = $6,
                quotas = $7,
                features = $8,
                settings = $9,
                status = $10,
                updated_at = $11
            WHERE id = $1
            "#,
            updated_tenant.id,
            updated_tenant.name,
            updated_tenant.slug,
            updated_tenant.admin_email,
            serde_json::to_string(&updated_tenant.subscription_tier)?,
            serde_json::to_string(&updated_tenant.isolation_level)?,
            quotas_json,
            &updated_tenant.features,
            settings_json,
            serde_json::to_string(&updated_tenant.status)?,
            updated_tenant.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(updated_tenant)
    }

    async fn delete(&self, id: &TenantId) -> Result<()> {
        sqlx::query!(
            "DELETE FROM tenants WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn count(&self) -> Result<u64> {
        let row = sqlx::query!(
            "SELECT COUNT(*) as count FROM tenants"
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(row.count.unwrap_or(0) as u64)
    }
}

pub struct PostgresTenantMembershipRepository {
    pool: PgPool,
}

impl PostgresTenantMembershipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantMembershipRepository for PostgresTenantMembershipRepository {
    async fn create(&self, membership: &TenantMembership) -> Result<TenantMembership> {
        let mut new_membership = membership.clone();
        if new_membership.id.is_empty() {
            new_membership.id = Uuid::new_v4().to_string();
        }
        new_membership.created_at = Utc::now();
        new_membership.updated_at = Utc::now();

        sqlx::query!(
            r#"
            INSERT INTO tenant_memberships (
                id, tenant_id, user_id, role, permissions, status,
                invited_by, invited_at, joined_at, created_at, updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
            new_membership.id,
            new_membership.tenant_id,
            new_membership.user_id,
            serde_json::to_string(&new_membership.role)?,
            &new_membership.permissions,
            serde_json::to_string(&new_membership.status)?,
            new_membership.invited_by,
            new_membership.invited_at,
            new_membership.joined_at,
            new_membership.created_at,
            new_membership.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(new_membership)
    }

    async fn find_by_id(&self, id: &str) -> Result<Option<TenantMembership>> {
        let row = sqlx::query!(
            "SELECT * FROM tenant_memberships WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let membership = TenantMembership {
                id: row.id,
                tenant_id: row.tenant_id,
                user_id: row.user_id,
                role: serde_json::from_str(&row.role)?,
                permissions: row.permissions,
                status: serde_json::from_str(&row.status)?,
                invited_by: row.invited_by,
                invited_at: row.invited_at,
                joined_at: row.joined_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            Ok(Some(membership))
        } else {
            Ok(None)
        }
    }

    async fn find_by_tenant_and_user(&self, tenant_id: &TenantId, user_id: &UserId) -> Result<Option<TenantMembership>> {
        let row = sqlx::query!(
            "SELECT * FROM tenant_memberships WHERE tenant_id = $1 AND user_id = $2",
            tenant_id,
            user_id
        )
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let membership = TenantMembership {
                id: row.id,
                tenant_id: row.tenant_id,
                user_id: row.user_id,
                role: serde_json::from_str(&row.role)?,
                permissions: row.permissions,
                status: serde_json::from_str(&row.status)?,
                invited_by: row.invited_by,
                invited_at: row.invited_at,
                joined_at: row.joined_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            Ok(Some(membership))
        } else {
            Ok(None)
        }
    }

    async fn list_by_tenant(&self, tenant_id: &TenantId) -> Result<Vec<TenantMembership>> {
        let rows = sqlx::query!(
            "SELECT * FROM tenant_memberships WHERE tenant_id = $1 ORDER BY created_at DESC",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut memberships = Vec::new();
        for row in rows {
            let membership = TenantMembership {
                id: row.id,
                tenant_id: row.tenant_id,
                user_id: row.user_id,
                role: serde_json::from_str(&row.role)?,
                permissions: row.permissions,
                status: serde_json::from_str(&row.status)?,
                invited_by: row.invited_by,
                invited_at: row.invited_at,
                joined_at: row.joined_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            memberships.push(membership);
        }

        Ok(memberships)
    }

    async fn list_by_user(&self, user_id: &UserId) -> Result<Vec<TenantMembership>> {
        let rows = sqlx::query!(
            "SELECT * FROM tenant_memberships WHERE user_id = $1 ORDER BY created_at DESC",
            user_id
        )
        .fetch_all(&self.pool)
        .await?;

        let mut memberships = Vec::new();
        for row in rows {
            let membership = TenantMembership {
                id: row.id,
                tenant_id: row.tenant_id,
                user_id: row.user_id,
                role: serde_json::from_str(&row.role)?,
                permissions: row.permissions,
                status: serde_json::from_str(&row.status)?,
                invited_by: row.invited_by,
                invited_at: row.invited_at,
                joined_at: row.joined_at,
                created_at: row.created_at,
                updated_at: row.updated_at,
            };
            memberships.push(membership);
        }

        Ok(memberships)
    }

    async fn update(&self, membership: &TenantMembership) -> Result<TenantMembership> {
        let mut updated_membership = membership.clone();
        updated_membership.updated_at = Utc::now();

        sqlx::query!(
            r#"
            UPDATE tenant_memberships SET
                role = $2,
                permissions = $3,
                status = $4,
                joined_at = $5,
                updated_at = $6
            WHERE id = $1
            "#,
            updated_membership.id,
            serde_json::to_string(&updated_membership.role)?,
            &updated_membership.permissions,
            serde_json::to_string(&updated_membership.status)?,
            updated_membership.joined_at,
            updated_membership.updated_at
        )
        .execute(&self.pool)
        .await?;

        Ok(updated_membership)
    }

    async fn delete(&self, id: &str) -> Result<()> {
        sqlx::query!(
            "DELETE FROM tenant_memberships WHERE id = $1",
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}