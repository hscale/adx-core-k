import { apiService } from './api'
import { LoginCredentials, RegisterData, AuthResponse, User, Tenant } from '@/types'

export class AuthService {
  async login(credentials: LoginCredentials): Promise<AuthResponse> {
    // Add demo tenant ID if not provided
    const loginData = {
      ...credentials,
      tenant_id: credentials.tenantId || '550e8400-e29b-41d4-a716-446655440000'
    }

    // The backend returns a different structure, so we need to handle it
    const backendResponse = await apiService.post<{
      access_token: string
      refresh_token: string
      expires_in: number
      user_id: string
    }>('/auth/login', loginData)

    // Create a mock user and tenant for now - in a real app, you'd fetch these
    const user: User = {
      id: backendResponse.user_id,
      email: credentials.email,
      firstName: 'Demo',
      lastName: 'User',
      role: 'admin',
      tenantId: loginData.tenant_id,
      isActive: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    }

    const tenant: Tenant = {
      id: loginData.tenant_id,
      name: 'Demo Tenant',
      domain: 'demo.adx-core.com',
      settings: {
        theme: 'light',
        language: 'en',
        timezone: 'UTC',
        features: []
      },
      isActive: true,
      createdAt: new Date().toISOString(),
      updatedAt: new Date().toISOString()
    }

    const response: AuthResponse = {
      user,
      token: backendResponse.access_token,
      refreshToken: backendResponse.refresh_token,
      tenant
    }

    // Store tokens and user data
    localStorage.setItem('auth_token', response.token)
    localStorage.setItem('refresh_token', response.refreshToken)
    localStorage.setItem('user', JSON.stringify(response.user))
    localStorage.setItem('tenant_id', response.tenant.id)

    return response
  }

  async register(data: RegisterData): Promise<AuthResponse> {
    const response = await apiService.post<AuthResponse>('/auth/register', data)

    // Store tokens and user data
    localStorage.setItem('auth_token', response.token)
    localStorage.setItem('refresh_token', response.refreshToken)
    localStorage.setItem('user', JSON.stringify(response.user))
    localStorage.setItem('tenant_id', response.tenant.id)

    return response
  }

  async logout(): Promise<void> {
    try {
      await apiService.post('/auth/logout')
    } catch (error) {
      // Continue with logout even if API call fails
      console.warn('Logout API call failed:', error)
    } finally {
      // Clear local storage
      localStorage.removeItem('auth_token')
      localStorage.removeItem('refresh_token')
      localStorage.removeItem('user')
      localStorage.removeItem('tenant_id')
    }
  }

  async refreshToken(): Promise<{ token: string }> {
    const refreshToken = localStorage.getItem('refresh_token')
    if (!refreshToken) {
      throw new Error('No refresh token available')
    }

    const response = await apiService.post<{ token: string }>('/auth/refresh', {
      refreshToken,
    })

    localStorage.setItem('auth_token', response.token)
    return response
  }

  async forgotPassword(email: string): Promise<{ message: string }> {
    return apiService.post('/auth/forgot-password', { email })
  }

  async resetPassword(token: string, password: string): Promise<{ message: string }> {
    return apiService.post('/auth/reset-password', { token, password })
  }

  async getCurrentUser(): Promise<User> {
    return apiService.get<User>('/auth/me')
  }

  async updateProfile(data: Partial<User>): Promise<User> {
    return apiService.put<User>('/auth/profile', data)
  }

  async changePassword(currentPassword: string, newPassword: string): Promise<{ message: string }> {
    return apiService.post('/auth/change-password', {
      currentPassword,
      newPassword,
    })
  }

  // Check if user is authenticated
  isAuthenticated(): boolean {
    const token = localStorage.getItem('auth_token')
    const user = localStorage.getItem('user')
    return !!(token && user)
  }

  // Get stored user data
  getStoredUser(): User | null {
    const userStr = localStorage.getItem('user')
    if (!userStr) return null

    try {
      return JSON.parse(userStr)
    } catch {
      return null
    }
  }

  // Get stored token
  getStoredToken(): string | null {
    return localStorage.getItem('auth_token')
  }

  // Get stored tenant ID
  getStoredTenantId(): string | null {
    return localStorage.getItem('tenant_id')
  }
}

export const authService = new AuthService()