export interface Workflow {
  id: string;
  type: string;
  status: WorkflowStatus;
  progress: WorkflowProgress;
  startedAt: string;
  updatedAt: string;
  completedAt?: string;
  duration?: number;
  result?: any;
  error?: WorkflowError;
  metadata: WorkflowMetadata;
  activities: WorkflowActivity[];
}

export enum WorkflowStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled',
  TIMED_OUT = 'timed_out',
}

export interface WorkflowProgress {
  currentStep: string;
  totalSteps: number;
  completedSteps: number;
  percentage: number;
  message?: string;
  estimatedTimeRemaining?: number;
}

export interface WorkflowError {
  code: string;
  message: string;
  details?: any;
  stackTrace?: string;
  retryable: boolean;
}

export interface WorkflowMetadata {
  tenantId: string;
  userId: string;
  userEmail: string;
  source: string;
  tags: string[];
  priority: WorkflowPriority;
  retryCount: number;
  maxRetries: number;
}

export enum WorkflowPriority {
  LOW = 'low',
  NORMAL = 'normal',
  HIGH = 'high',
  CRITICAL = 'critical',
}

export interface WorkflowActivity {
  id: string;
  name: string;
  status: ActivityStatus;
  startedAt: string;
  completedAt?: string;
  duration?: number;
  input?: any;
  output?: any;
  error?: WorkflowError;
  retryCount: number;
}

export enum ActivityStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled',
}

export interface WorkflowTemplate {
  id: string;
  name: string;
  description: string;
  category: string;
  version: string;
  parameters: WorkflowParameter[];
  estimatedDuration: number;
  complexity: WorkflowComplexity;
}

export interface WorkflowParameter {
  name: string;
  type: string;
  required: boolean;
  description: string;
  defaultValue?: any;
  validation?: ParameterValidation;
}

export interface ParameterValidation {
  min?: number;
  max?: number;
  pattern?: string;
  options?: string[];
}

export enum WorkflowComplexity {
  SIMPLE = 'simple',
  MODERATE = 'moderate',
  COMPLEX = 'complex',
}

export interface WorkflowMetrics {
  totalWorkflows: number;
  runningWorkflows: number;
  completedWorkflows: number;
  failedWorkflows: number;
  averageDuration: number;
  successRate: number;
  throughput: number;
  errorRate: number;
}

export interface WorkflowAnalytics {
  timeRange: string;
  metrics: WorkflowMetrics;
  trends: WorkflowTrend[];
  topWorkflowTypes: WorkflowTypeStats[];
  errorAnalysis: ErrorAnalysis;
  performanceMetrics: PerformanceMetrics;
}

export interface WorkflowTrend {
  timestamp: string;
  totalExecutions: number;
  successfulExecutions: number;
  failedExecutions: number;
  averageDuration: number;
}

export interface WorkflowTypeStats {
  type: string;
  count: number;
  successRate: number;
  averageDuration: number;
  errorRate: number;
}

export interface ErrorAnalysis {
  topErrors: ErrorStats[];
  errorTrends: ErrorTrend[];
  retryAnalysis: RetryAnalysis;
}

export interface ErrorStats {
  errorCode: string;
  errorMessage: string;
  count: number;
  percentage: number;
  lastOccurrence: string;
}

export interface ErrorTrend {
  timestamp: string;
  errorCount: number;
  errorRate: number;
}

export interface RetryAnalysis {
  totalRetries: number;
  averageRetriesPerWorkflow: number;
  retrySuccessRate: number;
  maxRetriesReached: number;
}

export interface PerformanceMetrics {
  p50Duration: number;
  p95Duration: number;
  p99Duration: number;
  throughputPerHour: number;
  resourceUtilization: ResourceUtilization;
}

export interface ResourceUtilization {
  cpuUsage: number;
  memoryUsage: number;
  networkUsage: number;
  storageUsage: number;
}

export interface WorkflowFilter {
  status?: WorkflowStatus[];
  type?: string[];
  dateRange?: {
    start: string;
    end: string;
  };
  userId?: string;
  tags?: string[];
  priority?: WorkflowPriority[];
}

export interface WorkflowSearchParams {
  query?: string;
  filter?: WorkflowFilter;
  sortBy?: string;
  sortOrder?: 'asc' | 'desc';
  page?: number;
  limit?: number;
}

export interface WorkflowListResponse {
  workflows: Workflow[];
  total: number;
  page: number;
  limit: number;
  hasMore: boolean;
}