import type {
  LoginRequest,
  RegisterRequest,
  ForgotPasswordRequest,
  ResetPasswordRequest,
  MFASetupRequest,
  MFAVerifyRequest,
  SSOLoginRequest,
  AuthResponse,
  WorkflowResponse,
  WorkflowStatus,
  MFASecret,
  BFFClientConfig
} from '../types';

export class AuthBFFClient {
  private baseUrl: string;
  private timeout: number;

  constructor(config: BFFClientConfig) {
    this.baseUrl = config.baseUrl || 'http://localhost:4001';
    this.timeout = config.timeout || 30000;
  }

  private async makeRequest<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    const controller = new AbortController();
    const timeoutId = setTimeout(() => controller.abort(), this.timeout);

    try {
      const response = await fetch(url, {
        ...options,
        signal: controller.signal,
        headers: {
          'Content-Type': 'application/json',
          ...options.headers,
        },
      });

      clearTimeout(timeoutId);

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.error?.message || `HTTP ${response.status}: ${response.statusText}`);
      }

      return await response.json();
    } catch (error) {
      clearTimeout(timeoutId);
      throw error;
    }
  }

  private async makeWorkflowRequest<T>(
    workflowType: string,
    request: any,
    options: { synchronous?: boolean } = {}
  ): Promise<WorkflowResponse<T>> {
    return this.makeRequest<WorkflowResponse<T>>(`/workflows/${workflowType}`, {
      method: 'POST',
      body: JSON.stringify({
        ...request,
        synchronous: options.synchronous,
      }),
    });
  }

  // Authentication workflows
  async login(request: LoginRequest): Promise<WorkflowResponse<AuthResponse>> {
    return this.makeWorkflowRequest<AuthResponse>('user-login', request, {
      synchronous: true,
    });
  }

  async register(request: RegisterRequest): Promise<WorkflowResponse<AuthResponse>> {
    return this.makeWorkflowRequest<AuthResponse>('user-registration', request);
  }

  async forgotPassword(request: ForgotPasswordRequest): Promise<WorkflowResponse<{ message: string }>> {
    return this.makeWorkflowRequest<{ message: string }>('password-reset-request', request, {
      synchronous: true,
    });
  }

  async resetPassword(request: ResetPasswordRequest): Promise<WorkflowResponse<{ message: string }>> {
    return this.makeWorkflowRequest<{ message: string }>('password-reset-confirm', request, {
      synchronous: true,
    });
  }

  async setupMFA(): Promise<WorkflowResponse<MFASecret>> {
    return this.makeWorkflowRequest<MFASecret>('mfa-setup-initiate', {}, {
      synchronous: true,
    });
  }

  async confirmMFASetup(request: MFASetupRequest): Promise<WorkflowResponse<{ backupCodes: string[] }>> {
    return this.makeWorkflowRequest<{ backupCodes: string[] }>('mfa-setup-confirm', request, {
      synchronous: true,
    });
  }

  async verifyMFA(request: MFAVerifyRequest): Promise<WorkflowResponse<AuthResponse>> {
    return this.makeWorkflowRequest<AuthResponse>('mfa-verify', request, {
      synchronous: true,
    });
  }

  async initiateSSO(request: SSOLoginRequest): Promise<WorkflowResponse<{ redirectUrl: string }>> {
    return this.makeWorkflowRequest<{ redirectUrl: string }>('sso-initiate', request, {
      synchronous: true,
    });
  }

  async completeSSOLogin(code: string, state: string): Promise<WorkflowResponse<AuthResponse>> {
    return this.makeWorkflowRequest<AuthResponse>('sso-complete', {
      code,
      state,
    });
  }

  // Direct endpoints for simple operations
  async refreshToken(refreshToken: string): Promise<AuthResponse> {
    return this.makeRequest<AuthResponse>('/auth/refresh', {
      method: 'POST',
      body: JSON.stringify({ refreshToken }),
    });
  }

  async logout(token: string): Promise<{ message: string }> {
    return this.makeRequest<{ message: string }>('/auth/logout', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
  }

  async validateToken(token: string): Promise<{ valid: boolean; user?: any }> {
    return this.makeRequest<{ valid: boolean; user?: any }>('/auth/validate', {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${token}`,
      },
    });
  }

  // Workflow status polling
  async getWorkflowStatus(operationId: string): Promise<WorkflowStatus> {
    return this.makeRequest<WorkflowStatus>(`/workflows/${operationId}/status`);
  }

  async pollWorkflowStatus(
    operationId: string,
    onProgress?: (progress: WorkflowStatus['progress']) => void
  ): Promise<any> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const status = await this.getWorkflowStatus(operationId);

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
            case 'cancelled':
              reject(new Error('Workflow was cancelled'));
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

  // Utility methods
  async healthCheck(): Promise<{ status: string; timestamp: string }> {
    return this.makeRequest<{ status: string; timestamp: string }>('/health');
  }
}

// Singleton instance
export const authBFFClient = new AuthBFFClient({
  baseUrl: import.meta.env.VITE_AUTH_BFF_URL || 'http://localhost:4001',
});