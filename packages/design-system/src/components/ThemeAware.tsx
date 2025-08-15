import React from 'react';
import { useThemeAwareStyle } from './ThemeProvider';

export interface ThemeAwareProps {
  children: React.ReactNode | ((theme: { isDark: boolean; isLight: boolean; resolvedTheme: 'light' | 'dark' }) => React.ReactNode);
  lightClassName?: string;
  darkClassName?: string;
  className?: string;
  as?: keyof JSX.IntrinsicElements;
  style?: React.CSSProperties;
  lightStyle?: React.CSSProperties;
  darkStyle?: React.CSSProperties;
}

export function ThemeAware({
  children,
  lightClassName = '',
  darkClassName = '',
  className = '',
  as: Component = 'div',
  style = {},
  lightStyle = {},
  darkStyle = {},
  ...props
}: ThemeAwareProps) {
  const { resolvedTheme, isDark, isLight, getThemeValue } = useThemeAwareStyle();

  const themeClassName = getThemeValue(lightClassName, darkClassName);
  const themeStyle = getThemeValue(lightStyle, darkStyle);
  
  const finalClassName = [className, themeClassName].filter(Boolean).join(' ');
  const finalStyle = { ...style, ...themeStyle };

  const themeContext = { isDark, isLight, resolvedTheme };

  return (
    <Component className={finalClassName} style={finalStyle} {...props}>
      {typeof children === 'function' ? children(themeContext) : children}
    </Component>
  );
}

// Higher-order component for theme awareness
export function withThemeAware<P extends object>(
  WrappedComponent: React.ComponentType<P>
) {
  const ThemeAwareComponent = (props: P) => {
    const themeProps = useThemeAwareStyle();
    return <WrappedComponent {...props} {...themeProps} />;
  };

  ThemeAwareComponent.displayName = `withThemeAware(${WrappedComponent.displayName || WrappedComponent.name})`;
  
  return ThemeAwareComponent;
}

// Hook for conditional theme values
export function useThemeValue<T>(lightValue: T, darkValue: T): T {
  const { getThemeValue } = useThemeAwareStyle();
  return getThemeValue(lightValue, darkValue);
}

// Hook for theme-aware CSS variables
export function useThemeVariables() {
  const { resolvedTheme } = useThemeAwareStyle();
  
  return {
    '--theme-primary': resolvedTheme === 'dark' ? 'var(--color-primary-400)' : 'var(--color-primary-600)',
    '--theme-background': resolvedTheme === 'dark' ? 'var(--color-background)' : 'var(--color-background)',
    '--theme-surface': resolvedTheme === 'dark' ? 'var(--color-surface)' : 'var(--color-surface)',
    '--theme-text': resolvedTheme === 'dark' ? 'var(--color-text-primary)' : 'var(--color-text-primary)',
    '--theme-border': resolvedTheme === 'dark' ? 'var(--color-border)' : 'var(--color-border)',
  } as React.CSSProperties;
}