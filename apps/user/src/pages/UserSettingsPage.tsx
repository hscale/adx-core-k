import React from 'react';
import { useParams } from 'react-router-dom';
import { UserSettings } from '../components';
import { useUserContext } from '../providers';

export const UserSettingsPage: React.FC = () => {
  const { userId } = useParams<{ userId: string }>();
  const { currentUser } = useUserContext();

  // If no userId in params, show current user's settings
  const targetUserId = userId || currentUser?.id;
  const isOwnSettings = !userId || userId === currentUser?.id;

  if (!targetUserId) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-gray-900 mb-2">User not found</h2>
          <p className="text-gray-600">The requested user settings could not be found.</p>
        </div>
      </div>
    );
  }

  // Only allow users to view their own settings for security
  if (!isOwnSettings) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-gray-900 mb-2">Access Denied</h2>
          <p className="text-gray-600">You can only view your own settings.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900">User Settings</h1>
        <p className="text-gray-600 mt-2">
          Manage your preferences, security settings, and account quotas
        </p>
      </div>
      
      <UserSettings userId={targetUserId} />
    </div>
  );
};

export default UserSettingsPage;