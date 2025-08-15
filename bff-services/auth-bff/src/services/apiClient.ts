import { WorkflowResponse, WorkflowStatus } from '../types/auth.js';

export interface ApiClientConfig {
  apiGatewayUrl: string;
  authServiceUrl: string;
  userServiceUrl: string;
  tenantServiceUrl: string;
  timeout?: number;
}

export class ApiClient {
  private config: ApiClientConfig;

  constructor(config: ApiClientConfig) {
    this.config = {
      timeout: 10000,
      ...config,
    };
  }

  private async makeRequest<T = any>(
    url: string,
    options: RequestInit = {},
    authToken?: string,
    tenantId?: string
  ): Promise<T> {
    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...((options.headers as Record<string, string>) || {}),
    };

    if (authToken) {
      headers['Authorization'] = `Bearer ${authToken}`;
    }

    if (tenantId) {
      headers['X-Tenant-ID'] = tenantId;
    }

    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.config.timeout);

    try {
      const response = await fetch(url, {
        ...options,
        headers,
        signal: controller.signal,
      });

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
          response.status,
          errorData.message || `HTTP ${response.status}`,
          errorData
        );
      }

      const contentType = response.headers.get('content-type');
      if (contentType && contentType.includes('application/json')) {
        return await response.json();
      }

      return await response.text() as T;
    } catch (error) {
      clearTimeout(timeoutId);
      
      if (error instanceof ApiError) {
        throw error;
      }
      
      if (error.name === 'AbortError') {
        throw new ApiError(408, 'Request timeout');
      }
      
      throw new ApiError(500, 'Network error', { originalError: error.message });
    }
  }

  // Auth Service Methods
  async login(email: string, password: string, tenantId?: string): Promise<any> {
    return this.makeRequest(`${this.config.authServiceUrl}/api/v1/auth/login`, {
      method: 'POST',
      body: JSON.stringify({ email, password, tenantId }),
    });
  }

  async register(userData: any): Promise<any> {
    return this.makeRequest(`${this.config.authServiceUrl}/api/v1/auth/register`, {
      method: 'POST',
      body: JSON.stringify(userData),
    });
  }

  async refreshToken(refreshToken: string): Promise<any> {
    return this.makeRequest(`${this.config.authServiceUrl}/api/v1/auth/refresh`, {
      method: 'POST',
      body: JSON.stringify({ refreshToken }),
    });
  }

  async logout(sessionId: string, authToken: string): Promise<void> {
    return this.makeRequest(`${this.config.authServiceUrl}/api/v1/auth/logout`, {
      method: 'POST',
      body: JSON.stringify({ sessionId }),
    }, authToken);
  }

  async requestPasswordReset(email: string, tenantId?: string): Promise<any> {
    return this.makeRequest(`${this.config.authServiceUrl}/api/v1/auth/password-reset`, {
      method: 'POST',
      body: JSON.stringify({ email, tenantId }),
    });
  }

  async resetPassword(resetToken: string, newPassword: string): Promise<any> {
    return this.makeRequest(`${this.config.authServiceUrl}/api/v1/auth/password-reset/confirm`, {
      method: 'POST',
      body: JSON.stringify({ resetToken, newPassword }),
    });
  }

  async verifyEmail(verificationToken: string): Promise<any> {
    return this.makeRequest(`${this.config.authServiceUrl}/api/v1/auth/verify-email`, {
      method: 'POST',
      body: JSON.stringify({ verificationToken }),
    });
  }

  // User Service Methods
  async getUserProfile(userId: string, authToken: string, tenantId: string): Promise<any> {
    return this.makeRequest(
      `${this.config.userServiceUrl}/api/v1/users/${userId}`,
      { method: 'GET' },
      authToken,
      tenantId
    );
  }

  async updateUserProfile(userId: string, updates: any, authToken: string, tenantId: string): Promise<any> {
    return this.makeRequest(
      `${this.config.userServiceUrl}/api/v1/users/${userId}`,
      {
        method: 'PUT',
        body: JSON.stringify(updates),
      },
      authToken,
      tenantId
    );
  }

  async getUserActivity(userId: string, authToken: string, tenantId: string, limit = 10): Promise<any> {
    return this.makeRequest(
      `${this.config.userServiceUrl}/api/v1/users/${userId}/activity?limit=${limit}`,
      { method: 'GET' },
      authToken,
      tenantId
    );
  }

  async getUserNotifications(userId: string, authToken: string, tenantId: string, limit = 10): Promise<any> {
    return this.makeRequest(
      `${this.config.userServiceUrl}/api/v1/users/${userId}/notifications?limit=${limit}`,
      { method: 'GET' },
      authToken,
      tenantId
    );
  }

  // Tenant Service Methods
  async getTenant(tenantId: string, authToken: string): Promise<any> {
    return this.makeRequest(
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}`,
      { method: 'GET' },
      authToken,
      tenantId
    );
  }

  async getUserTenants(userId: string, authToken: string): Promise<any> {
    return this.makeRequest(
      `${this.config.tenantServiceUrl}/api/v1/users/${userId}/tenants`,
      { method: 'GET' },
      authToken
    );
  }

  async getTenantStats(tenantId: string, authToken: string): Promise<any> {
    return this.makeRequest(
      `${this.config.tenantServiceUrl}/api/v1/tenants/${tenantId}/stats`,
      { method: 'GET' },
      authToken,
      tenantId
    );
  }

  // Workflow Methods (via API Gateway)
  async initiateWorkflow<T = any>(
    workflowType: string,
    request: any,
    authToken: string,
    tenantId?: string,
    synchronous = false
  ): Promise<WorkflowResponse<T>> {
    const url = `${this.config.apiGatewayUrl}/api/v1/workflows/${workflowType}`;
    const body = { ...request, synchronous };

    return this.makeRequest(url, {
      method: 'POST',
      body: JSON.stringify(body),
    }, authToken, tenantId);
  }

  async getWorkflowStatus(operationId: string, authToken: string, tenantId?: string): Promise<WorkflowStatus> {
    return this.makeRequest(
      `${this.config.apiGatewayUrl}/api/v1/workflows/${operationId}/status`,
      { method: 'GET' },
      authToken,
      tenantId
    );
  }

  async cancelWorkflow(operationId: string, authToken: string, tenantId?: string): Promise<void> {
    return this.makeRequest(
      `${this.config.apiGatewayUrl}/api/v1/workflows/${operationId}/cancel`,
      { method: 'POST' },
      authToken,
      tenantId
    );
  }

  // Batch request method for aggregated data
  async batchRequest(requests: BatchRequest[], authToken: string, tenantId?: string): Promise<BatchResponse[]> {
    return this.makeRequest(
      `${this.config.apiGatewayUrl}/api/v1/batch`,
      {
        method: 'POST',
        body: JSON.stringify({ requests }),
      },
      authToken,
      tenantId
    );
  }

  // Health check methods
  async healthCheck(service?: 'auth' | 'user' | 'tenant' | 'gateway'): Promise<any> {
    const urls = {
      auth: `${this.config.authServiceUrl}/health`,
      user: `${this.config.userServiceUrl}/health`,
      tenant: `${this.config.tenantServiceUrl}/health`,
      gateway: `${this.config.apiGatewayUrl}/health`,
    };

    if (service) {
      return this.makeRequest(urls[service], { method: 'GET' });
    }

    // Check all services
    const results = await Promise.allSettled(
      Object.entries(urls).map(async ([name, url]) => {
        try {
          const result = await this.makeRequest(url, { method: 'GET' });
          return { service: name, status: 'healthy', ...result };
        } catch (error) {
          return { service: name, status: 'unhealthy', error: error.message };
        }
      })
    );

    return results.map((result, index) => {
      const serviceName = Object.keys(urls)[index];
      return result.status === 'fulfilled' 
        ? result.value 
        : { service: serviceName, status: 'unhealthy', error: result.reason };
    });
  }
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    public data?: any
  ) {
    super(message);
    this.name = 'ApiError';
  }
}

export interface BatchRequest {
  id: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  url: string;
  body?: any;
  headers?: Record<string, string>;
}

export interface BatchResponse {
  id: string;
  status: number;
  data?: any;
  error?: string;
}