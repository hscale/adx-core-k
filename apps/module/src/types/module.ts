export interface Module {
  id: string;
  name: string;
  version: string;
  description: string;
  longDescription?: string;
  author: ModuleAuthor;
  category: ModuleCategory;
  subcategory?: string;
  price?: number;
  pricingModel: PricingModel;
  rating: number;
  reviewCount: number;
  downloads: number;
  activeInstallations: number;
  screenshots: Screenshot[];
  demoUrl?: string;
  documentationUrl: string;
  supportUrl: string;
  tags: string[];
  supportedPlatforms: Platform[];
  compatibility: CompatibilityInfo;
  securityScanResults: SecurityScanResults;
  performanceMetrics: PerformanceMetrics;
  lastUpdated: string;
  changelog: ChangelogEntry[];
  status: ModuleStatus;
  installationStatus?: InstallationStatus;
}

export interface ModuleAuthor {
  name: string;
  email: string;
  website?: string;
  avatar?: string;
}

export enum ModuleCategory {
  BusinessManagement = 'business-management',
  Analytics = 'analytics',
  Communication = 'communication',
  Integration = 'integration',
  Security = 'security',
  Productivity = 'productivity',
  Development = 'development',
  AI = 'ai',
  Other = 'other',
}

export enum PricingModel {
  Free = 'free',
  OneTime = 'one-time',
  Subscription = 'subscription',
  Usage = 'usage',
  Enterprise = 'enterprise',
}

export interface Screenshot {
  url: string;
  caption?: string;
  thumbnail?: string;
}

export enum Platform {
  Web = 'web',
  Desktop = 'desktop',
  Mobile = 'mobile',
}

export interface CompatibilityInfo {
  minAdxVersion: string;
  maxAdxVersion?: string;
  supportedPlatforms: Platform[];
  dependencies: ModuleDependency[];
}

export interface ModuleDependency {
  moduleId: string;
  version: string;
  optional: boolean;
}

export interface SecurityScanResults {
  passed: boolean;
  score: number;
  issues: SecurityIssue[];
  lastScanned: string;
}

export interface SecurityIssue {
  severity: 'low' | 'medium' | 'high' | 'critical';
  type: string;
  description: string;
  recommendation?: string;
}

export interface PerformanceMetrics {
  bundleSize: number;
  loadTime: number;
  memoryUsage: number;
  cpuUsage: number;
}

export interface ChangelogEntry {
  version: string;
  date: string;
  changes: string[];
  breaking?: boolean;
}

export enum ModuleStatus {
  Published = 'published',
  Draft = 'draft',
  Deprecated = 'deprecated',
  Suspended = 'suspended',
}

export enum InstallationStatus {
  NotInstalled = 'not-installed',
  Installing = 'installing',
  Installed = 'installed',
  Activating = 'activating',
  Active = 'active',
  Deactivating = 'deactivating',
  Uninstalling = 'uninstalling',
  Failed = 'failed',
}

export interface ModuleSearchFilters {
  category?: ModuleCategory;
  pricingModel?: PricingModel;
  platform?: Platform;
  rating?: number;
  priceRange?: [number, number];
  tags?: string[];
  author?: string;
}

export interface ModuleSearchResults {
  modules: Module[];
  total: number;
  page: number;
  pageSize: number;
  filters: ModuleSearchFilters;
}

export interface ModuleInstallRequest {
  moduleId: string;
  version?: string;
  tenantId: string;
}

export interface ModuleInstallResponse {
  operationId: string;
  status: 'sync' | 'async';
  result?: ModuleInstallResult;
  statusUrl?: string;
}

export interface ModuleInstallResult {
  moduleId: string;
  version: string;
  installationId: string;
  status: InstallationStatus;
}

export interface ModuleConfiguration {
  moduleId: string;
  settings: Record<string, any>;
  permissions: string[];
  resources: ResourceLimits;
  enabled: boolean;
}

export interface ResourceLimits {
  memory: string;
  cpu: string;
  storage: string;
  networkAccess: boolean;
}

export interface ModuleDevelopmentProject {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  created: string;
  lastModified: string;
  status: 'draft' | 'testing' | 'ready' | 'published';
  manifest: ModuleManifest;
  sourceFiles: SourceFile[];
  testResults?: TestResults;
}

export interface ModuleManifest {
  name: string;
  version: string;
  description: string;
  author: ModuleAuthor;
  license: string;
  adxCore: {
    minVersion: string;
    maxVersion?: string;
  };
  dependencies: Record<string, string>;
  permissions: string[];
  extensionPoints: {
    backend?: {
      activities?: string[];
      workflows?: string[];
      endpoints?: string[];
    };
    frontend?: {
      components?: string[];
      routes?: string[];
      hooks?: string[];
    };
  };
  resources: ResourceLimits;
}

export interface SourceFile {
  path: string;
  content: string;
  language: string;
  lastModified: string;
}

export interface TestResults {
  passed: number;
  failed: number;
  total: number;
  coverage: number;
  details: TestCase[];
}

export interface TestCase {
  name: string;
  status: 'passed' | 'failed' | 'skipped';
  duration: number;
  error?: string;
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