import React, { createContext, useContext, useEffect, useState } from 'react'
import { Tenant } from '@/types'
import { apiService } from '@/services/api'
import { useQuery } from '@tanstack/react-query'

interface TenantContextType {
  tenant: Tenant | null
  isLoading: boolean
  switchTenant: (tenantId: string) => Promise<void>
  availableTenants: Tenant[]
}

const TenantContext = createContext<TenantContextType | undefined>(undefined)

export function TenantProvider({ children }: { children: React.ReactNode }) {
  const [currentTenantId, setCurrentTenantId] = useState<string | null>(() => {
    return localStorage.getItem('tenant_id')
  })

  // Get current tenant
  const {
    data: tenant,
    isLoading: tenantLoading,
  } = useQuery({
    queryKey: ['tenant', currentTenantId],
    queryFn: () => apiService.get<Tenant>(`/tenants/${currentTenantId}`),
    enabled: !!currentTenantId,
    staleTime: 10 * 60 * 1000, // 10 minutes
  })

  // Get available tenants for the user
  const {
    data: availableTenants = [],
    isLoading: tenantsLoading,
  } = useQuery({
    queryKey: ['tenants', 'available'],
    queryFn: () => apiService.get<Tenant[]>('/tenants/available'),
    staleTime: 5 * 60 * 1000, // 5 minutes
  })

  // Switch tenant
  const switchTenant = async (tenantId: string) => {
    setCurrentTenantId(tenantId)
    localStorage.setItem('tenant_id', tenantId)
    
    // Refresh the page to reload all data with new tenant context
    window.location.reload()
  }

  // Apply tenant branding
  useEffect(() => {
    if (tenant?.settings?.branding) {
      const { primaryColor, favicon } = tenant.settings.branding
      
      // Apply primary color
      if (primaryColor) {
        document.documentElement.style.setProperty('--color-primary', primaryColor)
      }
      
      // Update favicon
      if (favicon) {
        const link = document.querySelector("link[rel*='icon']") as HTMLLinkElement
        if (link) {
          link.href = favicon
        }
      }
      
      // Update page title
      if (tenant.name) {
        document.title = `${tenant.name} - ADX CORE`
      }
    }
  }, [tenant])

  const value: TenantContextType = {
    tenant: tenant || null,
    isLoading: tenantLoading || tenantsLoading,
    switchTenant,
    availableTenants,
  }

  return <TenantContext.Provider value={value}>{children}</TenantContext.Provider>
}

export function useTenant() {
  const context = useContext(TenantContext)
  if (context === undefined) {
    throw new Error('useTenant must be used within a TenantProvider')
  }
  return context
}