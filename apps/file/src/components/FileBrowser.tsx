import React, { useState, useCallback } from 'react';
import { 
  Grid, 
  List, 
  Search, 
  Filter, 
  SortAsc, 
  SortDesc, 
  Folder, 
  File, 
  Image, 
  Video, 
  Music, 
  FileText,
  Archive,
  Code,
  MoreHorizontal,
  Download,
  Trash2
} from 'lucide-react';
import { Button, Input } from '@adx-core/design-system';
import { useFile } from '../hooks/useFile';
import { useFileOperations } from '../hooks/useFileOperations';
import type { FileItem } from '../types/file';
import { 
  formatFileSize, 
  formatDate, 
  getFileIcon, 
  parsePath, 
  buildPath 
} from '../utils/fileUtils';

interface FileBrowserProps {
  className?: string;
  onFileSelect?: (file: FileItem) => void;
  onFileDoubleClick?: (file: FileItem) => void;
  selectable?: boolean;
  multiSelect?: boolean;
}

export const FileBrowser: React.FC<FileBrowserProps> = ({
  className = '',
  onFileSelect,
  onFileDoubleClick,
  selectable = true,
  multiSelect = true,
}) => {
  const {
    files,
    loading,
    error,
    currentPath,
    selectedFiles,
    viewMode,
    sortBy,
    sortOrder,
    setCurrentPath,
    setSelectedFiles,
    setViewMode,
    setSortBy,
    setSortOrder,
    clearSelection,
  } = useFile();

  const { downloadFiles, deleteFiles } = useFileOperations();
  const [searchQuery, setSearchQuery] = useState('');
  const [showFilters, setShowFilters] = useState(false);

  // Navigation
  const navigateToPath = useCallback((path: string) => {
    setCurrentPath(path);
    clearSelection();
  }, [setCurrentPath, clearSelection]);



  // File selection
  const handleFileClick = useCallback((file: FileItem, event: React.MouseEvent) => {
    if (!selectable) return;

    if (multiSelect && (event.ctrlKey || event.metaKey)) {
      // Toggle selection
      const newSelection = selectedFiles.includes(file.id)
        ? selectedFiles.filter(id => id !== file.id)
        : [...selectedFiles, file.id];
      setSelectedFiles(newSelection);
    } else if (multiSelect && event.shiftKey && selectedFiles.length > 0) {
      // Range selection
      const lastSelectedIndex = files.findIndex(f => f.id === selectedFiles[selectedFiles.length - 1]);
      const currentIndex = files.findIndex(f => f.id === file.id);
      
      if (lastSelectedIndex !== -1 && currentIndex !== -1) {
        const start = Math.min(lastSelectedIndex, currentIndex);
        const end = Math.max(lastSelectedIndex, currentIndex);
        const rangeFiles = files.slice(start, end + 1).map(f => f.id);
        setSelectedFiles([...new Set([...selectedFiles, ...rangeFiles])]);
      }
    } else {
      // Single selection
      setSelectedFiles([file.id]);
    }

    onFileSelect?.(file);
  }, [files, selectedFiles, setSelectedFiles, selectable, multiSelect, onFileSelect]);

  const handleFileDoubleClick = useCallback((file: FileItem) => {
    if (file.type === 'folder') {
      navigateToPath(file.path);
    } else {
      onFileDoubleClick?.(file);
    }
  }, [navigateToPath, onFileDoubleClick]);

  // Sorting
  const handleSort = useCallback((newSortBy: typeof sortBy) => {
    if (sortBy === newSortBy) {
      setSortOrder(sortOrder === 'asc' ? 'desc' : 'asc');
    } else {
      setSortBy(newSortBy);
      setSortOrder('asc');
    }
  }, [sortBy, sortOrder, setSortBy, setSortOrder]);

  // File operations
  const handleDownload = useCallback(async () => {
    if (selectedFiles.length > 0) {
      try {
        await downloadFiles(selectedFiles);
      } catch (error) {
        console.error('Download failed:', error);
        // You could show a toast notification here
      }
    }
  }, [selectedFiles, downloadFiles]);

  const handleDelete = useCallback(async () => {
    if (selectedFiles.length > 0 && confirm('Are you sure you want to delete the selected files?')) {
      try {
        await deleteFiles(selectedFiles);
        clearSelection();
      } catch (error) {
        console.error('Delete failed:', error);
        // You could show a toast notification here
      }
    }
  }, [selectedFiles, deleteFiles, clearSelection]);

  // Filter files based on search
  const filteredFiles = React.useMemo(() => {
    if (!searchQuery.trim()) return files;
    
    const query = searchQuery.toLowerCase();
    return files.filter(file => 
      file.name.toLowerCase().includes(query) ||
      (file.tags && file.tags.some(tag => tag.toLowerCase().includes(query)))
    );
  }, [files, searchQuery]);

  if (error) {
    return (
      <div className="flex items-center justify-center h-64 text-red-600">
        <p>Error loading files: {error}</p>
      </div>
    );
  }

  return (
    <div className={`file-browser ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        {/* Breadcrumb */}
        <nav className="flex items-center space-x-2 text-sm">
          <button
            onClick={() => navigateToPath('/')}
            className="text-blue-600 hover:text-blue-800"
          >
            Home
          </button>
          {parsePath(currentPath).map((segment, index, array) => (
            <React.Fragment key={index}>
              <span className="text-gray-400">/</span>
              <button
                onClick={() => navigateToPath(buildPath(array.slice(0, index + 1)))}
                className={`${
                  index === array.length - 1 
                    ? 'text-gray-900 font-medium' 
                    : 'text-blue-600 hover:text-blue-800'
                }`}
              >
                {segment}
              </button>
            </React.Fragment>
          ))}
        </nav>

        {/* Actions */}
        <div className="flex items-center space-x-2">
          {selectedFiles.length > 0 && (
            <>
              <Button size="sm" variant="outline" onClick={handleDownload}>
                <Download className="w-4 h-4 mr-1" />
                Download
              </Button>
              <Button size="sm" variant="outline" onClick={handleDelete}>
                <Trash2 className="w-4 h-4 mr-1" />
                Delete
              </Button>
            </>
          )}
          
          <Button
            size="sm"
            variant="outline"
            onClick={() => setViewMode(viewMode === 'grid' ? 'list' : 'grid')}
          >
            {viewMode === 'grid' ? <List className="w-4 h-4" /> : <Grid className="w-4 h-4" />}
          </Button>
        </div>
      </div>

      {/* Search and Filters */}
      <div className="flex items-center space-x-4 mb-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-gray-400" />
          <Input
            placeholder="Search files..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-10"
          />
        </div>
        
        <Button
          variant="outline"
          size="sm"
          onClick={() => setShowFilters(!showFilters)}
        >
          <Filter className="w-4 h-4 mr-1" />
          Filters
        </Button>
      </div>

      {/* Sort Controls */}
      <div className="flex items-center space-x-4 mb-4 text-sm">
        <span className="text-gray-500">Sort by:</span>
        {(['name', 'size', 'date', 'type'] as const).map((option) => (
          <button
            key={option}
            onClick={() => handleSort(option)}
            className={`flex items-center space-x-1 px-2 py-1 rounded ${
              sortBy === option ? 'bg-blue-100 text-blue-700' : 'text-gray-600 hover:text-gray-900'
            }`}
          >
            <span className="capitalize">{option}</span>
            {sortBy === option && (
              sortOrder === 'asc' ? <SortAsc className="w-3 h-3" /> : <SortDesc className="w-3 h-3" />
            )}
          </button>
        ))}
      </div>

      {/* File List */}
      {loading ? (
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        </div>
      ) : filteredFiles.length === 0 ? (
        <div className="flex flex-col items-center justify-center h-64 text-gray-500">
          <Folder className="w-16 h-16 mb-4 text-gray-300" />
          <p className="text-lg font-medium">No files found</p>
          <p className="text-sm">
            {searchQuery ? 'Try adjusting your search terms' : 'This folder is empty'}
          </p>
        </div>
      ) : (
        <div className={viewMode === 'grid' ? 'file-grid' : 'space-y-1'}>
          {filteredFiles.map((file) => (
            <FileItem
              key={file.id}
              file={file}
              selected={selectedFiles.includes(file.id)}
              viewMode={viewMode}
              onClick={(e) => handleFileClick(file, e)}
              onDoubleClick={() => handleFileDoubleClick(file)}
            />
          ))}
        </div>
      )}
    </div>
  );
};

interface FileItemProps {
  file: FileItem;
  selected: boolean;
  viewMode: 'grid' | 'list';
  onClick: (event: React.MouseEvent) => void;
  onDoubleClick: () => void;
}

const FileItem: React.FC<FileItemProps> = ({
  file,
  selected,
  viewMode,
  onClick,
  onDoubleClick,
}) => {
  const getIcon = () => {
    const iconName = getFileIcon(file);
    const iconProps = { className: "w-6 h-6" };
    
    switch (iconName) {
      case 'folder':
        return <Folder {...iconProps} className="w-6 h-6 text-yellow-500" />;
      case 'image':
        return <Image {...iconProps} className="w-6 h-6 text-green-500" />;
      case 'video':
        return <Video {...iconProps} className="w-6 h-6 text-purple-500" />;
      case 'music':
        return <Music {...iconProps} className="w-6 h-6 text-pink-500" />;
      case 'file-text':
        return <FileText {...iconProps} className="w-6 h-6 text-blue-500" />;
      case 'archive':
        return <Archive {...iconProps} className="w-6 h-6 text-orange-500" />;
      case 'code':
        return <Code {...iconProps} className="w-6 h-6 text-gray-600" />;
      default:
        return <File {...iconProps} className="w-6 h-6 text-gray-500" />;
    }
  };

  if (viewMode === 'grid') {
    return (
      <div
        className={`file-item ${selected ? 'selected' : ''}`}
        onClick={onClick}
        onDoubleClick={onDoubleClick}
      >
        <div className="file-icon">
          {file.thumbnail ? (
            <img
              src={file.thumbnail}
              alt={file.name}
              className="w-12 h-12 object-cover rounded"
            />
          ) : (
            getIcon()
          )}
        </div>
        
        <div className="text-center">
          <p className="file-name" title={file.name}>
            {file.name}
          </p>
          {file.type === 'file' && (
            <p className="file-size">
              {formatFileSize(file.size)}
            </p>
          )}
          <p className="file-date">
            {formatDate(file.updatedAt)}
          </p>
        </div>
      </div>
    );
  }

  return (
    <div
      className={`flex items-center space-x-3 p-2 rounded hover:bg-gray-50 cursor-pointer ${
        selected ? 'bg-blue-50 border border-blue-200' : ''
      }`}
      onClick={onClick}
      onDoubleClick={onDoubleClick}
    >
      <div className="flex-shrink-0">
        {getIcon()}
      </div>
      
      <div className="flex-1 min-w-0">
        <p className="text-sm font-medium text-gray-900 truncate">
          {file.name}
        </p>
        <p className="text-xs text-gray-500">
          {file.type === 'file' ? formatFileSize(file.size) : 'Folder'} • {formatDate(file.updatedAt)}
        </p>
      </div>
      
      <div className="flex-shrink-0">
        <button className="p-1 rounded hover:bg-gray-200">
          <MoreHorizontal className="w-4 h-4 text-gray-400" />
        </button>
      </div>
    </div>
  );
};