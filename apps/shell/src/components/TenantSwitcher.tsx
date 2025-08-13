import React, { useState } from 'react';
import { ChevronDown, Building2, Loader2 } from 'lucide-react';
import { useTenantStore } from '@adx-core/shared-context';
import { Button } from '@adx-core/design-system';

interface TenantSwitcherProps {
  mobile?: boolean;
}

export const TenantSwitcher: React.FC<TenantSwitcherProps> = ({ mobile = false }) => {
  const [isOpen, setIsOpen] = useState(false);
  const { currentTenant, availableTenants, switchTenant, loading } = useTenantStore();

  const handleTenantSwitch = async (tenantId: string) => {
    if (tenantId === currentTenant?.id) {
      setIsOpen(false);
      return;
    }

    try {
      await switchTenant(tenantId);
      setIsOpen(false);
      
      // Reload the page to refresh all micro-frontends with new tenant context
      window.location.reload();
    } catch (error) {
      console.error('Failed to switch tenant:', error);
    }
  };

  if (!currentTenant) {
    return null;
  }

  if (mobile) {
    return (
      <div className="px-4 py-2">
        <div className="text-sm font-medium text-gray-500 dark:text-gray-400 mb-2">
          Current Tenant
        </div>
        <select
          value={currentTenant.id}
          onChange={(e) => handleTenantSwitch(e.target.value)}
          disabled={loading}
          className="block w-full px-3 py-2 border border-gray-300 dark:border-gray-600 rounded-md shadow-sm bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-primary-500 focus:border-primary-500"
        >
          {availableTenants.map((tenant: any) => (
            <option key={tenant.id} value={tenant.id}>
              {tenant.name}
            </option>
          ))}
        </select>
        {loading && (
          <div className="mt-2 flex items-center text-sm text-gray-500 dark:text-gray-400">
            <Loader2 className="mr-2 h-4 w-4 animate-spin" />
            Switching tenant...
          </div>
        )}
      </div>
    );
  }

  return (
    <div className="relative">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
        disabled={loading}
        className="flex items-center space-x-2 min-w-[160px] justify-between"
      >
        <div className="flex items-center space-x-2">
          <Building2 className="h-4 w-4" />
          <span className="truncate">{currentTenant.name}</span>
        </div>
        {loading ? (
          <Loader2 className="h-4 w-4 animate-spin" />
        ) : (
          <ChevronDown className="h-4 w-4" />
        )}
      </Button>

      {isOpen && !loading && (
        <div className="absolute right-0 mt-2 w-64 rounded-md shadow-lg bg-white dark:bg-gray-800 ring-1 ring-black ring-opacity-5 z-50">
          <div className="py-1">
            <div className="px-4 py-2 text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider border-b border-gray-200 dark:border-gray-700">
              Switch Tenant
            </div>
            {availableTenants.map((tenant: any) => (
              <button
                key={tenant.id}
                onClick={() => handleTenantSwitch(tenant.id)}
                className={`block w-full text-left px-4 py-2 text-sm transition-colors ${
                  tenant.id === currentTenant.id
                    ? 'bg-primary-50 text-primary-700 dark:bg-primary-900/20 dark:text-primary-300'
                    : 'text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700'
                }`}
              >
                <div className="flex items-center space-x-2">
                  <Building2 className="h-4 w-4" />
                  <div>
                    <div className="font-medium">{tenant.name}</div>
                    <div className="text-xs text-gray-500 dark:text-gray-400">
                      {tenant.subscriptionTier}
                    </div>
                  </div>
                </div>
              </button>
            ))}
          </div>
        </div>
      )}
    </div>
  );
};