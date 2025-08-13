import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'shell_app',
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
          requiredVersion: '^18.2.0'
        },
        'react-dom': { 
          singleton: true,
          requiredVersion: '^18.2.0'
        },
        '@tanstack/react-query': { 
          singleton: true,
          requiredVersion: '^5.8.4'
        },
        'react-router-dom': {
          singleton: true,
          requiredVersion: '^6.20.1'
        },
        'react-i18next': {
          singleton: true,
          requiredVersion: '^13.5.0'
        },
        'zustand': {
          singleton: true,
          requiredVersion: '^4.4.7'
        }
      },
    }),
  ],
  build: {
    target: 'esnext',
    minify: false,
    cssCodeSplit: false,
    rollupOptions: {
      external: ['react', 'react-dom']
    }
  },
  server: {
    port: 3000,
    cors: true,
    host: true
  },
  preview: {
    port: 3000,
    cors: true,
    host: true
  }
});