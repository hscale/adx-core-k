# Team Autonomy and Vertical Slice Ownership

## Overview

ADX CORE is designed around the principle of team autonomy through vertical slice ownership. Each team owns a complete domain-aligned slice of functionality, including backend service, frontend micro-app, and optional BFF service. This approach enables independent development, deployment, and scaling while maintaining system coherence.

## Vertical Slice Architecture

### What is a Vertical Slice?

A vertical slice is a complete, end-to-end implementation of a business capability that includes:

```
┌─────────────────────────────────────────────────────────────┐
│                    Vertical Slice                          │
├─────────────────────────────────────────────────────────────┤
│ Frontend Micro-App (React + Module Federation)             │
│ ├── Components, Pages, Hooks                               │
│ ├── State Management (Zustand, React Query)               │
│ ├── Event Bus Integration                                  │
│ └── Shared Design System Usage                            │
├─────────────────────────────────────────────────────────────┤
│ Optional BFF Service (Node.js/TypeScript or Rust)         │
│ ├── Data Aggregation and Caching                          │
│ ├── Temporal Workflow Client                              │
│ ├── Performance Optimization                              │
│ └── Frontend-Specific APIs                                │
├─────────────────────────────────────────────────────────────┤
│ Backend Service (Rust + Temporal)                         │
│ ├── HTTP Server (Direct Endpoints)                        │
│ ├── Temporal Worker (Workflow Activities)                 │
│ ├── Database Layer (Repository Pattern)                   │
│ ├── Business Logic and Domain Models                      │
│ └── Cross-Service Integration (via Workflows)             │
└─────────────────────────────────────────────────────────────┘
```

### Domain Alignment

Each vertical slice aligns with a specific business domain:

| Team | Domain | Backend Service | Frontend App | BFF Service |
|------|--------|----------------|--------------|-------------|
| **Auth Team** | Authentication & Security | Auth Service (8081) | Auth App (3001) | Auth BFF (4001) |
| **Tenant Team** | Multi-Tenant Management | Tenant Service (8085) | Tenant App (3002) | Tenant BFF (4002) |
| **File Team** | Document Management | File Service (8083) | File App (3003) | File BFF (4003) |
| **User Team** | User & Profile Management | User Service (8082) | User App (3004) | User BFF (4004) |
| **Workflow Team** | Process Orchestration | Workflow Service (8084) | Workflow App (3005) | Workflow BFF (4005) |
| **Module Team** | Platform Extensions | Module Service (8086) | Module App (3006) | Module BFF (4006) |

## Team Structure and Responsibilities

### Recommended Team Composition

Each vertical slice team should include:

```
Team Lead (1)
├── Technical leadership and architecture decisions
├── Cross-team coordination and communication
├── Code review and quality assurance
└── Sprint planning and delivery management

Backend Developers (2-3)
├── Rust service development
├── Temporal workflow implementation
├── Database design and optimization
├── API design and documentation
└── Performance monitoring and optimization

Frontend Developers (2-3)
├── React micro-frontend development
├── Module Federation integration
├── Shared design system usage
├── State management and caching
└── Cross-platform compatibility (web, desktop, mobile)

Full-Stack Developer (1) - Optional
├── BFF service development
├── Frontend-backend integration
├── Performance optimization
├── End-to-end testing
└── DevOps and deployment support
```

### Team Responsibilities

#### Development Responsibilities
- **Feature Development**: Complete end-to-end feature implementation
- **Code Quality**: Maintain high code quality standards and test coverage
- **Documentation**: Keep technical and user documentation up-to-date
- **Performance**: Monitor and optimize service performance
- **Security**: Implement security best practices and handle vulnerabilities

#### Operational Responsibilities
- **Deployment**: Manage independent deployment pipelines
- **Monitoring**: Set up and maintain service monitoring and alerting
- **Incident Response**: Handle service-specific incidents and outages
- **Capacity Planning**: Monitor resource usage and plan for scaling
- **Backup and Recovery**: Ensure data protection and disaster recovery

#### Collaboration Responsibilities
- **API Contracts**: Define and maintain service interfaces
- **Event Schemas**: Manage event bus message formats
- **Shared Libraries**: Contribute to and maintain shared components
- **Cross-Team Integration**: Coordinate with other teams for complex features
- **Knowledge Sharing**: Document decisions and share learnings

## Development Workflow

### Independent Development Cycle

Each team follows an independent development cycle:

```
┌─────────────────────────────────────────────────────────────┐
│                  Team Development Cycle                    │
├─────────────────────────────────────────────────────────────┤
│ 1. Planning                                                 │
│    ├── Sprint planning with domain-specific backlog        │
│    ├── Technical design and architecture decisions         │
│    ├── API contract definition and review                  │
│    └── Cross-team dependency identification                │
├─────────────────────────────────────────────────────────────┤
│ 2. Development                                              │
│    ├── Backend service implementation                      │
│    ├── Frontend micro-app development                      │
│    ├── BFF service optimization (if needed)                │
│    ├── Integration testing within vertical slice           │
│    └── Documentation and code review                       │
├─────────────────────────────────────────────────────────────┤
│ 3. Testing                                                  │
│    ├── Unit tests for all components                       │
│    ├── Integration tests within the slice                  │
│    ├── Contract tests for external interfaces              │
│    ├── End-to-end tests for user journeys                  │
│    └── Performance and security testing                    │
├─────────────────────────────────────────────────────────────┤
│ 4. Deployment                                               │
│    ├── Independent CI/CD pipeline execution                │
│    ├── Staging environment deployment and testing          │
│    ├── Production deployment with monitoring               │
│    ├── Health checks and rollback procedures               │
│    └── Post-deployment verification                        │
├─────────────────────────────────────────────────────────────┤
│ 5. Monitoring & Maintenance                                 │
│    ├── Service health and performance monitoring           │
│    ├── Error tracking and incident response                │
│    ├── User feedback collection and analysis               │
│    ├── Technical debt management                           │
│    └── Continuous improvement planning                     │
└─────────────────────────────────────────────────────────────┘
```

### Technology Choices

Teams have flexibility in technology choices within their vertical slice:

#### Backend Service Technology Stack
```rust
// Core requirements (consistent across teams)
- Language: Rust 1.70+
- Framework: Axum for HTTP services
- Workflow Engine: Temporal.io
- Database: PostgreSQL with SQLx
- Caching: Redis

// Team-specific choices
- Additional libraries and crates
- Database schema design
- Caching strategies
- Monitoring and logging approaches
- Testing frameworks and strategies
```

#### Frontend Micro-App Technology Stack
```typescript
// Core requirements (consistent across teams)
- Framework: React 18+ with TypeScript
- Build Tool: Vite with Module Federation
- Design System: Shared @adx-core/design-system
- State Management: Zustand + React Query
- Event Bus: Shared @adx-core/event-bus

// Team-specific choices
- Additional UI libraries and components
- State management patterns
- Testing approaches (Jest, Vitest, Testing Library)
- Performance optimization strategies
- Accessibility implementations
```

#### BFF Service Technology Stack
```typescript
// Technology choice per team
Option 1: Node.js/TypeScript
- Framework: Express or Fastify
- Temporal Client: @temporalio/client
- Caching: Redis with ioredis
- Testing: Jest or Vitest

Option 2: Rust/Axum
- Framework: Axum
- Temporal Client: temporal-sdk
- Caching: Redis with redis-rs
- Testing: Built-in Rust testing
```

## Communication Patterns

### Inter-Team Communication

#### API Contracts
Teams define and maintain clear API contracts:

```yaml
# auth-service-api.yaml
openapi: 3.0.0
info:
  title: Auth Service API
  version: 2.0.0
paths:
  /api/v1/auth/validate:
    post:
      summary: Validate JWT token
      requestBody:
        required: true
        content:
          application/json:
            schema:
              type: object
              properties:
                token:
                  type: string
                  description: JWT token to validate
      responses:
        '200':
          description: Token is valid
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/TokenValidationResult'
        '401':
          description: Token is invalid or expired
```

#### Event Bus Schemas
Frontend micro-apps communicate through typed events:

```typescript
// shared-events.ts
export interface UserLoginEvent {
  type: 'user:login';
  data: {
    userId: string;
    tenantId: string;
    timestamp: string;
  };
}

export interface TenantSwitchEvent {
  type: 'tenant:switch';
  data: {
    previousTenantId: string;
    newTenantId: string;
    userId: string;
  };
}

export interface FileUploadEvent {
  type: 'file:upload:complete';
  data: {
    fileId: string;
    fileName: string;
    tenantId: string;
    uploadedBy: string;
  };
}
```

#### Temporal Workflow Integration
Cross-service operations use Temporal workflows:

```rust
// Cross-service workflow example
#[workflow]
pub async fn user_onboarding_workflow(
    request: UserOnboardingRequest,
) -> Result<UserOnboardingResult, WorkflowError> {
    // Auth Service: Create user account
    let user = call_activity(
        AuthServiceActivities::create_user,
        CreateUserRequest::from(request.clone()),
    ).await?;
    
    // User Service: Setup user profile
    let profile = call_activity(
        UserServiceActivities::create_profile,
        CreateProfileRequest {
            user_id: user.id.clone(),
            profile_data: request.profile_data,
        },
    ).await?;
    
    // File Service: Setup user storage
    let storage = call_activity(
        FileServiceActivities::setup_user_storage,
        SetupStorageRequest {
            user_id: user.id.clone(),
            tenant_id: request.tenant_id,
        },
    ).await?;
    
    Ok(UserOnboardingResult {
        user_id: user.id,
        profile_id: profile.id,
        storage_quota: storage.quota,
    })
}
```

### Coordination Mechanisms

#### Weekly Cross-Team Sync
- **Architecture Review**: Discuss system-wide changes and improvements
- **Dependency Planning**: Coordinate features requiring multiple teams
- **API Changes**: Review breaking changes and migration strategies
- **Shared Library Updates**: Plan updates to shared components
- **Performance Review**: Discuss system-wide performance metrics

#### Quarterly Planning
- **Roadmap Alignment**: Ensure team roadmaps support business objectives
- **Technical Debt**: Plan cross-team technical debt reduction
- **Infrastructure Updates**: Coordinate major infrastructure changes
- **Security Reviews**: Conduct system-wide security assessments
- **Capacity Planning**: Plan for scaling and resource allocation

## Development Environment Setup

### Team-Specific Development Environment

Each team maintains their own development environment:

```bash
# Team workspace structure
team-workspace/
├── backend-service/          # Rust backend service
│   ├── src/
│   ├── tests/
│   ├── Cargo.toml
│   └── docker-compose.dev.yml
├── frontend-app/             # React micro-frontend
│   ├── src/
│   ├── tests/
│   ├── package.json
│   └── vite.config.ts
├── bff-service/              # Optional BFF service
│   ├── src/
│   ├── tests/
│   └── package.json or Cargo.toml
├── shared/                   # Team-specific shared code
│   ├── types/
│   ├── utils/
│   └── constants/
├── docs/                     # Team documentation
│   ├── api/
│   ├── architecture/
│   └── runbooks/
└── scripts/                  # Development and deployment scripts
    ├── dev-setup.sh
    ├── test.sh
    └── deploy.sh
```

### Local Development Setup

```bash
# 1. Clone team repository
git clone https://github.com/adxcore/auth-team.git
cd auth-team

# 2. Setup development environment
./scripts/dev-setup.sh

# 3. Start local services
docker-compose -f backend-service/docker-compose.dev.yml up -d

# 4. Start backend service
cd backend-service
cargo run

# 5. Start frontend app
cd ../frontend-app
npm run dev

# 6. Start BFF service (if applicable)
cd ../bff-service
npm run dev  # or cargo run
```

### Integration Testing Environment

```bash
# Team integration testing
./scripts/integration-test.sh

# Cross-team integration testing
./scripts/cross-team-test.sh --teams auth,user,tenant

# Full system integration testing
./scripts/full-system-test.sh
```

## Deployment and Operations

### Independent Deployment Pipelines

Each team maintains independent CI/CD pipelines:

```yaml
# .github/workflows/auth-team-deploy.yml
name: Auth Team Deployment

on:
  push:
    branches: [main]
    paths: ['auth-service/**', 'auth-frontend/**', 'auth-bff/**']

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run Backend Tests
        run: cd auth-service && cargo test
      - name: Run Frontend Tests
        run: cd auth-frontend && npm test
      - name: Run Integration Tests
        run: ./scripts/integration-test.sh

  deploy-backend:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Build and Deploy Backend
        run: |
          docker build -t adxcore/auth-service:${{ github.sha }} auth-service/
          docker push adxcore/auth-service:${{ github.sha }}
          kubectl set image deployment/auth-service \
            auth-service=adxcore/auth-service:${{ github.sha }} \
            -n adx-core-production

  deploy-frontend:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Build and Deploy Frontend
        run: |
          cd auth-frontend
          npm run build
          aws s3 sync dist/ s3://adx-core-auth-frontend/
          aws cloudfront create-invalidation --distribution-id $CLOUDFRONT_ID --paths "/*"

  deploy-bff:
    needs: test
    runs-on: ubuntu-latest
    if: github.event.paths contains 'auth-bff/**'
    steps:
      - name: Build and Deploy BFF
        run: |
          docker build -t adxcore/auth-bff:${{ github.sha }} auth-bff/
          docker push adxcore/auth-bff:${{ github.sha }}
          kubectl set image deployment/auth-bff \
            auth-bff=adxcore/auth-bff:${{ github.sha }} \
            -n adx-core-production
```

### Service Monitoring and Alerting

Each team is responsible for monitoring their services:

```yaml
# monitoring/auth-team-alerts.yml
groups:
  - name: auth-team-alerts
    rules:
      - alert: AuthServiceDown
        expr: up{job="auth-service"} == 0
        for: 1m
        labels:
          severity: critical
          team: auth
        annotations:
          summary: "Auth service is down"
          description: "Auth service has been down for more than 1 minute"
          runbook: "https://docs.adxcore.com/runbooks/auth-service-down"

      - alert: AuthServiceHighErrorRate
        expr: rate(http_requests_total{job="auth-service",status=~"5.."}[5m]) > 0.1
        for: 2m
        labels:
          severity: warning
          team: auth
        annotations:
          summary: "High error rate in auth service"
          description: "Auth service error rate is {{ $value }} errors per second"

      - alert: AuthWorkflowFailures
        expr: rate(workflow_executions_total{service="auth",status="failed"}[5m]) > 0.05
        for: 5m
        labels:
          severity: warning
          team: auth
        annotations:
          summary: "High auth workflow failure rate"
          description: "Auth workflow failure rate is {{ $value }} failures per second"
```

### Incident Response

Teams handle incidents for their vertical slice:

```bash
# Incident response runbook
# 1. Identify the affected service
kubectl get pods -n adx-core-production | grep auth

# 2. Check service logs
kubectl logs -f deployment/auth-service -n adx-core-production

# 3. Check metrics and alerts
curl http://prometheus:9090/api/v1/query?query=up{job="auth-service"}

# 4. Rollback if necessary
kubectl rollout undo deployment/auth-service -n adx-core-production

# 5. Communicate status
# - Update status page
# - Notify affected teams
# - Document incident in post-mortem
```

## Quality Assurance

### Testing Strategy

Each team implements comprehensive testing:

#### Backend Service Testing
```rust
// Unit tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_user_authentication() {
        let auth_service = AuthService::new_test();
        let result = auth_service.authenticate(
            "user@example.com",
            "password123"
        ).await;
        assert!(result.is_ok());
    }
}

// Integration tests
#[tokio::test]
async fn test_auth_workflow_integration() {
    let test_env = TestWorkflowEnvironment::new().await;
    let result = test_env.execute_workflow(
        user_registration_workflow,
        UserRegistrationRequest::default(),
    ).await;
    assert!(result.is_ok());
}
```

#### Frontend Testing
```typescript
// Component tests
import { render, screen, fireEvent } from '@testing-library/react';
import { LoginForm } from '../LoginForm';

test('should handle login submission', async () => {
  render(<LoginForm />);
  
  fireEvent.change(screen.getByLabelText(/email/i), {
    target: { value: 'user@example.com' }
  });
  fireEvent.change(screen.getByLabelText(/password/i), {
    target: { value: 'password123' }
  });
  fireEvent.click(screen.getByRole('button', { name: /login/i }));
  
  expect(await screen.findByText(/welcome/i)).toBeInTheDocument();
});

// Integration tests
test('should integrate with event bus', async () => {
  const mockEventBus = createMockEventBus();
  render(
    <EventBusProvider value={mockEventBus}>
      <LoginForm />
    </EventBusProvider>
  );
  
  // Test event emission and handling
  fireEvent.click(screen.getByRole('button', { name: /login/i }));
  expect(mockEventBus.emit).toHaveBeenCalledWith('user:login', expect.any(Object));
});
```

#### End-to-End Testing
```typescript
// E2E tests with Playwright
import { test, expect } from '@playwright/test';

test('complete user authentication flow', async ({ page }) => {
  // Navigate to auth micro-frontend
  await page.goto('http://localhost:3000/auth/login');
  
  // Fill login form
  await page.fill('[data-testid="email-input"]', 'user@example.com');
  await page.fill('[data-testid="password-input"]', 'password123');
  await page.click('[data-testid="login-button"]');
  
  // Verify successful login
  await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();
  
  // Verify event propagation to other micro-frontends
  await page.goto('http://localhost:3000/dashboard');
  await expect(page.locator('[data-testid="welcome-message"]')).toContainText('Welcome');
});
```

### Code Quality Standards

#### Code Review Process
1. **Pull Request Creation**: Include description, testing notes, and breaking changes
2. **Automated Checks**: CI pipeline runs tests, linting, and security scans
3. **Peer Review**: At least one team member reviews code changes
4. **Architecture Review**: Complex changes reviewed by team lead or architect
5. **Merge Requirements**: All checks pass and reviews approved

#### Quality Metrics
- **Test Coverage**: Minimum 80% code coverage for backend and frontend
- **Code Quality**: SonarQube quality gate passing
- **Performance**: No regression in response times or resource usage
- **Security**: No high or critical security vulnerabilities
- **Documentation**: API changes documented, README updated

## Best Practices

### Team Autonomy Best Practices

#### 1. Clear Boundaries
- **Domain Ownership**: Each team owns a specific business domain
- **Interface Contracts**: Well-defined APIs and event schemas
- **Data Ownership**: Teams own their data and database schemas
- **Deployment Independence**: Teams can deploy without coordinating with others

#### 2. Shared Standards
- **Code Standards**: Consistent formatting, naming, and structure
- **API Standards**: RESTful conventions and error handling
- **Security Standards**: Authentication, authorization, and data protection
- **Monitoring Standards**: Consistent metrics, logging, and alerting

#### 3. Communication Protocols
- **Async Communication**: Prefer asynchronous communication over synchronous
- **Documentation**: Keep API docs, runbooks, and architecture decisions updated
- **Change Management**: Communicate breaking changes well in advance
- **Knowledge Sharing**: Regular tech talks and documentation reviews

#### 4. Technology Governance
- **Approved Technologies**: Use approved languages, frameworks, and libraries
- **Shared Libraries**: Contribute to and use shared components
- **Security Compliance**: Follow security guidelines and vulnerability management
- **Performance Standards**: Meet performance and scalability requirements

### Common Pitfalls to Avoid

#### 1. Over-Coupling
- **Avoid Direct Service Calls**: Use Temporal workflows for cross-service operations
- **Avoid Shared Databases**: Each service should own its data
- **Avoid Tight Frontend Coupling**: Use event bus for micro-frontend communication
- **Avoid Shared State**: Keep state local to each vertical slice

#### 2. Under-Communication
- **Don't Skip API Reviews**: Always review interface changes with other teams
- **Don't Hide Breaking Changes**: Communicate changes that affect other teams
- **Don't Ignore Dependencies**: Coordinate when changes affect multiple teams
- **Don't Skip Documentation**: Keep documentation current and accessible

#### 3. Quality Shortcuts
- **Don't Skip Testing**: Maintain comprehensive test coverage
- **Don't Ignore Monitoring**: Set up proper monitoring and alerting
- **Don't Rush Deployments**: Follow proper deployment and rollback procedures
- **Don't Ignore Security**: Implement security best practices consistently

## Success Metrics

### Team Autonomy Metrics

#### Development Velocity
- **Deployment Frequency**: How often teams deploy to production
- **Lead Time**: Time from code commit to production deployment
- **Change Failure Rate**: Percentage of deployments causing failures
- **Recovery Time**: Time to recover from failures

#### Quality Metrics
- **Test Coverage**: Percentage of code covered by tests
- **Bug Rate**: Number of bugs per feature or story point
- **Performance**: Response times and resource utilization
- **Security**: Number of security vulnerabilities and time to fix

#### Collaboration Metrics
- **Cross-Team Dependencies**: Number of features requiring multiple teams
- **API Contract Stability**: Frequency of breaking changes
- **Knowledge Sharing**: Documentation quality and team cross-training
- **Incident Response**: Time to resolve incidents and post-mortem quality

### Continuous Improvement

#### Regular Retrospectives
- **Team Retrospectives**: Weekly or bi-weekly team improvement discussions
- **Cross-Team Retrospectives**: Monthly discussions on system-wide improvements
- **Architecture Reviews**: Quarterly reviews of system architecture and decisions
- **Technology Reviews**: Annual reviews of technology choices and standards

#### Metrics-Driven Improvements
- **Performance Optimization**: Regular performance analysis and optimization
- **Quality Improvements**: Continuous improvement of testing and code quality
- **Process Optimization**: Streamline development and deployment processes
- **Tool Evaluation**: Regular evaluation of development tools and practices

## Conclusion

Team autonomy through vertical slice ownership enables ADX CORE to scale both technically and organizationally. By giving teams complete ownership of their domain, from frontend to backend, we achieve:

- **Faster Development**: Teams can move quickly without coordination overhead
- **Better Quality**: Clear ownership leads to better code quality and testing
- **Improved Reliability**: Teams are responsible for the full lifecycle of their services
- **Enhanced Innovation**: Teams can choose the best technologies for their domain
- **Scalable Organization**: New teams can be added without disrupting existing ones

This approach requires discipline, clear communication, and adherence to shared standards, but the benefits in terms of development velocity, system reliability, and team satisfaction make it a worthwhile investment.

For more information on implementing team autonomy in your organization, see:
- [Microservices Architecture Guide](./microservices-architecture.md)
- [API Design Guidelines](./api-design-guidelines.md)
- [Frontend Microservices Guide](./frontend-microservices.md)
- [Deployment and Operations Guide](../deployment/README.md)