export interface User {
  id: string;
  email: string;
  name: string;
  avatar?: string;
  roles: string[];
  permissions: string[];
}

export interface Tenant {
  id: string;
  name: string;
  slug: string;
  features: string[];
  quotas: Record<string, QuotaInfo>;
  settings: TenantSettings;
  subscriptionTier: SubscriptionTier;
}

export interface QuotaInfo {
  used: number;
  limit: number;
  unit: string;
}

export interface TenantSettings {
  theme: 'light' | 'dark' | 'system';
  language: string;
  timezone: string;
  dateFormat: string;
  currency: string;
}

export enum SubscriptionTier {
  Free = 'free',
  Professional = 'professional',
  Enterprise = 'enterprise'
}

export interface AuthState {
  user: User | null;
  token: string | null;
  isAuthenticated: boolean;
  isLoading: boolean;
  error: string | null;
}

export interface TenantState {
  currentTenant: Tenant | null;
  availableTenants: Tenant[];
  loading: boolean;
  error: string | null;
}

export interface ThemeState {
  theme: 'light' | 'dark' | 'system';
  resolvedTheme: 'light' | 'dark';
  systemTheme: 'light' | 'dark';
}

export interface ThemePreferences {
  theme: 'light' | 'dark' | 'system';
  accentColor?: string;
  fontSize?: 'sm' | 'md' | 'lg';
  reducedMotion?: boolean;
  highContrast?: boolean;
}

export interface I18nState {
  language: string;
  availableLanguages: string[];
  isLoading: boolean;
}