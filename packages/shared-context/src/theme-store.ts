import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import type { ThemeState } from './types';

interface ThemeActions {
  setTheme: (theme: 'light' | 'dark' | 'system') => void;
  toggleTheme: () => void;
}

export const useThemeStore = create<ThemeState & ThemeActions>()(
  persist(
    (set, get) => ({
      // State
      theme: 'system',
      resolvedTheme: getSystemTheme(),

      // Actions
      setTheme: (theme: 'light' | 'dark' | 'system') => {
        const resolvedTheme = theme === 'system' ? getSystemTheme() : theme;
        
        set({ theme, resolvedTheme });
        
        // Apply theme to document
        applyTheme(resolvedTheme);
        
        // Emit theme change event
        window.dispatchEvent(new CustomEvent('theme:changed', {
          detail: { theme, resolvedTheme }
        }));
      },

      toggleTheme: () => {
        const { theme } = get();
        const newTheme = theme === 'light' ? 'dark' : 'light';
        get().setTheme(newTheme);
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

function getSystemTheme(): 'light' | 'dark' {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

function applyTheme(theme: 'light' | 'dark') {
  if (typeof document === 'undefined') return;
  
  const root = document.documentElement;
  
  if (theme === 'dark') {
    root.classList.add('dark');
  } else {
    root.classList.remove('dark');
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
}