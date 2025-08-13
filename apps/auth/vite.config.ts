import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'auth_app',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './LoginForm': './src/components/LoginForm.tsx',
        './RegisterForm': './src/components/RegisterForm.tsx',
        './MFASetup': './src/components/MFASetup.tsx',
        './SSOLogin': './src/components/SSOLogin.tsx',
        './AuthProvider': './src/providers/AuthProvider.tsx',
        './ProtectedRoute': './src/components/ProtectedRoute.tsx',
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
    port: 3001,
    cors: true,
    host: true
  },
  preview: {
    port: 3001,
    cors: true,
    host: true
  }
});