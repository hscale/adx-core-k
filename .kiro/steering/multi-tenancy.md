# ADX CORE Multi-Tenancy Development Guidelines

## Core Principles

ADX CORE implements comprehensive multi-tenancy with complete isolation at database, application, and workflow levels. All tenant operations leverage Temporal workflows for reliable tenant management and cross-tenant operations.

## Tenant Isolation Patterns

### Database-Level Isolation
```rust
// Tenant-aware database connection
pub struct TenantDatabase {
    connection_pool: Arc<PgPool>,
    tenant_id: String,
    isolation_level: TenantIsolationLevel,
}

#[derive(Debug, Clone)]
pub enum TenantIsolationLevel {
    Schema,     // Separate schema per tenant
    Database,   // Separate database per tenant
    Row,        // Row-level security with tenant_id column
}

impl TenantDatabase {
    pub async fn execute_query<T>(&self, query: &str, params: &[&dyn ToSql]) -> Result<Vec<T>, DatabaseError> {
        match self.isolation_level {
            TenantIsolationLevel::Schema => {
                let schema_query = format!("SET search_path = tenant_{};", self.tenant_id);
                self.connection_pool.execute(&schema_query, &[]).await?;
                self.connection_pool.query(query, params).await
            }
            TenantIsolationLevel::Database => {
                // Use tenant-specific connection pool
                let tenant_pool = self.get_tenant_pool(&self.tenant_id).await?;
                tenant_pool.query(query, params).await
            }
            TenantIsolationLevel::Row => {
                // Automatically inject tenant_id filter
                let tenant_query = self.inject_tenant_filter(query, &self.tenant_id)?;
                self.connection_pool.query(&tenant_query, params).await
            }
        }
    }
}
```

### Application-Level Isolation
```rust
// Tenant context propagation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantContext {
    pub tenant_id: String,
    pub tenant_name: String,
    pub subscription_tier: SubscriptionTier,
    pub features: Vec<String>,
    pub quotas: TenantQuotas,
    pub settings: TenantSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantQuotas {
    pub max_users: u32,
    pub max_storage_gb: u32,
    pub max_api_calls_per_hour: u32,
    pub max_workflows_per_hour: u32,
}

// Middleware for tenant context injection
pub async fn tenant_context_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, StatusCode> {
    let tenant_id = extract_tenant_id(&req)?;
    let tenant_context = get_tenant_context(&tenant_id).await?;
    
    // Validate tenant is active
    if !tenant_context.is_active() {
        return Err(StatusCode::FORBIDDEN);
    }
    
    // Inject tenant context into request
    req.extensions_mut().insert(tenant_context);
    
    Ok(next.run(req).await?)
}
```

## Temporal Workflow Tenant Isolation

### Tenant-Aware Workflows
```rust
#[workflow]
pub async fn tenant_aware_business_workflow(
    request: TenantAwareBusinessRequest,
) -> Result<TenantAwareBusinessResult, WorkflowError> {
    // Validate tenant context
    let tenant_validation = call_activity(
        TenantActivities::validate_tenant_access,
        ValidateTenantAccessRequest {
            tenant_id: request.tenant_context.tenant_id.clone(),
            user_id: request.user_context.user_id.clone(),
            required_permissions: request.required_permissions.clone(),
        },
    ).await?;
    
    if !tenant_validation.is_valid {
        return Err(WorkflowError::TenantAccessDenied(tenant_validation.reason));
    }
    
    // Check tenant quotas
    let quota_check = call_activity(
        TenantActivities::check_tenant_quotas,
        CheckTenantQuotasRequest {
            tenant_id: request.tenant_context.tenant_id.clone(),
            resource_type: ResourceType::WorkflowExecution,
            requested_amount: 1,
        },
    ).await?;
    
    if !quota_check.allowed {
        return Err(WorkflowError::QuotaExceeded(quota_check.current_usage));
    }
    
    // Execute business logic with tenant context
    let result = call_activity(
        BusinessActivities::process_tenant_data,
        ProcessTenantDataRequest {
            data: request.data,
            tenant_context: request.tenant_context.clone(),
        },
    ).await?;
    
    // Update tenant usage metrics
    call_activity(
        TenantActivities::update_tenant_usage,
        UpdateTenantUsageRequest {
            tenant_id: request.tenant_context.tenant_id,
            resource_type: ResourceType::WorkflowExecution,
            amount: 1,
        },
    ).await?;
    
    Ok(TenantAwareBusinessResult {
        result: result.data,
        tenant_id: request.tenant_context.tenant_id,
    })
}
```

### Cross-Tenant Operations
```rust
#[workflow]
pub async fn cross_tenant_data_sharing_workflow(
    request: CrossTenantSharingRequest,
) -> Result<CrossTenantSharingResult, WorkflowError> {
    // Validate source tenant permissions
    let source_validation = call_activity(
        TenantActivities::validate_cross_tenant_sharing,
        ValidateCrossTenantSharingRequest {
            source_tenant_id: request.source_tenant_id.clone(),
            target_tenant_id: request.target_tenant_id.clone(),
            data_type: request.data_type.clone(),
            sharing_level: request.sharing_level.clone(),
        },
    ).await?;
    
    if !source_validation.allowed {
        return Err(WorkflowError::CrossTenantSharingDenied(source_validation.reason));
    }
    
    // Create secure data export
    let export_data = call_activity(
        TenantActivities::export_tenant_data,
        ExportTenantDataRequest {
            tenant_id: request.source_tenant_id.clone(),
            data_selector: request.data_selector.clone(),
            anonymization_level: request.anonymization_level,
        },
    ).await?;
    
    // Import data to target tenant
    let import_result = call_activity(
        TenantActivities::import_tenant_data,
        ImportTenantDataRequest {
            tenant_id: request.target_tenant_id.clone(),
            data: export_data.data,
            import_options: request.import_options,
        },
    ).await?;
    
    // Log cross-tenant operation
    call_activity(
        AuditActivities::log_cross_tenant_operation,
        LogCrossTenantOperationRequest {
            source_tenant_id: request.source_tenant_id,
            target_tenant_id: request.target_tenant_id,
            operation_type: "data_sharing".to_string(),
            data_summary: import_result.summary,
        },
    ).await?;
    
    Ok(CrossTenantSharingResult {
        shared_data_id: import_result.data_id,
        records_shared: import_result.record_count,
    })
}
```

## Tenant Management Workflows

### Tenant Creation Workflow
```rust
#[workflow]
pub async fn create_tenant_workflow(
    request: CreateTenantRequest,
) -> Result<CreateTenantResult, WorkflowError> {
    // Step 1: Validate tenant creation request
    let validation = call_activity(
        TenantActivities::validate_tenant_creation,
        ValidateTenantCreationRequest {
            tenant_name: request.tenant_name.clone(),
            admin_email: request.admin_email.clone(),
            subscription_tier: request.subscription_tier.clone(),
        },
    ).await?;
    
    if !validation.is_valid {
        return Err(WorkflowError::ValidationFailed(validation.errors));
    }
    
    // Step 2: Create tenant database schema/database
    let database_setup = call_activity(
        TenantActivities::setup_tenant_database,
        SetupTenantDatabaseRequest {
            tenant_id: validation.tenant_id.clone(),
            isolation_level: request.isolation_level,
            initial_schema: request.initial_schema,
        },
    ).await?;
    
    // Step 3: Create tenant configuration
    let tenant_config = call_activity(
        TenantActivities::create_tenant_config,
        CreateTenantConfigRequest {
            tenant_id: validation.tenant_id.clone(),
            tenant_name: request.tenant_name,
            subscription_tier: request.subscription_tier,
            quotas: request.quotas,
            features: request.features,
        },
    ).await?;
    
    // Step 4: Create admin user
    let admin_user = call_activity(
        UserActivities::create_tenant_admin,
        CreateTenantAdminRequest {
            tenant_id: validation.tenant_id.clone(),
            email: request.admin_email,
            temporary_password: generate_secure_password(),
        },
    ).await?;
    
    // Step 5: Set up default modules
    for module_id in request.default_modules {
        call_activity(
            ModuleActivities::install_tenant_module,
            InstallTenantModuleRequest {
                tenant_id: validation.tenant_id.clone(),
                module_id,
                auto_activate: true,
            },
        ).await?;
    }
    
    // Step 6: Send welcome email
    call_activity(
        NotificationActivities::send_tenant_welcome,
        SendTenantWelcomeRequest {
            tenant_id: validation.tenant_id.clone(),
            admin_email: request.admin_email,
            admin_user_id: admin_user.user_id,
            temporary_password: admin_user.temporary_password,
        },
    ).await?;
    
    Ok(CreateTenantResult {
        tenant_id: validation.tenant_id,
        admin_user_id: admin_user.user_id,
        database_connection: database_setup.connection_string,
    })
}
```

### Tenant Migration Workflow
```rust
#[workflow]
pub async fn migrate_tenant_workflow(
    request: MigrateTenantRequest,
) -> Result<MigrateTenantResult, WorkflowError> {
    // Step 1: Validate migration request
    let validation = call_activity(
        TenantActivities::validate_tenant_migration,
        ValidateTenantMigrationRequest {
            tenant_id: request.tenant_id.clone(),
            target_tier: request.target_tier.clone(),
            migration_type: request.migration_type.clone(),
        },
    ).await?;
    
    // Step 2: Create backup
    let backup = call_activity(
        TenantActivities::backup_tenant_data,
        BackupTenantDataRequest {
            tenant_id: request.tenant_id.clone(),
            backup_type: BackupType::Full,
            encryption_key: request.encryption_key,
        },
    ).await?;
    
    // Step 3: Migrate data
    let migration_result = call_activity(
        TenantActivities::migrate_tenant_data,
        MigrateTenantDataRequest {
            tenant_id: request.tenant_id.clone(),
            source_tier: validation.current_tier,
            target_tier: request.target_tier.clone(),
            migration_options: request.migration_options,
        },
    ).await.map_err(|e| {
        // If migration fails, restore from backup
        spawn_workflow(
            restore_tenant_from_backup_workflow,
            RestoreTenantFromBackupRequest {
                tenant_id: request.tenant_id.clone(),
                backup_id: backup.backup_id,
            },
        );
        e
    })?;
    
    // Step 4: Update tenant configuration
    call_activity(
        TenantActivities::update_tenant_tier,
        UpdateTenantTierRequest {
            tenant_id: request.tenant_id.clone(),
            new_tier: request.target_tier,
            new_quotas: migration_result.new_quotas,
            new_features: migration_result.new_features,
        },
    ).await?;
    
    // Step 5: Notify tenant admin
    call_activity(
        NotificationActivities::send_migration_complete,
        SendMigrationCompleteRequest {
            tenant_id: request.tenant_id.clone(),
            migration_summary: migration_result.summary,
        },
    ).await?;
    
    Ok(MigrateTenantResult {
        tenant_id: request.tenant_id,
        migration_id: migration_result.migration_id,
        new_tier: request.target_tier,
    })
}
```

## Tenant Switching and Context Management

### User Tenant Switching Workflow
```rust
#[workflow]
pub async fn switch_user_tenant_workflow(
    request: SwitchUserTenantRequest,
) -> Result<SwitchUserTenantResult, WorkflowError> {
    // Step 1: Validate user has access to target tenant
    let access_validation = call_activity(
        TenantActivities::validate_user_tenant_access,
        ValidateUserTenantAccessRequest {
            user_id: request.user_id.clone(),
            target_tenant_id: request.target_tenant_id.clone(),
        },
    ).await?;
    
    if !access_validation.has_access {
        return Err(WorkflowError::TenantAccessDenied(access_validation.reason));
    }
    
    // Step 2: Save current session state
    let session_state = call_activity(
        SessionActivities::save_session_state,
        SaveSessionStateRequest {
            user_id: request.user_id.clone(),
            current_tenant_id: request.current_tenant_id.clone(),
            session_data: request.current_session_data,
        },
    ).await?;
    
    // Step 3: Load target tenant context
    let tenant_context = call_activity(
        TenantActivities::load_tenant_context,
        LoadTenantContextRequest {
            tenant_id: request.target_tenant_id.clone(),
            user_id: request.user_id.clone(),
        },
    ).await?;
    
    // Step 4: Create new session for target tenant
    let new_session = call_activity(
        SessionActivities::create_tenant_session,
        CreateTenantSessionRequest {
            user_id: request.user_id.clone(),
            tenant_id: request.target_tenant_id.clone(),
            tenant_context: tenant_context.clone(),
            session_duration: request.session_duration,
        },
    ).await?;
    
    // Step 5: Update user's active tenant
    call_activity(
        UserActivities::update_user_active_tenant,
        UpdateUserActiveTenantRequest {
            user_id: request.user_id.clone(),
            new_active_tenant_id: request.target_tenant_id.clone(),
        },
    ).await?;
    
    Ok(SwitchUserTenantResult {
        new_session_id: new_session.session_id,
        tenant_context,
        available_features: new_session.available_features,
    })
}
```

## Multi-Tenant Frontend Patterns

### Tenant-Aware Micro-Frontend
```typescript
// Tenant context hook
export const useTenantContext = () => {
    const context = useContext(TenantContext);
    if (!context) {
        throw new Error('useTenantContext must be used within TenantProvider');
    }
    return context;
};

// Tenant-aware component
export const TenantAwareComponent: React.FC = () => {
    const { tenantId, tenantName, features, quotas } = useTenantContext();
    const [data, setData] = useState(null);
    
    useEffect(() => {
        // Fetch tenant-specific data
        const fetchTenantData = async () => {
            const response = await fetch(`/api/tenants/${tenantId}/data`, {
                headers: {
                    'X-Tenant-ID': tenantId,
                    'Authorization': `Bearer ${getAuthToken()}`,
                },
            });
            
            if (response.ok) {
                const tenantData = await response.json();
                setData(tenantData);
            }
        };
        
        fetchTenantData();
    }, [tenantId]);
    
    // Check feature availability
    const hasFeature = (featureName: string) => {
        return features.includes(featureName);
    };
    
    // Check quota limits
    const isQuotaExceeded = (quotaType: string) => {
        const quota = quotas[quotaType];
        return quota && quota.used >= quota.limit;
    };
    
    return (
        <div className="tenant-aware-component">
            <h2>{tenantName} Dashboard</h2>
            
            {hasFeature('advanced_analytics') && (
                <AdvancedAnalyticsWidget tenantId={tenantId} />
            )}
            
            {isQuotaExceeded('storage') && (
                <QuotaExceededWarning quotaType="storage" />
            )}
            
            <TenantDataDisplay data={data} />
        </div>
    );
};
```

### Tenant Switching UI
```typescript
// Tenant switcher component
export const TenantSwitcher: React.FC = () => {
    const { currentTenant, availableTenants, switchTenant } = useTenantContext();
    const [switching, setSwitching] = useState(false);
    
    const handleTenantSwitch = async (targetTenantId: string) => {
        setSwitching(true);
        
        try {
            // Trigger tenant switch workflow
            const response = await fetch('/api/workflows/switch-tenant', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${getAuthToken()}`,
                },
                body: JSON.stringify({
                    targetTenantId,
                    currentTenantId: currentTenant.id,
                }),
            });
            
            if (response.ok) {
                const result = await response.json();
                
                if (result.operationId) {
                    // Long-running workflow - poll for completion
                    await pollTenantSwitchStatus(result.operationId);
                }
                
                // Update tenant context
                await switchTenant(targetTenantId);
                
                // Reload page to refresh all micro-frontends
                window.location.reload();
            }
        } catch (error) {
            console.error('Tenant switch failed:', error);
            // Show error notification
        } finally {
            setSwitching(false);
        }
    };
    
    return (
        <div className="tenant-switcher">
            <select
                value={currentTenant.id}
                onChange={(e) => handleTenantSwitch(e.target.value)}
                disabled={switching}
            >
                {availableTenants.map((tenant) => (
                    <option key={tenant.id} value={tenant.id}>
                        {tenant.name}
                    </option>
                ))}
            </select>
            
            {switching && <LoadingSpinner />}
        </div>
    );
};
```

## Tenant Data Isolation Testing

### Database Isolation Tests
```rust
#[tokio::test]
async fn test_tenant_data_isolation() {
    let test_env = IntegrationTestEnvironment::new().await;
    
    // Create two test tenants
    let tenant1_id = "tenant1";
    let tenant2_id = "tenant2";
    
    // Create data for tenant1
    let tenant1_data = test_env.create_tenant_data(tenant1_id, "test_data_1").await;
    
    // Create data for tenant2
    let tenant2_data = test_env.create_tenant_data(tenant2_id, "test_data_2").await;
    
    // Verify tenant1 can only see their data
    let tenant1_query_result = test_env
        .query_with_tenant_context(tenant1_id, "SELECT * FROM test_table")
        .await;
    
    assert_eq!(tenant1_query_result.len(), 1);
    assert_eq!(tenant1_query_result[0].data, "test_data_1");
    
    // Verify tenant2 can only see their data
    let tenant2_query_result = test_env
        .query_with_tenant_context(tenant2_id, "SELECT * FROM test_table")
        .await;
    
    assert_eq!(tenant2_query_result.len(), 1);
    assert_eq!(tenant2_query_result[0].data, "test_data_2");
}
```

### Workflow Isolation Tests
```rust
#[tokio::test]
async fn test_workflow_tenant_isolation() {
    let test_env = TestWorkflowEnvironment::new().await;
    
    // Mock tenant validation
    let mock_activities = MockTenantActivities::new()
        .expect_validate_tenant_access()
        .returning(|req| {
            if req.tenant_id == "valid_tenant" {
                Ok(TenantValidationResult { is_valid: true, reason: None })
            } else {
                Ok(TenantValidationResult { 
                    is_valid: false, 
                    reason: Some("Invalid tenant".to_string()) 
                })
            }
        });
    
    // Test valid tenant
    let valid_result = test_env.execute_workflow(
        tenant_aware_business_workflow,
        TenantAwareBusinessRequest {
            tenant_context: TenantContext {
                tenant_id: "valid_tenant".to_string(),
                ..Default::default()
            },
            ..Default::default()
        },
    ).await;
    
    assert!(valid_result.is_ok());
    
    // Test invalid tenant
    let invalid_result = test_env.execute_workflow(
        tenant_aware_business_workflow,
        TenantAwareBusinessRequest {
            tenant_context: TenantContext {
                tenant_id: "invalid_tenant".to_string(),
                ..Default::default()
            },
            ..Default::default()
        },
    ).await;
    
    assert!(invalid_result.is_err());
    assert!(matches!(
        invalid_result.unwrap_err(),
        WorkflowError::TenantAccessDenied(_)
    ));
}
```

## Development Guidelines

### Multi-Tenant Development Best Practices
1. **Always Validate Tenant Context**: Every operation must validate tenant access
2. **Propagate Tenant Context**: Pass tenant context through all layers
3. **Isolate Data**: Ensure complete data isolation at all levels
4. **Test Isolation**: Thoroughly test tenant data isolation
5. **Monitor Cross-Tenant Operations**: Log and monitor any cross-tenant operations
6. **Quota Enforcement**: Always check and enforce tenant quotas
7. **Feature Flags**: Use tenant-specific feature flags for functionality

### Security Considerations
1. **Tenant ID Validation**: Always validate tenant IDs from trusted sources
2. **Cross-Tenant Prevention**: Prevent accidental cross-tenant data access
3. **Audit Logging**: Log all tenant operations for compliance
4. **Encryption**: Use tenant-specific encryption keys where required
5. **Access Control**: Implement fine-grained access control per tenant
6. **Data Residency**: Respect tenant data residency requirements
7. **Backup Isolation**: Ensure tenant backups are isolated and encrypted

This multi-tenancy approach ensures complete isolation while providing flexibility for cross-tenant operations when explicitly authorized and audited.