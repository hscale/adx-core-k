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
        './WorkflowList': './src/components/WorkflowList.tsx',
        './WorkflowStatus': './src/components/WorkflowStatus.tsx',
        './WorkflowHistory': './src/components/WorkflowHistory.tsx',
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
  server: { port: 3005, cors: true, host: true },
  build: { target: 'esnext', minify: false, cssCodeSplit: false },
});