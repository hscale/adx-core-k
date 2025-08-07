import axios, { AxiosInstance, AxiosRequestConfig, AxiosResponse } from 'axios'
import { ApiResponse, ApiError } from '@/types'

class ApiService {
  private client: AxiosInstance
  private baseURL: string

  constructor() {
    this.baseURL = import.meta.env.VITE_API_URL || 'http://localhost:8080'
    
    this.client = axios.create({
      baseURL: `${this.baseURL}/api/v1`,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
      },
    })

    this.setupInterceptors()
  }

  private setupInterceptors() {
    // Request interceptor
    this.client.interceptors.request.use(
      (config) => {
        const token = localStorage.getItem('auth_token')
        if (token) {
          config.headers.Authorization = `Bearer ${token}`
        }

        const tenantId = localStorage.getItem('tenant_id')
        if (tenantId) {
          config.headers['X-Tenant-ID'] = tenantId
        }

        return config
      },
      (error) => {
        return Promise.reject(error)
      }
    )

    // Response interceptor
    this.client.interceptors.response.use(
      (response: AxiosResponse<ApiResponse>) => {
        return response
      },
      async (error) => {
        const originalRequest = error.config

        if (error.response?.status === 401 && !originalRequest._retry) {
          originalRequest._retry = true

          try {
            const refreshToken = localStorage.getItem('refresh_token')
            if (refreshToken) {
              const response = await this.client.post('/auth/refresh', {
                refreshToken,
              })

              const { token } = response.data.data
              localStorage.setItem('auth_token', token)

              return this.client(originalRequest)
            }
          } catch (refreshError) {
            // Refresh failed, redirect to login
            localStorage.removeItem('auth_token')
            localStorage.removeItem('refresh_token')
            localStorage.removeItem('user')
            window.location.href = '/auth/login'
          }
        }

        const apiError: ApiError = {
          message: error.response?.data?.message || error.message || 'An error occurred',
          code: error.response?.data?.code,
          status: error.response?.status,
          details: error.response?.data?.details,
        }

        return Promise.reject(apiError)
      }
    )
  }

  // Generic request methods
  async get<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.get<ApiResponse<T>>(url, config)
    return response.data.data
  }

  async post<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.post<ApiResponse<T>>(url, data, config)
    return response.data.data
  }

  async put<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.put<ApiResponse<T>>(url, data, config)
    return response.data.data
  }

  async patch<T = any>(url: string, data?: any, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.patch<ApiResponse<T>>(url, data, config)
    return response.data.data
  }

  async delete<T = any>(url: string, config?: AxiosRequestConfig): Promise<T> {
    const response = await this.client.delete<ApiResponse<T>>(url, config)
    return response.data.data
  }

  // File upload
  async uploadFile(file: File, onProgress?: (progress: number) => void): Promise<any> {
    const formData = new FormData()
    formData.append('file', file)

    return this.post('/files/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
      onUploadProgress: (progressEvent) => {
        if (onProgress && progressEvent.total) {
          const progress = Math.round((progressEvent.loaded * 100) / progressEvent.total)
          onProgress(progress)
        }
      },
    })
  }

  // Health check
  async healthCheck(): Promise<{ status: string }> {
    const response = await this.client.get('/health')
    return response.data
  }

  // Get base URL for direct access
  getBaseURL(): string {
    return this.baseURL
  }
}

export const apiService = new ApiService()
export default apiService