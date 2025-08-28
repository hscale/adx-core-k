import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DesignSystemProvider } from '@adx-core/design-system';
import { SharedContextProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';

import TenantDashboard from './components/TenantDashboard';
import TenantSettings from './components/TenantSettings';
import TenantMembers from './components/TenantMembers';
import TenantBilling from './components/TenantBilling';
import ErrorBoundary from './components/ErrorBoundary';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 3,
      staleTime: 5 * 60 * 1000,
      cacheTime: 10 * 60 * 1000,
    },
  },
});

const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <DesignSystemProvider>
          <SharedContextProvider>
            <EventBusProvider>
              <Router>
                <div className="tenant-app min-h-screen bg-gray-50 dark:bg-gray-900">
                  <Routes>
                    <Route path="/" element={<Navigate to="/dashboard" replace />} />
                    <Route path="/dashboard" element={<TenantDashboard />} />
                    <Route path="/settings" element={<TenantSettings />} />
                    <Route path="/members" element={<TenantMembers />} />
                    <Route path="/billing" element={<TenantBilling />} />
                    <Route path="*" element={<Navigate to="/dashboard" replace />} />
                  </Routes>
                </div>
              </Router>
            </EventBusProvider>
          </SharedContextProvider>
        </DesignSystemProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
};

export default App;