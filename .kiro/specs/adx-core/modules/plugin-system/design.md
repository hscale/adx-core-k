# Plugin System - Temporal-First Design

## Overview

The Plugin System uses Temporal workflows for all plugin lifecycle operations, making complex plugin management simple and reliable while maintaining WordPress-style familiarity.

```
┌─────────────────────────────────────────────────────────────┐
│              Temporal-First Plugin System                  │
├─────────────────┬─────────────────┬─────────────────────────┤
│  Plugin         │   Marketplace   │    Extension            │
│  Workflows      │   Workflows     │    Points               │
│                 │                 │                         │
│ • Installation  │ • Discovery     │ • UI Components        │
│ • Updates       │ • Payment       │ • API Endpoints        │
│ • Activation    │ • Reviews       │ • Workflow Steps       │
│ • Removal       │ • Analytics     │ • Database Schema      │
└─────────────────┴─────────────────┴─────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        │                       │                       │
┌───────────────┐    ┌───────────────┐    ┌───────────────┐
│   Temporal    │    │   PostgreSQL  │    │   Plugin      │
│   Workflows   │    │   (Metadata)  │    │   Sandbox     │
└───────────────┘    └───────────────┘    └───────────────┘
```

## Core Temporal Workflows

### 1. Plugin Installation Workflow
```rust
#[workflow]
pub async fn plugin_installation_workflow(
    installation_request: PluginInstallationRequest,
) -> WorkflowResult<InstalledPlugin> {
    // Step 1: Validate installation request
    validate_installation_request_activity(installation_request.clone()).await?;
    
    // Step 2: Check if plugin is premium and process payment
    if installation_request.is_premium {
        let payment_result = process_plugin_payment_activity(
            installation_request.tenant_id,
            installation_request.plugin_id.clone(),
        ).await?;
        
        if !payment_result.successful {
            return Err(WorkflowError::PaymentFailed);
        }
    }
    
    // Step 3: Download and validate plugin
    let plugin_package = download_plugin_activity(
        installation_request.plugin_id.clone(),
    ).await?;
    
    // Step 4: Security scanning and validation
    let (security_scan, dependency_check) = temporal_sdk::join!(
        security_scan_plugin_activity(plugin_package.clone()),
        check_plugin_dependencies_activity(
            installation_request.tenant_id,
            plugin_package.metadata.dependencies.clone()
        )
    );
    
    if !security_scan?.is_safe {
        return Err(WorkflowError::SecurityThreatDetected);
    }
    
    // Step 5: Install dependencies first
    for dependency in dependency_check?.missing_dependencies {
        temporal_sdk::start_child_workflow(
            plugin_installation_workflow,
            PluginInstallationRequest {
                tenant_id: installation_request.tenant_id,
                plugin_id: dependency.plugin_id,
                is_premium: dependency.is_premium,
                auto_activate: true,
            }
        ).await?;
    }
    
    // Step 6: Install plugin files and create sandbox
    let (plugin_files, sandbox) = temporal_sdk::join!(
        install_plugin_files_activity(
            installation_request.tenant_id,
            plugin_package.clone()
        ),
        create_plugin_sandbox_activity(
            installation_request.tenant_id,
            installation_request.plugin_id.clone()
        )
    );
    
    // Step 7: Run plugin database migrations
    run_plugin_migrations_activity(
        installation_request.tenant_id,
        plugin_package.migrations,
    ).await?;
    
    // Step 8: Register plugin
    let installed_plugin = register_plugin_activity(
        installation_request.tenant_id,
        plugin_package.metadata,
        plugin_files?,
        sandbox?,
    ).await?;
    
    // Step 9: Auto-activate if requested
    if installation_request.auto_activate {
        temporal_sdk::start_child_workflow(
            plugin_activation_workflow,
            PluginActivationRequest {
                tenant_id: installation_request.tenant_id,
                plugin_id: installation_request.plugin_id,
            }
        ).await?;
    }
    
    // Step 10: Record installation analytics
    record_plugin_installation_activity(
        installation_request.plugin_id,
        installation_request.tenant_id,
    ).await?;
    
    Ok(installed_plugin)
}
```

### 2. Plugin Update Workflow
```rust
#[workflow]
pub async fn plugin_update_workflow(
    update_request: PluginUpdateRequest,
) -> WorkflowResult<UpdateResult> {
    // Step 1: Check for available updates
    let available_update = check_plugin_update_activity(
        update_request.plugin_id.clone(),
        update_request.current_version.clone(),
    ).await?;
    
    if available_update.is_none() {
        return Ok(UpdateResult::NoUpdateAvailable);
    }
    
    let update_info = available_update.unwrap();
    
    // Step 2: Create backup of current plugin
    let backup_result = backup_plugin_activity(
        update_request.tenant_id,
        update_request.plugin_id.clone(),
    ).await?;
    
    // Step 3: Download and validate new version
    let new_plugin_package = download_plugin_activity(
        update_info.download_url,
    ).await?;
    
    // Step 4: Security scan new version
    let security_scan = security_scan_plugin_activity(
        new_plugin_package.clone(),
    ).await?;
    
    if !security_scan.is_safe {
        return Err(WorkflowError::SecurityThreatDetected);
    }
    
    // Step 5: Test compatibility
    let compatibility_test = test_plugin_compatibility_activity(
        update_request.tenant_id,
        new_plugin_package.clone(),
    ).await?;
    
    if !compatibility_test.is_compatible {
        return Ok(UpdateResult::IncompatibleVersion(compatibility_test.issues));
    }
    
    // Step 6: Deactivate current plugin
    deactivate_plugin_activity(
        update_request.tenant_id,
        update_request.plugin_id.clone(),
    ).await?;
    
    // Step 7: Install new version
    let install_result = install_plugin_update_activity(
        update_request.tenant_id,
        update_request.plugin_id.clone(),
        new_plugin_package,
    ).await?;
    
    // Step 8: Run update migrations
    run_plugin_update_migrations_activity(
        update_request.tenant_id,
        update_request.plugin_id.clone(),
        update_info.migrations,
    ).await?;
    
    // Step 9: Reactivate plugin
    let activation_result = activate_plugin_activity(
        update_request.tenant_id,
        update_request.plugin_id.clone(),
    ).await;
    
    // Step 10: Verify update success
    if activation_result.is_ok() {
        // Clean up backup
        cleanup_plugin_backup_activity(backup_result.backup_id).await?;
        
        Ok(UpdateResult::Success {
            old_version: update_request.current_version,
            new_version: update_info.version,
        })
    } else {
        // Rollback to backup
        temporal_sdk::start_child_workflow(
            plugin_rollback_workflow,
            PluginRollbackRequest {
                tenant_id: update_request.tenant_id,
                plugin_id: update_request.plugin_id,
                backup_id: backup_result.backup_id,
            }
        ).await?;
        
        Err(WorkflowError::UpdateFailed)
    }
}
```

### 3. Plugin Monitoring Workflow
```rust
#[workflow]
pub async fn plugin_monitoring_workflow(
    monitoring_data: PluginMonitoringData,
) -> WorkflowResult<()> {
    loop {
        // Step 1: Check plugin health
        let health_status = check_plugin_health_activity(
            monitoring_data.tenant_id,
            monitoring_data.plugin_id.clone(),
        ).await?;
        
        // Step 2: Check resource usage
        let resource_usage = check_plugin_resource_usage_activity(
            monitoring_data.tenant_id,
            monitoring_data.plugin_id.clone(),
        ).await?;
        
        // Step 3: Check for errors
        let error_rate = check_plugin_error_rate_activity(
            monitoring_data.tenant_id,
            monitoring_data.plugin_id.clone(),
        ).await?;
        
        // Step 4: Take action if needed
        if !health_status.is_healthy || resource_usage.is_excessive || error_rate.is_high {
            temporal_sdk::start_child_workflow(
                plugin_remediation_workflow,
                PluginRemediationData {
                    tenant_id: monitoring_data.tenant_id,
                    plugin_id: monitoring_data.plugin_id.clone(),
                    health_status,
                    resource_usage,
                    error_rate,
                }
            ).await?;
        }
        
        // Step 5: Wait before next check
        temporal_sdk::sleep(Duration::from_minutes(5)).await;
    }
}
```

## Simple Plugin Activities

### Installation Activities
```rust
#[activity]
pub async fn download_plugin_activity(
    plugin_id: String,
) -> Result<PluginPackage, ActivityError> {
    let marketplace = get_marketplace_client().await?;
    
    let download_url = marketplace.get_download_url(&plugin_id).await?;
    let package_data = download_file(&download_url).await?;
    
    // Verify package integrity
    let package = PluginPackage::from_bytes(package_data)?;
    verify_package_signature(&package)?;
    
    Ok(package)
}

#[activity]
pub async fn security_scan_plugin_activity(
    package: PluginPackage,
) -> Result<SecurityScanResult, ActivityError> {
    let scanner = get_security_scanner().await?;
    
    // Scan for malicious code
    let malware_scan = scanner.scan_for_malware(&package.code).await?;
    
    // Check for suspicious patterns
    let pattern_scan = scanner.scan_for_suspicious_patterns(&package.code).await?;
    
    // Validate permissions
    let permission_scan = scanner.validate_permissions(&package.manifest.permissions).await?;
    
    Ok(SecurityScanResult {
        is_safe: malware_scan.is_clean && pattern_scan.is_clean && permission_scan.is_valid,
        threats_found: malware_scan.threats,
        suspicious_patterns: pattern_scan.patterns,
        permission_issues: permission_scan.issues,
    })
}

#[activity]
pub async fn create_plugin_sandbox_activity(
    tenant_id: TenantId,
    plugin_id: String,
) -> Result<PluginSandbox, ActivityError> {
    let sandbox_manager = get_sandbox_manager().await?;
    
    let sandbox_config = SandboxConfig {
        tenant_id,
        plugin_id: plugin_id.clone(),
        memory_limit: 128 * 1024 * 1024, // 128MB
        cpu_limit: 0.5, // 50% of one CPU
        network_access: NetworkAccess::Restricted,
        file_system_access: FileSystemAccess::PluginDirectory,
    };
    
    let sandbox = sandbox_manager.create_sandbox(sandbox_config).await?;
    
    Ok(sandbox)
}
```

## Database Schema (Simplified)

```sql
CREATE TABLE installed_plugins (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    plugin_id VARCHAR(255) NOT NULL,
    plugin_name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    status plugin_status NOT NULL DEFAULT 'inactive',
    
    -- Installation details
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    installed_by UUID NOT NULL,
    installation_workflow_id VARCHAR(255),
    
    -- Configuration
    configuration JSONB DEFAULT '{}',
    permissions JSONB DEFAULT '[]',
    
    -- Monitoring
    last_health_check TIMESTAMPTZ,
    error_count INTEGER DEFAULT 0,
    resource_usage JSONB DEFAULT '{}',
    
    UNIQUE(tenant_id, plugin_id)
);

CREATE TABLE plugin_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id UUID NOT NULL,
    plugin_id VARCHAR(255) NOT NULL,
    execution_type VARCHAR(50) NOT NULL,
    
    -- Execution details
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    status VARCHAR(20) NOT NULL DEFAULT 'running',
    
    -- Performance metrics
    duration_ms INTEGER,
    memory_used_bytes BIGINT,
    cpu_time_ms INTEGER,
    
    -- Error tracking
    error_message TEXT,
    stack_trace TEXT,
    
    INDEX idx_plugin_executions_tenant (tenant_id, started_at),
    INDEX idx_plugin_executions_plugin (plugin_id, started_at)
);
```

## Key Benefits of Temporal-First Plugin System

### 1. Reliable Plugin Operations
- **Installation workflows** handle complex multi-step processes
- **Update workflows** with automatic rollback on failure
- **Monitoring workflows** run continuously with Temporal's durability
- **Removal workflows** ensure complete cleanup

### 2. Built-in Error Handling
- **Automatic retries** for failed operations
- **State persistence** across system restarts
- **Rollback capabilities** for failed updates
- **Circuit breaker** patterns for failing plugins

### 3. Easy Plugin Management
- **Visual workflow monitoring** using Temporal UI
- **Step-by-step execution** makes debugging straightforward
- **Workflow history** shows exactly what happened
- **Replay capability** for testing and troubleshooting

### 4. Scalable Architecture
- **Horizontal scaling** with Temporal workers
- **Resource isolation** through plugin sandboxes
- **Version management** for plugin updates
- **Clear separation** between plugin logic and platform

This Temporal-first approach makes plugin management **simple, reliable, and maintainable** while handling all the complexity of plugin lifecycle through proven workflow patterns.