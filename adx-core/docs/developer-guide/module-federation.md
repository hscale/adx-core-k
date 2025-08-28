# Module Federation and Micro-Frontend Development Guide

## Overview

ADX CORE uses Module Federation to implement a micro-frontend architecture where each domain-specific application can be developed, tested, and deployed independently while maintaining a cohesive user experience. This guide covers the setup, development patterns, and best practices for working with Module Federation in ADX CORE.

## Architecture Overview

### Module Federation Structure

```
┌─────────────────────────────────────────────────────────────┐
│                    Shell Application                       │
│                   (Host - Port 3000)                      │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│ │   Auth App  │ │ Tenant App  │ │  File App   │ │User App │ │
│ │(Remote 3001)│ │(Remote 3002)│ │(Remote 3003)│ │(3004)   │ │
│ └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
├─────────────────────────────────────────────────────────────┤
│ ┌─────────────┐ ┌─────────────┐                             │
│ │Workflow App │ │ Module App  │                             │
│ │(Remote 3005)│ │(Remote 3006)│                             │
│ └─────────────┘ └─────────────┘                             │
├─────────────────────────────────────────────────────────────┤
│                  Shared Dependencies                       │
│ React, React-DOM, Design System, Event Bus, Context       │
└─────────────────────────────────────────────────────────────┘
```

### Key Components

1. **Shell Application (Host)**: Main application that loads and orchestrates micro-frontends
2. **Remote Applications**: Independent micro-frontends for specific domains
3. **Shared Dependencies**: Common libraries shared across all applications
4. **Event Bus**: Communication mechanism between micro-frontends
5. **Shared Context**: Global state management for authentication, tenant, and theme

## Shell Application Setup

### Vite Configuration

```typescript
// apps/shell/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'shell_app',
      remotes: {
        auth_app: {
          external: 'Promise.resolve(window.authAppUrl || "http://localhost:3001/assets/remoteEntry.js")',
          externalType: 'promise',
        },
        tenant_app: {
          external: 'Promise.resolve(window.tenantAppUrl || "http://localhost:3002/assets/remoteEntry.js")',
          externalType: 'promise',
        },
        file_app: {
          external: 'Promise.resolve(window.fileAppUrl || "http://localhost:3003/assets/remoteEntry.js")',
          externalType: 'promise',
        },
        user_app: {
          external: 'Promise.resolve(window.userAppUrl || "http://localhost:3004/assets/remoteEntry.js")',
          externalType: 'promise',
        },
        workflow_app: {
          external: 'Promise.resolve(window.workflowAppUrl || "http://localhost:3005/assets/remoteEntry.js")',
          externalType: 'promise',
        },
        module_app: {
          external: 'Promise.resolve(window.moduleAppUrl || "http://localhost:3006/assets/remoteEntry.js")',
          externalType: 'promise',
        },
      },
      shared: {
        react: {
          singleton: true,
          requiredVersion: '^18.2.0',
        },
        'react-dom': {
          singleton: true,
          requiredVersion: '^18.2.0',
        },
        '@tanstack/react-query': {
          singleton: true,
          requiredVersion: '^5.8.4',
        },
        '@adx-core/design-system': {
          singleton: true,
          requiredVersion: 'workspace:*',
        },
        '@adx-core/shared-context': {
          singleton: true,
          requiredVersion: 'workspace:*',
        },
        '@adx-core/event-bus': {
          singleton: true,
          requiredVersion: 'workspace:*',
        },
        '@adx-core/i18n': {
          singleton: true,
          requiredVersion: 'workspace:*',
        },
        'react-router-dom': {
          singleton: true,
          requiredVersion: '^6.20.1',
        },
        zustand: {
          singleton: true,
          requiredVersion: '^4.4.7',
        },
      },
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
    rollupOptions: {
      external: ['react', 'react-dom'],
    },
  },
  server: {
    port: 3000,
    cors: true,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    },
  },
});
```

### Shell Application Structure

```typescript
// apps/shell/src/App.tsx
import React, { Suspense } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TenantProvider } from '@adx-core/shared-context';
import { AuthProvider } from '@adx-core/shared-context';
import { DesignSystemProvider } from '@adx-core/design-system';
import { EventBusProvider } from '@adx-core/event-bus';
import { I18nProvider } from '@adx-core/i18n';
import { ErrorBoundary } from './components/ErrorBoundary';
import { MicroFrontendLoader } from './components/MicroFrontendLoader';
import { Navigation } from './components/Navigation';
import { Dashboard } from './pages/Dashboard';

// Dynamic imports for micro-frontends
const AuthApp = React.lazy(() => import('auth_app/App'));
const TenantApp = React.lazy(() => import('tenant_app/App'));
const FileApp = React.lazy(() => import('file_app/App'));
const UserApp = React.lazy(() => import('user_app/App'));
const WorkflowApp = React.lazy(() => import('workflow_app/App'));
const ModuleApp = React.lazy(() => import('module_app/App'));

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
      retry: (failureCount, error) => {
        // Don't retry on 4xx errors
        if (error?.response?.status >= 400 && error?.response?.status < 500) {
          return false;
        }
        return failureCount < 3;
      },
    },
  },
});

export const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <I18nProvider>
          <AuthProvider>
            <TenantProvider>
              <DesignSystemProvider>
                <EventBusProvider>
                  <BrowserRouter>
                    <div className="app-shell min-h-screen bg-gray-50 dark:bg-gray-900">
                      <Navigation />
                      <main className="main-content">
                        <Suspense fallback={<MicroFrontendLoader />}>
                          <Routes>
                            <Route path="/" element={<Dashboard />} />
                            <Route 
                              path="/auth/*" 
                              element={
                                <ErrorBoundary fallback={<MicroFrontendError app="Auth" />}>
                                  <AuthApp />
                                </ErrorBoundary>
                              } 
                            />
                            <Route 
                              path="/tenant/*" 
                              element={
                                <ErrorBoundary fallback={<MicroFrontendError app="Tenant" />}>
                                  <TenantApp />
                                </ErrorBoundary>
                              } 
                            />
                            <Route 
                              path="/files/*" 
                              element={
                                <ErrorBoundary fallback={<MicroFrontendError app="File" />}>
                                  <FileApp />
                                </ErrorBoundary>
                              } 
                            />
                            <Route 
                              path="/users/*" 
                              element={
                                <ErrorBoundary fallback={<MicroFrontendError app="User" />}>
                                  <UserApp />
                                </ErrorBoundary>
                              } 
                            />
                            <Route 
                              path="/workflows/*" 
                              element={
                                <ErrorBoundary fallback={<MicroFrontendError app="Workflow" />}>
                                  <WorkflowApp />
                                </ErrorBoundary>
                              } 
                            />
                            <Route 
                              path="/modules/*" 
                              element={
                                <ErrorBoundary fallback={<MicroFrontendError app="Module" />}>
                                  <ModuleApp />
                                </ErrorBoundary>
                              } 
                            />
                          </Routes>
                        </Suspense>
                      </main>
                    </div>
                  </BrowserRouter>
                </EventBusProvider>
              </DesignSystemProvider>
            </TenantProvider>
          </AuthProvider>
        </I18nProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
};
```

### Error Boundaries and Fallbacks

```typescript
// apps/shell/src/components/ErrorBoundary.tsx
import React, { Component, ReactNode } from 'react';
import { Button } from '@adx-core/design-system';

interface Props {
  children: ReactNode;
  fallback?: ReactNode;
}

interface State {
  hasError: boolean;
  error?: Error;
}

export class ErrorBoundary extends Component<Props, State> {
  constructor(props: Props) {
    super(props);
    this.state = { hasError: false };
  }

  static getDerivedStateFromError(error: Error): State {
    return { hasError: true, error };
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Micro-frontend error:', error, errorInfo);
    
    // Report error to monitoring service
    if (window.analytics) {
      window.analytics.track('Micro-Frontend Error', {
        error: error.message,
        stack: error.stack,
        componentStack: errorInfo.componentStack,
      });
    }
  }

  render() {
    if (this.state.hasError) {
      if (this.props.fallback) {
        return this.props.fallback;
      }

      return (
        <div className="error-boundary p-8 text-center">
          <div className="max-w-md mx-auto">
            <div className="text-red-500 text-6xl mb-4">⚠️</div>
            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">
              Something went wrong
            </h2>
            <p className="text-gray-600 dark:text-gray-400 mb-6">
              This micro-frontend encountered an error. Please try refreshing the page.
            </p>
            <div className="space-x-4">
              <Button
                onClick={() => window.location.reload()}
                variant="primary"
              >
                Refresh Page
              </Button>
              <Button
                onClick={() => this.setState({ hasError: false })}
                variant="secondary"
              >
                Try Again
              </Button>
            </div>
            {process.env.NODE_ENV === 'development' && this.state.error && (
              <details className="mt-6 text-left">
                <summary className="cursor-pointer text-sm text-gray-500">
                  Error Details
                </summary>
                <pre className="mt-2 text-xs bg-gray-100 dark:bg-gray-800 p-4 rounded overflow-auto">
                  {this.state.error.stack}
                </pre>
              </details>
            )}
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

// Micro-frontend specific error component
export const MicroFrontendError: React.FC<{ app: string }> = ({ app }) => (
  <div className="micro-frontend-error p-8 text-center">
    <div className="max-w-md mx-auto">
      <div className="text-yellow-500 text-4xl mb-4">🔧</div>
      <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
        {app} App Unavailable
      </h3>
      <p className="text-gray-600 dark:text-gray-400 mb-4">
        The {app} micro-frontend is currently unavailable. Please try again later.
      </p>
      <Button
        onClick={() => window.location.reload()}
        variant="secondary"
        size="sm"
      >
        Refresh
      </Button>
    </div>
  </div>
);
```

## Remote Application Setup

### Remote Application Configuration

```typescript
// apps/auth/vite.config.ts
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'auth_app',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './LoginForm': './src/components/LoginForm.tsx',
        './AuthProvider': './src/providers/AuthProvider.tsx',
        './routes': './src/routes.tsx',
      },
      shared: {
        react: {
          singleton: true,
          requiredVersion: '^18.2.0',
        },
        'react-dom': {
          singleton: true,
          requiredVersion: '^18.2.0',
        },
        '@tanstack/react-query': {
          singleton: true,
        },
        '@adx-core/design-system': {
          singleton: true,
        },
        '@adx-core/shared-context': {
          singleton: true,
        },
        '@adx-core/event-bus': {
          singleton: true,
        },
        '@adx-core/i18n': {
          singleton: true,
        },
        'react-router-dom': {
          singleton: true,
        },
      },
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
    rollupOptions: {
      external: ['react', 'react-dom'],
    },
  },
  server: {
    port: 3001,
    cors: true,
    headers: {
      'Access-Control-Allow-Origin': '*',
      'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
      'Access-Control-Allow-Headers': 'Content-Type, Authorization',
    },
  },
});
```

### Remote Application Structure

```typescript
// apps/auth/src/App.tsx
import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { useAuthContext } from '@adx-core/shared-context';
import { useEventBus } from '@adx-core/event-bus';
import { useTranslation } from '@adx-core/i18n';
import { LoginPage } from './pages/LoginPage';
import { RegisterPage } from './pages/RegisterPage';
import { ProfilePage } from './pages/ProfilePage';
import { SecurityPage } from './pages/SecurityPage';
import { MFASetupPage } from './pages/MFASetupPage';

export const App: React.FC = () => {
  const { user, isAuthenticated } = useAuthContext();
  const { emit } = useEventBus();
  const { t } = useTranslation('auth');

  // Emit auth events for other micro-frontends
  React.useEffect(() => {
    if (isAuthenticated && user) {
      emit('auth:user_authenticated', {
        userId: user.id,
        tenantId: user.tenantId,
        timestamp: new Date().toISOString(),
      });
    }
  }, [isAuthenticated, user, emit]);

  return (
    <div className="auth-app">
      <Routes>
        <Route path="/login" element={<LoginPage />} />
        <Route path="/register" element={<RegisterPage />} />
        <Route path="/profile" element={<ProfilePage />} />
        <Route path="/security" element={<SecurityPage />} />
        <Route path="/mfa-setup" element={<MFASetupPage />} />
        <Route path="/" element={<LoginPage />} />
      </Routes>
    </div>
  );
};

export default App;
```

### Standalone Development Mode

```typescript
// apps/auth/src/main.tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DesignSystemProvider } from '@adx-core/design-system';
import { EventBusProvider } from '@adx-core/event-bus';
import { I18nProvider } from '@adx-core/i18n';
import { AuthProvider } from '@adx-core/shared-context';
import App from './App';
import './index.css';

// Create query client for standalone mode
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000,
      cacheTime: 10 * 60 * 1000,
    },
  },
});

// Check if running in standalone mode
const isStandalone = !window.__POWERED_BY_QIANKUN__ && !window.__MICRO_APP_ENVIRONMENT__;

const StandaloneApp: React.FC = () => (
  <QueryClientProvider client={queryClient}>
    <I18nProvider>
      <AuthProvider>
        <DesignSystemProvider>
          <EventBusProvider>
            <BrowserRouter>
              <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
                <header className="bg-white dark:bg-gray-800 shadow">
                  <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                    <div className="flex justify-between h-16">
                      <div className="flex items-center">
                        <h1 className="text-xl font-semibold text-gray-900 dark:text-white">
                          Auth App - Standalone Mode
                        </h1>
                      </div>
                    </div>
                  </div>
                </header>
                <main className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
                  <App />
                </main>
              </div>
            </BrowserRouter>
          </EventBusProvider>
        </DesignSystemProvider>
      </AuthProvider>
    </I18nProvider>
  </QueryClientProvider>
);

const root = ReactDOM.createRoot(document.getElementById('root') as HTMLElement);

if (isStandalone) {
  root.render(<StandaloneApp />);
} else {
  // When loaded as micro-frontend, just render the App component
  root.render(<App />);
}
```

## Shared Dependencies Management

### Design System Integration

```typescript
// packages/design-system/src/index.ts
export { Button } from './components/Button';
export { Input } from './components/Input';
export { Modal } from './components/Modal';
export { Card } from './components/Card';
export { Table } from './components/Table';
export { Form } from './components/Form';
export { Navigation } from './components/Navigation';
export { Sidebar } from './components/Sidebar';
export { Header } from './components/Header';
export { Footer } from './components/Footer';

// Theme provider
export { DesignSystemProvider } from './providers/DesignSystemProvider';
export { useTheme } from './hooks/useTheme';

// Utilities
export { cn } from './utils/className';
export { formatDate, formatCurrency } from './utils/formatters';

// Types
export type { Theme, ComponentVariant, Size } from './types';
```

### Shared Context Management

```typescript
// packages/shared-context/src/index.ts
export { AuthProvider, useAuthContext } from './auth/AuthContext';
export { TenantProvider, useTenantContext } from './tenant/TenantContext';
export { ThemeProvider, useThemeContext } from './theme/ThemeContext';
export { NotificationProvider, useNotificationContext } from './notification/NotificationContext';

// Types
export type { 
  User, 
  Tenant, 
  AuthState, 
  TenantState,
  Theme,
  Notification 
} from './types';
```

### Event Bus System

```typescript
// packages/event-bus/src/EventBus.tsx
import React, { createContext, useContext, useRef, useCallback } from 'react';

type EventHandler = (data: any) => void;
type EventType = string;

interface EventBusContextType {
  emit: (eventType: EventType, data?: any) => void;
  subscribe: (eventType: EventType, handler: EventHandler) => () => void;
  subscribePattern: (pattern: string, handler: EventHandler) => () => void;
}

const EventBusContext = createContext<EventBusContextType | null>(null);

export const EventBusProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const listeners = useRef<Map<EventType, Set<EventHandler>>>(new Map());
  const patternListeners = useRef<Map<string, Set<EventHandler>>>(new Map());

  const emit = useCallback((eventType: EventType, data?: any) => {
    const event = {
      type: eventType,
      data,
      timestamp: Date.now(),
      source: 'micro-frontend',
    };

    // Emit to exact listeners
    const exactListeners = listeners.current.get(eventType);
    if (exactListeners) {
      exactListeners.forEach(handler => {
        try {
          handler(event);
        } catch (error) {
          console.error(`Error in event handler for ${eventType}:`, error);
        }
      });
    }

    // Emit to pattern listeners
    patternListeners.current.forEach((handlers, pattern) => {
      if (matchesPattern(eventType, pattern)) {
        handlers.forEach(handler => {
          try {
            handler(event);
          } catch (error) {
            console.error(`Error in pattern handler for ${pattern}:`, error);
          }
        });
      }
    });

    // Log event for debugging
    if (process.env.NODE_ENV === 'development') {
      console.log(`[EventBus] ${eventType}:`, data);
    }
  }, []);

  const subscribe = useCallback((eventType: EventType, handler: EventHandler) => {
    if (!listeners.current.has(eventType)) {
      listeners.current.set(eventType, new Set());
    }
    listeners.current.get(eventType)!.add(handler);

    return () => {
      const eventListeners = listeners.current.get(eventType);
      if (eventListeners) {
        eventListeners.delete(handler);
        if (eventListeners.size === 0) {
          listeners.current.delete(eventType);
        }
      }
    };
  }, []);

  const subscribePattern = useCallback((pattern: string, handler: EventHandler) => {
    if (!patternListeners.current.has(pattern)) {
      patternListeners.current.set(pattern, new Set());
    }
    patternListeners.current.get(pattern)!.add(handler);

    return () => {
      const patternHandlers = patternListeners.current.get(pattern);
      if (patternHandlers) {
        patternHandlers.delete(handler);
        if (patternHandlers.size === 0) {
          patternListeners.current.delete(pattern);
        }
      }
    };
  }, []);

  return (
    <EventBusContext.Provider value={{ emit, subscribe, subscribePattern }}>
      {children}
    </EventBusContext.Provider>
  );
};

export const useEventBus = () => {
  const context = useContext(EventBusContext);
  if (!context) {
    throw new Error('useEventBus must be used within EventBusProvider');
  }
  return context;
};

// Pattern matching utility
function matchesPattern(eventType: string, pattern: string): boolean {
  if (pattern.endsWith('*')) {
    const prefix = pattern.slice(0, -1);
    return eventType.startsWith(prefix);
  }
  return eventType === pattern;
}
```

## Communication Patterns

### Event-Driven Communication

```typescript
// Event types definition
export interface MicroFrontendEvents {
  // Authentication events
  'auth:user_login': {
    userId: string;
    tenantId: string;
    timestamp: string;
  };
  'auth:user_logout': {
    userId: string;
    timestamp: string;
  };
  'auth:session_expired': {
    userId: string;
    timestamp: string;
  };

  // Tenant events
  'tenant:switch': {
    previousTenantId: string;
    newTenantId: string;
    userId: string;
  };
  'tenant:updated': {
    tenantId: string;
    changes: Record<string, any>;
  };

  // File events
  'file:uploaded': {
    fileId: string;
    fileName: string;
    tenantId: string;
    uploadedBy: string;
  };
  'file:shared': {
    fileId: string;
    sharedWith: string[];
    permissions: string[];
  };

  // Workflow events
  'workflow:started': {
    workflowId: string;
    workflowType: string;
    tenantId: string;
  };
  'workflow:completed': {
    workflowId: string;
    result: any;
  };
  'workflow:failed': {
    workflowId: string;
    error: string;
  };

  // User events
  'user:profile_updated': {
    userId: string;
    changes: Record<string, any>;
  };
  'user:preferences_changed': {
    userId: string;
    preferences: Record<string, any>;
  };

  // Module events
  'module:installed': {
    moduleId: string;
    tenantId: string;
    installedBy: string;
  };
  'module:activated': {
    moduleId: string;
    tenantId: string;
  };
}

// Typed event bus hooks
export const useTypedEventBus = () => {
  const { emit, subscribe, subscribePattern } = useEventBus();

  const typedEmit = <K extends keyof MicroFrontendEvents>(
    eventType: K,
    data: MicroFrontendEvents[K]
  ) => {
    emit(eventType, data);
  };

  const typedSubscribe = <K extends keyof MicroFrontendEvents>(
    eventType: K,
    handler: (event: { type: K; data: MicroFrontendEvents[K]; timestamp: number }) => void
  ) => {
    return subscribe(eventType, handler);
  };

  return { emit: typedEmit, subscribe: typedSubscribe, subscribePattern };
};
```

### Cross-Micro-Frontend State Synchronization

```typescript
// Shared state synchronization hook
export const useCrossMicroFrontendSync = () => {
  const { emit, subscribe } = useTypedEventBus();
  const { user, updateUser } = useAuthContext();
  const { currentTenant, updateTenant } = useTenantContext();

  // Sync user data across micro-frontends
  React.useEffect(() => {
    const unsubscribe = subscribe('auth:user_login', (event) => {
      // Update local user context when user logs in from another micro-frontend
      if (event.data.userId !== user?.id) {
        // Fetch updated user data
        updateUser(event.data.userId);
      }
    });

    return unsubscribe;
  }, [user?.id, updateUser, subscribe]);

  // Sync tenant data across micro-frontends
  React.useEffect(() => {
    const unsubscribe = subscribe('tenant:switch', (event) => {
      // Update local tenant context when tenant switches from another micro-frontend
      if (event.data.newTenantId !== currentTenant?.id) {
        updateTenant(event.data.newTenantId);
      }
    });

    return unsubscribe;
  }, [currentTenant?.id, updateTenant, subscribe]);

  // Emit events when local state changes
  const emitUserUpdate = React.useCallback((changes: Record<string, any>) => {
    if (user) {
      emit('user:profile_updated', {
        userId: user.id,
        changes,
      });
    }
  }, [user, emit]);

  const emitTenantSwitch = React.useCallback((newTenantId: string) => {
    if (currentTenant) {
      emit('tenant:switch', {
        previousTenantId: currentTenant.id,
        newTenantId,
        userId: user?.id || '',
      });
    }
  }, [currentTenant, user, emit]);

  return {
    emitUserUpdate,
    emitTenantSwitch,
  };
};
```

## Development Workflow

### Local Development Setup

```bash
# 1. Start all micro-frontends in development mode
npm run dev:all

# This runs:
# - Shell app on port 3000
# - Auth app on port 3001
# - Tenant app on port 3002
# - File app on port 3003
# - User app on port 3004
# - Workflow app on port 3005
# - Module app on port 3006

# 2. Start individual micro-frontend for focused development
cd apps/auth
npm run dev

# 3. Start shell app with specific remote URLs
VITE_AUTH_APP_URL=http://localhost:3001/assets/remoteEntry.js npm run dev
```

### Package.json Scripts

```json
{
  "scripts": {
    "dev:all": "concurrently \"npm run dev:shell\" \"npm run dev:auth\" \"npm run dev:tenant\" \"npm run dev:file\" \"npm run dev:user\" \"npm run dev:workflow\" \"npm run dev:module\"",
    "dev:shell": "cd apps/shell && npm run dev",
    "dev:auth": "cd apps/auth && npm run dev",
    "dev:tenant": "cd apps/tenant && npm run dev",
    "dev:file": "cd apps/file && npm run dev",
    "dev:user": "cd apps/user && npm run dev",
    "dev:workflow": "cd apps/workflow && npm run dev",
    "dev:module": "cd apps/module && npm run dev",
    
    "build:all": "npm run build:shell && npm run build:remotes",
    "build:shell": "cd apps/shell && npm run build",
    "build:remotes": "concurrently \"npm run build:auth\" \"npm run build:tenant\" \"npm run build:file\" \"npm run build:user\" \"npm run build:workflow\" \"npm run build:module\"",
    "build:auth": "cd apps/auth && npm run build",
    "build:tenant": "cd apps/tenant && npm run build",
    "build:file": "cd apps/file && npm run build",
    "build:user": "cd apps/user && npm run build",
    "build:workflow": "cd apps/workflow && npm run build",
    "build:module": "cd apps/module && npm run build",
    
    "test:all": "concurrently \"npm run test:shell\" \"npm run test:remotes\"",
    "test:shell": "cd apps/shell && npm run test",
    "test:remotes": "concurrently \"npm run test:auth\" \"npm run test:tenant\" \"npm run test:file\" \"npm run test:user\" \"npm run test:workflow\" \"npm run test:module\"",
    "test:auth": "cd apps/auth && npm run test",
    "test:tenant": "cd apps/tenant && npm run test",
    "test:file": "cd apps/file && npm run test",
    "test:user": "cd apps/user && npm run test",
    "test:workflow": "cd apps/workflow && npm run test",
    "test:module": "cd apps/module && npm run test",
    
    "lint:all": "eslint apps/*/src --ext .ts,.tsx",
    "type-check:all": "concurrently \"npm run type-check:shell\" \"npm run type-check:remotes\"",
    "type-check:shell": "cd apps/shell && tsc --noEmit",
    "type-check:remotes": "concurrently \"cd apps/auth && tsc --noEmit\" \"cd apps/tenant && tsc --noEmit\" \"cd apps/file && tsc --noEmit\" \"cd apps/user && tsc --noEmit\" \"cd apps/workflow && tsc --noEmit\" \"cd apps/module && tsc --noEmit\""
  }
}
```

### Environment Configuration

```typescript
// Environment-specific remote URLs
// apps/shell/src/config/remotes.ts
interface RemoteConfig {
  url: string;
  fallbackUrl?: string;
}

interface RemotesConfig {
  [key: string]: RemoteConfig;
}

const getRemotesConfig = (): RemotesConfig => {
  const env = process.env.NODE_ENV;
  
  switch (env) {
    case 'development':
      return {
        auth_app: {
          url: 'http://localhost:3001/assets/remoteEntry.js',
        },
        tenant_app: {
          url: 'http://localhost:3002/assets/remoteEntry.js',
        },
        file_app: {
          url: 'http://localhost:3003/assets/remoteEntry.js',
        },
        user_app: {
          url: 'http://localhost:3004/assets/remoteEntry.js',
        },
        workflow_app: {
          url: 'http://localhost:3005/assets/remoteEntry.js',
        },
        module_app: {
          url: 'http://localhost:3006/assets/remoteEntry.js',
        },
      };
    
    case 'staging':
      return {
        auth_app: {
          url: 'https://auth-staging.adxcore.com/remoteEntry.js',
          fallbackUrl: 'https://auth-staging-backup.adxcore.com/remoteEntry.js',
        },
        tenant_app: {
          url: 'https://tenant-staging.adxcore.com/remoteEntry.js',
          fallbackUrl: 'https://tenant-staging-backup.adxcore.com/remoteEntry.js',
        },
        // ... other apps
      };
    
    case 'production':
      return {
        auth_app: {
          url: 'https://auth.adxcore.com/remoteEntry.js',
          fallbackUrl: 'https://auth-backup.adxcore.com/remoteEntry.js',
        },
        tenant_app: {
          url: 'https://tenant.adxcore.com/remoteEntry.js',
          fallbackUrl: 'https://tenant-backup.adxcore.com/remoteEntry.js',
        },
        // ... other apps
      };
    
    default:
      throw new Error(`Unknown environment: ${env}`);
  }
};

export const remotesConfig = getRemotesConfig();
```

## Testing Strategies

### Unit Testing for Micro-Frontends

```typescript
// apps/auth/src/components/__tests__/LoginForm.test.tsx
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { MemoryRouter } from 'react-router-dom';
import { DesignSystemProvider } from '@adx-core/design-system';
import { EventBusProvider } from '@adx-core/event-bus';
import { AuthProvider } from '@adx-core/shared-context';
import { LoginForm } from '../LoginForm';

// Test wrapper with all required providers
const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return (
    <QueryClientProvider client={queryClient}>
      <MemoryRouter>
        <DesignSystemProvider>
          <EventBusProvider>
            <AuthProvider>
              {children}
            </AuthProvider>
          </EventBusProvider>
        </DesignSystemProvider>
      </MemoryRouter>
    </QueryClientProvider>
  );
};

describe('LoginForm', () => {
  it('should render login form correctly', () => {
    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /login/i })).toBeInTheDocument();
  });

  it('should handle form submission', async () => {
    const mockLogin = jest.fn().mockResolvedValue({ success: true });
    
    render(
      <TestWrapper>
        <LoginForm onLogin={mockLogin} />
      </TestWrapper>
    );

    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: 'user@example.com' },
    });
    fireEvent.change(screen.getByLabelText(/password/i), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByRole('button', { name: /login/i }));

    await waitFor(() => {
      expect(mockLogin).toHaveBeenCalledWith({
        email: 'user@example.com',
        password: 'password123',
      });
    });
  });

  it('should emit login event on successful authentication', async () => {
    const mockEmit = jest.fn();
    
    // Mock event bus
    jest.mock('@adx-core/event-bus', () => ({
      useEventBus: () => ({ emit: mockEmit }),
    }));

    render(
      <TestWrapper>
        <LoginForm />
      </TestWrapper>
    );

    // Simulate successful login
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: 'user@example.com' },
    });
    fireEvent.change(screen.getByLabelText(/password/i), {
      target: { value: 'password123' },
    });
    fireEvent.click(screen.getByRole('button', { name: /login/i }));

    await waitFor(() => {
      expect(mockEmit).toHaveBeenCalledWith('auth:user_login', {
        userId: expect.any(String),
        tenantId: expect.any(String),
        timestamp: expect.any(String),
      });
    });
  });
});
```

### Integration Testing Across Micro-Frontends

```typescript
// tests/integration/cross-microfrontend.test.ts
import { test, expect } from '@playwright/test';

test.describe('Cross Micro-Frontend Integration', () => {
  test('should handle user login across all micro-frontends', async ({ page }) => {
    // Navigate to shell application
    await page.goto('http://localhost:3000');

    // Login through auth micro-frontend
    await page.click('[data-testid="login-button"]');
    await page.fill('[data-testid="email-input"]', 'user@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="submit-login"]');

    // Verify login success
    await expect(page.locator('[data-testid="user-menu"]')).toBeVisible();

    // Navigate to different micro-frontends and verify user context
    await page.goto('http://localhost:3000/files');
    await expect(page.locator('[data-testid="user-name"]')).toContainText('user@example.com');

    await page.goto('http://localhost:3000/users');
    await expect(page.locator('[data-testid="user-profile"]')).toBeVisible();

    await page.goto('http://localhost:3000/workflows');
    await expect(page.locator('[data-testid="user-workflows"]')).toBeVisible();
  });

  test('should handle tenant switching across micro-frontends', async ({ page }) => {
    // Login first
    await page.goto('http://localhost:3000/auth/login');
    await page.fill('[data-testid="email-input"]', 'user@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="submit-login"]');

    // Switch tenant
    await page.click('[data-testid="tenant-switcher"]');
    await page.selectOption('[data-testid="tenant-select"]', 'tenant-2');
    await page.click('[data-testid="confirm-switch"]');

    // Wait for tenant switch to complete
    await expect(page.locator('[data-testid="tenant-switch-complete"]')).toBeVisible();

    // Verify tenant context updated across micro-frontends
    await page.goto('http://localhost:3000/files');
    await expect(page.locator('[data-testid="current-tenant"]')).toContainText('Tenant 2');

    await page.goto('http://localhost:3000/users');
    await expect(page.locator('[data-testid="tenant-users"]')).toBeVisible();
    await expect(page.locator('[data-testid="current-tenant"]')).toContainText('Tenant 2');
  });

  test('should handle micro-frontend failures gracefully', async ({ page }) => {
    // Simulate micro-frontend failure by blocking network requests
    await page.route('**/auth/remoteEntry.js', route => route.abort());

    await page.goto('http://localhost:3000');

    // Verify error boundary is displayed
    await expect(page.locator('[data-testid="micro-frontend-error"]')).toBeVisible();
    await expect(page.locator('text=Auth App Unavailable')).toBeVisible();

    // Verify other micro-frontends still work
    await page.goto('http://localhost:3000/files');
    await expect(page.locator('[data-testid="file-list"]')).toBeVisible();
  });
});
```

### Performance Testing

```typescript
// tests/performance/module-federation.test.ts
import { test, expect } from '@playwright/test';

test.describe('Module Federation Performance', () => {
  test('should load micro-frontends within performance budget', async ({ page }) => {
    // Start performance monitoring
    await page.goto('http://localhost:3000');

    // Measure initial load time
    const navigationTiming = await page.evaluate(() => {
      return JSON.parse(JSON.stringify(performance.getEntriesByType('navigation')[0]));
    });

    // Verify initial load time is under 2 seconds
    expect(navigationTiming.loadEventEnd - navigationTiming.fetchStart).toBeLessThan(2000);

    // Measure micro-frontend switching time
    const startTime = Date.now();
    await page.click('[data-testid="nav-files"]');
    await page.waitForSelector('[data-testid="file-list"]');
    const switchTime = Date.now() - startTime;

    // Verify micro-frontend switch is under 500ms
    expect(switchTime).toBeLessThan(500);
  });

  test('should have acceptable bundle sizes', async ({ page }) => {
    // Navigate to each micro-frontend and measure resource sizes
    const resourceSizes = new Map();

    for (const app of ['auth', 'tenant', 'file', 'user', 'workflow', 'module']) {
      await page.goto(`http://localhost:3000/${app}`);
      
      const resources = await page.evaluate(() => {
        return performance.getEntriesByType('resource')
          .filter(entry => entry.name.includes('remoteEntry.js'))
          .map(entry => ({
            name: entry.name,
            size: entry.transferSize,
          }));
      });

      resources.forEach(resource => {
        resourceSizes.set(resource.name, resource.size);
      });
    }

    // Verify each micro-frontend is under 500KB
    resourceSizes.forEach((size, name) => {
      expect(size).toBeLessThan(500 * 1024); // 500KB
    });

    // Verify total shell application is under 2MB
    const shellResources = await page.evaluate(() => {
      return performance.getEntriesByType('resource')
        .filter(entry => entry.name.includes('localhost:3000'))
        .reduce((total, entry) => total + entry.transferSize, 0);
    });

    expect(shellResources).toBeLessThan(2 * 1024 * 1024); // 2MB
  });
});
```

## Deployment Strategies

### Independent Deployment Pipeline

```yaml
# .github/workflows/deploy-microfrontends.yml
name: Deploy Micro-Frontends

on:
  push:
    branches: [main]
    paths: ['apps/**']

jobs:
  detect-changes:
    runs-on: ubuntu-latest
    outputs:
      shell: ${{ steps.changes.outputs.shell }}
      auth: ${{ steps.changes.outputs.auth }}
      tenant: ${{ steps.changes.outputs.tenant }}
      file: ${{ steps.changes.outputs.file }}
      user: ${{ steps.changes.outputs.user }}
      workflow: ${{ steps.changes.outputs.workflow }}
      module: ${{ steps.changes.outputs.module }}
    steps:
      - uses: actions/checkout@v3
      - uses: dorny/paths-filter@v2
        id: changes
        with:
          filters: |
            shell:
              - 'apps/shell/**'
            auth:
              - 'apps/auth/**'
            tenant:
              - 'apps/tenant/**'
            file:
              - 'apps/file/**'
            user:
              - 'apps/user/**'
            workflow:
              - 'apps/workflow/**'
            module:
              - 'apps/module/**'

  deploy-shell:
    needs: detect-changes
    if: needs.detect-changes.outputs.shell == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
      
      - name: Install dependencies
        run: cd apps/shell && npm ci
      
      - name: Build shell application
        run: cd apps/shell && npm run build
        env:
          VITE_AUTH_APP_URL: https://auth.adxcore.com/remoteEntry.js
          VITE_TENANT_APP_URL: https://tenant.adxcore.com/remoteEntry.js
          VITE_FILE_APP_URL: https://files.adxcore.com/remoteEntry.js
          VITE_USER_APP_URL: https://users.adxcore.com/remoteEntry.js
          VITE_WORKFLOW_APP_URL: https://workflows.adxcore.com/remoteEntry.js
          VITE_MODULE_APP_URL: https://modules.adxcore.com/remoteEntry.js
      
      - name: Deploy to S3
        run: |
          aws s3 sync apps/shell/dist/ s3://adx-core-shell-production/
          aws cloudfront create-invalidation --distribution-id ${{ secrets.SHELL_CLOUDFRONT_ID }} --paths "/*"

  deploy-auth:
    needs: detect-changes
    if: needs.detect-changes.outputs.auth == 'true'
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
      
      - name: Install dependencies
        run: cd apps/auth && npm ci
      
      - name: Build auth micro-frontend
        run: cd apps/auth && npm run build
      
      - name: Deploy to S3
        run: |
          aws s3 sync apps/auth/dist/ s3://adx-core-auth-production/
          aws cloudfront create-invalidation --distribution-id ${{ secrets.AUTH_CLOUDFRONT_ID }} --paths "/*"

  # Similar jobs for other micro-frontends...

  update-remote-urls:
    needs: [deploy-shell, deploy-auth, deploy-tenant, deploy-file, deploy-user, deploy-workflow, deploy-module]
    if: always()
    runs-on: ubuntu-latest
    steps:
      - name: Update remote URLs in shell application
        run: |
          # Update environment configuration with new remote URLs
          # This could involve updating a configuration service or CDN
          echo "Updating remote URLs configuration"
```

### Blue-Green Deployment for Micro-Frontends

```bash
#!/bin/bash
# scripts/deploy-microfrontend-blue-green.sh

APP_NAME=$1
VERSION=$2
ENVIRONMENT=$3

if [ -z "$APP_NAME" ] || [ -z "$VERSION" ] || [ -z "$ENVIRONMENT" ]; then
  echo "Usage: $0 <app-name> <version> <environment>"
  exit 1
fi

# Build and deploy to green environment
echo "Deploying $APP_NAME version $VERSION to green environment..."

# Build the micro-frontend
cd apps/$APP_NAME
npm ci
npm run build

# Deploy to green S3 bucket
aws s3 sync dist/ s3://adx-core-$APP_NAME-$ENVIRONMENT-green/

# Update green CloudFront distribution
aws cloudfront create-invalidation \
  --distribution-id $(aws ssm get-parameter --name "/adx-core/$ENVIRONMENT/cloudfront/$APP_NAME-green" --query 'Parameter.Value' --output text) \
  --paths "/*"

# Health check green environment
echo "Performing health check on green environment..."
GREEN_URL="https://$APP_NAME-green.$ENVIRONMENT.adxcore.com"
if curl -f "$GREEN_URL/remoteEntry.js" > /dev/null 2>&1; then
  echo "Green environment health check passed"
else
  echo "Green environment health check failed"
  exit 1
fi

# Switch traffic to green
echo "Switching traffic to green environment..."
aws route53 change-resource-record-sets \
  --hosted-zone-id $(aws ssm get-parameter --name "/adx-core/$ENVIRONMENT/route53/zone-id" --query 'Parameter.Value' --output text) \
  --change-batch file://route53-switch-to-green.json

# Wait for DNS propagation
sleep 30

# Verify production traffic
echo "Verifying production traffic..."
PROD_URL="https://$APP_NAME.$ENVIRONMENT.adxcore.com"
if curl -f "$PROD_URL/remoteEntry.js" > /dev/null 2>&1; then
  echo "Production traffic verification passed"
  
  # Clean up blue environment
  echo "Cleaning up blue environment..."
  aws s3 rm s3://adx-core-$APP_NAME-$ENVIRONMENT-blue/ --recursive
  
  echo "Deployment completed successfully"
else
  echo "Production traffic verification failed, rolling back..."
  
  # Rollback to blue
  aws route53 change-resource-record-sets \
    --hosted-zone-id $(aws ssm get-parameter --name "/adx-core/$ENVIRONMENT/route53/zone-id" --query 'Parameter.Value' --output text) \
    --change-batch file://route53-switch-to-blue.json
  
  exit 1
fi
```

## Best Practices

### Development Best Practices

1. **Shared Dependencies Management**
   - Use singleton pattern for React, React-DOM, and other core libraries
   - Version shared dependencies carefully to avoid conflicts
   - Use workspace dependencies for internal packages

2. **Error Handling**
   - Implement error boundaries for each micro-frontend
   - Provide fallback UI for failed micro-frontends
   - Log errors to monitoring services with context

3. **Performance Optimization**
   - Lazy load micro-frontends only when needed
   - Implement proper caching strategies
   - Monitor bundle sizes and loading times
   - Use code splitting within micro-frontends

4. **Communication Patterns**
   - Use event bus for loose coupling between micro-frontends
   - Avoid direct imports between micro-frontends
   - Define clear event schemas and contracts

5. **Testing Strategy**
   - Test micro-frontends in isolation
   - Test integration between micro-frontends
   - Use contract testing for shared interfaces
   - Implement visual regression testing

### Production Best Practices

1. **Deployment Strategy**
   - Deploy micro-frontends independently
   - Use blue-green deployment for zero-downtime updates
   - Implement proper rollback procedures
   - Monitor deployment health and performance

2. **Monitoring and Observability**
   - Track micro-frontend loading times and errors
   - Monitor cross-micro-frontend communication
   - Set up alerts for failed micro-frontend loads
   - Use distributed tracing for debugging

3. **Security Considerations**
   - Implement proper CORS policies
   - Validate remote entry points
   - Use CSP headers to prevent XSS attacks
   - Regularly audit dependencies for vulnerabilities

4. **Performance Monitoring**
   - Set performance budgets for each micro-frontend
   - Monitor Core Web Vitals
   - Track resource loading and caching effectiveness
   - Optimize for mobile and slow networks

## Troubleshooting

### Common Issues and Solutions

#### Micro-Frontend Won't Load
```typescript
// Debug micro-frontend loading issues
const debugMicroFrontendLoading = async (remoteName: string, remoteUrl: string) => {
  try {
    // Check if remote entry is accessible
    const response = await fetch(remoteUrl);
    if (!response.ok) {
      console.error(`Remote entry not accessible: ${remoteUrl}`);
      return false;
    }

    // Check if remote is properly exposed
    const remoteScript = await response.text();
    if (!remoteScript.includes(remoteName)) {
      console.error(`Remote name ${remoteName} not found in remote entry`);
      return false;
    }

    console.log(`Remote ${remoteName} loaded successfully`);
    return true;
  } catch (error) {
    console.error(`Error loading remote ${remoteName}:`, error);
    return false;
  }
};

// Usage
debugMicroFrontendLoading('auth_app', 'http://localhost:3001/assets/remoteEntry.js');
```

#### Shared Dependency Version Conflicts
```typescript
// Check for version conflicts in shared dependencies
const checkSharedDependencies = () => {
  const sharedDeps = {
    react: window.React?.version,
    'react-dom': window.ReactDOM?.version,
    '@tanstack/react-query': window.ReactQuery?.version,
  };

  console.log('Shared dependency versions:', sharedDeps);

  // Check for version mismatches
  Object.entries(sharedDeps).forEach(([dep, version]) => {
    if (!version) {
      console.warn(`Shared dependency ${dep} not found`);
    }
  });
};
```

#### Event Bus Communication Issues
```typescript
// Debug event bus communication
const debugEventBus = () => {
  const { emit, subscribe } = useEventBus();

  // Log all events
  const unsubscribe = subscribe('*', (event) => {
    console.log('[EventBus Debug]', event);
  });

  // Test event emission
  const testEvent = () => {
    emit('debug:test', { timestamp: Date.now() });
  };

  return { testEvent, unsubscribe };
};
```

### Performance Debugging

```typescript
// Monitor micro-frontend performance
const monitorMicroFrontendPerformance = () => {
  // Track micro-frontend loading times
  const observer = new PerformanceObserver((list) => {
    list.getEntries().forEach((entry) => {
      if (entry.name.includes('remoteEntry.js')) {
        console.log(`Micro-frontend load time: ${entry.name} - ${entry.duration}ms`);
        
        // Send to analytics
        if (window.analytics) {
          window.analytics.track('Micro-Frontend Load Time', {
            name: entry.name,
            duration: entry.duration,
            size: entry.transferSize,
          });
        }
      }
    });
  });

  observer.observe({ entryTypes: ['resource'] });

  return () => observer.disconnect();
};

// Monitor bundle sizes
const monitorBundleSizes = () => {
  const resources = performance.getEntriesByType('resource');
  const bundleSizes = resources
    .filter(entry => entry.name.includes('remoteEntry.js'))
    .map(entry => ({
      name: entry.name,
      size: entry.transferSize,
      compressed: entry.encodedBodySize,
    }));

  console.table(bundleSizes);
  return bundleSizes;
};
```

## Conclusion

Module Federation enables ADX CORE to achieve true micro-frontend architecture with independent development, deployment, and scaling. By following these guidelines and best practices, teams can build maintainable, performant, and reliable micro-frontends that work together seamlessly.

Key benefits of this approach:
- **Team Autonomy**: Each team can work independently on their domain
- **Technology Flexibility**: Teams can choose the best tools for their needs
- **Independent Deployment**: Deploy micro-frontends without affecting others
- **Fault Isolation**: Failures in one micro-frontend don't affect others
- **Performance Optimization**: Load only what's needed when it's needed

For more information, see:
- [Team Autonomy Guide](./team-autonomy.md)
- [Frontend Architecture Guide](./frontend-architecture.md)
- [Testing Strategies](./testing-strategies.md)
- [Deployment Guide](../deployment/README.md)