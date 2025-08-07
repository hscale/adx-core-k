# AI Coder Guidelines for ADX CORE Development

## Overview

This document provides comprehensive guidelines for AI coders working on ADX CORE, including coding standards, prompt templates, workflow patterns, and best practices for autonomous development within the Temporal-First architecture.

## Core AI Development Principles

### 1. Temporal-First Mindset
```
RULE: Every complex operation MUST be implemented as a Temporal workflow
REASON: Ensures reliability, observability, and maintainability at scale
IMPLEMENTATION: Use #[workflow] and #[activity] macros for all business logic
```

### 2. Multi-Tenant by Design
```
RULE: Every database query, API endpoint, and workflow MUST include tenant isolation
REASON: Prevents data leakage and ensures proper tenant boundaries
IMPLEMENTATION: Always include tenant_id in context and validate access
```

### 3. Plugin-First Architecture
```
RULE: Core functionality should be extensible through plugins
REASON: Enables customization and third-party integrations
IMPLEMENTATION: Use trait-based interfaces and dependency injection
```

### 4. API-First Development
```
RULE: Define OpenAPI specs before implementing endpoints
REASON: Ensures consistent interfaces and enables parallel development
IMPLEMENTATION: Generate types and validation from schemas
```

## AI Coding Standards

### Rust Development Standards

#### File Structure Template
```rust
// File: src/modules/{module_name}/mod.rs
//! {Module Name} - Brief description
//! 
//! This module implements {functionality} following the Temporal-First architecture.
//! All complex operations are implemented as workflows for reliability and observability.

use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use temporal_sdk::{workflow, activity, WorkflowResult, ActivityError};
use crate::common::{TenantId, DatabasePool, EventBus};

// Re-exports
pub use workflows::*;
pub use activities::*;
pub use types::*;
pub use repository::*;

// Module structure
pub mod workflows;
pub mod activities;
pub mod types;
pub mod repository;
pub mod handlers;

#[cfg(test)]
mod tests;
```

#### Workflow Implementation Template
```rust
// File: src/modules/{module_name}/workflows.rs
use super::*;

/// {Workflow Name} - Brief description
/// 
/// This workflow handles {specific functionality} with the following steps:
/// 1. Validate input and permissions
/// 2. Execute business logic through activities
/// 3. Handle errors and retries
/// 4. Return structured result
#[workflow]
pub async fn {workflow_name}_workflow(
    input: {WorkflowInput},
) -> WorkflowResult<{WorkflowOutput}> {
    // Step 1: Validate input
    let validation_result = validate_{workflow_name}_input_activity(input.clone()).await?;
    if !validation_result.is_valid {
        return Err(WorkflowError::InvalidInput(validation_result.errors));
    }
    
    // Step 2: Check permissions
    check_tenant_permissions_activity(
        input.tenant_id,
        Permission::{RequiredPermission},
    ).await?;
    
    // Step 3: Execute main business logic
    let result = execute_{workflow_name}_activity(input.clone()).await?;
    
    // Step 4: Publish events
    publish_event_activity(Event {
        event_type: "{module_name}.{workflow_name}.completed".to_string(),
        tenant_id: input.tenant_id,
        data: serde_json::to_value(&result)?,
        timestamp: Utc::now(),
    }).await?;
    
    // Step 5: Return result
    Ok({WorkflowOutput} {
        id: result.id,
        status: ProcessingStatus::Completed,
        created_at: result.created_at,
        // ... other fields
    })
}
```

#### Activity Implementation Template
```rust
// File: src/modules/{module_name}/activities.rs
use super::*;

/// Validate {workflow_name} input activity
#[activity]
pub async fn validate_{workflow_name}_input_activity(
    input: {WorkflowInput},
) -> Result<ValidationResult, ActivityError> {
    let mut errors = Vec::new();
    
    // Validate required fields
    if input.{required_field}.is_empty() {
        errors.push("Required field '{required_field}' is missing".to_string());
    }
    
    // Validate business rules
    if !input.{field}.meets_business_rule() {
        errors.push("Field '{field}' does not meet business requirements".to_string());
    }
    
    Ok(ValidationResult {
        is_valid: errors.is_empty(),
        errors,
    })
}

/// Execute {workflow_name} main logic activity
#[activity]
pub async fn execute_{workflow_name}_activity(
    input: {WorkflowInput},
) -> Result<{ActivityOutput}, ActivityError> {
    // Get database connection from context
    let db = get_database_connection().await?;
    let repository = {ModuleName}Repository::new(db);
    
    // Execute business logic
    let result = repository.{operation_name}(input.tenant_id, input.{params}).await
        .map_err(|e| ActivityError::DatabaseError(e.to_string()))?;
    
    Ok({ActivityOutput} {
        id: result.id,
        // ... map fields
    })
}
```

#### Repository Pattern Template
```rust
// File: src/modules/{module_name}/repository.rs
use super::*;

#[async_trait]
pub trait {ModuleName}Repository: Send + Sync {
    async fn create(
        &self,
        tenant_id: TenantId,
        data: Create{EntityName}Request,
    ) -> Result<{EntityName}, RepositoryError>;
    
    async fn get_by_id(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> Result<Option<{EntityName}>, RepositoryError>;
    
    async fn list_by_tenant(
        &self,
        tenant_id: TenantId,
        filters: {EntityName}Filters,
        pagination: Pagination,
    ) -> Result<PaginatedResult<{EntityName}>, RepositoryError>;
    
    async fn update(
        &self,
        tenant_id: TenantId,
        id: Uuid,
        data: Update{EntityName}Request,
    ) -> Result<{EntityName}, RepositoryError>;
    
    async fn delete(
        &self,
        tenant_id: TenantId,
        id: Uuid,
    ) -> Result<(), RepositoryError>;
}

pub struct Postgres{ModuleName}Repository {
    pool: Arc<DatabasePool>,
}

impl Postgres{ModuleName}Repository {
    pub fn new(pool: Arc<DatabasePool>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl {ModuleName}Repository for Postgres{ModuleName}Repository {
    async fn create(
        &self,
        tenant_id: TenantId,
        data: Create{EntityName}Request,
    ) -> Result<{EntityName}, RepositoryError> {
        let query = r#"
            INSERT INTO {table_name} (
                id, tenant_id, {field1}, {field2}, created_at, updated_at
            ) VALUES (
                $1, $2, $3, $4, NOW(), NOW()
            ) RETURNING *
        "#;
        
        let row = sqlx::query_as::<_, {EntityName}>(query)
            .bind(Uuid::new_v4())
            .bind(tenant_id)
            .bind(&data.{field1})
            .bind(&data.{field2})
            .fetch_one(&*self.pool)
            .await
            .map_err(|e| RepositoryError::DatabaseError(e.to_string()))?;
        
        Ok(row)
    }
    
    // ... implement other methods
}
```

### TypeScript/React Development Standards

#### Component Template
```typescript
// File: src/components/{ComponentName}/{ComponentName}.tsx
import React, { useState, useEffect } from 'react';
import { useQuery, useMutation } from '@tanstack/react-query';
import { useTemporalWorkflow } from '@/hooks/useTemporalWorkflow';
import { useTenantContext } from '@/contexts/TenantContext';
import { {ComponentName}Props, {DataType} } from './types';
import { {componentName}Api } from './api';

/**
 * {ComponentName} - Brief description
 * 
 * This component handles {functionality} with the following features:
 * - {Feature 1}
 * - {Feature 2}
 * - {Feature 3}
 */
export const {ComponentName}: React.FC<{ComponentName}Props> = ({
  {prop1},
  {prop2},
  onSuccess,
  onError,
}) => {
  const { tenantId } = useTenantContext();
  const { executeWorkflow, isExecuting } = useTemporalWorkflow('{workflow_name}');
  
  // State management
  const [localState, setLocalState] = useState<{StateType}>({
    // initial state
  });
  
  // Data fetching
  const {
    data: {dataName},
    isLoading,
    error,
    refetch,
  } = useQuery({
    queryKey: ['{queryKey}', tenantId, {dependencies}],
    queryFn: () => {componentName}Api.{fetchMethod}(tenantId, {params}),
    enabled: !!tenantId,
  });
  
  // Mutations
  const {mutationName}Mutation = useMutation({
    mutationFn: ({params}: {ParamsType}) =>
      executeWorkflow({
        operation: '{operation_name}',
        data: {params},
      }),
    onSuccess: (result) => {
      onSuccess?.(result);
      refetch();
    },
    onError: (error) => {
      onError?.(error);
    },
  });
  
  // Event handlers
  const handle{Action} = async ({params}: {ParamsType}) => {
    try {
      await {mutationName}Mutation.mutateAsync({params});
    } catch (error) {
      console.error('{Action} failed:', error);
    }
  };
  
  // Loading state
  if (isLoading) {
    return <{ComponentName}Skeleton />;
  }
  
  // Error state
  if (error) {
    return <ErrorBoundary error={error} onRetry={refetch} />;
  }
  
  return (
    <div className="{component-name}">
      {/* Component JSX */}
    </div>
  );
};

// Sub-components
const {ComponentName}Skeleton: React.FC = () => (
  <div className="animate-pulse">
    {/* Skeleton JSX */}
  </div>
);
```

#### Hook Template
```typescript
// File: src/hooks/use{HookName}.ts
import { useState, useEffect, useCallback } from 'react';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTenantContext } from '@/contexts/TenantContext';
import { {HookName}Options, {HookName}Result } from './types';

/**
 * use{HookName} - Brief description
 * 
 * This hook provides {functionality} with the following capabilities:
 * - {Capability 1}
 * - {Capability 2}
 * - {Capability 3}
 */
export const use{HookName} = (options: {HookName}Options = {}): {HookName}Result => {
  const { tenantId } = useTenantContext();
  const queryClient = useQueryClient();
  
  // Local state
  const [state, setState] = useState({
    // initial state
  });
  
  // Queries
  const query = useQuery({
    queryKey: ['{queryKey}', tenantId, options],
    queryFn: () => api.{fetchMethod}(tenantId, options),
    enabled: !!tenantId && options.enabled !== false,
  });
  
  // Mutations
  const mutation = useMutation({
    mutationFn: api.{mutationMethod},
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['{queryKey}', tenantId] });
    },
  });
  
  // Callbacks
  const {actionName} = useCallback(async ({params}: {ParamsType}) => {
    return mutation.mutateAsync({params});
  }, [mutation]);
  
  return {
    // Data
    data: query.data,
    isLoading: query.isLoading,
    error: query.error,
    
    // Actions
    {actionName},
    
    // State
    ...state,
    
    // Utilities
    refetch: query.refetch,
    reset: () => setState(/* initial state */),
  };
};
```

## AI Prompt Templates

### Module Development Prompt
```
CONTEXT: You are developing the {MODULE_NAME} module for ADX CORE, a multi-tenant SaaS platform using Temporal-First architecture.

REQUIREMENTS:
- Implement all complex operations as Temporal workflows
- Ensure multi-tenant isolation with tenant_id validation
- Follow the repository pattern for data access
- Include comprehensive error handling and logging
- Write unit and integration tests
- Generate OpenAPI specifications

SPECIFICATIONS:
{PASTE_MODULE_REQUIREMENTS_HERE}

TASKS:
1. Create module structure following the template
2. Implement workflows for all business operations
3. Create activities for atomic operations
4. Implement repository with database queries
5. Add API handlers with validation
6. Write comprehensive tests
7. Generate documentation

CONSTRAINTS:
- Use only approved dependencies
- Follow naming conventions
- Include tenant isolation in all operations
- Implement proper error handling
- Add observability (metrics, tracing, logging)

OUTPUT: Complete module implementation with all files and tests.
```

### Frontend Component Prompt
```
CONTEXT: You are developing a React component for ADX CORE's {INTERFACE_LEVEL} interface (Super Admin/Company Admin/End User).

REQUIREMENTS:
- Use TypeScript with strict type checking
- Implement responsive design with Tailwind CSS
- Follow accessibility guidelines (WCAG 2.1 AA)
- Use React Query for data fetching
- Integrate with Temporal workflows
- Include loading and error states
- Support real-time updates via WebSocket

SPECIFICATIONS:
{PASTE_COMPONENT_REQUIREMENTS_HERE}

TASKS:
1. Create component with proper TypeScript types
2. Implement data fetching with React Query
3. Add form validation and error handling
4. Create responsive design with Tailwind
5. Add accessibility features
6. Implement real-time updates
7. Write unit tests with React Testing Library
8. Create Storybook stories

CONSTRAINTS:
- Follow design system tokens
- Use approved UI components
- Implement proper error boundaries
- Include loading skeletons
- Support keyboard navigation

OUTPUT: Complete component implementation with tests and stories.
```

### Plugin Development Prompt
```
CONTEXT: You are developing a plugin for ADX CORE that extends platform functionality.

REQUIREMENTS:
- Implement plugin trait with lifecycle methods
- Support multi-language SDKs (Rust, Python, Node.js, Go)
- Include database migrations if needed
- Add API endpoints with proper validation
- Implement event handlers
- Support tenant-specific configuration
- Include comprehensive documentation

SPECIFICATIONS:
{PASTE_PLUGIN_REQUIREMENTS_HERE}

TASKS:
1. Implement core plugin trait in Rust
2. Create language-specific SDK implementations
3. Add database migrations and repository
4. Implement API endpoints with OpenAPI specs
5. Add event handlers and publishers
6. Create configuration schema
7. Write integration tests
8. Generate SDK documentation

CONSTRAINTS:
- Maintain backward compatibility
- Follow plugin security guidelines
- Include proper error handling
- Support plugin versioning
- Add telemetry and monitoring

OUTPUT: Complete plugin implementation with multi-language SDKs.
```

### Database Migration Prompt
```
CONTEXT: You are creating database migrations for ADX CORE with zero-downtime requirements.

REQUIREMENTS:
- Support multiple database providers (PostgreSQL, MySQL)
- Ensure multi-tenant data isolation
- Implement backward-compatible changes
- Include rollback procedures
- Add data validation and integrity checks
- Support horizontal sharding
- Include performance optimizations

SPECIFICATIONS:
{PASTE_MIGRATION_REQUIREMENTS_HERE}

TASKS:
1. Create forward migration SQL
2. Create rollback migration SQL
3. Add data validation queries
4. Implement migration in Rust
5. Add integration tests
6. Create performance benchmarks
7. Document migration procedure

CONSTRAINTS:
- Zero-downtime deployment
- Backward compatibility required
- Multi-tenant isolation
- Performance impact < 5%
- Rollback safety guaranteed

OUTPUT: Complete migration with SQL, Rust code, and tests.
```

## Workflow Patterns

### Standard Workflow Pattern
```rust
#[workflow]
pub async fn standard_workflow_pattern(
    input: WorkflowInput,
) -> WorkflowResult<WorkflowOutput> {
    // 1. Input Validation
    let validation = validate_input_activity(input.clone()).await?;
    if !validation.is_valid {
        return Err(WorkflowError::InvalidInput(validation.errors));
    }
    
    // 2. Permission Check
    check_permissions_activity(
        input.tenant_id,
        input.user_id,
        RequiredPermission,
    ).await?;
    
    // 3. Pre-processing
    let preprocessed = preprocess_data_activity(input.clone()).await?;
    
    // 4. Main Business Logic (with retry)
    let result = execute_main_logic_activity(preprocessed)
        .await
        .retry_on_failure(3, Duration::from_secs(5))?;
    
    // 5. Post-processing
    let final_result = postprocess_result_activity(result).await?;
    
    // 6. Event Publishing
    publish_completion_event_activity(
        input.tenant_id,
        final_result.clone(),
    ).await?;
    
    // 7. Cleanup (if needed)
    cleanup_resources_activity(input.resource_ids).await?;
    
    Ok(final_result)
}
```

### Error Handling Pattern
```rust
impl From<DatabaseError> for ActivityError {
    fn from(error: DatabaseError) -> Self {
        match error {
            DatabaseError::ConnectionFailed => {
                ActivityError::Retryable("Database connection failed".to_string())
            }
            DatabaseError::ConstraintViolation(msg) => {
                ActivityError::NonRetryable(format!("Constraint violation: {}", msg))
            }
            DatabaseError::NotFound => {
                ActivityError::NonRetryable("Resource not found".to_string())
            }
            _ => ActivityError::Retryable(error.to_string()),
        }
    }
}
```

## Testing Standards

### Unit Test Template
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use temporal_sdk::testing::WorkflowTestEnv;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_{workflow_name}_success() {
        // Arrange
        let mut env = WorkflowTestEnv::new().await;
        let input = {WorkflowInput} {
            tenant_id: TenantId::new_v4(),
            // ... test data
        };
        
        // Mock activities
        env.register_activity(mock_validate_input_activity);
        env.register_activity(mock_execute_main_logic_activity);
        
        // Act
        let result = env.execute_workflow({workflow_name}_workflow, input).await;
        
        // Assert
        assert!(result.is_ok());
        let output = result.unwrap();
        assert_eq!(output.status, ProcessingStatus::Completed);
    }
    
    #[tokio::test]
    async fn test_{workflow_name}_invalid_input() {
        // Test error scenarios
    }
    
    #[tokio::test]
    async fn test_{workflow_name}_permission_denied() {
        // Test permission scenarios
    }
}
```

### Integration Test Template
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use testcontainers::*;
    
    #[tokio::test]
    async fn test_{module_name}_end_to_end() {
        // Setup test environment
        let docker = clients::Cli::default();
        let postgres = docker.run(images::postgres::Postgres::default());
        let db_url = format!("postgresql://postgres:postgres@localhost:{}/postgres", 
            postgres.get_host_port_ipv4(5432));
        
        // Initialize test database
        let pool = create_database_pool(&db_url).await.unwrap();
        run_migrations(&pool).await.unwrap();
        
        // Create test tenant
        let tenant_id = create_test_tenant(&pool).await.unwrap();
        
        // Execute test scenario
        let result = execute_test_scenario(tenant_id, &pool).await;
        
        // Verify results
        assert!(result.is_ok());
        verify_database_state(&pool, tenant_id).await.unwrap();
    }
}
```

## Code Quality Standards

### Mandatory Code Checks
```yaml
# .github/workflows/code-quality.yml
name: Code Quality
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      # Rust checks
      - name: Rust Format Check
        run: cargo fmt -- --check
        
      - name: Rust Clippy
        run: cargo clippy -- -D warnings
        
      - name: Rust Tests
        run: cargo test
        
      # TypeScript checks
      - name: TypeScript Type Check
        run: npm run type-check
        
      - name: ESLint
        run: npm run lint
        
      - name: Prettier Check
        run: npm run format:check
        
      # Security checks
      - name: Cargo Audit
        run: cargo audit
        
      - name: npm Audit
        run: npm audit
        
      # Documentation
      - name: Generate Docs
        run: cargo doc --no-deps
```

### Performance Requirements
```
PERFORMANCE TARGETS:
- API Response Time: < 200ms (95th percentile)
- Database Query Time: < 50ms (95th percentile)
- Workflow Execution: < 5s for standard operations
- Memory Usage: < 512MB per service instance
- CPU Usage: < 70% under normal load

MONITORING:
- All functions must include timing metrics
- Database queries must be traced
- Memory allocations must be tracked
- Error rates must be monitored
```

This comprehensive guide ensures all AI coders follow consistent patterns and produce high-quality, maintainable code that integrates seamlessly with the ADX CORE architecture.