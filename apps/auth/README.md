# ADX Core - Auth Micro-Frontend

The authentication micro-frontend for ADX Core, built with React, TypeScript, and Module Federation.

## Features

- **Login & Registration**: Complete user authentication flows
- **Multi-Factor Authentication (MFA)**: TOTP-based 2FA with backup codes
- **Single Sign-On (SSO)**: Support for Google, Microsoft, GitHub, Okta, and SAML
- **Password Management**: Forgot password and reset password workflows
- **Form Validation**: Comprehensive client-side validation with Zod
- **Temporal Workflows**: Integration with Auth BFF service for workflow-based operations
- **Responsive Design**: Mobile-first design with TailwindCSS
- **Module Federation**: Exposes components for use in other micro-frontends

## Architecture

### Module Federation Exports

The auth micro-frontend exposes the following components:

- `./App` - Main auth application
- `./LoginForm` - Standalone login form component
- `./RegisterForm` - Standalone registration form component
- `./MFASetup` - MFA setup and verification component
- `./SSOLogin` - SSO authentication component
- `./AuthProvider` - Authentication context provider
- `./ProtectedRoute` - Route protection component

### BFF Integration

The auth micro-frontend communicates with the Auth BFF service (port 4001) which acts as a Temporal workflow client:

- **Synchronous Operations**: Login, token validation, logout
- **Asynchronous Operations**: Registration, password reset, MFA setup
- **Workflow Polling**: Real-time status updates for long-running operations

## Development

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
cd apps/auth
npm install
```

### Development Server

```bash
npm run dev
```

The auth micro-frontend will be available at http://localhost:3001

### Building

```bash
npm run build
```

### Testing

```bash
npm run test
```

### Linting

```bash
npm run lint
npm run lint:fix
```

## Environment Variables

Copy `.env.example` to `.env` and configure:

```env
VITE_AUTH_BFF_URL=http://localhost:4001
VITE_API_GATEWAY_URL=http://localhost:8080
VITE_NODE_ENV=development
VITE_DEBUG=true
```

## Usage in Shell Application

The shell application can load auth components dynamically:

```typescript
// Dynamic import
const AuthApp = React.lazy(() => import('auth_app/App'));
const LoginForm = React.lazy(() => import('auth_app/LoginForm'));

// Usage
<Route path="/auth/*" element={<AuthApp />} />
<LoginForm onSuccess={handleLoginSuccess} />
```

## Components

### LoginForm

```typescript
import { LoginForm } from 'auth_app/LoginForm';

<LoginForm />
```

### RegisterForm

```typescript
import { RegisterForm } from 'auth_app/RegisterForm';

<RegisterForm />
```

### MFASetup

```typescript
import { MFASetup } from 'auth_app/MFASetup';

<MFASetup 
  onComplete={() => navigate('/dashboard')}
  onSkip={() => navigate('/dashboard')}
/>
```

### ProtectedRoute

```typescript
import { ProtectedRoute } from 'auth_app/ProtectedRoute';

<ProtectedRoute 
  requireAuth={true}
  requiredRoles={['admin']}
  requiredPermissions={['user:read']}
>
  <AdminPanel />
</ProtectedRoute>
```

## Workflows

The auth micro-frontend integrates with the following Temporal workflows:

### User Authentication Workflows

- `user-login` - User login with credential validation
- `user-registration` - User registration with email verification
- `password-reset-request` - Password reset email sending
- `password-reset-confirm` - Password reset confirmation
- `mfa-setup-initiate` - MFA setup initialization
- `mfa-setup-confirm` - MFA setup confirmation
- `mfa-verify` - MFA code verification
- `sso-initiate` - SSO authentication initiation
- `sso-complete` - SSO authentication completion

### Workflow Status Polling

Long-running workflows are polled for status updates:

```typescript
const { data, isLoading } = useAuth();

// Initiate workflow
const result = await register(userData);

if (result.operationId) {
  // Poll for completion
  const finalResult = await pollWorkflowStatus(result.operationId);
}
```

## Security Features

- **Input Validation**: Client-side validation with Zod schemas
- **Password Strength**: Real-time password strength indicator
- **Rate Limiting**: Handled by BFF service
- **CSRF Protection**: Token-based protection
- **Secure Storage**: JWT tokens stored securely
- **Auto-refresh**: Automatic token refresh
- **Session Management**: Secure session handling

## Styling

The auth micro-frontend uses TailwindCSS with a custom design system:

- **Responsive Design**: Mobile-first approach
- **Dark Mode**: System preference detection
- **Animations**: Smooth transitions and loading states
- **Accessibility**: WCAG 2.1 AA compliance
- **Custom Components**: Reusable form components

## Error Handling

Comprehensive error handling with user-friendly messages:

- **Network Errors**: Retry mechanisms
- **Validation Errors**: Field-specific error messages
- **Workflow Errors**: Status-based error handling
- **Fallback UI**: Graceful degradation

## Performance

- **Code Splitting**: Route-based code splitting
- **Lazy Loading**: Dynamic component loading
- **Bundle Optimization**: Shared dependencies via Module Federation
- **Caching**: React Query for API response caching
- **Preloading**: Critical resource preloading

## Accessibility

- **Keyboard Navigation**: Full keyboard support
- **Screen Readers**: ARIA labels and descriptions
- **Focus Management**: Proper focus handling
- **Color Contrast**: WCAG AA compliance
- **Form Labels**: Proper form labeling
- **Error Announcements**: Screen reader announcements