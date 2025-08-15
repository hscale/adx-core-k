import React, { useCallback, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { Upload, X, File, AlertCircle, CheckCircle } from 'lucide-react';
import { Button } from '@adx-core/design-system';
import { useFileOperations } from '../hooks/useFileOperations';
import { useFile } from '../hooks/useFile';
import { formatFileSize, formatUploadSpeed } from '../utils/fileUtils';
import { FileUploadProgress } from '../types/file';

interface FileUploadProps {
  className?: string;
  onUploadComplete?: (files: any[]) => void;
  maxFiles?: number;
  maxSize?: number;
  acceptedFileTypes?: string[];
  showProgress?: boolean;
}

export const FileUpload: React.FC<FileUploadProps> = ({
  className = '',
  onUploadComplete,
  maxFiles = 10,
  maxSize = 100 * 1024 * 1024, // 100MB
  acceptedFileTypes,
  showProgress = true,
}) => {
  const { currentPath } = useFile();
  const { uploadFiles, uploadProgress } = useFileOperations();
  const [dragActive, setDragActive] = useState(false);
  const [uploadError, setUploadError] = useState<string | null>(null);

  const onDrop = useCallback(async (acceptedFiles: File[], rejectedFiles: any[]) => {
    setDragActive(false);
    setUploadError(null);

    if (rejectedFiles.length > 0) {
      const errors = rejectedFiles.map(({ file, errors }) => 
        `${file.name}: ${errors.map((e: any) => e.message).join(', ')}`
      );
      setUploadError(`Some files were rejected: ${errors.join('; ')}`);
    }

    if (acceptedFiles.length > 0) {
      try {
        const uploadedFiles = await uploadFiles(acceptedFiles, currentPath);
        onUploadComplete?.(uploadedFiles);
      } catch (error) {
        setUploadError(error instanceof Error ? error.message : 'Upload failed');
      }
    }
  }, [uploadFiles, currentPath, onUploadComplete]);

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    onDrop,
    onDragEnter: () => setDragActive(true),
    onDragLeave: () => setDragActive(false),
    maxFiles,
    maxSize,
    accept: acceptedFileTypes ? 
      acceptedFileTypes.reduce((acc, type) => ({ ...acc, [type]: [] }), {}) : 
      undefined,
    multiple: maxFiles > 1,
  });

  const hasActiveUploads = uploadProgress.length > 0;
  const completedUploads = uploadProgress.filter(p => p.status === 'completed').length;
  const failedUploads = uploadProgress.filter(p => p.status === 'failed').length;

  return (
    <div className={`file-upload-container ${className}`}>
      {/* Upload Zone */}
      <div
        {...getRootProps()}
        className={`
          file-upload-zone
          ${isDragActive || dragActive ? 'drag-active' : ''}
          ${hasActiveUploads ? 'opacity-50 pointer-events-none' : ''}
        `}
      >
        <input {...getInputProps()} />
        
        <div className="flex flex-col items-center justify-center space-y-4">
          <Upload className="w-12 h-12 text-gray-400" />
          
          <div className="text-center">
            <p className="text-lg font-medium text-gray-900">
              {isDragActive || dragActive ? 'Drop files here' : 'Upload files'}
            </p>
            <p className="text-sm text-gray-500 mt-1">
              Drag and drop files here, or click to select files
            </p>
            <p className="text-xs text-gray-400 mt-2">
              Max {maxFiles} files, {formatFileSize(maxSize)} per file
            </p>
          </div>

          <Button variant="outline" disabled={hasActiveUploads}>
            Choose Files
          </Button>
        </div>
      </div>

      {/* Upload Progress */}
      {showProgress && uploadProgress.length > 0 && (
        <div className="mt-6 space-y-3">
          <div className="flex items-center justify-between">
            <h3 className="text-sm font-medium text-gray-900">
              Uploading {uploadProgress.length} file{uploadProgress.length !== 1 ? 's' : ''}
            </h3>
            <div className="text-xs text-gray-500">
              {completedUploads > 0 && (
                <span className="text-green-600">
                  {completedUploads} completed
                </span>
              )}
              {failedUploads > 0 && (
                <span className="text-red-600 ml-2">
                  {failedUploads} failed
                </span>
              )}
            </div>
          </div>

          <div className="space-y-2">
            {uploadProgress.map((progress) => (
              <UploadProgressItem key={progress.fileId} progress={progress} />
            ))}
          </div>
        </div>
      )}

      {/* Error Display */}
      {uploadError && (
        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-md">
          <div className="flex items-start">
            <AlertCircle className="w-5 h-5 text-red-400 mt-0.5" />
            <div className="ml-3">
              <p className="text-sm text-red-800">{uploadError}</p>
            </div>
            <button
              onClick={() => setUploadError(null)}
              className="ml-auto text-red-400 hover:text-red-600"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

interface UploadProgressItemProps {
  progress: FileUploadProgress;
}

const UploadProgressItem: React.FC<UploadProgressItemProps> = ({ progress }) => {
  const getStatusIcon = () => {
    switch (progress.status) {
      case 'completed':
        return <CheckCircle className="w-4 h-4 text-green-500" />;
      case 'failed':
        return <AlertCircle className="w-4 h-4 text-red-500" />;
      default:
        return <File className="w-4 h-4 text-blue-500" />;
    }
  };

  const getStatusColor = () => {
    switch (progress.status) {
      case 'completed':
        return 'bg-green-500';
      case 'failed':
        return 'bg-red-500';
      case 'uploading':
        return 'bg-blue-500';
      default:
        return 'bg-gray-300';
    }
  };

  return (
    <div className="flex items-center space-x-3 p-3 bg-gray-50 rounded-lg">
      {getStatusIcon()}
      
      <div className="flex-1 min-w-0">
        <div className="flex items-center justify-between mb-1">
          <p className="text-sm font-medium text-gray-900 truncate">
            {progress.fileName}
          </p>
          <span className="text-xs text-gray-500">
            {Math.round(progress.progress)}%
          </span>
        </div>
        
        <div className="file-progress-bar">
          <div
            className={`file-progress-fill ${getStatusColor()}`}
            style={{ width: `${progress.progress}%` }}
          />
        </div>
        
        <div className="flex items-center justify-between mt-1 text-xs text-gray-500">
          <span>
            {progress.status === 'uploading' && progress.uploadSpeed && (
              <>Upload speed: {formatUploadSpeed(progress.uploadSpeed)}</>
            )}
            {progress.status === 'failed' && progress.error && (
              <span className="text-red-600">{progress.error}</span>
            )}
            {progress.status === 'completed' && (
              <span className="text-green-600">Upload complete</span>
            )}
          </span>
          
          {progress.estimatedTimeRemaining && progress.status === 'uploading' && (
            <span>
              {Math.ceil(progress.estimatedTimeRemaining)}s remaining
            </span>
          )}
        </div>
      </div>
    </div>
  );
};