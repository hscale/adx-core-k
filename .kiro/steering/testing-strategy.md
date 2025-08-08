# ADX CORE Testing Strategy Guidelines

## Core Principles

ADX CORE testing follows a comprehensive multi-layered approach that covers unit tests, integration tests, workflow tests, cross-service tests, and end-to-end tests. All testing leverages Temporal's testing capabilities for workflow reliability and supports multi-tenant isolation testing.

## Testing Architecture

### Testing Pyramid Structure
```
                    E2E Tests
                 (Cross-Platform)
                /               \
           Integration Tests    Workflow Tests
          (Cross-Service)      (Temporal-Focused)
         /                                      \
    Unit Tests                              Contract Tests
   (Individual                             (API/Event)
    Components)                                    \
                                            Performance Tests
                                           (Load/Stress/Chaos)
```

## Unit Testing Patterns

### Rust Backend Unit Tests
```rust
// Service unit tests with mocking
#[cfg(test)]
mod tenant_service_tests {
    use super::*;
    use mockall::predicate::*;
    use tokio_test;
    
    #[tokio::test]
    async fn test_create_tenant_validation() {
        // Arrange
        let mut mock_repo = MockTenantRepository::new();
        mock_repo
            .expect_find_by_name()
            .with(eq("Test Tenant"))
            .times(1)
            .returning(|_| Ok(None));
        
        mock_repo
            .expect_create()
            .times(1)
            .returning(|tenant| Ok(tenant));
        
        let service = TenantService::new(Arc::new(mock_repo));
        
        // Act
        let result = service.create_tenant(CreateTenantRequest {
            name: "Test Tenant".to_string(),
            admin_email: "admin@test.com".to_string(),
            subscription_tier: SubscriptionTier::Professional,
        }).await;
        
        // Assert
        assert!(result.is_ok());
        let tenant = result.unwrap();
        assert_eq!(tenant.name, "Test Tenant");
        assert_eq!(tenant.subscription_tier, SubscriptionTier::Professional);
    }
    
    #[tokio::test]
    async fn test_create_tenant_duplicate_name() {
        // Arrange
        let mut mock_repo = MockTenantRepository::new();
        mock_repo
            .expect_find_by_name()
            .with(eq("Existing Tenant"))
            .times(1)
            .returning(|_| Ok(Some(Tenant::default())));
        
        let service = TenantService::new(Arc::new(mock_repo));
        
        // Act
        let result = service.create_tenant(CreateTenantRequest {
            name: "Existing Tenant".to_string(),
            admin_email: "admin@test.com".to_string(),
            subscription_tier: SubscriptionTier::Professional,
        }).await;
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ServiceError::TenantAlreadyExists(_)));
    }
}

// Repository unit tests with test database
#[cfg(test)]
mod tenant_repository_tests {
    use super::*;
    use sqlx::PgPool;
    use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
    
    struct TestContext {
        _container: Container<'static, Postgres>,
        pool: PgPool,
    }
    
    impl TestContext {
        async fn new() -> Self {
            let docker = Cli::default();
            let container = docker.run(Postgres::default());
            let connection_string = format!(
                "postgres://postgres:postgres@127.0.0.1:{}/postgres",
                container.get_host_port_ipv4(5432)
            );
            
            let pool = PgPool::connect(&connection_string).await.unwrap();
            
            // Run migrations
            sqlx::migrate!("./migrations").run(&pool).await.unwrap();
            
            Self {
                _container: container,
                pool,
            }
        }
    }
    
    #[tokio::test]
    async fn test_tenant_crud_operations() {
        let ctx = TestContext::new().await;
        let repo = TenantRepository::new(ctx.pool.clone());
        
        // Test create
        let tenant = Tenant {
            id: "test-tenant-id".to_string(),
            name: "Test Tenant".to_string(),
            admin_email: "admin@test.com".to_string(),
            subscription_tier: SubscriptionTier::Professional,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        let created = repo.create(tenant.clone()).await.unwrap();
        assert_eq!(created.id, tenant.id);
        
        // Test find by id
        let found = repo.find_by_id(&tenant.id).await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, tenant.name);
        
        // Test update
        let updated_tenant = Tenant {
            name: "Updated Tenant".to_string(),
            ..tenant.clone()
        };
        
        let updated = repo.update(updated_tenant.clone()).await.unwrap();
        assert_eq!(updated.name, "Updated Tenant");
        
        // Test delete
        repo.delete(&tenant.id).await.unwrap();
        let deleted = repo.find_by_id(&tenant.id).await.unwrap();
        assert!(deleted.is_none());
    }
}
```

### Frontend Unit Tests
```typescript
// React component unit tests
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TenantProvider } from '@adx-core/shared-context';
import { TenantSwitcher } from '../TenantSwitcher';

const createTestWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return ({ children }: { children: React.ReactNode }) => (
    <QueryClientProvider client={queryClient}>
      <TenantProvider>
        {children}
      </TenantProvider>
    </QueryClientProvider>
  );
};

describe('TenantSwitcher', () => {
  it('should display available tenants', async () => {
    const mockTenants = [
      { id: 'tenant1', name: 'Tenant 1' },
      { id: 'tenant2', name: 'Tenant 2' },
    ];

    const mockTenantContext = {
      currentTenant: mockTenants[0],
      availableTenants: mockTenants,
      switchTenant: jest.fn(),
    };

    jest.spyOn(require('@adx-core/shared-context'), 'useTenantContext')
      .mockReturnValue(mockTenantContext);

    render(<TenantSwitcher />, { wrapper: createTestWrapper() });

    expect(screen.getByDisplayValue('Tenant 1')).toBeInTheDocument();
    expect(screen.getByText('Tenant 2')).toBeInTheDocument();
  });

  it('should handle tenant switching', async () => {
    const mockSwitchTenant = jest.fn().mockResolvedValue(undefined);
    const mockTenantContext = {
      currentTenant: { id: 'tenant1', name: 'Tenant 1' },
      availableTenants: [
        { id: 'tenant1', name: 'Tenant 1' },
        { id: 'tenant2', name: 'Tenant 2' },
      ],
      switchTenant: mockSwitchTenant,
    };

    jest.spyOn(require('@adx-core/shared-context'), 'useTenantContext')
      .mockReturnValue(mockTenantContext);

    // Mock fetch for workflow API
    global.fetch = jest.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ operationId: 'op-123' }),
    });

    render(<TenantSwitcher />, { wrapper: createTestWrapper() });

    const select = screen.getByDisplayValue('Tenant 1');
    fireEvent.change(select, { target: { value: 'tenant2' } });

    await waitFor(() => {
      expect(fetch).toHaveBeenCalledWith('/api/workflows/switch-tenant', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': expect.stringContaining('Bearer'),
        },
        body: JSON.stringify({
          targetTenantId: 'tenant2',
          currentTenantId: 'tenant1',
        }),
      });
    });
  });
});

// Hook unit tests
import { renderHook, waitFor } from '@testing-library/react';
import { useBFFQuery } from '../hooks/useBFFQuery';

describe('useBFFQuery', () => {
  it('should fetch data from BFF endpoint', async () => {
    const mockData = { users: [{ id: '1', name: 'John Doe' }] };
    
    global.fetch = jest.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(mockData),
    });

    const { result } = renderHook(
      () => useBFFQuery(['users'], '/api/bff/users'),
      { wrapper: createTestWrapper() }
    );

    await waitFor(() => {
      expect(result.current.isSuccess).toBe(true);
    });

    expect(result.current.data).toEqual(mockData);
    expect(fetch).toHaveBeenCalledWith(
      expect.stringContaining('/api/bff/users'),
      expect.objectContaining({
        headers: expect.objectContaining({
          'Authorization': expect.stringContaining('Bearer'),
          'X-Tenant-ID': expect.any(String),
        }),
      })
    );
  });
});
```

## Temporal Workflow Testing

### Workflow Unit Tests
```rust
#[cfg(test)]
mod workflow_tests {
    use super::*;
    use temporal_sdk_core_test_utils::TestWorkflowEnvironment;
    use mockall::predicate::*;
    
    #[tokio::test]
    async fn test_create_tenant_workflow_success() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock activities
        let mut mock_activities = MockTenantActivities::new();
        mock_activities
            .expect_validate_tenant_creation()
            .times(1)
            .returning(|_| Ok(TenantValidationResult {
                is_valid: true,
                tenant_id: "test-tenant-id".to_string(),
                errors: vec![],
            }));
        
        mock_activities
            .expect_setup_tenant_database()
            .times(1)
            .returning(|_| Ok(DatabaseSetupResult {
                connection_string: "postgres://...".to_string(),
                schema_created: true,
            }));
        
        mock_activities
            .expect_create_tenant_config()
            .times(1)
            .returning(|_| Ok(TenantConfig::default()));
        
        mock_activities
            .expect_create_tenant_admin()
            .times(1)
            .returning(|_| Ok(AdminUser {
                user_id: "admin-user-id".to_string(),
                temporary_password: "temp-pass-123".to_string(),
            }));
        
        mock_activities
            .expect_send_tenant_welcome()
            .times(1)
            .returning(|_| Ok(()));
        
        // Execute workflow
        let result = test_env.execute_workflow(
            create_tenant_workflow,
            CreateTenantRequest {
                tenant_name: "Test Tenant".to_string(),
                admin_email: "admin@test.com".to_string(),
                subscription_tier: SubscriptionTier::Professional,
                isolation_level: TenantIsolationLevel::Schema,
                quotas: TenantQuotas::default(),
                features: vec!["basic_features".to_string()],
                default_modules: vec!["client_management".to_string()],
            },
        ).await;
        
        // Assert
        assert!(result.is_ok());
        let workflow_result = result.unwrap();
        assert_eq!(workflow_result.tenant_id, "test-tenant-id");
        assert_eq!(workflow_result.admin_user_id, "admin-user-id");
    }
    
    #[tokio::test]
    async fn test_create_tenant_workflow_validation_failure() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock validation failure
        let mut mock_activities = MockTenantActivities::new();
        mock_activities
            .expect_validate_tenant_creation()
            .times(1)
            .returning(|_| Ok(TenantValidationResult {
                is_valid: false,
                tenant_id: String::new(),
                errors: vec!["Tenant name already exists".to_string()],
            }));
        
        // Execute workflow
        let result = test_env.execute_workflow(
            create_tenant_workflow,
            CreateTenantRequest {
                tenant_name: "Existing Tenant".to_string(),
                admin_email: "admin@test.com".to_string(),
                subscription_tier: SubscriptionTier::Professional,
                isolation_level: TenantIsolationLevel::Schema,
                quotas: TenantQuotas::default(),
                features: vec![],
                default_modules: vec![],
            },
        ).await;
        
        // Assert
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            WorkflowError::ValidationFailed(_)
        ));
    }
    
    #[tokio::test]
    async fn test_create_tenant_workflow_compensation() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock successful validation and database setup, but failed config creation
        let mut mock_activities = MockTenantActivities::new();
        mock_activities
            .expect_validate_tenant_creation()
            .times(1)
            .returning(|_| Ok(TenantValidationResult {
                is_valid: true,
                tenant_id: "test-tenant-id".to_string(),
                errors: vec![],
            }));
        
        mock_activities
            .expect_setup_tenant_database()
            .times(1)
            .returning(|_| Ok(DatabaseSetupResult {
                connection_string: "postgres://...".to_string(),
                schema_created: true,
            }));
        
        mock_activities
            .expect_create_tenant_config()
            .times(1)
            .returning(|_| Err(ActivityError::DatabaseError("Config creation failed".to_string())));
        
        // Expect compensation activity
        mock_activities
            .expect_cleanup_tenant_database()
            .times(1)
            .returning(|_| Ok(()));
        
        // Execute workflow
        let result = test_env.execute_workflow(
            create_tenant_workflow,
            CreateTenantRequest::default(),
        ).await;
        
        // Assert
        assert!(result.is_err());
        // Verify compensation was called
        mock_activities.checkpoint();
    }
}

// Workflow replay tests for versioning
#[tokio::test]
async fn test_workflow_replay_compatibility() {
    let test_env = TestWorkflowEnvironment::new().await;
    
    // Load historical workflow execution
    let workflow_history = load_workflow_history("create_tenant_v1_history.json");
    
    // Replay with current workflow implementation
    let replay_result = test_env.replay_workflow(
        create_tenant_workflow,
        workflow_history,
    ).await;
    
    // Assert replay succeeds (backward compatibility)
    assert!(replay_result.is_ok());
}
```

### Activity Integration Tests
```rust
#[cfg(test)]
mod activity_integration_tests {
    use super::*;
    use testcontainers::{clients::Cli, images::postgres::Postgres};
    
    #[tokio::test]
    async fn test_tenant_activities_integration() {
        // Set up test environment
        let docker = Cli::default();
        let postgres = docker.run(Postgres::default());
        let database_url = format!(
            "postgres://postgres:postgres@127.0.0.1:{}/postgres",
            postgres.get_host_port_ipv4(5432)
        );
        
        let pool = PgPool::connect(&database_url).await.unwrap();
        sqlx::migrate!("./migrations").run(&pool).await.unwrap();
        
        let activities = TenantActivities::new(pool.clone());
        
        // Test validate_tenant_creation activity
        let validation_result = activities.validate_tenant_creation(
            ValidateTenantCreationRequest {
                tenant_name: "Integration Test Tenant".to_string(),
                admin_email: "admin@integration.com".to_string(),
                subscription_tier: SubscriptionTier::Professional,
            }
        ).await.unwrap();
        
        assert!(validation_result.is_valid);
        assert!(!validation_result.tenant_id.is_empty());
        
        // Test setup_tenant_database activity
        let database_setup = activities.setup_tenant_database(
            SetupTenantDatabaseRequest {
                tenant_id: validation_result.tenant_id.clone(),
                isolation_level: TenantIsolationLevel::Schema,
                initial_schema: None,
            }
        ).await.unwrap();
        
        assert!(database_setup.schema_created);
        assert!(!database_setup.connection_string.is_empty());
        
        // Test create_tenant_config activity
        let tenant_config = activities.create_tenant_config(
            CreateTenantConfigRequest {
                tenant_id: validation_result.tenant_id.clone(),
                tenant_name: "Integration Test Tenant".to_string(),
                subscription_tier: SubscriptionTier::Professional,
                quotas: TenantQuotas::default(),
                features: vec!["basic_features".to_string()],
            }
        ).await.unwrap();
        
        assert_eq!(tenant_config.tenant_id, validation_result.tenant_id);
        assert_eq!(tenant_config.subscription_tier, SubscriptionTier::Professional);
    }
}
```

## Integration Testing

### Cross-Service Integration Tests
```rust
#[cfg(test)]
mod cross_service_integration_tests {
    use super::*;
    use axum_test::TestServer;
    use std::collections::HashMap;
    
    struct IntegrationTestEnvironment {
        servers: HashMap<String, TestServer>,
        temporal_client: TemporalClient,
        database_pool: PgPool,
    }
    
    impl IntegrationTestEnvironment {
        async fn new() -> Self {
            // Set up test databases
            let docker = Cli::default();
            let postgres = docker.run(Postgres::default());
            let database_url = format!(
                "postgres://postgres:postgres@127.0.0.1:{}/postgres",
                postgres.get_host_port_ipv4(5432)
            );
            
            let pool = PgPool::connect(&database_url).await.unwrap();
            sqlx::migrate!("./migrations").run(&pool).await.unwrap();
            
            // Set up Temporal test server
            let temporal_client = TemporalClient::new("localhost:7233").await.unwrap();
            
            // Set up service test servers
            let mut servers = HashMap::new();
            
            servers.insert(
                "api_gateway".to_string(),
                TestServer::new(create_api_gateway_app(pool.clone(), temporal_client.clone())).unwrap()
            );
            
            servers.insert(
                "auth_service".to_string(),
                TestServer::new(create_auth_service_app(pool.clone())).unwrap()
            );
            
            servers.insert(
                "tenant_service".to_string(),
                TestServer::new(create_tenant_service_app(pool.clone())).unwrap()
            );
            
            servers.insert(
                "user_service".to_string(),
                TestServer::new(create_user_service_app(pool.clone())).unwrap()
            );
            
            Self {
                servers,
                temporal_client,
                database_pool: pool,
            }
        }
    }
    
    #[tokio::test]
    async fn test_end_to_end_tenant_creation() {
        let env = IntegrationTestEnvironment::new().await;
        
        // Step 1: Create tenant through API Gateway
        let create_response = env.servers["api_gateway"]
            .post("/api/v1/workflows/create-tenant")
            .json(&CreateTenantWorkflowRequest {
                tenant_name: "E2E Test Tenant".to_string(),
                admin_email: "admin@e2e.com".to_string(),
                subscription_tier: SubscriptionTier::Professional,
                isolation_level: TenantIsolationLevel::Schema,
                quotas: TenantQuotas::default(),
                features: vec!["basic_features".to_string()],
                default_modules: vec!["client_management".to_string()],
            })
            .await;
        
        create_response.assert_status_accepted();
        let workflow_response: WorkflowApiResponse<CreateTenantResult> = create_response.json();
        
        let operation_id = match workflow_response {
            WorkflowApiResponse::Asynchronous { operation_id, .. } => operation_id,
            _ => panic!("Expected asynchronous workflow response"),
        };
        
        // Step 2: Poll workflow status until completion
        let mut workflow_completed = false;
        let mut attempts = 0;
        let max_attempts = 30; // 30 seconds timeout
        
        while !workflow_completed && attempts < max_attempts {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            
            let status_response = env.servers["api_gateway"]
                .get(&format!("/api/v1/workflows/{}/status", operation_id))
                .await;
            
            status_response.assert_status_ok();
            let status: WorkflowStatusResponse = status_response.json();
            
            match status.status {
                WorkflowStatus::Completed => {
                    workflow_completed = true;
                    
                    // Verify workflow result
                    let result: CreateTenantResult = serde_json::from_value(
                        status.result.unwrap()
                    ).unwrap();
                    
                    assert!(!result.tenant_id.is_empty());
                    assert!(!result.admin_user_id.is_empty());
                }
                WorkflowStatus::Failed => {
                    panic!("Workflow failed: {:?}", status.error);
                }
                _ => {
                    attempts += 1;
                }
            }
        }
        
        assert!(workflow_completed, "Workflow did not complete within timeout");
        
        // Step 3: Verify tenant was created in tenant service
        let tenant_list_response = env.servers["tenant_service"]
            .get("/api/v1/tenants")
            .await;
        
        tenant_list_response.assert_status_ok();
        let tenants: Vec<Tenant> = tenant_list_response.json();
        
        let created_tenant = tenants.iter()
            .find(|t| t.name == "E2E Test Tenant")
            .expect("Created tenant not found");
        
        assert_eq!(created_tenant.admin_email, "admin@e2e.com");
        assert_eq!(created_tenant.subscription_tier, SubscriptionTier::Professional);
        
        // Step 4: Verify admin user was created in user service
        let user_list_response = env.servers["user_service"]
            .get(&format!("/api/v1/tenants/{}/users", created_tenant.id))
            .await;
        
        user_list_response.assert_status_ok();
        let users: Vec<User> = user_list_response.json();
        
        let admin_user = users.iter()
            .find(|u| u.email == "admin@e2e.com")
            .expect("Admin user not found");
        
        assert!(admin_user.roles.contains(&"admin".to_string()));
    }
    
    #[tokio::test]
    async fn test_cross_service_tenant_switching() {
        let env = IntegrationTestEnvironment::new().await;
        
        // Set up test data: create two tenants and a user with access to both
        let tenant1 = create_test_tenant(&env, "Tenant 1", "admin1@test.com").await;
        let tenant2 = create_test_tenant(&env, "Tenant 2", "admin2@test.com").await;
        let user = create_test_user(&env, "user@test.com", vec![tenant1.id.clone(), tenant2.id.clone()]).await;
        
        // Step 1: Authenticate user in tenant1
        let auth_response = env.servers["auth_service"]
            .post("/api/v1/auth/login")
            .json(&LoginRequest {
                email: "user@test.com".to_string(),
                password: "password123".to_string(),
                tenant_id: Some(tenant1.id.clone()),
            })
            .await;
        
        auth_response.assert_status_ok();
        let auth_result: AuthResult = auth_response.json();
        let token = auth_result.token;
        
        // Step 2: Switch to tenant2 through workflow
        let switch_response = env.servers["api_gateway"]
            .post("/api/v1/workflows/switch-tenant")
            .header("Authorization", format!("Bearer {}", token))
            .json(&SwitchTenantRequest {
                target_tenant_id: tenant2.id.clone(),
                current_tenant_id: tenant1.id.clone(),
            })
            .await;
        
        switch_response.assert_status_accepted();
        let switch_workflow: WorkflowApiResponse<SwitchTenantResult> = switch_response.json();
        
        // Step 3: Poll for completion and verify result
        let operation_id = match switch_workflow {
            WorkflowApiResponse::Asynchronous { operation_id, .. } => operation_id,
            WorkflowApiResponse::Synchronous { data, .. } => {
                // Verify immediate result
                assert_eq!(data.new_tenant_id, tenant2.id);
                return;
            }
        };
        
        // Poll for async completion
        let final_status = poll_workflow_completion(&env, &operation_id).await;
        let switch_result: SwitchTenantResult = serde_json::from_value(
            final_status.result.unwrap()
        ).unwrap();
        
        assert_eq!(switch_result.new_tenant_id, tenant2.id);
        assert!(!switch_result.new_session_id.is_empty());
        
        // Step 4: Verify user context updated in user service
        let user_context_response = env.servers["user_service"]
            .get(&format!("/api/v1/users/{}/context", user.id))
            .header("Authorization", format!("Bearer {}", switch_result.new_session_id))
            .await;
        
        user_context_response.assert_status_ok();
        let user_context: UserContext = user_context_response.json();
        assert_eq!(user_context.active_tenant_id, tenant2.id);
    }
}
```

## End-to-End Testing

### Cross-Platform E2E Tests
```typescript
// Playwright E2E tests
import { test, expect, Page } from '@playwright/test';

test.describe('ADX Core E2E Tests', () => {
  let page: Page;

  test.beforeEach(async ({ browser }) => {
    page = await browser.newPage();
    
    // Set up test tenant and user
    await setupTestEnvironment();
  });

  test('complete user workflow: login, tenant switch, file upload', async () => {
    // Step 1: Login
    await page.goto('http://localhost:3000/auth/login');
    
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');
    
    // Wait for dashboard to load
    await page.waitForSelector('[data-testid="dashboard"]');
    await expect(page.locator('[data-testid="user-name"]')).toHaveText('Test User');
    
    // Step 2: Switch tenant
    await page.selectOption('[data-testid="tenant-switcher"]', 'tenant-2');
    
    // Wait for tenant switch workflow to complete
    await page.waitForSelector('[data-testid="tenant-switch-complete"]');
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('Tenant 2');
    
    // Step 3: Navigate to files and upload
    await page.click('[data-testid="nav-files"]');
    await page.waitForSelector('[data-testid="file-list"]');
    
    // Upload file
    const fileInput = page.locator('[data-testid="file-upload-input"]');
    await fileInput.setInputFiles('./test-files/sample.pdf');
    
    // Wait for upload workflow to complete
    await page.waitForSelector('[data-testid="upload-complete"]');
    
    // Verify file appears in list
    await expect(page.locator('[data-testid="file-list"]')).toContainText('sample.pdf');
    
    // Step 4: Verify workflow tracking
    await page.click('[data-testid="nav-workflows"]');
    await page.waitForSelector('[data-testid="workflow-list"]');
    
    // Should see tenant switch and file upload workflows
    await expect(page.locator('[data-testid="workflow-list"]')).toContainText('Tenant Switch');
    await expect(page.locator('[data-testid="workflow-list"]')).toContainText('File Upload');
    
    // Verify workflow statuses are completed
    const workflows = page.locator('[data-testid="workflow-item"]');
    const count = await workflows.count();
    
    for (let i = 0; i < count; i++) {
      const workflow = workflows.nth(i);
      await expect(workflow.locator('[data-testid="workflow-status"]')).toHaveText('Completed');
    }
  });

  test('module installation and usage workflow', async () => {
    // Login and navigate to modules
    await loginAsAdmin(page);
    await page.click('[data-testid="nav-modules"]');
    
    // Install a module from marketplace
    await page.click('[data-testid="marketplace-tab"]');
    await page.click('[data-testid="install-client-management"]');
    
    // Wait for installation workflow
    await page.waitForSelector('[data-testid="installation-complete"]');
    
    // Activate module
    await page.click('[data-testid="installed-modules-tab"]');
    await page.click('[data-testid="activate-client-management"]');
    
    // Wait for activation workflow
    await page.waitForSelector('[data-testid="activation-complete"]');
    
    // Verify module appears in navigation
    await expect(page.locator('[data-testid="nav-client-management"]')).toBeVisible();
    
    // Use module functionality
    await page.click('[data-testid="nav-client-management"]');
    await page.waitForSelector('[data-testid="client-list"]');
    
    // Create a client
    await page.click('[data-testid="create-client-button"]');
    await page.fill('[data-testid="client-name-input"]', 'Test Client');
    await page.fill('[data-testid="client-email-input"]', 'client@test.com');
    await page.click('[data-testid="save-client-button"]');
    
    // Wait for client creation workflow
    await page.waitForSelector('[data-testid="client-created"]');
    
    // Verify client appears in list
    await expect(page.locator('[data-testid="client-list"]')).toContainText('Test Client');
  });

  test('cross-platform consistency (web vs desktop)', async () => {
    // Test web version
    await page.goto('http://localhost:3000');
    await loginAsUser(page);
    
    const webFeatures = await extractFeatureList(page);
    
    // Test desktop version (if available)
    if (process.env.TEST_DESKTOP === 'true') {
      const { _electron: electron } = require('playwright');
      const electronApp = await electron.launch({
        args: ['./dist-electron/main.js'],
      });
      
      const desktopPage = await electronApp.firstWindow();
      await loginAsUser(desktopPage);
      
      const desktopFeatures = await extractFeatureList(desktopPage);
      
      // Verify feature parity
      expect(desktopFeatures).toEqual(webFeatures);
      
      await electronApp.close();
    }
  });
});

// Helper functions
async function setupTestEnvironment() {
  // Create test tenants, users, and data
  const response = await fetch('http://localhost:8080/api/test/setup', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      tenants: [
        { name: 'Tenant 1', adminEmail: 'admin1@test.com' },
        { name: 'Tenant 2', adminEmail: 'admin2@test.com' },
      ],
      users: [
        { email: 'test@example.com', password: 'password123', tenants: ['tenant-1', 'tenant-2'] },
        { email: 'admin@example.com', password: 'admin123', role: 'admin' },
      ],
    }),
  });
  
  if (!response.ok) {
    throw new Error('Failed to set up test environment');
  }
}

async function loginAsUser(page: Page) {
  await page.goto('http://localhost:3000/auth/login');
  await page.fill('[data-testid="email-input"]', 'test@example.com');
  await page.fill('[data-testid="password-input"]', 'password123');
  await page.click('[data-testid="login-button"]');
  await page.waitForSelector('[data-testid="dashboard"]');
}

async function loginAsAdmin(page: Page) {
  await page.goto('http://localhost:3000/auth/login');
  await page.fill('[data-testid="email-input"]', 'admin@example.com');
  await page.fill('[data-testid="password-input"]', 'admin123');
  await page.click('[data-testid="login-button"]');
  await page.waitForSelector('[data-testid="dashboard"]');
}

async function extractFeatureList(page: Page): Promise<string[]> {
  const features = await page.locator('[data-testid="feature-item"]').allTextContents();
  return features.sort();
}
```

## Performance Testing

### Load Testing with Temporal Workflows
```rust
// Load testing framework
#[cfg(test)]
mod performance_tests {
    use super::*;
    use tokio::time::{Duration, Instant};
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};
    
    #[tokio::test]
    async fn test_concurrent_workflow_execution() {
        let temporal_client = Arc::new(TemporalClient::new("localhost:7233").await.unwrap());
        let success_count = Arc::new(AtomicU64::new(0));
        let error_count = Arc::new(AtomicU64::new(0));
        
        let start_time = Instant::now();
        let concurrent_workflows = 100;
        let mut handles = Vec::new();
        
        for i in 0..concurrent_workflows {
            let client = temporal_client.clone();
            let success_counter = success_count.clone();
            let error_counter = error_count.clone();
            
            let handle = tokio::spawn(async move {
                let workflow_id = format!("load_test_workflow_{}", i);
                
                match client.start_workflow(
                    "create_tenant_workflow",
                    workflow_id,
                    "tenant-task-queue",
                    CreateTenantRequest {
                        tenant_name: format!("Load Test Tenant {}", i),
                        admin_email: format!("admin{}@loadtest.com", i),
                        subscription_tier: SubscriptionTier::Professional,
                        isolation_level: TenantIsolationLevel::Schema,
                        quotas: TenantQuotas::default(),
                        features: vec![],
                        default_modules: vec![],
                    },
                ).await {
                    Ok(handle) => {
                        match handle.get_result().await {
                            Ok(_) => success_counter.fetch_add(1, Ordering::Relaxed),
                            Err(_) => error_counter.fetch_add(1, Ordering::Relaxed),
                        };
                    }
                    Err(_) => {
                        error_counter.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
            
            handles.push(handle);
        }
        
        // Wait for all workflows to complete
        for handle in handles {
            handle.await.unwrap();
        }
        
        let duration = start_time.elapsed();
        let success = success_count.load(Ordering::Relaxed);
        let errors = error_count.load(Ordering::Relaxed);
        
        println!("Load test results:");
        println!("  Duration: {:?}", duration);
        println!("  Successful workflows: {}", success);
        println!("  Failed workflows: {}", errors);
        println!("  Workflows per second: {:.2}", concurrent_workflows as f64 / duration.as_secs_f64());
        
        // Assert performance criteria
        assert!(duration < Duration::from_secs(60), "Load test took too long");
        assert!(success >= concurrent_workflows * 95 / 100, "Success rate below 95%");
        assert!(success as f64 / duration.as_secs_f64() >= 10.0, "Throughput below 10 workflows/second");
    }
    
    #[tokio::test]
    async fn test_api_response_times() {
        let client = reqwest::Client::new();
        let base_url = "http://localhost:8080";
        let iterations = 1000;
        
        let mut response_times = Vec::new();
        
        for _ in 0..iterations {
            let start = Instant::now();
            
            let response = client
                .get(&format!("{}/api/v1/tenants", base_url))
                .header("Authorization", "Bearer test-token")
                .send()
                .await
                .unwrap();
            
            assert!(response.status().is_success());
            let _ = response.text().await.unwrap();
            
            response_times.push(start.elapsed());
        }
        
        // Calculate statistics
        response_times.sort();
        let p50 = response_times[iterations * 50 / 100];
        let p95 = response_times[iterations * 95 / 100];
        let p99 = response_times[iterations * 99 / 100];
        
        println!("API response time statistics:");
        println!("  P50: {:?}", p50);
        println!("  P95: {:?}", p95);
        println!("  P99: {:?}", p99);
        
        // Assert performance criteria
        assert!(p50 < Duration::from_millis(100), "P50 response time too high");
        assert!(p95 < Duration::from_millis(200), "P95 response time too high");
        assert!(p99 < Duration::from_millis(500), "P99 response time too high");
    }
}
```

## Test Data Management

### Test Data Factory
```rust
// Test data factory for consistent test data creation
pub struct TestDataFactory {
    database_pool: PgPool,
    tenant_counter: AtomicU64,
    user_counter: AtomicU64,
}

impl TestDataFactory {
    pub fn new(database_pool: PgPool) -> Self {
        Self {
            database_pool,
            tenant_counter: AtomicU64::new(1),
            user_counter: AtomicU64::new(1),
        }
    }
    
    pub async fn create_tenant(&self, overrides: Option<TenantOverrides>) -> Tenant {
        let counter = self.tenant_counter.fetch_add(1, Ordering::Relaxed);
        
        let tenant = Tenant {
            id: format!("test-tenant-{}", counter),
            name: overrides.as_ref()
                .and_then(|o| o.name.clone())
                .unwrap_or_else(|| format!("Test Tenant {}", counter)),
            admin_email: overrides.as_ref()
                .and_then(|o| o.admin_email.clone())
                .unwrap_or_else(|| format!("admin{}@test.com", counter)),
            subscription_tier: overrides.as_ref()
                .and_then(|o| o.subscription_tier.clone())
                .unwrap_or(SubscriptionTier::Professional),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Insert into database
        sqlx::query!(
            "INSERT INTO tenants (id, name, admin_email, subscription_tier, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6)",
            tenant.id,
            tenant.name,
            tenant.admin_email,
            tenant.subscription_tier.to_string(),
            tenant.created_at,
            tenant.updated_at
        )
        .execute(&self.database_pool)
        .await
        .unwrap();
        
        tenant
    }
    
    pub async fn create_user(&self, tenant_id: &str, overrides: Option<UserOverrides>) -> User {
        let counter = self.user_counter.fetch_add(1, Ordering::Relaxed);
        
        let user = User {
            id: format!("test-user-{}", counter),
            email: overrides.as_ref()
                .and_then(|o| o.email.clone())
                .unwrap_or_else(|| format!("user{}@test.com", counter)),
            password_hash: bcrypt::hash("password123", bcrypt::DEFAULT_COST).unwrap(),
            tenant_id: tenant_id.to_string(),
            roles: overrides.as_ref()
                .and_then(|o| o.roles.clone())
                .unwrap_or_else(|| vec!["user".to_string()]),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        
        // Insert into database
        sqlx::query!(
            "INSERT INTO users (id, email, password_hash, tenant_id, roles, created_at, updated_at) 
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
            user.id,
            user.email,
            user.password_hash,
            user.tenant_id,
            &user.roles,
            user.created_at,
            user.updated_at
        )
        .execute(&self.database_pool)
        .await
        .unwrap();
        
        user
    }
    
    pub async fn cleanup(&self) {
        // Clean up test data
        sqlx::query!("DELETE FROM users WHERE id LIKE 'test-user-%'")
            .execute(&self.database_pool)
            .await
            .unwrap();
            
        sqlx::query!("DELETE FROM tenants WHERE id LIKE 'test-tenant-%'")
            .execute(&self.database_pool)
            .await
            .unwrap();
    }
}

#[derive(Default)]
pub struct TenantOverrides {
    pub name: Option<String>,
    pub admin_email: Option<String>,
    pub subscription_tier: Option<SubscriptionTier>,
}

#[derive(Default)]
pub struct UserOverrides {
    pub email: Option<String>,
    pub roles: Option<Vec<String>>,
}
```

## Continuous Integration Testing

### GitHub Actions Workflow
```yaml
# .github/workflows/test.yml
name: Test Suite

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: adx_core_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      redis:
        image: redis:6
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 6379:6379
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        components: rustfmt, clippy
    
    - name: Cache dependencies
      uses: actions/cache@v3
      with:
        path: |
          ~/.cargo/registry
          ~/.cargo/git
          target
        key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    
    - name: Run database migrations
      run: |
        cargo install sqlx-cli
        sqlx migrate run
      env:
        DATABASE_URL: postgres://postgres:postgres@localhost:5432/adx_core_test
    
    - name: Run unit tests
      run: cargo test --workspace --lib
      env:
        DATABASE_URL: postgres://postgres:postgres@localhost:5432/adx_core_test
        REDIS_URL: redis://localhost:6379
    
    - name: Run clippy
      run: cargo clippy --workspace -- -D warnings
    
    - name: Check formatting
      run: cargo fmt --all -- --check

  workflow-tests:
    runs-on: ubuntu-latest
    
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: adx_core_test
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
        ports:
          - 5432:5432
      
      temporal:
        image: temporalio/auto-setup:latest
        env:
          - DB=postgresql
          - DB_PORT=5432
          - POSTGRES_USER=postgres
          - POSTGRES_PWD=postgres
          - POSTGRES_SEEDS=postgres
        ports:
          - 7233:7233
          - 8088:8088
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Wait for Temporal
      run: |
        timeout 60 bash -c 'until curl -f http://localhost:8088; do sleep 2; done'
    
    - name: Run workflow tests
      run: cargo test --workspace --test workflow_tests
      env:
        DATABASE_URL: postgres://postgres:postgres@localhost:5432/adx_core_test
        TEMPORAL_SERVER_URL: localhost:7233

  integration-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Set up Docker Compose
      run: |
        docker-compose -f docker-compose.test.yml up -d
        sleep 30  # Wait for services to start
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Run integration tests
      run: cargo test --workspace --test integration_tests
      env:
        TEST_MODE: integration
    
    - name: Tear down
      run: docker-compose -f docker-compose.test.yml down

  frontend-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Run unit tests
      run: npm run test:unit
    
    - name: Run integration tests
      run: npm run test:integration
    
    - name: Build all micro-frontends
      run: npm run build:all

  e2e-tests:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Setup Node.js
      uses: actions/setup-node@v3
      with:
        node-version: '18'
        cache: 'npm'
    
    - name: Install dependencies
      run: npm ci
    
    - name: Install Playwright
      run: npx playwright install
    
    - name: Start test environment
      run: |
        docker-compose -f docker-compose.test.yml up -d
        npm run dev:all &
        sleep 60  # Wait for services to start
    
    - name: Run E2E tests
      run: npm run test:e2e
    
    - name: Upload test results
      uses: actions/upload-artifact@v3
      if: failure()
      with:
        name: playwright-report
        path: playwright-report/
```

## Development Guidelines

### Testing Best Practices
1. **Test Pyramid**: Focus on unit tests, fewer integration tests, minimal E2E tests
2. **Temporal Testing**: Use Temporal's testing framework for workflow reliability
3. **Multi-Tenant Testing**: Test tenant isolation at all levels
4. **Performance Testing**: Include load testing for critical workflows
5. **Cross-Platform Testing**: Ensure consistency across web, desktop, and mobile
6. **Test Data Management**: Use factories for consistent test data
7. **Continuous Testing**: Run tests in CI/CD pipeline
8. **Test Documentation**: Document test scenarios and expected behaviors

### Testing Checklist
- [ ] Unit tests for all business logic
- [ ] Integration tests for cross-service operations
- [ ] Workflow tests for all Temporal workflows
- [ ] API tests for all endpoints
- [ ] Frontend component tests
- [ ] Cross-micro-frontend integration tests
- [ ] Multi-tenant isolation tests
- [ ] Performance and load tests
- [ ] Security and authorization tests
- [ ] E2E user journey tests
- [ ] Cross-platform compatibility tests
- [ ] Error handling and edge case tests

This comprehensive testing strategy ensures reliability, performance, and maintainability of the ADX CORE platform across all layers and components.