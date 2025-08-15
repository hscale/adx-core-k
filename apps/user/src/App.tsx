import React from 'react';
import { Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { EventBusProvider } from '@adx-core/event-bus';
import { UserProvider } from './providers';
import { UserProfilePage, UserSettingsPage, UserDirectoryPage } from './pages';

// Create a query client for this micro-frontend
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes
      retry: (failureCount, error: any) => {
        // Don't retry on 4xx errors
        if (error?.status >= 400 && error?.status < 500) {
          return false;
        }
        return failureCount < 3;
      },
    },
    mutations: {
      retry: (failureCount, error: any) => {
        // Don't retry on 4xx errors
        if (error?.status >= 400 && error?.status < 500) {
          return false;
        }
        return failureCount < 2;
      },
    },
  },
});

const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <EventBusProvider>
        <UserProvider>
          <div className="user-micro-app">
            <Routes>
              {/* Default route - redirect to directory */}
              <Route path="/" element={<Navigate to="/directory" replace />} />
              
              {/* User directory */}
              <Route path="/directory" element={<UserDirectoryPage />} />
              
              {/* Current user profile */}
              <Route path="/profile" element={<UserProfilePage />} />
              
              {/* Specific user profile */}
              <Route path="/profile/:userId" element={<UserProfilePage />} />
              
              {/* Current user settings */}
              <Route path="/settings" element={<UserSettingsPage />} />
              
              {/* Specific user settings (restricted) */}
              <Route path="/settings/:userId" element={<UserSettingsPage />} />
              
              {/* Catch all - redirect to directory */}
              <Route path="*" element={<Navigate to="/directory" replace />} />
            </Routes>
          </div>
        </UserProvider>
      </EventBusProvider>
    </QueryClientProvider>
  );
};

export default App;