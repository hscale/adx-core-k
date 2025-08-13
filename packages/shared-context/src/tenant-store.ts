import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { TenantState, Tenant } from './types';

interface TenantActions {
  setCurrentTenant: (tenant: Tenant | null) => void;
  setAvailableTenants: (tenants: Tenant[]) => void;
  switchTenant: (tenantId: string) => Promise<void>;
  updateTenant: (tenantId: string, updates: Partial<Tenant>) => void;
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useTenantStore = create<TenantState & TenantActions>()(
  persist(
    (set, get) => ({
      // State
      currentTenant: null,
      availableTenants: [],
      loading: false,
      error: null,

      // Actions
      setCurrentTenant: (tenant: Tenant | null) => {
        set({ currentTenant: tenant });
      },

      setAvailableTenants: (tenants: Tenant[]) => {
        set({ availableTenants: tenants });
      },

      switchTenant: async (tenantId: string) => {
        const { availableTenants } = get();
        const targetTenant = availableTenants.find(t => t.id === tenantId);
        
        if (!targetTenant) {
          set({ error: 'Tenant not found' });
          return;
        }

        set({ loading: true, error: null });

        try {
          // Call API to switch tenant
          const response = await fetch('/api/workflows/switch-tenant', {
            method: 'POST',
            headers: {
              'Content-Type': 'application/json',
              'Authorization': `Bearer ${localStorage.getItem('auth-token')}`,
            },
            body: JSON.stringify({
              targetTenantId: tenantId,
              currentTenantId: get().currentTenant?.id,
            }),
          });

          if (!response.ok) {
            throw new Error('Failed to switch tenant');
          }

          const result = await response.json();

          // Handle workflow response
          if (result.operationId) {
            // Poll for completion if it's an async workflow
            await pollWorkflowStatus(result.operationId);
          }

          set({ 
            currentTenant: targetTenant,
            loading: false,
            error: null 
          });

          // Emit tenant switch event
          window.dispatchEvent(new CustomEvent('tenant:switched', {
            detail: {
              previousTenantId: get().currentTenant?.id,
              newTenantId: tenantId,
              tenantContext: targetTenant,
            }
          }));

        } catch (error) {
          set({ 
            loading: false, 
            error: error instanceof Error ? error.message : 'Failed to switch tenant' 
          });
        }
      },

      updateTenant: (tenantId: string, updates: Partial<Tenant>) => {
        const { currentTenant, availableTenants } = get();
        
        // Update current tenant if it matches
        if (currentTenant?.id === tenantId) {
          set({ currentTenant: { ...currentTenant, ...updates } });
        }

        // Update in available tenants list
        const updatedTenants = availableTenants.map(tenant =>
          tenant.id === tenantId ? { ...tenant, ...updates } : tenant
        );
        set({ availableTenants: updatedTenants });
      },

      setLoading: (loading: boolean) => {
        set({ loading });
      },

      setError: (error: string | null) => {
        set({ error, loading: false });
      },

      clearError: () => {
        set({ error: null });
      },
    }),
    {
      name: 'adx-tenant-storage',
      partialize: (state) => ({
        currentTenant: state.currentTenant,
        availableTenants: state.availableTenants,
      }),
    }
  )
);

// Helper function to poll workflow status
async function pollWorkflowStatus(operationId: string): Promise<any> {
  return new Promise((resolve, reject) => {
    const poll = async () => {
      try {
        const response = await fetch(`/api/workflows/${operationId}/status`, {
          headers: {
            'Authorization': `Bearer ${localStorage.getItem('auth-token')}`,
          },
        });

        if (!response.ok) {
          throw new Error('Failed to check workflow status');
        }

        const status = await response.json();

        switch (status.status) {
          case 'completed':
            resolve(status.result);
            break;
          case 'failed':
            reject(new Error(status.error || 'Workflow failed'));
            break;
          case 'running':
          case 'pending':
            setTimeout(poll, 1000); // Poll every second
            break;
          default:
            reject(new Error(`Unknown workflow status: ${status.status}`));
        }
      } catch (error) {
        reject(error);
      }
    };

    poll();
  });
}