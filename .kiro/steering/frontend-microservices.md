# Frontend Microservices Architecture

## Overview

ADX CORE uses a frontend microservices architecture that mirrors our backend domain boundaries. This enables team autonomy, independent deployment, and technology flexibility while maintaining a cohesive user experience across web, desktop, and mobile platforms.

## Key Refinements

### Simplified Migration Strategy
- **Incremental Migration**: Transform existing monolithic components into micro-frontends one domain at a time
- **Backward Compatibility**: Maintain existing functionality during transition with feature flags
- **Risk Mitigation**: Each micro-frontend can be rolled back independently if issues arise
- **Performance First**: Optimize bundle sizes and loading performance from day one

## Architecture Principles

### Team Autonomy
- Each team owns a complete vertical slice (backend service + frontend micro-app + BFF)
- Teams can develop, test, and deploy independently without coordination
- Technology stack freedom within shared design system constraints
- Independent CI/CD pipelines per micro-frontend

### Domain-Driven Design
- Frontend boundaries mirror backend service boundaries
- Auth, Tenant, File, User, Workflow micro-frontends align with backend services
- Clear domain ownership and responsibility
- Consistent domain separation prevents monolithic coupling

### Technology Flexibility
- Framework agnostic: React, Vue, Svelte, Angular support
- Module Federation for runtime integration and dependency sharing
- Shared design system ensures UI consistency
- Platform detection for web/desktop/mobile adaptations

## Technical Architecture

### Shell Application
- **Purpose**: Container and orchestrator for all micro-frontends
- **Technology**: React + Vite + Module Federation
- **Port**: 3000 (development)
- **Responsibilities**:
  - Global routing and navigation
  - Authentication state management
  - Theme and i18n providers
  - Error boundaries and fallbacks
  - Cross-micro-frontend event bus
  - Performance monitoring and optimization
  - Feature flag management for gradual rollout

### Micro-Frontend Structure
```
micro-frontends/
├── shell/                   # Shell application (port 3000)
├── auth-micro-app/          # Authentication flows (port 3001)
├── tenant-micro-app/        # Tenant switching and management (port 3002)
├── file-micro-app/          # File management and sharing (port 3003)
├── user-micro-app/          # User profiles and settings (port 3004)
├── workflow-micro-app/      # Workflow management and AI (port 3005)
├── dashboard-micro-app/     # Main dashboard and analytics (port 3006)
└── shared/
    ├── design-system/       # Shared UI components
    ├── types/              # Common TypeScript types
    ├── utils/              # Shared utilities
    ├── hooks/              # Shared React hooks
    └── services/           # Shared API clients
```

### Backend for Frontend (BFF) Pattern
Each micro-frontend has its own BFF service for data aggregation and optimization:
- **Auth BFF** (Node.js/TypeScript, port 4001): Aggregates auth, user, and tenant data
- **Tenant BFF** (Node.js/TypeScript, port 4002): Tenant management and switching
- **File BFF** (Rust/Axum, port 4003): Combines file, permission, and storage data
- **User BFF** (Rust/Axum, port 4004): User profiles and preferences
- **Workflow BFF** (Rust/Axum, port 4005): Workflow execution and AI integration
- **Dashboard BFF** (Rust/Axum, port 4006): Analytics and dashboard data aggregation

### BFF Benefits
- **Reduced Network Calls**: Aggregate multiple backend calls into single requests
- **Frontend-Optimized APIs**: Data shaped specifically for UI needs
- **Independent Evolution**: BFF can evolve with micro-frontend without affecting backend
- **Performance Optimization**: Caching, batching, and response optimization

## Module Federation Configuration

### Shell Application (Port 3000)
```typescript
// vite.config.ts
federation({
  name: 'shell',
  remotes: {
    authMicroApp: 'http://localhost:3001/assets/remoteEntry.js',
    tenantMicroApp: 'http://localhost:3002/assets/remoteEntry.js',
    fileMicroApp: 'http://localhost:3003/assets/remoteEntry.js',
    userMicroApp: 'http://localhost:3004/assets/remoteEntry.js',
    workflowMicroApp: 'http://localhost:3005/assets/remoteEntry.js',
  },
  shared: {
    react: { singleton: true, requiredVersion: '^18.2.0' },
    'react-dom': { singleton: true },
    'react-router-dom': { singleton: true },
    '@tanstack/react-query': { singleton: true },
    'zustand': { singleton: true },
    'tailwindcss': { singleton: true },
  },
})
```

### Micro-Frontend Template
```typescript
// Each micro-frontend exposes components
federation({
  name: 'authMicroApp',
  filename: 'remoteEntry.js',
  exposes: {
    './App': './src/App.tsx',
    './LoginPage': './src/pages/LoginPage.tsx',
  },
  shared: { /* same as shell */ },
})
```

## Development Ports

### Frontend Services
- Shell Application: http://localhost:3000
- Auth Micro-App: http://localhost:3001
- Tenant Micro-App: http://localhost:3002
- File Micro-App: http://localhost:3003
- User Micro-App: http://localhost:3004
- Workflow Micro-App: http://localhost:3005
- Dashboard Micro-App: http://localhost:3006

### BFF Services
- Auth BFF: http://localhost:4001
- Tenant BFF: http://localhost:4002
- File BFF: http://localhost:4003
- User BFF: http://localhost:4004
- Workflow BFF: http://localhost:4005
- Dashboard BFF: http://localhost:4006

### Backend Services (Existing)
- API Gateway: http://localhost:8080
- Auth Service: http://localhost:8081
- User Service: http://localhost:8082
- File Service: http://localhost:8083
- Workflow Service: http://localhost:8084
- Tenant Service: http://localhost:8085

## Cross-Platform Integration

### Tauri Configuration
- **Security Policy**: Updated CSP for Module Federation
- **Native APIs**: File system, notifications, platform detection
- **Build Targets**: Web, desktop (Windows/macOS/Linux), mobile (iOS/Android)
- **Platform Detection**: Automatic adaptation for different platforms

### Platform-Specific Components
```typescript
// Platform-aware component wrapper
<PlatformAware
  web={<WebFileUpload />}
  desktop={<TauriFileUpload />}
  mobile={<MobileFileUpload />}
/>
```

## Communication Patterns

### Event Bus
- Cross-micro-frontend communication via custom event bus
- Typed events for auth, tenant switching, file operations
- Error boundaries prevent cascading failures
- Event debugging and monitoring capabilities

### API-Driven Communication
- Micro-frontends communicate via BFF services
- State persistence through backend APIs
- Real-time updates via WebSocket connections
- Offline-first approach with synchronization

## Development Commands

### Quick Start
```bash
# Start entire development environment (recommended)
./scripts/dev-start-frontend.sh

# Start from project root
npm run dev:all

# Start individual components for focused development
npm run dev:shell          # Shell application only
npm run dev:auth           # Auth micro-frontend only
npm run dev:file           # File micro-frontend only
npm run dev:dashboard      # Dashboard micro-frontend only
```

### BFF Services
```bash
# Start all BFF services
npm run dev:bff

# Start individual BFF services
npm run dev:auth-bff       # Auth BFF only
npm run dev:file-bff       # File BFF only
npm run dev:dashboard-bff  # Dashboard BFF only
```

### Development Modes
```bash
# Development with hot reload
npm run dev

# Development with mock data (no backend required)
npm run dev:mock

# Development with specific micro-frontend focus
npm run dev:focus auth     # Only load auth micro-frontend
```

### Building and Deployment
```bash
# Build all micro-frontends for production
npm run build:all

# Build individual micro-frontends
npm run build:shell
npm run build:auth
npm run build:file
npm run build:dashboard

# Build for specific platforms
npm run build:web          # Web browsers
npm run build:desktop      # Tauri desktop apps
npm run build:mobile       # Tauri mobile apps

# Deploy individual micro-frontends (CI/CD)
npm run deploy:auth
npm run deploy:file
npm run deploy:dashboard

# Deploy with feature flags
npm run deploy:auth --feature-flag=new-login-flow
```

### Testing
```bash
# Unit tests for each micro-frontend
npm run test:unit

# Integration tests across micro-frontends
npm run test:integration

# End-to-end tests
npm run test:e2e

# Performance testing
npm run test:performance
```

## Plugin System Integration

### Plugin Micro-Frontends
- Plugins can register their own micro-frontends
- Dynamic loading and unloading of plugin UIs
- Sandboxed execution with resource limits
- Plugin marketplace integration

### Plugin Development
```typescript
// Plugin micro-frontend registration
export const PluginMicroApp: React.FC = () => {
  return (
    <Routes>
      <Route path="/plugin-feature" element={<PluginFeature />} />
    </Routes>
  );
};
```

## Monitoring and Observability

### Performance Monitoring
- Micro-frontend loading times and bundle sizes
- Cross-micro-frontend communication latency
- User interaction tracking per micro-frontend
- Error attribution and debugging

### Development Tools
- Module Federation debugging
- Event bus monitoring
- Performance profiling
- Hot module replacement across micro-frontends

## Migration Strategy

### Phased Approach
1. **Foundation**: Shell application and Module Federation setup
2. **Core Micro-Frontends**: Auth, File, Tenant, User, Workflow
3. **Plugin Integration**: Plugin system and marketplace
4. **Optimization**: Performance tuning and monitoring
5. **Legacy Cleanup**: Remove monolithic frontend components

### Feature Flags
- Gradual rollout of micro-frontend features
- A/B testing for performance comparison
- Rollback mechanisms for failed deployments
- User feedback collection and analysis

## Performance Optimization

### Bundle Size Management
```bash
# Analyze bundle sizes
npm run analyze:bundles

# Check shared dependency usage
npm run analyze:shared-deps

# Performance budget checks
npm run check:performance-budget
```

### Loading Strategies
- **Lazy Loading**: Micro-frontends load only when needed
- **Preloading**: Critical micro-frontends preload in background
- **Code Splitting**: Route-based splitting within micro-frontends
- **Resource Hints**: DNS prefetch and preconnect for BFF services

### Caching Strategy
- **Shared Dependencies**: Cached across all micro-frontends
- **Micro-Frontend Assets**: Versioned and cached with long TTL
- **BFF Responses**: Redis caching for frequently accessed data
- **Static Assets**: CDN caching for images, fonts, and icons

## Monitoring and Observability

### Performance Metrics
- **Loading Time**: Time to interactive for each micro-frontend
- **Bundle Size**: Track size changes over time
- **Network Requests**: Monitor BFF call patterns and latency
- **Error Rates**: Track errors by micro-frontend

### Development Tools
```bash
# Start with performance monitoring
npm run dev:monitor

# Debug micro-frontend communication
npm run debug:events

# Test micro-frontend isolation
npm run test:isolation
```

### Production Monitoring
- **Real User Monitoring**: Track actual user performance
- **Error Tracking**: Attribute errors to specific micro-frontends
- **Usage Analytics**: Monitor feature adoption across micro-frontends
- **Health Checks**: Automated monitoring of micro-frontend availability

## Best Practices

### Code Organization
- Consistent project structure across micro-frontends
- Shared TypeScript types and utilities in `@adx-core/shared`
- Versioned design system components with semantic versioning
- Clear API contracts between micro-frontends and BFFs

### Performance Optimization
- Lazy loading of micro-frontends with loading states
- Bundle size monitoring with automated alerts
- Shared dependency optimization to prevent duplication
- Performance budgets enforced in CI/CD pipeline

### Security Considerations
- CSP policies configured for Module Federation dynamic loading
- Plugin sandboxing with restricted permissions
- Secure event bus communication with message validation
- JWT token sharing through secure context providers

### Development Experience
- Hot module replacement across micro-frontend boundaries
- Comprehensive error messages for integration issues
- Mock data support for independent development
- Automated testing across micro-frontend interactions