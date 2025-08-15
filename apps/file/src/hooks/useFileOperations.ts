import { useState, useCallback } from 'react';
import { useMutation, useQueryClient } from '@tanstack/react-query';
import { useEventBus } from '@adx-core/event-bus';
import { fileBFFClient } from '../services/fileBFFClient';
import { FileItem, FileUploadProgress } from '../types/file';
import { downloadBlob } from '../utils/fileUtils';

export const useFileOperations = () => {
  const [uploadProgress, setUploadProgress] = useState<FileUploadProgress[]>([]);
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  // Upload files mutation
  const uploadFilesMutation = useMutation({
    mutationFn: async ({ files, path }: { files: File[]; path?: string }) => {
      return fileBFFClient.uploadFiles(files, path, (progress) => {
        setUploadProgress(progress);
      });
    },
    onSuccess: (uploadedFiles) => {
      // Clear upload progress
      setUploadProgress([]);
      
      // Invalidate file queries
      queryClient.invalidateQueries({ queryKey: ['files'] });
      
      // Emit event for other components
      emit('files:uploaded', { files: uploadedFiles });
    },
    onError: (error) => {
      console.error('Upload failed:', error);
      setUploadProgress([]);
    },
  });

  // Download files mutation
  const downloadFilesMutation = useMutation({
    mutationFn: async ({ fileIds, fileName }: { fileIds: string[]; fileName?: string }) => {
      if (fileIds.length === 1) {
        const blob = await fileBFFClient.downloadFile(fileIds[0]);
        const file = queryClient.getQueryData<FileItem>(['file', fileIds[0]]);
        downloadBlob(blob, file?.name || fileName || 'download');
      } else {
        const blob = await fileBFFClient.downloadFiles(fileIds);
        downloadBlob(blob, fileName || 'files.zip');
      }
    },
    onSuccess: (_, variables) => {
      emit('files:downloaded', { fileIds: variables.fileIds });
    },
  });

  // Create folder mutation
  const createFolderMutation = useMutation({
    mutationFn: async ({ name, parentPath }: { name: string; parentPath?: string }) => {
      return fileBFFClient.createFolder(name, parentPath);
    },
    onSuccess: (folder) => {
      queryClient.invalidateQueries({ queryKey: ['files'] });
      emit('folder:created', { folder });
    },
  });

  // Rename file mutation
  const renameFileMutation = useMutation({
    mutationFn: async ({ fileId, newName }: { fileId: string; newName: string }) => {
      return fileBFFClient.renameFile(fileId, newName);
    },
    onSuccess: (file) => {
      queryClient.invalidateQueries({ queryKey: ['files'] });
      queryClient.setQueryData(['file', file.id], file);
      emit('file:renamed', { file });
    },
  });

  // Move files mutation
  const moveFilesMutation = useMutation({
    mutationFn: async ({ fileIds, targetPath }: { fileIds: string[]; targetPath: string }) => {
      return fileBFFClient.moveFiles(fileIds, targetPath);
    },
    onSuccess: (operation) => {
      queryClient.invalidateQueries({ queryKey: ['files'] });
      emit('files:moved', { operation });
    },
  });

  // Copy files mutation
  const copyFilesMutation = useMutation({
    mutationFn: async ({ fileIds, targetPath }: { fileIds: string[]; targetPath: string }) => {
      return fileBFFClient.copyFiles(fileIds, targetPath);
    },
    onSuccess: (operation) => {
      queryClient.invalidateQueries({ queryKey: ['files'] });
      emit('files:copied', { operation });
    },
  });

  // Delete files mutation
  const deleteFilesMutation = useMutation({
    mutationFn: async ({ fileIds }: { fileIds: string[] }) => {
      return fileBFFClient.deleteFiles(fileIds);
    },
    onSuccess: (operation) => {
      queryClient.invalidateQueries({ queryKey: ['files'] });
      emit('files:deleted', { operation });
    },
  });

  // Share file mutation
  const shareFileMutation = useMutation({
    mutationFn: async ({ fileId, shareSettings }: { fileId: string; shareSettings: any }) => {
      return fileBFFClient.shareFile(fileId, shareSettings);
    },
    onSuccess: (share) => {
      queryClient.invalidateQueries({ queryKey: ['file-shares'] });
      emit('file:shared', { share });
    },
  });

  // Convenience functions
  const uploadFiles = useCallback((files: File[], path?: string) => {
    return uploadFilesMutation.mutateAsync({ files, path });
  }, [uploadFilesMutation]);

  const downloadFiles = useCallback((fileIds: string[], fileName?: string) => {
    return downloadFilesMutation.mutateAsync({ fileIds, fileName });
  }, [downloadFilesMutation]);

  const createFolder = useCallback((name: string, parentPath?: string) => {
    return createFolderMutation.mutateAsync({ name, parentPath });
  }, [createFolderMutation]);

  const renameFile = useCallback((fileId: string, newName: string) => {
    return renameFileMutation.mutateAsync({ fileId, newName });
  }, [renameFileMutation]);

  const moveFiles = useCallback((fileIds: string[], targetPath: string) => {
    return moveFilesMutation.mutateAsync({ fileIds, targetPath });
  }, [moveFilesMutation]);

  const copyFiles = useCallback((fileIds: string[], targetPath: string) => {
    return copyFilesMutation.mutateAsync({ fileIds, targetPath });
  }, [copyFilesMutation]);

  const deleteFiles = useCallback((fileIds: string[]) => {
    return deleteFilesMutation.mutateAsync({ fileIds });
  }, [deleteFilesMutation]);

  const shareFile = useCallback((fileId: string, shareSettings: any) => {
    return shareFileMutation.mutateAsync({ fileId, shareSettings });
  }, [shareFileMutation]);

  return {
    // State
    uploadProgress,
    
    // Mutations
    uploadFilesMutation,
    downloadFilesMutation,
    createFolderMutation,
    renameFileMutation,
    moveFilesMutation,
    copyFilesMutation,
    deleteFilesMutation,
    shareFileMutation,
    
    // Convenience functions
    uploadFiles,
    downloadFiles,
    createFolder,
    renameFile,
    moveFiles,
    copyFiles,
    deleteFiles,
    shareFile,
  };
};