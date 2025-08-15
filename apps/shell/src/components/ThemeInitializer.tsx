import React, { useEffect } from 'react';
import { ThemeProvider, useTheme } from '@adx-core/design-system';
import { useThemeStore } from '@adx-core/shared-context';

interface ThemeInitializerProps {
  children: React.ReactNode;
}

export function ThemeInitializer({ children }: ThemeInitializerProps) {
  return (
    <ThemeProvider
      defaultTheme="system"
      storageKey="adx-theme"
      enableSystem={true}
      disableTransitionOnChange={false}
      attribute="class"
      themeValues={{ light: '', dark: 'dark' }}
    >
      <ThemeSync>
        {children}
      </ThemeSync>
    </ThemeProvider>
  );
}

function ThemeSync({ children }: { children: React.ReactNode }) {
  const { theme, resolvedTheme, setTheme } = useTheme();
  const themeStore = useThemeStore();

  // Sync theme provider with Zustand store
  useEffect(() => {
    themeStore.syncWithProvider(theme, resolvedTheme);
  }, [theme, resolvedTheme, themeStore]);

  // Listen for theme changes from store
  useEffect(() => {
    const unsubscribe = useThemeStore.subscribe((state) => {
      if (state.theme !== theme) {
        setTheme(state.theme);
      }
    });

    return unsubscribe;
  }, [theme, setTheme]);

  // Load user theme preferences on mount
  useEffect(() => {
    loadUserThemePreferences();
  }, []);

  return <>{children}</>;
}

async function loadUserThemePreferences() {
  try {
    const authToken = localStorage.getItem('auth-token');
    const tenantId = localStorage.getItem('current-tenant-id');
    
    if (!authToken || !tenantId) return;

    const response = await fetch('/api/v1/user/preferences', {
      headers: {
        'Authorization': `Bearer ${authToken}`,
        'X-Tenant-ID': tenantId,
      },
    });

    if (response.ok) {
      const preferences = await response.json();
      if (preferences.theme) {
        useThemeStore.getState().setTheme(preferences.theme);
      }
    }
  } catch (error) {
    console.warn('Failed to load user theme preferences:', error);
  }
}