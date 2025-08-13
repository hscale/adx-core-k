import React, { Suspense, useEffect } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import { useAuthStore, useTenantStore, useThemeStore, SubscriptionTier } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';
import { Navigation } from './components/Navigation';
import { Dashboard } from './components/Dashboard';
import { ErrorBoundary } from './components/ErrorBoundary';
import { MicroFrontendLoader } from './components/MicroFrontendLoader';
import './i18n';

// Dynamic imports for micro-frontends
const AuthApp = React.lazy(() => 
  import('auth_app/App').catch(() => ({
    default: () => (
      <div className="p-8 text-center">
        <h2 className="text-xl font-semibold mb-4">Auth Service Unavailable</h2>
        <p className="text-gray-600 dark:text-gray-400">
          The authentication service is currently unavailable. Please try again later.
        </p>
      </div>
    )
  }))
);

const TenantApp = React.lazy(() => 
  import('tenant_app/App').catch(() => ({
    default: () => (
      <div className="p-8 text-center">
        <h2 className="text-xl font-semibold mb-4">Tenant Service Unavailable</h2>
        <p className="text-gray-600 dark:text-gray-400">
          The tenant management service is currently unavailable. Please try again later.
        </p>
      </div>
    )
  }))
);

const FileApp = React.lazy(() => 
  import('file_app/App').catch(() => ({
    default: () => (
      <div className="p-8 text-center">
        <h2 className="text-xl font-semibold mb-4">File Service Unavailable</h2>
        <p className="text-gray-600 dark:text-gray-400">
          The file management service is currently unavailable. Please try again later.
        </p>
      </div>
    )
  }))
);

const UserApp = React.lazy(() => 
  import('user_app/App').catch(() => ({
    default: () => (
      <div className="p-8 text-center">
        <h2 className="text-xl font-semibold mb-4">User Service Unavailable</h2>
        <p className="text-gray-600 dark:text-gray-400">
          The user management service is currently unavailable. Please try again later.
        </p>
      </div>
    )
  }))
);

const WorkflowApp = React.lazy(() => 
  import('workflow_app/App').catch(() => ({
    default: () => (
      <div className="p-8 text-center">
        <h2 className="text-xl font-semibold mb-4">Workflow Service Unavailable</h2>
        <p className="text-gray-600 dark:text-gray-400">
          The workflow management service is currently unavailable. Please try again later.
        </p>
      </div>
    )
  }))
);

const ModuleApp = React.lazy(() => 
  import('module_app/App').catch(() => ({
    default: () => (
      <div className="p-8 text-center">
        <h2 className="text-xl font-semibold mb-4">Module Service Unavailable</h2>
        <p className="text-gray-600 dark:text-gray-400">
          The module management service is currently unavailable. Please try again later.
        </p>
      </div>
    )
  }))
);

// Create React Query client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes (formerly cacheTime)
      retry: (failureCount, error: any) => {
        // Don't retry on 4xx errors
        if (error?.status >= 400 && error?.status < 500) {
          return false;
        }
        return failureCount < 3;
      },
    },
    mutations: {
      retry: 1,
    },
  },
});

// Protected Route component
const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { isAuthenticated, isLoading } = useAuthStore();

  if (isLoading) {
    return <MicroFrontendLoader message="Checking authentication..." />;
  }

  if (!isAuthenticated) {
    return <Navigate to="/auth/login" replace />;
  }

  return <>{children}</>;
};

// Main App component
export const App: React.FC = () => {
  const { isAuthenticated } = useAuthStore();
  const { currentTenant, setAvailableTenants } = useTenantStore();
  const { setTheme } = useThemeStore();

  // Initialize theme on app start
  useEffect(() => {
    const savedTheme = localStorage.getItem('adx-theme-storage');
    if (savedTheme) {
      const parsed = JSON.parse(savedTheme);
      if (parsed.state?.theme) {
        setTheme(parsed.state.theme);
      }
    }
  }, [setTheme]);

  // Load tenant data when authenticated
  useEffect(() => {
    if (isAuthenticated && !currentTenant) {
      // Mock tenant data - in real app this would come from API
      const mockTenants = [
        {
          id: 'tenant-1',
          name: 'Acme Corporation',
          slug: 'acme-corp',
          features: ['advanced_analytics', 'custom_workflows'],
          quotas: {
            users: { used: 45, limit: 100, unit: 'users' },
            storage: { used: 2.5, limit: 10, unit: 'GB' },
            api_calls: { used: 8500, limit: 10000, unit: 'calls/month' },
          },
          settings: {
            theme: 'system' as const,
            language: 'en',
            timezone: 'UTC',
            dateFormat: 'MM/dd/yyyy',
            currency: 'USD',
          },
          subscriptionTier: SubscriptionTier.Professional,
        },
        {
          id: 'tenant-2',
          name: 'Beta Industries',
          slug: 'beta-industries',
          features: ['basic_analytics'],
          quotas: {
            users: { used: 12, limit: 25, unit: 'users' },
            storage: { used: 0.8, limit: 5, unit: 'GB' },
            api_calls: { used: 2100, limit: 5000, unit: 'calls/month' },
          },
          settings: {
            theme: 'light' as const,
            language: 'en',
            timezone: 'UTC',
            dateFormat: 'MM/dd/yyyy',
            currency: 'USD',
          },
          subscriptionTier: SubscriptionTier.Free,
        },
      ];

      setAvailableTenants(mockTenants);
      
      // Set first tenant as current if none selected
      if (!currentTenant) {
        useTenantStore.getState().setCurrentTenant(mockTenants[0]);
      }
    }
  }, [isAuthenticated, currentTenant, setAvailableTenants]);

  return (
    <QueryClientProvider client={queryClient}>
      <EventBusProvider>
        <BrowserRouter>
          <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
            <ErrorBoundary microFrontendName="Shell Application">
              {isAuthenticated && <Navigation />}
              
              <main className={isAuthenticated ? "max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8" : ""}>
                <Suspense fallback={<MicroFrontendLoader />}>
                  <Routes>
                    {/* Public routes */}
                    <Route 
                      path="/auth/*" 
                      element={
                        <ErrorBoundary microFrontendName="Auth Service">
                          <AuthApp />
                        </ErrorBoundary>
                      } 
                    />

                    {/* Protected routes */}
                    <Route 
                      path="/" 
                      element={
                        <ProtectedRoute>
                          <Dashboard />
                        </ProtectedRoute>
                      } 
                    />
                    
                    <Route 
                      path="/tenant/*" 
                      element={
                        <ProtectedRoute>
                          <ErrorBoundary microFrontendName="Tenant Service">
                            <TenantApp />
                          </ErrorBoundary>
                        </ProtectedRoute>
                      } 
                    />
                    
                    <Route 
                      path="/files/*" 
                      element={
                        <ProtectedRoute>
                          <ErrorBoundary microFrontendName="File Service">
                            <FileApp />
                          </ErrorBoundary>
                        </ProtectedRoute>
                      } 
                    />
                    
                    <Route 
                      path="/users/*" 
                      element={
                        <ProtectedRoute>
                          <ErrorBoundary microFrontendName="User Service">
                            <UserApp />
                          </ErrorBoundary>
                        </ProtectedRoute>
                      } 
                    />
                    
                    <Route 
                      path="/workflows/*" 
                      element={
                        <ProtectedRoute>
                          <ErrorBoundary microFrontendName="Workflow Service">
                            <WorkflowApp />
                          </ErrorBoundary>
                        </ProtectedRoute>
                      } 
                    />
                    
                    <Route 
                      path="/modules/*" 
                      element={
                        <ProtectedRoute>
                          <ErrorBoundary microFrontendName="Module Service">
                            <ModuleApp />
                          </ErrorBoundary>
                        </ProtectedRoute>
                      } 
                    />

                    {/* Fallback route */}
                    <Route 
                      path="*" 
                      element={
                        isAuthenticated ? (
                          <Navigate to="/" replace />
                        ) : (
                          <Navigate to="/auth/login" replace />
                        )
                      } 
                    />
                  </Routes>
                </Suspense>
              </main>
            </ErrorBoundary>
          </div>
        </BrowserRouter>
      </EventBusProvider>
    </QueryClientProvider>
  );
};