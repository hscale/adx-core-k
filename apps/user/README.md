# User Micro-Frontend

The User micro-frontend is part of the ADX CORE platform, providing comprehensive user management capabilities including user profiles, settings, and directory functionality.

## Features

### User Profile Management
- View and edit user profiles
- Personal information management
- Social links and bio
- Avatar support
- Role and permission display

### User Settings
- Preferences management (theme, language, timezone)
- Notification settings
- Privacy controls
- Security settings (MFA, session timeout)
- Quota and usage monitoring

### User Directory
- Search and filter users
- User invitation system
- User activation/deactivation
- Role-based access control
- Bulk operations support

## Architecture

### Module Federation
This micro-frontend uses Vite Module Federation to expose components:
- `./App` - Main application component
- `./UserProfile` - User profile component
- `./UserSettings` - User settings component
- `./UserDirectory` - User directory component
- `./UserProvider` - User context provider

### BFF Integration
Integrates with User BFF service (port 4004) for:
- Temporal workflow orchestration
- Data aggregation and caching
- Performance optimization
- Request batching

### State Management
- React Query for server state
- Zustand for local state
- Event bus for cross-micro-frontend communication
- Tenant context integration

## Development

### Prerequisites
- Node.js 18+
- npm or yarn
- Running User BFF service (port 4004)
- Running backend services

### Installation
```bash
cd apps/user
npm install
```

### Development Server
```bash
npm run dev
```
The app will be available at http://localhost:3004

### Building
```bash
npm run build
```

### Linting
```bash
npm run lint
```

## API Integration

### User BFF Client
The micro-frontend communicates with the User BFF service through:
- RESTful endpoints for simple operations
- Temporal workflow endpoints for complex operations
- Real-time status updates via polling
- Cached responses for performance

### Workflow Operations
Complex operations are handled as Temporal workflows:
- User profile updates
- Preference synchronization
- User invitation process
- Account activation/deactivation

## Components

### UserProfile
- Displays user information and profile details
- Editable mode for profile updates
- Social links management
- Avatar display and upload

### UserSettings
- Tabbed interface for different setting categories
- Preferences (theme, language, timezone)
- Security settings (MFA, session timeout)
- Quota monitoring and usage display

### UserDirectory
- Searchable user list with filters
- User invitation modal
- User management actions
- Pagination support
- Role-based permissions

## Routing

- `/` - Redirects to directory
- `/directory` - User directory page
- `/profile` - Current user profile
- `/profile/:userId` - Specific user profile
- `/settings` - Current user settings
- `/settings/:userId` - Specific user settings (restricted)

## Event Bus Integration

### Emitted Events
- `user:updated` - When user data is updated
- `user:profile_updated` - When profile is updated
- `user:preferences_updated` - When preferences change
- `user:invited` - When user is invited
- `user:deactivated` - When user is deactivated
- `user:reactivated` - When user is reactivated

### Subscribed Events
- `tenant:switched` - Refresh user data on tenant switch
- `user:*` - Handle user-related events from other micro-frontends

## Security

### Authentication
- JWT token-based authentication
- Automatic token refresh
- Secure token storage

### Authorization
- Role-based access control
- Permission-based feature access
- Tenant isolation enforcement

### Data Protection
- Input validation and sanitization
- XSS protection
- CSRF protection
- Secure API communication

## Performance

### Optimization Strategies
- Code splitting and lazy loading
- React Query caching
- Image optimization
- Bundle size monitoring

### Caching
- Server state caching via React Query
- BFF response caching
- Local storage for preferences
- Memory caching for frequently accessed data

## Testing

### Unit Tests
```bash
npm run test
```

### Integration Tests
```bash
npm run test:integration
```

### E2E Tests
```bash
npm run test:e2e
```

## Deployment

### Standalone Deployment
The micro-frontend can be deployed independently:
```bash
npm run build
# Deploy dist/ folder to CDN or static hosting
```

### Module Federation
When deployed as part of the shell application:
- Exposes components via Module Federation
- Shares dependencies with other micro-frontends
- Loads dynamically at runtime

## Configuration

### Environment Variables
- `VITE_USER_BFF_URL` - User BFF service URL
- `VITE_API_GATEWAY_URL` - API Gateway URL
- `VITE_TENANT_ID` - Default tenant ID

### Build Configuration
- Vite configuration in `vite.config.ts`
- TypeScript configuration in `tsconfig.json`
- ESLint configuration in `.eslintrc.cjs`

## Troubleshooting

### Common Issues
1. **Module Federation Loading Errors**
   - Check network connectivity
   - Verify remote entry URLs
   - Check shared dependency versions

2. **BFF Connection Issues**
   - Verify BFF service is running
   - Check authentication tokens
   - Validate tenant context

3. **State Synchronization Issues**
   - Check event bus configuration
   - Verify tenant context provider
   - Validate React Query cache

### Debug Mode
Enable debug logging:
```javascript
localStorage.setItem('debug', 'user-app:*');
```

## Contributing

1. Follow the established code style
2. Write tests for new features
3. Update documentation
4. Follow semantic versioning
5. Create pull requests for review

## License

This project is part of the ADX CORE platform and follows the same licensing terms.