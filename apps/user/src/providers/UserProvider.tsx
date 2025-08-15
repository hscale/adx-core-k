import React, { createContext, useContext, useEffect, useState } from 'react';
import { useEventBus } from '@adx-core/event-bus';
// import { useTenantContext } from '@adx-core/shared-context';
import { User, UserProfile, UserSettings } from '../types';
import { useCurrentUser, useUserProfile, useUserSettings } from '../hooks';

interface UserContextType {
  currentUser: User | null;
  currentUserProfile: UserProfile | null;
  currentUserSettings: UserSettings | null;
  isLoading: boolean;
  error: string | null;
  refreshUserData: () => void;
}

const UserContext = createContext<UserContextType | null>(null);

export const UserProvider: React.FC<{ children: React.ReactNode }> = ({ children }) => {
  const { emit, subscribe } = useEventBus();
  const [error, setError] = useState<string | null>(null);

  // Get current user data
  const {
    data: currentUser,
    isLoading: userLoading,
    error: userError,
    refetch: refetchUser,
  } = useCurrentUser();

  const {
    data: currentUserProfile,
    isLoading: profileLoading,
    error: profileError,
    refetch: refetchProfile,
  } = useUserProfile(currentUser?.id || '');

  const {
    data: currentUserSettings,
    isLoading: settingsLoading,
    error: settingsError,
    refetch: refetchSettings,
  } = useUserSettings(currentUser?.id || '');

  const isLoading = userLoading || profileLoading || settingsLoading;

  // Handle errors
  useEffect(() => {
    const errors = [userError, profileError, settingsError].filter(Boolean);
    if (errors.length > 0) {
      setError(errors[0]?.message || 'An error occurred');
    } else {
      setError(null);
    }
  }, [userError, profileError, settingsError]);

  // Refresh all user data
  const refreshUserData = () => {
    refetchUser();
    if (currentUser?.id) {
      refetchProfile();
      refetchSettings();
    }
  };

  // Subscribe to user-related events
  useEffect(() => {
    const unsubscribeUser = subscribe('user:*', (event) => {
      switch (event.type) {
        case 'user:updated':
          if (event.data.userId === currentUser?.id) {
            refetchUser();
          }
          break;
        case 'user:profile_updated':
          if (event.data.userId === currentUser?.id) {
            refetchProfile();
          }
          break;
        case 'user:preferences_updated':
          if (event.data.userId === currentUser?.id) {
            refetchSettings();
          }
          break;
      }
    });

    const unsubscribeTenant = subscribe('tenant:switched', () => {
      // Refresh user data when tenant switches
      refreshUserData();
    });

    return () => {
      unsubscribeUser();
      unsubscribeTenant();
    };
  }, [subscribe, currentUser?.id, refetchUser, refetchProfile, refetchSettings]);

  // Emit user context ready event when user data is loaded
  useEffect(() => {
    if (currentUser && !isLoading) {
      emit('user:context_ready', {
        user: currentUser,
        profile: currentUserProfile,
        settings: currentUserSettings,
      });
    }
  }, [currentUser, currentUserProfile, currentUserSettings, isLoading, emit]);

  const contextValue: UserContextType = {
    currentUser: currentUser || null,
    currentUserProfile: currentUserProfile || null,
    currentUserSettings: currentUserSettings || null,
    isLoading,
    error,
    refreshUserData,
  };

  return (
    <UserContext.Provider value={contextValue}>
      {children}
    </UserContext.Provider>
  );
};

export const useUserContext = () => {
  const context = useContext(UserContext);
  if (!context) {
    throw new Error('useUserContext must be used within UserProvider');
  }
  return context;
};

export default UserProvider;