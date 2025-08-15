import React, { createContext, useContext, useEffect, useState } from 'react';

export type Theme = 'light' | 'dark' | 'system';
export type ResolvedTheme = 'light' | 'dark';

export interface ThemeConfig {
  theme: Theme;
  resolvedTheme: ResolvedTheme;
  systemTheme: ResolvedTheme;
  setTheme: (theme: Theme) => void;
  toggleTheme: () => void;
}

const ThemeContext = createContext<ThemeConfig | undefined>(undefined);

export interface ThemeProviderProps {
  children: React.ReactNode;
  defaultTheme?: Theme;
  storageKey?: string;
  enableSystem?: boolean;
  disableTransitionOnChange?: boolean;
  attribute?: string;
  themeValues?: {
    light: string;
    dark: string;
  };
}

export function ThemeProvider({
  children,
  defaultTheme = 'system',
  storageKey = 'adx-theme',
  enableSystem = true,
  disableTransitionOnChange = false,
  attribute = 'class',
  themeValues = { light: '', dark: 'dark' },
}: ThemeProviderProps) {
  const [theme, setThemeState] = useState<Theme>(() => {
    if (typeof window === 'undefined') return defaultTheme;
    
    try {
      const stored = localStorage.getItem(storageKey);
      return (stored as Theme) || defaultTheme;
    } catch {
      return defaultTheme;
    }
  });

  const [systemTheme, setSystemTheme] = useState<ResolvedTheme>(() => {
    if (typeof window === 'undefined') return 'light';
    return getSystemTheme();
  });

  const resolvedTheme: ResolvedTheme = theme === 'system' ? systemTheme : theme;

  // Listen for system theme changes
  useEffect(() => {
    if (!enableSystem) return;

    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleChange = () => {
      const newSystemTheme = getSystemTheme();
      setSystemTheme(newSystemTheme);
    };

    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [enableSystem]);

  // Apply theme to DOM
  useEffect(() => {
    const root = document.documentElement;
    
    // Disable transitions temporarily if requested
    if (disableTransitionOnChange) {
      const css = document.createElement('style');
      css.appendChild(
        document.createTextNode(
          `*,*::before,*::after{-webkit-transition:none!important;-moz-transition:none!important;-o-transition:none!important;-ms-transition:none!important;transition:none!important}`
        )
      );
      document.head.appendChild(css);

      // Re-enable transitions after a frame
      requestAnimationFrame(() => {
        requestAnimationFrame(() => {
          document.head.removeChild(css);
        });
      });
    }

    if (attribute === 'class') {
      root.classList.remove(themeValues.light, themeValues.dark);
      if (themeValues[resolvedTheme]) {
        root.classList.add(themeValues[resolvedTheme]);
      }
    } else {
      root.setAttribute(attribute, themeValues[resolvedTheme] || resolvedTheme);
    }

    // Emit theme change event for micro-frontends
    window.dispatchEvent(
      new CustomEvent('theme:changed', {
        detail: {
          theme,
          resolvedTheme,
          systemTheme,
        },
      })
    );
  }, [resolvedTheme, theme, systemTheme, attribute, themeValues, disableTransitionOnChange]);

  const setTheme = (newTheme: Theme) => {
    try {
      localStorage.setItem(storageKey, newTheme);
    } catch {
      // Ignore localStorage errors
    }
    setThemeState(newTheme);
  };

  const toggleTheme = () => {
    if (theme === 'system') {
      setTheme(systemTheme === 'dark' ? 'light' : 'dark');
    } else {
      setTheme(theme === 'light' ? 'dark' : 'light');
    }
  };

  const contextValue: ThemeConfig = {
    theme,
    resolvedTheme,
    systemTheme,
    setTheme,
    toggleTheme,
  };

  return (
    <ThemeContext.Provider value={contextValue}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const context = useContext(ThemeContext);
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider');
  }
  return context;
}

// Utility functions
function getSystemTheme(): ResolvedTheme {
  if (typeof window === 'undefined') return 'light';
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
}

// Hook for theme-aware styling
export function useThemeAwareStyle() {
  const { resolvedTheme } = useTheme();
  
  const getThemeValue = React.useCallback(function<T>(lightValue: T, darkValue: T): T {
    return resolvedTheme === 'dark' ? darkValue : lightValue;
  }, [resolvedTheme]);
  
  return {
    resolvedTheme,
    isDark: resolvedTheme === 'dark',
    isLight: resolvedTheme === 'light',
    getThemeValue,
  };
}

// Hook for listening to theme changes across micro-frontends
export function useThemeListener(callback: (themeData: any) => void) {
  useEffect(() => {
    const handleThemeChange = (event: CustomEvent) => {
      callback(event.detail);
    };

    window.addEventListener('theme:changed', handleThemeChange as EventListener);
    return () => {
      window.removeEventListener('theme:changed', handleThemeChange as EventListener);
    };
  }, [callback]);
}