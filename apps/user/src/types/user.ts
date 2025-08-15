export interface User {
  id: string;
  email: string;
  firstName: string;
  lastName: string;
  displayName: string;
  avatar?: string;
  phone?: string;
  timezone: string;
  language: string;
  roles: string[];
  permissions: string[];
  tenantId: string;
  isActive: boolean;
  lastLoginAt?: string;
  createdAt: string;
  updatedAt: string;
}

export interface UserProfile {
  id: string;
  userId: string;
  bio?: string;
  department?: string;
  jobTitle?: string;
  location?: string;
  website?: string;
  socialLinks: {
    linkedin?: string;
    twitter?: string;
    github?: string;
  };
  preferences: UserPreferences;
}

export interface UserPreferences {
  theme: 'light' | 'dark' | 'system';
  language: string;
  timezone: string;
  dateFormat: string;
  timeFormat: '12h' | '24h';
  notifications: {
    email: boolean;
    push: boolean;
    desktop: boolean;
    workflow: boolean;
    mentions: boolean;
  };
  privacy: {
    profileVisibility: 'public' | 'team' | 'private';
    showOnlineStatus: boolean;
    allowDirectMessages: boolean;
  };
}

export interface UserSettings {
  security: {
    mfaEnabled: boolean;
    sessionTimeout: number;
    allowedIpRanges?: string[];
  };
  integrations: {
    [key: string]: {
      enabled: boolean;
      config: Record<string, any>;
    };
  };
  quotas: {
    storageUsed: number;
    storageLimit: number;
    apiCallsUsed: number;
    apiCallsLimit: number;
    workflowsUsed: number;
    workflowsLimit: number;
  };
}

export interface CreateUserRequest {
  email: string;
  firstName: string;
  lastName: string;
  roles: string[];
  tenantId: string;
  sendInvitation?: boolean;
}

export interface UpdateUserRequest {
  firstName?: string;
  lastName?: string;
  displayName?: string;
  phone?: string;
  timezone?: string;
  language?: string;
  roles?: string[];
}

export interface UpdateUserProfileRequest {
  bio?: string;
  department?: string;
  jobTitle?: string;
  location?: string;
  website?: string;
  socialLinks?: {
    linkedin?: string;
    twitter?: string;
    github?: string;
  };
}

export interface UpdateUserPreferencesRequest {
  theme?: 'light' | 'dark' | 'system';
  language?: string;
  timezone?: string;
  dateFormat?: string;
  timeFormat?: '12h' | '24h';
  notifications?: Partial<UserPreferences['notifications']>;
  privacy?: Partial<UserPreferences['privacy']>;
}

export interface UserSearchFilters {
  query?: string;
  roles?: string[];
  departments?: string[];
  isActive?: boolean;
  lastLoginAfter?: string;
  lastLoginBefore?: string;
}

export interface UserSearchResult {
  users: User[];
  total: number;
  page: number;
  pageSize: number;
  hasMore: boolean;
}

export interface UserActivity {
  id: string;
  userId: string;
  type: 'login' | 'logout' | 'profile_update' | 'password_change' | 'workflow_execution' | 'file_upload';
  description: string;
  metadata?: Record<string, any>;
  ipAddress?: string;
  userAgent?: string;
  timestamp: string;
}

export interface UserInvitation {
  id: string;
  email: string;
  roles: string[];
  invitedBy: string;
  invitedAt: string;
  expiresAt: string;
  status: 'pending' | 'accepted' | 'expired' | 'cancelled';
  acceptedAt?: string;
}

export interface WorkflowResponse<T> {
  type: 'sync' | 'async';
  data?: T;
  operationId?: string;
  statusUrl?: string;
  streamUrl?: string;
}

export interface UserBFFClient {
  getUser(userId: string): Promise<User>;
  getUserProfile(userId: string): Promise<UserProfile>;
  getUserSettings(userId: string): Promise<UserSettings>;
  updateUser(userId: string, updates: UpdateUserRequest): Promise<WorkflowResponse<User>>;
  updateUserProfile(userId: string, updates: UpdateUserProfileRequest): Promise<WorkflowResponse<UserProfile>>;
  updateUserPreferences(userId: string, updates: UpdateUserPreferencesRequest): Promise<WorkflowResponse<UserPreferences>>;
  searchUsers(filters: UserSearchFilters, page?: number, pageSize?: number): Promise<UserSearchResult>;
  getUserActivity(userId: string, limit?: number): Promise<UserActivity[]>;
  inviteUser(invitation: CreateUserRequest): Promise<WorkflowResponse<UserInvitation>>;
  deactivateUser(userId: string): Promise<WorkflowResponse<void>>;
  reactivateUser(userId: string): Promise<WorkflowResponse<void>>;
}