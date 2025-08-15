import type { Theme, ResolvedTheme } from '../components/ThemeProvider';

export interface ThemePreferences {
  theme: Theme;
  accentColor?: string;
  fontSize?: 'sm' | 'md' | 'lg';
  reducedMotion?: boolean;
  highContrast?: boolean;
  lastUpdated?: number;
}

export class ThemePersistence {
  private storageKey: string;
  private syncKey: string;

  constructor(storageKey = 'adx-theme', syncKey = 'adx-theme-sync') {
    this.storageKey = storageKey;
    this.syncKey = syncKey;
  }

  // Save theme preferences to localStorage
  savePreferences(preferences: ThemePreferences): void {
    try {
      const data = {
        ...preferences,
        lastUpdated: Date.now(),
      };
      localStorage.setItem(this.storageKey, JSON.stringify(data));
      
      // Broadcast to other tabs/micro-frontends
      this.broadcastThemeChange(data);
    } catch (error) {
      console.warn('Failed to save theme preferences:', error);
    }
  }

  // Load theme preferences from localStorage
  loadPreferences(): ThemePreferences | null {
    try {
      const stored = localStorage.getItem(this.storageKey);
      if (!stored) return null;
      
      return JSON.parse(stored);
    } catch (error) {
      console.warn('Failed to load theme preferences:', error);
      return null;
    }
  }

  // Sync theme preferences with backend
  async syncWithBackend(preferences: ThemePreferences): Promise<void> {
    try {
      const authToken = localStorage.getItem('auth-token');
      const tenantId = localStorage.getItem('current-tenant-id');
      
      if (!authToken || !tenantId) return;

      await fetch('/api/v1/user/preferences/theme', {
        method: 'PUT',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${authToken}`,
          'X-Tenant-ID': tenantId,
        },
        body: JSON.stringify(preferences),
      });
    } catch (error) {
      console.warn('Failed to sync theme preferences with backend:', error);
    }
  }

  // Load theme preferences from backend
  async loadFromBackend(): Promise<ThemePreferences | null> {
    try {
      const authToken = localStorage.getItem('auth-token');
      const tenantId = localStorage.getItem('current-tenant-id');
      
      if (!authToken || !tenantId) return null;

      const response = await fetch('/api/v1/user/preferences/theme', {
        headers: {
          'Authorization': `Bearer ${authToken}`,
          'X-Tenant-ID': tenantId,
        },
      });

      if (response.ok) {
        return await response.json();
      }
    } catch (error) {
      console.warn('Failed to load theme preferences from backend:', error);
    }
    
    return null;
  }

  // Broadcast theme changes to other tabs/micro-frontends
  private broadcastThemeChange(preferences: ThemePreferences): void {
    try {
      localStorage.setItem(this.syncKey, JSON.stringify({
        ...preferences,
        timestamp: Date.now(),
      }));
      
      // Remove sync key after broadcasting
      setTimeout(() => {
        localStorage.removeItem(this.syncKey);
      }, 100);
    } catch (error) {
      console.warn('Failed to broadcast theme change:', error);
    }
  }

  // Listen for theme changes from other tabs/micro-frontends
  onThemeChange(callback: (preferences: ThemePreferences) => void): () => void {
    const handleStorageChange = (event: StorageEvent) => {
      if (event.key === this.syncKey && event.newValue) {
        try {
          const preferences = JSON.parse(event.newValue);
          callback(preferences);
        } catch (error) {
          console.warn('Failed to parse theme change event:', error);
        }
      }
    };

    window.addEventListener('storage', handleStorageChange);
    
    return () => {
      window.removeEventListener('storage', handleStorageChange);
    };
  }

  // Apply theme to document
  applyTheme(theme: ResolvedTheme, preferences?: ThemePreferences): void {
    const root = document.documentElement;
    
    // Apply theme class
    root.classList.remove('light', 'dark');
    root.classList.add(theme);
    
    // Apply data attribute
    root.setAttribute('data-theme', theme);
    
    // Apply additional preferences
    if (preferences) {
      if (preferences.fontSize) {
        root.setAttribute('data-font-size', preferences.fontSize);
      }
      
      if (preferences.reducedMotion) {
        root.setAttribute('data-reduced-motion', 'true');
      } else {
        root.removeAttribute('data-reduced-motion');
      }
      
      if (preferences.highContrast) {
        root.setAttribute('data-high-contrast', 'true');
      } else {
        root.removeAttribute('data-high-contrast');
      }
      
      if (preferences.accentColor) {
        root.style.setProperty('--accent-color', preferences.accentColor);
      }
    }
    
    // Update meta theme-color for mobile browsers
    this.updateMetaThemeColor(theme);
  }

  // Update meta theme-color for mobile browsers
  private updateMetaThemeColor(theme: ResolvedTheme): void {
    let metaThemeColor = document.querySelector('meta[name="theme-color"]');
    if (!metaThemeColor) {
      metaThemeColor = document.createElement('meta');
      metaThemeColor.setAttribute('name', 'theme-color');
      document.head.appendChild(metaThemeColor);
    }
    
    const themeColor = theme === 'dark' ? '#0f172a' : '#ffffff';
    metaThemeColor.setAttribute('content', themeColor);
  }

  // Get system theme preference
  getSystemTheme(): ResolvedTheme {
    if (typeof window === 'undefined') return 'light';
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
  }

  // Check if user prefers reduced motion
  prefersReducedMotion(): boolean {
    if (typeof window === 'undefined') return false;
    return window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  }

  // Check if user prefers high contrast
  prefersHighContrast(): boolean {
    if (typeof window === 'undefined') return false;
    return window.matchMedia('(prefers-contrast: high)').matches;
  }

  // Initialize theme with system preferences
  initializeTheme(): ThemePreferences {
    const stored = this.loadPreferences();
    const systemTheme = this.getSystemTheme();
    const reducedMotion = this.prefersReducedMotion();
    const highContrast = this.prefersHighContrast();
    
    const preferences: ThemePreferences = {
      theme: stored?.theme || 'system',
      fontSize: stored?.fontSize || 'md',
      reducedMotion: stored?.reducedMotion ?? reducedMotion,
      highContrast: stored?.highContrast ?? highContrast,
      accentColor: stored?.accentColor,
    };
    
    const resolvedTheme = preferences.theme === 'system' ? systemTheme : preferences.theme;
    this.applyTheme(resolvedTheme, preferences);
    
    return preferences;
  }
}

// Global theme persistence instance
export const themePersistence = new ThemePersistence();

// Hook for React components
export function useThemePersistence() {
  return themePersistence;
}