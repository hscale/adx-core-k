use crate::types::*;
use crate::workflows::*;
use adx_shared::{DatabaseManager, TenantId};
use chrono::{DateTime, Utc};
use redis::Client as RedisClient;
use std::sync::Arc;
use uuid::Uuid;

/// Tenant Service - Following Temporal-First Architecture
///
/// TEMPORAL-FIRST PRINCIPLE:
/// - Simple operations: Direct database/cache access for <10ms response
/// - Complex operations: Temporal workflows for multi-step processes
pub struct TenantService {
    db: Arc<DatabaseManager>,
    redis: RedisClient,
}

impl TenantService {
    pub async fn new(db: Arc<DatabaseManager>) -> Result<Self, Box<dyn std::error::Error>> {
        let redis = RedisClient::open("redis://localhost:6379")?;

        Ok(Self { db, redis })
    }

    // ==================== FAST OPERATIONS (Direct API) ====================

    /// Get tenant details - Fast operation with caching
    pub async fn get_tenant(
        &self,
        tenant_id: TenantId,
    ) -> Result<Option<TenantDetails>, TenantError> {
        // Try cache first for fast response
        if let Ok(cached_tenant) = self.get_tenant_from_cache(&tenant_id).await {
            return Ok(Some(cached_tenant));
        }

        // Fallback to database
        let tenant = self.get_tenant_from_database(&tenant_id).await?;

        // Cache for future requests
        if let Some(ref tenant_data) = tenant {
            let _ = self.cache_tenant(tenant_data).await;
        }

        Ok(tenant)
    }

    /// List tenants with filtering and pagination - Fast operation
    pub async fn list_tenants(
        &self,
        query: TenantListQuery,
    ) -> Result<TenantListResponse, TenantError> {
        let limit = query.limit.unwrap_or(50).min(100); // Max 100 items
        let offset = query.offset.unwrap_or(0);

        // Build query based on filters
        let mut conditions = Vec::new();
        let mut params: Vec<String> = Vec::new();

        if let Some(ref status) = query.status {
            conditions.push("status = $1".to_string());
            params.push(status.to_string());
        }

        if let Some(ref plan) = query.plan {
            conditions.push(format!("plan = ${}", params.len() + 1));
            params.push(plan.to_string());
        }

        if let Some(ref search) = query.search {
            conditions.push(format!("(name ILIKE ${})", params.len() + 1));
            let search_pattern = format!("%{}%", search);
            params.push(search_pattern);
        }

        // Execute query (simplified - in real implementation would use actual SQL)
        let tenants = self
            .execute_tenant_list_query(conditions, params, limit, offset)
            .await?;
        let total_count = self.get_tenant_count(query.status, query.plan).await?;
        let tenants_len = tenants.len() as u32;

        Ok(TenantListResponse {
            tenants,
            total_count,
            has_more: (offset + tenants_len) < total_count,
        })
    }

    /// Update tenant information - Fast operation for simple updates
    pub async fn update_tenant(
        &self,
        tenant_id: TenantId,
        request: TenantUpdateRequest,
    ) -> Result<TenantDetails, TenantError> {
        // Validate update request
        self.validate_tenant_update(&request)?;

        // Update database
        let updated_tenant = self.update_tenant_in_database(&tenant_id, &request).await?;

        // Invalidate cache
        let _ = self.invalidate_tenant_cache(&tenant_id).await;

        // Cache updated tenant
        let _ = self.cache_tenant(&updated_tenant).await;

        Ok(updated_tenant)
    }

    /// Update tenant status - Fast operation
    pub async fn update_tenant_status(
        &self,
        tenant_id: TenantId,
        status: TenantStatus,
    ) -> Result<TenantDetails, TenantError> {
        // Update status in database
        let updated_tenant = self
            .update_tenant_status_in_database(&tenant_id, &status)
            .await?;

        // Invalidate cache
        let _ = self.invalidate_tenant_cache(&tenant_id).await;

        // Cache updated tenant
        let _ = self.cache_tenant(&updated_tenant).await;

        Ok(updated_tenant)
    }

    /// Get tenant billing information - Fast operation with caching
    pub async fn get_tenant_billing(
        &self,
        tenant_id: TenantId,
    ) -> Result<TenantBillingInfo, TenantError> {
        // Try cache first
        if let Ok(cached_billing) = self.get_billing_from_cache(&tenant_id).await {
            return Ok(cached_billing);
        }

        // Query billing data
        let billing = self.get_billing_from_database(&tenant_id).await?;

        // Cache for future requests
        let _ = self.cache_billing(&tenant_id, &billing).await;

        Ok(billing)
    }

    /// Get tenant usage statistics - Fast operation with caching
    pub async fn get_tenant_usage(
        &self,
        tenant_id: TenantId,
    ) -> Result<TenantUsageStats, TenantError> {
        // Try cache first (usage stats cached for 5 minutes)
        if let Ok(cached_usage) = self.get_usage_from_cache(&tenant_id).await {
            return Ok(cached_usage);
        }

        // Calculate current usage
        let usage = self.calculate_tenant_usage(&tenant_id).await?;

        // Cache for 5 minutes
        let _ = self.cache_usage(&tenant_id, &usage, 300).await;

        Ok(usage)
    }

    // ==================== COMPLEX OPERATIONS (Temporal Workflows) ====================

    /// Create tenant - Complex workflow: validation → provisioning → activation → notifications
    pub async fn create_tenant_workflow(
        &self,
        request: TenantCreationRequest,
    ) -> Result<TenantCreationResponse, TenantError> {
        // Validate request
        self.validate_tenant_creation(&request)?;

        // Create basic tenant record
        let tenant = self.create_basic_tenant_record(&request).await?;

        // Start provisioning workflow
        let workflow_id = format!("tenant_provisioning_{}", Uuid::new_v4());

        let provisioning_input = TenantProvisioningInput {
            tenant_id: tenant.id,
            plan: request.plan.clone(),
            resource_requirements: ResourceRequirements {
                database_schema: true,
                storage_bucket: true,
                cdn_setup: matches!(
                    request.plan,
                    TenantPlan::Professional | TenantPlan::Enterprise
                ),
                monitoring_setup: true,
                backup_setup: !matches!(request.plan, TenantPlan::Free),
            },
            features_to_enable: self.get_plan_features(&request.plan),
            notification_settings: request
                .initial_settings
                .as_ref()
                .map(|s| s.notification_preferences.clone())
                .unwrap_or_default(),
        };

        // Execute workflow (in real implementation, would use Temporal client)
        let _workflow_result = tenant_provisioning_workflow(provisioning_input)
            .await
            .map_err(|e| TenantError::WorkflowFailed(e))?;

        Ok(TenantCreationResponse {
            tenant,
            workflow_id,
            provisioning_status: "in_progress".to_string(),
            estimated_completion_minutes: 15,
        })
    }

    /// Delete tenant - Complex workflow: validation → backup → cleanup → notifications
    pub async fn delete_tenant_workflow(&self, tenant_id: TenantId) -> Result<String, TenantError> {
        // Validate deletion
        let tenant = self
            .get_tenant(tenant_id)
            .await?
            .ok_or(TenantError::NotFound)?;

        if tenant.status == TenantStatus::Deleted {
            return Err(TenantError::AlreadyDeleted);
        }

        // Start deletion workflow
        let workflow_id = format!("tenant_deletion_{}", Uuid::new_v4());

        // Execute workflow (in real implementation, would use Temporal client)
        let _workflow_result = tenant_deletion_workflow(tenant_id)
            .await
            .map_err(|e| TenantError::WorkflowFailed(e))?;

        Ok(workflow_id)
    }

    /// Provision tenant resources - Complex workflow
    pub async fn provision_tenant_workflow(
        &self,
        tenant_id: TenantId,
        input: TenantProvisioningInput,
    ) -> Result<String, TenantError> {
        let workflow_id = format!("tenant_provisioning_{}", Uuid::new_v4());

        // Execute workflow
        let _workflow_result = tenant_provisioning_workflow(input)
            .await
            .map_err(|e| TenantError::WorkflowFailed(e))?;

        Ok(workflow_id)
    }

    /// Upgrade tenant plan - Complex workflow
    pub async fn upgrade_tenant_workflow(
        &self,
        tenant_id: TenantId,
        input: TenantUpgradeInput,
    ) -> Result<String, TenantError> {
        let workflow_id = format!("tenant_upgrade_{}", Uuid::new_v4());

        // Execute workflow
        let _workflow_result = tenant_upgrade_workflow(input)
            .await
            .map_err(|e| TenantError::WorkflowFailed(e))?;

        Ok(workflow_id)
    }

    /// Monitor tenant health - Complex workflow
    pub async fn monitor_tenant_workflow(
        &self,
        tenant_id: TenantId,
        input: TenantMonitoringInput,
    ) -> Result<String, TenantError> {
        let workflow_id = format!("tenant_monitoring_{}", Uuid::new_v4());

        // Execute workflow
        let _workflow_result = tenant_monitoring_workflow(input)
            .await
            .map_err(|e| TenantError::WorkflowFailed(e))?;

        Ok(workflow_id)
    }

    // ==================== PRIVATE HELPER METHODS ====================

    async fn get_tenant_from_cache(
        &self,
        _tenant_id: &TenantId,
    ) -> Result<TenantDetails, TenantError> {
        // Implementation would use Redis to get cached tenant data
        // For now, return error to fallback to database
        Err(TenantError::CacheNotFound)
    }

    async fn cache_tenant(&self, _tenant: &TenantDetails) -> Result<(), TenantError> {
        // Implementation would cache tenant data in Redis with TTL
        Ok(())
    }

    async fn invalidate_tenant_cache(&self, _tenant_id: &TenantId) -> Result<(), TenantError> {
        // Implementation would remove tenant from Redis cache
        Ok(())
    }

    async fn get_tenant_from_database(
        &self,
        tenant_id: &TenantId,
    ) -> Result<Option<TenantDetails>, TenantError> {
        // Implementation would query PostgreSQL for tenant data
        // For now, return sample data
        Ok(Some(TenantDetails {
            id: *tenant_id,
            name: "Sample Tenant".to_string(),
            slug: "sample-tenant".to_string(),
            status: TenantStatus::Active,
            plan: TenantPlan::Professional,
            owner_email: "owner@example.com".to_string(),
            billing_email: "billing@example.com".to_string(),
            settings: TenantSettings {
                timezone: "UTC".to_string(),
                locale: "en_US".to_string(),
                features_enabled: vec!["workflows".to_string(), "ai_assistant".to_string()],
                integrations: vec![],
                security_settings: SecuritySettings {
                    enforce_2fa: false,
                    session_timeout_minutes: 60,
                    password_policy: PasswordPolicy {
                        min_length: 8,
                        require_uppercase: true,
                        require_lowercase: true,
                        require_numbers: true,
                        require_symbols: false,
                        max_age_days: Some(90),
                    },
                    ip_whitelist: vec![],
                    allowed_domains: vec![],
                },
                notification_preferences: NotificationPreferences {
                    email_notifications: true,
                    sms_notifications: false,
                    webhook_url: None,
                    notification_types: vec!["billing".to_string(), "security".to_string()],
                },
            },
            resource_limits: ResourceLimits {
                max_users: 100,
                max_storage_gb: 1000,
                max_api_calls_per_hour: 10000,
                max_workflows_per_month: 1000,
                max_files: 10000,
                max_file_size_mb: 100,
            },
            created_at: Utc::now() - chrono::Duration::days(30),
            updated_at: Utc::now() - chrono::Duration::days(1),
            last_activity: Some(Utc::now() - chrono::Duration::hours(2)),
        }))
    }

    async fn execute_tenant_list_query(
        &self,
        _conditions: Vec<String>,
        _params: Vec<String>,
        limit: u32,
        _offset: u32,
    ) -> Result<Vec<TenantSummary>, TenantError> {
        // Implementation would execute SQL query with conditions
        // For now, return sample data
        let mut tenants = Vec::new();

        for i in 0..limit.min(3) {
            tenants.push(TenantSummary {
                id: Uuid::new_v4(),
                name: format!("Tenant {}", i + 1),
                slug: format!("tenant-{}", i + 1),
                status: if i == 0 {
                    TenantStatus::Active
                } else {
                    TenantStatus::Provisioning
                },
                plan: if i == 0 {
                    TenantPlan::Enterprise
                } else {
                    TenantPlan::Professional
                },
                created_at: Utc::now() - chrono::Duration::days(30),
                last_activity: Some(Utc::now() - chrono::Duration::hours(i as i64 + 1)),
                user_count: (i + 1) * 25,
            });
        }

        Ok(tenants)
    }

    async fn get_tenant_count(
        &self,
        _status: Option<TenantStatus>,
        _plan: Option<TenantPlan>,
    ) -> Result<u32, TenantError> {
        // Implementation would count tenants in database with filters
        Ok(3)
    }

    fn validate_tenant_update(&self, _request: &TenantUpdateRequest) -> Result<(), TenantError> {
        // Validate update request fields
        Ok(())
    }

    async fn update_tenant_in_database(
        &self,
        tenant_id: &TenantId,
        request: &TenantUpdateRequest,
    ) -> Result<TenantDetails, TenantError> {
        // Implementation would update tenant in PostgreSQL
        // For now, return updated sample data
        let mut tenant = self
            .get_tenant_from_database(tenant_id)
            .await?
            .ok_or(TenantError::NotFound)?;

        if let Some(ref name) = request.name {
            tenant.name = name.clone();
        }

        if let Some(ref billing_email) = request.billing_email {
            tenant.billing_email = billing_email.clone();
        }

        if let Some(ref settings) = request.settings {
            tenant.settings = settings.clone();
        }

        tenant.updated_at = Utc::now();

        Ok(tenant)
    }

    async fn update_tenant_status_in_database(
        &self,
        tenant_id: &TenantId,
        status: &TenantStatus,
    ) -> Result<TenantDetails, TenantError> {
        // Implementation would update status in PostgreSQL
        let mut tenant = self
            .get_tenant_from_database(tenant_id)
            .await?
            .ok_or(TenantError::NotFound)?;

        tenant.status = status.clone();
        tenant.updated_at = Utc::now();

        Ok(tenant)
    }

    async fn get_billing_from_cache(
        &self,
        _tenant_id: &TenantId,
    ) -> Result<TenantBillingInfo, TenantError> {
        Err(TenantError::CacheNotFound)
    }

    async fn cache_billing(
        &self,
        _tenant_id: &TenantId,
        _billing: &TenantBillingInfo,
    ) -> Result<(), TenantError> {
        Ok(())
    }

    async fn get_billing_from_database(
        &self,
        tenant_id: &TenantId,
    ) -> Result<TenantBillingInfo, TenantError> {
        // Implementation would query billing data from database
        Ok(TenantBillingInfo {
            tenant_id: *tenant_id,
            plan: TenantPlan::Professional,
            billing_cycle: BillingCycle::Monthly,
            current_period: BillingPeriod {
                start_date: Utc::now() - chrono::Duration::days(15),
                end_date: Utc::now() + chrono::Duration::days(15),
                is_current: true,
            },
            charges: vec![BillingCharge {
                description: "Monthly subscription".to_string(),
                amount: 99.00,
                currency: "USD".to_string(),
                charge_type: ChargeType::Subscription,
                date: Utc::now() - chrono::Duration::days(15),
            }],
            payment_method: Some(PaymentMethod {
                method_type: "credit_card".to_string(),
                last_four: "4242".to_string(),
                expires_at: Some(Utc::now() + chrono::Duration::days(365)),
            }),
            billing_status: BillingStatus::Current,
        })
    }

    async fn get_usage_from_cache(
        &self,
        _tenant_id: &TenantId,
    ) -> Result<TenantUsageStats, TenantError> {
        Err(TenantError::CacheNotFound)
    }

    async fn cache_usage(
        &self,
        _tenant_id: &TenantId,
        _usage: &TenantUsageStats,
        _ttl: u32,
    ) -> Result<(), TenantError> {
        Ok(())
    }

    async fn calculate_tenant_usage(
        &self,
        tenant_id: &TenantId,
    ) -> Result<TenantUsageStats, TenantError> {
        // Implementation would calculate usage from various data sources
        Ok(TenantUsageStats {
            tenant_id: *tenant_id,
            period: BillingPeriod {
                start_date: Utc::now() - chrono::Duration::days(15),
                end_date: Utc::now() + chrono::Duration::days(15),
                is_current: true,
            },
            usage_metrics: vec![
                UsageMetric {
                    metric_name: "Active Users".to_string(),
                    current_value: 45.0,
                    limit: Some(100.0),
                    unit: "users".to_string(),
                    percentage_of_limit: Some(45.0),
                },
                UsageMetric {
                    metric_name: "Storage Usage".to_string(),
                    current_value: 250.0,
                    limit: Some(1000.0),
                    unit: "GB".to_string(),
                    percentage_of_limit: Some(25.0),
                },
                UsageMetric {
                    metric_name: "API Calls".to_string(),
                    current_value: 125000.0,
                    limit: Some(300000.0),
                    unit: "calls".to_string(),
                    percentage_of_limit: Some(41.7),
                },
            ],
            resource_consumption: ResourceConsumption {
                cpu_hours: 120.5,
                memory_gb_hours: 480.2,
                network_gb: 15.8,
                database_queries: 1250000,
            },
            api_usage: ApiUsage {
                total_requests: 125000,
                successful_requests: 123750,
                failed_requests: 1250,
                average_response_time_ms: 125.5,
                requests_by_endpoint: serde_json::json!({
                    "/api/v1/users": 45000,
                    "/api/v1/files": 35000,
                    "/api/v1/workflows": 25000,
                    "/api/v1/auth": 20000
                }),
            },
            storage_usage: StorageUsage {
                total_files: 2500,
                total_size_gb: 250.0,
                files_by_type: serde_json::json!({
                    "documents": 1500,
                    "images": 800,
                    "videos": 150,
                    "other": 50
                }),
                largest_files: vec![
                    FileInfo {
                        name: "presentation.pptx".to_string(),
                        size_mb: 45.2,
                        created_at: Utc::now() - chrono::Duration::days(5),
                        last_accessed: Some(Utc::now() - chrono::Duration::hours(2)),
                    },
                    FileInfo {
                        name: "backup.zip".to_string(),
                        size_mb: 38.7,
                        created_at: Utc::now() - chrono::Duration::days(10),
                        last_accessed: Some(Utc::now() - chrono::Duration::days(3)),
                    },
                ],
            },
        })
    }

    fn validate_tenant_creation(
        &self,
        _request: &TenantCreationRequest,
    ) -> Result<(), TenantError> {
        // Validate slug uniqueness, email formats, plan availability
        Ok(())
    }

    async fn create_basic_tenant_record(
        &self,
        request: &TenantCreationRequest,
    ) -> Result<TenantDetails, TenantError> {
        // Implementation would create basic tenant record in database
        Ok(TenantDetails {
            id: Uuid::new_v4(),
            name: request.name.clone(),
            slug: request.slug.clone(),
            status: TenantStatus::Provisioning,
            plan: request.plan.clone(),
            owner_email: request.owner_email.clone(),
            billing_email: request.billing_email.clone(),
            settings: request
                .initial_settings
                .clone()
                .unwrap_or_else(|| TenantSettings {
                    timezone: "UTC".to_string(),
                    locale: "en_US".to_string(),
                    features_enabled: self.get_plan_features(&request.plan),
                    integrations: vec![],
                    security_settings: SecuritySettings {
                        enforce_2fa: false,
                        session_timeout_minutes: 60,
                        password_policy: PasswordPolicy {
                            min_length: 8,
                            require_uppercase: true,
                            require_lowercase: true,
                            require_numbers: true,
                            require_symbols: false,
                            max_age_days: Some(90),
                        },
                        ip_whitelist: vec![],
                        allowed_domains: vec![],
                    },
                    notification_preferences: NotificationPreferences {
                        email_notifications: true,
                        sms_notifications: false,
                        webhook_url: None,
                        notification_types: vec!["billing".to_string(), "security".to_string()],
                    },
                }),
            resource_limits: self.get_plan_limits(&request.plan),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            last_activity: None,
        })
    }

    fn get_plan_features(&self, plan: &TenantPlan) -> Vec<String> {
        match plan {
            TenantPlan::Free => vec!["basic_workflows".to_string()],
            TenantPlan::Starter => vec!["basic_workflows".to_string(), "file_sharing".to_string()],
            TenantPlan::Professional => vec![
                "basic_workflows".to_string(),
                "file_sharing".to_string(),
                "advanced_workflows".to_string(),
                "analytics".to_string(),
            ],
            TenantPlan::Enterprise => vec![
                "basic_workflows".to_string(),
                "file_sharing".to_string(),
                "advanced_workflows".to_string(),
                "analytics".to_string(),
                "ai_assistant".to_string(),
                "custom_integrations".to_string(),
                "priority_support".to_string(),
            ],
            TenantPlan::Custom => vec![], // Custom features configured separately
        }
    }

    fn get_plan_limits(&self, plan: &TenantPlan) -> ResourceLimits {
        match plan {
            TenantPlan::Free => ResourceLimits {
                max_users: 5,
                max_storage_gb: 5,
                max_api_calls_per_hour: 1000,
                max_workflows_per_month: 100,
                max_files: 100,
                max_file_size_mb: 10,
            },
            TenantPlan::Starter => ResourceLimits {
                max_users: 25,
                max_storage_gb: 50,
                max_api_calls_per_hour: 5000,
                max_workflows_per_month: 500,
                max_files: 1000,
                max_file_size_mb: 50,
            },
            TenantPlan::Professional => ResourceLimits {
                max_users: 100,
                max_storage_gb: 500,
                max_api_calls_per_hour: 25000,
                max_workflows_per_month: 2500,
                max_files: 10000,
                max_file_size_mb: 100,
            },
            TenantPlan::Enterprise => ResourceLimits {
                max_users: 1000,
                max_storage_gb: 5000,
                max_api_calls_per_hour: 100000,
                max_workflows_per_month: 10000,
                max_files: 100000,
                max_file_size_mb: 500,
            },
            TenantPlan::Custom => ResourceLimits {
                max_users: u32::MAX,
                max_storage_gb: u32::MAX,
                max_api_calls_per_hour: u32::MAX,
                max_workflows_per_month: u32::MAX,
                max_files: u32::MAX,
                max_file_size_mb: u32::MAX,
            },
        }
    }
}

impl Default for NotificationPreferences {
    fn default() -> Self {
        Self {
            email_notifications: true,
            sms_notifications: false,
            webhook_url: None,
            notification_types: vec!["billing".to_string(), "security".to_string()],
        }
    }
}

// ==================== Error Types ====================

#[derive(Debug, thiserror::Error)]
pub enum TenantError {
    #[error("Tenant not found")]
    NotFound,

    #[error("Tenant already exists")]
    AlreadyExists,

    #[error("Tenant already deleted")]
    AlreadyDeleted,

    #[error("Invalid tenant data: {0}")]
    InvalidData(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Cache error")]
    CacheNotFound,

    #[error("Workflow failed: {0}")]
    WorkflowFailed(String),

    #[error("Permission denied")]
    PermissionDenied,

    #[error("Plan limit exceeded: {0}")]
    PlanLimitExceeded(String),

    #[error("Billing error: {0}")]
    BillingError(String),
}

impl From<TenantError> for axum::http::StatusCode {
    fn from(error: TenantError) -> Self {
        match error {
            TenantError::NotFound => axum::http::StatusCode::NOT_FOUND,
            TenantError::AlreadyExists => axum::http::StatusCode::CONFLICT,
            TenantError::AlreadyDeleted => axum::http::StatusCode::GONE,
            TenantError::InvalidData(_) => axum::http::StatusCode::BAD_REQUEST,
            TenantError::PermissionDenied => axum::http::StatusCode::FORBIDDEN,
            TenantError::PlanLimitExceeded(_) => axum::http::StatusCode::PAYMENT_REQUIRED,
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
