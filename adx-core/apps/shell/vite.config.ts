import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';
import path from 'path';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'shell',
      remotes: {
        auth_app: 'http://localhost:3001/assets/remoteEntry.js',
        tenant_app: 'http://localhost:3002/assets/remoteEntry.js',
        file_app: 'http://localhost:3003/assets/remoteEntry.js',
        user_app: 'http://localhost:3004/assets/remoteEntry.js',
        workflow_app: 'http://localhost:3005/assets/remoteEntry.js',
        module_app: 'http://localhost:3006/assets/remoteEntry.js',
      },
      shared: {
        react: {
          singleton: true,
          requiredVersion: '^18.2.0',
        },
        'react-dom': {
          singleton: true,
          requiredVersion: '^18.2.0',
        },
        'react-router-dom': {
          singleton: true,
          requiredVersion: '^6.8.1',
        },
        '@tanstack/react-query': {
          singleton: true,
          requiredVersion: '^4.24.6',
        },
        zustand: {
          singleton: true,
          requiredVersion: '^4.3.2',
        },
        '@adx-core/shared-context': {
          singleton: true,
        },
        '@adx-core/design-system': {
          singleton: true,
        },
        '@adx-core/event-bus': {
          singleton: true,
        },
        '@adx-core/i18n': {
          singleton: true,
        },
      },
    }),
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  server: {
    port: 3000,
    cors: true,
  },
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
  },
});