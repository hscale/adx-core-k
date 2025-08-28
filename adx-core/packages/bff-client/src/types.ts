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
  completed: number;
  percentage: number;
  message?: string;
}