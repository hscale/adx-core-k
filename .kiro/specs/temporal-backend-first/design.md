# Temporal Backend First Design

## Overview

This design transforms ADX CORE's backend microservices into a workflow-driven architecture where Temporal workflows become the primary orchestration mechanism for all multi-step operations. Instead of services calling each other directly, complex operations are orchestrated through Temporal workflows, providing reliability, observability, and maintainability at scale.

## Architecture

### High-Level Architecture

```mermaid
graph TB
    subgraph "Frontend Layer"
        Shell[Shell App :3000]
        AuthMF[Auth Micro-App :3001]
        TenantMF[Tenant Micro-App :3002]
        FileMF[File Micro-App :3003]
        UserMF[User Micro-App :3004]
        WorkflowMF[Workflow Micro-App :3005]
    end

    subgraph "BFF Layer (Optional Optimization)"
        AuthBFF[Auth BFF :4001<br/>Workflow Client]
        TenantBFF[Tenant BFF :4002<br/>Workflow Client]
        FileBFF[File BFF :4003<br/>Workflow Client]
        UserBFF[User BFF :4004<br/>Workflow Client]
        WorkflowBFF[Workflow BFF :4005<br/>Workflow Client]
    end

    subgraph "API Gateway Layer"
        APIGateway[API Gateway :8080<br/>Workflow Endpoints + Direct Endpoints]
    end

    subgraph "Temporal Infrastructure"
        TemporalServer[Temporal Server :7233]
        TemporalUI[Temporal UI :8088]
        TemporalDB[(Temporal Database)]
        
        subgraph "Workflow Workers"
            AuthWorker[Auth Workflow Worker]
            UserWorker[User Workflow Worker]
            FileWorker[File Workflow Worker]
            TenantWorker[Tenant Workflow Worker]
            CrossServiceWorker[Cross-Service Workflow Worker]
        end
    end

    subgraph "Backend Microservices"
        AuthService[Auth Service :8081<br/>Activities + Direct Endpoints]
        UserService[User Service :8082<br/>Activities + Direct Endpoints]
        FileService[File Service :8083<br/>Activities + Direct Endpoints]
        WorkflowService[Workflow Service :8084<br/>Workflow Management]
        TenantService[Tenant Service :8085<br/>Activities + Direct Endpoints]
    end

    subgraph "Data Layer"
        PostgresDB[(PostgreSQL)]
        RedisCache[(Redis Cache)]
        FileStorage[(File Storage)]
    end

    Shell --> AuthBFF
    AuthMF --> AuthBFF
    TenantMF --> TenantBFF
    FileMF --> FileBFF
    UserMF --> UserBFF
    WorkflowMF --> WorkflowBFF

    Shell -.-> APIGateway
    AuthMF -.-> APIGateway
    TenantMF -.-> APIGateway
    FileMF -.-> APIGateway
    UserMF -.-> APIGateway
    WorkflowMF -.-> APIGateway

    AuthBFF --> APIGateway
    TenantBFF --> APIGateway
    FileBFF --> APIGateway
    UserBFF --> APIGateway
    WorkflowBFF --> APIGateway

    AuthBFF --> TemporalServer
    TenantBFF --> TemporalServer
    FileBFF --> TemporalServer
    UserBFF --> TemporalServer
    WorkflowBFF --> TemporalServer

    APIGateway --> TemporalServer
    APIGateway --> AuthService
    APIGateway --> UserService
    APIGateway --> FileService
    APIGateway --> TenantService

    AuthWorker --> TemporalServer
    UserWorker --> TemporalServer
    FileWorker --> TemporalServer
    TenantWorker --> TemporalServer
    CrossServiceWorker --> TemporalServer

    AuthWorker --> AuthService
    UserWorker --> UserService
    FileWorker --> FileService
    TenantWorker --> TenantService
    CrossServiceWorker --> AuthService
    CrossServiceWorker --> UserService
    CrossServiceWorker --> FileService
    CrossServiceWorker --> TenantService

    AuthService --> PostgresDB
    UserService --> PostgresDB
    FileService --> PostgresDB
    TenantService --> PostgresDB
    FileService --> FileStorage

    AuthService --> RedisCache
    UserService --> RedisCache
    FileService --> RedisCache
    TenantService --> RedisCache
```

### Service Architecture Pattern

Each backend service implements a dual pattern with both direct endpoints and Temporal activities:

```mermaid
graph TB
    subgraph "Backend Service Architecture"
        HTTPServer[HTTP Server]
        
        subgraph "Direct Operations"
            DirectHandlers[Direct REST Handlers]
            SimpleOperations[Simple CRUD Operations]
        end
        
        subgraph "Workflow Integration"
            ActivityImplementations[Temporal Activities]
            WorkflowClient[Temporal Client]
            CompensationLogic[Compensation Activities]
        end
        
        subgraph "Shared Components"
            BusinessLogic[Business Logic Layer]
            DataAccess[Data Access Layer]
            Auth[Authentication/Authorization]
            Logging[Structured Logging]
            Metrics[Prometheus Metrics]
        end
    end

    HTTPServer --> DirectHandlers
    HTTPServer --> ActivityImplementations
    DirectHandlers --> BusinessLogic
    ActivityImplementations --> BusinessLogic
    ActivityImplementations --> CompensationLogic
    BusinessLogic --> DataAccess
    BusinessLogic --> Auth
```

## Components and Interfaces

### 1. BFF Integration Strategy

The Temporal-first backend supports both direct frontend integration and optional BFF optimization:

**Direct Integration (Phase 1):**
- Frontend micro-apps call API Gateway workflow endpoints directly
- Simple operations use direct service endpoints
- Complex operations trigger Temporal workflows
- Real-time updates via WebSocket/SSE from API Gateway

**BFF Integration (Phase 2 - Optional Optimization):**
- BFF services act as Temporal workflow clients
- Aggregate multiple workflow calls for complex UI needs
- Provide frontend-optimized data shapes
- Cache workflow results for performance

**BFF as Workflow Client Pattern:**
```rust
// BFF service implementing workflow client pattern
pub struct AuthBFF {
    temporal_client: Arc<TemporalClient>,
    api_gateway_client: Arc<ApiGatewayClient>,
    redis: Arc<RedisClient>,
}

impl AuthBFF {
    // Direct workflow initiation from BFF
    pub async fn initiate_user_login_flow(
        &self,
        request: LoginRequest,
    ) -> Result<LoginFlowResult, BFFError> {
        // Start workflow directly from BFF
        let workflow_id = format!("login-{}-{}", request.user_id, Uuid::new_v4());
        
        let workflow_handle = self.temporal_client
            .start_workflow(
                user_login_workflow,
                workflow_id.clone(),
                "auth-task-queue",
                request.clone(),
            )
            .await?;
        
        // For quick operations, wait for completion
        if request.is_simple_login {
            let result = workflow_handle.get_result().await?;
            return Ok(LoginFlowResult::Completed(result));
        }
        
        // For complex operations (MFA, etc.), return operation ID
        Ok(LoginFlowResult::InProgress {
            operation_id: workflow_id,
            status_url: format!("/api/auth/login-status/{}", workflow_id),
            websocket_url: format!("/api/auth/login-stream/{}", workflow_id),
        })
    }
    
    // Aggregate multiple workflows for complex UI needs
    pub async fn get_user_dashboard_data(
        &self,
        user_id: &str,
    ) -> Result<DashboardData, BFFError> {
        // Check cache first
        if let Some(cached) = self.get_cached_dashboard_data(user_id).await? {
            return Ok(cached);
        }
        
        // Start multiple workflows in parallel
        let user_profile_future = self.temporal_client.start_workflow(
            get_user_profile_workflow,
            format!("profile-{}", user_id),
            "user-task-queue",
            GetUserProfileRequest { user_id: user_id.to_string() },
        );
        
        let tenant_data_future = self.temporal_client.start_workflow(
            get_tenant_data_workflow,
            format!("tenant-data-{}", user_id),
            "tenant-task-queue",
            GetTenantDataRequest { user_id: user_id.to_string() },
        );
        
        let recent_files_future = self.temporal_client.start_workflow(
            get_recent_files_workflow,
            format!("files-{}", user_id),
            "file-task-queue",
            GetRecentFilesRequest { user_id: user_id.to_string() },
        );
        
        // Wait for all workflows to complete
        let (user_profile, tenant_data, recent_files) = tokio::try_join!(
            user_profile_future.and_then(|h| h.get_result()),
            tenant_data_future.and_then(|h| h.get_result()),
            recent_files_future.and_then(|h| h.get_result()),
        )?;
        
        let dashboard_data = DashboardData {
            user_profile,
            tenant_data,
            recent_files,
            last_updated: Utc::now(),
        };
        
        // Cache the result
        self.cache_dashboard_data(user_id, &dashboard_data).await?;
        
        Ok(dashboard_data)
    }
    
    // Fallback to API Gateway for simple operations
    pub async fn get_user_preferences(
        &self,
        user_id: &str,
    ) -> Result<UserPreferences, BFFError> {
        // Simple operation - call API Gateway directly
        self.api_gateway_client
            .get(&format!("/api/users/{}/preferences", user_id))
            .await
            .map_err(BFFError::ApiGatewayError)
    }
}
```

**BFF Workflow Integration Patterns:**

```typescript
// Node.js BFF with Temporal integration
import { Client, WorkflowHandle } from '@temporalio/client';

export class TenantBFF {
  private temporalClient: Client;
  private apiGatewayClient: ApiGatewayClient;
  
  constructor() {
    this.temporalClient = new Client({
      connection: { address: process.env.TEMPORAL_SERVER_URL }
    });
  }
  
  // Complex tenant switching via workflow
  async switchTenant(userId: string, toTenantId: string): Promise<TenantSwitchResponse> {
    const workflowId = `tenant-switch-${userId}-${Date.now()}`;
    
    const handle: WorkflowHandle = await this.temporalClient.workflow.start(
      'tenantSwitchWorkflow',
      {
        workflowId,
        taskQueue: 'tenant-task-queue',
        args: [userId, toTenantId],
      }
    );
    
    // For tenant switching, we typically want to wait for completion
    const result = await handle.result();
    
    return {
      success: true,
      newTenantId: toTenantId,
      sessionId: result.sessionId,
      tenantData: result.tenantData,
    };
  }
  
  // Aggregate tenant-related data from multiple workflows
  async getTenantOverview(userId: string): Promise<TenantOverview> {
    const [tenantList, currentTenant, permissions] = await Promise.all([
      this.startWorkflow('getUserTenantsWorkflow', { userId }),
      this.startWorkflow('getCurrentTenantWorkflow', { userId }),
      this.startWorkflow('getUserPermissionsWorkflow', { userId }),
    ]);
    
    return {
      availableTenants: await tenantList.result(),
      currentTenant: await currentTenant.result(),
      permissions: await permissions.result(),
    };
  }
  
  private async startWorkflow(workflowType: string, args: any): Promise<WorkflowHandle> {
    return this.temporalClient.workflow.start(workflowType, {
      workflowId: `${workflowType}-${Date.now()}-${Math.random()}`,
      taskQueue: 'tenant-task-queue',
      args: [args],
    });
  }
}
```

### 2. Enhanced API Gateway

The API Gateway becomes the orchestration entry point, determining whether to handle requests directly or initiate workflows:

**Workflow Endpoint Pattern:**
```rust
// API Gateway workflow endpoints
#[derive(Serialize, Deserialize)]
pub struct WorkflowRequest {
    pub operation: String,
    pub parameters: serde_json::Value,
    pub user_context: UserContext,
    pub tenant_context: TenantContext,
}

#[derive(Serialize, Deserialize)]
pub struct WorkflowResponse {
    pub operation_id: String,
    pub status: WorkflowStatus,
    pub result: Option<serde_json::Value>,
    pub progress: Option<WorkflowProgress>,
}

pub enum WorkflowStatus {
    Started,
    Running,
    Completed,
    Failed,
    Cancelled,
}

// API Gateway implementation
impl ApiGateway {
    pub async fn handle_workflow_request(
        &self,
        request: WorkflowRequest,
    ) -> Result<WorkflowResponse, ApiError> {
        match request.operation.as_str() {
            "tenant_switch" => self.start_tenant_switch_workflow(request).await,
            "user_onboarding" => self.start_user_onboarding_workflow(request).await,
            "file_processing" => self.start_file_processing_workflow(request).await,
            _ => Err(ApiError::UnsupportedOperation),
        }
    }

    pub async fn get_workflow_status(
        &self,
        operation_id: &str,
    ) -> Result<WorkflowResponse, ApiError> {
        let execution = self.temporal_client
            .get_workflow_execution_info(operation_id)
            .await?;
        
        Ok(WorkflowResponse {
            operation_id: operation_id.to_string(),
            status: execution.status.into(),
            result: execution.result,
            progress: execution.progress,
        })
    }
}
```

### 2. Service Activity Implementations

Each microservice implements Temporal activities alongside direct endpoints:

**Auth Service Activities:**
```rust
#[async_trait]
pub trait AuthActivities {
    async fn validate_credentials(&self, request: ValidateCredentialsRequest) -> Result<AuthResult, ActivityError>;
    async fn create_session(&self, user_id: &str, tenant_id: &str) -> Result<SessionInfo, ActivityError>;
    async fn invalidate_session(&self, session_id: &str) -> Result<(), ActivityError>;
    async fn check_permissions(&self, user_id: &str, resource: &str, action: &str) -> Result<bool, ActivityError>;
    
    // Compensation activities
    async fn rollback_session_creation(&self, session_id: &str) -> Result<(), ActivityError>;
    async fn restore_previous_session(&self, user_id: &str, previous_session_id: &str) -> Result<(), ActivityError>;
}

pub struct AuthService {
    db: Arc<DatabasePool>,
    redis: Arc<RedisClient>,
    temporal_client: Arc<TemporalClient>,
}

#[async_trait]
impl AuthActivities for AuthService {
    async fn validate_credentials(&self, request: ValidateCredentialsRequest) -> Result<AuthResult, ActivityError> {
        // Activity implementation with proper error handling
        let user = self.db.get_user_by_email(&request.email).await
            .map_err(|e| ActivityError::DatabaseError(e.to_string()))?;
        
        let is_valid = bcrypt::verify(&request.password, &user.password_hash)
            .map_err(|e| ActivityError::AuthenticationError(e.to_string()))?;
        
        if is_valid {
            Ok(AuthResult {
                user_id: user.id,
                tenant_ids: user.tenant_ids,
                permissions: user.permissions,
            })
        } else {
            Err(ActivityError::InvalidCredentials)
        }
    }
    
    async fn create_session(&self, user_id: &str, tenant_id: &str) -> Result<SessionInfo, ActivityError> {
        let session = SessionInfo {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            tenant_id: tenant_id.to_string(),
            created_at: Utc::now(),
            expires_at: Utc::now() + Duration::hours(24),
        };
        
        // Store in Redis with expiration
        self.redis.set_ex(
            &format!("session:{}", session.id),
            &serde_json::to_string(&session)?,
            24 * 3600,
        ).await?;
        
        Ok(session)
    }
}
```

**User Service Activities:**
```rust
#[async_trait]
pub trait UserActivities {
    async fn create_user(&self, request: CreateUserRequest) -> Result<User, ActivityError>;
    async fn update_user_tenant(&self, user_id: &str, tenant_id: &str) -> Result<User, ActivityError>;
    async fn get_user_profile(&self, user_id: &str) -> Result<UserProfile, ActivityError>;
    async fn update_user_preferences(&self, user_id: &str, preferences: UserPreferences) -> Result<(), ActivityError>;
    
    // Compensation activities
    async fn delete_user(&self, user_id: &str) -> Result<(), ActivityError>;
    async fn restore_user_tenant(&self, user_id: &str, previous_tenant_id: &str) -> Result<(), ActivityError>;
}
```

**File Service Activities:**
```rust
#[async_trait]
pub trait FileActivities {
    async fn validate_file_upload(&self, request: FileUploadRequest) -> Result<ValidationResult, ActivityError>;
    async fn create_file_record(&self, metadata: FileMetadata) -> Result<FileRecord, ActivityError>;
    async fn upload_to_storage(&self, file_data: Vec<u8>, path: &str) -> Result<StorageResult, ActivityError>;
    async fn update_file_metadata(&self, file_id: &str, metadata: FileMetadata) -> Result<(), ActivityError>;
    async fn set_file_permissions(&self, file_id: &str, permissions: FilePermissions) -> Result<(), ActivityError>;
    
    // Compensation activities
    async fn delete_file_record(&self, file_id: &str) -> Result<(), ActivityError>;
    async fn remove_from_storage(&self, storage_path: &str) -> Result<(), ActivityError>;
}
```

### 3. Cross-Service Workflow Definitions

Complex business operations are implemented as workflows that coordinate multiple services:

**Tenant Switch Workflow:**
```rust
#[workflow]
pub async fn tenant_switch_workflow(
    user_id: String,
    from_tenant_id: String,
    to_tenant_id: String,
) -> Result<TenantSwitchResult, WorkflowError> {
    // Step 1: Validate user has access to target tenant
    let access_check = call_activity(
        AuthActivities::check_permissions,
        CheckPermissionsRequest {
            user_id: user_id.clone(),
            resource: format!("tenant:{}", to_tenant_id),
            action: "access".to_string(),
        },
    ).await?;
    
    if !access_check {
        return Err(WorkflowError::PermissionDenied);
    }
    
    // Step 2: Get current session info for rollback
    let current_session = call_activity(
        AuthActivities::get_current_session,
        user_id.clone(),
    ).await?;
    
    // Step 3: Update user's current tenant
    let updated_user = call_activity(
        UserActivities::update_user_tenant,
        UpdateUserTenantRequest {
            user_id: user_id.clone(),
            tenant_id: to_tenant_id.clone(),
        },
    ).await.map_err(|e| {
        // If this fails, no compensation needed yet
        WorkflowError::UserUpdateFailed(e.to_string())
    })?;
    
    // Step 4: Create new session for target tenant
    let new_session = call_activity(
        AuthActivities::create_session,
        CreateSessionRequest {
            user_id: user_id.clone(),
            tenant_id: to_tenant_id.clone(),
        },
    ).await.map_err(|e| {
        // Compensation: restore previous tenant
        spawn_compensation_activity(
            UserActivities::restore_user_tenant,
            RestoreUserTenantRequest {
                user_id: user_id.clone(),
                previous_tenant_id: from_tenant_id.clone(),
            },
        );
        WorkflowError::SessionCreationFailed(e.to_string())
    })?;
    
    // Step 5: Load tenant-specific data
    let tenant_data = call_activity(
        TenantActivities::load_tenant_data,
        LoadTenantDataRequest {
            user_id: user_id.clone(),
            tenant_id: to_tenant_id.clone(),
        },
    ).await.map_err(|e| {
        // Compensation: rollback session and user tenant
        spawn_compensation_activity(
            AuthActivities::rollback_session_creation,
            new_session.id.clone(),
        );
        spawn_compensation_activity(
            UserActivities::restore_user_tenant,
            RestoreUserTenantRequest {
                user_id: user_id.clone(),
                previous_tenant_id: from_tenant_id.clone(),
            },
        );
        WorkflowError::TenantDataLoadFailed(e.to_string())
    })?;
    
    // Step 6: Invalidate old session
    call_activity(
        AuthActivities::invalidate_session,
        current_session.id,
    ).await?;
    
    Ok(TenantSwitchResult {
        success: true,
        new_session_id: new_session.id,
        tenant_data,
        user_context: updated_user,
    })
}
```

**File Upload Workflow:**
```rust
#[workflow]
pub async fn file_upload_workflow(
    upload_request: FileUploadRequest,
    user_context: UserContext,
) -> Result<FileUploadResult, WorkflowError> {
    // Step 1: Validate file and user permissions
    let validation = call_activity(
        FileActivities::validate_file_upload,
        ValidationRequest {
            file_metadata: upload_request.metadata.clone(),
            user_id: user_context.user_id.clone(),
            tenant_id: user_context.tenant_id.clone(),
        },
    ).await?;
    
    if !validation.is_valid {
        return Err(WorkflowError::ValidationFailed(validation.errors));
    }
    
    // Step 2: Create file record in database
    let file_record = call_activity(
        FileActivities::create_file_record,
        CreateFileRecordRequest {
            metadata: upload_request.metadata.clone(),
            owner_id: user_context.user_id.clone(),
            tenant_id: user_context.tenant_id.clone(),
        },
    ).await?;
    
    // Step 3: Upload file to storage
    let storage_result = call_activity(
        FileActivities::upload_to_storage,
        UploadToStorageRequest {
            file_id: file_record.id.clone(),
            file_data: upload_request.file_data,
            storage_path: generate_storage_path(&file_record),
        },
    ).await.map_err(|e| {
        // Compensation: delete file record
        spawn_compensation_activity(
            FileActivities::delete_file_record,
            file_record.id.clone(),
        );
        WorkflowError::StorageUploadFailed(e.to_string())
    })?;
    
    // Step 4: Update file metadata with storage information
    call_activity(
        FileActivities::update_file_metadata,
        UpdateFileMetadataRequest {
            file_id: file_record.id.clone(),
            storage_path: storage_result.path.clone(),
            checksum: storage_result.checksum.clone(),
            size: storage_result.size,
        },
    ).await.map_err(|e| {
        // Compensation: remove from storage and delete record
        spawn_compensation_activity(
            FileActivities::remove_from_storage,
            storage_result.path.clone(),
        );
        spawn_compensation_activity(
            FileActivities::delete_file_record,
            file_record.id.clone(),
        );
        WorkflowError::MetadataUpdateFailed(e.to_string())
    })?;
    
    // Step 5: Set file permissions
    call_activity(
        FileActivities::set_file_permissions,
        SetFilePermissionsRequest {
            file_id: file_record.id.clone(),
            permissions: FilePermissions {
                owner_id: user_context.user_id.clone(),
                tenant_id: user_context.tenant_id.clone(),
                access_level: AccessLevel::Private,
            },
        },
    ).await?;
    
    Ok(FileUploadResult {
        file_id: file_record.id,
        storage_path: storage_result.path,
        metadata: file_record.metadata,
        upload_time: Utc::now(),
    })
}
```

**User Onboarding Workflow:**
```rust
#[workflow]
pub async fn user_onboarding_workflow(
    onboarding_request: UserOnboardingRequest,
) -> Result<UserOnboardingResult, WorkflowError> {
    // Step 1: Create user account
    let user = call_activity(
        UserActivities::create_user,
        CreateUserRequest {
            email: onboarding_request.email.clone(),
            name: onboarding_request.name.clone(),
            tenant_id: onboarding_request.tenant_id.clone(),
        },
    ).await?;
    
    // Step 2: Set up authentication
    let auth_setup = call_activity(
        AuthActivities::setup_user_authentication,
        SetupAuthRequest {
            user_id: user.id.clone(),
            password_hash: onboarding_request.password_hash.clone(),
            mfa_enabled: onboarding_request.mfa_enabled,
        },
    ).await.map_err(|e| {
        // Compensation: delete user
        spawn_compensation_activity(
            UserActivities::delete_user,
            user.id.clone(),
        );
        WorkflowError::AuthSetupFailed(e.to_string())
    })?;
    
    // Step 3: Assign tenant permissions
    call_activity(
        TenantActivities::assign_user_to_tenant,
        AssignUserToTenantRequest {
            user_id: user.id.clone(),
            tenant_id: onboarding_request.tenant_id.clone(),
            role: onboarding_request.initial_role.clone(),
        },
    ).await.map_err(|e| {
        // Compensation: cleanup auth and delete user
        spawn_compensation_activity(
            AuthActivities::cleanup_user_authentication,
            user.id.clone(),
        );
        spawn_compensation_activity(
            UserActivities::delete_user,
            user.id.clone(),
        );
        WorkflowError::TenantAssignmentFailed(e.to_string())
    })?;
    
    // Step 4: Create initial workspace
    let workspace = call_activity(
        WorkspaceActivities::create_user_workspace,
        CreateWorkspaceRequest {
            user_id: user.id.clone(),
            tenant_id: onboarding_request.tenant_id.clone(),
            workspace_name: format!("{}'s Workspace", user.name),
        },
    ).await?;
    
    // Step 5: Send welcome email
    call_activity(
        NotificationActivities::send_welcome_email,
        WelcomeEmailRequest {
            user_email: user.email.clone(),
            user_name: user.name.clone(),
            workspace_url: workspace.url.clone(),
        },
    ).await?;
    
    Ok(UserOnboardingResult {
        user_id: user.id,
        tenant_id: onboarding_request.tenant_id,
        workspace_id: workspace.id,
        auth_setup_complete: true,
    })
}
```

## Data Models

### Workflow Execution Context

```rust
#[derive(Serialize, Deserialize, Clone)]
pub struct WorkflowExecutionContext {
    pub execution_id: String,
    pub workflow_type: String,
    pub user_context: UserContext,
    pub tenant_context: TenantContext,
    pub start_time: DateTime<Utc>,
    pub status: WorkflowStatus,
    pub progress: Option<WorkflowProgress>,
    pub result: Option<serde_json::Value>,
    pub error: Option<WorkflowError>,
    pub compensation_executed: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserContext {
    pub user_id: String,
    pub email: String,
    pub current_tenant_id: String,
    pub permissions: Vec<Permission>,
    pub session_id: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct TenantContext {
    pub tenant_id: String,
    pub tenant_name: String,
    pub subscription_tier: SubscriptionTier,
    pub feature_flags: HashMap<String, bool>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub estimated_time_remaining: Option<Duration>,
    pub step_details: HashMap<String, serde_json::Value>,
}
```

### Activity Error Handling

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum ActivityError {
    DatabaseError(String),
    NetworkError(String),
    AuthenticationError(String),
    AuthorizationError(String),
    ValidationError(Vec<String>),
    ExternalServiceError(String),
    TimeoutError,
    RetryableError(String),
    NonRetryableError(String),
}

impl ActivityError {
    pub fn is_retryable(&self) -> bool {
        match self {
            ActivityError::NetworkError(_) => true,
            ActivityError::TimeoutError => true,
            ActivityError::RetryableError(_) => true,
            ActivityError::ExternalServiceError(_) => true,
            _ => false,
        }
    }
    
    pub fn retry_policy(&self) -> RetryPolicy {
        match self {
            ActivityError::NetworkError(_) => RetryPolicy::exponential_backoff(3, Duration::seconds(1)),
            ActivityError::TimeoutError => RetryPolicy::exponential_backoff(2, Duration::seconds(5)),
            ActivityError::ExternalServiceError(_) => RetryPolicy::exponential_backoff(5, Duration::seconds(2)),
            _ => RetryPolicy::no_retry(),
        }
    }
}
```

## Error Handling

### Comprehensive Error Handling Strategy

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum WorkflowError {
    ActivityFailed(String, ActivityError),
    CompensationFailed(String),
    TimeoutExceeded,
    ValidationFailed(Vec<String>),
    PermissionDenied,
    TenantIsolationViolation,
    ExternalServiceUnavailable(String),
    DataInconsistency(String),
}

impl WorkflowError {
    pub fn requires_compensation(&self) -> bool {
        match self {
            WorkflowError::ActivityFailed(_, _) => true,
            WorkflowError::DataInconsistency(_) => true,
            _ => false,
        }
    }
    
    pub fn is_user_recoverable(&self) -> bool {
        match self {
            WorkflowError::ValidationFailed(_) => true,
            WorkflowError::PermissionDenied => false,
            WorkflowError::ExternalServiceUnavailable(_) => true,
            _ => false,
        }
    }
}

// Compensation execution
pub async fn execute_compensation(
    workflow_context: &WorkflowExecutionContext,
    compensation_activities: Vec<CompensationActivity>,
) -> Result<(), CompensationError> {
    for activity in compensation_activities.into_iter().rev() {
        match activity.execute().await {
            Ok(_) => {
                log::info!("Compensation activity {} completed successfully", activity.name());
            }
            Err(e) => {
                log::error!("Compensation activity {} failed: {}", activity.name(), e);
                // Continue with other compensation activities
            }
        }
    }
    Ok(())
}
```

## Testing Strategy

### Workflow Testing Framework

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use temporal_sdk_core_test_utils::TestWorkflowEnvironment;
    
    #[tokio::test]
    async fn test_tenant_switch_workflow_success() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock activities
        let auth_activities = MockAuthActivities::new()
            .expect_check_permissions()
            .returning(|_| Ok(true))
            .expect_create_session()
            .returning(|_| Ok(SessionInfo::default()));
            
        let user_activities = MockUserActivities::new()
            .expect_update_user_tenant()
            .returning(|_| Ok(User::default()));
        
        // Execute workflow
        let result = test_env.execute_workflow(
            tenant_switch_workflow,
            TenantSwitchRequest {
                user_id: "user-123".to_string(),
                from_tenant_id: "tenant-old".to_string(),
                to_tenant_id: "tenant-new".to_string(),
            },
        ).await;
        
        assert!(result.is_ok());
        let switch_result = result.unwrap();
        assert!(switch_result.success);
    }
    
    #[tokio::test]
    async fn test_tenant_switch_workflow_permission_denied() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock permission check failure
        let auth_activities = MockAuthActivities::new()
            .expect_check_permissions()
            .returning(|_| Ok(false));
        
        let result = test_env.execute_workflow(
            tenant_switch_workflow,
            TenantSwitchRequest {
                user_id: "user-123".to_string(),
                from_tenant_id: "tenant-old".to_string(),
                to_tenant_id: "tenant-new".to_string(),
            },
        ).await;
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), WorkflowError::PermissionDenied));
    }
    
    #[tokio::test]
    async fn test_file_upload_workflow_with_compensation() {
        let test_env = TestWorkflowEnvironment::new().await;
        
        // Mock successful validation and file creation, but storage failure
        let file_activities = MockFileActivities::new()
            .expect_validate_file_upload()
            .returning(|_| Ok(ValidationResult { is_valid: true, errors: vec![] }))
            .expect_create_file_record()
            .returning(|_| Ok(FileRecord::default()))
            .expect_upload_to_storage()
            .returning(|_| Err(ActivityError::ExternalServiceError("Storage unavailable".to_string())))
            .expect_delete_file_record()
            .returning(|_| Ok(()));
        
        let result = test_env.execute_workflow(
            file_upload_workflow,
            FileUploadRequest::default(),
        ).await;
        
        assert!(result.is_err());
        // Verify compensation was executed
        file_activities.verify_delete_file_record_called();
    }
}
```

### Integration Testing

```rust
#[tokio::test]
async fn test_end_to_end_tenant_switch() {
    // Start test environment with real services
    let test_env = IntegrationTestEnvironment::new().await;
    
    // Create test user and tenants
    let user = test_env.create_test_user().await;
    let tenant_a = test_env.create_test_tenant().await;
    let tenant_b = test_env.create_test_tenant().await;
    
    // Assign user to both tenants
    test_env.assign_user_to_tenant(&user.id, &tenant_a.id).await;
    test_env.assign_user_to_tenant(&user.id, &tenant_b.id).await;
    
    // Execute tenant switch via API Gateway
    let response = test_env.api_client
        .post("/api/workflows/tenant-switch")
        .json(&json!({
            "user_id": user.id,
            "from_tenant_id": tenant_a.id,
            "to_tenant_id": tenant_b.id
        }))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let result: TenantSwitchResult = response.json().await.unwrap();
    assert!(result.success);
    
    // Verify user's current tenant was updated
    let updated_user = test_env.get_user(&user.id).await;
    assert_eq!(updated_user.current_tenant_id, tenant_b.id);
    
    // Verify new session was created
    let session = test_env.get_session(&result.new_session_id).await;
    assert_eq!(session.tenant_id, tenant_b.id);
}
```

## Deployment and Operations

### Service Configuration

```rust
#[derive(Serialize, Deserialize)]
pub struct ServiceConfig {
    pub temporal: TemporalConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub workflows: WorkflowConfig,
    pub activities: ActivityConfig,
}

#[derive(Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub task_queue: String,
    pub default_timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub worker_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ActivityConfig {
    pub default_timeout: Duration,
    pub retry_policy: RetryPolicy,
    pub circuit_breaker: CircuitBreakerConfig,
}
```

### Monitoring and Observability

```rust
// Prometheus metrics
lazy_static! {
    static ref WORKFLOW_EXECUTIONS: Counter = Counter::new(
        "workflow_executions_total",
        "Total number of workflow executions"
    ).unwrap();
    
    static ref WORKFLOW_DURATION: Histogram = Histogram::with_opts(
        HistogramOpts::new(
            "workflow_duration_seconds",
            "Workflow execution duration"
        ).buckets(vec![0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 30.0, 60.0])
    ).unwrap();
    
    static ref ACTIVITY_FAILURES: Counter = Counter::new(
        "activity_failures_total",
        "Total number of activity failures"
    ).unwrap();
    
    static ref COMPENSATION_EXECUTIONS: Counter = Counter::new(
        "compensation_executions_total",
        "Total number of compensation executions"
    ).unwrap();
}

// Structured logging
pub fn log_workflow_start(context: &WorkflowExecutionContext) {
    info!(
        workflow_id = %context.execution_id,
        workflow_type = %context.workflow_type,
        user_id = %context.user_context.user_id,
        tenant_id = %context.tenant_context.tenant_id,
        "Workflow execution started"
    );
}

pub fn log_activity_execution(
    workflow_id: &str,
    activity_name: &str,
    duration: Duration,
    result: &Result<(), ActivityError>,
) {
    match result {
        Ok(_) => {
            info!(
                workflow_id = %workflow_id,
                activity_name = %activity_name,
                duration_ms = duration.num_milliseconds(),
                "Activity completed successfully"
            );
        }
        Err(e) => {
            error!(
                workflow_id = %workflow_id,
                activity_name = %activity_name,
                duration_ms = duration.num_milliseconds(),
                error = %e,
                "Activity failed"
            );
        }
    }
}
```

### Deployment Strategy

**Phase 1: Temporal-First Backend**
```yaml
# Docker Compose for development
version: '3.8'
services:
  # Existing services...
  
  # Workflow workers
  auth-workflow-worker:
    build: ./adx-core/services/auth-service
    command: ["./auth-service", "--mode", "worker"]
    environment:
      - TEMPORAL_SERVER_URL=temporal:7233
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/adx_core
      - REDIS_URL=redis://redis:6379
    depends_on:
      - temporal
      - postgres
      - redis
  
  user-workflow-worker:
    build: ./adx-core/services/user-service
    command: ["./user-service", "--mode", "worker"]
    environment:
      - TEMPORAL_SERVER_URL=temporal:7233
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/adx_core
      - REDIS_URL=redis://redis:6379
    depends_on:
      - temporal
      - postgres
      - redis
  
  file-workflow-worker:
    build: ./adx-core/services/file-service
    command: ["./file-service", "--mode", "worker"]
    environment:
      - TEMPORAL_SERVER_URL=temporal:7233
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/adx_core
      - REDIS_URL=redis://redis:6379
    depends_on:
      - temporal
      - postgres
      - redis
  
  cross-service-workflow-worker:
    build: ./adx-core/services/workflow-service
    command: ["./workflow-service", "--mode", "worker"]
    environment:
      - TEMPORAL_SERVER_URL=temporal:7233
      - DATABASE_URL=postgresql://postgres:password@postgres:5432/adx_core
      - REDIS_URL=redis://redis:6379
    depends_on:
      - temporal
      - postgres
      - redis
```

**Phase 2: BFF Integration (Optional)**
```yaml
  # BFF services with Temporal client integration
  auth-bff:
    build: ./bff-services/auth-bff
    ports:
      - "4001:4001"
    environment:
      - TEMPORAL_SERVER_URL=temporal:7233
      - API_GATEWAY_URL=http://api-gateway:8080
      - REDIS_URL=redis://redis:6379
    depends_on:
      - temporal
      - api-gateway
      - redis
  
  tenant-bff:
    build: ./bff-services/tenant-bff
    ports:
      - "4002:4002"
    environment:
      - TEMPORAL_SERVER_URL=temporal:7233
      - API_GATEWAY_URL=http://api-gateway:8080
      - REDIS_URL=redis://redis:6379
    depends_on:
      - temporal
      - api-gateway
      - redis
  
  file-bff:
    build: ./bff-services/file-bff
    ports:
      - "4003:4003"
    environment:
      - TEMPORAL_SERVER_URL=temporal:7233
      - API_GATEWAY_URL=http://api-gateway:8080
      - REDIS_URL=redis://redis:6379
    depends_on:
      - temporal
      - api-gateway
      - redis
```

**Migration Strategy:**

1. **Phase 1**: Implement Temporal-first backend with direct frontend integration
   - Frontend micro-apps call API Gateway workflow endpoints directly
   - Establish workflow patterns and reliability
   - Validate performance and observability

2. **Phase 2**: Add BFF optimization layer
   - Implement BFF services as Temporal workflow clients
   - Migrate complex frontend integrations to use BFFs
   - Maintain direct API Gateway access for simple operations

3. **Phase 3**: Optimize and scale
   - Fine-tune BFF caching and aggregation strategies
   - Implement advanced workflow patterns
   - Scale workflow workers based on load patterns

This design establishes a comprehensive workflow-driven backend architecture where Temporal becomes the central orchestration mechanism for all complex operations, providing reliability, observability, and maintainability while maintaining the ability to handle simple operations directly through traditional REST endpoints.