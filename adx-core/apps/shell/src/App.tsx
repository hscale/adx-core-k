import React, { Suspense } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DesignSystemProvider } from '@adx-core/design-system';
import { SharedContextProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';
import { I18nextProvider } from 'react-i18next';
import i18n from '@adx-core/i18n';

import Navigation from './components/Navigation';
import Dashboard from './components/Dashboard';
import ErrorBoundary from './components/ErrorBoundary';
import MicroFrontendLoader from './components/MicroFrontendLoader';

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
      retry: 3,
    },
  },
});

const App: React.FC = () => {
  return (
    <ErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <I18nextProvider i18n={i18n}>
          <DesignSystemProvider>
            <SharedContextProvider>
              <EventBusProvider>
                <BrowserRouter>
                  <div className="app-shell min-h-screen bg-gray-50 dark:bg-gray-900">
                    <Navigation />
                    <main className="main-content">
                      <Suspense fallback={<MicroFrontendLoader />}>
                        <Routes>
                          <Route path="/" element={<Dashboard />} />
                          <Route path="/auth/*" element={<AuthApp />} />
                          <Route path="/tenant/*" element={<TenantApp />} />
                          <Route path="/files/*" element={<FileApp />} />
                          <Route path="/users/*" element={<UserApp />} />
                          <Route path="/workflows/*" element={<WorkflowApp />} />
                          <Route path="/modules/*" element={<ModuleApp />} />
                          <Route path="*" element={<Navigate to="/" replace />} />
                        </Routes>
                      </Suspense>
                    </main>
                  </div>
                </BrowserRouter>
              </EventBusProvider>
            </SharedContextProvider>
          </DesignSystemProvider>
        </I18nextProvider>
      </QueryClientProvider>
    </ErrorBoundary>
  );
};

export default App;