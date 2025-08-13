import React, { createContext, useContext, useEffect, ReactNode } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useAuthStore } from '@adx-core/shared-context';
import { useEventBus } from '@adx-core/event-bus';
import { authBFFClient } from '../services';
import { storage } from '../utils';

interface AuthProviderProps {
  children: ReactNode;
}

interface AuthContextType {
  initialized: boolean;
}

const AuthContext = createContext<AuthContextType | null>(null);

export const AuthProvider: React.FC<AuthProviderProps> = ({ children }) => {
  const queryClient = useQueryClient();
  const { emit, subscribe } = useEventBus();
  const {
    login: setAuthState,
    logout: clearAuthState,
    setLoading,
    setError,
  } = useAuthStore();
  const [initialized, setInitialized] = React.useState(false);

  // Initialize auth state from storage
  useEffect(() => {
    const initializeAuth = async () => {
      setLoading(true);

      try {
        const token = storage.getToken();
        const user = storage.getUser();

        if (token && user) {
          // Validate token with BFF
          try {
            const validation = await authBFFClient.validateToken(token);
            
            if (validation.valid) {
              setAuthState(user, token);
              emit('auth:initialized', { user, timestamp: new Date().toISOString() });
            } else {
              // Token is invalid, clear storage
              storage.clearAuth();
              clearAuthState();
            }
          } catch (error) {
            console.error('Token validation failed:', error);
            // Try to refresh token
            const refreshToken = storage.getRefreshToken();
            if (refreshToken) {
              try {
                const refreshResponse = await authBFFClient.refreshToken(refreshToken);
                storage.setToken(refreshResponse.token);
                storage.setRefreshToken(refreshResponse.refreshToken);
                storage.setUser(refreshResponse.user);
                setAuthState(refreshResponse.user, refreshResponse.token);
                emit('auth:token_refreshed', { 
                  user: refreshResponse.user, 
                  timestamp: new Date().toISOString() 
                });
              } catch (refreshError) {
                console.error('Token refresh failed:', refreshError);
                storage.clearAuth();
                clearAuthState();
              }
            } else {
              storage.clearAuth();
              clearAuthState();
            }
          }
        }
      } catch (error) {
        console.error('Auth initialization failed:', error);
        setError('Failed to initialize authentication');
        storage.clearAuth();
        clearAuthState();
      } finally {
        setLoading(false);
        setInitialized(true);
      }
    };

    initializeAuth();
  }, [setAuthState, clearAuthState, setLoading, setError, emit]);

  // Set up token refresh interval
  useEffect(() => {
    const token = storage.getToken();
    const refreshToken = storage.getRefreshToken();

    if (token && refreshToken) {
      // Refresh token every 50 minutes (assuming 1-hour expiry)
      const refreshInterval = setInterval(async () => {
        try {
          const refreshResponse = await authBFFClient.refreshToken(refreshToken);
          storage.setToken(refreshResponse.token);
          storage.setRefreshToken(refreshResponse.refreshToken);
          storage.setUser(refreshResponse.user);
          setAuthState(refreshResponse.user, refreshResponse.token);
          emit('auth:token_refreshed', { 
            user: refreshResponse.user, 
            timestamp: new Date().toISOString() 
          });
        } catch (error) {
          console.error('Automatic token refresh failed:', error);
          // Clear auth state on refresh failure
          storage.clearAuth();
          clearAuthState();
          emit('auth:token_expired', { timestamp: new Date().toISOString() });
        }
      }, 50 * 60 * 1000); // 50 minutes

      return () => clearInterval(refreshInterval);
    }
  }, [setAuthState, clearAuthState, emit]);

  // Listen for auth events from other micro-frontends
  useEffect(() => {
    const unsubscribe = subscribe('auth:*', (event) => {
      switch (event.type) {
        case 'auth:logout_requested':
          storage.clearAuth();
          clearAuthState();
          queryClient.clear();
          break;
        case 'auth:token_expired':
          storage.clearAuth();
          clearAuthState();
          queryClient.clear();
          break;
      }
    });

    return unsubscribe;
  }, [subscribe, clearAuthState, queryClient]);

  // Handle browser tab visibility for token validation
  useEffect(() => {
    const handleVisibilityChange = async () => {
      if (!document.hidden && initialized) {
        const token = storage.getToken();
        if (token) {
          try {
            const validation = await authBFFClient.validateToken(token);
            if (!validation.valid) {
              storage.clearAuth();
              clearAuthState();
              emit('auth:token_expired', { timestamp: new Date().toISOString() });
            }
          } catch (error) {
            console.error('Token validation on visibility change failed:', error);
          }
        }
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    return () => document.removeEventListener('visibilitychange', handleVisibilityChange);
  }, [initialized, clearAuthState, emit]);

  return (
    <AuthContext.Provider value={{ initialized }}>
      {children}
    </AuthContext.Provider>
  );
};

export const useAuthContext = () => {
  const context = useContext(AuthContext);
  if (!context) {
    throw new Error('useAuthContext must be used within AuthProvider');
  }
  return context;
};