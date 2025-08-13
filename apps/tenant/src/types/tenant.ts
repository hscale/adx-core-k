export interface Tenant {
  id: string;
  name: string;
  slug: string;
  description?: string;
  adminEmail: string;
  subscriptionTier: SubscriptionTier;
  status: TenantStatus;
  features: string[];
  quotas: TenantQuotas;
  settings: TenantSettings;
  createdAt: string;
  updatedAt: string;
}

export enum SubscriptionTier {
  FREE = 'free',
  PROFESSIONAL = 'professional',
  ENTERPRISE = 'enterprise',
}

export enum TenantStatus {
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
  PENDING = 'pending',
  CANCELLED = 'cancelled',
}

export interface TenantQuotas {
  maxUsers: number;
  maxStorageGB: number;
  maxApiCallsPerHour: number;
  maxWorkflowsPerHour: number;
  currentUsers: number;
  currentStorageGB: number;
  currentApiCallsThisHour: number;
  currentWorkflowsThisHour: number;
}

export interface TenantSettings {
  timezone: string;
  dateFormat: string;
  language: string;
  theme: 'light' | 'dark' | 'system';
  notifications: {
    email: boolean;
    push: boolean;
    sms: boolean;
  };
  security: {
    mfaRequired: boolean;
    sessionTimeout: number;
    passwordPolicy: PasswordPolicy;
  };
  branding: {
    logo?: string;
    primaryColor?: string;
    secondaryColor?: string;
    customDomain?: string;
  };
}

export interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSpecialChars: boolean;
  maxAge: number;
}

export interface TenantMember {
  id: string;
  userId: string;
  tenantId: string;
  email: string;
  name: string;
  role: TenantRole;
  status: MemberStatus;
  invitedAt?: string;
  joinedAt?: string;
  lastActiveAt?: string;
}

export enum TenantRole {
  OWNER = 'owner',
  ADMIN = 'admin',
  MEMBER = 'member',
  VIEWER = 'viewer',
}

export enum MemberStatus {
  ACTIVE = 'active',
  INVITED = 'invited',
  SUSPENDED = 'suspended',
}

export interface TenantInvitation {
  id: string;
  tenantId: string;
  email: string;
  role: TenantRole;
  invitedBy: string;
  invitedAt: string;
  expiresAt: string;
  status: InvitationStatus;
  token: string;
}

export enum InvitationStatus {
  PENDING = 'pending',
  ACCEPTED = 'accepted',
  EXPIRED = 'expired',
  CANCELLED = 'cancelled',
}

export interface CreateTenantRequest {
  name: string;
  slug?: string;
  description?: string;
  subscriptionTier: SubscriptionTier;
  adminEmail: string;
}

export interface UpdateTenantRequest {
  name?: string;
  description?: string;
  settings?: Partial<TenantSettings>;
}

export interface InviteMemberRequest {
  email: string;
  role: TenantRole;
  message?: string;
}

export interface UpdateMemberRequest {
  role?: TenantRole;
  status?: MemberStatus;
}

export interface TenantSwitchRequest {
  targetTenantId: string;
  currentTenantId?: string;
}

export interface TenantSwitchResult {
  success: boolean;
  newTenantId: string;
  newSessionId: string;
  tenantContext: Tenant;
}

export interface WorkflowResponse<T> {
  type: 'sync' | 'async';
  data?: T;
  operationId?: string;
  statusUrl?: string;
  streamUrl?: string;
}

export interface WorkflowProgress {
  currentStep: string;
  totalSteps: number;
  completedSteps: number;
  percentage: number;
  message?: string;
}