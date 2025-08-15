export interface FileItem {
  id: string;
  name: string;
  type: 'file' | 'folder';
  size: number;
  mimeType?: string;
  extension?: string;
  path: string;
  parentId?: string;
  createdAt: string;
  updatedAt: string;
  createdBy: string;
  modifiedBy: string;
  isShared: boolean;
  permissions: FilePermissions;
  metadata?: FileMetadata;
  versions?: FileVersion[];
  tags?: string[];
  thumbnail?: string;
  previewUrl?: string;
  downloadUrl?: string;
}

export interface FilePermissions {
  owner: string;
  permissions: {
    [userId: string]: FilePermissionLevel;
  };
  teamPermissions?: {
    [teamId: string]: FilePermissionLevel;
  };
  publicAccess?: PublicAccessSettings;
  inheritFromParent: boolean;
}

export type FilePermissionLevel = 'read' | 'write' | 'admin' | 'none';

export interface PublicAccessSettings {
  enabled: boolean;
  level: 'read' | 'write';
  expiresAt?: string;
  password?: boolean;
  downloadAllowed: boolean;
}

export interface FileMetadata {
  checksum: string;
  virusScanStatus: 'pending' | 'clean' | 'infected' | 'failed';
  virusScanDate?: string;
  processingStatus: 'pending' | 'processing' | 'completed' | 'failed';
  extractedText?: string;
  imageMetadata?: ImageMetadata;
  documentMetadata?: DocumentMetadata;
}

export interface ImageMetadata {
  width: number;
  height: number;
  format: string;
  colorSpace?: string;
  hasAlpha?: boolean;
  exif?: Record<string, any>;
}

export interface DocumentMetadata {
  pageCount?: number;
  wordCount?: number;
  author?: string;
  title?: string;
  subject?: string;
  keywords?: string[];
  createdDate?: string;
  modifiedDate?: string;
}

export interface FileVersion {
  id: string;
  version: number;
  size: number;
  checksum: string;
  createdAt: string;
  createdBy: string;
  comment?: string;
  downloadUrl: string;
}

export interface FileUploadProgress {
  fileId: string;
  fileName: string;
  progress: number;
  status: 'pending' | 'uploading' | 'processing' | 'completed' | 'failed';
  error?: string;
  estimatedTimeRemaining?: number;
  uploadSpeed?: number;
}

export interface FileShare {
  id: string;
  fileId: string;
  shareType: 'user' | 'team' | 'public' | 'email';
  sharedWith?: string; // userId, teamId, or email
  permissions: FilePermissionLevel;
  createdAt: string;
  createdBy: string;
  expiresAt?: string;
  accessCount: number;
  lastAccessedAt?: string;
  shareUrl?: string;
  password?: boolean;
  downloadAllowed: boolean;
  message?: string;
}

export interface StorageQuota {
  used: number;
  limit: number;
  percentage: number;
  breakdown: {
    files: number;
    images: number;
    documents: number;
    videos: number;
    other: number;
  };
}

export interface FileSearchFilters {
  query?: string;
  type?: 'file' | 'folder' | 'all';
  mimeTypes?: string[];
  extensions?: string[];
  sizeMin?: number;
  sizeMax?: number;
  dateFrom?: string;
  dateTo?: string;
  createdBy?: string[];
  tags?: string[];
  sharedOnly?: boolean;
  path?: string;
}

export interface FileSearchResult {
  items: FileItem[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
  facets?: {
    types: { [key: string]: number };
    extensions: { [key: string]: number };
    sizes: { [key: string]: number };
    dates: { [key: string]: number };
    creators: { [key: string]: number };
    tags: { [key: string]: number };
  };
}

export interface FileOperation {
  id: string;
  type: 'upload' | 'download' | 'copy' | 'move' | 'delete' | 'share' | 'process';
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  progress: number;
  fileIds: string[];
  targetPath?: string;
  error?: string;
  startedAt: string;
  completedAt?: string;
  estimatedTimeRemaining?: number;
}

export interface FileBFFClient {
  // File operations
  uploadFile(file: File, path?: string, onProgress?: (progress: FileUploadProgress) => void): Promise<FileItem>;
  uploadFiles(files: File[], path?: string, onProgress?: (progress: FileUploadProgress[]) => void): Promise<FileItem[]>;
  downloadFile(fileId: string): Promise<Blob>;
  downloadFiles(fileIds: string[]): Promise<Blob>;
  
  // File management
  getFile(fileId: string): Promise<FileItem>;
  getFiles(path?: string, filters?: FileSearchFilters): Promise<FileSearchResult>;
  searchFiles(filters: FileSearchFilters): Promise<FileSearchResult>;
  createFolder(name: string, parentPath?: string): Promise<FileItem>;
  renameFile(fileId: string, newName: string): Promise<FileItem>;
  moveFiles(fileIds: string[], targetPath: string): Promise<FileOperation>;
  copyFiles(fileIds: string[], targetPath: string): Promise<FileOperation>;
  deleteFiles(fileIds: string[]): Promise<FileOperation>;
  
  // Sharing and permissions
  shareFile(fileId: string, shareSettings: Partial<FileShare>): Promise<FileShare>;
  updateFilePermissions(fileId: string, permissions: Partial<FilePermissions>): Promise<FilePermissions>;
  getFileShares(fileId: string): Promise<FileShare[]>;
  revokeShare(shareId: string): Promise<void>;
  
  // Storage and quotas
  getStorageQuota(): Promise<StorageQuota>;
  getFileOperations(): Promise<FileOperation[]>;
  cancelOperation(operationId: string): Promise<void>;
}

export interface FileContextType {
  // Current state
  currentPath: string;
  selectedFiles: string[];
  viewMode: 'grid' | 'list';
  sortBy: 'name' | 'size' | 'date' | 'type';
  sortOrder: 'asc' | 'desc';
  
  // Data
  files: FileItem[];
  loading: boolean;
  error: string | null;
  uploadProgress: FileUploadProgress[];
  operations: FileOperation[];
  storageQuota: StorageQuota | null;
  
  // Actions
  setCurrentPath: (path: string) => void;
  setSelectedFiles: (fileIds: string[]) => void;
  setViewMode: (mode: 'grid' | 'list') => void;
  setSortBy: (sortBy: 'name' | 'size' | 'date' | 'type') => void;
  setSortOrder: (order: 'asc' | 'desc') => void;
  
  // File operations
  uploadFiles: (files: File[], path?: string) => Promise<void>;
  downloadFiles: (fileIds: string[]) => Promise<void>;
  createFolder: (name: string) => Promise<void>;
  renameFile: (fileId: string, newName: string) => Promise<void>;
  moveFiles: (fileIds: string[], targetPath: string) => Promise<void>;
  copyFiles: (fileIds: string[], targetPath: string) => Promise<void>;
  deleteFiles: (fileIds: string[]) => Promise<void>;
  
  // Sharing
  shareFiles: (fileIds: string[], shareSettings: Partial<FileShare>) => Promise<void>;
  
  // Utility
  refreshFiles: () => Promise<void>;
  clearSelection: () => void;
}