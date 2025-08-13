import React, { useState } from 'react';
import { ChevronDownIcon, CheckIcon, PlusIcon } from 'lucide-react';
import { useCurrentTenant, useUserTenants, useSwitchTenant } from '../hooks';
import { formatSubscriptionTier, getSubscriptionTierColorClass } from '../utils';
import { Tenant } from '../types';

interface TenantSwitcherProps {
  onCreateTenant?: () => void;
  className?: string;
}

export const TenantSwitcher: React.FC<TenantSwitcherProps> = ({
  onCreateTenant,
  className = '',
}) => {
  const [isOpen, setIsOpen] = useState(false);
  const { data: currentTenant, isLoading: currentTenantLoading } = useCurrentTenant();
  const { data: userTenants, isLoading: tenantsLoading } = useUserTenants();
  const switchTenantMutation = useSwitchTenant();

  const handleTenantSwitch = async (tenant: Tenant) => {
    if (tenant.id === currentTenant?.id) {
      setIsOpen(false);
      return;
    }

    try {
      await switchTenantMutation.mutateAsync({
        targetTenantId: tenant.id,
        currentTenantId: currentTenant?.id,
      });
      setIsOpen(false);
    } catch (error) {
      console.error('Failed to switch tenant:', error);
    }
  };

  const handleCreateTenant = () => {
    setIsOpen(false);
    onCreateTenant?.();
  };

  if (currentTenantLoading || tenantsLoading) {
    return (
      <div className={`animate-pulse ${className}`}>
        <div className="h-10 bg-gray-200 rounded-lg"></div>
      </div>
    );
  }

  if (!currentTenant) {
    return (
      <div className={`text-sm text-gray-500 ${className}`}>
        No tenant selected
      </div>
    );
  }

  return (
    <div className={`relative ${className}`}>
      <button
        onClick={() => setIsOpen(!isOpen)}
        disabled={switchTenantMutation.isPending}
        className="flex items-center justify-between w-full px-3 py-2 text-left bg-white border border-gray-300 rounded-lg shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        <div className="flex items-center space-x-3">
          <div className="flex-shrink-0">
            <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
              <span className="text-sm font-medium text-blue-600">
                {currentTenant.name.charAt(0).toUpperCase()}
              </span>
            </div>
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-gray-900 truncate">
              {currentTenant.name}
            </p>
            <p className="text-xs text-gray-500">
              {formatSubscriptionTier(currentTenant.subscriptionTier)}
            </p>
          </div>
        </div>
        <ChevronDownIcon
          className={`w-4 h-4 text-gray-400 transition-transform ${
            isOpen ? 'transform rotate-180' : ''
          }`}
        />
      </button>

      {isOpen && (
        <>
          {/* Backdrop */}
          <div
            className="fixed inset-0 z-10"
            onClick={() => setIsOpen(false)}
          />
          
          {/* Dropdown */}
          <div className="absolute z-20 w-full mt-1 bg-white border border-gray-200 rounded-lg shadow-lg max-h-80 overflow-auto">
            <div className="py-1">
              {userTenants?.map((tenant) => (
                <button
                  key={tenant.id}
                  onClick={() => handleTenantSwitch(tenant)}
                  disabled={switchTenantMutation.isPending}
                  className="flex items-center justify-between w-full px-3 py-2 text-left hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  <div className="flex items-center space-x-3">
                    <div className="flex-shrink-0">
                      <div className="w-8 h-8 bg-blue-100 rounded-full flex items-center justify-center">
                        <span className="text-sm font-medium text-blue-600">
                          {tenant.name.charAt(0).toUpperCase()}
                        </span>
                      </div>
                    </div>
                    <div className="flex-1 min-w-0">
                      <p className="text-sm font-medium text-gray-900 truncate">
                        {tenant.name}
                      </p>
                      <div className="flex items-center space-x-2">
                        <span
                          className={`inline-flex items-center px-2 py-0.5 rounded text-xs font-medium ${getSubscriptionTierColorClass(
                            tenant.subscriptionTier
                          )}`}
                        >
                          {formatSubscriptionTier(tenant.subscriptionTier)}
                        </span>
                      </div>
                    </div>
                  </div>
                  {tenant.id === currentTenant.id && (
                    <CheckIcon className="w-4 h-4 text-blue-600" />
                  )}
                </button>
              ))}
              
              {onCreateTenant && (
                <>
                  <div className="border-t border-gray-200 my-1" />
                  <button
                    onClick={handleCreateTenant}
                    className="flex items-center w-full px-3 py-2 text-left hover:bg-gray-50"
                  >
                    <div className="flex items-center space-x-3">
                      <div className="flex-shrink-0">
                        <div className="w-8 h-8 bg-gray-100 rounded-full flex items-center justify-center">
                          <PlusIcon className="w-4 h-4 text-gray-600" />
                        </div>
                      </div>
                      <div>
                        <p className="text-sm font-medium text-gray-900">
                          Create new tenant
                        </p>
                        <p className="text-xs text-gray-500">
                          Set up a new organization
                        </p>
                      </div>
                    </div>
                  </button>
                </>
              )}
            </div>
          </div>
        </>
      )}
      
      {switchTenantMutation.isPending && (
        <div className="absolute inset-0 bg-white bg-opacity-75 flex items-center justify-center rounded-lg">
          <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-blue-600"></div>
        </div>
      )}
    </div>
  );
};