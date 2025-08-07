import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { ReactQueryDevtools } from '@tanstack/react-query-devtools'
import { Toaster } from 'react-hot-toast'

import App from './App'
import { AuthProvider } from './contexts/AuthContext'
import { ThemeProvider } from './contexts/ThemeContext'
import { TenantProvider } from './contexts/TenantContext'
import { PlatformProvider } from './contexts/PlatformContext'
import './i18n'
import './index.css'

// Create a client
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      retry: (failureCount, error: any) => {
        if (error?.status === 401 || error?.status === 403) {
          return false
        }
        return failureCount < 3
      },
    },
  },
})

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <PlatformProvider>
          <ThemeProvider>
            <TenantProvider>
              <AuthProvider>
                <App />
                <Toaster
                  position="top-right"
                  toastOptions={{
                    duration: 4000,
                    className: 'dark:bg-gray-800 dark:text-white',
                  }}
                />
                <ReactQueryDevtools initialIsOpen={false} />
              </AuthProvider>
            </TenantProvider>
          </ThemeProvider>
        </PlatformProvider>
      </QueryClientProvider>
    </BrowserRouter>
  </React.StrictMode>,
)