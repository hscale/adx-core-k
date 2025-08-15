import React, { createContext, useReducer, useCallback, useEffect } from 'react';
import { useQuery } from '@tanstack/react-query';
import { useEventBus } from '@adx-core/event-bus';
import { useTenantStore } from '@adx-core/shared-context';
import { fileBFFClient } from '../services/fileBFFClient';
import { 
  FileContextType, 
  FileUploadProgress, 
  FileOperation, 
  FileSearchFilters 
} from '../types/file';
import { sortFiles, filterFiles } from '../utils/fileUtils';

interface FileState {
  currentPath: string;
  selectedFiles: string[];
  viewMode: 'grid' | 'list';
  sortBy: 'name' | 'size' | 'date' | 'type';
  sortOrder: 'asc' | 'desc';
  searchQuery: string;
  uploadProgress: FileUploadProgress[];
  operations: FileOperation[];
}

type FileAction =
  | { type: 'SET_CURRENT_PATH'; payload: string }
  | { type: 'SET_SELECTED_FILES'; payload: string[] }
  | { type: 'SET_VIEW_MODE'; payload: 'grid' | 'list' }
  | { type: 'SET_SORT_BY'; payload: 'name' | 'size' | 'date' | 'type' }
  | { type: 'SET_SORT_ORDER'; payload: 'asc' | 'desc' }
  | { type: 'SET_SEARCH_QUERY'; payload: string }
  | { type: 'SET_UPLOAD_PROGRESS'; payload: FileUploadProgress[] }
  | { type: 'SET_OPERATIONS'; payload: FileOperation[] }
  | { type: 'CLEAR_SELECTION' };

const initialState: FileState = {
  currentPath: '/',
  selectedFiles: [],
  viewMode: 'grid',
  sortBy: 'name',
  sortOrder: 'asc',
  searchQuery: '',
  uploadProgress: [],
  operations: [],
};

const fileReducer = (state: FileState, action: FileAction): FileState => {
  switch (action.type) {
    case 'SET_CURRENT_PATH':
      return { ...state, currentPath: action.payload, selectedFiles: [] };
    case 'SET_SELECTED_FILES':
      return { ...state, selectedFiles: action.payload };
    case 'SET_VIEW_MODE':
      return { ...state, viewMode: action.payload };
    case 'SET_SORT_BY':
      return { ...state, sortBy: action.payload };
    case 'SET_SORT_ORDER':
      return { ...state, sortOrder: action.payload };
    case 'SET_SEARCH_QUERY':
      return { ...state, searchQuery: action.payload };
    case 'SET_UPLOAD_PROGRESS':
      return { ...state, uploadProgress: action.payload };
    case 'SET_OPERATIONS':
      return { ...state, operations: action.payload };
    case 'CLEAR_SELECTION':
      return { ...state, selectedFiles: [] };
    default:
      return state;
  }
};

export const FileContext = createContext<FileContextType | null>(null);

interface FileProviderProps {
  children: React.ReactNode;
}

export const FileProvider: React.FC<FileProviderProps> = ({ children }) => {
  const [state, dispatch] = useReducer(fileReducer, initialState);
  const { subscribe } = useEventBus();
  const { currentTenant } = useTenantStore();

  // Set up BFF client with tenant context
  useEffect(() => {
    if (currentTenant) {
      const authToken = localStorage.getItem('authToken') || '';
      fileBFFClient.setTenantContext(currentTenant.id, authToken);
    }
  }, [currentTenant]);

  // Query for files in current path
  const {
    data: filesResult,
    isLoading: filesLoading,
    error: filesError,
    refetch: refetchFiles,
  } = useQuery({
    queryKey: ['files', state.currentPath, state.searchQuery],
    queryFn: async () => {
      const filters: FileSearchFilters = {};
      if (state.searchQuery) {
        filters.query = state.searchQuery;
      }
      return fileBFFClient.getFiles(state.currentPath, filters);
    },
    enabled: !!currentTenant,
    staleTime: 30000, // 30 seconds
  });

  // Query for storage quota
  const { data: storageQuota } = useQuery({
    queryKey: ['storage-quota'],
    queryFn: () => fileBFFClient.getStorageQuota(),
    enabled: !!currentTenant,
    staleTime: 60000, // 1 minute
  });

  // Query for file operations
  const { data: operations } = useQuery({
    queryKey: ['file-operations'],
    queryFn: () => fileBFFClient.getFileOperations(),
    enabled: !!currentTenant,
    refetchInterval: 5000, // Poll every 5 seconds
  });

  // Update operations in state
  useEffect(() => {
    if (operations) {
      dispatch({ type: 'SET_OPERATIONS', payload: operations });
    }
  }, [operations]);

  // Process and sort files
  const files = React.useMemo(() => {
    if (!filesResult?.items) return [];
    
    let processedFiles = filesResult.items;
    
    // Apply search filter
    if (state.searchQuery) {
      processedFiles = filterFiles(processedFiles, state.searchQuery);
    }
    
    // Apply sorting
    processedFiles = sortFiles(processedFiles, state.sortBy, state.sortOrder);
    
    return processedFiles;
  }, [filesResult?.items, state.searchQuery, state.sortBy, state.sortOrder]);

  // Subscribe to file-related events
  useEffect(() => {
    const unsubscribe = subscribe('files:*', (event) => {
      switch (event.type) {
        case 'files:uploaded':
        case 'files:deleted':
        case 'files:moved':
        case 'files:copied':
        case 'folder:created':
        case 'file:renamed':
          refetchFiles();
          break;
      }
    });

    return unsubscribe;
  }, [subscribe, refetchFiles]);

  // Action handlers
  const setCurrentPath = useCallback((path: string) => {
    dispatch({ type: 'SET_CURRENT_PATH', payload: path });
  }, []);

  const setSelectedFiles = useCallback((fileIds: string[]) => {
    dispatch({ type: 'SET_SELECTED_FILES', payload: fileIds });
  }, []);

  const setViewMode = useCallback((mode: 'grid' | 'list') => {
    dispatch({ type: 'SET_VIEW_MODE', payload: mode });
    localStorage.setItem('fileViewMode', mode);
  }, []);

  const setSortBy = useCallback((sortBy: 'name' | 'size' | 'date' | 'type') => {
    dispatch({ type: 'SET_SORT_BY', payload: sortBy });
  }, []);

  const setSortOrder = useCallback((order: 'asc' | 'desc') => {
    dispatch({ type: 'SET_SORT_ORDER', payload: order });
  }, []);

  // Removed unused setSearchQuery function

  const clearSelection = useCallback(() => {
    dispatch({ type: 'CLEAR_SELECTION' });
  }, []);

  const refreshFiles = useCallback(async () => {
    await refetchFiles();
  }, [refetchFiles]);

  // File operation handlers (these will be implemented by useFileOperations hook)
  const uploadFiles = useCallback(async () => {
    // This will be overridden by components using useFileOperations
    console.warn('uploadFiles not implemented - use useFileOperations hook');
  }, []);

  const downloadFiles = useCallback(async () => {
    console.warn('downloadFiles not implemented - use useFileOperations hook');
  }, []);

  const createFolder = useCallback(async () => {
    console.warn('createFolder not implemented - use useFileOperations hook');
  }, []);

  const renameFile = useCallback(async () => {
    console.warn('renameFile not implemented - use useFileOperations hook');
  }, []);

  const moveFiles = useCallback(async () => {
    console.warn('moveFiles not implemented - use useFileOperations hook');
  }, []);

  const copyFiles = useCallback(async () => {
    console.warn('copyFiles not implemented - use useFileOperations hook');
  }, []);

  const deleteFiles = useCallback(async () => {
    console.warn('deleteFiles not implemented - use useFileOperations hook');
  }, []);

  const shareFiles = useCallback(async () => {
    console.warn('shareFiles not implemented - use useFileOperations hook');
  }, []);

  // Load saved preferences
  useEffect(() => {
    const savedViewMode = localStorage.getItem('fileViewMode') as 'grid' | 'list';
    if (savedViewMode) {
      dispatch({ type: 'SET_VIEW_MODE', payload: savedViewMode });
    }
  }, []);

  const contextValue: FileContextType = {
    // Current state
    currentPath: state.currentPath,
    selectedFiles: state.selectedFiles,
    viewMode: state.viewMode,
    sortBy: state.sortBy,
    sortOrder: state.sortOrder,
    
    // Data
    files,
    loading: filesLoading,
    error: filesError?.message || null,
    uploadProgress: state.uploadProgress,
    operations: state.operations,
    storageQuota: storageQuota || null,
    
    // Actions
    setCurrentPath,
    setSelectedFiles,
    setViewMode,
    setSortBy,
    setSortOrder,
    
    // File operations (placeholders - should be overridden by useFileOperations)
    uploadFiles,
    downloadFiles,
    createFolder,
    renameFile,
    moveFiles,
    copyFiles,
    deleteFiles,
    shareFiles,
    
    // Utility
    refreshFiles,
    clearSelection,
  };

  return (
    <FileContext.Provider value={contextValue}>
      {children}
    </FileContext.Provider>
  );
};