import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import federation from '@originjs/vite-plugin-federation';

export default defineConfig({
  plugins: [
    react(),
    federation({
      name: 'user_app',
      filename: 'remoteEntry.js',
      exposes: {
        './App': './src/App.tsx',
        './UserProfile': './src/components/UserProfile.tsx',
        './UserSettings': './src/components/UserSettings.tsx',
        './UserDirectory': './src/components/UserDirectory.tsx',
        './UserProvider': './src/providers/UserProvider.tsx',
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
    port: 3004,
    cors: true,
  },
});