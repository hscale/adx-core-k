import { z } from 'zod';

// Tenant-related types
export interface Tenant {
  id: string;
  name: string;
  displayName: string;
  description?: string;
  adminEmail: string;
  subscriptionTier: SubscriptionTier;
  status: TenantStatus;
  features: string[];
  quotas: TenantQuotas;
  settings: TenantSettings;
  branding: TenantBranding;
  createdAt: string;
  updatedAt: string;
  lastActivityAt?: string;
}

export enum SubscriptionTier {
  FREE = 'free',
  PROFESSIONAL = 'professional',
  ENTERPRISE = 'enterprise',
  CUSTOM = 'custom',
}

export enum TenantStatus {
  ACTIVE = 'active',
  SUSPENDED = 'suspended',
  PENDING = 'pending',
  ARCHIVED = 'archived',
}

export interface TenantQuotas {
  maxUsers: number;
  maxStorageGB: number;
  maxApiCallsPerHour: number;
  maxWorkflowsPerHour: number;
  maxModules: number;
  maxCustomDomains: number;
}

export interface TenantSettings {
  timezone: string;
  locale: string;
  dateFormat: string;
  timeFormat: string;
  currency: string;
  allowUserRegistration: boolean;
  requireEmailVerification: boolean;
  enableMFA: boolean;
  sessionTimeoutMinutes: number;
  passwordPolicy: PasswordPolicy;
  auditLogRetentionDays: number;
}

export interface PasswordPolicy {
  minLength: number;
  requireUppercase: boolean;
  requireLowercase: boolean;
  requireNumbers: boolean;
  requireSpecialChars: boolean;
  preventReuse: number;
  maxAge: number;
}

export interface TenantBranding {
  logoUrl?: string;
  faviconUrl?: string;
  primaryColor: string;
  secondaryColor: string;
  accentColor: string;
  customCSS?: string;
  customDomain?: string;
  emailTemplates: EmailTemplates;
}

export interface EmailTemplates {
  welcomeEmail?: string;
  passwordResetEmail?: string;
  invitationEmail?: string;
  notificationEmail?: string;
}

// Tenant membership types
export interface TenantMembership {
  id: string;
  tenantId: string;
  userId: string;
  userEmail: string;
  userName: string;
  roles: string[];
  permissions: string[];
  status: MembershipStatus;
  invitedBy?: string;
  invitedAt?: string;
  joinedAt?: string;
  lastActiveAt?: string;
}

export enum MembershipStatus {
  ACTIVE = 'active',
  INVITED = 'invited',
  SUSPENDED = 'suspended',
  REMOVED = 'removed',
}

// Tenant analytics types
export interface TenantAnalytics {
  tenantId: string;
  period: AnalyticsPeriod;
  metrics: TenantMetrics;
  usage: TenantUsage;
  trends: TenantTrends;
  generatedAt: string;
}

export enum AnalyticsPeriod {
  HOUR = 'hour',
  DAY = 'day',
  WEEK = 'week',
  MONTH = 'month',
  QUARTER = 'quarter',
  YEAR = 'year',
}

export interface TenantMetrics {
  activeUsers: number;
  totalUsers: number;
  newUsersThisPeriod: number;
  apiCallsThisPeriod: number;
  workflowExecutionsThisPeriod: number;
  storageUsedGB: number;
  averageSessionDuration: number;
  errorRate: number;
  uptime: number;
}

export interface TenantUsage {
  quotaUsage: QuotaUsage;
  featureUsage: FeatureUsage;
  resourceUsage: ResourceUsage;
}

export interface QuotaUsage {
  users: { used: number; limit: number; percentage: number };
  storage: { used: number; limit: number; percentage: number };
  apiCalls: { used: number; limit: number; percentage: number };
  workflows: { used: number; limit: number; percentage: number };
  modules: { used: number; limit: number; percentage: number };
}

export interface FeatureUsage {
  [featureName: string]: {
    enabled: boolean;
    usageCount: number;
    lastUsed?: string;
  };
}

export interface ResourceUsage {
  cpu: { average: number; peak: number };
  memory: { average: number; peak: number };
  bandwidth: { inbound: number; outbound: number };
  database: { connections: number; queries: number };
}

export interface TenantTrends {
  userGrowth: TrendData[];
  usageGrowth: TrendData[];
  errorTrend: TrendData[];
  performanceTrend: TrendData[];
}

export interface TrendData {
  timestamp: string;
  value: number;
  change?: number;
  changePercentage?: number;
}

// Tenant switching types
export interface TenantSwitchRequest {
  targetTenantId: string;
  currentTenantId?: string;
  preserveSession?: boolean;
}

export interface TenantSwitchResult {
  success: boolean;
  newTenantId: string;
  newSessionId?: string;
  tenantContext: TenantContext;
  availableFeatures: string[];
  redirectUrl?: string;
}

export interface TenantContext {
  tenant: Tenant;
  membership: TenantMembership;
  permissions: string[];
  features: string[];
  quotas: TenantQuotas;
  settings: TenantSettings;
  branding: TenantBranding;
}

// Workflow types
export interface TenantWorkflowRequest {
  workflowType: string;
  tenantId: string;
  userId: string;
  data: any;
  options?: WorkflowOptions;
}

export interface WorkflowOptions {
  synchronous?: boolean;
  timeout?: number;
  retryPolicy?: RetryPolicy;
  priority?: WorkflowPriority;
}

export interface RetryPolicy {
  maxAttempts: number;
  backoffMultiplier: number;
  initialInterval: number;
  maxInterval: number;
}

export enum WorkflowPriority {
  LOW = 'low',
  NORMAL = 'normal',
  HIGH = 'high',
  CRITICAL = 'critical',
}

export interface WorkflowResponse<T = any> {
  type: 'sync' | 'async';
  operationId?: string;
  statusUrl?: string;
  streamUrl?: string;
  data?: T;
  estimatedDuration?: number;
}

// Validation schemas
export const TenantSwitchRequestSchema = z.object({
  targetTenantId: z.string().min(1, 'Target tenant ID is required'),
  currentTenantId: z.string().optional(),
  preserveSession: z.boolean().optional().default(false),
});

export type TenantSwitchRequestInput = z.input<typeof TenantSwitchRequestSchema>;

export const TenantWorkflowRequestSchema = z.object({
  workflowType: z.string().min(1, 'Workflow type is required'),
  tenantId: z.string().min(1, 'Tenant ID is required'),
  userId: z.string().min(1, 'User ID is required'),
  data: z.any().optional(),
  options: z.object({
    synchronous: z.boolean().optional().default(false),
    timeout: z.number().positive().optional(),
    retryPolicy: z.object({
      maxAttempts: z.number().positive(),
      backoffMultiplier: z.number().positive(),
      initialInterval: z.number().positive(),
      maxInterval: z.number().positive(),
    }).optional(),
    priority: z.nativeEnum(WorkflowPriority).optional().default(WorkflowPriority.NORMAL),
  }).optional(),
});

export type TenantWorkflowRequestInput = z.input<typeof TenantWorkflowRequestSchema>;

export const AnalyticsPeriodSchema = z.nativeEnum(AnalyticsPeriod);

// Error types
export class TenantError extends Error {
  constructor(
    message: string,
    public code: string,
    public statusCode: number = 500,
    public details?: any
  ) {
    super(message);
    this.name = 'TenantError';
  }
}

export class TenantNotFoundError extends TenantError {
  constructor(tenantId: string) {
    super(`Tenant not found: ${tenantId}`, 'TENANT_NOT_FOUND', 404);
  }
}

export class TenantAccessDeniedError extends TenantError {
  constructor(tenantId: string, userId: string) {
    super(`Access denied to tenant ${tenantId} for user ${userId}`, 'TENANT_ACCESS_DENIED', 403);
  }
}

export class TenantSuspendedError extends TenantError {
  constructor(tenantId: string) {
    super(`Tenant is suspended: ${tenantId}`, 'TENANT_SUSPENDED', 403);
  }
}

export class QuotaExceededError extends TenantError {
  constructor(quotaType: string, current: number, limit: number) {
    super(
      `Quota exceeded for ${quotaType}: ${current}/${limit}`,
      'QUOTA_EXCEEDED',
      429,
      { quotaType, current, limit }
    );
  }
}