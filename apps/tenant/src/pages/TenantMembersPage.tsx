import React from 'react';
import { TenantMembership, TenantInvitation } from '../components';

interface TenantMembersPageProps {
  className?: string;
}

export const TenantMembersPage: React.FC<TenantMembersPageProps> = ({
  className = '',
}) => {
  return (
    <div className={`space-y-6 ${className}`}>
      <div>
        <h1 className="text-2xl font-bold text-gray-900">Team Management</h1>
        <p className="text-gray-500">
          Manage your team members and invitations
        </p>
      </div>
      
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <TenantMembership />
        <TenantInvitation />
      </div>
    </div>
  );
};