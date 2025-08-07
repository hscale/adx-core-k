# Frontend Microservices Migration Guide

This guide helps you migrate from the monolithic frontend to the microservices architecture incrementally and safely.

## Overview

The migration strategy follows these principles:
- **Incremental**: Migrate one domain at a time
- **Safe**: Maintain backward compatibility during transition
- **Testable**: Each step can be validated independently
- **Reversible**: Easy rollback if issues arise

## Migration Phases

### Phase 1: Infrastructure Setup (Week 1-2)

#### 1.1 Create Shell Application
```bash
# Create the shell application
./scripts/create-micro-frontend.sh shell 3000 react

# Install dependencies
cd micro-frontends/shell
npm install
```

#### 1.2 Setup Shared Design System
```bash
# Create shared package
mkdir -p micro-frontends/shared/design-system
cd micro-frontends/shared/design-system

# Extract existing components from frontend/src/components
# This will be done gradually as we migrate each micro-frontend
```

#### 1.3 Configure Module Federation
Update the shell application to load micro-frontends dynamically:

```typescript
// micro-frontends/shell/vite.config.ts
federation({
  name: 'shell',
  remotes: {
    // Add remotes as micro-frontends are created
    authMicroApp: 'http://localhost:3001/assets/remoteEntry.js',
  },
  shared: {
    react: { singleton: true, requiredVersion: '^18.2.0' },
    'react-dom': { singleton: true },
    'react-router-dom': { singleton: true },
    // ... other shared dependencies
  },
})
```

### Phase 2: Auth Micro-Frontend (Week 3-4)

#### 2.1 Create Auth Micro-Frontend
```bash
# Create auth micro-frontend
./scripts/create-micro-frontend.sh auth 3001 react
```

#### 2.2 Migrate Auth Components
Move these components from `frontend/src/` to `micro-frontends/auth-micro-app/src/`:

- `pages/auth/LoginPage.tsx`
- `pages/auth/RegisterPage.tsx`
- `pages/auth/ForgotPasswordPage.tsx`
- `contexts/AuthContext.tsx`
- `hooks/useAuth.ts`
- `services/auth.ts`

#### 2.3 Create Auth BFF Service
```bash
# Create Node.js BFF service
mkdir -p bff-services/auth-bff
cd bff-services/auth-bff

# Initialize package.json
npm init -y
npm install express cors axios typescript @types/node @types/express

# Create basic server structure
mkdir -p src/{routes,services,middleware,types}
```

#### 2.4 Update Shell Application
Add auth micro-frontend to shell routing:

```typescript
// micro-frontends/shell/src/App.tsx
const authMicroApp = React.lazy(() => import('authMicroApp/App'));

<Route path="/auth/*" element={
  <Suspense fallback={<div>Loading...</div>}>
    <authMicroApp />
  </Suspense>
} />
```

#### 2.5 Feature Flag Integration
Add feature flag to gradually roll out:

```typescript
// Feature flag to switch between old and new auth
const useNewAuth = process.env.REACT_APP_USE_NEW_AUTH === 'true';

if (useNewAuth) {
  // Route to auth micro-frontend
} else {
  // Route to legacy auth pages
}
```

### Phase 3: File Management Micro-Frontend (Week 5-6)

#### 3.1 Create File Micro-Frontend
```bash
./scripts/create-micro-frontend.sh file 3003 react
```

#### 3.2 Migrate File Components
Move these components:
- `pages/files/FilesPage.tsx`
- File upload components
- File list and grid components
- File sharing modals

#### 3.3 Create File BFF (Rust)
```bash
mkdir -p bff-services/file-bff
cd bff-services/file-bff

# Initialize Cargo project
cargo init
```

Add dependencies to `Cargo.toml`:
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
reqwest = { version = "0.11", features = ["json"] }
tower-http = { version = "0.4", features = ["cors"] }
```

### Phase 4: User Management Micro-Frontend (Week 7-8)

#### 4.1 Create User Micro-Frontend
```bash
./scripts/create-micro-frontend.sh user 3004 react
```

#### 4.2 Migrate User Components
- `pages/users/UsersPage.tsx`
- `pages/profile/ProfilePage.tsx`
- User management components
- Profile editing components

### Phase 5: Tenant Management Micro-Frontend (Week 9-10)

#### 5.1 Create Tenant Micro-Frontend
```bash
./scripts/create-micro-frontend.sh tenant 3002 react
```

#### 5.2 Migrate Tenant Components
- `pages/tenants/TenantsPage.tsx`
- `contexts/TenantContext.tsx`
- Tenant switching components

### Phase 6: Workflow Management Micro-Frontend (Week 11-12)

#### 6.1 Create Workflow Micro-Frontend
```bash
./scripts/create-micro-frontend.sh workflow 3005 react
```

#### 6.2 Migrate Workflow Components
- `pages/workflows/WorkflowsPage.tsx`
- Workflow builder components
- Workflow execution monitoring

### Phase 7: Dashboard Micro-Frontend (Week 13-14)

#### 7.1 Create Dashboard Micro-Frontend
```bash
./scripts/create-micro-frontend.sh dashboard 3006 react
```

#### 7.2 Migrate Dashboard Components
- `pages/dashboard/DashboardPage.tsx`
- Analytics components
- Dashboard widgets

## Migration Checklist

### Before Starting Migration
- [ ] Backend services are stable and running
- [ ] Current frontend tests are passing
- [ ] Development environment is set up
- [ ] Team is trained on micro-frontend concepts

### For Each Micro-Frontend
- [ ] Create micro-frontend using script
- [ ] Set up development environment
- [ ] Migrate components incrementally
- [ ] Create corresponding BFF service
- [ ] Add to shell application routing
- [ ] Implement feature flags
- [ ] Write tests for micro-frontend
- [ ] Test integration with shell
- [ ] Deploy to staging environment
- [ ] Gradual rollout with monitoring
- [ ] Full rollout after validation

### After Migration
- [ ] Remove legacy components
- [ ] Clean up unused dependencies
- [ ] Update documentation
- [ ] Performance optimization
- [ ] Monitor production metrics

## Testing Strategy

### Unit Testing
Each micro-frontend should have its own test suite:
```bash
cd micro-frontends/auth-micro-app
npm run test
```

### Integration Testing
Test micro-frontend communication:
```bash
# Test event bus communication
npm run test:integration

# Test BFF integration
npm run test:bff
```

### End-to-End Testing
Test complete user flows across micro-frontends:
```bash
npm run test:e2e
```

## Rollback Strategy

### Immediate Rollback
If issues are detected, use feature flags to immediately switch back:
```bash
# Disable new micro-frontend
export REACT_APP_USE_NEW_AUTH=false
```

### Gradual Rollback
Roll back users gradually:
```typescript
// Rollback 50% of users
const useNewAuth = Math.random() < 0.5;
```

### Complete Rollback
If major issues occur:
1. Set feature flags to disable all micro-frontends
2. Route all traffic to legacy components
3. Investigate and fix issues
4. Re-enable gradually

## Performance Monitoring

### Key Metrics to Track
- **Loading Time**: Time to interactive for each micro-frontend
- **Bundle Size**: Track size changes over time
- **Error Rates**: Monitor errors by micro-frontend
- **User Experience**: Core Web Vitals and user satisfaction

### Monitoring Tools
```bash
# Bundle analysis
npm run analyze:bundles

# Performance monitoring
npm run monitor:performance

# Error tracking
npm run monitor:errors
```

## Common Issues and Solutions

### Issue: Shared Dependency Conflicts
**Solution**: Ensure consistent versions in all micro-frontends
```json
{
  "shared": {
    "react": { "singleton": true, "requiredVersion": "^18.2.0" }
  }
}
```

### Issue: Slow Loading Times
**Solution**: Implement preloading and code splitting
```typescript
// Preload critical micro-frontends
const preloadMicroFrontend = (name: string) => {
  import(/* webpackPreload: true */ `${name}/App`);
};
```

### Issue: Cross-Micro-Frontend Communication
**Solution**: Use event bus or API-driven communication
```typescript
// Event bus communication
eventBus.emit('auth.login', { user, tenant });

// API-driven communication
await api.post('/user/preferences', preferences);
```

## Support and Resources

### Documentation
- [Frontend Microservices Architecture](/.kiro/steering/frontend-microservices.md)
- [Technology Stack](/.kiro/steering/tech.md)
- [Project Structure](/.kiro/steering/structure.md)

### Scripts and Tools
- `./scripts/create-micro-frontend.sh` - Create new micro-frontend
- `./scripts/dev-start-frontend.sh` - Start development environment
- `npm run analyze:bundles` - Analyze bundle sizes
- `npm run test:integration` - Run integration tests

### Getting Help
1. Check existing documentation
2. Review similar micro-frontend implementations
3. Ask team for guidance
4. Create detailed issue reports with reproduction steps

## Success Criteria

### Technical Success
- [ ] All micro-frontends load within performance budgets
- [ ] No shared dependency conflicts
- [ ] Independent deployment working
- [ ] Cross-platform compatibility maintained

### Team Success
- [ ] Teams can develop independently
- [ ] Deployment frequency increased
- [ ] Reduced cross-team dependencies
- [ ] Improved development velocity

### User Success
- [ ] No degradation in user experience
- [ ] Improved application performance
- [ ] Consistent UI across all features
- [ ] Reliable functionality across platforms