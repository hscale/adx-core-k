import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useTenantContext } from '@adx-core/shared-context';
import { ModuleBFFClient } from '../services/moduleBFFClient';
import { ModuleDevelopmentProject } from '../types/module';

const moduleBFFClient = new ModuleBFFClient();

export const useModuleDevelopment = () => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  // Set tenant context when it changes
  if (tenantState.currentTenant) {
    moduleBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      localStorage.getItem('authToken') || ''
    );
  }

  const projects = useQuery({
    queryKey: ['module-development', 'projects', tenantState.currentTenant?.id],
    queryFn: () => moduleBFFClient.getDevelopmentProjects(),
    enabled: !!tenantState.currentTenant,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });

  const createProject = useMutation({
    mutationFn: (project: Omit<ModuleDevelopmentProject, 'id' | 'created' | 'lastModified'>) =>
      moduleBFFClient.createDevelopmentProject(project),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['module-development', 'projects'] });
    },
  });

  const updateProject = useMutation({
    mutationFn: ({ projectId, updates }: { 
      projectId: string; 
      updates: Partial<ModuleDevelopmentProject> 
    }) =>
      moduleBFFClient.updateDevelopmentProject(projectId, updates),
    onSuccess: (_, { projectId }) => {
      queryClient.invalidateQueries({ queryKey: ['module-development', 'projects'] });
      queryClient.invalidateQueries({ queryKey: ['module-development', 'project', projectId] });
    },
  });

  const testModule = useMutation({
    mutationFn: async (projectId: string) => {
      const response = await moduleBFFClient.testModule(projectId);
      
      if (response.type === 'async' && response.operationId) {
        return moduleBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (_, projectId) => {
      queryClient.invalidateQueries({ queryKey: ['module-development', 'project', projectId] });
    },
  });

  const publishModule = useMutation({
    mutationFn: async (projectId: string) => {
      const response = await moduleBFFClient.publishModule(projectId);
      
      if (response.type === 'async' && response.operationId) {
        return moduleBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (_, projectId) => {
      queryClient.invalidateQueries({ queryKey: ['module-development', 'project', projectId] });
      queryClient.invalidateQueries({ queryKey: ['modules', 'search'] });
    },
  });

  return {
    projects,
    createProject,
    updateProject,
    testModule,
    publishModule,
  };
};

export const useModuleDevelopmentProject = (projectId: string) => {
  const { state: tenantState } = useTenantContext();
  const queryClient = useQueryClient();

  if (tenantState.currentTenant) {
    moduleBFFClient.setTenantContext(
      tenantState.currentTenant.id,
      localStorage.getItem('authToken') || ''
    );
  }

  const project = useQuery({
    queryKey: ['module-development', 'project', projectId],
    queryFn: () => moduleBFFClient.getDevelopmentProject(projectId),
    enabled: !!tenantState.currentTenant && !!projectId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });

  const updateProject = useMutation({
    mutationFn: (updates: Partial<ModuleDevelopmentProject>) =>
      moduleBFFClient.updateDevelopmentProject(projectId, updates),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['module-development', 'project', projectId] });
      queryClient.invalidateQueries({ queryKey: ['module-development', 'projects'] });
    },
  });

  return {
    project,
    updateProject,
  };
};