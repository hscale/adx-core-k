import React from 'react';
import { TenantSettings } from '../components';

interface TenantSettingsPageProps {
  className?: string;
}

export const TenantSettingsPage: React.FC<TenantSettingsPageProps> = ({
  className = '',
}) => {
  return (
    <div className={`space-y-6 ${className}`}>
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Tenant Settings</h1>
        <p className="text-gray-500">
          Manage your tenant configuration and preferences
        </p>
      </div>
      
      <TenantSettings />
    </div>
  );
};