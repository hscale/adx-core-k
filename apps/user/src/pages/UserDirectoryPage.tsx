import React from 'react';
import { UserDirectory } from '../components';
import { useUserContext } from '../providers';

export const UserDirectoryPage: React.FC = () => {
  const { currentUser } = useUserContext();

  // Check if user has permission to manage users
  const canManageUsers = currentUser?.roles.includes('admin') || 
                        currentUser?.roles.includes('manager') ||
                        currentUser?.permissions.includes('user:manage');

  return (
    <div className="max-w-7xl mx-auto p-6">
      <UserDirectory canManageUsers={canManageUsers} />
    </div>
  );
};

export default UserDirectoryPage;