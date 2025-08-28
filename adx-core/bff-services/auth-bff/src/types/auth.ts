export interface User {
  id: string;
  email: string;
  firstName?: string;
  lastName?: string;
  avatar?: string;
  roles: string[];
  permissions: string[];
  createdAt: string;
  updatedAt: string;
  lastLoginAt?: string;
  isActive: boolean;
  emailVerified: boolean;
  mfaEnabled: boolean;
}

export interface Tenant {
  id: string;
  name: string;
  slug: string;
  features: string[];
  quotas: Record<string, QuotaInfo>;
  settings: TenantSettings;
  subscriptionTier: string;
  isActive: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface QuotaInfo {
  used: number;
  limit: number;
  unit: string;
}

export interface TenantSettings {
  theme: string;
  locale: string;
  timezone: string;
  [key: string]: any;
}

export interface AuthSession {
  id: string;
  userId: string;
  tenantId: string;
  token: string;
  refreshToken: string;
  expiresAt: string;
  deviceInfo?: DeviceInfo;
  ipAddress: string;
  userAgent: string;
  isActive: boolean;
  createdAt: string;
  lastActivityAt: string;
}

export interface DeviceInfo {
  type: 'web' | 'mobile' | 'desktop';
  os?: string;
  browser?: string;
  version?: string;
}

export interface LoginRequest {
  email: string;
  password: string;
  tenantId?: string;
  deviceInfo?: DeviceInfo;
  rememberMe?: boolean;
}

export interface LoginResponse {
  user: User;
  tenant: Tenant;
  session: AuthSession;
  availableTenants: Tenant[];
  permissions: string[];
  features: string[];
}

export interface RegisterRequest {
  email: string;
  password: string;
  firstName?: string;
  lastName?: string;
  tenantName?: string;
  inviteToken?: string;
}

export interface RegisterResponse {
  user: User;
  tenant?: Tenant;
  requiresEmailVerification: boolean;
  message: string;
}

export interface RefreshTokenRequest {
  refreshToken: string;
}

export interface RefreshTokenResponse {
  token: string;
  refreshToken: string;
  expiresAt: string;
}

export interface PasswordResetRequest {
  email: string;
  tenantId?: string;
}

export interface PasswordResetResponse {
  message: string;
  resetToken?: string; // Only in development
}

export interface PasswordUpdateRequest {
  resetToken: string;
  newPassword: string;
}

export interface TenantSwitchRequest {
  targetTenantId: string;
}

export interface TenantSwitchResponse {
  tenant: Tenant;
  session: AuthSession;
  permissions: string[];
  features: string[];
}

export interface UserProfileUpdateRequest {
  firstName?: string;
  lastName?: string;
  avatar?: string;
  preferences?: Record<string, any>;
}

export interface MfaSetupRequest {
  type: 'totp' | 'sms';
  phoneNumber?: string;
}

export interface MfaSetupResponse {
  secret?: string;
  qrCode?: string;
  backupCodes: string[];
}

export interface MfaVerifyRequest {
  code: string;
  type: 'totp' | 'sms';
}

export interface WorkflowResponse<T = any> {
  type: 'sync' | 'async';
  data?: T;
  operationId?: string;
  statusUrl?: string;
  streamUrl?: string;
  estimatedDuration?: number;
}

export interface WorkflowStatus {
  operationId: string;
  status: 'pending' | 'running' | 'completed' | 'failed' | 'cancelled';
  progress?: {
    current: number;
    total: number;
    message?: string;
  };
  result?: any;
  error?: string;
  startedAt: string;
  updatedAt: string;
  estimatedCompletion?: string;
}

export interface AggregatedDashboardData {
  user: User;
  tenant: Tenant;
  recentActivity: ActivityItem[];
  notifications: NotificationItem[];
  quickStats: QuickStats;
  availableTenants: Tenant[];
}

export interface ActivityItem {
  id: string;
  type: string;
  title: string;
  description: string;
  timestamp: string;
  metadata?: Record<string, any>;
}

export interface NotificationItem {
  id: string;
  type: 'info' | 'warning' | 'error' | 'success';
  title: string;
  message: string;
  isRead: boolean;
  createdAt: string;
  actionUrl?: string;
}

export interface QuickStats {
  activeUsers: number;
  totalFiles: number;
  storageUsed: number;
  workflowsRunning: number;
}

export interface WebSocketMessage {
  type: string;
  data: any;
  timestamp: string;
  userId?: string;
  tenantId?: string;
}

export interface AuthStatusUpdate {
  type: 'login' | 'logout' | 'session_expired' | 'tenant_switched' | 'profile_updated';
  userId: string;
  tenantId?: string;
  sessionId?: string;
  data?: any;
}