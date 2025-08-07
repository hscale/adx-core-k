import { Routes, Route, Navigate } from 'react-router-dom'
import { useAuth } from './hooks/useAuth'
import { usePlatform } from './hooks/usePlatform'

// Layouts
import AuthLayout from './layouts/AuthLayout'
import DashboardLayout from './layouts/DashboardLayout'

// Auth Pages
import LoginPage from './pages/auth/LoginPage'
import RegisterPage from './pages/auth/RegisterPage'
import ForgotPasswordPage from './pages/auth/ForgotPasswordPage'

// Dashboard Pages
import DashboardPage from './pages/dashboard/DashboardPage'
import UsersPage from './pages/users/UsersPage'
import TenantsPage from './pages/tenants/TenantsPage'
import FilesPage from './pages/files/FilesPage'
import WorkflowsPage from './pages/workflows/WorkflowsPage'
import SettingsPage from './pages/settings/SettingsPage'
import ProfilePage from './pages/profile/ProfilePage'

// Components
import LoadingSpinner from './components/ui/LoadingSpinner'
import ErrorBoundary from './components/ErrorBoundary'

function App() {
  const { user, isLoading } = useAuth()
  const { platform } = usePlatform()

  if (isLoading) {
    return (
      <div className="min-h-screen flex items-center justify-center">
        <LoadingSpinner size="lg" />
      </div>
    )
  }

  return (
    <ErrorBoundary>
      <div className={`min-h-screen ${platform.isMobile ? 'mobile-app' : ''}`}>
        <Routes>
          {/* Public Routes */}
          <Route path="/auth" element={<AuthLayout />}>
            <Route path="login" element={<LoginPage />} />
            <Route path="register" element={<RegisterPage />} />
            <Route path="forgot-password" element={<ForgotPasswordPage />} />
            <Route index element={<Navigate to="login" replace />} />
          </Route>

          {/* Protected Routes */}
          <Route
            path="/*"
            element={
              user ? (
                <DashboardLayout>
                  <Routes>
                    <Route index element={<Navigate to="/dashboard" replace />} />
                    <Route path="dashboard" element={<DashboardPage />} />
                    <Route path="users" element={<UsersPage />} />
                    <Route path="tenants" element={<TenantsPage />} />
                    <Route path="files" element={<FilesPage />} />
                    <Route path="workflows" element={<WorkflowsPage />} />
                    <Route path="settings" element={<SettingsPage />} />
                    <Route path="profile" element={<ProfilePage />} />
                  </Routes>
                </DashboardLayout>
              ) : (
                <Navigate to="/auth/login" replace />
              )
            }
          />
        </Routes>
      </div>
    </ErrorBoundary>
  )
}

export default App