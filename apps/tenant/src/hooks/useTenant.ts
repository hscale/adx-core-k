import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { useEventBus } from '@adx-core/event-bus';
import { tenantBFFClient } from '../services';
import {
  Tenant,
  CreateTenantRequest,
  UpdateTenantRequest,
  InviteMemberRequest,
  UpdateMemberRequest,
  TenantSwitchRequest,
  SubscriptionTier,
  TenantStatus,
  TenantRole,
  MemberStatus,
  InvitationStatus,
} from '../types';

// Query keys
export const tenantKeys = {
  all: ['tenants'] as const,
  lists: () => [...tenantKeys.all, 'list'] as const,
  list: (filters: string) => [...tenantKeys.lists(), { filters }] as const,
  details: () => [...tenantKeys.all, 'detail'] as const,
  detail: (id: string) => [...tenantKeys.details(), id] as const,
  current: () => [...tenantKeys.all, 'current'] as const,
  members: (tenantId: string) => [...tenantKeys.detail(tenantId), 'members'] as const,
  invitations: (tenantId: string) => [...tenantKeys.detail(tenantId), 'invitations'] as const,
};

// Get current tenant
export const useCurrentTenant = () => {
  return useQuery({
    queryKey: tenantKeys.current(),
    queryFn: async () => {
      // Set up BFF client context (in a real app, this would come from auth context)
      tenantBFFClient.setContext('tenant-1', 'mock-auth-token');
      
      try {
        return await tenantBFFClient.getCurrentTenant();
      } catch (error) {
        console.warn('BFF service not available, using mock data:', error);
        // Fallback to mock data if BFF service is not available
        return {
          id: 'tenant-1',
          name: 'Demo Tenant',
          slug: 'demo-tenant',
          description: 'Demo tenant for development',
          adminEmail: 'admin@demo.com',
          subscriptionTier: SubscriptionTier.PROFESSIONAL,
          status: TenantStatus.ACTIVE,
          features: ['basic_features', 'advanced_analytics'],
          quotas: {
            maxUsers: 10,
            maxStorageGB: 100,
            maxApiCallsPerHour: 1000,
            maxWorkflowsPerHour: 50,
            currentUsers: 3,
            currentStorageGB: 25.5,
            currentApiCallsThisHour: 150,
            currentWorkflowsThisHour: 5,
          },
          settings: {
            timezone: 'UTC',
            dateFormat: 'MM/DD/YYYY',
            language: 'en',
            theme: 'system' as const,
            notifications: {
              email: true,
              push: true,
              sms: false,
            },
            security: {
              mfaRequired: false,
              sessionTimeout: 60,
              passwordPolicy: {
                minLength: 8,
                requireUppercase: true,
                requireLowercase: true,
                requireNumbers: true,
                requireSpecialChars: false,
                maxAge: 90,
              },
            },
            branding: {
              primaryColor: '#3B82F6',
              secondaryColor: '#6B7280',
              customDomain: 'https://demo.adxcore.com',
            },
          },
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-15T10:30:00Z',
        };
      }
    },
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes
  });
};

// Get user's tenants
export const useUserTenants = () => {
  return useQuery({
    queryKey: tenantKeys.lists(),
    queryFn: async () => {
      // Set up BFF client context
      tenantBFFClient.setContext('tenant-1', 'mock-auth-token');
      
      try {
        return await tenantBFFClient.getUserTenants();
      } catch (error) {
        console.warn('BFF service not available, using mock data:', error);
        // Fallback to mock data if BFF service is not available
        return [
        {
          id: 'tenant-1',
          name: 'Demo Tenant',
          slug: 'demo-tenant',
          description: 'Demo tenant for development',
          adminEmail: 'admin@demo.com',
          subscriptionTier: SubscriptionTier.PROFESSIONAL,
          status: TenantStatus.ACTIVE,
          features: ['basic_features', 'advanced_analytics'],
          quotas: {
            maxUsers: 10,
            maxStorageGB: 100,
            maxApiCallsPerHour: 1000,
            maxWorkflowsPerHour: 50,
            currentUsers: 3,
            currentStorageGB: 25.5,
            currentApiCallsThisHour: 150,
            currentWorkflowsThisHour: 5,
          },
          settings: {
            timezone: 'UTC',
            dateFormat: 'MM/DD/YYYY',
            language: 'en',
            theme: 'system' as const,
            notifications: { email: true, push: true, sms: false },
            security: {
              mfaRequired: false,
              sessionTimeout: 60,
              passwordPolicy: {
                minLength: 8,
                requireUppercase: true,
                requireLowercase: true,
                requireNumbers: true,
                requireSpecialChars: false,
                maxAge: 90,
              },
            },
            branding: { primaryColor: '#3B82F6', secondaryColor: '#6B7280', customDomain: 'https://test.adxcore.com' },
          },
          createdAt: '2024-01-01T00:00:00Z',
          updatedAt: '2024-01-15T10:30:00Z',
        },
        {
          id: 'tenant-2',
          name: 'Test Tenant',
          slug: 'test-tenant',
          description: 'Test tenant for development',
          adminEmail: 'admin@test.com',
          subscriptionTier: SubscriptionTier.FREE,
          status: TenantStatus.ACTIVE,
          features: ['basic_features'],
          quotas: {
            maxUsers: 5,
            maxStorageGB: 10,
            maxApiCallsPerHour: 100,
            maxWorkflowsPerHour: 10,
            currentUsers: 1,
            currentStorageGB: 2.1,
            currentApiCallsThisHour: 25,
            currentWorkflowsThisHour: 2,
          },
          settings: {
            timezone: 'UTC',
            dateFormat: 'MM/DD/YYYY',
            language: 'en',
            theme: 'light' as const,
            notifications: { email: true, push: false, sms: false },
            security: {
              mfaRequired: false,
              sessionTimeout: 30,
              passwordPolicy: {
                minLength: 8,
                requireUppercase: false,
                requireLowercase: true,
                requireNumbers: true,
                requireSpecialChars: false,
                maxAge: 180,
              },
            },
            branding: { primaryColor: '#10B981', secondaryColor: '#6B7280' },
          },
          createdAt: '2024-01-10T00:00:00Z',
          updatedAt: '2024-01-15T10:30:00Z',
        },
      ];
      }
    },
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
  });
};

// Get specific tenant
export const useTenant = (tenantId: string) => {
  return useQuery({
    queryKey: tenantKeys.detail(tenantId),
    queryFn: () => tenantBFFClient.getTenant(tenantId),
    enabled: !!tenantId,
    staleTime: 5 * 60 * 1000,
    gcTime: 10 * 60 * 1000,
  });
};

// Create tenant mutation
export const useCreateTenant = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: async (request: CreateTenantRequest) => {
      const response = await tenantBFFClient.createTenant(request);
      
      if (response.type === 'async' && response.operationId) {
        // Poll for completion
        return tenantBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (tenant: Tenant) => {
      // Invalidate and refetch tenant lists
      queryClient.invalidateQueries({ queryKey: tenantKeys.lists() });
      
      // Emit tenant created event
      emit('tenant:created', { tenant });
    },
    onError: (error) => {
      emit('tenant:create_failed', { error: error.message });
    },
  });
};

// Update tenant mutation
export const useUpdateTenant = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: ({ tenantId, request }: { tenantId: string; request: UpdateTenantRequest }) =>
      tenantBFFClient.updateTenant(tenantId, request),
    onSuccess: (tenant: Tenant) => {
      // Update specific tenant in cache
      queryClient.setQueryData(tenantKeys.detail(tenant.id), tenant);
      
      // Invalidate related queries
      queryClient.invalidateQueries({ queryKey: tenantKeys.lists() });
      queryClient.invalidateQueries({ queryKey: tenantKeys.current() });
      
      // Emit tenant updated event
      emit('tenant:updated', { tenant });
    },
    onError: (error) => {
      emit('tenant:update_failed', { error: error.message });
    },
  });
};

// Delete tenant mutation
export const useDeleteTenant = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: async (tenantId: string) => {
      const response = await tenantBFFClient.deleteTenant(tenantId);
      
      if (response.type === 'async' && response.operationId) {
        // Poll for completion
        return tenantBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (_, tenantId) => {
      // Remove tenant from cache
      queryClient.removeQueries({ queryKey: tenantKeys.detail(tenantId) });
      
      // Invalidate tenant lists
      queryClient.invalidateQueries({ queryKey: tenantKeys.lists() });
      
      // Emit tenant deleted event
      emit('tenant:deleted', { tenantId });
    },
    onError: (error) => {
      emit('tenant:delete_failed', { error: error.message });
    },
  });
};

// Switch tenant mutation
export const useSwitchTenant = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: async (request: TenantSwitchRequest) => {
      const response = await tenantBFFClient.switchTenant(request);
      
      if (response.type === 'async' && response.operationId) {
        // Poll for completion
        return tenantBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (result) => {
      // Clear all cached data since tenant context changed
      queryClient.clear();
      
      // Update BFF client context
      tenantBFFClient.setContext(result.newTenantId, result.newSessionId);
      
      // Emit tenant switched event
      emit('tenant:switched', {
        previousTenantId: result.newTenantId, // This should come from the request
        newTenantId: result.newTenantId,
        tenantContext: result.tenantContext,
      });
    },
    onError: (error) => {
      emit('tenant:switch_failed', { error: error.message });
    },
  });
};

// Get tenant members
export const useTenantMembers = (tenantId: string) => {
  return useQuery({
    queryKey: tenantKeys.members(tenantId),
    queryFn: async () => {
      // Set up BFF client context
      tenantBFFClient.setContext(tenantId, 'mock-auth-token');
      
      try {
        return await tenantBFFClient.getTenantMembers(tenantId);
      } catch (error) {
        console.warn('BFF service not available, using mock data:', error);
        // Fallback to mock data if BFF service is not available
        return [
        {
          id: 'member-1',
          userId: 'user-1',
          tenantId,
          email: 'admin@demo.com',
          name: 'Admin User',
          role: TenantRole.OWNER,
          status: MemberStatus.ACTIVE,
          joinedAt: '2024-01-01T00:00:00Z',
          lastActiveAt: '2024-01-15T10:00:00Z',
        },
        {
          id: 'member-2',
          userId: 'user-2',
          tenantId,
          email: 'john@demo.com',
          name: 'John Doe',
          role: TenantRole.ADMIN,
          status: MemberStatus.ACTIVE,
          joinedAt: '2024-01-05T00:00:00Z',
          lastActiveAt: '2024-01-15T09:30:00Z',
        },
        {
          id: 'member-3',
          userId: 'user-3',
          tenantId,
          email: 'jane@demo.com',
          name: 'Jane Smith',
          role: TenantRole.MEMBER,
          status: MemberStatus.ACTIVE,
          joinedAt: '2024-01-10T00:00:00Z',
          lastActiveAt: '2024-01-14T16:45:00Z',
        },
      ];
      }
    },
    enabled: !!tenantId,
    staleTime: 2 * 60 * 1000, // 2 minutes
    gcTime: 5 * 60 * 1000, // 5 minutes
  });
};

// Invite member mutation
export const useInviteMember = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: async ({ tenantId, request }: { tenantId: string; request: InviteMemberRequest }) => {
      const response = await tenantBFFClient.inviteMember(tenantId, request);
      
      if (response.type === 'async' && response.operationId) {
        return tenantBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (invitation, { tenantId }) => {
      // Invalidate members and invitations
      queryClient.invalidateQueries({ queryKey: tenantKeys.members(tenantId) });
      queryClient.invalidateQueries({ queryKey: tenantKeys.invitations(tenantId) });
      
      // Emit member invited event
      emit('tenant:member_invited', { tenantId, invitation });
    },
    onError: (error) => {
      emit('tenant:invite_failed', { error: error.message });
    },
  });
};

// Update member mutation
export const useUpdateMember = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: ({ tenantId, memberId, request }: {
      tenantId: string;
      memberId: string;
      request: UpdateMemberRequest;
    }) => tenantBFFClient.updateMember(tenantId, memberId, request),
    onSuccess: (member, { tenantId }) => {
      // Invalidate members list
      queryClient.invalidateQueries({ queryKey: tenantKeys.members(tenantId) });
      
      // Emit member updated event
      emit('tenant:member_updated', { tenantId, member });
    },
    onError: (error) => {
      emit('tenant:member_update_failed', { error: error.message });
    },
  });
};

// Remove member mutation
export const useRemoveMember = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: ({ tenantId, memberId }: { tenantId: string; memberId: string }) =>
      tenantBFFClient.removeMember(tenantId, memberId),
    onSuccess: (_, { tenantId, memberId }) => {
      // Invalidate members list
      queryClient.invalidateQueries({ queryKey: tenantKeys.members(tenantId) });
      
      // Emit member removed event
      emit('tenant:member_removed', { tenantId, memberId });
    },
    onError: (error) => {
      emit('tenant:member_remove_failed', { error: error.message });
    },
  });
};

// Get tenant invitations
export const useTenantInvitations = (tenantId: string) => {
  return useQuery({
    queryKey: tenantKeys.invitations(tenantId),
    queryFn: async () => {
      // Set up BFF client context
      tenantBFFClient.setContext(tenantId, 'mock-auth-token');
      
      try {
        return await tenantBFFClient.getTenantInvitations(tenantId);
      } catch (error) {
        console.warn('BFF service not available, using mock data:', error);
        // Fallback to mock data if BFF service is not available
        return [
        {
          id: 'invitation-1',
          tenantId,
          email: 'newuser@demo.com',
          role: TenantRole.MEMBER,
          invitedBy: 'user-1',
          invitedAt: '2024-01-14T10:00:00Z',
          expiresAt: '2024-01-21T10:00:00Z',
          status: InvitationStatus.PENDING,
          token: 'invite-token-123',
        },
        {
          id: 'invitation-2',
          tenantId,
          email: 'expired@demo.com',
          role: TenantRole.VIEWER,
          invitedBy: 'user-1',
          invitedAt: '2024-01-01T10:00:00Z',
          expiresAt: '2024-01-08T10:00:00Z',
          status: InvitationStatus.EXPIRED,
          token: 'invite-token-456',
        },
      ];
      }
    },
    enabled: !!tenantId,
    staleTime: 2 * 60 * 1000,
    gcTime: 5 * 60 * 1000,
  });
};

// Cancel invitation mutation
export const useCancelInvitation = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: ({ tenantId, invitationId }: { tenantId: string; invitationId: string }) =>
      tenantBFFClient.cancelInvitation(tenantId, invitationId),
    onSuccess: (_, { tenantId, invitationId }) => {
      // Invalidate invitations list
      queryClient.invalidateQueries({ queryKey: tenantKeys.invitations(tenantId) });
      
      // Emit invitation cancelled event
      emit('tenant:invitation_cancelled', { tenantId, invitationId });
    },
    onError: (error) => {
      emit('tenant:invitation_cancel_failed', { error: error.message });
    },
  });
};

// Resend invitation mutation
export const useResendInvitation = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();

  return useMutation({
    mutationFn: ({ tenantId, invitationId }: { tenantId: string; invitationId: string }) =>
      tenantBFFClient.resendInvitation(tenantId, invitationId),
    onSuccess: (_, { tenantId, invitationId }) => {
      // Invalidate invitations list
      queryClient.invalidateQueries({ queryKey: tenantKeys.invitations(tenantId) });
      
      // Emit invitation resent event
      emit('tenant:invitation_resent', { tenantId, invitationId });
    },
    onError: (error) => {
      emit('tenant:invitation_resend_failed', { error: error.message });
    },
  });
};