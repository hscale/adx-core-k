import React from 'react';
import { 
  ThemeProvider, 
  ThemeToggle, 
  ThemeAware, 
  useTheme, 
  useThemeAwareStyle,
  Button,
  Card,
  Alert,
  Badge,
  Spinner
} from '../index';

// Example component showing theme-aware styling
export function ThemeExample() {
  return (
    <ThemeProvider defaultTheme="system">
      <div className="min-h-screen bg-background text-text-primary p-8">
        <div className="max-w-4xl mx-auto space-y-8">
          {/* Header with theme controls */}
          <header className="flex items-center justify-between">
            <h1 className="text-3xl font-bold">ADX Core Theme System</h1>
            <div className="flex items-center space-x-4">
              <ThemeToggle variant="button" />
              <ThemeToggle variant="dropdown" />
              <ThemeToggle variant="switch" showLabel />
            </div>
          </header>

          {/* Theme information */}
          <ThemeInfo />

          {/* Component showcase */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <div className="card-header">
                <h2 className="card-title">Buttons</h2>
                <p className="card-description">Various button styles with theme awareness</p>
              </div>
              <div className="card-content space-y-4">
                <div className="flex flex-wrap gap-2">
                  <Button variant="primary">Primary</Button>
                  <Button variant="secondary">Secondary</Button>
                  <Button variant="outline">Outline</Button>
                  <Button variant="ghost">Ghost</Button>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button variant="success">Success</Button>
                  <Button variant="warning">Warning</Button>
                  <Button variant="error">Error</Button>
                </div>
                <div className="flex flex-wrap gap-2">
                  <Button size="sm">Small</Button>
                  <Button size="md">Medium</Button>
                  <Button size="lg">Large</Button>
                </div>
              </div>
            </Card>

            <Card>
              <div className="card-header">
                <h2 className="card-title">Alerts</h2>
                <p className="card-description">Contextual alerts with theme support</p>
              </div>
              <div className="card-content space-y-4">
                <Alert variant="info" title="Information">
                  This is an informational alert message.
                </Alert>
                <Alert variant="success" title="Success">
                  Operation completed successfully!
                </Alert>
                <Alert variant="warning" title="Warning">
                  Please review your settings.
                </Alert>
                <Alert variant="error" title="Error">
                  Something went wrong.
                </Alert>
              </div>
            </Card>

            <Card>
              <div className="card-header">
                <h2 className="card-title">Badges</h2>
                <p className="card-description">Status indicators and labels</p>
              </div>
              <div className="card-content">
                <div className="flex flex-wrap gap-2">
                  <Badge variant="primary">Primary</Badge>
                  <Badge variant="secondary">Secondary</Badge>
                  <Badge variant="success">Success</Badge>
                  <Badge variant="warning">Warning</Badge>
                  <Badge variant="error">Error</Badge>
                </div>
              </div>
            </Card>

            <Card>
              <div className="card-header">
                <h2 className="card-title">Loading States</h2>
                <p className="card-description">Spinners and loading indicators</p>
              </div>
              <div className="card-content">
                <div className="flex items-center space-x-4">
                  <Spinner size="sm" />
                  <Spinner size="md" />
                  <Spinner size="lg" />
                  <Button disabled>
                    <Spinner size="sm" className="mr-2" />
                    Loading...
                  </Button>
                </div>
              </div>
            </Card>
          </div>

          {/* Theme-aware component example */}
          <ThemeAwareExample />

          {/* Custom styled component */}
          <CustomThemedComponent />
        </div>
      </div>
    </ThemeProvider>
  );
}

// Component showing theme information
function ThemeInfo() {
  const { theme, resolvedTheme, systemTheme } = useTheme();
  
  return (
    <Card>
      <div className="card-header">
        <h2 className="card-title">Theme Information</h2>
        <p className="card-description">Current theme state and system preferences</p>
      </div>
      <div className="card-content">
        <div className="grid grid-cols-3 gap-4 text-sm">
          <div>
            <strong>Selected Theme:</strong>
            <br />
            <Badge variant="primary">{theme}</Badge>
          </div>
          <div>
            <strong>Resolved Theme:</strong>
            <br />
            <Badge variant="secondary">{resolvedTheme}</Badge>
          </div>
          <div>
            <strong>System Theme:</strong>
            <br />
            <Badge variant="neutral">{systemTheme}</Badge>
          </div>
        </div>
      </div>
    </Card>
  );
}

// Example of ThemeAware component usage
function ThemeAwareExample() {
  return (
    <Card>
      <div className="card-header">
        <h2 className="card-title">Theme-Aware Components</h2>
        <p className="card-description">Components that adapt to theme changes</p>
      </div>
      <div className="card-content space-y-4">
        <ThemeAware
          lightClassName="bg-yellow-100 text-yellow-800 border border-yellow-300"
          darkClassName="bg-yellow-900/20 text-yellow-300 border border-yellow-700"
          className="p-4 rounded-lg"
        >
          This component has different styles for light and dark themes.
        </ThemeAware>

        <ThemeAware
          as="button"
          lightClassName="bg-blue-500 hover:bg-blue-600 text-white"
          darkClassName="bg-blue-600 hover:bg-blue-700 text-white"
          className="px-4 py-2 rounded-md transition-colors"
        >
          Theme-aware button
        </ThemeAware>

        <ThemeAware>
          {({ isDark, isLight, resolvedTheme }) => (
            <div className="p-4 rounded-lg bg-surface-secondary">
              <p>Current theme: <strong>{resolvedTheme}</strong></p>
              <p>Is dark mode: <strong>{isDark ? 'Yes' : 'No'}</strong></p>
              <p>Is light mode: <strong>{isLight ? 'Yes' : 'No'}</strong></p>
            </div>
          )}
        </ThemeAware>
      </div>
    </Card>
  );
}

// Example of custom component with theme awareness
function CustomThemedComponent() {
  const { getThemeValue } = useThemeAwareStyle();
  
  const backgroundColor = getThemeValue('#f8fafc', '#1e293b');
  const textColor = getThemeValue('#334155', '#e2e8f0');
  const borderColor = getThemeValue('#e2e8f0', '#475569');
  
  return (
    <Card>
      <div className="card-header">
        <h2 className="card-title">Custom Themed Component</h2>
        <p className="card-description">Using theme values programmatically</p>
      </div>
      <div className="card-content">
        <div
          style={{
            backgroundColor,
            color: textColor,
            border: `2px solid ${borderColor}`,
            padding: '1rem',
            borderRadius: '0.5rem',
            transition: 'all 0.2s ease-in-out',
          }}
        >
          <p>This component uses programmatic theme values:</p>
          <ul className="mt-2 space-y-1 text-sm">
            <li>Background: {backgroundColor}</li>
            <li>Text: {textColor}</li>
            <li>Border: {borderColor}</li>
          </ul>
        </div>
      </div>
    </Card>
  );
}