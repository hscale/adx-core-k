import {
  User,
  UserProfile,
  UserSettings,
  UpdateUserRequest,
  UpdateUserProfileRequest,
  UpdateUserPreferencesRequest,
  UserSearchFilters,
  UserSearchResult,
  UserActivity,
  CreateUserRequest,
  UserInvitation,
  WorkflowResponse,
  UserBFFClient as IUserBFFClient,
} from '../types';

class UserBFFClient implements IUserBFFClient {
  private baseUrl: string;

  constructor(baseUrl: string = 'http://localhost:4004') {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    endpoint: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${endpoint}`;
    
    const defaultHeaders = {
      'Content-Type': 'application/json',
      'Authorization': `Bearer ${this.getAuthToken()}`,
      'X-Tenant-ID': this.getTenantId(),
    };

    const response = await fetch(url, {
      ...options,
      headers: {
        ...defaultHeaders,
        ...options.headers,
      },
    });

    if (!response.ok) {
      const errorData = await response.json().catch(() => ({}));
      throw new Error(errorData.message || `HTTP ${response.status}: ${response.statusText}`);
    }

    return response.json();
  }

  private getAuthToken(): string {
    return localStorage.getItem('auth_token') || '';
  }

  private getTenantId(): string {
    return localStorage.getItem('current_tenant_id') || '';
  }

  async getUser(userId: string): Promise<User> {
    return this.request<User>(`/api/users/${userId}`);
  }

  async getUserProfile(userId: string): Promise<UserProfile> {
    return this.request<UserProfile>(`/api/users/${userId}/profile`);
  }

  async getUserSettings(userId: string): Promise<UserSettings> {
    return this.request<UserSettings>(`/api/users/${userId}/settings`);
  }

  async updateUser(userId: string, updates: UpdateUserRequest): Promise<WorkflowResponse<User>> {
    return this.request<WorkflowResponse<User>>(`/api/workflows/update-user`, {
      method: 'POST',
      body: JSON.stringify({ userId, updates }),
    });
  }

  async updateUserProfile(userId: string, updates: UpdateUserProfileRequest): Promise<WorkflowResponse<UserProfile>> {
    return this.request<WorkflowResponse<UserProfile>>(`/api/workflows/update-user-profile`, {
      method: 'POST',
      body: JSON.stringify({ userId, updates }),
    });
  }

  async updateUserPreferences(userId: string, updates: UpdateUserPreferencesRequest): Promise<WorkflowResponse<any>> {
    return this.request<WorkflowResponse<any>>(`/api/workflows/update-user-preferences`, {
      method: 'POST',
      body: JSON.stringify({ userId, updates }),
    });
  }

  async searchUsers(filters: UserSearchFilters, page = 1, pageSize = 20): Promise<UserSearchResult> {
    const params = new URLSearchParams({
      page: page.toString(),
      pageSize: pageSize.toString(),
      ...Object.entries(filters).reduce((acc, [key, value]) => {
        if (value !== undefined && value !== null) {
          if (Array.isArray(value)) {
            acc[key] = value.join(',');
          } else {
            acc[key] = value.toString();
          }
        }
        return acc;
      }, {} as Record<string, string>),
    });

    return this.request<UserSearchResult>(`/api/users/search?${params}`);
  }

  async getUserActivity(userId: string, limit = 50): Promise<UserActivity[]> {
    return this.request<UserActivity[]>(`/api/users/${userId}/activity?limit=${limit}`);
  }

  async inviteUser(invitation: CreateUserRequest): Promise<WorkflowResponse<UserInvitation>> {
    return this.request<WorkflowResponse<UserInvitation>>(`/api/workflows/invite-user`, {
      method: 'POST',
      body: JSON.stringify(invitation),
    });
  }

  async deactivateUser(userId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/deactivate-user`, {
      method: 'POST',
      body: JSON.stringify({ userId }),
    });
  }

  async reactivateUser(userId: string): Promise<WorkflowResponse<void>> {
    return this.request<WorkflowResponse<void>>(`/api/workflows/reactivate-user`, {
      method: 'POST',
      body: JSON.stringify({ userId }),
    });
  }

  // Workflow status polling
  async pollWorkflowStatus(operationId: string): Promise<any> {
    return new Promise((resolve, reject) => {
      const poll = async () => {
        try {
          const response = await this.request<any>(`/api/workflows/${operationId}/status`);
          
          switch (response.status) {
            case 'completed':
              resolve(response.result);
              break;
            case 'failed':
              reject(new Error(response.error || 'Workflow failed'));
              break;
            case 'running':
            case 'pending':
              setTimeout(poll, 1000); // Poll every second
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
}

export const userBFFClient = new UserBFFClient();
export default userBFFClient;