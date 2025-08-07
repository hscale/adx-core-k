# Team 1: Core Infrastructure - AI Development Package

## Team Mission
Build the foundational infrastructure that all other teams depend on. You are the critical path - other teams cannot start until your deliverables are ready.

## Core AI Rules for Infrastructure Development

### Rule 1: Database-First Design
```
ALWAYS: Start with database schema and migrations
REASON: Data structure drives API design and business logic
PATTERN: Schema → Repository → Service → API
```

### Rule 2: Multi-Tenant by Default
```
EVERY database table MUST include tenant_id
EVERY query MUST filter by tenant_id
EVERY API MUST validate tenant access
NO exceptions - tenant isolation is non-negotiable
```

### Rule 3: Temporal-First Architecture
```
COMPLEX operations = Temporal workflows
ATOMIC operations = Temporal activities
SIMPLE CRUD = Direct repository calls
RULE: If it takes >3 steps, make it a workflow
```

### Rule 4: Observability Built-In
```
EVERY function MUST have tracing
EVERY error MUST be logged with context
EVERY metric MUST be collected
PATTERN: trace_span!("function_name", tenant_id = %tenant_id)
```

## Your Specific Deliverables

### 1. Database Infrastructure
```rust
// YOU MUST DELIVER: Complete database abstraction
pub trait DatabaseProvider: Send + Sync {
    async fn execute_query(&self, query: &str) -> Result<QueryResult, DatabaseError>;
    async fn begin_transaction(&self) -> Result<Transaction, DatabaseError>;
    async fn migrate(&self, migration: Migration) -> Result<(), MigrationError>;
}

// REQUIRED IMPLEMENTATIONS:
- PostgresProvider
- MySQLProvider  
- ConnectionPool with failover
- Zero-downtime migrations
- Multi-tenant isolation strategies
```

### 2. Temporal Infrastructure
```rust
// YOU MUST DELIVER: Workflow framework
#[workflow]
pub async fn example_workflow(input: Input) -> WorkflowResult<Output> {
    // Your framework must make this pattern work
}

// REQUIRED COMPONENTS:
- Workflow/Activity macros
- Error handling patterns
- Testing utilities
- Worker management
- Client connection handling
```

### 3. API Gateway
```rust
// YOU MUST DELIVER: Request routing and middleware
pub struct ApiGateway {
    // Must handle: routing, auth, rate limiting, CORS
}

// REQUIRED FEATURES:
- Multi-tenant request routing
- Authentication middleware
- Rate limiting per tenant
- Health check endpoints
- Request/response logging
```

## AI Development Prompts

### Database Infrastructure Prompt
```
ROLE: Senior Rust database architect building multi-tenant SaaS infrastructure

TASK: Create production-ready database infrastructure for ADX CORE

REQUIREMENTS:
- Multi-provider support (PostgreSQL, MySQL)
- Zero-downtime migrations with rollback
- Multi-tenant isolation (database/schema/row-level)
- Connection pooling with automatic failover
- Comprehensive error handling and retries

CONSTRAINTS:
- Use sqlx for database operations
- Support async/await throughout
- Include proper error types
- Add tracing and metrics
- Follow repository pattern

DELIVERABLES:
1. DatabaseProvider trait with implementations
2. Migration framework with rollback support
3. Multi-tenant isolation strategies
4. Connection pool with health monitoring
5. Comprehensive test suite with testcontainers

CODE STRUCTURE:
```rust
// src/database/mod.rs
pub mod providers;
pub mod pool;
pub mod migration;
pub mod isolation;
pub mod health;

pub use providers::*;
pub use pool::*;
pub use migration::*;
```

Generate complete, production-ready database infrastructure code.
```

### Temporal Infrastructure Prompt
```
ROLE: Temporal workflow expert building distributed system orchestration

TASK: Create Temporal infrastructure framework for ADX CORE

REQUIREMENTS:
- Workflow and activity macros for easy development
- Error handling with automatic retries
- Testing utilities for workflow development
- Worker lifecycle management
- Client connection handling

CONSTRAINTS:
- Use temporal-sdk-core
- Support procedural macros
- Include comprehensive error handling
- Add distributed tracing
- Support workflow versioning

DELIVERABLES:
1. Workflow/Activity procedural macros
2. Temporal client wrapper with connection management
3. Worker implementation with proper lifecycle
4. Testing framework for workflows
5. Error handling and retry strategies

CODE STRUCTURE:
```rust
// src/temporal/mod.rs
pub mod macros;
pub mod client;
pub mod worker;
pub mod testing;
pub mod errors;

// Usage example:
#[workflow]
pub async fn my_workflow(input: Input) -> WorkflowResult<Output> {
    let result = my_activity(input.data).await?;
    Ok(Output { result })
}
```

Generate framework that makes Temporal workflows simple to write and test.
```

### API Gateway Prompt
```
ROLE: API infrastructure expert building high-performance gateways

TASK: Create API gateway for ADX CORE multi-tenant platform

REQUIREMENTS:
- Request routing with tenant isolation
- Authentication and authorization middleware
- Rate limiting per tenant and user
- CORS handling and security headers
- Health monitoring and metrics

CONSTRAINTS:
- Use axum for HTTP handling
- Support middleware composition
- Include proper error handling
- Add comprehensive logging
- Support graceful shutdown

DELIVERABLES:
1. API gateway with routing and middleware
2. Authentication middleware integration
3. Rate limiting with tenant awareness
4. Health check and monitoring endpoints
5. Security middleware (CORS, headers)

CODE STRUCTURE:
```rust
// src/gateway/mod.rs
pub mod router;
pub mod middleware;
pub mod health;
pub mod security;

// Usage example:
let app = Router::new()
    .layer(AuthenticationLayer::new())
    .layer(RateLimitLayer::new())
    .layer(SecurityLayer::new());
```

Generate high-performance API gateway with comprehensive middleware.
```

## Success Criteria

### Database Infrastructure ✅
- [ ] Multi-provider abstraction working
- [ ] Zero-downtime migrations tested
- [ ] Multi-tenant isolation enforced
- [ ] Connection pooling with failover
- [ ] Performance benchmarks met (>1000 QPS)

### Temporal Infrastructure ✅
- [ ] Workflow macros generate correct code
- [ ] Activities execute with proper error handling
- [ ] Testing framework enables easy workflow testing
- [ ] Worker handles lifecycle correctly
- [ ] Distributed tracing works end-to-end

### API Gateway ✅
- [ ] Requests route correctly by tenant
- [ ] Authentication middleware works
- [ ] Rate limiting enforces tenant limits
- [ ] Health checks report accurate status
- [ ] Performance meets SLA (<50ms routing)

## Integration Points

### What You Provide to Other Teams
```yaml
database_infrastructure:
  provides_to: [team_2, team_3, team_4, team_5, team_9]
  interface: DatabaseProvider trait + connection pools
  
temporal_infrastructure:
  provides_to: [team_3, team_4, team_5, team_8, team_9]
  interface: Workflow/Activity macros + testing utilities
  
api_gateway:
  provides_to: [team_2, team_3, team_4, team_5, team_8, team_9]
  interface: Router + middleware framework
  
observability:
  provides_to: [all_teams]
  interface: Tracing, metrics, and logging utilities
```

### Dependencies
- **None** - You are the foundation team
- **Timeline**: Must complete by end of Sprint 1 (Week 2)
- **Blockers**: Your delays block all other teams

## Quality Standards

### Code Quality
```rust
// MANDATORY: Every function must have tracing
#[tracing::instrument(skip(self), fields(tenant_id = %tenant_id))]
pub async fn my_function(&self, tenant_id: TenantId) -> Result<T, Error> {
    // Implementation
}

// MANDATORY: Comprehensive error handling
#[derive(Debug, thiserror::Error)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query failed: {0}")]
    QueryFailed(String),
    // More specific errors
}

// MANDATORY: Multi-tenant isolation
pub async fn get_user(&self, tenant_id: TenantId, user_id: Uuid) -> Result<User, Error> {
    sqlx::query_as!(
        User,
        "SELECT * FROM users WHERE tenant_id = $1 AND id = $2",
        tenant_id,
        user_id
    )
    .fetch_one(&self.pool)
    .await
    .map_err(DatabaseError::from)
}
```

### Performance Requirements
- Database queries: <50ms (95th percentile)
- API routing: <10ms overhead
- Memory usage: <256MB per service
- Connection pool: Handle 1000+ concurrent connections

### Testing Requirements
```rust
#[cfg(test)]
mod tests {
    use testcontainers::*;
    
    #[tokio::test]
    async fn test_multi_tenant_isolation() {
        // Setup test containers
        let docker = clients::Cli::default();
        let postgres = docker.run(images::postgres::Postgres::default());
        
        // Test tenant isolation
        // Verify no cross-tenant data access
    }
}
```

## Timeline
- **Week 1**: Database and Temporal infrastructure
- **Week 2**: API gateway and observability
- **End of Week 2**: All deliverables ready for other teams

You are the foundation - build it solid! 🏗️