import React from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { EventBusProvider } from '@adx-core/event-bus';
import { FileProvider } from './providers/FileProvider';
import { FileManager } from './components/FileManager';
import { FileUpload } from './components/FileUpload';
import { FileBrowser } from './components/FileBrowser';
import { FileSharing } from './components/FileSharing';
import { FilePermissions } from './components/FilePermissions';

// Create a query client for React Query
const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      gcTime: 10 * 60 * 1000, // 10 minutes (renamed from cacheTime)
      retry: (failureCount, error: any) => {
        // Don't retry on 4xx errors
        if (error?.status >= 400 && error?.status < 500) {
          return false;
        }
        return failureCount < 3;
      },
    },
    mutations: {
      retry: 1,
    },
  },
});

const App: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <EventBusProvider>
        <FileProvider>
            <BrowserRouter>
              <div className="min-h-screen bg-gray-50">
                <Routes>
                  {/* Main file management interface */}
                  <Route path="/" element={<FileManager />} />
                  <Route path="/files" element={<FileManager />} />
                  <Route path="/files/*" element={<FileManager />} />
                  
                  {/* Upload interface */}
                  <Route path="/upload" element={<UploadPage />} />
                  
                  {/* Browser interface */}
                  <Route path="/browse" element={<BrowsePage />} />
                  <Route path="/browse/*" element={<BrowsePage />} />
                  
                  {/* Sharing interface */}
                  <Route path="/share/:fileId" element={<SharePage />} />
                  
                  {/* Permissions interface */}
                  <Route path="/permissions/:fileId" element={<PermissionsPage />} />
                  
                  {/* Redirect root to files */}
                  <Route path="*" element={<Navigate to="/files" replace />} />
                </Routes>
              </div>
            </BrowserRouter>
          </FileProvider>
        </EventBusProvider>
    </QueryClientProvider>
  );
};

// Page components for different routes
const UploadPage: React.FC = () => {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900">Upload Files</h1>
          <p className="text-gray-600 mt-2">
            Upload files to your workspace
          </p>
        </div>
        
        <FileUpload
          className="mb-8"
          maxFiles={20}
          maxSize={1024 * 1024 * 1024} // 1GB
          onUploadComplete={(files) => {
            console.log('Upload completed:', files);
          }}
        />
        
        <div className="mt-8">
          <h2 className="text-xl font-semibold text-gray-900 mb-4">Recent Files</h2>
          <FileBrowser />
        </div>
      </div>
    </div>
  );
};

const BrowsePage: React.FC = () => {
  return (
    <div className="container mx-auto px-4 py-8">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900">Browse Files</h1>
        <p className="text-gray-600 mt-2">
          Navigate and manage your files and folders
        </p>
      </div>
      
      <FileBrowser
        onFileDoubleClick={(file) => {
          if (file.type === 'file') {
            // Handle file preview or download
            console.log('Open file:', file);
          }
        }}
      />
    </div>
  );
};

const SharePage: React.FC = () => {
  const fileId = window.location.pathname.split('/').pop();
  
  // In a real implementation, you would fetch the file data
  const mockFile = {
    id: fileId || '',
    name: 'Example File.pdf',
    type: 'file' as const,
    size: 1024 * 1024,
    mimeType: 'application/pdf',
    path: '/documents/example.pdf',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    createdBy: 'user123',
    modifiedBy: 'user123',
    isShared: true,
    permissions: {
      owner: 'user123',
      permissions: {},
      inheritFromParent: false,
    },
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-2xl mx-auto">
        <FileSharing
          file={mockFile}
          onClose={() => window.history.back()}
        />
      </div>
    </div>
  );
};

const PermissionsPage: React.FC = () => {
  const fileId = window.location.pathname.split('/').pop();
  
  // In a real implementation, you would fetch the file data
  const mockFile = {
    id: fileId || '',
    name: 'Example File.pdf',
    type: 'file' as const,
    size: 1024 * 1024,
    mimeType: 'application/pdf',
    path: '/documents/example.pdf',
    createdAt: new Date().toISOString(),
    updatedAt: new Date().toISOString(),
    createdBy: 'user123',
    modifiedBy: 'user123',
    isShared: true,
    permissions: {
      owner: 'user123',
      permissions: {
        'user456': 'read' as const,
        'user789': 'write' as const,
      },
      teamPermissions: {
        'team1': 'read' as const,
      },
      inheritFromParent: false,
    },
  };

  return (
    <div className="container mx-auto px-4 py-8">
      <div className="max-w-2xl mx-auto">
        <FilePermissions
          file={mockFile}
          onClose={() => window.history.back()}
          onPermissionsUpdated={(permissions) => {
            console.log('Permissions updated:', permissions);
          }}
        />
      </div>
    </div>
  );
};

export default App;