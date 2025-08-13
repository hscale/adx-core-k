import {
  Tenant,
  TenantMember,
  TenantInvitation,
  CreateTenantRequest,
  UpdateTenantRequest,
  InviteMemberRequest,
  UpdateMemberRequest,
  TenantSwitchRequest,
  TenantSwitchResult,
  WorkflowResponse,
} from '../types';

export class TenantBFFClient {
  private baseUrl: string;
  private tenantId: string | null = null;
  private authToken: string | null = null;

  constructor(baseUrl: string = 'http://localhost:4002') {
    this.baseUrl = baseUrl;
  }

  setContext(tenantId: string, authToken: string) {
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
    const url = `${this.baseUrl}${endpoint}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        ...this.getHeaders(),
        ...options.headers,
      },
    });

    if (!response.ok) {
      const error = await response.json().catch(() => ({
        message: response.statusText,
      }));
      throw new Error(error.message || `HTTP ${response.status}`);
    }

    return response.json();
  }

  // Tenant Management
  async getCurrentTenant(): Promise<Tenant> {
    return this.request<Tenant>('/api/tenant/current');
  }

  async getUserTenants(): Promise<Tenant[]> {
    return this.request<Tenant[]>('/api/tenants');
  }

  async getTenant(tenantId: string): Promise<Tenant> {
    return this.request<Tenant>(`/api/tenants/${tenantId}`);
  }

  async createTenant(request: CreateTenantRequest): Promise<WorkflowResponse<Tenant>> {
    return this.request<WorkflowResponse<Tenant>>('/api/workflows/create-tenant', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  async updateTenant(
    tenantId: string,
    request: UpdateTenantRequest
  ): Promise<Tenant> {
    return this.request<Tenant>(`/api/tenants/${tenantId}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  async deleteTenant(tenantId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/delete-tenant/${tenantId}`, {
      method: 'POST',
    });
  }

  // Tenant Switching
  async switchTenant(request: TenantSwitchRequest): Promise<WorkflowResponse<TenantSwitchResult>> {
    return this.request<WorkflowResponse<TenantSwitchResult>>('/api/workflows/switch-tenant', {
      method: 'POST',
      body: JSON.stringify(request),
    });
  }

  // Member Management
  async getTenantMembers(tenantId?: string): Promise<TenantMember[]> {
    const id = tenantId || this.tenantId;
    return this.request<TenantMember[]>(`/api/tenants/${id}/members`);
  }

  async inviteMember(
    tenantId: string,
    request: InviteMemberRequest
  ): Promise<WorkflowResponse<TenantInvitation>> {
    return this.request<WorkflowResponse<TenantInvitation>>(
      `/api/workflows/invite-member/${tenantId}`,
      {
        method: 'POST',
        body: JSON.stringify(request),
      }
    );
  }

  async updateMember(
    tenantId: string,
    memberId: string,
    request: UpdateMemberRequest
  ): Promise<TenantMember> {
    return this.request<TenantMember>(`/api/tenants/${tenantId}/members/${memberId}`, {
      method: 'PUT',
      body: JSON.stringify(request),
    });
  }

  async removeMember(tenantId: string, memberId: string): Promise<void> {
    return this.request<void>(`/api/tenants/${tenantId}/members/${memberId}`, {
      method: 'DELETE',
    });
  }

  // Invitations
  async getTenantInvitations(tenantId?: string): Promise<TenantInvitation[]> {
    const id = tenantId || this.tenantId;
    return this.request<TenantInvitation[]>(`/api/tenants/${id}/invitations`);
  }

  async cancelInvitation(tenantId: string, invitationId: string): Promise<void> {
    return this.request<void>(`/api/tenants/${tenantId}/invitations/${invitationId}`, {
      method: 'DELETE',
    });
  }

  async resendInvitation(tenantId: string, invitationId: string): Promise<void> {
    return this.request<void>(`/api/tenants/${tenantId}/invitations/${invitationId}/resend`, {
      method: 'POST',
    });
  }

  // Workflow Status
  async getWorkflowStatus(operationId: string): Promise<any> {
    return this.request<any>(`/api/workflows/${operationId}/status`);
  }

  async pollWorkflowStatus(
    operationId: string,
    onProgress?: (progress: any) => void
  ): Promise<any> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const status = await this.getWorkflowStatus(operationId);

          if (onProgress) {
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

  // Cache management
  private cache = new Map<string, { data: any; expiry: number }>();

  private getFromCache<T>(key: string): T | null {
    const cached = this.cache.get(key);
    if (!cached || Date.now() > cached.expiry) {
      this.cache.delete(key);
      return null;
    }
    return cached.data;
  }

  private setCache(key: string, data: any, ttlSeconds: number = 300) {
    const expiry = Date.now() + ttlSeconds * 1000;
    this.cache.set(key, { data, expiry });
  }

  async getCachedData<T>(
    key: string,
    fetcher: () => Promise<T>,
    ttlSeconds: number = 300
  ): Promise<T> {
    const cached = this.getFromCache<T>(key);
    if (cached) {
      return cached;
    }

    const data = await fetcher();
    this.setCache(key, data, ttlSeconds);
    return data;
  }
}

export const tenantBFFClient = new TenantBFFClient();