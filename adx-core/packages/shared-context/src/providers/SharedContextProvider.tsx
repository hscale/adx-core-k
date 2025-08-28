import React from 'react';
import { TenantProvider } from './TenantProvider';
import { UserProvider } from './UserProvider';
import { AuthProvider } from './AuthProvider';

interface SharedContextProviderProps {
  children: React.ReactNode;
}

export const SharedContextProvider: React.FC<SharedContextProviderProps> = ({ children }) => {
  return (
    <AuthProvider>
      <UserProvider>
        <TenantProvider>
          {children}
        </TenantProvider>
      </UserProvider>
    </AuthProvider>
  );
};