import React from 'react';
import { Moon, Sun, Monitor } from 'lucide-react';
import { useTheme, type Theme } from './ThemeProvider';
import { Button } from './Button';

export interface ThemeToggleProps {
  variant?: 'button' | 'dropdown' | 'switch';
  size?: 'sm' | 'md' | 'lg';
  showLabel?: boolean;
  className?: string;
}

export function ThemeToggle({ 
  variant = 'button', 
  size = 'md', 
  showLabel = false,
  className = '' 
}: ThemeToggleProps) {
  const { theme, setTheme, toggleTheme } = useTheme();

  if (variant === 'button') {
    return (
      <Button
        variant="ghost"
        size={size}
        onClick={toggleTheme}
        className={`theme-toggle ${className}`}
        aria-label="Toggle theme"
      >
        <Sun className="h-4 w-4 rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0" />
        <Moon className="absolute h-4 w-4 rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100" />
        {showLabel && <span className="ml-2">Toggle theme</span>}
      </Button>
    );
  }

  if (variant === 'dropdown') {
    return (
      <ThemeDropdown 
        currentTheme={theme} 
        onThemeChange={setTheme} 
        size={size}
        className={className}
      />
    );
  }

  if (variant === 'switch') {
    return (
      <ThemeSwitch 
        currentTheme={theme} 
        onThemeChange={setTheme}
        showLabel={showLabel}
        className={className}
      />
    );
  }

  return null;
}

interface ThemeDropdownProps {
  currentTheme: Theme;
  onThemeChange: (theme: Theme) => void;
  size: 'sm' | 'md' | 'lg';
  className: string;
}

function ThemeDropdown({ currentTheme, onThemeChange, size, className }: ThemeDropdownProps) {
  const [isOpen, setIsOpen] = React.useState(false);

  const themes: Array<{ value: Theme; label: string; icon: React.ReactNode }> = [
    { value: 'light', label: 'Light', icon: <Sun className="h-4 w-4" /> },
    { value: 'dark', label: 'Dark', icon: <Moon className="h-4 w-4" /> },
    { value: 'system', label: 'System', icon: <Monitor className="h-4 w-4" /> },
  ];

  const currentThemeData = themes.find(t => t.value === currentTheme);

  return (
    <div className={`relative ${className}`}>
      <Button
        variant="ghost"
        size={size}
        onClick={() => setIsOpen(!isOpen)}
        className="theme-dropdown-trigger"
        aria-label="Select theme"
        aria-expanded={isOpen}
      >
        {currentThemeData?.icon}
        <span className="ml-2">{currentThemeData?.label}</span>
        <svg
          className={`ml-2 h-4 w-4 transition-transform ${isOpen ? 'rotate-180' : ''}`}
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
        </svg>
      </Button>

      {isOpen && (
        <>
          <div 
            className="fixed inset-0 z-10" 
            onClick={() => setIsOpen(false)}
            aria-hidden="true"
          />
          <div className="absolute right-0 mt-2 w-48 bg-surface border border-default rounded-md shadow-lg z-20">
            <div className="py-1">
              {themes.map((themeOption) => (
                <button
                  key={themeOption.value}
                  onClick={() => {
                    onThemeChange(themeOption.value);
                    setIsOpen(false);
                  }}
                  className={`
                    w-full px-4 py-2 text-left text-sm flex items-center hover:bg-surface-secondary
                    ${currentTheme === themeOption.value ? 'bg-surface-secondary text-primary' : 'text-primary'}
                  `}
                >
                  {themeOption.icon}
                  <span className="ml-3">{themeOption.label}</span>
                  {currentTheme === themeOption.value && (
                    <svg className="ml-auto h-4 w-4" fill="currentColor" viewBox="0 0 20 20">
                      <path fillRule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clipRule="evenodd" />
                    </svg>
                  )}
                </button>
              ))}
            </div>
          </div>
        </>
      )}
    </div>
  );
}

interface ThemeSwitchProps {
  currentTheme: Theme;
  onThemeChange: (theme: Theme) => void;
  showLabel: boolean;
  className: string;
}

function ThemeSwitch({ currentTheme, onThemeChange, showLabel, className }: ThemeSwitchProps) {
  return (
    <div className={`flex items-center space-x-3 ${className}`}>
      {showLabel && <span className="text-sm text-secondary">Theme</span>}
      <div className="flex items-center space-x-1 bg-surface-secondary rounded-lg p-1">
        <button
          onClick={() => onThemeChange('light')}
          className={`
            p-2 rounded-md transition-colors
            ${currentTheme === 'light' ? 'bg-surface text-primary shadow-sm' : 'text-tertiary hover:text-secondary'}
          `}
          aria-label="Light theme"
        >
          <Sun className="h-4 w-4" />
        </button>
        <button
          onClick={() => onThemeChange('system')}
          className={`
            p-2 rounded-md transition-colors
            ${currentTheme === 'system' ? 'bg-surface text-primary shadow-sm' : 'text-tertiary hover:text-secondary'}
          `}
          aria-label="System theme"
        >
          <Monitor className="h-4 w-4" />
        </button>
        <button
          onClick={() => onThemeChange('dark')}
          className={`
            p-2 rounded-md transition-colors
            ${currentTheme === 'dark' ? 'bg-surface text-primary shadow-sm' : 'text-tertiary hover:text-secondary'}
          `}
          aria-label="Dark theme"
        >
          <Moon className="h-4 w-4" />
        </button>
      </div>
    </div>
  );
}