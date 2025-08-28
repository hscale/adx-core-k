import React, { createContext, useContext, useState } from 'react';

interface Tenant {
  id: string;
  name: string;
  features: string[];
  quotas: Record<string, any>;
  settings: Record<string, any>;
}

interface TenantContextType {
  currentTenant: Tenant | null;
  availableTenants: Tenant[];
  switchTenant: (tenantId: string) => Promise<void>;
  refreshTenantData: () => Promise<void>;
}

const TenantContext = createContext<TenantContextType | undefined>(undefined);

export const useTenantContext = () => {
  const context = useContext(TenantContext);
  if (!context) {
    throw new Error('useTenantContext must be used within a TenantProvider');
  }
  return context;
};

interface TenantProviderProps {
  children: React.ReactNode;
}

export const TenantProvider: React.FC<TenantProviderProps> = ({ children }) => {
  const [currentTenant, setCurrentTenant] = useState<Tenant | null>(null);
  const [availableTenants, setAvailableTenants] = useState<Tenant[]>([]);

  const switchTenant = async (tenantId: string) => {
    // Mock implementation - replace with actual API call
    const tenant = availableTenants.find(t => t.id === tenantId);
    if (tenant) {
      setCurrentTenant(tenant);
    }
  };

  const refreshTenantData = async () => {
    // Mock implementation - replace with actual API call
    console.log('Refreshing tenant data...');
  };

  return (
    <TenantContext.Provider value={{
      currentTenant,
      availableTenants,
      switchTenant,
      refreshTenantData,
    }}>
      {children}
    </TenantContext.Provider>
  );
};