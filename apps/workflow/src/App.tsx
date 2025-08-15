import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { useTenantContext } from '@adx-core/shared-context';
import { useEventBus } from '@adx-core/event-bus';
import WorkflowDashboard from './pages/WorkflowDashboard';
import WorkflowMonitor from './components/WorkflowMonitor';
import WorkflowHistory from './components/WorkflowHistory';
import WorkflowAnalytics from './components/WorkflowAnalytics';
import WorkflowManagement from './components/WorkflowManagement';
import Navigation from './components/Navigation';
import ErrorBoundary from './components/ErrorBoundary';

const App: React.FC = () => {
  const { state: tenantState } = useTenantContext();
  const { emit } = useEventBus();

  React.useEffect(() => {
    // Emit workflow app loaded event
    emit('workflow:app_loaded', {
      tenantId: tenantState.currentTenant?.id,
      timestamp: Date.now(),
    });

    // Listen for workflow events from other micro-frontends
    const handleWorkflowEvent = (event: any) => {
      console.log('Workflow app received event:', event);
    };

    return () => {
      emit('workflow:app_unloaded', {
        tenantId: tenantState.currentTenant?.id,
        timestamp: Date.now(),
      });
    };
  }, [emit, tenantState.currentTenant?.id]);

  if (!tenantState.currentTenant) {
    return (
      <div className="flex items-center justify-center min-h-screen">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-gray-900 mb-2">
            No Tenant Selected
          </h2>
          <p className="text-gray-600">
            Please select a tenant to access workflow management.
          </p>
        </div>
      </div>
    );
  }

  return (
    <ErrorBoundary>
      <BrowserRouter basename="/workflows">
        <div className="min-h-screen bg-gray-50">
          <Navigation />
          <main className="container mx-auto px-4 py-8">
            <Routes>
              <Route path="/" element={<WorkflowDashboard />} />
              <Route path="/monitor" element={<WorkflowMonitor />} />
              <Route path="/history" element={<WorkflowHistory />} />
              <Route path="/analytics" element={<WorkflowAnalytics />} />
              <Route path="/management" element={<WorkflowManagement />} />
              <Route path="*" element={<Navigate to="/" replace />} />
            </Routes>
          </main>
        </div>
      </BrowserRouter>
    </ErrorBoundary>
  );
};

export default App;