# ADX CORE Frontend Microservices Development Guidelines

## Core Principles

ADX CORE frontend follows a microservices architecture using Module Federation, where each micro-frontend mirrors backend service boundaries and can be developed, tested, and deployed independently while maintaining a cohesive user experience.

## Architecture Overview

### Shell Application Pattern
```typescript
// Shell application - orchestrates all micro-frontends
// apps/shell/src/App.tsx
import React, { Suspense } from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TenantProvider } from '@adx-core/shared-context';
import { DesignSystemProvider } from '@adx-core/design-system';
import { EventBusProvider } from '@adx-core/event-bus';

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
    },
  },
});

export const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <TenantProvider>
        <DesignSystemProvider>
          <EventBusProvider>
            <BrowserRouter>
              <div className="app-shell">
                <Navigation />
                <main className="main-content">
                  <Suspense fallback={<MicroFrontendLoader />}>
                    <Routes>
                      <Route path="/auth/*" element={<AuthApp />} />
                      <Route path="/tenant/*" element={<TenantApp />} />
                      <Route path="/files/*" element={<FileApp />} />
                      <Route path="/users/*" element={<UserApp />} />
                      <Route path="/workflows/*" element={<WorkflowApp />} />
                      <Route path="/modules/*" element={<ModuleApp />} />
                      <Route path="/" element={<Dashboard />} />
                    </Routes>
                  </Suspense>
                </main>
              </div>
            </BrowserRouter>
          </EventBusProvider>
        </DesignSystemProvider>
      </TenantProvider>
    </QueryClientProvider>
  );
};
```

### Module Federation Configuration
```typescript
// vite.config.ts for Shell Application
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'shell_app',
      remotes: {
        auth_app: 'http://localhost:3001/assets/remoteEntry.js',
        tenant_app: 'http://localhost:3002/assets/remoteEntry.js',
        file_app: 'http://localhost:3003/assets/remoteEntry.js',
        user_app: 'http://localhost:3004/assets/remoteEntry.js',
        workflow_app: 'http://localhost:3005/assets/remoteEntry.js',
        module_app: 'http://localhost:3006/assets/remoteEntry.js',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        '@tanstack/react-query': { singleton: true },
        '@adx-core/design-system': { singleton: true },
        '@adx-core/shared-context': { singleton: true },
        '@adx-core/event-bus': { singleton: true },
      },
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
  },
});
```

### Micro-Frontend Configuration
```typescript
// vite.config.ts for Auth Micro-Frontend
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
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        '@tanstack/react-query': { singleton: true },
        '@adx-core/design-system': { singleton: true },
        '@adx-core/shared-context': { singleton: true },
        '@adx-core/event-bus': { singleton: true },
      },
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
  },
  server: {
    port: 3001,
    cors: true,
  },
});
```

## Shared Context and State Management

### Tenant Context Provider
```typescript
// packages/shared-context/src/TenantContext.tsx
import React, { createContext, useContext, useReducer, useEffect } from 'react';
import { useEventBus } from '@adx-core/event-bus';

interface TenantState {
  currentTenant: Tenant | null;
  availableTenants: Tenant[];
  loading: boolean;
  error: string | null;
}

interface Tenant {
  id: string;
  name: string;
  features: string[];
  quotas: Record<string, QuotaInfo>;
  settings: TenantSettings;
}

const TenantContext = createContext<{
  state: TenantState;
  switchTenant: (tenantId: string) => Promise<void>;
  refreshTenantData: () => Promise<void>;
} | null>(null);

export const TenantProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const [state, dispatch] = useReducer(tenantReducer, initialState);
  const { emit, subscribe } = useEventBus();

  const switchTenant = async (tenantId: string) => {
    dispatch({ type: 'SWITCH_TENANT_START' });
    
    try {
      // Call BFF or API Gateway for tenant switch
      const response = await fetch('/api/workflows/switch-tenant', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${getAuthToken()}`,
        },
        body: JSON.stringify({
          targetTenantId: tenantId,
          currentTenantId: state.currentTenant?.id,
        }),
      });

      if (response.ok) {
        const result = await response.json();
        
        // Handle workflow response
        if (result.operationId) {
          await pollWorkflowStatus(result.operationId);
        }
        
        // Update tenant context
        dispatch({ 
          type: 'SWITCH_TENANT_SUCCESS', 
          payload: result.tenantContext 
        });
        
        // Emit tenant switch event for other micro-frontends
        emit('tenant:switched', { 
          previousTenantId: state.currentTenant?.id,
          newTenantId: tenantId,
          tenantContext: result.tenantContext,
        });
      }
    } catch (error) {
      dispatch({ 
        type: 'SWITCH_TENANT_ERROR', 
        payload: error.message 
      });
    }
  };

  // Subscribe to tenant-related events
  useEffect(() => {
    const unsubscribe = subscribe('tenant:*', (event) => {
      // Handle tenant events from other micro-frontends
      switch (event.type) {
        case 'tenant:quota_updated':
          dispatch({ type: 'UPDATE_QUOTA', payload: event.data });
          break;
        case 'tenant:feature_toggled':
          dispatch({ type: 'TOGGLE_FEATURE', payload: event.data });
          break;
      }
    });

    return unsubscribe;
  }, [subscribe]);

  return (
    <TenantContext.Provider value={{ state, switchTenant, refreshTenantData }}>
      {children}
    </TenantContext.Provider>
  );
};

export const useTenantContext = () => {
  const context = useContext(TenantContext);
  if (!context) {
    throw new Error('useTenantContext must be used within TenantProvider');
  }
  return context;
};
```

### Event Bus for Micro-Frontend Communication
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
    // Emit to exact listeners
    const exactListeners = listeners.current.get(eventType);
    if (exactListeners) {
      exactListeners.forEach(handler => {
        try {
          handler({ type: eventType, data, timestamp: Date.now() });
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
            handler({ type: eventType, data, timestamp: Date.now() });
          } catch (error) {
            console.error(`Error in pattern handler for ${pattern}:`, error);
          }
        });
      }
    });
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

## BFF Integration Pattern

### Micro-Frontend BFF Client
```typescript
// packages/bff-client/src/BFFClient.ts
export class MicroFrontendBFFClient {
  private baseUrl: string;
  private tenantId: string;
  private authToken: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  setTenantContext(tenantId: string, authToken: string) {
    this.tenantId = tenantId;
    this.authToken = authToken;
  }

  // Workflow-aware API calls
  async initiateWorkflow<T>(
    workflowType: string,
    request: any,
    options: { synchronous?: boolean } = {}
  ): Promise<WorkflowResponse<T>> {
    const response = await fetch(`${this.baseUrl}/workflows/${workflowType}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.authToken}`,
        'X-Tenant-ID': this.tenantId,
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Workflow initiation failed: ${response.statusText}`);
    }

    const result = await response.json();

    if (result.operationId && !options.synchronous) {
      // Long-running workflow - return operation tracking
      return {
        type: 'async',
        operationId: result.operationId,
        statusUrl: result.statusUrl,
        streamUrl: result.streamUrl,
      };
    } else {
      // Synchronous result
      return {
        type: 'sync',
        data: result.data || result,
      };
    }
  }

  // Aggregated data fetching
  async getAggregatedData<T>(
    endpoint: string,
    options: { cache?: boolean; ttl?: number } = {}
  ): Promise<T> {
    const cacheKey = options.cache ? `${this.tenantId}:${endpoint}` : null;
    
    if (cacheKey) {
      const cached = this.getFromCache<T>(cacheKey);
      if (cached) return cached;
    }

    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
        'X-Tenant-ID': this.tenantId,
        'X-Cache-TTL': options.ttl?.toString() || '300',
      },
    });

    if (!response.ok) {
      throw new Error(`BFF request failed: ${response.statusText}`);
    }

    const data = await response.json();

    if (cacheKey) {
      this.setCache(cacheKey, data, options.ttl || 300);
    }

    return data;
  }

  // Workflow status polling
  async pollWorkflowStatus(
    operationId: string,
    onProgress?: (progress: WorkflowProgress) => void
  ): Promise<any> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const response = await fetch(`${this.baseUrl}/workflows/${operationId}/status`, {
            headers: {
              'Authorization': `Bearer ${this.authToken}`,
              'X-Tenant-ID': this.tenantId,
            },
          });

          if (!response.ok) {
            throw new Error(`Status check failed: ${response.statusText}`);
          }

          const status = await response.json();

          if (onProgress) {
            onProgress(status.progress);
          }

          switch (status.status) {
            case 'completed':
              resolve(status.result);
              break;
            case 'failed':
              reject(new Error(status.error || 'Workflow failed'));
              break;
            case 'running':
            case 'pending':
              setTimeout(poll, 1000); // Poll every second
              break;
            default:
              reject(new Error(`Unknown workflow status: ${status.status}`));
          }
        } catch (error) {
          reject(error);
        }
      };

      poll();
    });
  }

  private getFromCache<T>(key: string): T | null {
    const cached = localStorage.getItem(`bff_cache_${key}`);
    if (!cached) return null;

    const { data, expiry } = JSON.parse(cached);
    if (Date.now() > expiry) {
      localStorage.removeItem(`bff_cache_${key}`);
      return null;
    }

    return data;
  }

  private setCache(key: string, data: any, ttlSeconds: number) {
    const expiry = Date.now() + (ttlSeconds * 1000);
    localStorage.setItem(`bff_cache_${key}`, JSON.stringify({ data, expiry }));
  }
}
```

### React Query Integration with BFF
```typescript
// packages/bff-client/src/hooks/useBFFQuery.ts
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTenantContext } from '@adx-core/shared-context';
import { MicroFrontendBFFClient } from '../BFFClient';

export const useBFFQuery = <T>(
  queryKey: string[],
  endpoint: string,
  options: {
    enabled?: boolean;
    staleTime?: number;
    cacheTime?: number;
    refetchOnWindowFocus?: boolean;
  } = {}
) => {
  const { state: tenantState } = useTenantContext();
  const bffClient = new MicroFrontendBFFClient(getBFFUrl());

  return useQuery({
    queryKey: [tenantState.currentTenant?.id, ...queryKey],
    queryFn: async () => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      bffClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return bffClient.getAggregatedData<T>(endpoint, {
        cache: true,
        ttl: options.staleTime ? options.staleTime / 1000 : 300,
      });
    },
    enabled: options.enabled !== false && !!tenantState.currentTenant,
    staleTime: options.staleTime || 5 * 60 * 1000,
    cacheTime: options.cacheTime || 10 * 60 * 1000,
    refetchOnWindowFocus: options.refetchOnWindowFocus !== false,
  });
};

export const useBFFWorkflow = <TRequest, TResponse>(
  workflowType: string,
  options: {
    onSuccess?: (data: TResponse) => void;
    onError?: (error: Error) => void;
    synchronous?: boolean;
  } = {}
) => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();
  const bffClient = new MicroFrontendBFFClient(getBFFUrl());

  return useMutation({
    mutationFn: async (request: TRequest) => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      bffClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      const response = await bffClient.initiateWorkflow<TResponse>(
        workflowType,
        request,
        { synchronous: options.synchronous }
      );

      if (response.type === 'async') {
        // Poll for completion
        return bffClient.pollWorkflowStatus(response.operationId);
      } else {
        return response.data;
      }
    },
    onSuccess: (data) => {
      // Invalidate related queries
      queryClient.invalidateQueries([tenantState.currentTenant?.id]);
      options.onSuccess?.(data);
    },
    onError: options.onError,
  });
};
```

## Cross-Platform Integration with Tauri

### Tauri Configuration
```json
// src-tauri/tauri.conf.json
{
  "build": {
    "beforeDevCommand": "npm run dev:all",
    "beforeBuildCommand": "npm run build:all",
    "devPath": "http://localhost:3000",
    "distDir": "../dist"
  },
  "package": {
    "productName": "ADX Core",
    "version": "2.0.0"
  },
  "tauri": {
    "allowlist": {
      "all": false,
      "shell": {
        "all": false,
        "open": true
      },
      "window": {
        "all": false,
        "close": true,
        "hide": true,
        "show": true,
        "maximize": true,
        "minimize": true,
        "unmaximize": true,
        "unminimize": true,
        "startDragging": true
      },
      "fs": {
        "all": false,
        "readFile": true,
        "writeFile": true,
        "readDir": true,
        "copyFile": true,
        "createDir": true,
        "removeDir": true,
        "removeFile": true,
        "renameFile": true
      },
      "notification": {
        "all": true
      },
      "globalShortcut": {
        "all": true
      }
    },
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.adxcore.app",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/128x128@2x.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    },
    "security": {
      "csp": null
    },
    "windows": [
      {
        "fullscreen": false,
        "resizable": true,
        "title": "ADX Core",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600
      }
    ]
  }
}
```

### Native Integration Hooks
```typescript
// packages/native-integration/src/hooks/useNativeFeatures.ts
import { invoke } from '@tauri-apps/api/tauri';
import { sendNotification } from '@tauri-apps/api/notification';
import { open } from '@tauri-apps/api/shell';
import { readTextFile, writeTextFile } from '@tauri-apps/api/fs';

export const useNativeFeatures = () => {
  const showNotification = async (title: string, body: string) => {
    if (window.__TAURI__) {
      await sendNotification({ title, body });
    } else {
      // Web fallback
      if ('Notification' in window && Notification.permission === 'granted') {
        new Notification(title, { body });
      }
    }
  };

  const openExternal = async (url: string) => {
    if (window.__TAURI__) {
      await open(url);
    } else {
      window.open(url, '_blank');
    }
  };

  const saveFile = async (path: string, content: string) => {
    if (window.__TAURI__) {
      await writeTextFile(path, content);
    } else {
      // Web fallback - download file
      const blob = new Blob([content], { type: 'text/plain' });
      const url = URL.createObjectURL(blob);
      const a = document.createElement('a');
      a.href = url;
      a.download = path.split('/').pop() || 'file.txt';
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  const readFile = async (path: string): Promise<string> => {
    if (window.__TAURI__) {
      return await readTextFile(path);
    } else {
      throw new Error('File reading not supported in web mode');
    }
  };

  return {
    showNotification,
    openExternal,
    saveFile,
    readFile,
    isNative: !!window.__TAURI__,
  };
};
```

## Testing Strategies

### Micro-Frontend Unit Testing
```typescript
// apps/auth/src/components/__tests__/LoginForm.test.tsx
import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TenantProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';
import { LoginForm } from '../LoginForm';

const TestWrapper: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return (
    <QueryClientProvider client={queryClient}>
      <TenantProvider>
        <EventBusProvider>
          {children}
        </EventBusProvider>
      </TenantProvider>
    </QueryClientProvider>
  );
};

describe('LoginForm', () => {
  it('should handle login workflow correctly', async () => {
    // Mock BFF client
    const mockBFFClient = {
      initiateWorkflow: jest.fn().mockResolvedValue({
        type: 'sync',
        data: { token: 'test-token', user: { id: '1', email: 'test@example.com' } },
      }),
    };

    render(
      <TestWrapper>
        <LoginForm bffClient={mockBFFClient} />
      </TestWrapper>
    );

    // Fill in form
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByLabelText(/password/i), {
      target: { value: 'password123' },
    });

    // Submit form
    fireEvent.click(screen.getByRole('button', { name: /login/i }));

    // Wait for workflow to complete
    await waitFor(() => {
      expect(mockBFFClient.initiateWorkflow).toHaveBeenCalledWith(
        'user_login',
        {
          email: 'test@example.com',
          password: 'password123',
        },
        { synchronous: true }
      );
    });
  });
});
```

### Integration Testing Across Micro-Frontends
```typescript
// tests/integration/cross-microfrontend.test.ts
import { test, expect } from '@playwright/test';

test.describe('Cross Micro-Frontend Integration', () => {
  test('should handle tenant switching across all micro-frontends', async ({ page }) => {
    // Navigate to shell application
    await page.goto('http://localhost:3000');

    // Login
    await page.fill('[data-testid="email-input"]', 'test@example.com');
    await page.fill('[data-testid="password-input"]', 'password123');
    await page.click('[data-testid="login-button"]');

    // Wait for dashboard to load
    await page.waitForSelector('[data-testid="dashboard"]');

    // Switch tenant
    await page.selectOption('[data-testid="tenant-switcher"]', 'tenant-2');

    // Wait for tenant switch workflow to complete
    await page.waitForSelector('[data-testid="tenant-switch-complete"]');

    // Verify all micro-frontends updated
    await page.goto('http://localhost:3000/files');
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('Tenant 2');

    await page.goto('http://localhost:3000/users');
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('Tenant 2');

    await page.goto('http://localhost:3000/workflows');
    await expect(page.locator('[data-testid="current-tenant"]')).toHaveText('Tenant 2');
  });

  test('should handle workflow status updates across micro-frontends', async ({ page }) => {
    await page.goto('http://localhost:3000');

    // Start a long-running workflow from one micro-frontend
    await page.goto('http://localhost:3000/files');
    await page.click('[data-testid="upload-large-file"]');

    // Navigate to workflows micro-frontend
    await page.goto('http://localhost:3000/workflows');

    // Verify workflow appears in workflow list
    await expect(page.locator('[data-testid="workflow-list"]')).toContainText('File Upload');

    // Navigate back to files micro-frontend
    await page.goto('http://localhost:3000/files');

    // Verify progress is shown
    await expect(page.locator('[data-testid="upload-progress"]')).toBeVisible();
  });
});
```

## Performance Optimization

### Bundle Optimization
```typescript
// webpack.config.js - Bundle analysis and optimization
const BundleAnalyzerPlugin = require('webpack-bundle-analyzer').BundleAnalyzerPlugin;

module.exports = {
  optimization: {
    splitChunks: {
      chunks: 'all',
      cacheGroups: {
        vendor: {
          test: /[\\/]node_modules[\\/]/,
          name: 'vendors',
          chunks: 'all',
        },
        shared: {
          test: /[\\/]packages[\\/]/,
          name: 'shared',
          chunks: 'all',
          minChunks: 2,
        },
      },
    },
  },
  plugins: [
    process.env.ANALYZE && new BundleAnalyzerPlugin(),
  ].filter(Boolean),
};
```

### Lazy Loading and Code Splitting
```typescript
// Lazy loading for micro-frontend components
const LazyComponent = React.lazy(() => 
  import('./HeavyComponent').then(module => ({
    default: module.HeavyComponent
  }))
);

// Route-based code splitting
const routes = [
  {
    path: '/dashboard',
    component: React.lazy(() => import('./pages/Dashboard')),
  },
  {
    path: '/settings',
    component: React.lazy(() => import('./pages/Settings')),
  },
];
```

## Development Guidelines

### Micro-Frontend Development Best Practices
1. **Independent Deployability**: Each micro-frontend should be deployable independently
2. **Shared Dependencies**: Use Module Federation to share common dependencies
3. **Event-Driven Communication**: Use event bus for cross-micro-frontend communication
4. **Consistent Design**: Use shared design system for consistent UI/UX
5. **Error Boundaries**: Implement error boundaries to isolate failures
6. **Performance Budgets**: Set and monitor performance budgets for each micro-frontend
7. **Testing Isolation**: Test micro-frontends in isolation and integration

### Team Ownership Model
1. **Vertical Slices**: Each team owns backend service + micro-frontend + optional BFF
2. **API Contracts**: Define clear API contracts between micro-frontends
3. **Shared Libraries**: Contribute to shared libraries for common functionality
4. **Design System**: Follow shared design system guidelines
5. **Event Contracts**: Define clear event contracts for cross-micro-frontend communication
6. **Documentation**: Maintain documentation for micro-frontend APIs and events
7. **Monitoring**: Monitor micro-frontend performance and errors independently

This frontend microservices architecture provides the foundation for scalable, maintainable, and independently deployable frontend applications while maintaining a cohesive user experience.