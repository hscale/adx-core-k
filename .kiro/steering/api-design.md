# ADX CORE API Design Guidelines

## Core Principles

ADX CORE APIs follow a Temporal-first approach where complex operations are implemented as workflows, while maintaining RESTful conventions for simple operations. All APIs support multi-tenancy, comprehensive authentication, and provide both synchronous and asynchronous operation modes.

## API Architecture Patterns

### Dual-Mode API Design
```rust
// API Gateway routing pattern
pub enum ApiOperation {
    Direct(DirectOperation),     // Simple CRUD operations
    Workflow(WorkflowOperation), // Complex multi-step operations
}

impl ApiGateway {
    pub async fn handle_request(&self, request: ApiRequest) -> Result<ApiResponse, ApiError> {
        let operation = self.classify_operation(&request)?;
        
        match operation {
            ApiOperation::Direct(op) => {
                // Route directly to backend service
                self.handle_direct_operation(op).await
            }
            ApiOperation::Workflow(op) => {
                // Initiate Temporal workflow
                self.handle_workflow_operation(op).await
            }
        }
    }
    
    fn classify_operation(&self, request: &ApiRequest) -> Result<ApiOperation, ApiError> {
        match (request.method.as_str(), request.path.as_str()) {
            // Direct operations (simple CRUD)
            ("GET", path) if path.starts_with("/api/v1/") => {
                Ok(ApiOperation::Direct(DirectOperation::Read))
            }
            ("POST", "/api/v1/users") => {
                Ok(ApiOperation::Direct(DirectOperation::Create))
            }
            
            // Workflow operations (complex processes)
            ("POST", "/api/v1/workflows/tenant-switch") => {
                Ok(ApiOperation::Workflow(WorkflowOperation::TenantSwitch))
            }
            ("POST", "/api/v1/workflows/file-upload") => {
                Ok(ApiOperation::Workflow(WorkflowOperation::FileUpload))
            }
            ("POST", "/api/v1/workflows/user-onboarding") => {
                Ok(ApiOperation::Workflow(WorkflowOperation::UserOnboarding))
            }
            
            _ => Err(ApiError::UnsupportedOperation)
        }
    }
}
```

### Workflow API Response Pattern
```rust
// Standardized workflow response format
#[derive(Debug, Serialize, Deserialize)]
pub enum WorkflowApiResponse<T> {
    Synchronous {
        data: T,
        execution_time_ms: u64,
        workflow_id: String,
    },
    Asynchronous {
        operation_id: String,
        status_url: String,
        stream_url: Option<String>,
        estimated_duration_seconds: Option<u64>,
    },
}

// Workflow status response
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowStatusResponse {
    pub operation_id: String,
    pub status: WorkflowStatus,
    pub progress: Option<WorkflowProgress>,
    pub result: Option<serde_json::Value>,
    pub error: Option<WorkflowError>,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WorkflowProgress {
    pub current_step: String,
    pub total_steps: u32,
    pub completed_steps: u32,
    pub percentage: f32,
    pub message: Option<String>,
}
```

## RESTful API Conventions

### Resource Naming and Structure
```
# Standard resource patterns
GET    /api/v1/tenants                    # List tenants
GET    /api/v1/tenants/{id}               # Get tenant
POST   /api/v1/tenants                    # Create tenant (simple)
PUT    /api/v1/tenants/{id}               # Update tenant
DELETE /api/v1/tenants/{id}               # Delete tenant

# Nested resource patterns
GET    /api/v1/tenants/{id}/users         # List tenant users
GET    /api/v1/tenants/{id}/users/{uid}   # Get tenant user
POST   /api/v1/tenants/{id}/users         # Create tenant user

# Workflow operation patterns
POST   /api/v1/workflows/create-tenant    # Complex tenant creation
POST   /api/v1/workflows/migrate-tenant   # Tenant migration
POST   /api/v1/workflows/bulk-user-import # Bulk operations

# Workflow status patterns
GET    /api/v1/workflows/{operation_id}/status    # Get workflow status
GET    /api/v1/workflows/{operation_id}/stream    # Stream workflow progress
POST   /api/v1/workflows/{operation_id}/cancel    # Cancel workflow
```

### HTTP Status Code Standards
```rust
// Standard HTTP status codes for different scenarios
pub enum ApiStatusCode {
    // Success responses
    Ok = 200,                    // Successful GET, PUT
    Created = 201,               // Successful POST
    Accepted = 202,              // Workflow initiated (async)
    NoContent = 204,             // Successful DELETE
    
    // Client error responses
    BadRequest = 400,            // Invalid request format
    Unauthorized = 401,          // Authentication required
    Forbidden = 403,             // Insufficient permissions
    NotFound = 404,              // Resource not found
    MethodNotAllowed = 405,      // HTTP method not supported
    Conflict = 409,              // Resource conflict
    UnprocessableEntity = 422,   // Validation errors
    TooManyRequests = 429,       // Rate limit exceeded
    
    // Server error responses
    InternalServerError = 500,   // Unexpected server error
    BadGateway = 502,            // Upstream service error
    ServiceUnavailable = 503,    // Service temporarily unavailable
    GatewayTimeout = 504,        // Upstream service timeout
}
```

## Multi-Tenant API Design

### Tenant Context Injection
```rust
// Tenant context middleware
pub async fn tenant_context_middleware(
    req: Request<Body>,
    next: Next<Body>,
) -> Result<Response<Body>, StatusCode> {
    // Extract tenant ID from various sources
    let tenant_id = extract_tenant_id(&req)?;
    
    // Validate tenant and load context
    let tenant_context = match load_tenant_context(&tenant_id).await {
        Ok(context) => context,
        Err(TenantError::NotFound) => return Err(StatusCode::NOT_FOUND),
        Err(TenantError::Suspended) => return Err(StatusCode::FORBIDDEN),
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
    };
    
    // Inject tenant context into request
    req.extensions_mut().insert(tenant_context);
    
    Ok(next.run(req).await?)
}

fn extract_tenant_id(req: &Request<Body>) -> Result<String, StatusCode> {
    // Priority order for tenant ID extraction:
    // 1. X-Tenant-ID header
    if let Some(header_value) = req.headers().get("X-Tenant-ID") {
        return Ok(header_value.to_str().unwrap().to_string());
    }
    
    // 2. Subdomain (tenant.adxcore.com)
    if let Some(host) = req.headers().get("Host") {
        let host_str = host.to_str().unwrap();
        if let Some(subdomain) = extract_subdomain(host_str) {
            return Ok(subdomain);
        }
    }
    
    // 3. Path prefix (/tenant/{id}/...)
    if let Some(path_tenant) = extract_tenant_from_path(req.uri().path()) {
        return Ok(path_tenant);
    }
    
    // 4. JWT token tenant claim
    if let Some(auth_header) = req.headers().get("Authorization") {
        if let Ok(token) = extract_jwt_token(auth_header) {
            if let Ok(claims) = decode_jwt_claims(&token) {
                if let Some(tenant_id) = claims.tenant_id {
                    return Ok(tenant_id);
                }
            }
        }
    }
    
    Err(StatusCode::BAD_REQUEST)
}
```

### Tenant-Aware Resource Filtering
```rust
// Automatic tenant filtering for database queries
#[derive(Debug)]
pub struct TenantAwareQuery {
    base_query: String,
    tenant_id: String,
    isolation_level: TenantIsolationLevel,
}

impl TenantAwareQuery {
    pub fn new(base_query: &str, tenant_context: &TenantContext) -> Self {
        Self {
            base_query: base_query.to_string(),
            tenant_id: tenant_context.tenant_id.clone(),
            isolation_level: tenant_context.isolation_level,
        }
    }
    
    pub fn build(&self) -> Result<String, QueryError> {
        match self.isolation_level {
            TenantIsolationLevel::Schema => {
                // Use tenant-specific schema
                Ok(format!(
                    "SET search_path = tenant_{}; {}",
                    self.tenant_id,
                    self.base_query
                ))
            }
            TenantIsolationLevel::Row => {
                // Inject tenant_id filter
                self.inject_tenant_filter()
            }
            TenantIsolationLevel::Database => {
                // Query will use tenant-specific connection
                Ok(self.base_query.clone())
            }
        }
    }
    
    fn inject_tenant_filter(&self) -> Result<String, QueryError> {
        // Parse SQL and inject WHERE tenant_id = $tenant_id
        let mut parser = SqlParser::new(&self.base_query)?;
        parser.inject_where_clause(&format!("tenant_id = '{}'", self.tenant_id))?;
        Ok(parser.to_string())
    }
}
```

## Authentication and Authorization

### JWT Token Structure
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct JwtClaims {
    // Standard claims
    pub sub: String,           // User ID
    pub exp: i64,              // Expiration time
    pub iat: i64,              // Issued at
    pub iss: String,           // Issuer
    pub aud: String,           // Audience
    
    // ADX Core specific claims
    pub tenant_id: String,     // Current tenant
    pub tenant_name: String,   // Tenant display name
    pub user_email: String,    // User email
    pub user_roles: Vec<String>, // User roles in current tenant
    pub permissions: Vec<String>, // Specific permissions
    pub features: Vec<String>, // Available features
    pub quotas: UserQuotas,    // User-specific quotas
    
    // Session information
    pub session_id: String,    // Session identifier
    pub device_id: Option<String>, // Device identifier
    pub ip_address: String,    // Client IP address
    
    // Multi-tenant support
    pub available_tenants: Vec<String>, // Tenants user has access to
    pub tenant_roles: HashMap<String, Vec<String>>, // Roles per tenant
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserQuotas {
    pub api_calls_per_hour: u32,
    pub storage_gb: u32,
    pub concurrent_workflows: u32,
    pub file_upload_size_mb: u32,
}
```

### Permission-Based Authorization
```rust
// Permission checking middleware
pub async fn permission_middleware(
    required_permission: &str,
) -> impl Filter<Extract = (), Error = Rejection> + Clone {
    warp::header::optional::<String>("authorization")
        .and_then(move |auth_header: Option<String>| {
            let permission = required_permission.to_string();
            async move {
                let token = extract_bearer_token(auth_header)?;
                let claims = validate_jwt_token(&token).await?;
                
                if !has_permission(&claims, &permission) {
                    return Err(warp::reject::custom(InsufficientPermissions));
                }
                
                Ok(())
            }
        })
}

fn has_permission(claims: &JwtClaims, required_permission: &str) -> bool {
    // Check direct permissions
    if claims.permissions.contains(&required_permission.to_string()) {
        return true;
    }
    
    // Check role-based permissions
    for role in &claims.user_roles {
        if let Some(role_permissions) = get_role_permissions(role) {
            if role_permissions.contains(&required_permission.to_string()) {
                return true;
            }
        }
    }
    
    // Check wildcard permissions
    let permission_parts: Vec<&str> = required_permission.split(':').collect();
    for perm in &claims.permissions {
        if matches_wildcard_permission(perm, &permission_parts) {
            return true;
        }
    }
    
    false
}

// Permission patterns:
// "tenant:read" - Read tenant information
// "tenant:write" - Modify tenant information
// "tenant:admin" - Full tenant administration
// "user:*" - All user operations
// "file:upload" - Upload files
// "workflow:execute" - Execute workflows
// "module:install" - Install modules
```

## Rate Limiting and Quotas

### Rate Limiting Implementation
```rust
// Rate limiting middleware with tenant awareness
pub struct RateLimiter {
    redis_client: Arc<RedisClient>,
    default_limits: RateLimits,
    tenant_limits: Arc<RwLock<HashMap<String, RateLimits>>>,
}

#[derive(Debug, Clone)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub requests_per_hour: u32,
    pub requests_per_day: u32,
    pub burst_limit: u32,
}

impl RateLimiter {
    pub async fn check_rate_limit(
        &self,
        tenant_id: &str,
        user_id: &str,
        endpoint: &str,
    ) -> Result<RateLimitResult, RateLimitError> {
        let limits = self.get_tenant_limits(tenant_id).await?;
        
        // Check multiple time windows
        let minute_key = format!("rate_limit:{}:{}:{}:minute", tenant_id, user_id, endpoint);
        let hour_key = format!("rate_limit:{}:{}:{}:hour", tenant_id, user_id, endpoint);
        let day_key = format!("rate_limit:{}:{}:{}:day", tenant_id, user_id, endpoint);
        
        let minute_count = self.increment_counter(&minute_key, 60).await?;
        let hour_count = self.increment_counter(&hour_key, 3600).await?;
        let day_count = self.increment_counter(&day_key, 86400).await?;
        
        if minute_count > limits.requests_per_minute {
            return Ok(RateLimitResult::Exceeded {
                retry_after: 60,
                limit_type: "per_minute".to_string(),
            });
        }
        
        if hour_count > limits.requests_per_hour {
            return Ok(RateLimitResult::Exceeded {
                retry_after: 3600,
                limit_type: "per_hour".to_string(),
            });
        }
        
        if day_count > limits.requests_per_day {
            return Ok(RateLimitResult::Exceeded {
                retry_after: 86400,
                limit_type: "per_day".to_string(),
            });
        }
        
        Ok(RateLimitResult::Allowed {
            remaining_minute: limits.requests_per_minute - minute_count,
            remaining_hour: limits.requests_per_hour - hour_count,
            remaining_day: limits.requests_per_day - day_count,
        })
    }
}
```

## Error Handling and Responses

### Standardized Error Format
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiError {
    pub error: ErrorDetails,
    pub request_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub details: Option<serde_json::Value>,
    pub validation_errors: Option<Vec<ValidationError>>,
    pub retry_after: Option<u64>,
    pub documentation_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ValidationError {
    pub field: String,
    pub code: String,
    pub message: String,
    pub rejected_value: Option<serde_json::Value>,
}

// Example error responses:
// 400 Bad Request
{
  "error": {
    "code": "VALIDATION_FAILED",
    "message": "Request validation failed",
    "validation_errors": [
      {
        "field": "email",
        "code": "INVALID_FORMAT",
        "message": "Email address format is invalid",
        "rejected_value": "invalid-email"
      }
    ],
    "documentation_url": "https://docs.adxcore.com/api/validation"
  },
  "request_id": "req_123456789",
  "timestamp": "2024-01-15T10:30:00Z"
}

// 429 Too Many Requests
{
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded for this endpoint",
    "details": {
      "limit_type": "per_hour",
      "current_usage": 1001,
      "limit": 1000
    },
    "retry_after": 3600
  },
  "request_id": "req_123456790",
  "timestamp": "2024-01-15T10:30:00Z"
}
```

## API Versioning Strategy

### Version Management
```rust
// API version routing
pub enum ApiVersion {
    V1,
    V2,
    Beta,
}

impl ApiVersion {
    pub fn from_header(version_header: &str) -> Result<Self, ApiError> {
        match version_header {
            "application/vnd.adxcore.v1+json" => Ok(ApiVersion::V1),
            "application/vnd.adxcore.v2+json" => Ok(ApiVersion::V2),
            "application/vnd.adxcore.beta+json" => Ok(ApiVersion::Beta),
            _ => Err(ApiError::UnsupportedVersion),
        }
    }
    
    pub fn from_path(path: &str) -> Result<Self, ApiError> {
        if path.starts_with("/api/v1/") {
            Ok(ApiVersion::V1)
        } else if path.starts_with("/api/v2/") {
            Ok(ApiVersion::V2)
        } else if path.starts_with("/api/beta/") {
            Ok(ApiVersion::Beta)
        } else {
            Err(ApiError::UnsupportedVersion)
        }
    }
}

// Version-specific handlers
pub struct VersionedApiHandler {
    v1_handler: V1ApiHandler,
    v2_handler: V2ApiHandler,
    beta_handler: BetaApiHandler,
}

impl VersionedApiHandler {
    pub async fn handle_request(
        &self,
        version: ApiVersion,
        request: ApiRequest,
    ) -> Result<ApiResponse, ApiError> {
        match version {
            ApiVersion::V1 => self.v1_handler.handle(request).await,
            ApiVersion::V2 => self.v2_handler.handle(request).await,
            ApiVersion::Beta => self.beta_handler.handle(request).await,
        }
    }
}
```

## OpenAPI Documentation

### Automated Documentation Generation
```rust
// OpenAPI schema generation
use utoipa::{OpenApi, ToSchema};

#[derive(OpenApi)]
#[openapi(
    paths(
        create_tenant,
        get_tenant,
        list_tenants,
        update_tenant,
        delete_tenant,
        create_tenant_workflow,
        get_workflow_status,
    ),
    components(
        schemas(
            Tenant,
            CreateTenantRequest,
            UpdateTenantRequest,
            WorkflowApiResponse,
            WorkflowStatusResponse,
            ApiError,
        )
    ),
    tags(
        (name = "tenants", description = "Tenant management operations"),
        (name = "workflows", description = "Workflow operations"),
    ),
    info(
        title = "ADX Core API",
        version = "2.0.0",
        description = "ADX Core Temporal-first multi-tenant SaaS platform API",
        contact(
            name = "ADX Core Support",
            email = "support@adxcore.com",
            url = "https://adxcore.com/support"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "https://api.adxcore.com", description = "Production server"),
        (url = "https://staging-api.adxcore.com", description = "Staging server"),
        (url = "http://localhost:8080", description = "Development server")
    )
)]
pub struct ApiDoc;

// Generate OpenAPI spec
pub fn generate_openapi_spec() -> String {
    ApiDoc::openapi().to_pretty_json().unwrap()
}
```

## Testing Strategies

### API Testing Framework
```rust
// Integration test framework for APIs
#[cfg(test)]
mod api_tests {
    use super::*;
    use axum_test::TestServer;
    
    #[tokio::test]
    async fn test_tenant_crud_operations() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        // Test create tenant
        let create_response = server
            .post("/api/v1/tenants")
            .json(&CreateTenantRequest {
                name: "Test Tenant".to_string(),
                admin_email: "admin@test.com".to_string(),
                subscription_tier: SubscriptionTier::Professional,
            })
            .await;
        
        create_response.assert_status_ok();
        let tenant: Tenant = create_response.json();
        
        // Test get tenant
        let get_response = server
            .get(&format!("/api/v1/tenants/{}", tenant.id))
            .await;
        
        get_response.assert_status_ok();
        let retrieved_tenant: Tenant = get_response.json();
        assert_eq!(retrieved_tenant.id, tenant.id);
        
        // Test update tenant
        let update_response = server
            .put(&format!("/api/v1/tenants/{}", tenant.id))
            .json(&UpdateTenantRequest {
                name: Some("Updated Tenant".to_string()),
                ..Default::default()
            })
            .await;
        
        update_response.assert_status_ok();
        
        // Test delete tenant
        let delete_response = server
            .delete(&format!("/api/v1/tenants/{}", tenant.id))
            .await;
        
        delete_response.assert_status_no_content();
    }
    
    #[tokio::test]
    async fn test_workflow_operations() {
        let app = create_test_app().await;
        let server = TestServer::new(app).unwrap();
        
        // Test workflow initiation
        let workflow_response = server
            .post("/api/v1/workflows/create-tenant")
            .json(&CreateTenantWorkflowRequest {
                tenant_name: "Workflow Tenant".to_string(),
                admin_email: "admin@workflow.com".to_string(),
                subscription_tier: SubscriptionTier::Enterprise,
            })
            .await;
        
        workflow_response.assert_status_accepted();
        let workflow_result: WorkflowApiResponse<()> = workflow_response.json();
        
        match workflow_result {
            WorkflowApiResponse::Asynchronous { operation_id, .. } => {
                // Test workflow status
                let status_response = server
                    .get(&format!("/api/v1/workflows/{}/status", operation_id))
                    .await;
                
                status_response.assert_status_ok();
                let status: WorkflowStatusResponse = status_response.json();
                assert!(matches!(status.status, WorkflowStatus::Running | WorkflowStatus::Pending));
            }
            _ => panic!("Expected asynchronous workflow response"),
        }
    }
}
```

## Development Guidelines

### API Development Best Practices
1. **Temporal-First Design**: Use workflows for complex operations, direct endpoints for simple CRUD
2. **Multi-Tenant Awareness**: Always validate and inject tenant context
3. **Comprehensive Error Handling**: Provide detailed, actionable error messages
4. **Rate Limiting**: Implement appropriate rate limits for all endpoints
5. **Versioning**: Support multiple API versions with clear migration paths
6. **Documentation**: Maintain up-to-date OpenAPI specifications
7. **Testing**: Write comprehensive integration tests for all endpoints
8. **Security**: Implement proper authentication, authorization, and input validation
9. **Monitoring**: Add metrics and logging for all API operations
10. **Performance**: Optimize for low latency and high throughput

### API Security Checklist
- [ ] Input validation and sanitization
- [ ] SQL injection prevention
- [ ] XSS protection
- [ ] CSRF protection
- [ ] Rate limiting implementation
- [ ] Authentication required for protected endpoints
- [ ] Authorization checks for all operations
- [ ] Tenant isolation enforcement
- [ ] Audit logging for sensitive operations
- [ ] HTTPS enforcement
- [ ] Security headers implementation
- [ ] API key management
- [ ] JWT token validation and refresh

This API design approach ensures scalable, secure, and maintainable APIs that leverage Temporal workflows for reliability while providing excellent developer experience.