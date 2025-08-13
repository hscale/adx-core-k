import type { WorkflowResponse } from '../types';

// Utility function to extract data from workflow response
export function extractWorkflowData<T>(response: T | WorkflowResponse<T>): T {
  if (response && typeof response === 'object' && 'type' in response) {
    const workflowResponse = response as WorkflowResponse<T>;
    if (workflowResponse.data) {
      return workflowResponse.data;
    }
  }
  return response as T;
}