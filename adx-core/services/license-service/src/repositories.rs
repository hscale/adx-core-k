use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::{
    error::{LicenseError, Result},
    models::*,
};

#[derive(Clone)]
pub struct LicenseRepository {
    pool: PgPool,
}

impl LicenseRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create(&self, request: CreateLicenseRequest) -> Result<License> {
        let license_key = self.generate_license_key(&request.tenant_id).await?;
        let features_json = serde_json::to_value(&request.features)?;
        
        let license = sqlx::query_as!(
            License,
            r#"
            INSERT INTO licenses (
                tenant_id, license_key, subscription_tier, billing_cycle,
                base_price, currency, features, custom_quotas, auto_renew
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING 
                id, tenant_id, license_key, 
                subscription_tier as "subscription_tier: SubscriptionTier",
                status as "status: LicenseStatus",
                billing_cycle as "billing_cycle: BillingCycle",
                base_price, currency, starts_at, expires_at, auto_renew,
                features, custom_quotas, stripe_subscription_id, stripe_customer_id,
                paypal_subscription_id, created_at, updated_at, created_by
            "#,
            request.tenant_id,
            license_key,
            request.subscription_tier as SubscriptionTier,
            request.billing_cycle as BillingCycle,
            request.base_price,
            request.currency,
            features_json,
            request.custom_quotas,
            request.auto_renew
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(license)
    }

    pub async fn get_by_id(&self, id: Uuid) -> Result<Option<License>> {
        let license = sqlx::query_as!(
            License,
            r#"
            SELECT 
                id, tenant_id, license_key,
                subscription_tier as "subscription_tier: SubscriptionTier",
                status as "status: LicenseStatus",
                billing_cycle as "billing_cycle: BillingCycle",
                base_price, currency, starts_at, expires_at, auto_renew,
                features, custom_quotas, stripe_subscription_id, stripe_customer_id,
                paypal_subscription_id, created_at, updated_at, created_by
            FROM licenses 
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(license)
    }

    pub async fn get_by_tenant_id(&self, tenant_id: Uuid) -> Result<Option<License>> {
        let license = sqlx::query_as!(
            License,
            r#"
            SELECT 
                id, tenant_id, license_key,
                subscription_tier as "subscription_tier: SubscriptionTier",
                status as "status: LicenseStatus",
                billing_cycle as "billing_cycle: BillingCycle",
                base_price, currency, starts_at, expires_at, auto_renew,
                features, custom_quotas, stripe_subscription_id, stripe_customer_id,
                paypal_subscription_id, created_at, updated_at, created_by
            FROM licenses 
            WHERE tenant_id = $1 AND status = 'active'
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            tenant_id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(license)
    }

    pub async fn get_by_license_key(&self, license_key: &str) -> Result<Option<License>> {
        let license = sqlx::query_as!(
            License,
            r#"
            SELECT 
                id, tenant_id, license_key,
                subscription_tier as "subscription_tier: SubscriptionTier",
                status as "status: LicenseStatus",
                billing_cycle as "billing_cycle: BillingCycle",
                base_price, currency, starts_at, expires_at, auto_renew,
                features, custom_quotas, stripe_subscription_id, stripe_customer_id,
                paypal_subscription_id, created_at, updated_at, created_by
            FROM licenses 
            WHERE license_key = $1
            "#,
            license_key
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(license)
    }

    pub async fn update(&self, id: Uuid, request: UpdateLicenseRequest) -> Result<License> {
        let features_json = request.features.map(|f| serde_json::to_value(f)).transpose()?;
        
        let license = sqlx::query_as!(
            License,
            r#"
            UPDATE licenses SET
                subscription_tier = COALESCE($2, subscription_tier),
                status = COALESCE($3, status),
                base_price = COALESCE($4, base_price),
                expires_at = COALESCE($5, expires_at),
                auto_renew = COALESCE($6, auto_renew),
                features = COALESCE($7, features),
                custom_quotas = COALESCE($8, custom_quotas),
                updated_at = NOW()
            WHERE id = $1
            RETURNING 
                id, tenant_id, license_key,
                subscription_tier as "subscription_tier: SubscriptionTier",
                status as "status: LicenseStatus",
                billing_cycle as "billing_cycle: BillingCycle",
                base_price, currency, starts_at, expires_at, auto_renew,
                features, custom_quotas, stripe_subscription_id, stripe_customer_id,
                paypal_subscription_id, created_at, updated_at, created_by
            "#,
            id,
            request.subscription_tier as Option<SubscriptionTier>,
            request.status as Option<LicenseStatus>,
            request.base_price,
            request.expires_at,
            request.auto_renew,
            features_json,
            request.custom_quotas
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(license)
    }

    pub async fn get_expiring_licenses(&self, days_ahead: i32) -> Result<Vec<License>> {
        let licenses = sqlx::query_as!(
            License,
            r#"
            SELECT 
                id, tenant_id, license_key,
                subscription_tier as "subscription_tier: SubscriptionTier",
                status as "status: LicenseStatus",
                billing_cycle as "billing_cycle: BillingCycle",
                base_price, currency, starts_at, expires_at, auto_renew,
                features, custom_quotas, stripe_subscription_id, stripe_customer_id,
                paypal_subscription_id, created_at, updated_at, created_by
            FROM licenses 
            WHERE status = 'active' 
            AND expires_at IS NOT NULL 
            AND expires_at <= NOW() + INTERVAL '%d days'
            ORDER BY expires_at ASC
            "#,
            days_ahead
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(licenses)
    }

    async fn generate_license_key(&self, tenant_id: &Uuid) -> Result<String> {
        // Generate a unique license key
        let key = format!("ADX-{}-{}", 
            tenant_id.to_string().replace("-", "").to_uppercase()[..8].to_string(),
            uuid::Uuid::new_v4().to_string().replace("-", "").to_uppercase()[..8].to_string()
        );
        Ok(key)
    }
}

#[derive(Clone)]
pub struct QuotaRepository {
    pool: PgPool,
}

impl QuotaRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn get_quota_definitions(&self) -> Result<Vec<QuotaDefinition>> {
        let definitions = sqlx::query_as!(
            QuotaDefinition,
            "SELECT * FROM quota_definitions ORDER BY category, name"
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(definitions)
    }

    pub async fn get_quota_definition_by_name(&self, name: &str) -> Result<Option<QuotaDefinition>> {
        let definition = sqlx::query_as!(
            QuotaDefinition,
            "SELECT * FROM quota_definitions WHERE name = $1",
            name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(definition)
    }

    pub async fn get_tenant_quotas(&self, tenant_id: Uuid) -> Result<Vec<TenantQuota>> {
        let quotas = sqlx::query_as!(
            TenantQuota,
            "SELECT * FROM tenant_quotas WHERE tenant_id = $1",
            tenant_id
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(quotas)
    }

    pub async fn get_tenant_quota(&self, tenant_id: Uuid, quota_name: &str) -> Result<Option<TenantQuota>> {
        let quota = sqlx::query_as!(
            TenantQuota,
            r#"
            SELECT tq.* FROM tenant_quotas tq
            JOIN quota_definitions qd ON tq.quota_definition_id = qd.id
            WHERE tq.tenant_id = $1 AND qd.name = $2
            "#,
            tenant_id,
            quota_name
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(quota)
    }

    pub async fn initialize_tenant_quotas(&self, tenant_id: Uuid, subscription_tier: SubscriptionTier) -> Result<()> {
        let definitions = self.get_quota_definitions().await?;
        
        for definition in definitions {
            let quota_limit = match subscription_tier {
                SubscriptionTier::Free => definition.free_limit,
                SubscriptionTier::Professional => definition.professional_limit,
                SubscriptionTier::Enterprise => definition.enterprise_limit,
                SubscriptionTier::Custom => definition.enterprise_limit, // Default to enterprise for custom
            };

            sqlx::query!(
                r#"
                INSERT INTO tenant_quotas (tenant_id, quota_definition_id, quota_limit)
                VALUES ($1, $2, $3)
                ON CONFLICT (tenant_id, quota_definition_id) DO NOTHING
                "#,
                tenant_id,
                definition.id,
                quota_limit
            )
            .execute(&self.pool)
            .await?;
        }

        Ok(())
    }

    pub async fn update_quota_usage(&self, tenant_id: Uuid, quota_name: &str, amount: i64) -> Result<TenantQuota> {
        let quota = sqlx::query_as!(
            TenantQuota,
            r#"
            UPDATE tenant_quotas SET
                current_usage = current_usage + $3,
                updated_at = NOW()
            FROM quota_definitions
            WHERE tenant_quotas.quota_definition_id = quota_definitions.id
            AND tenant_quotas.tenant_id = $1
            AND quota_definitions.name = $2
            RETURNING tenant_quotas.*
            "#,
            tenant_id,
            quota_name,
            amount
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(quota)
    }

    pub async fn reset_quota_usage(&self, tenant_id: Uuid, quota_name: &str) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE tenant_quotas SET
                current_usage = 0,
                last_reset_at = NOW(),
                updated_at = NOW()
            FROM quota_definitions
            WHERE tenant_quotas.quota_definition_id = quota_definitions.id
            AND tenant_quotas.tenant_id = $1
            AND quota_definitions.name = $2
            "#,
            tenant_id,
            quota_name
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn log_usage(&self, request: QuotaUsageRequest) -> Result<UsageLog> {
        let definition = self.get_quota_definition_by_name(&request.quota_name).await?
            .ok_or_else(|| LicenseError::QuotaNotFound { quota_name: request.quota_name.clone() })?;

        let usage_log = sqlx::query_as!(
            UsageLog,
            r#"
            INSERT INTO usage_logs (
                tenant_id, quota_definition_id, amount, operation_type,
                resource_id, user_id, metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING *
            "#,
            request.tenant_id,
            definition.id,
            request.amount,
            request.operation_type,
            request.resource_id,
            request.user_id,
            request.metadata
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(usage_log)
    }
}

#[derive(Clone)]
pub struct BillingRepository {
    pool: PgPool,
}

impl BillingRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_billing_record(&self, record: BillingHistory) -> Result<BillingHistory> {
        let billing_record = sqlx::query_as!(
            BillingHistory,
            r#"
            INSERT INTO billing_history (
                tenant_id, license_id, invoice_number, amount, currency, tax_amount,
                billing_period_start, billing_period_end, payment_status,
                payment_method, payment_reference, usage_details
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
            RETURNING 
                id, tenant_id, license_id, invoice_number, amount, currency, tax_amount,
                billing_period_start, billing_period_end,
                payment_status as "payment_status: PaymentStatus",
                payment_method, payment_reference, paid_at, usage_details,
                created_at, updated_at
            "#,
            record.tenant_id,
            record.license_id,
            record.invoice_number,
            record.amount,
            record.currency,
            record.tax_amount,
            record.billing_period_start,
            record.billing_period_end,
            record.payment_status as PaymentStatus,
            record.payment_method,
            record.payment_reference,
            record.usage_details
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(billing_record)
    }

    pub async fn update_payment_status(&self, id: Uuid, status: PaymentStatus, payment_reference: Option<String>) -> Result<()> {
        let paid_at = if matches!(status, PaymentStatus::Completed) {
            Some(Utc::now())
        } else {
            None
        };

        sqlx::query!(
            r#"
            UPDATE billing_history SET
                payment_status = $2,
                payment_reference = COALESCE($3, payment_reference),
                paid_at = $4,
                updated_at = NOW()
            WHERE id = $1
            "#,
            id,
            status as PaymentStatus,
            payment_reference,
            paid_at
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_billing_history(&self, tenant_id: Uuid, limit: i64, offset: i64) -> Result<Vec<BillingHistory>> {
        let records = sqlx::query_as!(
            BillingHistory,
            r#"
            SELECT 
                id, tenant_id, license_id, invoice_number, amount, currency, tax_amount,
                billing_period_start, billing_period_end,
                payment_status as "payment_status: PaymentStatus",
                payment_method, payment_reference, paid_at, usage_details,
                created_at, updated_at
            FROM billing_history 
            WHERE tenant_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            tenant_id,
            limit,
            offset
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(records)
    }
}

#[derive(Clone)]
pub struct ComplianceRepository {
    pool: PgPool,
}

impl ComplianceRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn log_compliance_event(&self, log: ComplianceLog) -> Result<ComplianceLog> {
        let compliance_log = sqlx::query_as!(
            ComplianceLog,
            r#"
            INSERT INTO compliance_logs (
                tenant_id, event_type, event_category, severity,
                description, details, user_id, resource_id, ip_address
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING *
            "#,
            log.tenant_id,
            log.event_type,
            log.event_category,
            log.severity,
            log.description,
            log.details,
            log.user_id,
            log.resource_id,
            log.ip_address
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(compliance_log)
    }

    pub async fn get_compliance_logs(&self, tenant_id: Uuid, start_date: DateTime<Utc>, end_date: DateTime<Utc>) -> Result<Vec<ComplianceLog>> {
        let logs = sqlx::query_as!(
            ComplianceLog,
            r#"
            SELECT * FROM compliance_logs
            WHERE tenant_id = $1
            AND created_at >= $2
            AND created_at <= $3
            ORDER BY created_at DESC
            "#,
            tenant_id,
            start_date,
            end_date
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }

    pub async fn resolve_compliance_issue(&self, id: Uuid, resolved_by: Uuid, resolution_notes: String) -> Result<()> {
        sqlx::query!(
            r#"
            UPDATE compliance_logs SET
                resolved = true,
                resolved_at = NOW(),
                resolved_by = $2,
                resolution_notes = $3
            WHERE id = $1
            "#,
            id,
            resolved_by,
            resolution_notes
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}