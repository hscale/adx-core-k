import { 
  Module, 
  ModuleSearchFilters, 
  ModuleSearchResults, 
  ModuleInstallRequest, 
  ModuleInstallResponse,
  ModuleConfiguration,
  ModuleDevelopmentProject,
  WorkflowResponse,
  WorkflowProgress
} from '../types/module';

export class ModuleBFFClient {
  private baseUrl: string;
  private tenantId: string | null = null;
  private authToken: string | null = null;

  constructor(baseUrl: string = 'http://localhost:4006') {
    this.baseUrl = baseUrl;
  }

  setTenantContext(tenantId: string, authToken: string) {
    this.tenantId = tenantId;
    this.authToken = authToken;
  }

  private getHeaders(): Record<string, string> {
    const headers: Record<string, string> = {
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

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      ...options,
      headers: {
        ...this.getHeaders(),
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ message: response.statusText }));
      throw new Error(error.message || `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }

  // Marketplace operations
  async searchModules(
    query?: string,
    filters?: ModuleSearchFilters,
    page: number = 1,
    pageSize: number = 20
  ): Promise<ModuleSearchResults> {
    const params = new URLSearchParams({
      page: page.toString(),
      pageSize: pageSize.toString(),
    });

    if (query) {
      params.append('q', query);
    }

    if (filters) {
      Object.entries(filters).forEach(([key, value]) => {
        if (value !== undefined && value !== null) {
          params.append(key, value.toString());
        }
      });
    }

    return this.request<ModuleSearchResults>(`/api/marketplace/search?${params}`);
  }

  async getModule(moduleId: string): Promise<Module> {
    return this.request<Module>(`/api/marketplace/modules/${moduleId}`);
  }

  async getFeaturedModules(limit: number = 10): Promise<Module[]> {
    return this.request<Module[]>(`/api/marketplace/featured?limit=${limit}`);
  }

  async getTrendingModules(limit: number = 10): Promise<Module[]> {
    return this.request<Module[]>(`/api/marketplace/trending?limit=${limit}`);
  }

  async getRecommendedModules(limit: number = 10): Promise<Module[]> {
    return this.request<Module[]>(`/api/marketplace/recommended?limit=${limit}`);
  }

  // Module installation and management
  async installModule(request: ModuleInstallRequest): Promise<WorkflowResponse<ModuleInstallResponse>> {
    return this.request<WorkflowResponse<ModuleInstallResponse>>('/api/workflows/install-module', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async uninstallModule(moduleId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/uninstall-module`, {
      method: 'POST',
      body: JSON.stringify({ moduleId }),
    });
  }

  async activateModule(moduleId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/activate-module`, {
      method: 'POST',
      body: JSON.stringify({ moduleId }),
    });
  }

  async deactivateModule(moduleId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/deactivate-module`, {
      method: 'POST',
      body: JSON.stringify({ moduleId }),
    });
  }

  async getInstalledModules(): Promise<Module[]> {
    return this.request<Module[]>('/api/modules/installed');
  }

  async getModuleConfiguration(moduleId: string): Promise<ModuleConfiguration> {
    return this.request<ModuleConfiguration>(`/api/modules/${moduleId}/configuration`);
  }

  async updateModuleConfiguration(
    moduleId: string,
    configuration: Partial<ModuleConfiguration>
  ): Promise<ModuleConfiguration> {
    return this.request<ModuleConfiguration>(`/api/modules/${moduleId}/configuration`, {
      method: 'PUT',
      body: JSON.stringify(configuration),
    });
  }

  // Module development
  async getDevelopmentProjects(): Promise<ModuleDevelopmentProject[]> {
    return this.request<ModuleDevelopmentProject[]>('/api/development/projects');
  }

  async createDevelopmentProject(
    project: Omit<ModuleDevelopmentProject, 'id' | 'created' | 'lastModified'>
  ): Promise<ModuleDevelopmentProject> {
    return this.request<ModuleDevelopmentProject>('/api/development/projects', {
      method: 'POST',
      body: JSON.stringify(project),
    });
  }

  async getDevelopmentProject(projectId: string): Promise<ModuleDevelopmentProject> {
    return this.request<ModuleDevelopmentProject>(`/api/development/projects/${projectId}`);
  }

  async updateDevelopmentProject(
    projectId: string,
    updates: Partial<ModuleDevelopmentProject>
  ): Promise<ModuleDevelopmentProject> {
    return this.request<ModuleDevelopmentProject>(`/api/development/projects/${projectId}`, {
      method: 'PUT',
      body: JSON.stringify(updates),
    });
  }

  async testModule(projectId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/test-module`, {
      method: 'POST',
      body: JSON.stringify({ projectId }),
    });
  }

  async publishModule(projectId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/publish-module`, {
      method: 'POST',
      body: JSON.stringify({ projectId }),
    });
  }

  // Workflow status polling
  async pollWorkflowStatus(
    operationId: string,
    onProgress?: (progress: WorkflowProgress) => void
  ): Promise<any> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const response = await this.request<{
            status: string;
            progress?: WorkflowProgress;
            result?: any;
            error?: string;
          }>(`/api/workflows/${operationId}/status`);

          if (onProgress && response.progress) {
            onProgress(response.progress);
          }

          switch (response.status) {
            case 'completed':
              resolve(response.result);
              break;
            case 'failed':
              reject(new Error(response.error || 'Workflow failed'));
              break;
            case 'running':
            case 'pending':
              setTimeout(poll, 1000);
              break;
            default:
              reject(new Error(`Unknown workflow status: ${response.status}`));
          }
        } catch (error) {
          reject(error);
        }
      };

      poll();
    });
  }

  // Health check
  async healthCheck(): Promise<{ status: string; timestamp: string }> {
    return this.request<{ status: string; timestamp: string }>('/api/health');
  }
}