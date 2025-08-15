# Module Micro-Frontend

The Module Micro-Frontend provides comprehensive module management capabilities for ADX Core, including marketplace browsing, module installation, configuration, and development tools.

## Features

### Module Marketplace
- Browse and search available modules
- Filter by category, pricing, platform, and rating
- View detailed module information and screenshots
- Install modules with workflow tracking
- Featured, trending, and recommended modules

### Module Management
- View and manage installed modules
- Activate/deactivate modules
- Configure module settings and permissions
- Uninstall modules with confirmation
- Module status monitoring

### Module Configuration
- Configure module-specific settings
- Manage resource limits (memory, CPU, storage)
- Control module permissions
- Enable/disable modules
- Real-time configuration updates

### Module Development
- Create new module projects
- Code editor with syntax highlighting
- Test runner with coverage reporting
- Module publishing workflow
- Project management and versioning

## Architecture

### Components
- **ModuleMarketplace**: Browse and install modules
- **ModuleManager**: Manage installed modules
- **ModuleSettings**: Configure module settings
- **ModuleDeveloper**: Development tools and project management
- **ModuleCard**: Display module information
- **ModuleFilters**: Search and filter functionality
- **ModuleEditor**: Code editing interface
- **ModuleTestRunner**: Test execution and reporting

### Services
- **ModuleBFFClient**: Communication with Module BFF service
- **Module Types**: TypeScript definitions for module system
- **Module Utilities**: Helper functions for module operations

### Hooks
- **useModules**: Module marketplace and management operations
- **useModuleSearch**: Advanced module search functionality
- **useModuleConfiguration**: Module configuration management
- **useModuleDevelopment**: Development project management

## Integration

### Module Federation
The micro-frontend exposes the following components:
- `./App`: Main application component
- `./ModuleMarketplace`: Marketplace browsing interface
- `./ModuleManager`: Module management interface
- `./ModuleSettings`: Configuration interface
- `./ModuleDeveloper`: Development tools interface

### BFF Integration
Communicates with the Module BFF service (port 4006) for:
- Module marketplace operations
- Installation and management workflows
- Configuration updates
- Development project management

### Temporal Workflows
All complex operations are handled through Temporal workflows:
- Module installation and uninstallation
- Module activation and deactivation
- Module testing and publishing
- Configuration updates

## Development

### Setup
```bash
cd apps/module
npm install
npm run dev
```

### Testing
```bash
npm run test
npm run test:watch
```

### Building
```bash
npm run build
```

### Linting
```bash
npm run lint
npm run type-check
```

## Configuration

### Environment Variables
- Module BFF service URL (defaults to http://localhost:4006)
- Tenant context from shared context provider
- Authentication tokens from local storage

### Module Federation
Configured as a remote module exposing key components to the shell application.

## Security

### Permissions
- Module installation requires appropriate permissions
- Configuration changes are validated
- Development tools require developer permissions

### Sandboxing
- Module execution is sandboxed with resource limits
- Network access is controlled per module
- File system access is restricted

### Validation
- Module packages are security scanned
- Code is validated before publishing
- Dependencies are checked for vulnerabilities

## Performance

### Optimization
- Lazy loading of components
- Query caching with React Query
- Bundle size optimization
- Code splitting by route

### Monitoring
- Module performance metrics
- Installation success rates
- User interaction tracking
- Error reporting and logging