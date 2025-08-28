import { useEffect, useCallback } from 'react';
import { useEventBus } from './EventBus';
import type { EventHandler, EventType } from './types';

/**
 * Hook to subscribe to a specific event type
 */
export const useEventSubscription = (
  eventType: EventType,
  handler: EventHandler,
  deps: React.DependencyList = []
) => {
  const { subscribe } = useEventBus();

  useEffect(() => {
    const unsubscribe = subscribe(eventType, handler);
    return unsubscribe;
  }, [subscribe, eventType, ...deps]);
};

/**
 * Hook to subscribe to events matching a pattern
 */
export const useEventPatternSubscription = (
  pattern: string,
  handler: EventHandler,
  deps: React.DependencyList = []
) => {
  const { subscribePattern } = useEventBus();

  useEffect(() => {
    const unsubscribe = subscribePattern(pattern, handler);
    return unsubscribe;
  }, [subscribePattern, pattern, ...deps]);
};

/**
 * Hook to emit events
 */
export const useEventEmitter = () => {
  const { emit } = useEventBus();

  return useCallback((eventType: EventType, data?: any, source?: string) => {
    emit(eventType, data, source);
  }, [emit]);
};

/**
 * Hook for tenant-related events
 */
export const useTenantEvents = () => {
  const emit = useEventEmitter();

  return {
    emitTenantSwitched: (data: { previousTenantId?: string; newTenantId: string; tenantContext: any }) =>
      emit('tenant:switched', data, 'tenant-context'),
    
    emitTenantUpdated: (data: { tenantId: string; updates: any }) =>
      emit('tenant:updated', data, 'tenant-context'),
    
    emitQuotaUpdated: (data: { tenantId: string; quotaType: string; usage: any }) =>
      emit('tenant:quota_updated', data, 'tenant-context'),
    
    emitFeatureToggled: (data: { tenantId: string; feature: string; enabled: boolean }) =>
      emit('tenant:feature_toggled', data, 'tenant-context'),
  };
};

/**
 * Hook for authentication events
 */
export const useAuthEvents = () => {
  const emit = useEventEmitter();

  return {
    emitUserLogin: (data: { userId: string; tenantId: string }) =>
      emit('auth:login', data, 'auth-context'),
    
    emitUserLogout: (data: { userId: string; reason?: string }) =>
      emit('auth:logout', data, 'auth-context'),
    
    emitTokenRefresh: (data: { userId: string; newToken: string }) =>
      emit('auth:token_refresh', data, 'auth-context'),
    
    emitPermissionChanged: (data: { userId: string; permissions: string[] }) =>
      emit('auth:permission_changed', data, 'auth-context'),
  };
};

/**
 * Hook for workflow events
 */
export const useWorkflowEvents = () => {
  const emit = useEventEmitter();

  return {
    emitWorkflowStarted: (data: { operationId: string; workflowType: string }) =>
      emit('workflow:started', data, 'workflow-context'),
    
    emitWorkflowCompleted: (data: { operationId: string; result: any }) =>
      emit('workflow:completed', data, 'workflow-context'),
    
    emitWorkflowFailed: (data: { operationId: string; error: string }) =>
      emit('workflow:failed', data, 'workflow-context'),
    
    emitWorkflowProgress: (data: { operationId: string; progress: any }) =>
      emit('workflow:progress', data, 'workflow-context'),
  };
};