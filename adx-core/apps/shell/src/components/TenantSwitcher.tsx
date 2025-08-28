import React, { useState } from 'react';
import { ChevronDownIcon, CheckIcon } from 'lucide-react';
import { useTenantContext } from '@adx-core/shared-context';
import { Button } from '@adx-core/design-system';

const TenantSwitcher: React.FC = () => {
  const { currentTenant, availableTenants, switchTenant } = useTenantContext();
  const [isOpen, setIsOpen] = useState(false);
  const [switching, setSwitching] = useState(false);

  const handleTenantSwitch = async (tenantId: string) => {
    if (tenantId === currentTenant?.id || switching) return;
    
    setSwitching(true);
    setIsOpen(false);
    
    try {
      await switchTenant(tenantId);
    } catch (error) {
      console.error('Failed to switch tenant:', error);
    } finally {
      setSwitching(false);
    }
  };

  if (!currentTenant || availableTenants.length <= 1) {
    return null;
  }

  return (
    <div className="relative">
      <Button
        variant="outline"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
        disabled={switching}
        className="flex items-center space-x-2"
      >
        <span className="truncate max-w-32">{currentTenant.name}</span>
        <ChevronDownIcon className="h-4 w-4" />
      </Button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-56 bg-white dark:bg-gray-800 rounded-md shadow-lg ring-1 ring-black ring-opacity-5 z-50">
          <div className="py-1">
            {availableTenants.map((tenant) => (
              <button
                key={tenant.id}
                onClick={() => handleTenantSwitch(tenant.id)}
                className="flex items-center justify-between w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
              >
                <span className="truncate">{tenant.name}</span>
                {tenant.id === currentTenant.id && (
                  <CheckIcon className="h-4 w-4 text-blue-500" />
                )}
              </button>
            ))}
          </div>
        </div>
      )}

      {switching && (
        <div className="absolute inset-0 bg-white bg-opacity-75 dark:bg-gray-800 dark:bg-opacity-75 rounded-md flex items-center justify-center">
          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-500"></div>
        </div>
      )}
    </div>
  );
};

export default TenantSwitcher;