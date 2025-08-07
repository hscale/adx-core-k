import React, { Suspense } from 'react'
import { Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { GlobalAuthProvider } from './providers/GlobalAuthProvider'
import { GlobalThemeProvider } from './providers/GlobalThemeProvider'
import { EventBusProvider } from './providers/EventBusProvider'
import { NavigationShell } from './components/NavigationShell'
import { GlobalErrorBoundary } from './components/GlobalErrorBoundary'
import { MicroFrontendLoader } from './components/MicroFrontendLoader'
import { LoadingSpinner } from './components/LoadingSpinner'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      retry: 1,
    },
  },
})

// Lazy load micro-frontends
const AuthMicroApp = React.lazy(() => import('authMicroApp/App'))
const TenantMicroApp = React.lazy(() => import('tenantMicroApp/App'))
const FileMicroApp = React.lazy(() => import('fileMicroApp/App'))
const UserMicroApp = React.lazy(() => import('userMicroApp/App'))
const WorkflowMicroApp = React.lazy(() => import('workflowMicroApp/App'))
const DashboardMicroApp = React.lazy(() => import('dashboardMicroApp/App'))

export const App: React.FC = () => {
  return (
    <GlobalErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <GlobalThemeProvider>
          <GlobalAuthProvider>
            <EventBusProvider>
              <div className="min-h-screen bg-gray-50">
                <NavigationShell>
                  <Routes>
                    {/* Dashboard - Default route */}
                    <Route 
                      path="/" 
                      element={
                        <MicroFrontendLoader name="dashboard">
                          <Suspense fallback={<LoadingSpinner />}>
                            <DashboardMicroApp />
                          </Suspense>
                        </MicroFrontendLoader>
                      } 
                    />
                    
                    {/* Auth routes */}
                    <Route 
                      path="/auth/*" 
                      element={
                        <MicroFrontendLoader name="auth">
                          <Suspense fallback={<LoadingSpinner />}>
                            <AuthMicroApp />
                          </Suspense>
                        </MicroFrontendLoader>
                      } 
                    />
                    
                    {/* Tenant routes */}
                    <Route 
                      path="/tenants/*" 
                      element={
                        <MicroFrontendLoader name="tenant">
                          <Suspense fallback={<LoadingSpinner />}>
                            <TenantMicroApp />
                          </Suspense>
                        </MicroFrontendLoader>
                      } 
                    />
                    
                    {/* File routes */}
                    <Route 
                      path="/files/*" 
                      element={
                        <MicroFrontendLoader name="file">
                          <Suspense fallback={<LoadingSpinner />}>
                            <FileMicroApp />
                          </Suspense>
                        </MicroFrontendLoader>
                      } 
                    />
                    
                    {/* User routes */}
                    <Route 
                      path="/users/*" 
                      element={
                        <MicroFrontendLoader name="user">
                          <Suspense fallback={<LoadingSpinner />}>
                            <UserMicroApp />
                          </Suspense>
                        </MicroFrontendLoader>
                      } 
                    />
                    
                    {/* Workflow routes */}
                    <Route 
                      path="/workflows/*" 
                      element={
                        <MicroFrontendLoader name="workflow">
                          <Suspense fallback={<LoadingSpinner />}>
                            <WorkflowMicroApp />
                          </Suspense>
                        </MicroFrontendLoader>
                      } 
                    />
                    
                    {/* Dashboard routes */}
                    <Route 
                      path="/dashboard/*" 
                      element={
                        <MicroFrontendLoader name="dashboard">
                          <Suspense fallback={<LoadingSpinner />}>
                            <DashboardMicroApp />
                          </Suspense>
                        </MicroFrontendLoader>
                      } 
                    />
                    
                    {/* 404 fallback */}
                    <Route 
                      path="*" 
                      element={
                        <div className="flex items-center justify-center min-h-screen">
                          <div className="text-center">
                            <h1 className="text-4xl font-bold text-gray-900 mb-4">404</h1>
                            <p className="text-gray-600">Page not found</p>
                          </div>
                        </div>
                      } 
                    />
                  </Routes>
                </NavigationShell>
              </div>
            </EventBusProvider>
          </GlobalAuthProvider>
        </GlobalThemeProvider>
      </QueryClientProvider>
    </GlobalErrorBoundary>
  )
}

export default App