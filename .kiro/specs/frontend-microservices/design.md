# Frontend Microservices Architecture - Design Document

## Overview

This design document outlines the frontend microservices architecture for ADX CORE that integrates with our temporal-first backend microservices. The architecture enables team autonomy, independent deployment, and technology flexibility while maintaining cross-platform compatibility with Tauri.

The architecture leverages Vite's Module Federation capabilities, integrates with Temporal workflows through optional BFF services, and ensures seamless operation across web, desktop, and mobile platforms.

## Architecture

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    Cross-Platform Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │ Web Browser │  │   Desktop   │  │ Mobile Web  │  │ Native Apps │  │
│  │   (Direct)  │  │   (Tauri)   │  │(Responsive) │  │(Tauri 2.0)  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                    Shell Application                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  Shell Container (React + Vite Module Federation)      │   │
│  │  - Routing & Navigation                                 │   │
│  │  - Authentication State                                 │   │
│  │  - Global Theme & i18n                                 │   │
│  │  - Shared Design System                                │   │
│  │  - Error Boundaries                                    │   │
│  │  - Event Bus                                           │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                    Micro-Frontend Layer                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │    Auth     │  │   Tenant    │  │    File     │  │    User     │  │
│  │ Micro-App   │  │ Micro-App   │  │ Micro-App   │  │ Micro-App   │  │
│  │             │  │             │  │             │  │             │  │
│  │ React/Vue/  │  │ React/Vue/  │  │ React/Vue/  │  │ React/Vue/  │  │
│  │ Svelte/etc  │  │ Svelte/etc  │  │ Svelte/etc  │  │ Svelte/etc  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │  Workflow   │  │   Plugin    │  │ Analytics   │  │   Custom    │  │
│  │ Micro-App   │  │ Micro-Apps  │  │ Micro-App   │  │ Micro-Apps  │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                    Optional BFF Layer                         │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │   Auth BFF  │  │ Tenant BFF  │  │  File BFF   │  │  User BFF   │  │
│  │(Port 4001)  │  │(Port 4002)  │  │(Port 4003)  │  │(Port 4004)  │  │
│  │Temporal     │  │Temporal     │  │Temporal     │  │Temporal     │  │
│  │Workflow     │  │Workflow     │  │Workflow     │  │Workflow     │  │
│  │Clients      │  │Clients      │  │Clients      │  │Clients      │  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                    Temporal-First API Gateway                  │
│                    (Port 8080 - Workflow Orchestration)       │
└─────────────────────────────────────────────────────────────────┘
                                 │
┌─────────────────────────────────────────────────────────────────┐
│                    Backend Microservices                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │
│  │Auth Service │  │Tenant Service│ │File Service │  │User Service │  │
│  │(Port 8081)  │  │(Port 8085)  │  │(Port 8083)  │  │(Port 8082)  │  │
│  │HTTP + Worker│  │HTTP + Worker│  │HTTP + Worker│  │HTTP + Worker│  │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘  │
│                    Temporal Activities + Direct Endpoints         │
└─────────────────────────────────────────────────────────────────┘
```

## Components and Interfaces

### 1. Shell Application

The shell application serves as the container and orchestrator for all micro-frontends.

#### Shell Architecture

```typescript
// Shell application structure
interface ShellApplication {
  // Core shell responsibilities
  router: MicroFrontendRouter;
  authenticator: GlobalAuthenticator;
  themeManager: GlobalThemeManager;
  i18nManager: GlobalI18nManager;
  eventBus: CrossMicroFrontendEventBus;
  errorBoundary: GlobalErrorBoundary;
  
  // Micro-frontend management
  microFrontendRegistry: MicroFrontendRegistry;
  moduleLoader: ModuleFederationLoader;
  dependencyManager: SharedDependencyManager;
}

// Shell configuration
interface ShellConfig {
  microFrontends: MicroFrontendConfig[];
  sharedDependencies: SharedDependency[];
  routing: RoutingConfig;
  authentication: AuthConfig;
  theme: ThemeConfig;
  i18n: I18nConfig;
}

interface MicroFrontendConfig {
  name: string;
  entry: string;
  routes: string[];
  exposedModule: string;
  framework: 'react' | 'vue' | 'svelte' | 'angular';
  version: string;
  dependencies: string[];
  permissions: string[];
  bffEndpoint?: string;
}
```

#### Vite Module Federation Configuration

```typescript
// vite.config.ts for Shell Application
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'shell',
      remotes: {
        authMicroApp: 'http://localhost:3001/assets/remoteEntry.js',
        tenantMicroApp: 'http://localhost:3002/assets/remoteEntry.js',
        fileMicroApp: 'http://localhost:3003/assets/remoteEntry.js',
        userMicroApp: 'http://localhost:3004/assets/remoteEntry.js',
        workflowMicroApp: 'http://localhost:3005/assets/remoteEntry.js',
      },
      shared: {
        react: { singleton: true, requiredVersion: '^18.2.0' },
        'react-dom': { singleton: true, requiredVersion: '^18.2.0' },
        'react-router-dom': { singleton: true, requiredVersion: '^6.20.1' },
        '@tanstack/react-query': { singleton: true, requiredVersion: '^5.8.4' },
        'zustand': { singleton: true, requiredVersion: '^4.4.7' },
        '@headlessui/react': { singleton: true, requiredVersion: '^1.7.17' },
        'tailwindcss': { singleton: true, requiredVersion: '^3.3.6' },
        'framer-motion': { singleton: true, requiredVersion: '^10.16.5' },
        'react-i18next': { singleton: true, requiredVersion: '^13.5.0' },
        'lucide-react': { singleton: true, requiredVersion: '^0.294.0' },
      },
    }),
  ],
  
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
    rollupOptions: {
      external: ['@tauri-apps/api'],
    },
  },
  
  server: {
    port: 3000,
    cors: true,
  },
});
```

#### Shell Application Implementation

```typescript
// Shell App Component
import React, { Suspense, useEffect, useState } from 'react';
import { BrowserRouter, Routes, Route, Navigate } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { I18nextProvider } from 'react-i18next';

import { GlobalErrorBoundary } from './components/GlobalErrorBoundary';
import { GlobalThemeProvider } from './providers/GlobalThemeProvider';
import { GlobalAuthProvider } from './providers/GlobalAuthProvider';
import { MicroFrontendLoader } from './components/MicroFrontendLoader';
import { NavigationShell } from './components/NavigationShell';
import { EventBusProvider } from './providers/EventBusProvider';

import { microFrontendRegistry } from './config/microFrontends';
import { i18n } from './config/i18n';

const queryClient = new QueryClient();

export const ShellApp: React.FC = () => {
  const [microFrontends, setMicroFrontends] = useState(microFrontendRegistry);

  useEffect(() => {
    // Load dynamic micro-frontend configurations
    loadDynamicMicroFrontends().then(setMicroFrontends);
  }, []);

  return (
    <GlobalErrorBoundary>
      <QueryClientProvider client={queryClient}>
        <I18nextProvider i18n={i18n}>
          <GlobalThemeProvider>
            <GlobalAuthProvider>
              <EventBusProvider>
                <BrowserRouter>
                  <NavigationShell>
                    <Suspense fallback={<div>Loading...</div>}>
                      <Routes>
                        {microFrontends.map((mf) => (
                          <Route
                            key={mf.name}
                            path={`${mf.basePath}/*`}
                            element={
                              <MicroFrontendLoader
                                name={mf.name}
                                entry={mf.entry}
                                exposedModule={mf.exposedModule}
                                fallback={<div>Loading {mf.name}...</div>}
                              />
                            }
                          />
                        ))}
                        <Route path="/" element={<Navigate to="/dashboard" replace />} />
                      </Routes>
                    </Suspense>
                  </NavigationShell>
                </BrowserRouter>
              </EventBusProvider>
            </GlobalAuthProvider>
          </GlobalThemeProvider>
        </I18nextProvider>
      </QueryClientProvider>
    </GlobalErrorBoundary>
  );
};

// Micro-frontend loader component
interface MicroFrontendLoaderProps {
  name: string;
  entry: string;
  exposedModule: string;
  fallback: React.ReactNode;
}

export const MicroFrontendLoader: React.FC<MicroFrontendLoaderProps> = ({
  name,
  entry,
  exposedModule,
  fallback,
}) => {
  const [Component, setComponent] = useState<React.ComponentType | null>(null);
  const [error, setError] = useState<Error | null>(null);

  useEffect(() => {
    const loadMicroFrontend = async () => {
      try {
        // Dynamic import with Module Federation
        const module = await import(/* @vite-ignore */ entry);
        const MicroFrontendComponent = module[exposedModule];
        setComponent(() => MicroFrontendComponent);
      } catch (err) {
        console.error(`Failed to load micro-frontend ${name}:`, err);
        setError(err as Error);
      }
    };

    loadMicroFrontend();
  }, [name, entry, exposedModule]);

  if (error) {
    return (
      <div className="p-4 bg-red-50 border border-red-200 rounded-md">
        <h3 className="text-red-800 font-medium">Failed to load {name}</h3>
        <p className="text-red-600 text-sm mt-1">{error.message}</p>
      </div>
    );
  }

  if (!Component) {
    return <>{fallback}</>;
  }

  return <Component />;
};
```

### 2. Micro-Frontend Architecture

Each micro-frontend follows a standardized structure while allowing technology flexibility.

#### Micro-Frontend Template Structure

```
micro-frontends/
├── auth-micro-app/
│   ├── src/
│   │   ├── components/
│   │   ├── pages/
│   │   ├── hooks/
│   │   ├── services/
│   │   ├── types/
│   │   ├── App.tsx
│   │   └── bootstrap.tsx
│   ├── vite.config.ts
│   ├── package.json
│   └── Dockerfile
├── tenant-micro-app/
├── file-micro-app/
├── user-micro-app/
├── workflow-micro-app/
└── shared/
    ├── design-system/
    ├── types/
    ├── utils/
    └── hooks/
```

#### Micro-Frontend Configuration Template

```typescript
// vite.config.ts for Micro-Frontend
import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import { federation } from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'authMicroApp',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './AuthProvider': './src/providers/AuthProvider.tsx',
        './LoginPage': './src/pages/LoginPage.tsx',
      },
      shared: {
        react: { singleton: true, requiredVersion: '^18.2.0' },
        'react-dom': { singleton: true, requiredVersion: '^18.2.0' },
        'react-router-dom': { singleton: true, requiredVersion: '^6.20.1' },
        '@tanstack/react-query': { singleton: true, requiredVersion: '^5.8.4' },
        'zustand': { singleton: true, requiredVersion: '^4.4.7' },
        '@headlessui/react': { singleton: true, requiredVersion: '^1.7.17' },
        'tailwindcss': { singleton: true, requiredVersion: '^3.3.6' },
      },
    }),
  ],
  
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
  },
  
  server: {
    port: 3001,
    cors: true,
  },
});

// package.json for Micro-Frontend
{
  "name": "@adx-core/auth-micro-app",
  "version": "1.0.0",
  "type": "module",
  "scripts": {
    "dev": "vite",
    "build": "tsc && vite build",
    "preview": "vite preview",
    "test": "vitest",
    "lint": "eslint . --ext ts,tsx"
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
    "@vitejs/plugin-react": "^4.1.1",
    "typescript": "^5.2.2",
    "vite": "^5.0.0"
  }
}
```

#### Micro-Frontend Implementation Example

```typescript
// src/App.tsx - Auth Micro-Frontend
import React from 'react';
import { Routes, Route } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';

import { LoginPage } from './pages/LoginPage';
import { RegisterPage } from './pages/RegisterPage';
import { ForgotPasswordPage } from './pages/ForgotPasswordPage';
import { MFASetupPage } from './pages/MFASetupPage';
import { AuthProvider } from './providers/AuthProvider';

const queryClient = new QueryClient();

export const AuthMicroApp: React.FC = () => {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <Routes>
          <Route path="/login" element={<LoginPage />} />
          <Route path="/register" element={<RegisterPage />} />
          <Route path="/forgot-password" element={<ForgotPasswordPage />} />
          <Route path="/mfa-setup" element={<MFASetupPage />} />
        </Routes>
      </AuthProvider>
    </QueryClientProvider>
  );
};

// Bootstrap file for standalone development
// src/bootstrap.tsx
import React from 'react';
import ReactDOM from 'react-dom/client';
import { BrowserRouter } from 'react-router-dom';
import { AuthMicroApp } from './App';

const root = ReactDOM.createRoot(document.getElementById('root')!);

root.render(
  <React.StrictMode>
    <BrowserRouter>
      <AuthMicroApp />
    </BrowserRouter>
  </React.StrictMode>
);
```

### 3. Backend for Frontend (BFF) Services

Each micro-frontend has its own BFF service that aggregates data from multiple backend microservices.

#### BFF Architecture

```typescript
// BFF Service Interface
interface BFFService {
  name: string;
  port: number;
  microFrontend: string;
  backendServices: string[];
  endpoints: BFFEndpoint[];
}

interface BFFEndpoint {
  path: string;
  method: 'GET' | 'POST' | 'PUT' | 'DELETE';
  aggregatedServices: ServiceCall[];
  responseTransformer: (data: any[]) => any;
}

interface ServiceCall {
  service: string;
  endpoint: string;
  method: string;
  dependencies?: string[];
}
```

#### Auth BFF Implementation (Node.js/TypeScript)

```typescript
// auth-bff/src/server.ts
import express from 'express';
import cors from 'cors';
import { createProxyMiddleware } from 'http-proxy-middleware';

import { authRoutes } from './routes/auth';
import { userProfileRoutes } from './routes/userProfile';
import { tenantSwitchingRoutes } from './routes/tenantSwitching';

const app = express();
const PORT = process.env.PORT || 4001;

// Middleware
app.use(cors());
app.use(express.json());

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'healthy', service: 'auth-bff' });
});

// Aggregated routes specific to Auth micro-frontend
app.use('/api/auth', authRoutes);
app.use('/api/user-profile', userProfileRoutes);
app.use('/api/tenant-switching', tenantSwitchingRoutes);

// Proxy for direct backend calls when aggregation isn't needed
app.use('/api/auth-service', createProxyMiddleware({
  target: 'http://localhost:8081',
  changeOrigin: true,
  pathRewrite: { '^/api/auth-service': '' },
}));

app.listen(PORT, () => {
  console.log(`Auth BFF running on port ${PORT}`);
});

// auth-bff/src/routes/auth.ts
import { Router } from 'express';
import axios from 'axios';

const router = Router();

// Aggregated login endpoint
router.post('/login', async (req, res) => {
  try {
    const { email, password, tenantId } = req.body;
    
    // Call multiple backend services
    const [authResult, userProfile, tenantInfo] = await Promise.all([
      // Authenticate user
      axios.post('http://localhost:8081/authenticate', { email, password }),
      
      // Get user profile
      axios.get(`http://localhost:8082/users/${email}`),
      
      // Get tenant information if provided
      tenantId ? axios.get(`http://localhost:8085/tenants/${tenantId}`) : null,
    ]);
    
    // Transform and combine data for frontend
    const response = {
      token: authResult.data.token,
      user: {
        id: userProfile.data.id,
        email: userProfile.data.email,
        name: userProfile.data.name,
        avatar: userProfile.data.avatar,
        preferences: userProfile.data.preferences,
      },
      tenant: tenantInfo?.data ? {
        id: tenantInfo.data.id,
        name: tenantInfo.data.name,
        logo: tenantInfo.data.branding?.logo,
        theme: tenantInfo.data.branding?.theme,
      } : null,
      permissions: authResult.data.permissions,
    };
    
    res.json(response);
  } catch (error) {
    console.error('Login aggregation error:', error);
    res.status(500).json({ error: 'Login failed' });
  }
});

// Aggregated user switching endpoint
router.post('/switch-tenant', async (req, res) => {
  try {
    const { userId, tenantId } = req.body;
    
    // Call multiple services to switch context
    const [tenantInfo, userPermissions, tenantPreferences] = await Promise.all([
      axios.get(`http://localhost:8085/tenants/${tenantId}`),
      axios.get(`http://localhost:8081/permissions/${userId}/${tenantId}`),
      axios.get(`http://localhost:8082/users/${userId}/tenant-preferences/${tenantId}`),
    ]);
    
    const response = {
      tenant: tenantInfo.data,
      permissions: userPermissions.data,
      preferences: tenantPreferences.data,
    };
    
    res.json(response);
  } catch (error) {
    console.error('Tenant switching error:', error);
    res.status(500).json({ error: 'Tenant switching failed' });
  }
});

export { router as authRoutes };
```

#### File BFF Implementation (Rust/Axum)

```rust
// file-bff/src/main.rs
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    auth_service_url: String,
    file_service_url: String,
    tenant_service_url: String,
    user_service_url: String,
}

#[derive(Serialize, Deserialize)]
struct FileListResponse {
    files: Vec<FileInfo>,
    folders: Vec<FolderInfo>,
    permissions: FilePermissions,
    storage_usage: StorageUsage,
    sharing_info: Vec<SharingInfo>,
}

#[derive(Serialize, Deserialize)]
struct FileInfo {
    id: String,
    name: String,
    size: u64,
    mime_type: String,
    created_at: String,
    updated_at: String,
    owner: UserInfo,
    permissions: FilePermissions,
    versions: Vec<FileVersion>,
    sharing: Option<SharingInfo>,
}

// Aggregated file listing endpoint
async fn list_files(
    Path(tenant_id): Path<String>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Result<Json<FileListResponse>, StatusCode> {
    let folder_id = params.get("folder_id").cloned().unwrap_or_default();
    let user_id = params.get("user_id").ok_or(StatusCode::BAD_REQUEST)?;
    
    // Make parallel requests to multiple services
    let (files_result, permissions_result, usage_result, sharing_result) = tokio::join!(
        // Get files from file service
        reqwest::get(&format!(
            "{}/files?tenant_id={}&folder_id={}&user_id={}",
            state.file_service_url, tenant_id, folder_id, user_id
        )),
        
        // Get permissions from auth service
        reqwest::get(&format!(
            "{}/permissions/files?tenant_id={}&user_id={}",
            state.auth_service_url, tenant_id, user_id
        )),
        
        // Get storage usage from tenant service
        reqwest::get(&format!(
            "{}/tenants/{}/storage-usage",
            state.tenant_service_url, tenant_id
        )),
        
        // Get sharing information
        reqwest::get(&format!(
            "{}/sharing?tenant_id={}&folder_id={}",
            state.file_service_url, tenant_id, folder_id
        ))
    );
    
    // Process and combine results
    let files = files_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<Vec<FileInfo>>().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let permissions = permissions_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<FilePermissions>().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let usage = usage_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<StorageUsage>().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let sharing = sharing_result.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .json::<Vec<SharingInfo>>().await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    // Separate files and folders
    let (files, folders): (Vec<_>, Vec<_>) = files.into_iter()
        .partition(|f| f.mime_type != "application/x-directory");
    
    let folders = folders.into_iter()
        .map(|f| FolderInfo {
            id: f.id,
            name: f.name,
            created_at: f.created_at,
            updated_at: f.updated_at,
            owner: f.owner,
            permissions: f.permissions,
        })
        .collect();
    
    Ok(Json(FileListResponse {
        files,
        folders,
        permissions,
        storage_usage: usage,
        sharing_info: sharing,
    }))
}

#[tokio::main]
async fn main() {
    let state = AppState {
        auth_service_url: "http://localhost:8081".to_string(),
        file_service_url: "http://localhost:8083".to_string(),
        tenant_service_url: "http://localhost:8085".to_string(),
        user_service_url: "http://localhost:8082".to_string(),
    };
    
    let app = Router::new()
        .route("/health", get(|| async { "healthy" }))
        .route("/api/files/:tenant_id", get(list_files))
        .route("/api/files/:tenant_id/upload", post(upload_file))
        .route("/api/files/:tenant_id/:file_id/share", post(share_file))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let listener = TcpListener::bind("0.0.0.0:4003").await.unwrap();
    println!("File BFF running on port 4003");
    
    axum::serve(listener, app).await.unwrap();
}
```

### 4. Cross-Platform Integration with Tauri

The micro-frontend architecture maintains full compatibility with Tauri for desktop and mobile applications.

#### Tauri Configuration for Micro-Frontends

```json
// src-tauri/tauri.conf.json
{
  "productName": "ADX CORE",
  "version": "1.0.0",
  "build": {
    "beforeBuildCommand": "npm run build:shell && npm run build:micro-frontends",
    "beforeDevCommand": "npm run dev:all",
    "devUrl": "http://localhost:3000",
    "frontendDist": "../dist"
  },
  "app": {
    "windows": [
      {
        "title": "ADX CORE",
        "width": 1200,
        "height": 800,
        "minWidth": 800,
        "minHeight": 600,
        "resizable": true,
        "fullscreen": false
      }
    ],
    "security": {
      "csp": "default-src 'self' http://localhost:* ws://localhost:*; connect-src ipc: http://ipc.localhost http://localhost:* ws://localhost:*; script-src 'self' 'unsafe-inline' 'unsafe-eval' http://localhost:*"
    }
  },
  "bundle": {
    "active": true,
    "targets": {
      "desktop": ["deb", "msi", "dmg", "appimage"],
      "mobile": ["ios", "android"]
    },
    "identifier": "com.adxcore.app",
    "icon": ["icons/32x32.png", "icons/128x128.png", "icons/icon.icns", "icons/icon.ico"]
  },
  "plugins": {
    "fs": {
      "scope": ["$APPDATA/adx-core/*", "$DOCUMENT/*"]
    },
    "notification": {},
    "os": {},
    "shell": {
      "scope": [
        {
          "name": "open-url",
          "cmd": "open",
          "args": ["{{url}}"]
        }
      ]
    },
    "http": {
      "scope": ["http://localhost:*", "https://*.adxcore.com"]
    }
  }
}
```

#### Platform Detection and Adaptation

```typescript
// shared/utils/platform.ts
import { invoke } from '@tauri-apps/api/core';
import { platform } from '@tauri-apps/plugin-os';

export type Platform = 'web' | 'desktop' | 'mobile';

export const detectPlatform = async (): Promise<Platform> => {
  try {
    // Check if we're in Tauri environment
    await invoke('tauri');
    
    // Get platform info
    const platformName = await platform();
    
    if (platformName === 'ios' || platformName === 'android') {
      return 'mobile';
    } else {
      return 'desktop';
    }
  } catch {
    // Not in Tauri, must be web
    return 'web';
  }
};

// Platform-specific component wrapper
export const PlatformAware: React.FC<{
  web?: React.ReactNode;
  desktop?: React.ReactNode;
  mobile?: React.ReactNode;
  children?: React.ReactNode;
}> = ({ web, desktop, mobile, children }) => {
  const [currentPlatform, setCurrentPlatform] = useState<Platform>('web');
  
  useEffect(() => {
    detectPlatform().then(setCurrentPlatform);
  }, []);
  
  switch (currentPlatform) {
    case 'web':
      return <>{web || children}</>;
    case 'desktop':
      return <>{desktop || children}</>;
    case 'mobile':
      return <>{mobile || children}</>;
    default:
      return <>{children}</>;
  }
};

// Usage in micro-frontends
export const FileUploadComponent: React.FC = () => {
  return (
    <PlatformAware
      web={<WebFileUpload />}
      desktop={<TauriFileUpload />}
      mobile={<MobileFileUpload />}
    />
  );
};
```

## Data Models

### Micro-Frontend Registry

```typescript
interface MicroFrontendRegistry {
  microFrontends: MicroFrontendDefinition[];
  sharedDependencies: SharedDependency[];
  routingConfig: RoutingConfig;
}

interface MicroFrontendDefinition {
  id: string;
  name: string;
  displayName: string;
  version: string;
  framework: 'react' | 'vue' | 'svelte' | 'angular';
  
  // Deployment configuration
  entry: {
    development: string;
    production: string;
  };
  exposedModules: Record<string, string>;
  
  // Routing configuration
  basePath: string;
  routes: RouteDefinition[];
  
  // Dependencies and sharing
  dependencies: string[];
  sharedDependencies: string[];
  
  // Permissions and access control
  permissions: string[];
  roles: string[];
  
  // BFF configuration
  bffService?: {
    url: string;
    healthCheck: string;
  };
  
  // Platform support
  platforms: Platform[];
  
  // Team ownership
  team: string;
  maintainers: string[];
  
  // Metadata
  description: string;
  tags: string[];
  category: string;
}

interface RouteDefinition {
  path: string;
  component: string;
  exact?: boolean;
  permissions?: string[];
  title?: string;
  icon?: string;
}

interface SharedDependency {
  name: string;
  version: string;
  singleton: boolean;
  requiredVersion?: string;
  eager?: boolean;
}
```

### Event Bus Schema

```typescript
interface EventBusMessage {
  type: string;
  source: string;
  target?: string;
  payload: any;
  timestamp: number;
  correlationId?: string;
}

// Standard event types
type EventType = 
  | 'auth.login'
  | 'auth.logout'
  | 'auth.token-refresh'
  | 'tenant.switch'
  | 'theme.change'
  | 'language.change'
  | 'file.upload'
  | 'file.delete'
  | 'notification.show'
  | 'navigation.change'
  | 'error.boundary'
  | 'plugin.install'
  | 'plugin.uninstall';

// Event bus implementation
class CrossMicroFrontendEventBus {
  private listeners: Map<string, Set<EventListener>> = new Map();
  
  subscribe(eventType: string, listener: EventListener): () => void {
    if (!this.listeners.has(eventType)) {
      this.listeners.set(eventType, new Set());
    }
    
    this.listeners.get(eventType)!.add(listener);
    
    // Return unsubscribe function
    return () => {
      this.listeners.get(eventType)?.delete(listener);
    };
  }
  
  emit(event: EventBusMessage): void {
    const listeners = this.listeners.get(event.type);
    if (listeners) {
      listeners.forEach(listener => {
        try {
          listener(event);
        } catch (error) {
          console.error(`Error in event listener for ${event.type}:`, error);
        }
      });
    }
  }
  
  // Typed event emitters
  emitAuthLogin(user: User, tenant?: Tenant): void {
    this.emit({
      type: 'auth.login',
      source: 'auth-micro-app',
      payload: { user, tenant },
      timestamp: Date.now(),
    });
  }
  
  emitTenantSwitch(tenant: Tenant): void {
    this.emit({
      type: 'tenant.switch',
      source: 'tenant-micro-app',
      payload: { tenant },
      timestamp: Date.now(),
    });
  }
}
```

## Error Handling

### Global Error Boundary

```typescript
interface ErrorInfo {
  microFrontend: string;
  error: Error;
  errorInfo: React.ErrorInfo;
  timestamp: number;
  userId?: string;
  tenantId?: string;
  platform: Platform;
}

export class GlobalErrorBoundary extends React.Component<
  { children: React.ReactNode },
  { hasError: boolean; errorInfo?: ErrorInfo }
> {
  constructor(props: { children: React.ReactNode }) {
    super(props);
    this.state = { hasError: false };
  }
  
  static getDerivedStateFromError(error: Error): { hasError: boolean } {
    return { hasError: true };
  }
  
  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    const errorData: ErrorInfo = {
      microFrontend: this.getMicroFrontendFromStack(error.stack),
      error,
      errorInfo,
      timestamp: Date.now(),
      platform: window.__PLATFORM__ || 'web',
    };
    
    // Log error to monitoring service
    this.logError(errorData);
    
    // Emit error event for other micro-frontends to handle
    window.dispatchEvent(new CustomEvent('micro-frontend-error', {
      detail: errorData,
    }));
    
    this.setState({ errorInfo: errorData });
  }
  
  private getMicroFrontendFromStack(stack?: string): string {
    // Parse stack trace to identify which micro-frontend caused the error
    if (!stack) return 'unknown';
    
    const microFrontendPatterns = [
      /auth-micro-app/,
      /tenant-micro-app/,
      /file-micro-app/,
      /user-micro-app/,
      /workflow-micro-app/,
    ];
    
    for (const pattern of microFrontendPatterns) {
      if (pattern.test(stack)) {
        return pattern.source.replace(/[\/\\]/g, '');
      }
    }
    
    return 'shell';
  }
  
  private async logError(errorInfo: ErrorInfo) {
    try {
      await fetch('/api/errors', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(errorInfo),
      });
    } catch (logError) {
      console.error('Failed to log error:', logError);
    }
  }
  
  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50">
          <div className="max-w-md w-full bg-white shadow-lg rounded-lg p-6">
            <div className="flex items-center mb-4">
              <div className="flex-shrink-0">
                <svg className="h-8 w-8 text-red-400" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L3.732 16.5c-.77.833.192 2.5 1.732 2.5z" />
                </svg>
              </div>
              <div className="ml-3">
                <h3 className="text-lg font-medium text-gray-900">
                  Something went wrong
                </h3>
              </div>
            </div>
            
            <div className="mb-4">
              <p className="text-sm text-gray-600">
                A component failed to load properly. The error has been logged and our team has been notified.
              </p>
              
              {this.state.errorInfo && (
                <details className="mt-2">
                  <summary className="text-sm text-gray-500 cursor-pointer">
                    Technical details
                  </summary>
                  <div className="mt-2 text-xs text-gray-400 font-mono bg-gray-100 p-2 rounded">
                    <p>Micro-frontend: {this.state.errorInfo.microFrontend}</p>
                    <p>Error: {this.state.errorInfo.error.message}</p>
                    <p>Time: {new Date(this.state.errorInfo.timestamp).toISOString()}</p>
                  </div>
                </details>
              )}
            </div>
            
            <div className="flex space-x-3">
              <button
                onClick={() => window.location.reload()}
                className="flex-1 bg-blue-600 text-white px-4 py-2 rounded-md text-sm font-medium hover:bg-blue-700"
              >
                Reload Page
              </button>
              <button
                onClick={() => this.setState({ hasError: false })}
                className="flex-1 bg-gray-200 text-gray-800 px-4 py-2 rounded-md text-sm font-medium hover:bg-gray-300"
              >
                Try Again
              </button>
            </div>
          </div>
        </div>
      );
    }
    
    return this.props.children;
  }
}
```

## Testing Strategy

### Testing Architecture

```typescript
// Testing configuration for micro-frontends
interface TestingConfig {
  unit: UnitTestConfig;
  integration: IntegrationTestConfig;
  e2e: E2ETestConfig;
  performance: PerformanceTestConfig;
}

interface UnitTestConfig {
  framework: 'vitest' | 'jest';
  coverage: {
    threshold: number;
    include: string[];
    exclude: string[];
  };
  mocking: {
    moduleLoader: boolean;
    eventBus: boolean;
    bffServices: boolean;
  };
}

// Example unit test for micro-frontend
// auth-micro-app/src/__tests__/LoginPage.test.tsx
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { vi } from 'vitest';
import { LoginPage } from '../pages/LoginPage';
import { createMockEventBus } from '@adx-core/test-utils';

// Mock the module federation loader
vi.mock('@/utils/moduleLoader', () => ({
  loadMicroFrontend: vi.fn(),
}));

// Mock the event bus
const mockEventBus = createMockEventBus();

describe('LoginPage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });
  
  it('should emit auth.login event on successful login', async () => {
    render(<LoginPage eventBus={mockEventBus} />);
    
    fireEvent.change(screen.getByLabelText(/email/i), {
      target: { value: 'test@example.com' },
    });
    fireEvent.change(screen.getByLabelText(/password/i), {
      target: { value: 'password123' },
    });
    
    fireEvent.click(screen.getByRole('button', { name: /sign in/i }));
    
    await waitFor(() => {
      expect(mockEventBus.emit).toHaveBeenCalledWith({
        type: 'auth.login',
        source: 'auth-micro-app',
        payload: expect.objectContaining({
          user: expect.any(Object),
        }),
        timestamp: expect.any(Number),
      });
    });
  });
});

// Integration test for shell application
// shell/src/__tests__/integration/MicroFrontendIntegration.test.tsx
import { render, screen, waitFor } from '@testing-library/react';
import { BrowserRouter } from 'react-router-dom';
import { ShellApp } from '../ShellApp';
import { createMockMicroFrontends } from '@adx-core/test-utils';

// Mock micro-frontends for testing
vi.mock('@/config/microFrontends', () => ({
  microFrontendRegistry: createMockMicroFrontends(),
}));

describe('Micro-Frontend Integration', () => {
  it('should load and render micro-frontends correctly', async () => {
    render(
      <BrowserRouter>
        <ShellApp />
      </BrowserRouter>
    );
    
    // Wait for micro-frontends to load
    await waitFor(() => {
      expect(screen.getByTestId('auth-micro-app')).toBeInTheDocument();
      expect(screen.getByTestId('file-micro-app')).toBeInTheDocument();
    });
  });
  
  it('should handle micro-frontend loading errors gracefully', async () => {
    // Simulate loading error
    vi.mocked(import).mockRejectedValueOnce(new Error('Failed to load'));
    
    render(
      <BrowserRouter>
        <ShellApp />
      </BrowserRouter>
    );
    
    await waitFor(() => {
      expect(screen.getByText(/failed to load/i)).toBeInTheDocument();
    });
  });
});

// E2E test configuration
// e2e/playwright.config.ts
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './tests',
  fullyParallel: true,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 1 : undefined,
  reporter: 'html',
  
  use: {
    baseURL: 'http://localhost:3000',
    trace: 'on-first-retry',
  },
  
  projects: [
    {
      name: 'chromium',
      use: { ...devices['Desktop Chrome'] },
    },
    {
      name: 'webkit',
      use: { ...devices['Desktop Safari'] },
    },
    {
      name: 'mobile-chrome',
      use: { ...devices['Pixel 5'] },
    },
  ],
  
  webServer: [
    {
      command: 'npm run dev:shell',
      port: 3000,
    },
    {
      command: 'npm run dev:micro-frontends',
      port: 3001,
    },
  ],
});

// E2E test example
// e2e/tests/cross-micro-frontend-workflow.spec.ts
import { test, expect } from '@playwright/test';

test('complete user workflow across micro-frontends', async ({ page }) => {
  // Start at login (auth micro-frontend)
  await page.goto('/auth/login');
  
  await page.fill('[data-testid="email-input"]', 'test@example.com');
  await page.fill('[data-testid="password-input"]', 'password123');
  await page.click('[data-testid="login-button"]');
  
  // Should navigate to dashboard after login
  await expect(page).toHaveURL('/dashboard');
  
  // Navigate to files (file micro-frontend)
  await page.click('[data-testid="nav-files"]');
  await expect(page).toHaveURL('/files');
  
  // Upload a file
  await page.setInputFiles('[data-testid="file-upload"]', 'test-file.txt');
  await page.click('[data-testid="upload-button"]');
  
  // Verify file appears in list
  await expect(page.locator('[data-testid="file-list"]')).toContainText('test-file.txt');
  
  // Switch tenant (tenant micro-frontend)
  await page.click('[data-testid="tenant-switcher"]');
  await page.click('[data-testid="tenant-option-2"]');
  
  // Verify tenant switch worked
  await expect(page.locator('[data-testid="current-tenant"]')).toContainText('Tenant 2');
});
```

This comprehensive design document provides a complete architecture for transforming ADX CORE's frontend into a microservices architecture while maintaining our existing technology stack and cross-platform capabilities. The design emphasizes practical implementation details, team autonomy, and seamless integration with our Rust backend services.

<function_calls>
<invoke name="userInput">
<parameter name="question">**Does the design look good? If so, we can move on to the implementation plan.**