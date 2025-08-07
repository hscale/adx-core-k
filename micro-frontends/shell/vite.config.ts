import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { federation } from '@originjs/vite-plugin-federation'
import { resolve } from 'path'

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
        dashboardMicroApp: 'http://localhost:3006/assets/remoteEntry.js',
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
    port: 3000,
    cors: true,
    host: true,
  },
})