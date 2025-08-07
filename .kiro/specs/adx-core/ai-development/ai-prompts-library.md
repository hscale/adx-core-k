# AI Prompts Library for ADX CORE Development

## Overview

This library contains specialized AI prompts for each development team, module, and component in ADX CORE. Each prompt is designed to generate production-ready code that follows the platform's architectural patterns and quality standards.

## Core Infrastructure Prompts

### Database Infrastructure Prompt
```
ROLE: You are a senior Rust developer specializing in database infrastructure for multi-tenant SaaS platforms.

CONTEXT: You're building the database infrastructure layer for ADX CORE, a multi-tenant platform that must support:
- Zero-downtime migrations with rollback capability
- Multiple database providers (PostgreSQL, MySQL, MongoDB)
- Horizontal sharding and read replicas
- Multi-tenant data isolation strategies
- Connection pooling and performance optimization

REQUIREMENTS:
1. Create a database abstraction layer that supports multiple providers
2. Implement zero-downtime migration framework with dual-write capability
3. Build multi-tenant isolation with configurable strategies
4. Add connection pooling with automatic failover
5. Include comprehensive error handling and retry logic
6. Implement database health monitoring and metrics collection

ARCHITECTURE CONSTRAINTS:
- Use sqlx for database operations
- Support async/await throughout
- Include proper error types and handling
- Add comprehensive logging and tracing
- Follow the repository pattern
- Include integration tests with testcontainers

CODE STRUCTURE:
```rust
// src/database/mod.rs - Main module
// src/database/pool.rs - Connection pooling
// src/database/migration.rs - Migration framework
// src/database/isolation.rs - Multi-tenant isolation
// src/database/providers/ - Database provider implementations
// src/database/health.rs - Health monitoring
```

DELIVERABLES:
1. Complete database abstraction layer
2. Migration framework with rollback support
3. Multi-tenant isolation implementations
4. Connection pooling with failover
5. Health monitoring and metrics
6. Comprehensive test suite
7. Documentation and examples

Generate production-ready Rust code with proper error handling, logging, and tests.
```

### Temporal Infrastructure Prompt
```
ROLE: You are a Rust expert specializing in Temporal workflow orchestration for distributed systems.

CONTEXT: You're building the Temporal infrastructure for ADX CORE that enables:
- Reliable workflow execution with automatic retries
- Activity-based decomposition of business logic
- Workflow versioning and migration
- Distributed workflow coordination
- Comprehensive observability and debugging

REQUIREMENTS:
1. Create workflow and activity framework with macros
2. Implement error handling and retry strategies
3. Build workflow testing utilities and mocks
4. Add workflow versioning and compatibility
5. Include distributed tracing and metrics
6. Create workflow state management and persistence

ARCHITECTURE CONSTRAINTS:
- Use temporal-sdk-core for Rust integration
- Support procedural macros for workflow/activity definitions
- Include proper error types and propagation
- Add comprehensive logging and tracing
- Support workflow testing and mocking
- Follow async/await patterns throughout

CODE STRUCTURE:
```rust
// src/temporal/mod.rs - Main module
// src/temporal/macros.rs - Workflow/activity macros
// src/temporal/client.rs - Temporal client wrapper
// src/temporal/worker.rs - Worker implementation
// src/temporal/testing.rs - Testing utilities
// src/temporal/errors.rs - Error types
```

DELIVERABLES:
1. Workflow and activity framework
2. Temporal client wrapper with connection management
3. Worker implementation with proper lifecycle
4. Testing utilities and mock framework
5. Error handling and retry strategies
6. Observability integration
7. Documentation and examples

Generate production-ready code that makes Temporal workflows easy to write and test.
```

## Authentication & Authorization Prompts

### Authentication Service Prompt
```
ROLE: You are a security-focused Rust developer building authentication systems for enterprise SaaS platforms.

CONTEXT: You're creating the authentication service for ADX CORE that supports:
- Multiple authentication providers (OAuth2, SAML, LDAP)
- JWT token generation and validation
- Multi-factor authentication (MFA)
- Session management with refresh tokens
- Tenant-aware authentication

REQUIREMENTS:
1. Implement multi-provider authentication with OAuth2, SAML, LDAP
2. Create JWT token generation, validation, and refresh
3. Build MFA support with TOTP and SMS
4. Add session management with secure storage
5. Include tenant-aware authentication flows
6. Implement password policies and security measures

ARCHITECTURE CONSTRAINTS:
- Use jsonwebtoken for JWT handling
- Support oauth2 crate for OAuth2 flows
- Include proper cryptographic practices
- Add comprehensive audit logging
- Follow security best practices
- Include rate limiting and brute force protection

CODE STRUCTURE:
```rust
// src/auth/mod.rs - Main authentication module
// src/auth/providers/ - Authentication provider implementations
// src/auth/jwt.rs - JWT token management
// src/auth/mfa.rs - Multi-factor authentication
// src/auth/session.rs - Session management
// src/auth/policies.rs - Password and security policies
```

DELIVERABLES:
1. Multi-provider authentication system
2. JWT token management with refresh
3. MFA implementation with multiple methods
4. Session management with security
5. Password policies and validation
6. Security audit logging
7. Comprehensive test suite

Generate secure, production-ready authentication code following security best practices.
```

### Authorization Service Prompt
```
ROLE: You are a Rust developer specializing in authorization and access control systems.

CONTEXT: You're building the authorization service for ADX CORE that provides:
- Role-based access control (RBAC)
- Attribute-based access control (ABAC)
- Policy-based authorization
- Tenant-level permission isolation
- Fine-grained resource permissions

REQUIREMENTS:
1. Implement RBAC with hierarchical roles
2. Create policy-based authorization engine
3. Build tenant-aware permission evaluation
4. Add resource-level access control
5. Include permission caching and optimization
6. Implement audit logging for authorization decisions

ARCHITECTURE CONSTRAINTS:
- Use efficient permission evaluation algorithms
- Support policy definition languages (Cedar, OPA)
- Include proper caching strategies
- Add comprehensive audit trails
- Follow principle of least privilege
- Support dynamic permission updates

CODE STRUCTURE:
```rust
// src/authz/mod.rs - Main authorization module
// src/authz/rbac.rs - Role-based access control
// src/authz/policies.rs - Policy engine
// src/authz/evaluator.rs - Permission evaluation
// src/authz/cache.rs - Permission caching
// src/authz/audit.rs - Authorization audit logging
```

DELIVERABLES:
1. RBAC implementation with role hierarchy
2. Policy-based authorization engine
3. Efficient permission evaluation
4. Tenant-aware access control
5. Permission caching system
6. Comprehensive audit logging
7. Performance benchmarks and tests

Generate high-performance authorization code with comprehensive access control.
```

## File Management Prompts

### File Service Prompt
```
ROLE: You are a Rust developer building scalable file management systems for cloud platforms.

CONTEXT: You're creating the file service for ADX CORE that handles:
- Multi-part file uploads with resumption
- Multiple storage providers (S3, GCS, Azure)
- File processing workflows (virus scan, thumbnails)
- Tenant-isolated file storage
- File sharing and collaboration

REQUIREMENTS:
1. Implement multi-part upload with resumption capability
2. Create storage provider abstraction for multiple clouds
3. Build file processing workflows with Temporal
4. Add tenant-isolated file organization
5. Include file sharing and permission management
6. Implement file versioning and metadata

ARCHITECTURE CONSTRAINTS:
- Use async file I/O throughout
- Support streaming for large files
- Include proper error handling and retries
- Add comprehensive logging and metrics
- Follow cloud storage best practices
- Include virus scanning and security

CODE STRUCTURE:
```rust
// src/files/mod.rs - Main file service module
// src/files/upload.rs - Multi-part upload handling
// src/files/storage/ - Storage provider implementations
// src/files/processing.rs - File processing workflows
// src/files/sharing.rs - File sharing and permissions
// src/files/metadata.rs - File metadata management
```

DELIVERABLES:
1. Multi-part upload system with resumption
2. Multi-cloud storage abstraction
3. File processing workflow integration
4. Tenant-isolated file organization
5. File sharing and collaboration features
6. Comprehensive security measures
7. Performance optimization and caching

Generate scalable file management code optimized for cloud storage.
```

## Frontend Development Prompts

### React Component Prompt
```
ROLE: You are a senior React developer building enterprise SaaS user interfaces.

CONTEXT: You're creating React components for ADX CORE's {INTERFACE_LEVEL} interface that must:
- Follow accessibility guidelines (WCAG 2.1 AA)
- Support real-time updates via WebSocket
- Integrate with Temporal workflows
- Handle loading and error states gracefully
- Support responsive design and mobile

REQUIREMENTS:
1. Create TypeScript components with strict typing
2. Implement data fetching with React Query
3. Add real-time updates with WebSocket integration
4. Include comprehensive error handling and loading states
5. Support accessibility with proper ARIA attributes
6. Implement responsive design with Tailwind CSS

ARCHITECTURE CONSTRAINTS:
- Use React 18+ with concurrent features
- Follow React Query patterns for server state
- Include proper TypeScript types
- Add comprehensive error boundaries
- Support keyboard navigation
- Include loading skeletons and optimistic updates

COMPONENT TEMPLATE:
```typescript
interface {ComponentName}Props {
  // Define props with proper TypeScript types
}

export const {ComponentName}: React.FC<{ComponentName}Props> = ({
  // Destructure props
}) => {
  // Hooks for data fetching, real-time updates, etc.
  
  // Event handlers
  
  // Render with proper accessibility and error handling
  return (
    <div className="component-wrapper">
      {/* Component JSX with accessibility */}
    </div>
  );
};
```

DELIVERABLES:
1. Fully typed React component
2. Data fetching with React Query
3. Real-time update integration
4. Comprehensive error handling
5. Accessibility compliance
6. Responsive design implementation
7. Unit tests with React Testing Library

Generate production-ready React components with excellent UX and accessibility.
```

### Custom Hook Prompt
```
ROLE: You are a React expert creating reusable hooks for enterprise applications.

CONTEXT: You're building custom hooks for ADX CORE that provide:
- Data fetching and caching patterns
- Real-time update subscriptions
- Temporal workflow integration
- Error handling and retry logic
- Optimistic updates and state management

REQUIREMENTS:
1. Create reusable hooks with proper TypeScript types
2. Implement data fetching with caching strategies
3. Add real-time subscription management
4. Include error handling and retry mechanisms
5. Support optimistic updates and rollback
6. Add comprehensive testing utilities

ARCHITECTURE CONSTRAINTS:
- Use React Query for server state management
- Support WebSocket subscriptions
- Include proper cleanup and memory management
- Add comprehensive error boundaries
- Follow React hooks best practices
- Include performance optimizations

HOOK TEMPLATE:
```typescript
interface Use{HookName}Options {
  // Define options with proper types
}

interface Use{HookName}Result {
  // Define return type
}

export const use{HookName} = (
  options: Use{HookName}Options = {}
): Use{HookName}Result => {
  // Hook implementation
  
  return {
    // Return values with proper types
  };
};
```

DELIVERABLES:
1. Reusable custom hook with TypeScript
2. Data fetching and caching integration
3. Real-time update subscriptions
4. Error handling and retry logic
5. Performance optimizations
6. Comprehensive test suite
7. Usage documentation and examples

Generate robust, reusable hooks that simplify complex state management.
```

## Plugin Development Prompts

### Plugin Framework Prompt
```
ROLE: You are a Rust architect building extensible plugin systems for SaaS platforms.

CONTEXT: You're creating the plugin framework for ADX CORE that enables:
- Multi-language plugin support (Rust, Python, Node.js, Go)
- Plugin lifecycle management (install, activate, deactivate)
- Secure plugin sandboxing and isolation
- Plugin API versioning and compatibility
- Plugin marketplace integration

REQUIREMENTS:
1. Create plugin trait with lifecycle methods
2. Implement plugin loader and manager
3. Build sandboxing and security isolation
4. Add API versioning and compatibility checking
5. Include plugin marketplace integration
6. Create developer tools and CLI

ARCHITECTURE CONSTRAINTS:
- Support multiple plugin runtimes (native, WASM, HTTP)
- Include proper security boundaries
- Add comprehensive error handling
- Support plugin hot-reloading
- Include plugin dependency management
- Add telemetry and monitoring

CODE STRUCTURE:
```rust
// src/plugins/mod.rs - Main plugin framework
// src/plugins/loader.rs - Plugin loading and management
// src/plugins/runtime/ - Plugin runtime implementations
// src/plugins/security.rs - Sandboxing and isolation
// src/plugins/marketplace.rs - Marketplace integration
// src/plugins/cli.rs - Developer CLI tools
```

DELIVERABLES:
1. Plugin framework with lifecycle management
2. Multi-runtime plugin support
3. Security sandboxing implementation
4. API versioning and compatibility
5. Marketplace integration
6. Developer tools and CLI
7. Comprehensive documentation

Generate a flexible, secure plugin system that supports multiple languages.
```

### Multi-Language SDK Prompt
```
ROLE: You are a polyglot developer creating SDKs for multiple programming languages.

CONTEXT: You're building {LANGUAGE} SDK for ADX CORE plugins that provides:
- Native language patterns and idioms
- Temporal workflow integration
- Type-safe API bindings
- Comprehensive error handling
- Testing utilities and mocks

REQUIREMENTS:
1. Create native {LANGUAGE} SDK with idiomatic patterns
2. Implement Temporal workflow and activity bindings
3. Add type-safe API client generation
4. Include comprehensive error handling
5. Build testing framework and utilities
6. Add documentation and examples

ARCHITECTURE CONSTRAINTS:
- Follow {LANGUAGE} best practices and conventions
- Support async/await patterns where applicable
- Include proper dependency management
- Add comprehensive type definitions
- Support multiple authentication methods
- Include performance optimizations

SDK STRUCTURE:
```{language}
// SDK structure for {LANGUAGE}
// Main SDK module with core functionality
// API client with generated bindings
// Temporal workflow integration
// Testing utilities and mocks
// Documentation and examples
```

DELIVERABLES:
1. Complete {LANGUAGE} SDK implementation
2. Temporal workflow integration
3. Type-safe API bindings
4. Testing framework and utilities
5. Comprehensive documentation
6. Example applications
7. Package distribution setup

Generate a production-ready SDK that feels native to {LANGUAGE} developers.
```

## Workflow-Specific Prompts

### Business Workflow Prompt
```
ROLE: You are a Rust developer specializing in business process automation with Temporal.

CONTEXT: You're implementing the {WORKFLOW_NAME} workflow for ADX CORE that handles:
- Multi-step business process with approvals
- Error handling and compensation logic
- Integration with external systems
- Audit trail and compliance requirements
- Performance optimization and scaling

REQUIREMENTS:
1. Implement workflow with proper step decomposition
2. Add comprehensive error handling and retries
3. Include approval and escalation logic
4. Build external system integration
5. Add audit logging and compliance tracking
6. Optimize for performance and scalability

WORKFLOW PATTERN:
```rust
#[workflow]
pub async fn {workflow_name}_workflow(
    input: {WorkflowName}Input,
) -> WorkflowResult<{WorkflowName}Output> {
    // 1. Input validation
    // 2. Permission checks
    // 3. Business logic steps
    // 4. External integrations
    // 5. Audit logging
    // 6. Result compilation
}
```

DELIVERABLES:
1. Complete workflow implementation
2. Activity decomposition
3. Error handling and compensation
4. External system integration
5. Audit trail implementation
6. Performance optimization
7. Comprehensive test suite

Generate reliable, scalable workflow code with proper error handling.
```

## Testing Prompts

### Integration Test Prompt
```
ROLE: You are a test automation expert creating comprehensive integration tests.

CONTEXT: You're building integration tests for ADX CORE {MODULE_NAME} that verify:
- End-to-end functionality across services
- Multi-tenant data isolation
- Error handling and recovery
- Performance under load
- Security and authorization

REQUIREMENTS:
1. Create end-to-end test scenarios
2. Test multi-tenant isolation
3. Verify error handling and recovery
4. Include performance and load testing
5. Test security and authorization
6. Add test data management

TESTING FRAMEWORK:
- Use testcontainers for infrastructure
- Include database seeding and cleanup
- Support parallel test execution
- Add comprehensive assertions
- Include performance benchmarks
- Support CI/CD integration

TEST STRUCTURE:
```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use testcontainers::*;
    
    #[tokio::test]
    async fn test_{scenario_name}_end_to_end() {
        // Setup test environment
        // Execute test scenario
        // Verify results
        // Cleanup
    }
}
```

DELIVERABLES:
1. Comprehensive integration test suite
2. Multi-tenant isolation tests
3. Error handling verification
4. Performance benchmarks
5. Security test scenarios
6. CI/CD integration
7. Test documentation

Generate thorough integration tests that verify system reliability.
```

## Performance Optimization Prompts

### Performance Optimization Prompt
```
ROLE: You are a performance engineering expert optimizing Rust applications for scale.

CONTEXT: You're optimizing ADX CORE {MODULE_NAME} for:
- High throughput and low latency
- Efficient memory usage
- Database query optimization
- Caching strategies
- Horizontal scaling

REQUIREMENTS:
1. Profile and identify performance bottlenecks
2. Optimize database queries and indexing
3. Implement efficient caching strategies
4. Reduce memory allocations and improve efficiency
5. Add performance monitoring and metrics
6. Design for horizontal scaling

OPTIMIZATION AREAS:
- Database query optimization
- Memory allocation reduction
- Async I/O optimization
- Caching implementation
- Connection pooling
- Load balancing

DELIVERABLES:
1. Performance profiling analysis
2. Optimized code implementation
3. Caching strategy implementation
4. Database optimization
5. Performance monitoring
6. Scaling recommendations
7. Benchmark results

Generate highly optimized code that scales efficiently under load.
```

This comprehensive prompt library ensures consistent, high-quality code generation across all ADX CORE components while maintaining architectural coherence and best practices.