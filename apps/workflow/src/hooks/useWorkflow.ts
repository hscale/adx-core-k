import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTenantContext } from '@adx-core/shared-context';
import { WorkflowBFFClient } from '../services';
import { 
  Workflow, 
  WorkflowSearchParams, 
  WorkflowAnalytics, 
  WorkflowMetrics,
  WorkflowTemplate 
} from '../types';

const workflowBFFClient = new WorkflowBFFClient();

// Get auth token from context or localStorage
const getAuthToken = (): string => {
  return localStorage.getItem('authToken') || '';
};

export const useWorkflows = (params: WorkflowSearchParams = {}) => {
  const { state: tenantState } = useTenantContext();

  return useQuery({
    queryKey: ['workflows', tenantState.currentTenant?.id, params],
    queryFn: async () => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.getWorkflows(params);
    },
    enabled: !!tenantState.currentTenant,
    staleTime: 30 * 1000, // 30 seconds
    refetchInterval: 60 * 1000, // Refetch every minute
  });
};

export const useWorkflow = (workflowId: string) => {
  const { state: tenantState } = useTenantContext();

  return useQuery({
    queryKey: ['workflow', tenantState.currentTenant?.id, workflowId],
    queryFn: async () => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.getWorkflow(workflowId);
    },
    enabled: !!tenantState.currentTenant && !!workflowId,
    staleTime: 10 * 1000, // 10 seconds
    refetchInterval: 5 * 1000, // Refetch every 5 seconds for active workflows
  });
};

export const useRunningWorkflows = () => {
  const { state: tenantState } = useTenantContext();

  return useQuery({
    queryKey: ['workflows', 'running', tenantState.currentTenant?.id],
    queryFn: async () => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.getRunningWorkflows();
    },
    enabled: !!tenantState.currentTenant,
    staleTime: 5 * 1000, // 5 seconds
    refetchInterval: 10 * 1000, // Refetch every 10 seconds
  });
};

export const useWorkflowAnalytics = (timeRange: string = '7d') => {
  const { state: tenantState } = useTenantContext();

  return useQuery({
    queryKey: ['workflow-analytics', tenantState.currentTenant?.id, timeRange],
    queryFn: async () => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.getWorkflowAnalytics(timeRange);
    },
    enabled: !!tenantState.currentTenant,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const useWorkflowMetrics = () => {
  const { state: tenantState } = useTenantContext();

  return useQuery({
    queryKey: ['workflow-metrics', tenantState.currentTenant?.id],
    queryFn: async () => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.getWorkflowMetrics();
    },
    enabled: !!tenantState.currentTenant,
    staleTime: 30 * 1000, // 30 seconds
    refetchInterval: 60 * 1000, // Refetch every minute
  });
};

export const useWorkflowTemplates = () => {
  const { state: tenantState } = useTenantContext();

  return useQuery({
    queryKey: ['workflow-templates', tenantState.currentTenant?.id],
    queryFn: async () => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.getWorkflowTemplates();
    },
    enabled: !!tenantState.currentTenant,
    staleTime: 10 * 60 * 1000, // 10 minutes
  });
};

export const useCancelWorkflow = () => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ workflowId, reason }: { workflowId: string; reason?: string }) => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.cancelWorkflow(workflowId, reason);
    },
    onSuccess: () => {
      // Invalidate workflow queries to refresh data
      queryClient.invalidateQueries(['workflows', tenantState.currentTenant?.id]);
      queryClient.invalidateQueries(['workflow-metrics', tenantState.currentTenant?.id]);
    },
  });
};

export const useRetryWorkflow = () => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async (workflowId: string) => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.retryWorkflow(workflowId);
    },
    onSuccess: () => {
      // Invalidate workflow queries to refresh data
      queryClient.invalidateQueries(['workflows', tenantState.currentTenant?.id]);
      queryClient.invalidateQueries(['workflow-metrics', tenantState.currentTenant?.id]);
    },
  });
};

export const useStartWorkflow = () => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ 
      templateId, 
      parameters 
    }: { 
      templateId: string; 
      parameters: Record<string, any> 
    }) => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.startWorkflowFromTemplate(templateId, parameters);
    },
    onSuccess: () => {
      // Invalidate workflow queries to refresh data
      queryClient.invalidateQueries(['workflows', tenantState.currentTenant?.id]);
      queryClient.invalidateQueries(['workflow-metrics', tenantState.currentTenant?.id]);
    },
  });
};

export const useBulkCancelWorkflows = () => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: async ({ 
      workflowIds, 
      reason 
    }: { 
      workflowIds: string[]; 
      reason?: string 
    }) => {
      if (!tenantState.currentTenant) {
        throw new Error('No tenant context available');
      }

      workflowBFFClient.setTenantContext(
        tenantState.currentTenant.id,
        getAuthToken()
      );

      return workflowBFFClient.bulkCancelWorkflows(workflowIds, reason);
    },
    onSuccess: () => {
      // Invalidate workflow queries to refresh data
      queryClient.invalidateQueries(['workflows', tenantState.currentTenant?.id]);
      queryClient.invalidateQueries(['workflow-metrics', tenantState.currentTenant?.id]);
    },
  });
};