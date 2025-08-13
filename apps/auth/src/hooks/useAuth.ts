import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { useAuthStore } from '@adx-core/shared-context';
import { useEventBus } from '@adx-core/event-bus';
import { authBFFClient } from '../services';
import { storage, extractWorkflowData } from '../utils';
import type {
  LoginRequest,
  RegisterRequest,
  ForgotPasswordRequest,
  ResetPasswordRequest,
  MFASetupRequest,
  MFAVerifyRequest,
  SSOLoginRequest,
  AuthResponse,
} from '../types';

export const useAuth = () => {
  const queryClient = useQueryClient();
  const { emit } = useEventBus();
  const {
    user,
    token,
    isAuthenticated,
    isLoading,
    error,
    login: setAuthState,
    logout: clearAuthState,
    setLoading,
    setError,
    clearError,
  } = useAuthStore();

  // Login mutation
  const loginMutation = useMutation({
    mutationFn: async (request: LoginRequest): Promise<AuthResponse> => {
      setLoading(true);
      clearError();

      const response = await authBFFClient.login(request);

      if (response.type === 'async' && response.operationId) {
        // Poll for completion
        return authBFFClient.pollWorkflowStatus(response.operationId);
      } else {
        return extractWorkflowData(response);
      }
    },
    onSuccess: (data: AuthResponse) => {
      // Store tokens
      storage.setToken(data.token);
      storage.setRefreshToken(data.refreshToken);
      storage.setUser(data.user);

      // Update auth state
      setAuthState(data.user, data.token);

      // Emit login event
      emit('auth:login', {
        user: data.user,
        timestamp: new Date().toISOString(),
      });

      // Invalidate queries
      queryClient.invalidateQueries({ queryKey: ['auth'] });
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // Register mutation
  const registerMutation = useMutation({
    mutationFn: async (request: RegisterRequest): Promise<AuthResponse> => {
      setLoading(true);
      clearError();

      const response = await authBFFClient.register(request);

      if (response.type === 'async' && response.operationId) {
        // Poll for completion
        return authBFFClient.pollWorkflowStatus(response.operationId);
      } else {
        return extractWorkflowData(response);
      }
    },
    onSuccess: (data: AuthResponse) => {
      // Store tokens
      storage.setToken(data.token);
      storage.setRefreshToken(data.refreshToken);
      storage.setUser(data.user);

      // Update auth state
      setAuthState(data.user, data.token);

      // Emit registration event
      emit('auth:register', {
        user: data.user,
        timestamp: new Date().toISOString(),
      });

      // Invalidate queries
      queryClient.invalidateQueries({ queryKey: ['auth'] });
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // Forgot password mutation
  const forgotPasswordMutation = useMutation({
    mutationFn: async (request: ForgotPasswordRequest) => {
      const response = await authBFFClient.forgotPassword(request);
      return extractWorkflowData(response);
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // Reset password mutation
  const resetPasswordMutation = useMutation({
    mutationFn: async (request: ResetPasswordRequest) => {
      const response = await authBFFClient.resetPassword(request);
      return extractWorkflowData(response);
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // MFA setup mutation
  const mfaSetupMutation = useMutation({
    mutationFn: async () => {
      const response = await authBFFClient.setupMFA();
      return extractWorkflowData(response);
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // MFA confirm mutation
  const mfaConfirmMutation = useMutation({
    mutationFn: async (request: MFASetupRequest) => {
      const response = await authBFFClient.confirmMFASetup(request);
      return extractWorkflowData(response);
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // MFA verify mutation
  const mfaVerifyMutation = useMutation({
    mutationFn: async (request: MFAVerifyRequest): Promise<AuthResponse> => {
      const response = await authBFFClient.verifyMFA(request);
      return extractWorkflowData(response);
    },
    onSuccess: (data: AuthResponse) => {
      // Store tokens
      storage.setToken(data.token);
      storage.setRefreshToken(data.refreshToken);
      storage.setUser(data.user);

      // Update auth state
      setAuthState(data.user, data.token);

      // Emit MFA verification event
      emit('auth:mfa_verified', {
        user: data.user,
        timestamp: new Date().toISOString(),
      });

      // Invalidate queries
      queryClient.invalidateQueries({ queryKey: ['auth'] });
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // SSO initiate mutation
  const ssoInitiateMutation = useMutation({
    mutationFn: async (request: SSOLoginRequest) => {
      const response = await authBFFClient.initiateSSO(request);
      return extractWorkflowData(response);
    },
    onSuccess: (data: { redirectUrl: string }) => {
      // Redirect to SSO provider
      window.location.href = data.redirectUrl;
    },
    onError: (error: Error) => {
      setError(error.message);
    },
  });

  // Logout function
  const logout = async () => {
    try {
      if (token) {
        await authBFFClient.logout(token);
      }
    } catch (error) {
      console.error('Logout error:', error);
    } finally {
      // Clear storage and state
      storage.clearAuth();
      clearAuthState();

      // Emit logout event
      emit('auth:logout', {
        timestamp: new Date().toISOString(),
      });

      // Clear all queries
      queryClient.clear();
    }
  };

  // Token validation query
  const { data: tokenValidation } = useQuery({
    queryKey: ['auth', 'validate', token],
    queryFn: async () => {
      if (!token) return { valid: false };
      return authBFFClient.validateToken(token);
    },
    enabled: !!token,
    staleTime: 5 * 60 * 1000, // 5 minutes
    retry: false,
  });

  return {
    // State
    user,
    token,
    isAuthenticated,
    isLoading: isLoading || loginMutation.isPending || registerMutation.isPending,
    error,

    // Actions
    login: loginMutation.mutateAsync,
    register: registerMutation.mutateAsync,
    forgotPassword: forgotPasswordMutation.mutateAsync,
    resetPassword: resetPasswordMutation.mutateAsync,
    setupMFA: mfaSetupMutation.mutateAsync,
    confirmMFA: mfaConfirmMutation.mutateAsync,
    verifyMFA: mfaVerifyMutation.mutateAsync,
    initiateSSO: ssoInitiateMutation.mutateAsync,
    logout,
    clearError,

    // Mutation states
    isLoginPending: loginMutation.isPending,
    isRegisterPending: registerMutation.isPending,
    isForgotPasswordPending: forgotPasswordMutation.isPending,
    isResetPasswordPending: resetPasswordMutation.isPending,
    isMFASetupPending: mfaSetupMutation.isPending,
    isMFAConfirmPending: mfaConfirmMutation.isPending,
    isMFAVerifyPending: mfaVerifyMutation.isPending,
    isSSOInitiatePending: ssoInitiateMutation.isPending,

    // Token validation
    isTokenValid: tokenValidation?.valid ?? false,
  };
};