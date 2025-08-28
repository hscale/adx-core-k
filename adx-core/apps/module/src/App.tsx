import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DesignSystemProvider } from '@adx-core/design-system';
import { SharedContextProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';

import ModuleMarketplace from './components/ModuleMarketplace';
import ModuleManager from './components/ModuleManager';
import ModuleInstaller from './components/ModuleInstaller';
import ModuleSettings from './components/ModuleSettings';
import ModuleDeveloper from './components/ModuleDeveloper';
import ErrorBoundary from './components/ErrorBoundary';
import LoadingSpinner from './components/LoadingSpinner';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      retry: 3,
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
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
                <div className="module-app min-h-screen bg-gray-50 dark:bg-gray-900">
                  <Routes>
                    <Route path="/" element={<Navigate to="/marketplace" replace />} />
                    <Route path="/marketplace" element={<ModuleMarketplace />} />
                    <Route path="/manager" element={<ModuleManager />} />
                    <Route path="/installer" element={<ModuleInstaller />} />
                    <Route path="/settings" element={<ModuleSettings />} />
                    <Route path="/developer" element={<ModuleDeveloper />} />
                    <Route path="*" element={<Navigate to="/marketplace" replace />} />
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