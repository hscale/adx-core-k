import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'workflow_app',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './WorkflowMonitor': './src/components/WorkflowMonitor.tsx',
        './WorkflowHistory': './src/components/WorkflowHistory.tsx',
        './WorkflowAnalytics': './src/components/WorkflowAnalytics.tsx',
        './WorkflowManagement': './src/components/WorkflowManagement.tsx',
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
    port: 3005,
    cors: true,
  },
});