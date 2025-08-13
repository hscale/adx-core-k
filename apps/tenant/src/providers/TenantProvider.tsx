import React, { createContext, useContext, useEffect } from 'react';
import { useEventBus } from '@adx-core/event-bus';
import { useCurrentTenant, useUserTenants } from '../hooks';
import { tenantBFFClient } from '../services';
import { Tenant } from '../types';

interface TenantContextType {
  currentTenant: Tenant | null;
  availableTenants: Tenant[];
  isLoading: boolean;
  error: string | null;
}

const TenantContext = createContext<TenantContextType | null>(null);

interface TenantProviderProps {
  children: React.ReactNode;
}

export const TenantProvider: React.FC<TenantProviderProps> = ({ children }) => {
  const { emit, subscribe } = useEventBus();
  const { 
    data: currentTenant, 
    isLoading: currentTenantLoading, 
    error: currentTenantError 
  } = useCurrentTenant();
  const { 
    data: availableTenants, 
    isLoading: tenantsLoading, 
    error: tenantsError 
  } = useUserTenants();

  // Set up BFF client context when tenant changes
  useEffect(() => {
    if (currentTenant) {
      // Get auth token from storage or auth context
      const authToken = localStorage.getItem('auth_token') || '';
      tenantBFFClient.setContext(currentTenant.id, authToken);
    }
  }, [currentTenant]);

  // Subscribe to tenant-related events from other micro-frontends
  useEffect(() => {
    const unsubscribeTenantEvents = subscribe('tenant:*', (event) => {
      switch (event.type) {
        case 'tenant:switched':
          // Handle tenant switch from other micro-frontends
          console.log('Tenant switched:', event.data);
          break;
        case 'tenant:updated':
          // Handle tenant updates from other micro-frontends
          console.log('Tenant updated:', event.data);
          break;
        case 'tenant:member_invited':
          // Handle member invitation events
          console.log('Member invited:', event.data);
          break;
        case 'tenant:member_updated':
          // Handle member update events
          console.log('Member updated:', event.data);
          break;
        case 'tenant:member_removed':
          // Handle member removal events
          console.log('Member removed:', event.data);
          break;
        default:
          break;
      }
    });

    return unsubscribeTenantEvents;
  }, [subscribe]);

  // Emit tenant context changes to other micro-frontends
  useEffect(() => {
    if (currentTenant) {
      emit('tenant:context_updated', {
        currentTenant,
        availableTenants: availableTenants || [],
      });
    }
  }, [currentTenant, availableTenants, emit]);

  const contextValue: TenantContextType = {
    currentTenant: currentTenant || null,
    availableTenants: availableTenants || [],
    isLoading: currentTenantLoading || tenantsLoading,
    error: currentTenantError?.message || tenantsError?.message || null,
  };

  return (
    <TenantContext.Provider value={contextValue}>
      {children}
    </TenantContext.Provider>
  );
};

export const useTenantProvider = () => {
  const context = useContext(TenantContext);
  if (!context) {
    throw new Error('useTenantProvider must be used within TenantProvider');
  }
  return context;
};