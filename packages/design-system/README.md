# ADX Core Design System

A comprehensive theming system for ADX Core micro-frontends with support for light/dark modes, accessibility features, and cross-micro-frontend synchronization.

## Features

- 🎨 **Comprehensive Theming**: Light, dark, and system theme support
- 🔄 **Cross-Micro-Frontend Sync**: Theme changes propagate across all micro-frontends
- ♿ **Accessibility First**: WCAG compliant colors and high contrast support
- 🎯 **CSS Custom Properties**: Dynamic theming with CSS variables
- 📱 **Responsive Design**: Mobile-first approach with Tailwind CSS
- 🔧 **Developer Experience**: TypeScript support and intuitive APIs

## Installation

```bash
npm install @adx-core/design-system
```

## Quick Start

### 1. Wrap your app with ThemeProvider

```tsx
import { ThemeProvider } from '@adx-core/design-system';

function App() {
  return (
    <ThemeProvider defaultTheme="system">
      <YourApp />
    </ThemeProvider>
  );
}
```

### 2. Add theme toggle components

```tsx
import { ThemeToggle } from '@adx-core/design-system';

function Navigation() {
  return (
    <nav>
      {/* Simple toggle button */}
      <ThemeToggle variant="button" />
      
      {/* Dropdown with all options */}
      <ThemeToggle variant="dropdown" />
      
      {/* Switch-style toggle */}
      <ThemeToggle variant="switch" showLabel />
    </nav>
  );
}
```

### 3. Use theme-aware components

```tsx
import { useTheme, ThemeAware, Button, Card } from '@adx-core/design-system';

function MyComponent() {
  const { theme, resolvedTheme } = useTheme();
  
  return (
    <Card>
      <div className="card-header">
        <h2 className="card-title">Current Theme: {resolvedTheme}</h2>
      </div>
      <div className="card-content">
        <ThemeAware
          lightClassName="bg-yellow-100 text-yellow-800"
          darkClassName="bg-yellow-900/20 text-yellow-300"
          className="p-4 rounded"
        >
          This content adapts to the theme!
        </ThemeAware>
        
        <Button variant="primary">Themed Button</Button>
      </div>
    </Card>
  );
}
```

## Theme Configuration

### CSS Custom Properties

The design system uses CSS custom properties for dynamic theming:

```css
:root {
  /* Primary Colors */
  --color-primary-50: 239 246 255;
  --color-primary-500: 59 130 246;
  --color-primary-600: 37 99 235;
  
  /* Background Colors */
  --color-background: 255 255 255;
  --color-surface: 255 255 255;
  
  /* Text Colors */
  --color-text-primary: 17 24 39;
  --color-text-secondary: 75 85 99;
  
  /* Spacing */
  --spacing-sm: 0.5rem;
  --spacing-md: 1rem;
  
  /* Transitions */
  --transition-fast: 150ms ease-in-out;
}

.dark {
  --color-background: 3 7 18;
  --color-surface: 15 23 42;
  --color-text-primary: 248 250 252;
  --color-text-secondary: 203 213 225;
}
```

### Tailwind CSS Integration

The design system extends Tailwind CSS with theme-aware utilities:

```tsx
// Use theme-aware classes
<div className="bg-surface text-text-primary border-border">
  Content that adapts to theme
</div>

// Use color scales
<div className="bg-primary-500 text-white">
  Primary colored content
</div>
```

## Components

### Buttons

```tsx
import { Button } from '@adx-core/design-system';

<Button variant="primary" size="md">Primary Button</Button>
<Button variant="secondary" size="sm">Secondary Button</Button>
<Button variant="outline" size="lg">Outline Button</Button>
<Button variant="ghost">Ghost Button</Button>
<Button variant="success">Success Button</Button>
<Button variant="warning">Warning Button</Button>
<Button variant="error">Error Button</Button>
```

### Cards

```tsx
import { Card } from '@adx-core/design-system';

<Card>
  <div className="card-header">
    <h3 className="card-title">Card Title</h3>
    <p className="card-description">Card description</p>
  </div>
  <div className="card-content">
    Card content goes here
  </div>
  <div className="card-footer">
    <Button>Action</Button>
  </div>
</Card>
```

### Alerts

```tsx
import { Alert } from '@adx-core/design-system';

<Alert variant="info" title="Information">
  This is an informational message.
</Alert>

<Alert variant="success" title="Success" onClose={() => {}}>
  Operation completed successfully!
</Alert>

<Alert variant="warning" title="Warning">
  Please review your settings.
</Alert>

<Alert variant="error" title="Error">
  Something went wrong.
</Alert>
```

### Badges

```tsx
import { Badge } from '@adx-core/design-system';

<Badge variant="primary">Primary</Badge>
<Badge variant="success" size="lg">Success</Badge>
<Badge variant="warning" size="sm">Warning</Badge>
```

### Loading States

```tsx
import { Spinner } from '@adx-core/design-system';

<Spinner size="sm" />
<Spinner size="md" />
<Spinner size="lg" />

<Button disabled>
  <Spinner size="sm" className="mr-2" />
  Loading...
</Button>
```

## Advanced Usage

### Theme-Aware Components

```tsx
import { ThemeAware, useThemeAwareStyle } from '@adx-core/design-system';

// Using ThemeAware wrapper
<ThemeAware
  lightClassName="bg-blue-100 text-blue-800"
  darkClassName="bg-blue-900/20 text-blue-300"
  className="p-4 rounded"
>
  Theme-aware content
</ThemeAware>

// Using render prop pattern
<ThemeAware>
  {({ isDark, isLight, resolvedTheme }) => (
    <div className={isDark ? 'dark-specific-class' : 'light-specific-class'}>
      Current theme: {resolvedTheme}
    </div>
  )}
</ThemeAware>

// Using hook for programmatic access
function CustomComponent() {
  const { getThemeValue, isDark } = useThemeAwareStyle();
  
  const backgroundColor = getThemeValue('#ffffff', '#1a1a1a');
  const textColor = getThemeValue('#000000', '#ffffff');
  
  return (
    <div style={{ backgroundColor, color: textColor }}>
      {isDark ? 'Dark mode content' : 'Light mode content'}
    </div>
  );
}
```

### Cross-Micro-Frontend Synchronization

Theme changes automatically synchronize across all micro-frontends:

```tsx
// In any micro-frontend
import { useTheme } from '@adx-core/design-system';

function MyMicroFrontend() {
  const { theme, setTheme } = useTheme();
  
  // This change will propagate to all other micro-frontends
  const handleThemeChange = () => {
    setTheme(theme === 'light' ? 'dark' : 'light');
  };
  
  return (
    <button onClick={handleThemeChange}>
      Switch to {theme === 'light' ? 'dark' : 'light'} mode
    </button>
  );
}
```

### Theme Persistence

Themes are automatically persisted to localStorage and synchronized with user preferences:

```tsx
import { useThemePersistence } from '@adx-core/design-system';

function ThemeSettings() {
  const persistence = useThemePersistence();
  
  const handleSavePreferences = async () => {
    const preferences = {
      theme: 'dark',
      fontSize: 'lg',
      reducedMotion: false,
      highContrast: false,
    };
    
    // Save locally
    persistence.savePreferences(preferences);
    
    // Sync with backend
    await persistence.syncWithBackend(preferences);
  };
  
  return (
    <button onClick={handleSavePreferences}>
      Save Theme Preferences
    </button>
  );
}
```

## Accessibility

The design system includes comprehensive accessibility features:

### High Contrast Support

```css
@media (prefers-contrast: high) {
  :root {
    --color-border: 0 0 0;
    --color-text-secondary: 0 0 0;
  }
  
  .dark {
    --color-border: 255 255 255;
    --color-text-secondary: 255 255 255;
  }
}
```

### Reduced Motion Support

```css
@media (prefers-reduced-motion: reduce) {
  :root {
    --transition-fast: 0ms;
    --transition-normal: 0ms;
    --transition-slow: 0ms;
  }
}
```

### WCAG Compliant Colors

All color combinations meet WCAG AA standards for contrast ratios.

## Micro-Frontend Integration

### Shell Application Setup

```tsx
// apps/shell/src/App.tsx
import { ThemeProvider } from '@adx-core/design-system';
import { ThemeInitializer } from './components/ThemeInitializer';

function App() {
  return (
    <ThemeInitializer>
      <div className="min-h-screen bg-background text-text-primary">
        {/* Your app content */}
      </div>
    </ThemeInitializer>
  );
}
```

### Individual Micro-Frontend Setup

```tsx
// apps/auth/src/App.tsx
import { useTheme } from '@adx-core/design-system';

function AuthApp() {
  const { resolvedTheme } = useTheme();
  
  // Theme is automatically synchronized from Shell
  return (
    <div className={`auth-app theme-${resolvedTheme}`}>
      {/* Auth-specific content */}
    </div>
  );
}
```

## Development

### Building the Design System

```bash
cd packages/design-system
npm run build
```

### Running Examples

```bash
npm run dev
```

### Testing

```bash
npm run test
```

## Contributing

1. Follow the existing code style and patterns
2. Ensure all components are theme-aware
3. Test across light and dark themes
4. Verify accessibility compliance
5. Update documentation for new components

## License

MIT License - see LICENSE file for details.