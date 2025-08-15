import { FileItem } from '../types/file';

export const formatFileSize = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes';

  const k = 1024;
  const sizes = ['Bytes', 'KB', 'MB', 'GB', 'TB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));

  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
};

export const formatDate = (dateString: string): string => {
  const date = new Date(dateString);
  const now = new Date();
  const diffInHours = (now.getTime() - date.getTime()) / (1000 * 60 * 60);

  if (diffInHours < 1) {
    const diffInMinutes = Math.floor(diffInHours * 60);
    return `${diffInMinutes} minute${diffInMinutes !== 1 ? 's' : ''} ago`;
  } else if (diffInHours < 24) {
    const hours = Math.floor(diffInHours);
    return `${hours} hour${hours !== 1 ? 's' : ''} ago`;
  } else if (diffInHours < 24 * 7) {
    const days = Math.floor(diffInHours / 24);
    return `${days} day${days !== 1 ? 's' : ''} ago`;
  } else {
    return date.toLocaleDateString();
  }
};

export const getFileIcon = (file: FileItem): string => {
  if (file.type === 'folder') {
    return 'folder';
  }

  if (!file.mimeType) {
    return 'file';
  }

  const mimeType = file.mimeType.toLowerCase();

  // Images
  if (mimeType.startsWith('image/')) {
    return 'image';
  }

  // Videos
  if (mimeType.startsWith('video/')) {
    return 'video';
  }

  // Audio
  if (mimeType.startsWith('audio/')) {
    return 'music';
  }

  // Documents
  if (mimeType.includes('pdf')) {
    return 'file-text';
  }

  if (mimeType.includes('word') || mimeType.includes('document')) {
    return 'file-text';
  }

  if (mimeType.includes('sheet') || mimeType.includes('excel')) {
    return 'file-spreadsheet';
  }

  if (mimeType.includes('presentation') || mimeType.includes('powerpoint')) {
    return 'presentation';
  }

  // Archives
  if (mimeType.includes('zip') || mimeType.includes('rar') || mimeType.includes('tar')) {
    return 'archive';
  }

  // Code files
  if (mimeType.includes('javascript') || mimeType.includes('typescript')) {
    return 'code';
  }

  if (mimeType.includes('html') || mimeType.includes('css')) {
    return 'code';
  }

  if (mimeType.includes('json') || mimeType.includes('xml')) {
    return 'code';
  }

  // Default
  return 'file';
};

export const getFileExtension = (filename: string): string => {
  const lastDotIndex = filename.lastIndexOf('.');
  if (lastDotIndex === -1 || lastDotIndex === filename.length - 1) {
    return '';
  }
  return filename.substring(lastDotIndex + 1).toLowerCase();
};

export const isImageFile = (file: FileItem): boolean => {
  return file.mimeType?.startsWith('image/') || false;
};

export const isVideoFile = (file: FileItem): boolean => {
  return file.mimeType?.startsWith('video/') || false;
};

export const isAudioFile = (file: FileItem): boolean => {
  return file.mimeType?.startsWith('audio/') || false;
};

export const isDocumentFile = (file: FileItem): boolean => {
  if (!file.mimeType) return false;
  
  const documentTypes = [
    'application/pdf',
    'application/msword',
    'application/vnd.openxmlformats-officedocument.wordprocessingml.document',
    'application/vnd.ms-excel',
    'application/vnd.openxmlformats-officedocument.spreadsheetml.sheet',
    'application/vnd.ms-powerpoint',
    'application/vnd.openxmlformats-officedocument.presentationml.presentation',
    'text/plain',
    'text/html',
    'text/css',
    'application/json',
    'application/xml',
  ];

  return documentTypes.includes(file.mimeType);
};

export const canPreview = (file: FileItem): boolean => {
  return isImageFile(file) || isVideoFile(file) || isAudioFile(file) || isDocumentFile(file);
};

export const sortFiles = (
  files: FileItem[], 
  sortBy: 'name' | 'size' | 'date' | 'type', 
  sortOrder: 'asc' | 'desc'
): FileItem[] => {
  const sorted = [...files].sort((a, b) => {
    // Always put folders first
    if (a.type === 'folder' && b.type === 'file') return -1;
    if (a.type === 'file' && b.type === 'folder') return 1;

    let comparison = 0;

    switch (sortBy) {
      case 'name':
        comparison = a.name.localeCompare(b.name);
        break;
      case 'size':
        comparison = a.size - b.size;
        break;
      case 'date':
        comparison = new Date(a.updatedAt).getTime() - new Date(b.updatedAt).getTime();
        break;
      case 'type':
        const aExt = getFileExtension(a.name);
        const bExt = getFileExtension(b.name);
        comparison = aExt.localeCompare(bExt);
        break;
    }

    return sortOrder === 'asc' ? comparison : -comparison;
  });

  return sorted;
};

export const filterFiles = (files: FileItem[], query: string): FileItem[] => {
  if (!query.trim()) return files;

  const lowercaseQuery = query.toLowerCase();
  
  return files.filter(file => 
    file.name.toLowerCase().includes(lowercaseQuery) ||
    (file.tags && file.tags.some(tag => tag.toLowerCase().includes(lowercaseQuery)))
  );
};

export const buildPath = (segments: string[]): string => {
  return '/' + segments.filter(segment => segment && segment !== '/').join('/');
};

export const parsePath = (path: string): string[] => {
  return path.split('/').filter(segment => segment && segment !== '/');
};

export const getParentPath = (path: string): string => {
  const segments = parsePath(path);
  if (segments.length <= 1) return '/';
  return buildPath(segments.slice(0, -1));
};

export const validateFileName = (name: string): { valid: boolean; error?: string } => {
  if (!name.trim()) {
    return { valid: false, error: 'File name cannot be empty' };
  }

  if (name.length > 255) {
    return { valid: false, error: 'File name is too long (max 255 characters)' };
  }

  // Check for invalid characters
  const invalidChars = /[<>:"/\\|?*\x00-\x1f]/;
  if (invalidChars.test(name)) {
    return { valid: false, error: 'File name contains invalid characters' };
  }

  // Check for reserved names (Windows)
  const reservedNames = /^(CON|PRN|AUX|NUL|COM[1-9]|LPT[1-9])(\.|$)/i;
  if (reservedNames.test(name)) {
    return { valid: false, error: 'File name is reserved' };
  }

  return { valid: true };
};

export const generateShareUrl = (shareId: string, baseUrl?: string): string => {
  const base = baseUrl || window.location.origin;
  return `${base}/shared/${shareId}`;
};

export const downloadBlob = (blob: Blob, filename: string): void => {
  const url = URL.createObjectURL(blob);
  const a = document.createElement('a');
  a.href = url;
  a.download = filename;
  document.body.appendChild(a);
  a.click();
  document.body.removeChild(a);
  URL.revokeObjectURL(url);
};

export const copyToClipboard = async (text: string): Promise<boolean> => {
  try {
    await navigator.clipboard.writeText(text);
    return true;
  } catch (error) {
    // Fallback for older browsers
    const textArea = document.createElement('textarea');
    textArea.value = text;
    document.body.appendChild(textArea);
    textArea.select();
    const success = document.execCommand('copy');
    document.body.removeChild(textArea);
    return success;
  }
};

export const calculateUploadTime = (fileSize: number, uploadSpeed: number): number => {
  if (uploadSpeed === 0) return 0;
  return Math.ceil((fileSize - (fileSize * 0.1)) / uploadSpeed); // Rough estimate
};

export const formatUploadSpeed = (bytesPerSecond: number): string => {
  return `${formatFileSize(bytesPerSecond)}/s`;
};