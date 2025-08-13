# ADX Core - Tenant Micro-Frontend

The Tenant micro-frontend provides comprehensive tenant management capabilities including tenant switching, settings management, team member management, and invitation handling.

## Features

- **Tenant Switching**: Seamless switching between multiple tenants with workflow-based context updates
- **Tenant Settings**: Comprehensive tenant configuration including general settings, security policies, and branding
- **Team Management**: Member management with role-based permissions and status tracking
- **Invitation System**: Send, manage, and track team member invitations with workflow orchestration
- **Dashboard**: Overview of tenant usage, quotas, and quick actions
- **Real-time Updates**: Event-driven updates across micro-frontends

## Architecture

### Module Federation Setup
- **Port**: 3002
- **Remote Name**: `tenant_app`
- **Exposed Components**:
  - `./App` - Main tenant application
  - `./TenantSwitcher` - Tenant switching component
  - `./TenantSettings` - Tenant settings management
  - `./TenantInvitation` - Team invitation management
  - `./TenantMembership` - Team member management
  - `./TenantProvider` - Tenant context provider

### BFF Integration
- **BFF Service**: http://localhost:4002
- **Workflow Client**: Integrates with Temporal workflows for complex operations
- **Caching**: Redis-based caching for performance optimization
- **Real-time Updates**: WebSocket support for live updates

## Components

### Core Components

#### TenantSwitcher
- Dropdown interface for switching between available tenants
- Workflow-based tenant switching with progress tracking
- Visual indicators for current tenant and subscription tiers
- Support for creating new tenants

#### TenantSettings
- Tabbed interface for different setting categories:
  - **General**: Name, description, timezone, language, theme, notifications
  - **Security**: MFA requirements, session timeout, password policies
  - **Branding**: Custom colors, logo, domain configuration
- Form validation with Zod schemas
- Real-time preview for branding changes

#### TenantInvitation
- Send team member invitations with role selection
- Manage pending and expired invitations
- Resend and cancel invitation capabilities
- Personal message support for invitations

#### TenantMembership
- View and manage team members
- Role-based permission management
- Member status tracking (active, invited, suspended)
- Bulk operations support

### Pages

#### TenantDashboard
- Overview of tenant metrics and usage
- Quota visualization with progress bars
- Quick action buttons for common tasks
- Recent activity feed

#### TenantSettingsPage
- Wrapper for TenantSettings component
- Navigation and breadcrumb support

#### TenantMembersPage
- Combined view of membership and invitations
- Side-by-side layout for efficient management

## Hooks

### useTenant Hooks
- `useCurrentTenant()` - Get current tenant information
- `useUserTenants()` - Get all available tenants for user
- `useTenant(id)` - Get specific tenant details
- `useCreateTenant()` - Create new tenant (workflow)
- `useUpdateTenant()` - Update tenant settings
- `useDeleteTenant()` - Delete tenant (workflow)
- `useSwitchTenant()` - Switch active tenant (workflow)

### Member Management Hooks
- `useTenantMembers(tenantId)` - Get tenant members
- `useInviteMember()` - Send member invitation (workflow)
- `useUpdateMember()` - Update member role/status
- `useRemoveMember()` - Remove team member

### Invitation Hooks
- `useTenantInvitations(tenantId)` - Get tenant invitations
- `useCancelInvitation()` - Cancel pending invitation
- `useResendInvitation()` - Resend expired invitation

## Services

### TenantBFFClient
- HTTP client for BFF service communication
- Workflow initiation and status polling
- Caching layer for performance
- Error handling and retry logic

## Types

### Core Types
- `Tenant` - Tenant entity with settings and quotas
- `TenantMember` - Team member with role and status
- `TenantInvitation` - Invitation with expiration and status
- `TenantSettings` - Comprehensive settings structure
- `TenantQuotas` - Usage limits and current consumption

### Enums
- `SubscriptionTier` - Free, Professional, Enterprise
- `TenantStatus` - Active, Suspended, Pending, Cancelled
- `TenantRole` - Owner, Admin, Member, Viewer
- `MemberStatus` - Active, Invited, Suspended
- `InvitationStatus` - Pending, Accepted, Expired, Cancelled

## Utilities

### Validation
- Zod schemas for form validation
- Custom validation functions
- Error message formatting

### Formatting
- Display formatters for enums and values
- Date and time formatting
- File size and quota formatting
- Color coding for status indicators

## Development

### Setup
```bash
cd apps/tenant
npm install
npm run dev
```

### Building
```bash
npm run build
```

### Testing
```bash
npm run test
npm run test:coverage
```

### Linting
```bash
npm run lint
npm run lint:fix
```

## Integration

### Event Bus Communication
The tenant micro-frontend communicates with other micro-frontends through the event bus:

#### Emitted Events
- `tenant:switched` - When user switches tenants
- `tenant:updated` - When tenant settings are updated
- `tenant:member_invited` - When new member is invited
- `tenant:member_updated` - When member role/status changes
- `tenant:member_removed` - When member is removed

#### Subscribed Events
- `tenant:*` - All tenant-related events from other micro-frontends
- `auth:logout` - Clear tenant context on logout
- `user:profile_updated` - Update member information

### Shared Context
The TenantProvider integrates with the shared context system to provide tenant information to other micro-frontends.

## Workflow Integration

### Temporal Workflows
Complex operations are handled through Temporal workflows:

- **Tenant Creation**: Multi-step tenant provisioning
- **Tenant Switching**: Cross-service context updates
- **Member Invitation**: Email sending and tracking
- **Tenant Deletion**: Cleanup and data archival

### Workflow Status Tracking
- Real-time progress updates
- Error handling and retry logic
- Cancellation support where applicable

## Performance

### Optimization Strategies
- React Query for efficient data fetching and caching
- BFF service for data aggregation
- Lazy loading of heavy components
- Memoization of expensive calculations
- Debounced search and filtering

### Bundle Size
- Module Federation for shared dependencies
- Tree shaking for unused code elimination
- Code splitting for route-based loading
- Optimized asset loading

## Security

### Authentication
- JWT token-based authentication
- Automatic token refresh
- Secure token storage

### Authorization
- Role-based access control
- Permission checking at component level
- API endpoint protection

### Data Protection
- Input sanitization and validation
- XSS protection
- CSRF protection
- Secure communication with BFF service

## Deployment

### Development
```bash
npm run dev  # Starts on port 3002
```

### Production
```bash
npm run build
npm run preview
```

### Docker
```dockerfile
FROM node:18-alpine
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production
COPY dist ./dist
EXPOSE 3002
CMD ["npm", "run", "preview"]
```

## Monitoring

### Metrics
- Component render performance
- API response times
- Error rates and types
- User interaction tracking

### Logging
- Structured logging with context
- Error boundary integration
- Performance monitoring
- User action tracking

## Contributing

1. Follow the established component patterns
2. Add comprehensive TypeScript types
3. Include unit tests for new functionality
4. Update documentation for API changes
5. Follow the established styling conventions
6. Ensure accessibility compliance

## Dependencies

### Core Dependencies
- React 18+ with TypeScript
- React Router for navigation
- React Query for data fetching
- React Hook Form for form management
- Zod for validation
- Tailwind CSS for styling

### Shared Dependencies
- @adx-core/shared-context
- @adx-core/event-bus
- @adx-core/design-system

### Development Dependencies
- Vite with Module Federation
- TypeScript
- ESLint and Prettier
- Testing utilities