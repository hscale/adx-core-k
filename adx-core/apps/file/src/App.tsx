import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { DesignSystemProvider } from '@adx-core/design-system';
import { SharedContextProvider } from '@adx-core/shared-context';
import { EventBusProvider } from '@adx-core/event-bus';
import { Card } from '@adx-core/design-system';
import { FolderIcon } from 'lucide-react';

const queryClient = new QueryClient();

const FileDashboard: React.FC = () => (
  <div className="p-6">
    <Card className="p-8 text-center">
      <FolderIcon className="h-16 w-16 text-blue-500 mx-auto mb-4" />
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">File Management</h1>
      <p className="text-gray-600 dark:text-gray-400">File management features coming soon</p>
    </Card>
  </div>
);

const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <DesignSystemProvider>
        <SharedContextProvider>
          <EventBusProvider>
            <Router>
              <div className="file-app min-h-screen bg-gray-50 dark:bg-gray-900">
                <Routes>
                  <Route path="/" element={<FileDashboard />} />
                  <Route path="*" element={<Navigate to="/" replace />} />
                </Routes>
              </div>
            </Router>
          </EventBusProvider>
        </SharedContextProvider>
      </DesignSystemProvider>
    </QueryClientProvider>
  );
};

export default App;