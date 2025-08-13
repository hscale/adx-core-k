export interface LoginRequest {
  email: string;
  password: string;
  tenantId?: string;
  rememberMe?: boolean;
}

export interface RegisterRequest {
  email: string;
  password: string;
  confirmPassword: string;
  firstName: string;
  lastName: string;
  tenantName?: string;
  acceptTerms: boolean;
}

export interface ForgotPasswordRequest {
  email: string;
}

export interface ResetPasswordRequest {
  token: string;
  password: string;
  confirmPassword: string;
}

export interface MFASetupRequest {
  secret: string;
  code: string;
}

export interface MFAVerifyRequest {
  code: string;
}

export interface SSOLoginRequest {
  provider: SSOProvider;
  redirectUrl?: string;
}

export interface AuthResponse {
  user: {
    id: string;
    email: string;
    name: string;
    avatar?: string;
    roles: string[];
    permissions: string[];
  };
  token: string;
  refreshToken: string;
  expiresAt: string;
  mfaRequired?: boolean;
  mfaSetupRequired?: boolean;
}

export interface WorkflowResponse<T> {
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
    currentStep: string;
    totalSteps: number;
    completedSteps: number;
    percentage: number;
    message?: string;
  };
  result?: any;
  error?: string;
  startedAt: string;
  updatedAt: string;
  estimatedCompletion?: string;
}

export enum SSOProvider {
  GOOGLE = 'google',
  MICROSOFT = 'microsoft',
  GITHUB = 'github',
  OKTA = 'okta',
  SAML = 'saml'
}

export interface MFASecret {
  secret: string;
  qrCode: string;
  backupCodes: string[];
}

export interface AuthError {
  code: string;
  message: string;
  details?: Record<string, any>;
  validationErrors?: Array<{
    field: string;
    code: string;
    message: string;
  }>;
}

export interface BFFClientConfig {
  baseUrl: string;
  timeout?: number;
  retries?: number;
}

export interface AuthFormState {
  isLoading: boolean;
  error: string | null;
  success: string | null;
}