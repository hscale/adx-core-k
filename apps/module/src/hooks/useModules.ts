import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTenantContext } from '@adx-core/shared-context';
import { ModuleBFFClient } from '../services/moduleBFFClient';
import { 
  Module, 
  ModuleSearchFilters, 
  ModuleInstallRequest,
  ModuleConfiguration 
} from '../types/module';

const moduleBFFClient = new ModuleBFFClient();

export const useModules = () => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  // Set tenant context when it changes
  if (tenantState.currentTenant) {
    moduleBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      localStorage.getItem('authToken') || ''
    );
  }

  const searchModules = useQuery({
    queryKey: ['modules', 'search'],
    queryFn: () => moduleBFFClient.searchModules(),
    enabled: !!tenantState.currentTenant,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });

  const featuredModules = useQuery({
    queryKey: ['modules', 'featured'],
    queryFn: () => moduleBFFClient.getFeaturedModules(),
    enabled: !!tenantState.currentTenant,
    staleTime: 10 * 60 * 1000, // 10 minutes
  });

  const trendingModules = useQuery({
    queryKey: ['modules', 'trending'],
    queryFn: () => moduleBFFClient.getTrendingModules(),
    enabled: !!tenantState.currentTenant,
    staleTime: 10 * 60 * 1000, // 10 minutes
  });

  const recommendedModules = useQuery({
    queryKey: ['modules', 'recommended'],
    queryFn: () => moduleBFFClient.getRecommendedModules(),
    enabled: !!tenantState.currentTenant,
    staleTime: 15 * 60 * 1000, // 15 minutes
  });

  const installedModules = useQuery({
    queryKey: ['modules', 'installed', tenantState.currentTenant?.id],
    queryFn: () => moduleBFFClient.getInstalledModules(),
    enabled: !!tenantState.currentTenant,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });

  const installModule = useMutation({
    mutationFn: async (request: ModuleInstallRequest) => {
      const response = await moduleBFFClient.installModule(request);
      
      if (response.type === 'async' && response.operationId) {
        return moduleBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['modules', 'installed'] });
      queryClient.invalidateQueries({ queryKey: ['modules', 'search'] });
    },
  });

  const uninstallModule = useMutation({
    mutationFn: async (moduleId: string) => {
      const response = await moduleBFFClient.uninstallModule(moduleId);
      
      if (response.type === 'async' && response.operationId) {
        return moduleBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['modules', 'installed'] });
    },
  });

  const activateModule = useMutation({
    mutationFn: async (moduleId: string) => {
      const response = await moduleBFFClient.activateModule(moduleId);
      
      if (response.type === 'async' && response.operationId) {
        return moduleBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['modules', 'installed'] });
    },
  });

  const deactivateModule = useMutation({
    mutationFn: async (moduleId: string) => {
      const response = await moduleBFFClient.deactivateModule(moduleId);
      
      if (response.type === 'async' && response.operationId) {
        return moduleBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['modules', 'installed'] });
    },
  });

  return {
    searchModules,
    featuredModules,
    trendingModules,
    recommendedModules,
    installedModules,
    installModule,
    uninstallModule,
    activateModule,
    deactivateModule,
  };
};

export const useModuleSearch = (
  query?: string,
  filters?: ModuleSearchFilters,
  page: number = 1,
  pageSize: number = 20
) => {
  const { state: tenantState } = useTenantContext();

  if (tenantState.currentTenant) {
    moduleBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      localStorage.getItem('authToken') || ''
    );
  }

  return useQuery({
    queryKey: ['modules', 'search', query, filters, page, pageSize],
    queryFn: () => moduleBFFClient.searchModules(query, filters, page, pageSize),
    enabled: !!tenantState.currentTenant,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
};

export const useModule = (moduleId: string) => {
  const { state: tenantState } = useTenantContext();

  if (tenantState.currentTenant) {
    moduleBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      localStorage.getItem('authToken') || ''
    );
  }

  return useQuery({
    queryKey: ['modules', 'detail', moduleId],
    queryFn: () => moduleBFFClient.getModule(moduleId),
    enabled: !!tenantState.currentTenant && !!moduleId,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

export const useModuleConfiguration = (moduleId: string) => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  if (tenantState.currentTenant) {
    moduleBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      localStorage.getItem('authToken') || ''
    );
  }

  const configuration = useQuery({
    queryKey: ['modules', 'configuration', moduleId],
    queryFn: () => moduleBFFClient.getModuleConfiguration(moduleId),
    enabled: !!tenantState.currentTenant && !!moduleId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });

  const updateConfiguration = useMutation({
    mutationFn: (updates: Partial<ModuleConfiguration>) =>
      moduleBFFClient.updateModuleConfiguration(moduleId, updates),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['modules', 'configuration', moduleId] });
    },
  });

  return {
    configuration,
    updateConfiguration,
  };
};