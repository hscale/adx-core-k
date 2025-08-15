# File Micro-Frontend

The File micro-frontend provides comprehensive file management capabilities including upload, browsing, sharing, and permission management.

## Features

- **File Upload**: Drag-and-drop file upload with progress tracking
- **File Browser**: Navigate and manage files and folders
- **File Sharing**: Share files with users, teams, and external links
- **Permission Management**: Granular access control for files and folders
- **File Preview**: Preview common file types
- **Version Control**: Track file versions and changes
- **Storage Management**: Monitor storage usage and quotas

## Architecture

This micro-frontend follows the ADX Core architecture patterns:

- **Module Federation**: Exposes components for use in the Shell application
- **BFF Integration**: Communicates with File BFF service (port 4003)
- **Temporal Workflows**: Complex operations like file processing are handled as workflows
- **Multi-tenant**: Full tenant isolation and context awareness
- **Event-driven**: Communicates with other micro-frontends via event bus

## Development

```bash
# Install dependencies
npm install

# Start development server (port 3003)
npm run dev

# Build for production
npm run build

# Type checking
npm run type-check

# Linting
npm run lint
npm run lint:fix
```

## Integration

The File micro-frontend integrates with:

- **File BFF Service** (port 4003): Optimized API for file operations
- **File Service** (backend): Core file management workflows
- **Shell Application**: Main container application
- **Other Micro-frontends**: Via event bus for file-related events

## Components

### Exposed Components

- `FileUpload`: File upload interface with progress tracking
- `FileBrowser`: File and folder navigation
- `FileManager`: Complete file management interface
- `FileSharing`: File sharing and link management
- `FilePermissions`: Access control management
- `FileProvider`: Context provider for file state

### Internal Components

- File preview components
- Storage quota displays
- File operation modals
- Permission management dialogs

## API Integration

The micro-frontend communicates with the File BFF service for:

- File upload and download
- File metadata management
- Sharing and permissions
- Storage quota tracking
- File search and filtering

All complex operations (file processing, virus scanning, etc.) are handled as Temporal workflows through the BFF service.