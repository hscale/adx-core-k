import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'file_app',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './FileUpload': './src/components/FileUpload.tsx',
        './FileBrowser': './src/components/FileBrowser.tsx',
        './FileManager': './src/components/FileManager.tsx',
        './FileSharing': './src/components/FileSharing.tsx',
        './FilePermissions': './src/components/FilePermissions.tsx',
        './FileProvider': './src/providers/FileProvider.tsx',
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
        },
        '@adx-core/shared-context': {
          singleton: true
        },
        '@adx-core/event-bus': {
          singleton: true
        },
        '@adx-core/design-system': {
          singleton: true
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
    port: 3003,
    cors: true,
    host: true
  },
  preview: {
    port: 3003,
    cors: true,
    host: true
  }
});