import React from 'react';
import ReactDOM from 'react-dom/client';
import { App } from './App';
import '@adx-core/design-system/styles';
import './index.css';

// Initialize mock authentication state for development
if (process.env.NODE_ENV === 'development') {
  const mockUser = {
    id: 'user-1',
    email: 'admin@adxcore.com',
    name: 'Admin User',
    avatar: undefined,
    roles: ['admin', 'user'],
    permissions: ['*'],
  };

  const mockToken = 'mock-jwt-token-for-development';

  // Set initial auth state
  setTimeout(() => {
    const { useAuthStore } = require('@adx-core/shared-context');
    useAuthStore.getState().login(mockUser, mockToken);
  }, 100);
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>
);