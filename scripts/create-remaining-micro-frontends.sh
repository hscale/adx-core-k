#!/bin/bash

# Create remaining micro-frontends for ADX CORE

set -e

# Colors for output
GREEN='\033[0;32m'
NC='\033[0m'

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

# Micro-frontends to create
MICRO_FRONTENDS=(
    "file:3003"
    "user:3004" 
    "workflow:3005"
    "dashboard:3006"
)

for mf_config in "${MICRO_FRONTENDS[@]}"; do
    IFS=':' read -r name port <<< "$mf_config"
    
    print_status "Creating $name micro-frontend on port $port"
    
    # Create directory structure
    mkdir -p "micro-frontends/${name}-micro-app/src/{components,pages,hooks,services,types,utils}"
    
    # Create package.json
    cat > "micro-frontends/${name}-micro-app/package.json" << EOF
{
  "name": "@adx-core/${name}-micro-app",
  "version": "1.0.0",
  "description": "ADX CORE ${name^} Management Micro-Frontend",
  "type": "module",
  "scripts": {
    "dev": "vite --port ${port}",
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
    cat > "micro-frontends/${name}-micro-app/vite.config.ts" << EOF
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { federation } from '@originjs/vite-plugin-federation'
import { resolve } from 'path'

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: '${name}MicroApp',
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
    port: ${port},
    cors: true,
    host: true,
  },
})
EOF

    # Create tsconfig.json
    cat > "micro-frontends/${name}-micro-app/tsconfig.json" << EOF
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

    # Create tsconfig.node.json
    cat > "micro-frontends/${name}-micro-app/tsconfig.node.json" << EOF
{
  "compilerOptions": {
    "composite": true,
    "skipLibCheck": true,
    "module": "ESNext",
    "moduleResolution": "bundler",
    "allowSyntheticDefaultImports": true
  },
  "include": ["vite.config.ts"]
}
EOF

    # Create index.html
    cat > "micro-frontends/${name}-micro-app/index.html" << EOF
<!DOCTYPE html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>ADX CORE - ${name^} Management</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
EOF

    # Create main.tsx
    cat > "micro-frontends/${name}-micro-app/src/main.tsx" << EOF
import('./bootstrap')
EOF

    # Create bootstrap.tsx
    cat > "micro-frontends/${name}-micro-app/src/bootstrap.tsx" << EOF
import React from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { ${name^}MicroApp } from './App'

const root = ReactDOM.createRoot(document.getElementById('root')!)

root.render(
  <React.StrictMode>
    <BrowserRouter>
      <${name^}MicroApp />
    </BrowserRouter>
  </React.StrictMode>
)
EOF

    # Create App.tsx
    cat > "micro-frontends/${name}-micro-app/src/App.tsx" << EOF
import React from 'react'
import { Routes, Route } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { HomePage } from './pages/HomePage'

const queryClient = new QueryClient({
  defaultOptions: {
    queries: {
      staleTime: 5 * 60 * 1000, // 5 minutes
      retry: 1,
    },
  },
})

export const ${name^}MicroApp: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <div className="p-6">
        <Routes>
          <Route path="/" element={<HomePage />} />
          <Route path="/*" element={<div>404 - Page not found</div>} />
        </Routes>
      </div>
    </QueryClientProvider>
  )
}

export default ${name^}MicroApp
EOF

    # Create routes.tsx
    cat > "micro-frontends/${name}-micro-app/src/routes.tsx" << EOF
import React from 'react'
import { RouteObject } from 'react-router-dom'
import { HomePage } from './pages/HomePage'

export const ${name}Routes: RouteObject[] = [
  {
    path: '/',
    element: <HomePage />,
  },
]

export default ${name}Routes
EOF

    # Create HomePage.tsx
    cat > "micro-frontends/${name}-micro-app/src/pages/HomePage.tsx" << EOF
import React from 'react'

export const HomePage: React.FC = () => {
  return (
    <div className="max-w-7xl mx-auto">
      <div className="mb-8">
        <h1 className="text-3xl font-bold text-gray-900 mb-2">
          ${name^} Management
        </h1>
        <p className="text-gray-600">
          Manage your ${name} resources and configurations.
        </p>
      </div>
      
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            ${name^} Overview
          </h3>
          <p className="text-gray-600 text-sm">
            View and manage your ${name} resources.
          </p>
        </div>
        
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            Settings
          </h3>
          <p className="text-gray-600 text-sm">
            Configure ${name} settings and preferences.
          </p>
        </div>
        
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-2">
            Analytics
          </h3>
          <p className="text-gray-600 text-sm">
            View ${name} usage and performance metrics.
          </p>
        </div>
      </div>
      
      <div className="mt-8 p-4 bg-blue-50 border border-blue-200 rounded-md">
        <h2 className="text-lg font-semibold text-blue-900 mb-2">
          Development Status
        </h2>
        <ul className="text-blue-800 space-y-1 text-sm">
          <li>✅ Micro-frontend structure created</li>
          <li>✅ Module Federation configured</li>
          <li>✅ TypeScript setup complete</li>
          <li>✅ Running on port ${port}</li>
          <li>🔄 Ready for feature development</li>
        </ul>
      </div>
    </div>
  )
}
EOF

    # Create service file
    cat > "micro-frontends/${name}-micro-app/src/services/${name}Service.ts" << EOF
import axios from 'axios'

const BFF_BASE_URL = process.env.NODE_ENV === 'development' 
  ? 'http://localhost:400${port: -1}' 
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

export const ${name}Service = {
  async getHealth() {
    const response = await api.get('/health')
    return response.data
  },
  
  // Add your ${name}-specific service methods here
}
EOF

    # Create types file
    cat > "micro-frontends/${name}-micro-app/src/types/index.ts" << EOF
// ${name^} Micro-Frontend Types

export interface ${name^}State {
  isLoading: boolean
  error: string | null
}

export interface ${name^}Config {
  apiUrl: string
  features: {
    [key: string]: boolean
  }
}

// Add your ${name}-specific types here
EOF

    print_status "✅ ${name^} micro-frontend created successfully"
done

print_status "All micro-frontends created successfully!"
print_status ""
print_status "Next steps:"
print_status "1. cd micro-frontends/shell && npm install"
print_status "2. cd micro-frontends/tenant-micro-app && npm install"
print_status "3. cd micro-frontends/file-micro-app && npm install"
print_status "4. cd micro-frontends/user-micro-app && npm install"
print_status "5. cd micro-frontends/workflow-micro-app && npm install"
print_status "6. cd micro-frontends/dashboard-micro-app && npm install"
print_status "7. ./scripts/dev-start-frontend.sh"