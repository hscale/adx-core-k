import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import { federation } from '@originjs/vite-plugin-federation'
import { resolve } from 'path'

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'tenantMicroApp',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './routes': './src/routes.tsx',
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
    port: 3002,
    cors: true,
    host: true,
  },
})