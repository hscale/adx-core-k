import React from 'react';
import { useParams } from 'react-router-dom';
import { UserProfile } from '../components';
import { useUserContext } from '../providers';

export const UserProfilePage: React.FC = () => {
  const { userId } = useParams<{ userId: string }>();
  const { currentUser } = useUserContext();

  // If no userId in params, show current user's profile
  const targetUserId = userId || currentUser?.id;
  const isOwnProfile = !userId || userId === currentUser?.id;

  if (!targetUserId) {
    return (
      <div className="flex items-center justify-center min-h-[400px]">
        <div className="text-center">
          <h2 className="text-xl font-semibold text-gray-900 mb-2">User not found</h2>
          <p className="text-gray-600">The requested user profile could not be found.</p>
        </div>
      </div>
    );
  }

  return (
    <div className="max-w-4xl mx-auto p-6">
      <div className="mb-6">
        <h1 className="text-3xl font-bold text-gray-900">
          {isOwnProfile ? 'My Profile' : 'User Profile'}
        </h1>
        <p className="text-gray-600 mt-2">
          {isOwnProfile 
            ? 'Manage your personal information and preferences' 
            : 'View user information and details'
          }
        </p>
      </div>
      
      <UserProfile userId={targetUserId} editable={isOwnProfile} />
    </div>
  );
};

export default UserProfilePage;