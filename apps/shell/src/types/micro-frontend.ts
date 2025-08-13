export interface MicroFrontendConfig {
  name: string;
  url: string;
  scope: string;
  module: string;
  displayName: string;
  icon?: string;
  route: string;
  permissions?: string[];
  enabled: boolean;
}

export interface MicroFrontendError {
  name: string;
  error: Error;
  timestamp: number;
  retryCount: number;
}

export interface MicroFrontendStatus {
  name: string;
  status: 'loading' | 'loaded' | 'error' | 'retrying';
  lastUpdated: number;
}