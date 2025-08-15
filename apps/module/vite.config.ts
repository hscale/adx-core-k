import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'module_app',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './ModuleMarketplace': './src/components/ModuleMarketplace.tsx',
        './ModuleManager': './src/components/ModuleManager.tsx',
        './ModuleSettings': './src/components/ModuleSettings.tsx',
        './ModuleDeveloper': './src/components/ModuleDeveloper.tsx',
      },
      shared: {
        react: { singleton: true },
        'react-dom': { singleton: true },
        '@tanstack/react-query': { singleton: true },
        '@adx-core/design-system': { singleton: true },
        '@adx-core/shared-context': { singleton: true },
        '@adx-core/event-bus': { singleton: true },
      },
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
  },
  server: {
    port: 3006,
    cors: true,
  },
});