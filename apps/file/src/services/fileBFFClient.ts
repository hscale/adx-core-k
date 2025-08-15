import { 
  FileItem, 
  FileUploadProgress, 
  FileSearchFilters, 
  FileSearchResult, 
  FileShare, 
  FilePermissions, 
  StorageQuota, 
  FileOperation,
  FileBFFClient 
} from '../types/file';

class FileBFFClientImpl implements FileBFFClient {
  private baseUrl: string;
  private tenantId: string | null = null;
  private authToken: string | null = null;

  constructor(baseUrl: string = 'http://localhost:4003') {
    this.baseUrl = baseUrl;
  }

  setTenantContext(tenantId: string, authToken: string) {
    this.tenantId = tenantId;
    this.authToken = authToken;
  }

  private getHeaders(): HeadersInit {
    const headers: HeadersInit = {
      'Content-Type': 'application/json',
    };

    if (this.authToken) {
      headers['Authorization'] = `Bearer ${this.authToken}`;
    }

    if (this.tenantId) {
      headers['X-Tenant-ID'] = this.tenantId;
    }

    return headers;
  }

  private getFormHeaders(): HeadersInit {
    const headers: HeadersInit = {};

    if (this.authToken) {
      headers['Authorization'] = `Bearer ${this.authToken}`;
    }

    if (this.tenantId) {
      headers['X-Tenant-ID'] = this.tenantId;
    }

    return headers;
  }

  private async handleResponse<T>(response: Response): Promise<T> {
    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: response.statusText }));
      throw new Error(error.message || `HTTP ${response.status}: ${response.statusText}`);
    }

    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('application/json')) {
      return response.json();
    }

    return response as unknown as T;
  }

  // File upload operations
  async uploadFile(
    file: File, 
    path: string = '/', 
    onProgress?: (progress: FileUploadProgress) => void
  ): Promise<FileItem> {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('path', path);

    const xhr = new XMLHttpRequest();
    
    return new Promise((resolve, reject) => {
      xhr.upload.addEventListener('progress', (event) => {
        if (event.lengthComputable && onProgress) {
          const progress: FileUploadProgress = {
            fileId: `temp-${Date.now()}`,
            fileName: file.name,
            progress: (event.loaded / event.total) * 100,
            status: 'uploading',
            uploadSpeed: event.loaded / ((Date.now() - startTime) / 1000),
          };
          onProgress(progress);
        }
      });

      xhr.addEventListener('load', () => {
        if (xhr.status >= 200 && xhr.status < 300) {
          try {
            const result = JSON.parse(xhr.responseText);
            resolve(result);
          } catch (error) {
            reject(new Error('Invalid response format'));
          }
        } else {
          reject(new Error(`Upload failed: ${xhr.statusText}`));
        }
      });

      xhr.addEventListener('error', () => {
        reject(new Error('Upload failed'));
      });

      const startTime = Date.now();
      xhr.open('POST', `${this.baseUrl}/api/files/upload`);
      
      // Set headers
      const headers = this.getFormHeaders();
      Object.entries(headers).forEach(([key, value]) => {
        xhr.setRequestHeader(key, value);
      });

      xhr.send(formData);
    });
  }

  async uploadFiles(
    files: File[], 
    path: string = '/', 
    onProgress?: (progress: FileUploadProgress[]) => void
  ): Promise<FileItem[]> {
    const uploadPromises = files.map((file, index) => 
      this.uploadFile(file, path, (progress) => {
        if (onProgress) {
          const allProgress = files.map((f, i) => 
            i === index ? progress : {
              fileId: `temp-${Date.now()}-${i}`,
              fileName: f.name,
              progress: 0,
              status: 'pending' as const,
            }
          );
          onProgress(allProgress);
        }
      })
    );

    return Promise.all(uploadPromises);
  }

  async downloadFile(fileId: string): Promise<Blob> {
    const response = await fetch(`${this.baseUrl}/api/files/${fileId}/download`, {
      headers: this.getHeaders(),
    });

    if (!response.ok) {
      throw new Error(`Download failed: ${response.statusText}`);
    }

    return response.blob();
  }

  async downloadFiles(fileIds: string[]): Promise<Blob> {
    const response = await fetch(`${this.baseUrl}/api/files/download`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ fileIds }),
    });

    if (!response.ok) {
      throw new Error(`Download failed: ${response.statusText}`);
    }

    return response.blob();
  }

  // File management operations
  async getFile(fileId: string): Promise<FileItem> {
    const response = await fetch(`${this.baseUrl}/api/files/${fileId}`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<FileItem>(response);
  }

  async getFiles(path: string = '/', filters?: FileSearchFilters): Promise<FileSearchResult> {
    const params = new URLSearchParams();
    params.append('path', path);
    
    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          if (Array.isArray(value)) {
            value.forEach(v => params.append(`${key}[]`, v.toString()));
          } else {
            params.append(key, value.toString());
          }
        }
      });
    }

    const response = await fetch(`${this.baseUrl}/api/files?${params.toString()}`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<FileSearchResult>(response);
  }

  async searchFiles(filters: FileSearchFilters): Promise<FileSearchResult> {
    const response = await fetch(`${this.baseUrl}/api/files/search`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify(filters),
    });

    return this.handleResponse<FileSearchResult>(response);
  }

  async createFolder(name: string, parentPath: string = '/'): Promise<FileItem> {
    const response = await fetch(`${this.baseUrl}/api/files/folders`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ name, parentPath }),
    });

    return this.handleResponse<FileItem>(response);
  }

  async renameFile(fileId: string, newName: string): Promise<FileItem> {
    const response = await fetch(`${this.baseUrl}/api/files/${fileId}/rename`, {
      method: 'PUT',
      headers: this.getHeaders(),
      body: JSON.stringify({ name: newName }),
    });

    return this.handleResponse<FileItem>(response);
  }

  async moveFiles(fileIds: string[], targetPath: string): Promise<FileOperation> {
    const response = await fetch(`${this.baseUrl}/api/files/move`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ fileIds, targetPath }),
    });

    return this.handleResponse<FileOperation>(response);
  }

  async copyFiles(fileIds: string[], targetPath: string): Promise<FileOperation> {
    const response = await fetch(`${this.baseUrl}/api/files/copy`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ fileIds, targetPath }),
    });

    return this.handleResponse<FileOperation>(response);
  }

  async deleteFiles(fileIds: string[]): Promise<FileOperation> {
    const response = await fetch(`${this.baseUrl}/api/files/delete`, {
      method: 'DELETE',
      headers: this.getHeaders(),
      body: JSON.stringify({ fileIds }),
    });

    return this.handleResponse<FileOperation>(response);
  }

  // Sharing and permissions
  async shareFile(fileId: string, shareSettings: Partial<FileShare>): Promise<FileShare> {
    const response = await fetch(`${this.baseUrl}/api/files/${fileId}/share`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify(shareSettings),
    });

    return this.handleResponse<FileShare>(response);
  }

  async updateFilePermissions(fileId: string, permissions: Partial<FilePermissions>): Promise<FilePermissions> {
    const response = await fetch(`${this.baseUrl}/api/files/${fileId}/permissions`, {
      method: 'PUT',
      headers: this.getHeaders(),
      body: JSON.stringify(permissions),
    });

    return this.handleResponse<FilePermissions>(response);
  }

  async getFileShares(fileId: string): Promise<FileShare[]> {
    const response = await fetch(`${this.baseUrl}/api/files/${fileId}/shares`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<FileShare[]>(response);
  }

  async revokeShare(shareId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/shares/${shareId}`, {
      method: 'DELETE',
      headers: this.getHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  // Storage and operations
  async getStorageQuota(): Promise<StorageQuota> {
    const response = await fetch(`${this.baseUrl}/api/storage/quota`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<StorageQuota>(response);
  }

  async getFileOperations(): Promise<FileOperation[]> {
    const response = await fetch(`${this.baseUrl}/api/operations`, {
      headers: this.getHeaders(),
    });

    return this.handleResponse<FileOperation[]>(response);
  }

  async cancelOperation(operationId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/api/operations/${operationId}/cancel`, {
      method: 'POST',
      headers: this.getHeaders(),
    });

    await this.handleResponse<void>(response);
  }

  // Workflow operations
  async initiateWorkflow<T>(
    workflowType: string,
    request: any,
    options: { synchronous?: boolean } = {}
  ): Promise<{ type: 'sync' | 'async'; data?: T; operationId?: string; statusUrl?: string }> {
    const response = await fetch(`${this.baseUrl}/api/workflows/${workflowType}`, {
      method: 'POST',
      headers: this.getHeaders(),
      body: JSON.stringify({ ...request, synchronous: options.synchronous }),
    });

    const result = await this.handleResponse<any>(response);

    if (result.operationId && !options.synchronous) {
      return {
        type: 'async',
        operationId: result.operationId,
        statusUrl: result.statusUrl,
      };
    } else {
      return {
        type: 'sync',
        data: result.data || result,
      };
    }
  }

  async pollWorkflowStatus(operationId: string): Promise<any> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const response = await fetch(`${this.baseUrl}/api/workflows/${operationId}/status`, {
            headers: this.getHeaders(),
          });

          const status = await this.handleResponse<any>(response);

          switch (status.status) {
            case 'completed':
              resolve(status.result);
              break;
            case 'failed':
              reject(new Error(status.error || 'Workflow failed'));
              break;
            case 'running':
            case 'pending':
              setTimeout(poll, 1000); // Poll every second
              break;
            default:
              reject(new Error(`Unknown workflow status: ${status.status}`));
          }
        } catch (error) {
          reject(error);
        }
      };

      poll();
    });
  }
}

export const fileBFFClient = new FileBFFClientImpl();
export default fileBFFClient;