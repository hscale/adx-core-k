# ADX Core Module Development Portal

Welcome to the ADX Core Module Development Portal! This comprehensive guide will help you create, test, and publish modules for the ADX Core platform.

## Table of Contents

1. [Getting Started](#getting-started)
2. [Module Architecture](#module-architecture)
3. [Development Environment Setup](#development-environment-setup)
4. [Creating Your First Module](#creating-your-first-module)
5. [Module SDK Reference](#module-sdk-reference)
6. [Temporal Workflow Integration](#temporal-workflow-integration)
7. [Frontend Components](#frontend-components)
8. [Testing Your Module](#testing-your-module)
9. [Security Guidelines](#security-guidelines)
10. [Publishing to Marketplace](#publishing-to-marketplace)
11. [Best Practices](#best-practices)
12. [Troubleshooting](#troubleshooting)

## Getting Started

### Prerequisites

- Node.js 18+ or Rust 1.70+
- ADX Core CLI tool
- Docker (for local development)
- Git

### Installation

```bash
# Install ADX Core CLI
npm install -g @adx-core/cli

# Verify installation
adx-cli --version
```

## Module Architecture

ADX Core modules follow a comprehensive architecture that supports:

- **Backend Services**: Temporal activities and workflows
- **Frontend Components**: React components with Module Federation
- **Database Extensions**: Migrations and schema extensions
- **API Endpoints**: RESTful APIs with automatic documentation
- **Cross-Platform Support**: Web, desktop, and mobile compatibility

### Module Structure

```
my-module/
├── manifest.json           # Module metadata and configuration
├── package.json           # Dependencies and scripts
├── README.md              # Module documentation
├── src/
│   ├── index.js          # Main entry point
│   ├── activities.js     # Temporal activities
│   ├── workflows.js      # Temporal workflows
│   ├── routes.js         # API endpoints
│   └── components.js     # Frontend components
├── tests/
│   ├── unit/             # Unit tests
│   ├── integration/      # Integration tests
│   └── e2e/              # End-to-end tests
├── docs/
│   ├── api.md            # API documentation
│   └── user-guide.md     # User guide
└── migrations/
    └── 001_initial.sql   # Database migrations
```

## Development Environment Setup

### 1. Create Development Workspace

```bash
# Create a new directory for your module
mkdir my-awesome-module
cd my-awesome-module

# Initialize the module
adx-cli module init --template=basic --name=my-awesome-module
```

### 2. Install Dependencies

```bash
# Install module dependencies
npm install

# Install development dependencies
npm install --save-dev jest eslint prettier
```

### 3. Start Development Server

```bash
# Start local development environment
adx-cli dev start

# This will start:
# - ADX Core platform (http://localhost:3000)
# - Module development server (http://localhost:3001)
# - Temporal UI (http://localhost:8088)
# - Database and Redis
```

## Creating Your First Module

### 1. Generate Module Template

```bash
adx-cli module create --template=basic --name=hello-world
cd hello-world
```

### 2. Update Module Manifest

Edit `manifest.json`:

```json
{
  "name": "hello-world",
  "version": "1.0.0",
  "description": "A simple hello world module",
  "author": {
    "name": "Your Name",
    "email": "your.email@example.com"
  },
  "license": "MIT",
  "adxCore": {
    "minVersion": "2.0.0"
  },
  "permissions": [
    "database:read",
    "api:external"
  ],
  "extensionPoints": {
    "backend": {
      "activities": ["./src/activities.js"],
      "endpoints": ["./src/routes.js"]
    },
    "frontend": {
      "components": ["./src/components.js"]
    }
  },
  "resources": {
    "memory": "256MB",
    "cpu": "0.5",
    "storage": "100MB"
  }
}
```

### 3. Implement Module Logic

**src/index.js** (Main Entry Point):
```javascript
const { ModuleBase } = require('@adx-core/module-sdk');

class HelloWorldModule extends ModuleBase {
  constructor() {
    super();
    this.name = 'hello-world';
    this.version = '1.0.0';
  }

  async initialize() {
    console.log(`Initializing ${this.name} v${this.version}`);
    // Module initialization logic
  }

  async activate() {
    console.log(`Activating ${this.name}`);
    // Module activation logic
  }

  async deactivate() {
    console.log(`Deactivating ${this.name}`);
    // Module deactivation logic
  }
}

module.exports = HelloWorldModule;
```

**src/activities.js** (Temporal Activities):
```javascript
const { Activity } = require('@adx-core/module-sdk');

class HelloWorldActivities {
  @Activity()
  async sayHello(input) {
    const { name = 'World' } = input;
    return {
      message: `Hello, ${name}!`,
      timestamp: new Date().toISOString()
    };
  }

  @Activity()
  async processGreeting(input) {
    const { greeting, language = 'en' } = input;
    
    const translations = {
      en: 'Hello',
      es: 'Hola',
      fr: 'Bonjour',
      de: 'Hallo'
    };

    return {
      originalGreeting: greeting,
      translatedGreeting: translations[language] || translations.en,
      language
    };
  }
}

module.exports = HelloWorldActivities;
```

**src/workflows.js** (Temporal Workflows):
```javascript
const { Workflow, Activity } = require('@adx-core/temporal-sdk');

@Workflow()
async function greetingWorkflow(input) {
  const { name, language } = input;

  // Step 1: Generate greeting
  const greeting = await Activity.execute('sayHello', { name });

  // Step 2: Process greeting with language
  const processedGreeting = await Activity.execute('processGreeting', {
    greeting: greeting.message,
    language
  });

  return {
    finalMessage: processedGreeting.translatedGreeting + ', ' + name + '!',
    processedAt: new Date().toISOString()
  };
}

module.exports = { greetingWorkflow };
```

**src/routes.js** (API Endpoints):
```javascript
const express = require('express');
const router = express.Router();

router.get('/health', (req, res) => {
  res.json({
    status: 'healthy',
    module: 'hello-world',
    version: '1.0.0'
  });
});

router.post('/greet', async (req, res) => {
  try {
    const { name, language = 'en' } = req.body;
    
    // Initiate greeting workflow
    const workflowResult = await req.temporalClient.startWorkflow(
      'greetingWorkflow',
      { name, language }
    );

    res.json({
      success: true,
      workflowId: workflowResult.workflowId,
      message: workflowResult.result?.finalMessage
    });
  } catch (error) {
    res.status(500).json({
      success: false,
      error: error.message
    });
  }
});

module.exports = router;
```

**src/components.js** (Frontend Components):
```javascript
import React, { useState } from 'react';
import { ModuleComponent, useModuleContext } from '@adx-core/module-sdk';

export const HelloWorldDashboard = () => {
  const { moduleConfig, tenantContext } = useModuleContext();
  const [name, setName] = useState('');
  const [language, setLanguage] = useState('en');
  const [greeting, setGreeting] = useState('');
  const [loading, setLoading] = useState(false);

  const handleGreet = async () => {
    setLoading(true);
    try {
      const response = await fetch('/api/modules/hello-world/greet', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Authorization': `Bearer ${tenantContext.authToken}`,
        },
        body: JSON.stringify({ name, language }),
      });

      const result = await response.json();
      if (result.success) {
        setGreeting(result.message);
      }
    } catch (error) {
      console.error('Greeting failed:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <ModuleComponent>
      <div className="hello-world-dashboard">
        <h1>Hello World Module</h1>
        <div className="greeting-form">
          <input
            type="text"
            placeholder="Enter your name"
            value={name}
            onChange={(e) => setName(e.target.value)}
          />
          <select value={language} onChange={(e) => setLanguage(e.target.value)}>
            <option value="en">English</option>
            <option value="es">Spanish</option>
            <option value="fr">French</option>
            <option value="de">German</option>
          </select>
          <button onClick={handleGreet} disabled={loading || !name}>
            {loading ? 'Greeting...' : 'Say Hello'}
          </button>
        </div>
        {greeting && (
          <div className="greeting-result">
            <h2>{greeting}</h2>
          </div>
        )}
      </div>
    </ModuleComponent>
  );
};
```

## Module SDK Reference

### Core Classes

#### ModuleBase
Base class for all modules.

```javascript
class MyModule extends ModuleBase {
  constructor() {
    super();
    this.name = 'my-module';
    this.version = '1.0.0';
  }

  async initialize() {
    // Module initialization
  }

  async activate() {
    // Module activation
  }

  async deactivate() {
    // Module deactivation
  }

  async cleanup() {
    // Module cleanup
  }
}
```

#### Activity Decorator
Marks methods as Temporal activities.

```javascript
class MyActivities {
  @Activity({
    startToCloseTimeout: '1m',
    retryPolicy: {
      maximumAttempts: 3,
      initialInterval: '1s',
      maximumInterval: '10s'
    }
  })
  async myActivity(input) {
    // Activity implementation
    return result;
  }
}
```

#### Workflow Decorator
Marks functions as Temporal workflows.

```javascript
@Workflow({
  taskQueue: 'my-module-queue',
  workflowExecutionTimeout: '10m'
})
async function myWorkflow(input) {
  // Workflow implementation
  const result = await Activity.execute('myActivity', input);
  return result;
}
```

### React Components

#### ModuleComponent
Wrapper component for module UI.

```javascript
import { ModuleComponent } from '@adx-core/module-sdk';

export const MyComponent = () => {
  return (
    <ModuleComponent>
      <div>Module content here</div>
    </ModuleComponent>
  );
};
```

#### useModuleContext Hook
Access module and tenant context.

```javascript
import { useModuleContext } from '@adx-core/module-sdk';

export const MyComponent = () => {
  const { moduleConfig, tenantContext, userContext } = useModuleContext();
  
  return (
    <div>
      <h1>Welcome to {tenantContext.tenantName}</h1>
      <p>Module: {moduleConfig.name}</p>
    </div>
  );
};
```

## Temporal Workflow Integration

### Best Practices

1. **Idempotent Activities**: Ensure activities can be safely retried
2. **Proper Error Handling**: Use appropriate error types for different scenarios
3. **Timeouts**: Set reasonable timeouts for activities and workflows
4. **Versioning**: Use workflow versioning for backward compatibility

### Example: Complex Workflow

```javascript
@Workflow()
async function dataProcessingWorkflow(input) {
  const { dataId, processingOptions } = input;

  try {
    // Step 1: Validate input data
    const validation = await Activity.execute('validateData', { dataId });
    if (!validation.valid) {
      throw new Error(`Data validation failed: ${validation.errors.join(', ')}`);
    }

    // Step 2: Process data in parallel
    const [processedData, metadata] = await Promise.all([
      Activity.execute('processData', { dataId, options: processingOptions }),
      Activity.execute('extractMetadata', { dataId })
    ]);

    // Step 3: Store results
    const storageResult = await Activity.execute('storeResults', {
      dataId,
      processedData,
      metadata
    });

    // Step 4: Send notification
    await Activity.execute('sendNotification', {
      type: 'processing_complete',
      dataId,
      resultId: storageResult.id
    });

    return {
      success: true,
      resultId: storageResult.id,
      processedAt: new Date().toISOString()
    };
  } catch (error) {
    // Compensation logic
    await Activity.execute('cleanupFailedProcessing', { dataId });
    throw error;
  }
}
```

## Frontend Components

### Module Federation Setup

Your module's frontend components are automatically integrated using Module Federation. The build process handles the configuration.

### Styling Guidelines

Use CSS modules or styled-components for component styling:

```javascript
// Using CSS modules
import styles from './MyComponent.module.css';

export const MyComponent = () => {
  return (
    <div className={styles.container}>
      <h1 className={styles.title}>My Module</h1>
    </div>
  );
};
```

```css
/* MyComponent.module.css */
.container {
  padding: 20px;
  background: var(--module-bg-color);
}

.title {
  color: var(--module-text-color);
  font-size: 24px;
}
```

### State Management

Use React hooks and context for state management:

```javascript
import React, { createContext, useContext, useReducer } from 'react';

const ModuleStateContext = createContext();

export const ModuleStateProvider = ({ children }) => {
  const [state, dispatch] = useReducer(moduleReducer, initialState);
  
  return (
    <ModuleStateContext.Provider value={{ state, dispatch }}>
      {children}
    </ModuleStateContext.Provider>
  );
};

export const useModuleState = () => {
  const context = useContext(ModuleStateContext);
  if (!context) {
    throw new Error('useModuleState must be used within ModuleStateProvider');
  }
  return context;
};
```

## Testing Your Module

### Unit Tests

```javascript
// tests/unit/activities.test.js
const HelloWorldActivities = require('../../src/activities');

describe('HelloWorldActivities', () => {
  let activities;

  beforeEach(() => {
    activities = new HelloWorldActivities();
  });

  test('sayHello should return greeting message', async () => {
    const result = await activities.sayHello({ name: 'Alice' });
    
    expect(result.message).toBe('Hello, Alice!');
    expect(result.timestamp).toBeDefined();
  });

  test('processGreeting should translate greeting', async () => {
    const result = await activities.processGreeting({
      greeting: 'Hello, World!',
      language: 'es'
    });
    
    expect(result.translatedGreeting).toBe('Hola');
    expect(result.language).toBe('es');
  });
});
```

### Integration Tests

```javascript
// tests/integration/workflow.test.js
const { TestWorkflowEnvironment } = require('@adx-core/temporal-testing');

describe('Greeting Workflow Integration', () => {
  let testEnv;

  beforeAll(async () => {
    testEnv = await TestWorkflowEnvironment.createTimeSkipping();
  });

  afterAll(async () => {
    await testEnv.teardown();
  });

  test('should complete greeting workflow successfully', async () => {
    const { client, nativeConnection } = testEnv;
    
    const result = await client.workflow.execute('greetingWorkflow', {
      args: [{ name: 'Test User', language: 'en' }],
      taskQueue: 'hello-world-queue',
      workflowId: 'test-greeting-workflow',
    });

    expect(result.finalMessage).toContain('Hello, Test User!');
    expect(result.processedAt).toBeDefined();
  });
});
```

### End-to-End Tests

```javascript
// tests/e2e/module.test.js
const { test, expect } = require('@playwright/test');

test.describe('Hello World Module E2E', () => {
  test('should display greeting form and process greeting', async ({ page }) => {
    // Navigate to module
    await page.goto('http://localhost:3000/modules/hello-world');
    
    // Fill in form
    await page.fill('[data-testid="name-input"]', 'E2E Test');
    await page.selectOption('[data-testid="language-select"]', 'es');
    
    // Submit form
    await page.click('[data-testid="greet-button"]');
    
    // Wait for result
    await page.waitForSelector('[data-testid="greeting-result"]');
    
    // Verify result
    const greeting = await page.textContent('[data-testid="greeting-result"]');
    expect(greeting).toContain('Hola, E2E Test!');
  });
});
```

### Running Tests

```bash
# Run all tests
npm test

# Run unit tests only
npm run test:unit

# Run integration tests
npm run test:integration

# Run e2e tests
npm run test:e2e

# Run tests with coverage
npm run test:coverage
```

## Security Guidelines

### Input Validation

Always validate and sanitize user inputs:

```javascript
const { body, validationResult } = require('express-validator');

router.post('/greet', [
  body('name')
    .isLength({ min: 1, max: 100 })
    .trim()
    .escape()
    .withMessage('Name must be between 1 and 100 characters'),
  body('language')
    .isIn(['en', 'es', 'fr', 'de'])
    .withMessage('Invalid language code')
], async (req, res) => {
  const errors = validationResult(req);
  if (!errors.isEmpty()) {
    return res.status(400).json({
      success: false,
      errors: errors.array()
    });
  }
  
  // Process request...
});
```

### SQL Injection Prevention

Use parameterized queries:

```javascript
// Good - parameterized query
const result = await db.query(
  'SELECT * FROM users WHERE tenant_id = $1 AND name = $2',
  [tenantId, name]
);

// Bad - string concatenation
const result = await db.query(
  `SELECT * FROM users WHERE tenant_id = '${tenantId}' AND name = '${name}'`
);
```

### XSS Prevention

Sanitize output in frontend components:

```javascript
import DOMPurify from 'dompurify';

export const SafeContent = ({ content }) => {
  const sanitizedContent = DOMPurify.sanitize(content);
  
  return (
    <div dangerouslySetInnerHTML={{ __html: sanitizedContent }} />
  );
};
```

### Authentication and Authorization

Check permissions in your endpoints:

```javascript
const { requirePermission } = require('@adx-core/auth');

router.post('/admin-action', 
  requirePermission('module:hello-world:admin'),
  async (req, res) => {
    // Admin-only action
  }
);
```

## Publishing to Marketplace

### 1. Validate Your Module

```bash
# Validate module structure and code
adx-cli module validate

# Run security scan
adx-cli module scan --deep

# Run all tests
npm test
```

### 2. Build and Package

```bash
# Build module for production
npm run build

# Create package
adx-cli module package --output=./dist/hello-world-1.0.0.tar.gz
```

### 3. Publish to Marketplace

```bash
# Login to ADX Core marketplace
adx-cli auth login

# Publish module
adx-cli module publish ./dist/hello-world-1.0.0.tar.gz \
  --category=utilities \
  --tags=greeting,demo \
  --price=free
```

### 4. Monitor Your Module

After publishing, monitor your module's performance:

- View download statistics
- Monitor user reviews and ratings
- Track usage analytics
- Respond to user feedback

## Best Practices

### Code Organization

1. **Separation of Concerns**: Keep activities, workflows, and UI components separate
2. **Error Handling**: Implement comprehensive error handling with proper error types
3. **Logging**: Use structured logging for debugging and monitoring
4. **Configuration**: Use environment variables for configuration
5. **Documentation**: Document your APIs and components thoroughly

### Performance

1. **Async Operations**: Use async/await for all I/O operations
2. **Caching**: Implement caching for frequently accessed data
3. **Lazy Loading**: Load components and data only when needed
4. **Resource Management**: Clean up resources properly in cleanup methods

### Temporal Workflows

1. **Deterministic Code**: Ensure workflow code is deterministic
2. **Activity Timeouts**: Set appropriate timeouts for activities
3. **Retry Policies**: Configure retry policies for transient failures
4. **Versioning**: Use workflow versioning for backward compatibility

### Frontend Development

1. **Responsive Design**: Ensure components work on all screen sizes
2. **Accessibility**: Follow WCAG guidelines for accessibility
3. **Performance**: Optimize bundle size and loading times
4. **User Experience**: Provide clear feedback for user actions

## Troubleshooting

### Common Issues

#### Module Not Loading

**Problem**: Module doesn't appear in the ADX Core interface.

**Solutions**:
1. Check module manifest syntax
2. Verify module is properly installed
3. Check browser console for errors
4. Ensure module permissions are correct

#### Workflow Execution Fails

**Problem**: Temporal workflows fail to execute.

**Solutions**:
1. Check Temporal server connection
2. Verify activity implementations
3. Check workflow timeout settings
4. Review activity retry policies

#### Frontend Components Not Rendering

**Problem**: React components don't render properly.

**Solutions**:
1. Check Module Federation configuration
2. Verify component exports
3. Check for JavaScript errors in console
4. Ensure proper context providers

#### Database Connection Issues

**Problem**: Database operations fail.

**Solutions**:
1. Verify database connection string
2. Check database permissions
3. Ensure migrations are applied
4. Review connection pool settings

### Debug Mode

Enable debug mode for detailed logging:

```bash
# Start development server with debug mode
DEBUG=adx-core:* adx-cli dev start

# Run module with debug logging
DEBUG=my-module:* npm start
```

### Getting Help

1. **Documentation**: Check the official ADX Core documentation
2. **Community Forum**: Ask questions in the developer community
3. **GitHub Issues**: Report bugs and feature requests
4. **Support**: Contact support for enterprise customers

## Conclusion

This developer portal provides everything you need to create powerful modules for the ADX Core platform. Start with the basic template, follow the best practices, and gradually add more advanced features as you become comfortable with the platform.

Happy coding! 🚀

---

For more information, visit:
- [ADX Core Documentation](https://docs.adxcore.com)
- [Module SDK Reference](https://docs.adxcore.com/sdk)
- [Temporal Documentation](https://docs.temporal.io)
- [Community Forum](https://community.adxcore.com)