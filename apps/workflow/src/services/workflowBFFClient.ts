import { 
  Workflow, 
  WorkflowListResponse, 
  WorkflowSearchParams, 
  WorkflowAnalytics, 
  WorkflowTemplate,
  WorkflowMetrics 
} from '../types';

export class WorkflowBFFClient {
  private baseUrl: string;
  private tenantId: string | null = null;
  private authToken: string | null = null;

  constructor(baseUrl: string = 'http://localhost:4005') {
    this.baseUrl = baseUrl;
  }

  setTenantContext(tenantId: string, authToken: string) {
    this.tenantId = tenantId;
    this.authToken = authToken;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    
    const response = await fetch(url, {
      ...options,
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.authToken}`,
        'X-Tenant-ID': this.tenantId || '',
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({ 
        message: response.statusText 
      }));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    return response.json();
  }

  // Workflow listing and search
  async getWorkflows(params: WorkflowSearchParams = {}): Promise<WorkflowListResponse> {
    const searchParams = new URLSearchParams();
    
    if (params.query) searchParams.set('query', params.query);
    if (params.filter?.status) {
      searchParams.set('status', params.filter.status.join(','));
    }
    if (params.filter?.type) {
      searchParams.set('type', params.filter.type.join(','));
    }
    if (params.filter?.dateRange) {
      searchParams.set('startDate', params.filter.dateRange.start);
      searchParams.set('endDate', params.filter.dateRange.end);
    }
    if (params.sortBy) searchParams.set('sortBy', params.sortBy);
    if (params.sortOrder) searchParams.set('sortOrder', params.sortOrder);
    if (params.page) searchParams.set('page', params.page.toString());
    if (params.limit) searchParams.set('limit', params.limit.toString());

    const queryString = searchParams.toString();
    const endpoint = `/api/workflows${queryString ? `?${queryString}` : ''}`;
    
    return this.request<WorkflowListResponse>(endpoint);
  }

  // Get specific workflow details
  async getWorkflow(workflowId: string): Promise<Workflow> {
    return this.request<Workflow>(`/api/workflows/${workflowId}`);
  }

  // Get workflow status with real-time updates
  async getWorkflowStatus(workflowId: string): Promise<Workflow> {
    return this.request<Workflow>(`/api/workflows/${workflowId}/status`);
  }

  // Cancel workflow
  async cancelWorkflow(workflowId: string, reason?: string): Promise<void> {
    await this.request(`/api/workflows/${workflowId}/cancel`, {
      method: 'POST',
      body: JSON.stringify({ reason }),
    });
  }

  // Retry failed workflow
  async retryWorkflow(workflowId: string): Promise<{ newWorkflowId: string }> {
    return this.request<{ newWorkflowId: string }>(`/api/workflows/${workflowId}/retry`, {
      method: 'POST',
    });
  }

  // Get workflow templates
  async getWorkflowTemplates(): Promise<WorkflowTemplate[]> {
    return this.request<WorkflowTemplate[]>('/api/workflow-templates');
  }

  // Start workflow from template
  async startWorkflowFromTemplate(
    templateId: string, 
    parameters: Record<string, any>
  ): Promise<{ workflowId: string }> {
    return this.request<{ workflowId: string }>('/api/workflows/start', {
      method: 'POST',
      body: JSON.stringify({
        templateId,
        parameters,
      }),
    });
  }

  // Get workflow analytics
  async getWorkflowAnalytics(timeRange: string = '7d'): Promise<WorkflowAnalytics> {
    return this.request<WorkflowAnalytics>(`/api/analytics/workflows?timeRange=${timeRange}`);
  }

  // Get workflow metrics
  async getWorkflowMetrics(): Promise<WorkflowMetrics> {
    return this.request<WorkflowMetrics>('/api/metrics/workflows');
  }

  // Get running workflows
  async getRunningWorkflows(): Promise<Workflow[]> {
    return this.request<Workflow[]>('/api/workflows/running');
  }

  // Get workflow history for a specific type
  async getWorkflowHistory(
    workflowType: string, 
    limit: number = 50
  ): Promise<Workflow[]> {
    return this.request<Workflow[]>(
      `/api/workflows/history/${workflowType}?limit=${limit}`
    );
  }

  // Stream workflow progress (Server-Sent Events)
  streamWorkflowProgress(
    workflowId: string,
    onProgress: (workflow: Workflow) => void,
    onError: (error: Error) => void
  ): () => void {
    const eventSource = new EventSource(
      `${this.baseUrl}/api/workflows/${workflowId}/stream`,
      {
        headers: {
          'Authorization': `Bearer ${this.authToken}`,
          'X-Tenant-ID': this.tenantId || '',
        } as any,
      }
    );

    eventSource.onmessage = (event) => {
      try {
        const workflow = JSON.parse(event.data);
        onProgress(workflow);
      } catch (error) {
        onError(new Error('Failed to parse workflow progress data'));
      }
    };

    eventSource.onerror = () => {
      onError(new Error('Workflow progress stream connection failed'));
    };

    return () => {
      eventSource.close();
    };
  }

  // Bulk operations
  async bulkCancelWorkflows(workflowIds: string[], reason?: string): Promise<void> {
    await this.request('/api/workflows/bulk/cancel', {
      method: 'POST',
      body: JSON.stringify({
        workflowIds,
        reason,
      }),
    });
  }

  async bulkRetryWorkflows(workflowIds: string[]): Promise<{ results: Array<{ originalId: string; newId: string }> }> {
    return this.request<{ results: Array<{ originalId: string; newId: string }> }>('/api/workflows/bulk/retry', {
      method: 'POST',
      body: JSON.stringify({ workflowIds }),
    });
  }

  // Health check
  async healthCheck(): Promise<{ status: string; timestamp: string }> {
    return this.request<{ status: string; timestamp: string }>('/api/health');
  }
}