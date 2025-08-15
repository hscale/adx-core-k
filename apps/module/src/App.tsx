import React from 'react';
import { BrowserRouter, Routes, Route, Navigate, useParams } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TenantProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';
import { ModuleMarketplace } from './components/ModuleMarketplace';
import { ModuleManager } from './components/ModuleManager';
import { ModuleSettings } from './components/ModuleSettings';
import { ModuleDeveloper } from './components/ModuleDeveloper';
import { Navigation } from './components/Navigation';
import { ErrorBoundary } from './components/ErrorBoundary';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
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

export const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <TenantProvider>
          <EventBusProvider>
            <BrowserRouter>
              <div className="min-h-screen bg-gray-50">
                <Navigation />
                <main className="container mx-auto px-4 py-8">
                  <Routes>
                    <Route path="/" element={<Navigate to="/marketplace" replace />} />
                    <Route path="/marketplace" element={<ModuleMarketplace />} />
                    <Route path="/installed" element={<ModuleManager />} />
                    <Route path="/settings/:moduleId" element={<ModuleSettingsRoute />} />
                    <Route path="/developer" element={<ModuleDeveloper />} />
                    <Route path="*" element={<Navigate to="/marketplace" replace />} />
                  </Routes>
                </main>
              </div>
            </BrowserRouter>
          </EventBusProvider>
        </TenantProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
};

// Route wrapper for ModuleSettings to extract moduleId from params
const ModuleSettingsRoute: React.FC = () => {
  const { moduleId } = useParams<{ moduleId: string }>();
  
  if (!moduleId) {
    return <Navigate to="/installed" replace />;
  }
  
  return <ModuleSettings moduleId={moduleId} />;
};

// Re-export for Module Federation
export default App;