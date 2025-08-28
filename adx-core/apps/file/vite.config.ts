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
        './FileViewer': './src/components/FileViewer.tsx',
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
  server: { port: 3003, cors: true, host: true },
  build: { target: 'esnext', minify: false, cssCodeSplit: false },
});