import React from 'react';
import ReactDOM from 'react-dom/client';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { TenantProvider } from '@adx-core/shared-context';
import { DesignSystemProvider } from '@adx-core/design-system';
import { EventBusProvider } from '@adx-core/event-bus';
import App from './App';
import './index.css';

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      cacheTime: 10 * 60 * 1000, // 10 minutes
      refetchOnWindowFocus: false,
    },
  },
});

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <QueryClientProvider client={queryClient}>
      <TenantProvider>
        <DesignSystemProvider>
          <EventBusProvider>
            <App />
          </EventBusProvider>
        </DesignSystemProvider>
      </TenantProvider>
    </QueryClientProvider>
  </React.StrictMode>
);