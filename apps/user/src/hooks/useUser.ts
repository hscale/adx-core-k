import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
// import { useTenantContext } from '@adx-core/shared-context';
import { useEventBus } from '@adx-core/event-bus';
import { userBFFClient } from '../services';
import {
  UpdateUserRequest,
  UpdateUserProfileRequest,
  UpdateUserPreferencesRequest,
  UserSearchFilters,
  CreateUserRequest,
} from '../types';

// Query keys
export const userKeys = {
  all: ['users'] as const,
  user: (id: string) => [...userKeys.all, 'user', id] as const,
  profile: (id: string) => [...userKeys.all, 'profile', id] as const,
  settings: (id: string) => [...userKeys.all, 'settings', id] as const,
  activity: (id: string) => [...userKeys.all, 'activity', id] as const,
  search: (filters: UserSearchFilters, page?: number, pageSize?: number) => [...userKeys.all, 'search', filters, page, pageSize] as const,
};

// Get current user
export const useCurrentUser = () => {
  return useQuery({
    queryKey: userKeys.user('current'),
    queryFn: async () => {
      // Get current user ID from auth context or local storage
      const currentUserId = localStorage.getItem('current_user_id');
      if (!currentUserId) {
        throw new Error('No current user ID found');
      }
      return userBFFClient.getUser(currentUserId);
    },
    enabled: true,
    staleTime: 5 * 60 * 1000, // 5 minutes
  });
};

// Get user by ID
export const useUser = (userId: string) => {
  return useQuery({
    queryKey: userKeys.user(userId),
    queryFn: () => userBFFClient.getUser(userId),
    enabled: !!userId,
    staleTime: 5 * 60 * 1000,
  });
};

// Get user profile
export const useUserProfile = (userId: string) => {
  return useQuery({
    queryKey: userKeys.profile(userId),
    queryFn: () => userBFFClient.getUserProfile(userId),
    enabled: !!userId,
    staleTime: 5 * 60 * 1000,
  });
};

// Get user settings
export const useUserSettings = (userId: string) => {
  return useQuery({
    queryKey: userKeys.settings(userId),
    queryFn: () => userBFFClient.getUserSettings(userId),
    enabled: !!userId,
    staleTime: 2 * 60 * 1000, // 2 minutes
  });
};

// Get user activity
export const useUserActivity = (userId: string, limit = 50) => {
  return useQuery({
    queryKey: userKeys.activity(userId),
    queryFn: () => userBFFClient.getUserActivity(userId, limit),
    enabled: !!userId,
    staleTime: 1 * 60 * 1000, // 1 minute
  });
};

// Search users
export const useUserSearch = (filters: UserSearchFilters, page = 1, pageSize = 20) => {
  return useQuery({
    queryKey: userKeys.search(filters, page, pageSize),
    queryFn: () => userBFFClient.searchUsers(filters, page, pageSize),
    enabled: true,
    staleTime: 2 * 60 * 1000,
  });
};

// Update user mutation
export const useUpdateUser = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();
  
  return useMutation({
    mutationFn: async ({ userId, updates }: { userId: string; updates: UpdateUserRequest }) => {
      const response = await userBFFClient.updateUser(userId, updates);
      
      if (response.type === 'async' && response.operationId) {
        return userBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (data, variables) => {
      // Invalidate and refetch user data
      queryClient.invalidateQueries({ queryKey: userKeys.user(variables.userId) });
      queryClient.invalidateQueries({ queryKey: userKeys.all });
      
      // Emit user updated event
      emit('user:updated', {
        userId: variables.userId,
        updates: variables.updates,
        user: data,
      });
    },
  });
};

// Update user profile mutation
export const useUpdateUserProfile = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();
  
  return useMutation({
    mutationFn: async ({ userId, updates }: { userId: string; updates: UpdateUserProfileRequest }) => {
      const response = await userBFFClient.updateUserProfile(userId, updates);
      
      if (response.type === 'async' && response.operationId) {
        return userBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (data, variables) => {
      // Invalidate and refetch profile data
      queryClient.invalidateQueries({ queryKey: userKeys.profile(variables.userId) });
      
      // Emit profile updated event
      emit('user:profile_updated', {
        userId: variables.userId,
        updates: variables.updates,
        profile: data,
      });
    },
  });
};

// Update user preferences mutation
export const useUpdateUserPreferences = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();
  
  return useMutation({
    mutationFn: async ({ userId, updates }: { userId: string; updates: UpdateUserPreferencesRequest }) => {
      const response = await userBFFClient.updateUserPreferences(userId, updates);
      
      if (response.type === 'async' && response.operationId) {
        return userBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (data, variables) => {
      // Invalidate and refetch settings data
      queryClient.invalidateQueries({ queryKey: userKeys.settings(variables.userId) });
      
      // Emit preferences updated event
      emit('user:preferences_updated', {
        userId: variables.userId,
        updates: variables.updates,
        preferences: data,
      });
    },
  });
};

// Invite user mutation
export const useInviteUser = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();
  
  return useMutation({
    mutationFn: async (invitation: CreateUserRequest) => {
      const response = await userBFFClient.inviteUser(invitation);
      
      if (response.type === 'async' && response.operationId) {
        return userBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (data, variables) => {
      // Invalidate user lists
      queryClient.invalidateQueries({ queryKey: userKeys.all });
      
      // Emit user invited event
      emit('user:invited', {
        invitation: variables,
        result: data,
      });
    },
  });
};

// Deactivate user mutation
export const useDeactivateUser = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();
  
  return useMutation({
    mutationFn: async (userId: string) => {
      const response = await userBFFClient.deactivateUser(userId);
      
      if (response.type === 'async' && response.operationId) {
        return userBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (_, userId) => {
      // Invalidate user data
      queryClient.invalidateQueries({ queryKey: userKeys.user(userId) });
      queryClient.invalidateQueries({ queryKey: userKeys.all });
      
      // Emit user deactivated event
      emit('user:deactivated', { userId });
    },
  });
};

// Reactivate user mutation
export const useReactivateUser = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();
  
  return useMutation({
    mutationFn: async (userId: string) => {
      const response = await userBFFClient.reactivateUser(userId);
      
      if (response.type === 'async' && response.operationId) {
        return userBFFClient.pollWorkflowStatus(response.operationId);
      }
      
      return response.data;
    },
    onSuccess: (_, userId) => {
      // Invalidate user data
      queryClient.invalidateQueries({ queryKey: userKeys.user(userId) });
      queryClient.invalidateQueries({ queryKey: userKeys.all });
      
      // Emit user reactivated event
      emit('user:reactivated', { userId });
    },
  });
};