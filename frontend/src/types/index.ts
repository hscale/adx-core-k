// User types
export interface User {
  id: string
  email: string
  firstName?: string
  lastName?: string
  avatar?: string
  role: UserRole
  tenantId: string
  isActive: boolean
  lastLoginAt?: string
  createdAt: string
  updatedAt: string
}

export type UserRole = 'admin' | 'user' | 'viewer'

// Tenant types
export interface Tenant {
  id: string
  name: string
  domain?: string
  logo?: string
  settings: TenantSettings
  isActive: boolean
  createdAt: string
  updatedAt: string
}

export interface TenantSettings {
  theme: 'light' | 'dark' | 'auto'
  language: string
  timezone: string
  features: string[]
  branding?: {
    primaryColor?: string
    logo?: string
    favicon?: string
  }
}

// Auth types
export interface LoginCredentials {
  email: string
  password: string
  tenantId?: string
}

export interface RegisterData {
  email: string
  password: string
  firstName?: string
  lastName?: string
  tenantId?: string
}

export interface AuthResponse {
  user: User
  token: string
  refreshToken: string
  tenant: Tenant
}

// API types
export interface ApiResponse<T = any> {
  data: T
  message?: string
  success: boolean
}

export interface ApiError {
  message: string
  code?: string
  status?: number
  details?: any
}

export interface PaginatedResponse<T> {
  data: T[]
  pagination: {
    page: number
    limit: number
    total: number
    totalPages: number
  }
}

// File types
export interface FileItem {
  id: string
  name: string
  type: 'file' | 'folder'
  size?: number
  mimeType?: string
  path: string
  parentId?: string
  ownerId: string
  tenantId: string
  permissions: FilePermissions
  createdAt: string
  updatedAt: string
}

export interface FilePermissions {
  read: boolean
  write: boolean
  delete: boolean
  share: boolean
}

// Workflow types
export interface Workflow {
  id: string
  name: string
  description?: string
  status: WorkflowStatus
  definition: WorkflowDefinition
  tenantId: string
  createdBy: string
  createdAt: string
  updatedAt: string
}

export type WorkflowStatus = 'draft' | 'active' | 'paused' | 'archived'

export interface WorkflowDefinition {
  steps: WorkflowStep[]
  triggers: WorkflowTrigger[]
  variables?: Record<string, any>
}

export interface WorkflowStep {
  id: string
  name: string
  type: string
  config: Record<string, any>
  nextSteps?: string[]
}

export interface WorkflowTrigger {
  id: string
  type: 'manual' | 'schedule' | 'event'
  config: Record<string, any>
}

// Platform types
export interface PlatformInfo {
  type: 'web' | 'desktop' | 'mobile'
  os?: string
  version?: string
  isMobile: boolean
  isDesktop: boolean
  isWeb: boolean
  capabilities: PlatformCapabilities
}

export interface PlatformCapabilities {
  notifications: boolean
  fileSystem: boolean
  camera: boolean
  geolocation: boolean
  clipboard: boolean
}

// Theme types
export type Theme = 'light' | 'dark' | 'auto'

// Language types
export type Language = 'en' | 'es' | 'fr' | 'de' | 'ja' | 'zh'

// Component types
export interface SelectOption {
  value: string
  label: string
  disabled?: boolean
}

export interface TableColumn<T = any> {
  key: keyof T | string
  label: string
  sortable?: boolean
  render?: (value: any, item: T) => React.ReactNode
}

export interface FilterOption {
  key: string
  label: string
  type: 'text' | 'select' | 'date' | 'boolean'
  options?: SelectOption[]
}

// Form types
export interface FormField {
  name: string
  label: string
  type: 'text' | 'email' | 'password' | 'select' | 'textarea' | 'checkbox' | 'file'
  required?: boolean
  placeholder?: string
  options?: SelectOption[]
  validation?: any
}