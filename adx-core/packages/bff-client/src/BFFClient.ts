import { WorkflowResponse, WorkflowProgress } from './types';

export class BFFClient {
  private baseUrl: string;
  private tenantId?: string;
  private authToken?: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  setTenantContext(tenantId: string, authToken: string) {
    this.tenantId = tenantId;
    this.authToken = authToken;
  }

  async initiateWorkflow<T>(
    workflowType: string,
    request: any,
    options: { synchronous?: boolean } = {}
  ): Promise<WorkflowResponse<T>> {
    const response = await fetch(`${this.baseUrl}/workflows/${workflowType}`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${this.authToken}`,
        'X-Tenant-ID': this.tenantId || '',
      },
      body: JSON.stringify(request),
    });

    if (!response.ok) {
      throw new Error(`Workflow initiation failed: ${response.statusText}`);
    }

    const result = await response.json();

    if (result.operationId && !options.synchronous) {
      return {
        type: 'async',
        operationId: result.operationId,
        statusUrl: result.statusUrl,
        streamUrl: result.streamUrl,
      };
    } else {
      return {
        type: 'sync',
        data: result.data || result,
      };
    }
  }

  async getAggregatedData<T>(
    endpoint: string,
    options: { cache?: boolean; ttl?: number } = {}
  ): Promise<T> {
    const response = await fetch(`${this.baseUrl}${endpoint}`, {
      headers: {
        'Authorization': `Bearer ${this.authToken}`,
        'X-Tenant-ID': this.tenantId || '',
        'X-Cache-TTL': options.ttl?.toString() || '300',
      },
    });

    if (!response.ok) {
      throw new Error(`BFF request failed: ${response.statusText}`);
    }

    return response.json();
  }

  async pollWorkflowStatus(
    operationId: string,
    onProgress?: (progress: WorkflowProgress) => void
  ): Promise<any> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const response = await fetch(`${this.baseUrl}/workflows/${operationId}/status`, {
            headers: {
              'Authorization': `Bearer ${this.authToken}`,
              'X-Tenant-ID': this.tenantId || '',
            },
          });

          if (!response.ok) {
            throw new Error(`Status check failed: ${response.statusText}`);
          }

          const status = await response.json();

          if (onProgress && status.progress) {
            onProgress(status.progress);
          }

          switch (status.status) {
            case 'completed':
              resolve(status.result);
              break;
            case 'failed':
              reject(new Error(status.error || 'Workflow failed'));
              break;
            case 'running':
            case 'pending':
              setTimeout(poll, 1000);
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