# ADX Core Shell Application

The Shell Application is the main host application for the ADX Core frontend microservices architecture. It uses Module Federation to dynamically load and orchestrate micro-frontends while providing shared context, routing, and error handling.

## Features

- **Module Federation Host**: Dynamically loads micro-frontends at runtime
- **Global Routing**: Centralized routing system for all micro-frontends
- **Shared Context**: Authentication, tenant, and theme state management
- **Event Bus**: Cross-micro-frontend communication system
- **Error Boundaries**: Isolated error handling for each micro-frontend
- **Internationalization**: Multi-language support with react-i18next
- **Theme System**: Light/dark mode with system preference detection
- **Responsive Design**: Mobile-first design with TailwindCSS

## Architecture

### Module Federation Configuration

The shell app is configured as a Module Federation host that loads the following micro-frontends:

- **Auth App** (port 3001): Authentication and user management
- **Tenant App** (port 3002): Tenant management and switching
- **File App** (port 3003): File management and storage
- **User App** (port 3004): User profiles and preferences
- **Workflow App** (port 3005): Workflow monitoring and AI features
- **Module App** (port 3006): Module marketplace and management

### Shared Dependencies

The following dependencies are shared across all micro-frontends:

- React & React DOM (singleton)
- React Router DOM
- React Query
- React i18next
- Zustand

### Shared Packages

- **@adx-core/shared-context**: Authentication, tenant, and theme stores
- **@adx-core/event-bus**: Cross-micro-frontend communication
- **@adx-core/design-system**: Shared UI components and styles

## Development

### Prerequisites

- Node.js 18+
- npm 9+

### Setup

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Type checking
npm run type-check

# Linting
npm run lint
npm run lint:fix
```

### Development Server

The shell application runs on http://localhost:3000 and expects micro-frontends to be available on their respective ports:

- Auth: http://localhost:3001
- Tenant: http://localhost:3002
- File: http://localhost:3003
- User: http://localhost:3004
- Workflow: http://localhost:3005
- Module: http://localhost:3006

### Mock Data

In development mode, the application automatically sets up mock authentication and tenant data for testing purposes.

## Error Handling

The shell application implements comprehensive error handling:

- **Error Boundaries**: Each micro-frontend is wrapped in an error boundary
- **Fallback Components**: Graceful degradation when micro-frontends fail to load
- **Retry Mechanism**: Automatic retry with exponential backoff
- **Error Reporting**: Centralized error logging and monitoring

## Routing

The shell application uses React Router for navigation:

- `/` - Dashboard (protected)
- `/auth/*` - Authentication micro-frontend (public)
- `/tenant/*` - Tenant management (protected)
- `/files/*` - File management (protected)
- `/users/*` - User management (protected)
- `/workflows/*` - Workflow management (protected)
- `/modules/*` - Module management (protected)

## State Management

### Authentication State

Managed by `useAuthStore` from `@adx-core/shared-context`:

- User information
- Authentication token
- Login/logout actions
- Loading and error states

### Tenant State

Managed by `useTenantStore` from `@adx-core/shared-context`:

- Current tenant information
- Available tenants list
- Tenant switching functionality
- Tenant-specific settings and quotas

### Theme State

Managed by `useThemeStore` from `@adx-core/shared-context`:

- Theme preference (light/dark/system)
- Automatic system theme detection
- Theme persistence

## Event Communication

Cross-micro-frontend communication is handled through the event bus:

```typescript
import { useEventBus } from '@adx-core/event-bus';

const { emit, subscribe } = useEventBus();

// Emit an event
emit('tenant:switched', { tenantId: 'new-tenant-id' });

// Subscribe to events
useEffect(() => {
  const unsubscribe = subscribe('tenant:*', (event) => {
    console.log('Tenant event:', event);
  });
  return unsubscribe;
}, []);
```

## Internationalization

The shell application supports multiple languages:

- English (en)
- Spanish (es)
- French (fr)
- German (de)

Language detection is automatic based on browser preferences, with fallback to English.

## Deployment

### Production Build

```bash
npm run build
```

The build output is optimized for Module Federation with proper chunk splitting and shared dependencies.

### Environment Variables

- `NODE_ENV`: Environment mode (development/production)
- `VITE_API_BASE_URL`: Base URL for API calls
- `VITE_AUTH_DOMAIN`: Authentication domain for SSO

## Contributing

1. Follow the established patterns for components and hooks
2. Ensure all micro-frontend integrations are properly error-handled
3. Add appropriate TypeScript types for new features
4. Test cross-micro-frontend communication thoroughly
5. Update documentation for new features or changes

## Troubleshooting

### Micro-frontend Loading Issues

1. Ensure all micro-frontends are running on their expected ports
2. Check browser console for Module Federation errors
3. Verify shared dependency versions match across applications
4. Clear browser cache and restart development servers

### State Synchronization Issues

1. Check event bus communication between micro-frontends
2. Verify shared context providers are properly configured
3. Ensure state persistence is working correctly
4. Check for race conditions in state updates

### Build Issues

1. Ensure all shared packages are built before building the shell
2. Check for TypeScript errors in micro-frontend type declarations
3. Verify Module Federation configuration matches across applications
4. Clear node_modules and reinstall dependencies if needed