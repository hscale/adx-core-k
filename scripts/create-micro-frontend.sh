#!/bin/bash

# ADX CORE - Micro-Frontend Creation Script
# Usage: ./scripts/create-micro-frontend.sh <name> <port> <framework>
# Example: ./scripts/create-micro-frontend.sh auth 3001 react

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check arguments
if [ $# -lt 2 ]; then
    print_error "Usage: $0 <name> <port> [framework]"
    print_error "Example: $0 auth 3001 react"
    exit 1
fi

NAME=$1
PORT=$2
FRAMEWORK=${3:-react}
MICRO_FRONTEND_DIR="micro-frontends/${NAME}-micro-app"

print_status "Creating micro-frontend: $NAME on port $PORT with $FRAMEWORK"

# Check if directory already exists
if [ -d "$MICRO_FRONTEND_DIR" ]; then
    print_error "Directory $MICRO_FRONTEND_DIR already exists"
    exit 1
fi

# Create directory structure
print_status "Creating directory structure..."
mkdir -p "$MICRO_FRONTEND_DIR"/{src/{components,pages,hooks,services,types,utils},public}

# Create package.json
print_status "Creating package.json..."
cat > "$MICRO_FRONTEND_DIR/package.json" << EOF
{
  "name": "@adx-core/${NAME}-micro-app",
  "version": "1.0.0",
  "description": "ADX CORE ${NAME^} Micro-Frontend",
  "type": "module",
  "scripts": {
    "dev": "vite --port $PORT",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "vitest",
    "test:ui": "vitest --ui",
    "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "type-check": "tsc --noEmit"
  },
  "dependencies": {
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.20.1",
    "@tanstack/react-query": "^5.8.4",
    "zustand": "^4.4.7",
    "@adx-core/design-system": "workspace:*",
    "@adx-core/shared-types": "workspace:*",
    "@adx-core/shared-utils": "workspace:*"
  },
  "devDependencies": {
    "@originjs/vite-plugin-federation": "^1.3.5",
    "@types/react": "^18.2.37",
    "@types/react-dom": "^18.2.15",
    "@typescript-eslint/eslint-plugin": "^6.10.0",
    "@typescript-eslint/parser": "^6.10.0",
    "@vitejs/plugin-react": "^4.1.1",
    "@vitest/ui": "^0.34.6",
    "eslint": "^8.53.0",
    "eslint-plugin-react-hooks": "^4.6.0",
    "eslint-plugin-react-refresh": "^0.4.4",
    "typescript": "^5.2.2",
    "vite": "^5.0.0",
    "vitest": "^0.34.6"
  }
}
EOF

# Create vite.config.ts
print_status "Creating Vite configuration..."
cat > "$MICRO_FRONTEND_DIR/vite.config.ts" << EOF
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { federation } from '@originjs/vite-plugin-federation'
import { resolve } from 'path'

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: '${NAME}MicroApp',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './routes': './src/routes.tsx',
      },
      shared: {
        react: { singleton: true, requiredVersion: '^18.2.0' },
        'react-dom': { singleton: true, requiredVersion: '^18.2.0' },
        'react-router-dom': { singleton: true, requiredVersion: '^6.20.1' },
        '@tanstack/react-query': { singleton: true, requiredVersion: '^5.8.4' },
        'zustand': { singleton: true, requiredVersion: '^4.4.7' },
        '@headlessui/react': { singleton: true },
        'tailwindcss': { singleton: true },
        'framer-motion': { singleton: true },
        'react-i18next': { singleton: true },
        'lucide-react': { singleton: true },
      },
    }),
  ],
  
  resolve: {
    alias: {
      '@': resolve(__dirname, './src'),
    },
  },
  
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
    rollupOptions: {
      external: ['@tauri-apps/api'],
    },
  },
  
  server: {
    port: $PORT,
    cors: true,
    host: true,
  },
})
EOF

# Create TypeScript config
print_status "Creating TypeScript configuration..."
cat > "$MICRO_FRONTEND_DIR/tsconfig.json" << EOF
{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true,
    "baseUrl": ".",
    "paths": {
      "@/*": ["./src/*"]
    }
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
EOF

# Create main App component
print_status "Creating main App component..."
cat > "$MICRO_FRONTEND_DIR/src/App.tsx" << EOF
import React from 'react'
import { Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'

// Import your pages here
import { HomePage } from './pages/HomePage'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      retry: 1,
    },
  },
})

export const ${NAME^}MicroApp: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <div className="min-h-screen bg-gray-50">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/*" element={<div>404 - Page not found</div>} />
        </Routes>
      </div>
    </QueryClientProvider>
  )
}

export default ${NAME^}MicroApp
EOF

# Create routes export
print_status "Creating routes configuration..."
cat > "$MICRO_FRONTEND_DIR/src/routes.tsx" << EOF
import React from 'react'
import { RouteObject } from 'react-router-dom'
import { HomePage } from './pages/HomePage'

export const ${NAME}Routes: RouteObject[] = [
  {
    path: '/',
    element: <HomePage />,
  },
]

export default ${NAME}Routes
EOF

# Create sample page
print_status "Creating sample page..."
cat > "$MICRO_FRONTEND_DIR/src/pages/HomePage.tsx" << EOF
import React from 'react'

export const HomePage: React.FC = () => {
  return (
    <div className="container mx-auto px-4 py-8">
      <h1 className="text-3xl font-bold text-gray-900 mb-6">
        ${NAME^} Micro-Frontend
      </h1>
      <p className="text-gray-600">
        Welcome to the ${NAME} micro-frontend. This is running on port $PORT.
      </p>
      <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded-md">
        <h2 className="text-lg font-semibold text-blue-900 mb-2">
          Development Status
        </h2>
        <ul className="text-blue-800 space-y-1">
          <li>✅ Micro-frontend structure created</li>
          <li>✅ Module Federation configured</li>
          <li>✅ TypeScript setup complete</li>
          <li>🔄 Ready for feature development</li>
        </ul>
      </div>
    </div>
  )
}
EOF

# Create bootstrap file for standalone development
print_status "Creating bootstrap file..."
cat > "$MICRO_FRONTEND_DIR/src/bootstrap.tsx" << EOF
import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { ${NAME^}MicroApp } from './App'

// This file is used for standalone development
const root = ReactDOM.createRoot(document.getElementById('root')!)

root.render(
  <React.StrictMode>
    <BrowserRouter>
      <${NAME^}MicroApp />
    </BrowserRouter>
  </React.StrictMode>
)
EOF

# Create main entry point
print_status "Creating main entry point..."
cat > "$MICRO_FRONTEND_DIR/src/main.tsx" << EOF
import('./bootstrap')
EOF

# Create index.html
print_status "Creating index.html..."
cat > "$MICRO_FRONTEND_DIR/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>ADX CORE - ${NAME^} Micro-Frontend</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
EOF

# Create basic service file
print_status "Creating service template..."
cat > "$MICRO_FRONTEND_DIR/src/services/${NAME}Service.ts" << EOF
import axios from 'axios'

const BFF_BASE_URL = process.env.NODE_ENV === 'development' 
  ? 'http://localhost:400${PORT: -1}' // Extract last digit for BFF port
  : '/api'

const api = axios.create({
  baseURL: BFF_BASE_URL,
  timeout: 10000,
})

// Add request interceptor for auth token
api.interceptors.request.use((config) => {
  const token = localStorage.getItem('auth_token')
  if (token) {
    config.headers.Authorization = \`Bearer \${token}\`
  }
  return config
})

export const ${NAME}Service = {
  // Add your service methods here
  async getHealth() {
    const response = await api.get('/health')
    return response.data
  },
}
EOF

# Create types file
print_status "Creating types file..."
cat > "$MICRO_FRONTEND_DIR/src/types/index.ts" << EOF
// ${NAME^} Micro-Frontend Types

export interface ${NAME^}State {
  isLoading: boolean
  error: string | null
}

export interface ${NAME^}Config {
  apiUrl: string
  features: {
    [key: string]: boolean
  }
}

// Add your domain-specific types here
EOF

# Create README
print_status "Creating README..."
cat > "$MICRO_FRONTEND_DIR/README.md" << EOF
# ${NAME^} Micro-Frontend

This is the ${NAME} micro-frontend for ADX CORE, running on port $PORT.

## Development

\`\`\`bash
# Install dependencies
npm install

# Start development server
npm run dev

# Build for production
npm run build

# Run tests
npm run test
\`\`\`

## Architecture

This micro-frontend is part of the ADX CORE frontend microservices architecture:

- **Framework**: $FRAMEWORK
- **Port**: $PORT
- **BFF Service**: Port 400${PORT: -1}
- **Module Federation**: Exposes \`./App\` and \`./routes\`

## Integration

This micro-frontend integrates with:
- Shell application (port 3000)
- ${NAME^} BFF service (port 400${PORT: -1})
- Shared design system
- Global event bus

## Features

- [ ] Add your features here
- [ ] Feature 1
- [ ] Feature 2

## API Integration

The micro-frontend communicates with its BFF service at:
- Development: http://localhost:400${PORT: -1}
- Production: /api

## Testing

\`\`\`bash
# Unit tests
npm run test

# UI tests
npm run test:ui

# Type checking
npm run type-check
\`\`\`
EOF

print_status "Micro-frontend '$NAME' created successfully!"
print_status "Location: $MICRO_FRONTEND_DIR"
print_status "Port: $PORT"
print_status ""
print_status "Next steps:"
print_status "1. cd $MICRO_FRONTEND_DIR"
print_status "2. npm install"
print_status "3. npm run dev"
print_status "4. Open http://localhost:$PORT"
print_status ""
print_warning "Don't forget to:"
print_warning "- Add the micro-frontend to the shell application's remote configuration"
print_warning "- Create the corresponding BFF service on port 400${PORT: -1}"
print_warning "- Update the root package.json scripts"