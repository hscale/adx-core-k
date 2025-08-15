import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { ThemeState } from './types';

export type Theme = 'light' | 'dark' | 'system';
export type ResolvedTheme = 'light' | 'dark';

interface ThemeActions {
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
  syncWithProvider: (theme: Theme, resolvedTheme: ResolvedTheme) => void;
}

export const useThemeStore = create<ThemeState & ThemeActions>()(
  persist(
    (set, get) => ({
      // State
      theme: 'system',
      resolvedTheme: getSystemTheme(),
      systemTheme: getSystemTheme(),

      // Actions
      setTheme: (theme: Theme) => {
        const systemTheme = getSystemTheme();
        const resolvedTheme = theme === 'system' ? systemTheme : theme;
        
        set({ theme, resolvedTheme, systemTheme });
        
        // Apply theme to document
        applyTheme(resolvedTheme);
        
        // Emit theme change event for micro-frontends
        window.dispatchEvent(new CustomEvent('theme:changed', {
          detail: { theme, resolvedTheme, systemTheme }
        }));

        // Sync user preferences if authenticated
        syncUserPreferences(theme);
      },

      toggleTheme: () => {
        const { theme, resolvedTheme } = get();
        if (theme === 'system') {
          const newTheme = resolvedTheme === 'dark' ? 'light' : 'dark';
          get().setTheme(newTheme);
        } else {
          const newTheme = theme === 'light' ? 'dark' : 'light';
          get().setTheme(newTheme);
        }
      },

      syncWithProvider: (theme: Theme, resolvedTheme: ResolvedTheme) => {
        set({ theme, resolvedTheme, systemTheme: getSystemTheme() });
      },
    }),
    {
      name: 'adx-theme-storage',
      partialize: (state) => ({
        theme: state.theme,
      }),
    }
  )
);

function getSystemTheme(): ResolvedTheme {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme: ResolvedTheme) {
  if (typeof document === 'undefined') return;
  
  const root = document.documentElement;
  
  // Remove existing theme classes
  root.classList.remove('light', 'dark');
  
  // Add new theme class
  root.classList.add(theme);
  
  // Set data attribute for CSS selectors
  root.setAttribute('data-theme', theme);
  
  // Update meta theme-color for mobile browsers
  updateMetaThemeColor(theme);
}

function updateMetaThemeColor(theme: ResolvedTheme) {
  if (typeof document === 'undefined') return;
  
  let metaThemeColor = document.querySelector('meta[name="theme-color"]');
  if (!metaThemeColor) {
    metaThemeColor = document.createElement('meta');
    metaThemeColor.setAttribute('name', 'theme-color');
    document.head.appendChild(metaThemeColor);
  }
  
  // Set theme color based on current theme
  const themeColor = theme === 'dark' ? '#0f172a' : '#ffffff';
  metaThemeColor.setAttribute('content', themeColor);
}

async function syncUserPreferences(theme: Theme) {
  try {
    // Only sync if user is authenticated
    const authToken = localStorage.getItem('auth-token');
    if (!authToken) return;

    // Get current tenant context
    const tenantId = localStorage.getItem('current-tenant-id');
    if (!tenantId) return;

    // Sync theme preference with backend
    await fetch('/api/v1/user/preferences', {
      method: 'PATCH',
      headers: {
        'Content-Type': 'application/json',
        'Authorization': `Bearer ${authToken}`,
        'X-Tenant-ID': tenantId,
      },
      body: JSON.stringify({
        theme: theme,
      }),
    });
  } catch (error) {
    console.warn('Failed to sync theme preferences:', error);
  }
}

// Listen for system theme changes
if (typeof window !== 'undefined') {
  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
  
  mediaQuery.addEventListener('change', () => {
    const { theme, setTheme } = useThemeStore.getState();
    if (theme === 'system') {
      setTheme('system'); // This will trigger a re-evaluation
    }
  });

  // Listen for theme changes from other micro-frontends
  window.addEventListener('theme:changed', (event: CustomEvent) => {
    const { theme, resolvedTheme } = event.detail;
    const { syncWithProvider } = useThemeStore.getState();
    syncWithProvider(theme, resolvedTheme);
  });
}

// Hook for micro-frontend theme synchronization
export function useThemeSync() {
  const { theme, resolvedTheme, setTheme } = useThemeStore();

  // Listen for theme changes from other micro-frontends
  if (typeof window !== 'undefined') {
    const handleThemeChange = (event: CustomEvent) => {
      const { theme: newTheme, resolvedTheme: newResolvedTheme } = event.detail;
      if (newTheme !== theme || newResolvedTheme !== resolvedTheme) {
        useThemeStore.getState().syncWithProvider(newTheme, newResolvedTheme);
      }
    };

    window.addEventListener('theme:changed', handleThemeChange as EventListener);
    
    // Cleanup function
    return () => {
      window.removeEventListener('theme:changed', handleThemeChange as EventListener);
    };
  }

  return { theme, resolvedTheme, setTheme };
}