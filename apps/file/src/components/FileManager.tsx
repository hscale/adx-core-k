import React, { useState } from 'react';
import { Plus, Upload, FolderPlus, Info } from 'lucide-react';
import { Button } from '@adx-core/design-system';
import { FileUpload } from './FileUpload';
import { FileBrowser } from './FileBrowser';
import { StorageQuota } from './StorageQuota';
import { useFile } from '../hooks/useFile';
import { useFileOperations } from '../hooks/useFileOperations';
import { FileItem } from '../types/file';

interface FileManagerProps {
  className?: string;
  showUpload?: boolean;
  showQuota?: boolean;
  onFileSelect?: (file: FileItem) => void;
  onFileDoubleClick?: (file: FileItem) => void;
}

export const FileManager: React.FC<FileManagerProps> = ({
  className = '',
  showUpload = true,
  showQuota = true,
  onFileSelect,
  onFileDoubleClick,
}) => {
  const { currentPath, selectedFiles, storageQuota } = useFile();
  const { createFolder } = useFileOperations();
  const [showUploadModal, setShowUploadModal] = useState(false);
  const [showNewFolderModal, setShowNewFolderModal] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');

  const handleCreateFolder = async () => {
    if (newFolderName.trim()) {
      try {
        await createFolder(newFolderName.trim(), currentPath);
        setNewFolderName('');
        setShowNewFolderModal(false);
      } catch (error) {
        console.error('Failed to create folder:', error);
      }
    }
  };

  const handleUploadComplete = () => {
    setShowUploadModal(false);
    // Files will be automatically refreshed via the FileProvider
  };

  return (
    <div className={`file-manager ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Files</h1>
          <p className="text-sm text-gray-500 mt-1">
            Manage your files and folders
          </p>
        </div>

        <div className="flex items-center space-x-3">
          {showQuota && storageQuota && (
            <StorageQuota quota={storageQuota} />
          )}
          
          <Button
            variant="outline"
            size="sm"
            onClick={() => setShowNewFolderModal(true)}
          >
            <FolderPlus className="w-4 h-4 mr-2" />
            New Folder
          </Button>

          {showUpload && (
            <Button
              size="sm"
              onClick={() => setShowUploadModal(true)}
            >
              <Upload className="w-4 h-4 mr-2" />
              Upload Files
            </Button>
          )}
        </div>
      </div>

      {/* Quick Upload Zone (when no modal) */}
      {showUpload && !showUploadModal && (
        <div className="mb-6">
          <FileUpload
            className="border-2 border-dashed border-gray-200 rounded-lg p-4"
            onUploadComplete={handleUploadComplete}
            showProgress={false}
            maxFiles={5}
          />
        </div>
      )}

      {/* File Browser */}
      <FileBrowser
        onFileSelect={onFileSelect}
        onFileDoubleClick={onFileDoubleClick}
      />

      {/* Upload Modal */}
      {showUploadModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-2xl mx-4">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Upload Files</h2>
              <button
                onClick={() => setShowUploadModal(false)}
                className="text-gray-400 hover:text-gray-600"
              >
                <Plus className="w-6 h-6 rotate-45" />
              </button>
            </div>

            <FileUpload
              onUploadComplete={handleUploadComplete}
              maxFiles={20}
              maxSize={500 * 1024 * 1024} // 500MB
            />
          </div>
        </div>
      )}

      {/* New Folder Modal */}
      {showNewFolderModal && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-full max-w-md mx-4">
            <div className="flex items-center justify-between mb-4">
              <h2 className="text-lg font-semibold">Create New Folder</h2>
              <button
                onClick={() => {
                  setShowNewFolderModal(false);
                  setNewFolderName('');
                }}
                className="text-gray-400 hover:text-gray-600"
              >
                <Plus className="w-6 h-6 rotate-45" />
              </button>
            </div>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Folder Name
                </label>
                <input
                  type="text"
                  value={newFolderName}
                  onChange={(e) => setNewFolderName(e.target.value)}
                  placeholder="Enter folder name"
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  onKeyPress={(e) => {
                    if (e.key === 'Enter') {
                      handleCreateFolder();
                    }
                  }}
                  autoFocus
                />
              </div>

              <div className="flex justify-end space-x-3">
                <Button
                  variant="outline"
                  onClick={() => {
                    setShowNewFolderModal(false);
                    setNewFolderName('');
                  }}
                >
                  Cancel
                </Button>
                <Button
                  onClick={handleCreateFolder}
                  disabled={!newFolderName.trim()}
                >
                  Create Folder
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}

      {/* Selection Info */}
      {selectedFiles.length > 0 && (
        <div className="fixed bottom-4 left-1/2 transform -translate-x-1/2 bg-white border border-gray-200 rounded-lg shadow-lg px-4 py-2">
          <div className="flex items-center space-x-2 text-sm">
            <Info className="w-4 h-4 text-blue-500" />
            <span>
              {selectedFiles.length} file{selectedFiles.length !== 1 ? 's' : ''} selected
            </span>
          </div>
        </div>
      )}
    </div>
  );
};