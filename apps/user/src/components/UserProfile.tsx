import React, { useState } from 'react';
import { Button, Card, Input } from '@adx-core/design-system';
import { User, Edit3, Save, X, Mail, Phone, MapPin, Globe, Linkedin, Twitter, Github } from 'lucide-react';
import { useUser, useUserProfile, useUpdateUser, useUpdateUserProfile } from '../hooks';
import { formatUserDisplayName, formatUserInitials } from '../utils';
import { UpdateUserRequest, UpdateUserProfileRequest } from '../types';

interface UserProfileProps {
  userId: string;
  editable?: boolean;
}

export const UserProfile: React.FC<UserProfileProps> = ({ userId, editable = false }) => {
  const [isEditing, setIsEditing] = useState(false);
  const [editingProfile, setEditingProfile] = useState(false);

  const { data: user, isLoading: userLoading } = useUser(userId);
  const { data: profile, isLoading: profileLoading } = useUserProfile(userId);
  const updateUserMutation = useUpdateUser();
  const updateProfileMutation = useUpdateUserProfile();

  const [userForm, setUserForm] = useState<UpdateUserRequest>({});
  const [profileForm, setProfileForm] = useState<UpdateUserProfileRequest>({});

  const isLoading = userLoading || profileLoading;

  const handleEditUser = () => {
    if (user) {
      setUserForm({
        firstName: user.firstName,
        lastName: user.lastName,
        displayName: user.displayName,
        phone: user.phone || '',
        timezone: user.timezone,
        language: user.language,
      });
      setIsEditing(true);
    }
  };

  const handleEditProfile = () => {
    if (profile) {
      setProfileForm({
        bio: profile.bio || '',
        department: profile.department || '',
        jobTitle: profile.jobTitle || '',
        location: profile.location || '',
        website: profile.website || '',
        socialLinks: {
          linkedin: profile.socialLinks?.linkedin || '',
          twitter: profile.socialLinks?.twitter || '',
          github: profile.socialLinks?.github || '',
        },
      });
      setEditingProfile(true);
    }
  };

  const handleSaveUser = async () => {
    try {
      await updateUserMutation.mutateAsync({ userId, updates: userForm });
      setIsEditing(false);
      setUserForm({});
    } catch (error) {
      console.error('Failed to update user:', error);
    }
  };

  const handleSaveProfile = async () => {
    try {
      await updateProfileMutation.mutateAsync({ userId, updates: profileForm });
      setEditingProfile(false);
      setProfileForm({});
    } catch (error) {
      console.error('Failed to update profile:', error);
    }
  };

  const handleCancelUser = () => {
    setIsEditing(false);
    setUserForm({});
  };

  const handleCancelProfile = () => {
    setEditingProfile(false);
    setProfileForm({});
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!user) {
    return (
      <Card className="p-6">
        <div className="text-center text-gray-500">
          User not found
        </div>
      </Card>
    );
  }

  return (
    <div className="space-y-6">
      {/* User Basic Info */}
      <Card className="p-6">
        <div className="flex items-start justify-between mb-6">
          <div className="flex items-center space-x-4">
            <div className="w-16 h-16 bg-blue-600 rounded-full flex items-center justify-center text-white text-xl font-semibold">
              {user.avatar ? (
                <img src={user.avatar} alt={formatUserDisplayName(user)} className="w-16 h-16 rounded-full" />
              ) : (
                formatUserInitials(user)
              )}
            </div>
            <div>
              <h2 className="text-2xl font-bold text-gray-900">
                {formatUserDisplayName(user)}
              </h2>
              <p className="text-gray-600">{user.email}</p>
              <div className="flex items-center space-x-2 mt-1">
                <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                  user.isActive 
                    ? 'bg-green-100 text-green-800' 
                    : 'bg-red-100 text-red-800'
                }`}>
                  {user.isActive ? 'Active' : 'Inactive'}
                </span>
              </div>
            </div>
          </div>
          {editable && (
            <Button
              variant="outline"
              size="sm"
              onClick={handleEditUser}
              disabled={isEditing}
            >
              <Edit3 className="w-4 h-4 mr-2" />
              Edit
            </Button>
          )}
        </div>

        {isEditing ? (
          <div className="space-y-4">
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="First Name"
                value={userForm.firstName || ''}
                onChange={(e) => setUserForm({ ...userForm, firstName: e.target.value })}
                placeholder="Enter first name"
              />
              <Input
                label="Last Name"
                value={userForm.lastName || ''}
                onChange={(e) => setUserForm({ ...userForm, lastName: e.target.value })}
                placeholder="Enter last name"
              />
            </div>
            <Input
              label="Display Name"
              value={userForm.displayName || ''}
              onChange={(e) => setUserForm({ ...userForm, displayName: e.target.value })}
              placeholder="Enter display name"
            />
            <div className="grid grid-cols-2 gap-4">
              <Input
                label="Phone"
                value={userForm.phone || ''}
                onChange={(e) => setUserForm({ ...userForm, phone: e.target.value })}
                placeholder="Enter phone number"
              />
              <Input
                label="Timezone"
                value={userForm.timezone || ''}
                onChange={(e) => setUserForm({ ...userForm, timezone: e.target.value })}
                placeholder="Enter timezone"
              />
            </div>
            <div className="flex space-x-2">
              <Button
                onClick={handleSaveUser}
                disabled={updateUserMutation.isPending}
              >
                <Save className="w-4 h-4 mr-2" />
                Save
              </Button>
              <Button
                variant="outline"
                onClick={handleCancelUser}
              >
                <X className="w-4 h-4 mr-2" />
                Cancel
              </Button>
            </div>
          </div>
        ) : (
          <div className="grid grid-cols-2 gap-6">
            <div className="space-y-3">
              <div className="flex items-center space-x-2">
                <Mail className="w-4 h-4 text-gray-400" />
                <span className="text-sm text-gray-600">Email:</span>
                <span className="text-sm font-medium">{user.email}</span>
              </div>
              {user.phone && (
                <div className="flex items-center space-x-2">
                  <Phone className="w-4 h-4 text-gray-400" />
                  <span className="text-sm text-gray-600">Phone:</span>
                  <span className="text-sm font-medium">{user.phone}</span>
                </div>
              )}
              <div className="flex items-center space-x-2">
                <User className="w-4 h-4 text-gray-400" />
                <span className="text-sm text-gray-600">Roles:</span>
                <span className="text-sm font-medium">{user.roles.join(', ')}</span>
              </div>
            </div>
            <div className="space-y-3">
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-600">Timezone:</span>
                <span className="text-sm font-medium">{user.timezone}</span>
              </div>
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-600">Language:</span>
                <span className="text-sm font-medium">{user.language}</span>
              </div>
              <div className="flex items-center space-x-2">
                <span className="text-sm text-gray-600">Last Login:</span>
                <span className="text-sm font-medium">
                  {user.lastLoginAt ? new Date(user.lastLoginAt).toLocaleDateString() : 'Never'}
                </span>
              </div>
            </div>
          </div>
        )}
      </Card>

      {/* User Profile Details */}
      {profile && (
        <Card className="p-6">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900">Profile Details</h3>
            {editable && (
              <Button
                variant="outline"
                size="sm"
                onClick={handleEditProfile}
                disabled={editingProfile}
              >
                <Edit3 className="w-4 h-4 mr-2" />
                Edit
              </Button>
            )}
          </div>

          {editingProfile ? (
            <div className="space-y-4">
              <div className="grid grid-cols-2 gap-4">
                <Input
                  label="Job Title"
                  value={profileForm.jobTitle || ''}
                  onChange={(e) => setProfileForm({ ...profileForm, jobTitle: e.target.value })}
                  placeholder="Enter job title"
                />
                <Input
                  label="Department"
                  value={profileForm.department || ''}
                  onChange={(e) => setProfileForm({ ...profileForm, department: e.target.value })}
                  placeholder="Enter department"
                />
              </div>
              <Input
                label="Location"
                value={profileForm.location || ''}
                onChange={(e) => setProfileForm({ ...profileForm, location: e.target.value })}
                placeholder="Enter location"
              />
              <Input
                label="Website"
                value={profileForm.website || ''}
                onChange={(e) => setProfileForm({ ...profileForm, website: e.target.value })}
                placeholder="Enter website URL"
              />
              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-700">Bio</label>
                <textarea
                  className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                  rows={3}
                  value={profileForm.bio || ''}
                  onChange={(e) => setProfileForm({ ...profileForm, bio: e.target.value })}
                  placeholder="Tell us about yourself..."
                />
              </div>
              <div className="space-y-2">
                <label className="block text-sm font-medium text-gray-700">Social Links</label>
                <div className="space-y-2">
                  <Input
                    placeholder="LinkedIn URL"
                    value={profileForm.socialLinks?.linkedin || ''}
                    onChange={(e) => setProfileForm({
                      ...profileForm,
                      socialLinks: { ...profileForm.socialLinks, linkedin: e.target.value }
                    })}
                  />
                  <Input
                    placeholder="Twitter URL"
                    value={profileForm.socialLinks?.twitter || ''}
                    onChange={(e) => setProfileForm({
                      ...profileForm,
                      socialLinks: { ...profileForm.socialLinks, twitter: e.target.value }
                    })}
                  />
                  <Input
                    placeholder="GitHub URL"
                    value={profileForm.socialLinks?.github || ''}
                    onChange={(e) => setProfileForm({
                      ...profileForm,
                      socialLinks: { ...profileForm.socialLinks, github: e.target.value }
                    })}
                  />
                </div>
              </div>
              <div className="flex space-x-2">
                <Button
                  onClick={handleSaveProfile}
                  disabled={updateProfileMutation.isPending}
                >
                  <Save className="w-4 h-4 mr-2" />
                  Save
                </Button>
                <Button
                  variant="outline"
                  onClick={handleCancelProfile}
                >
                  <X className="w-4 h-4 mr-2" />
                  Cancel
                </Button>
              </div>
            </div>
          ) : (
            <div className="space-y-4">
              {profile.bio && (
                <div>
                  <h4 className="text-sm font-medium text-gray-700 mb-1">Bio</h4>
                  <p className="text-sm text-gray-600">{profile.bio}</p>
                </div>
              )}
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-3">
                  {profile.jobTitle && (
                    <div className="flex items-center space-x-2">
                      <span className="text-sm text-gray-600">Job Title:</span>
                      <span className="text-sm font-medium">{profile.jobTitle}</span>
                    </div>
                  )}
                  {profile.department && (
                    <div className="flex items-center space-x-2">
                      <span className="text-sm text-gray-600">Department:</span>
                      <span className="text-sm font-medium">{profile.department}</span>
                    </div>
                  )}
                  {profile.location && (
                    <div className="flex items-center space-x-2">
                      <MapPin className="w-4 h-4 text-gray-400" />
                      <span className="text-sm text-gray-600">Location:</span>
                      <span className="text-sm font-medium">{profile.location}</span>
                    </div>
                  )}
                </div>
                <div className="space-y-3">
                  {profile.website && (
                    <div className="flex items-center space-x-2">
                      <Globe className="w-4 h-4 text-gray-400" />
                      <span className="text-sm text-gray-600">Website:</span>
                      <a
                        href={profile.website}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="text-sm font-medium text-blue-600 hover:text-blue-800"
                      >
                        {profile.website}
                      </a>
                    </div>
                  )}
                  {(profile.socialLinks?.linkedin || profile.socialLinks?.twitter || profile.socialLinks?.github) && (
                    <div>
                      <span className="text-sm text-gray-600 block mb-2">Social Links:</span>
                      <div className="flex space-x-3">
                        {profile.socialLinks?.linkedin && (
                          <a
                            href={profile.socialLinks.linkedin}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-600 hover:text-blue-800"
                          >
                            <Linkedin className="w-4 h-4" />
                          </a>
                        )}
                        {profile.socialLinks?.twitter && (
                          <a
                            href={profile.socialLinks.twitter}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-600 hover:text-blue-800"
                          >
                            <Twitter className="w-4 h-4" />
                          </a>
                        )}
                        {profile.socialLinks?.github && (
                          <a
                            href={profile.socialLinks.github}
                            target="_blank"
                            rel="noopener noreferrer"
                            className="text-blue-600 hover:text-blue-800"
                          >
                            <Github className="w-4 h-4" />
                          </a>
                        )}
                      </div>
                    </div>
                  )}
                </div>
              </div>
            </div>
          )}
        </Card>
      )}
    </div>
  );
};

export default UserProfile;