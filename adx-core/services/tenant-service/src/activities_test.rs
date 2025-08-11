#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::collections::HashMap;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use crate::services::TenantService;
    use crate::repositories_simple::{SimpleTenantRepository, SimpleTenantMembershipRepository};
    use crate::activities::*;
    use adx_shared::types::{SubscriptionTier, TenantQuotas};

    fn create_test_activities() -> TenantActivitiesImpl {
        let tenant_repo = Arc::new(SimpleTenantRepository::new());
        let membership_repo = Arc::new(SimpleTenantMembershipRepository::new());
        let tenant_service = Arc::new(TenantService::new(tenant_repo, membership_repo));
        TenantActivitiesImpl::new(tenant_service)
    }

    #[tokio::test]
    async fn test_create_tenant_activity() {
        let activities = create_test_activities();
        
        let request = CreateTenantActivityRequest {
            tenant_name: "Test Tenant".to_string(),
            admin_email: "admin@test.com".to_string(),
            subscription_tier: SubscriptionTier::Professional,
            isolation_level: adx_shared::types::TenantIsolationLevel::Schema,
            quotas: TenantQuotas::default(),
            features: vec!["basic_auth".to_string(), "file_storage".to_string()],
            infrastructure_config: InfrastructureConfig {
                database_config: DatabaseConfig {
                    isolation_level: adx_shared::types::TenantIsolationLevel::Schema,
                    backup_enabled: true,
                    backup_retention_days: 30,
                    encryption_enabled: true,
                },
                storage_config: StorageConfig {
                    storage_type: "s3".to_string(),
                    bucket_name: Some("test-bucket".to_string()),
                    encryption_enabled: true,
                    versioning_enabled: true,
                },
                compute_config: ComputeConfig {
                    cpu_limit: 2.0,
                    memory_limit_gb: 4,
                    auto_scaling_enabled: true,
                },
                network_config: NetworkConfig {
                    custom_domain: Some("test.example.com".to_string()),
                    ssl_enabled: true,
                    cdn_enabled: true,
                },
            },
        };

        let result = activities.create_tenant_activity(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert!(!result.tenant_id.is_empty());
        assert!(result.infrastructure_status.database_ready);
        assert!(result.infrastructure_status.storage_ready);
        assert!(result.infrastructure_status.compute_ready);
        assert!(result.infrastructure_status.network_ready);
    }

    #[tokio::test]
    async fn test_setup_tenant_permissions_activity() {
        let activities = create_test_activities();
        
        let request = SetupTenantPermissionsRequest {
            tenant_id: "test-tenant-123".to_string(),
            admin_user_id: "admin-user-456".to_string(),
            role_definitions: vec![
                RoleDefinition {
                    name: "admin".to_string(),
                    description: "Administrator role".to_string(),
                    permissions: vec!["tenant:admin".to_string(), "user:manage".to_string()],
                    is_default: false,
                    hierarchy_level: 1,
                },
                RoleDefinition {
                    name: "user".to_string(),
                    description: "Standard user role".to_string(),
                    permissions: vec!["tenant:read".to_string(), "user:read".to_string()],
                    is_default: true,
                    hierarchy_level: 2,
                },
            ],
            default_permissions: vec!["tenant:read".to_string()],
        };

        let result = activities.setup_tenant_permissions_activity(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.roles_created.len(), 2);
        assert!(result.roles_created.contains(&"admin".to_string()));
        assert!(result.roles_created.contains(&"user".to_string()));
        assert!(!result.admin_role_id.is_empty());
    }

    #[tokio::test]
    async fn test_monitor_tenant_usage_activity() {
        let activities = create_test_activities();
        
        // First create a tenant to monitor
        let create_request = crate::models::CreateTenantRequest {
            name: "Test Tenant".to_string(),
            admin_email: "admin@test.com".to_string(),
            subscription_tier: Some(SubscriptionTier::Professional),
            isolation_level: None,
            features: None,
            settings: None,
        };
        let tenant = activities.tenant_service().create_tenant(create_request).await.unwrap();
        
        let request = MonitorTenantUsageRequest {
            tenant_id: tenant.id.clone(),
            monitoring_period: MonitoringPeriod::Daily,
            metrics_to_track: vec![
                UsageMetric::ApiCalls,
                UsageMetric::StorageUsage,
                UsageMetric::ActiveUsers,
            ],
        };

        let result = activities.monitor_tenant_usage_activity(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.tenant_id, tenant.id);
        assert_eq!(result.usage_data.len(), 3);
        assert!(result.usage_data.contains_key("ApiCalls"));
        assert!(result.usage_data.contains_key("StorageUsage"));
        assert!(result.usage_data.contains_key("ActiveUsers"));
    }

    #[tokio::test]
    async fn test_process_tenant_billing_activity() {
        let activities = create_test_activities();
        
        // First create a tenant for billing
        let create_request = crate::models::CreateTenantRequest {
            name: "Billing Test Tenant".to_string(),
            admin_email: "billing@test.com".to_string(),
            subscription_tier: Some(SubscriptionTier::Professional),
            isolation_level: None,
            features: None,
            settings: None,
        };
        let tenant = activities.tenant_service().create_tenant(create_request).await.unwrap();
        
        let request = ProcessTenantBillingRequest {
            tenant_id: tenant.id.clone(),
            billing_period: BillingPeriod {
                start_date: Utc::now() - chrono::Duration::days(30),
                end_date: Utc::now(),
                period_type: PeriodType::Monthly,
            },
            usage_data: {
                let mut usage = HashMap::new();
                usage.insert("api_calls".to_string(), 1500.0);
                usage.insert("storage_gb".to_string(), 5.2);
                usage
            },
            pricing_model: PricingModel {
                base_price: Decimal::new(2999, 2), // $29.99
                usage_rates: {
                    let mut rates = HashMap::new();
                    rates.insert("api_calls".to_string(), Decimal::new(1, 4)); // $0.0001 per call
                    rates.insert("storage_gb".to_string(), Decimal::new(50, 2)); // $0.50 per GB
                    rates
                },
                tier_discounts: HashMap::new(),
                currency: "USD".to_string(),
            },
        };

        let result = activities.process_tenant_billing_activity(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.tenant_id, tenant.id);
        assert!(!result.invoice_id.is_empty());
        assert!(result.total_amount > Decimal::new(0, 0));
        assert!(result.line_items.len() >= 2); // Base price + usage items
    }

    #[tokio::test]
    async fn test_cleanup_tenant_data_activity() {
        let activities = create_test_activities();
        
        let request = CleanupTenantDataRequest {
            tenant_id: "test-tenant-123".to_string(),
            cleanup_type: CleanupType::SoftDelete,
            data_retention_policy: DataRetentionPolicy {
                retain_audit_logs: true,
                retain_user_data: false,
                retain_file_metadata: true,
                retention_period_days: 90,
            },
            backup_before_cleanup: true,
        };

        let result = activities.cleanup_tenant_data_activity(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.tenant_id, "test-tenant-123");
        assert!(result.backup_info.is_some());
        assert!(result.cleanup_summary.records_deleted > 0);
    }

    #[tokio::test]
    async fn test_migrate_tenant_data_activity() {
        let activities = create_test_activities();
        
        let request = MigrateTenantDataRequest {
            tenant_id: "test-tenant-123".to_string(),
            migration_type: MigrationType::TierUpgrade,
            source_config: MigrationSourceConfig {
                current_tier: SubscriptionTier::Professional,
                current_isolation: adx_shared::types::TenantIsolationLevel::Schema,
                current_region: "us-east-1".to_string(),
                current_storage_provider: "s3".to_string(),
            },
            target_config: MigrationTargetConfig {
                target_tier: SubscriptionTier::Enterprise,
                target_isolation: adx_shared::types::TenantIsolationLevel::Database,
                target_region: "us-east-1".to_string(),
                target_storage_provider: "s3".to_string(),
            },
            migration_options: MigrationOptions {
                validate_before_migration: true,
                create_backup: true,
                rollback_on_failure: true,
                migration_batch_size: 1000,
                max_downtime_minutes: 5,
            },
        };

        let result = activities.migrate_tenant_data_activity(request).await;
        assert!(result.is_ok());
        
        let result = result.unwrap();
        assert_eq!(result.tenant_id, "test-tenant-123");
        assert!(!result.migration_id.is_empty());
        assert!(result.migration_summary.success);
        assert_eq!(result.new_configuration.tier, SubscriptionTier::Enterprise);
        assert!(result.rollback_info.is_some());
    }
}