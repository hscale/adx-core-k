import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'tenant_app',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './TenantSwitcher': './src/components/TenantSwitcher.tsx',
        './TenantSettings': './src/components/TenantSettings.tsx',
        './TenantMembers': './src/components/TenantMembers.tsx',
        './TenantBilling': './src/components/TenantBilling.tsx',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        '@tanstack/react-query': { singleton: true },
        '@adx-core/design-system': { singleton: true },
        '@adx-core/shared-context': { singleton: true },
        '@adx-core/event-bus': { singleton: true },
        '@adx-core/bff-client': { singleton: true },
      },
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
  },
  server: {
    port: 3002,
    cors: true,
    host: true,
  },
  preview: {
    port: 3002,
    cors: true,
    host: true,
  },
  define: {
    'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV || 'development'),
  },
});