import { 
  Tenant, 
  TenantAnalytics, 
  TenantMembership, 
  TenantSwitchRequest, 
  TenantSwitchResult,
  TenantWorkflowRequest,
  WorkflowResponse,
  AnalyticsPeriod
} from '../types/tenant.js';

export interface ApiClientConfig {
  apiGatewayUrl: string;
  tenantServiceUrl: string;
  userServiceUrl: string;
  authServiceUrl: string;
  workflowServiceUrl: string;
  timeout: number;
}

export class ApiClient {
  constructor(private config: ApiClientConfig) {}

  // Health check
  public async healthCheck(): Promise<string> {
    try {
      const response = await this.makeRequest('GET', `${this.config.apiGatewayUrl}/health`);
      return response.status === 'healthy' ? 'healthy' : 'unhealthy';
    } catch (error) {
      return 'unhealthy';
    }
  }

  // Tenant operations
  public async getTenant(tenantId: string, authToken?: string): Promise<Tenant> {
    const headers: Record<string, string> = {};
    if (authToken) headers.Authorization = `Bearer ${authToken}`;

    return await this.makeRequest(
      'GET',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}`,
      undefined,
      headers
    );
  }

  public async getUserTenants(userId: string, authToken: string): Promise<Tenant[]> {
    return await this.makeRequest(
      'GET',
      `${this.config.userServiceUrl}/api/v1/users/${userId}/tenants`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  public async getTenantMemberships(
    tenantId: string,
    authToken: string,
    page: number = 1,
    limit: number = 50
  ): Promise<{ memberships: TenantMembership[]; total: number; page: number; limit: number }> {
    return await this.makeRequest(
      'GET',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/memberships?page=${page}&limit=${limit}`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  public async getTenantAnalytics(
    tenantId: string,
    period: AnalyticsPeriod,
    authToken: string
  ): Promise<TenantAnalytics> {
    return await this.makeRequest(
      'GET',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/analytics?period=${period}`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  public async getTenantUsage(
    tenantId: string,
    authToken: string
  ): Promise<any> {
    return await this.makeRequest(
      'GET',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/usage`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  public async getTenantConfiguration(
    tenantId: string,
    authToken: string
  ): Promise<any> {
    return await this.makeRequest(
      'GET',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/configuration`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  public async updateTenantConfiguration(
    tenantId: string,
    configuration: any,
    authToken: string
  ): Promise<any> {
    return await this.makeRequest(
      'PUT',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/configuration`,
      configuration,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  // Workflow operations
  public async initiateWorkflow<T>(
    request: TenantWorkflowRequest,
    authToken: string
  ): Promise<WorkflowResponse<T>> {
    return await this.makeRequest(
      'POST',
      `${this.config.apiGatewayUrl}/api/v1/workflows/${request.workflowType}`,
      {
        tenantId: request.tenantId,
        userId: request.userId,
        data: request.data,
        options: request.options,
      },
      { 
        Authorization: `Bearer ${authToken}`,
        'X-Tenant-ID': request.tenantId,
      }
    );
  }

  public async getWorkflowStatus(
    operationId: string,
    authToken: string
  ): Promise<any> {
    return await this.makeRequest(
      'GET',
      `${this.config.apiGatewayUrl}/api/v1/workflows/${operationId}/status`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  public async cancelWorkflow(
    operationId: string,
    authToken: string
  ): Promise<void> {
    await this.makeRequest(
      'POST',
      `${this.config.apiGatewayUrl}/api/v1/workflows/${operationId}/cancel`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  // Tenant switching workflow
  public async switchTenant(
    request: TenantSwitchRequest,
    userId: string,
    authToken: string
  ): Promise<WorkflowResponse<TenantSwitchResult>> {
    return await this.initiateWorkflow(
      {
        workflowType: 'switch-tenant',
        tenantId: request.currentTenantId || request.targetTenantId,
        userId,
        data: request,
        options: {
          synchronous: true, // Tenant switching should be fast
          timeout: 30000, // 30 seconds
        },
      },
      authToken
    );
  }

  // Tenant provisioning workflow
  public async provisionTenant(
    tenantData: any,
    userId: string,
    authToken: string
  ): Promise<WorkflowResponse<Tenant>> {
    return await this.initiateWorkflow(
      {
        workflowType: 'provision-tenant',
        tenantId: 'system', // System-level operation
        userId,
        data: tenantData,
        options: {
          synchronous: false, // Provisioning can take time
          timeout: 300000, // 5 minutes
        },
      },
      authToken
    );
  }

  // Tenant migration workflow
  public async migrateTenant(
    tenantId: string,
    migrationData: any,
    userId: string,
    authToken: string
  ): Promise<WorkflowResponse<any>> {
    return await this.initiateWorkflow(
      {
        workflowType: 'migrate-tenant',
        tenantId,
        userId,
        data: migrationData,
        options: {
          synchronous: false, // Migration takes time
          timeout: 1800000, // 30 minutes
        },
      },
      authToken
    );
  }

  // Bulk operations
  public async bulkInviteUsers(
    tenantId: string,
    invitations: any[],
    userId: string,
    authToken: string
  ): Promise<WorkflowResponse<any>> {
    return await this.initiateWorkflow(
      {
        workflowType: 'bulk-invite-users',
        tenantId,
        userId,
        data: { invitations },
        options: {
          synchronous: false,
          timeout: 600000, // 10 minutes
        },
      },
      authToken
    );
  }

  public async bulkUpdateMemberships(
    tenantId: string,
    updates: any[],
    userId: string,
    authToken: string
  ): Promise<WorkflowResponse<any>> {
    return await this.initiateWorkflow(
      {
        workflowType: 'bulk-update-memberships',
        tenantId,
        userId,
        data: { updates },
        options: {
          synchronous: false,
          timeout: 300000, // 5 minutes
        },
      },
      authToken
    );
  }

  // Aggregated data operations
  public async getTenantDashboardData(
    tenantId: string,
    userId: string,
    authToken: string
  ): Promise<any> {
    // This could be a workflow or direct aggregation
    return await this.makeRequest(
      'GET',
      `${this.config.apiGatewayUrl}/api/v1/tenants/${tenantId}/dashboard?userId=${userId}`,
      undefined,
      { 
        Authorization: `Bearer ${authToken}`,
        'X-Tenant-ID': tenantId,
      }
    );
  }

  public async getTenantOverview(
    tenantId: string,
    authToken: string
  ): Promise<any> {
    return await this.makeRequest(
      'GET',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/overview`,
      undefined,
      { 
        Authorization: `Bearer ${authToken}`,
        'X-Tenant-ID': tenantId,
      }
    );
  }

  // User context operations
  public async getUserTenantContext(
    tenantId: string,
    userId: string,
    authToken: string
  ): Promise<any> {
    return await this.makeRequest(
      'GET',
      `${this.config.userServiceUrl}/api/v1/users/${userId}/tenants/${tenantId}/context`,
      undefined,
      { 
        Authorization: `Bearer ${authToken}`,
        'X-Tenant-ID': tenantId,
      }
    );
  }

  // Validation operations
  public async validateTenantAccess(
    tenantId: string,
    userId: string,
    authToken: string
  ): Promise<{ hasAccess: boolean; permissions: string[]; roles: string[] }> {
    return await this.makeRequest(
      'GET',
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/access/${userId}`,
      undefined,
      { Authorization: `Bearer ${authToken}` }
    );
  }

  // Generic HTTP request method
  private async makeRequest(
    method: string,
    url: string,
    data?: any,
    headers: Record<string, string> = {}
  ): Promise<any> {
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

    try {
      const requestHeaders: Record<string, string> = {
        'Content-Type': 'application/json',
        'User-Agent': 'ADX-Core-Tenant-BFF/1.0.0',
        ...headers,
      };

      const requestOptions: RequestInit = {
        method,
        headers: requestHeaders,
        signal: controller.signal,
      };

      if (data && (method === 'POST' || method === 'PUT' || method === 'PATCH')) {
        requestOptions.body = JSON.stringify(data);
      }

      const response = await fetch(url, requestOptions);

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorText = await response.text();
        let errorData;
        
        try {
          errorData = JSON.parse(errorText);
        } catch {
          errorData = { message: errorText };
        }

        throw new ApiError(
          errorData.message || `HTTP ${response.status}: ${response.statusText}`,
          response.status,
          errorData
        );
      }

      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json();
      } else {
        return await response.text();
      }
    } catch (error: any) {
      clearTimeout(timeoutId);
      
      if (error.name === 'AbortError') {
        throw new ApiError('Request timeout', 408);
      }
      
      if (error instanceof ApiError) {
        throw error;
      }
      
      throw new ApiError(
        error.message || 'Network request failed',
        0,
        { originalError: error }
      );
    }
  }
}

export class ApiError extends Error {
  constructor(
    message: string,
    public statusCode: number,
    public data?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}