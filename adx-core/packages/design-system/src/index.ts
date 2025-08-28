// Basic design system exports for ADX Core micro-frontends
export { default as Button } from './components/Button';
export { default as Input } from './components/Input';
export { default as Card } from './components/Card';
export { DesignSystemProvider, useTheme } from './providers/DesignSystemProvider';

// Re-export common utilities
export { default as clsx } from 'clsx';