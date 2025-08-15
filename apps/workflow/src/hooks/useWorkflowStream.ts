import { useState, useEffect, useCallback } from 'react';
import { useTenantContext } from '@adx-core/shared-context';
import { WorkflowBFFClient } from '../services';
import { Workflow } from '../types';

const workflowBFFClient = new WorkflowBFFClient();

// Get auth token from context or localStorage
const getAuthToken = (): string => {
  return localStorage.getItem('authToken') || '';
};

export const useWorkflowStream = (workflowId: string) => {
  const { state: tenantState } = useTenantContext();
  const [workflow, setWorkflow] = useState<Workflow | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [error, setError] = useState<Error | null>(null);

  const connect = useCallback(() => {
    if (!tenantState.currentTenant || !workflowId) {
      return;
    }

    workflowBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      getAuthToken()
    );

    setError(null);
    setIsConnected(true);

    const cleanup = workflowBFFClient.streamWorkflowProgress(
      workflowId,
      (updatedWorkflow) => {
        setWorkflow(updatedWorkflow);
      },
      (streamError) => {
        setError(streamError);
        setIsConnected(false);
      }
    );

    return cleanup;
  }, [workflowId, tenantState.currentTenant]);

  useEffect(() => {
    const cleanup = connect();
    
    return () => {
      if (cleanup) {
        cleanup();
      }
      setIsConnected(false);
    };
  }, [connect]);

  const reconnect = useCallback(() => {
    const cleanup = connect();
    return cleanup;
  }, [connect]);

  return {
    workflow,
    isConnected,
    error,
    reconnect,
  };
};

export const useMultipleWorkflowStreams = (workflowIds: string[]) => {
  const { state: tenantState } = useTenantContext();
  const [workflows, setWorkflows] = useState<Record<string, Workflow>>({});
  const [connections, setConnections] = useState<Record<string, boolean>>({});
  const [errors, setErrors] = useState<Record<string, Error>>({});

  useEffect(() => {
    if (!tenantState.currentTenant || workflowIds.length === 0) {
      return;
    }

    workflowBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      getAuthToken()
    );

    const cleanupFunctions: Record<string, () => void> = {};

    workflowIds.forEach((workflowId) => {
      setConnections(prev => ({ ...prev, [workflowId]: true }));
      setErrors(prev => {
        const newErrors = { ...prev };
        delete newErrors[workflowId];
        return newErrors;
      });

      const cleanup = workflowBFFClient.streamWorkflowProgress(
        workflowId,
        (updatedWorkflow) => {
          setWorkflows(prev => ({
            ...prev,
            [workflowId]: updatedWorkflow,
          }));
        },
        (error) => {
          setErrors(prev => ({ ...prev, [workflowId]: error }));
          setConnections(prev => ({ ...prev, [workflowId]: false }));
        }
      );

      cleanupFunctions[workflowId] = cleanup;
    });

    return () => {
      Object.values(cleanupFunctions).forEach(cleanup => cleanup());
      setConnections({});
    };
  }, [workflowIds, tenantState.currentTenant]);

  const reconnect = useCallback((workflowId: string) => {
    if (!tenantState.currentTenant) {
      return;
    }

    workflowBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      getAuthToken()
    );

    setConnections(prev => ({ ...prev, [workflowId]: true }));
    setErrors(prev => {
      const newErrors = { ...prev };
      delete newErrors[workflowId];
      return newErrors;
    });

    return workflowBFFClient.streamWorkflowProgress(
      workflowId,
      (updatedWorkflow) => {
        setWorkflows(prev => ({
          ...prev,
          [workflowId]: updatedWorkflow,
        }));
      },
      (error) => {
        setErrors(prev => ({ ...prev, [workflowId]: error }));
        setConnections(prev => ({ ...prev, [workflowId]: false }));
      }
    );
  }, [tenantState.currentTenant]);

  return {
    workflows,
    connections,
    errors,
    reconnect,
  };
};