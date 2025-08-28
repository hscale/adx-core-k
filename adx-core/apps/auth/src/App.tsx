import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DesignSystemProvider } from '@adx-core/design-system';
import { SharedContextProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';

import LoginForm from './components/LoginForm';
import RegisterForm from './components/RegisterForm';
import MFASetup from './components/MFASetup';
import PasswordReset from './components/PasswordReset';
import SSOLogin from './components/SSOLogin';
import ErrorBoundary from './components/ErrorBoundary';

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
                <div className="auth-app min-h-screen bg-gray-50 dark:bg-gray-900">
                  <Routes>
                    <Route path="/" element={<Navigate to="/login" replace />} />
                    <Route path="/login" element={<LoginForm />} />
                    <Route path="/register" element={<RegisterForm />} />
                    <Route path="/mfa-setup" element={<MFASetup />} />
                    <Route path="/password-reset" element={<PasswordReset />} />
                    <Route path="/sso" element={<SSOLogin />} />
                    <Route path="*" element={<Navigate to="/login" replace />} />
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