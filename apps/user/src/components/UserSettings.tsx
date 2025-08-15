import React, { useState } from 'react';
import { Button, Card, Input } from '@adx-core/design-system';
import { Settings, Save, X, Shield, Bell, Globe, Clock } from 'lucide-react';
import { useUserSettings, useUpdateUserPreferences } from '../hooks';
import { formatQuotaUsage } from '../utils';
import { UpdateUserPreferencesRequest } from '../types';

interface UserSettingsProps {
  userId: string;
}

export const UserSettings: React.FC<UserSettingsProps> = ({ userId }) => {
  const [activeTab, setActiveTab] = useState<'preferences' | 'security' | 'quotas'>('preferences');
  const [isEditing, setIsEditing] = useState(false);

  const { data: settings, isLoading } = useUserSettings(userId);
  const updatePreferencesMutation = useUpdateUserPreferences();

  const [preferencesForm, setPreferencesForm] = useState<UpdateUserPreferencesRequest>({});

  const handleEditPreferences = () => {
    if (settings) {
      setPreferencesForm({
        theme: settings.quotas ? 'light' : 'dark', // This should come from actual preferences
        language: 'en',
        timezone: 'UTC',
        dateFormat: 'MM/DD/YYYY',
        timeFormat: '12h',
        notifications: {
          email: true,
          push: true,
          desktop: true,
          workflow: true,
          mentions: true,
        },
        privacy: {
          profileVisibility: 'public',
          showOnlineStatus: true,
          allowDirectMessages: true,
        },
      });
      setIsEditing(true);
    }
  };

  const handleSavePreferences = async () => {
    try {
      await updatePreferencesMutation.mutateAsync({ userId, updates: preferencesForm });
      setIsEditing(false);
      setPreferencesForm({});
    } catch (error) {
      console.error('Failed to update preferences:', error);
    }
  };

  const handleCancelPreferences = () => {
    setIsEditing(false);
    setPreferencesForm({});
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center p-8">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (!settings) {
    return (
      <Card className="p-6">
        <div className="text-center text-gray-500">
          Settings not found
        </div>
      </Card>
    );
  }

  const tabs = [
    { id: 'preferences', label: 'Preferences', icon: Settings },
    { id: 'security', label: 'Security', icon: Shield },
    { id: 'quotas', label: 'Quotas', icon: Globe },
  ] as const;

  return (
    <div className="space-y-6">
      {/* Tab Navigation */}
      <div className="border-b border-gray-200">
        <nav className="-mb-px flex space-x-8">
          {tabs.map((tab) => {
            const Icon = tab.icon;
            return (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`flex items-center space-x-2 py-2 px-1 border-b-2 font-medium text-sm ${
                  activeTab === tab.id
                    ? 'border-blue-500 text-blue-600'
                    : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
                }`}
              >
                <Icon className="w-4 h-4" />
                <span>{tab.label}</span>
              </button>
            );
          })}
        </nav>
      </div>

      {/* Preferences Tab */}
      {activeTab === 'preferences' && (
        <Card className="p-6">
          <div className="flex items-center justify-between mb-6">
            <h3 className="text-lg font-semibold text-gray-900">User Preferences</h3>
            <Button
              variant="outline"
              size="sm"
              onClick={handleEditPreferences}
              disabled={isEditing}
            >
              <Settings className="w-4 h-4 mr-2" />
              Edit
            </Button>
          </div>

          {isEditing ? (
            <div className="space-y-6">
              {/* Theme Settings */}
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-3">Appearance</h4>
                <div className="space-y-3">
                  <div>
                    <label className="block text-sm text-gray-600 mb-1">Theme</label>
                    <select
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={preferencesForm.theme || 'light'}
                      onChange={(e) => setPreferencesForm({
                        ...preferencesForm,
                        theme: e.target.value as 'light' | 'dark' | 'system'
                      })}
                    >
                      <option value="light">Light</option>
                      <option value="dark">Dark</option>
                      <option value="system">System</option>
                    </select>
                  </div>
                  <div className="grid grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm text-gray-600 mb-1">Language</label>
                      <select
                        className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                        value={preferencesForm.language || 'en'}
                        onChange={(e) => setPreferencesForm({ ...preferencesForm, language: e.target.value })}
                      >
                        <option value="en">English</option>
                        <option value="es">Spanish</option>
                        <option value="fr">French</option>
                        <option value="de">German</option>
                      </select>
                    </div>
                    <div>
                      <label className="block text-sm text-gray-600 mb-1">Timezone</label>
                      <Input
                        value={preferencesForm.timezone || ''}
                        onChange={(e) => setPreferencesForm({ ...preferencesForm, timezone: e.target.value })}
                        placeholder="Enter timezone"
                      />
                    </div>
                  </div>
                </div>
              </div>

              {/* Date & Time Settings */}
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-3">Date & Time</h4>
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-sm text-gray-600 mb-1">Date Format</label>
                    <select
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={preferencesForm.dateFormat || 'MM/DD/YYYY'}
                      onChange={(e) => setPreferencesForm({ ...preferencesForm, dateFormat: e.target.value })}
                    >
                      <option value="MM/DD/YYYY">MM/DD/YYYY</option>
                      <option value="DD/MM/YYYY">DD/MM/YYYY</option>
                      <option value="YYYY-MM-DD">YYYY-MM-DD</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-sm text-gray-600 mb-1">Time Format</label>
                    <select
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={preferencesForm.timeFormat || '12h'}
                      onChange={(e) => setPreferencesForm({
                        ...preferencesForm,
                        timeFormat: e.target.value as '12h' | '24h'
                      })}
                    >
                      <option value="12h">12 Hour</option>
                      <option value="24h">24 Hour</option>
                    </select>
                  </div>
                </div>
              </div>

              {/* Notification Settings */}
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-3">Notifications</h4>
                <div className="space-y-3">
                  {[
                    { key: 'email', label: 'Email Notifications', icon: Bell },
                    { key: 'push', label: 'Push Notifications', icon: Bell },
                    { key: 'desktop', label: 'Desktop Notifications', icon: Bell },
                    { key: 'workflow', label: 'Workflow Updates', icon: Bell },
                    { key: 'mentions', label: 'Mentions & Messages', icon: Bell },
                  ].map((notification) => (
                    <label key={notification.key} className="flex items-center space-x-3">
                      <input
                        type="checkbox"
                        checked={preferencesForm.notifications?.[notification.key as keyof typeof preferencesForm.notifications] || false}
                        onChange={(e) => setPreferencesForm({
                          ...preferencesForm,
                          notifications: {
                            ...preferencesForm.notifications,
                            [notification.key]: e.target.checked,
                          },
                        })}
                        className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                      />
                      <span className="text-sm text-gray-700">{notification.label}</span>
                    </label>
                  ))}
                </div>
              </div>

              {/* Privacy Settings */}
              <div>
                <h4 className="text-sm font-medium text-gray-700 mb-3">Privacy</h4>
                <div className="space-y-3">
                  <div>
                    <label className="block text-sm text-gray-600 mb-1">Profile Visibility</label>
                    <select
                      className="w-full px-3 py-2 border border-gray-300 rounded-md focus:outline-none focus:ring-2 focus:ring-blue-500"
                      value={preferencesForm.privacy?.profileVisibility || 'public'}
                      onChange={(e) => setPreferencesForm({
                        ...preferencesForm,
                        privacy: {
                          ...preferencesForm.privacy,
                          profileVisibility: e.target.value as 'public' | 'team' | 'private',
                        },
                      })}
                    >
                      <option value="public">Public</option>
                      <option value="team">Team Only</option>
                      <option value="private">Private</option>
                    </select>
                  </div>
                  <label className="flex items-center space-x-3">
                    <input
                      type="checkbox"
                      checked={preferencesForm.privacy?.showOnlineStatus || false}
                      onChange={(e) => setPreferencesForm({
                        ...preferencesForm,
                        privacy: {
                          ...preferencesForm.privacy,
                          showOnlineStatus: e.target.checked,
                        },
                      })}
                      className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-700">Show online status</span>
                  </label>
                  <label className="flex items-center space-x-3">
                    <input
                      type="checkbox"
                      checked={preferencesForm.privacy?.allowDirectMessages || false}
                      onChange={(e) => setPreferencesForm({
                        ...preferencesForm,
                        privacy: {
                          ...preferencesForm.privacy,
                          allowDirectMessages: e.target.checked,
                        },
                      })}
                      className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-700">Allow direct messages</span>
                  </label>
                </div>
              </div>

              <div className="flex space-x-2">
                <Button
                  onClick={handleSavePreferences}
                  disabled={updatePreferencesMutation.isPending}
                >
                  <Save className="w-4 h-4 mr-2" />
                  Save
                </Button>
                <Button
                  variant="outline"
                  onClick={handleCancelPreferences}
                >
                  <X className="w-4 h-4 mr-2" />
                  Cancel
                </Button>
              </div>
            </div>
          ) : (
            <div className="space-y-6">
              <div className="text-center text-gray-500">
                <Settings className="w-12 h-12 mx-auto mb-4 text-gray-300" />
                <p>Click "Edit" to modify your preferences</p>
              </div>
            </div>
          )}
        </Card>
      )}

      {/* Security Tab */}
      {activeTab === 'security' && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-6">Security Settings</h3>
          <div className="space-y-6">
            <div className="flex items-center justify-between p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center space-x-3">
                <Shield className="w-5 h-5 text-green-600" />
                <div>
                  <h4 className="text-sm font-medium text-gray-900">Multi-Factor Authentication</h4>
                  <p className="text-sm text-gray-600">
                    {settings.security.mfaEnabled ? 'Enabled' : 'Disabled'}
                  </p>
                </div>
              </div>
              <Button variant="outline" size="sm">
                {settings.security.mfaEnabled ? 'Disable' : 'Enable'}
              </Button>
            </div>

            <div className="p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center space-x-3 mb-3">
                <Clock className="w-5 h-5 text-blue-600" />
                <h4 className="text-sm font-medium text-gray-900">Session Timeout</h4>
              </div>
              <p className="text-sm text-gray-600 mb-2">
                Current timeout: {settings.security.sessionTimeout} minutes
              </p>
              <Button variant="outline" size="sm">
                Change
              </Button>
            </div>

            {settings.security.allowedIpRanges && (
              <div className="p-4 border border-gray-200 rounded-lg">
                <div className="flex items-center space-x-3 mb-3">
                  <Globe className="w-5 h-5 text-orange-600" />
                  <h4 className="text-sm font-medium text-gray-900">IP Restrictions</h4>
                </div>
                <p className="text-sm text-gray-600 mb-2">
                  {settings.security.allowedIpRanges.length} IP range(s) configured
                </p>
                <Button variant="outline" size="sm">
                  Manage
                </Button>
              </div>
            )}
          </div>
        </Card>
      )}

      {/* Quotas Tab */}
      {activeTab === 'quotas' && (
        <Card className="p-6">
          <h3 className="text-lg font-semibold text-gray-900 mb-6">Usage & Quotas</h3>
          <div className="space-y-6">
            {/* Storage Quota */}
            <div className="p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-sm font-medium text-gray-900">Storage</h4>
                <span className="text-sm text-gray-600">
                  {formatQuotaUsage(settings.quotas.storageUsed, settings.quotas.storageLimit)} GB
                </span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className={`h-2 rounded-full ${
                    settings.quotas.storageUsed / settings.quotas.storageLimit > 0.8
                      ? 'bg-red-600'
                      : settings.quotas.storageUsed / settings.quotas.storageLimit > 0.6
                      ? 'bg-yellow-600'
                      : 'bg-green-600'
                  }`}
                  style={{
                    width: `${Math.min((settings.quotas.storageUsed / settings.quotas.storageLimit) * 100, 100)}%`,
                  }}
                ></div>
              </div>
            </div>

            {/* API Calls Quota */}
            <div className="p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-sm font-medium text-gray-900">API Calls (This Hour)</h4>
                <span className="text-sm text-gray-600">
                  {formatQuotaUsage(settings.quotas.apiCallsUsed, settings.quotas.apiCallsLimit)}
                </span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className={`h-2 rounded-full ${
                    settings.quotas.apiCallsUsed / settings.quotas.apiCallsLimit > 0.8
                      ? 'bg-red-600'
                      : settings.quotas.apiCallsUsed / settings.quotas.apiCallsLimit > 0.6
                      ? 'bg-yellow-600'
                      : 'bg-green-600'
                  }`}
                  style={{
                    width: `${Math.min((settings.quotas.apiCallsUsed / settings.quotas.apiCallsLimit) * 100, 100)}%`,
                  }}
                ></div>
              </div>
            </div>

            {/* Workflows Quota */}
            <div className="p-4 border border-gray-200 rounded-lg">
              <div className="flex items-center justify-between mb-3">
                <h4 className="text-sm font-medium text-gray-900">Workflows (This Hour)</h4>
                <span className="text-sm text-gray-600">
                  {formatQuotaUsage(settings.quotas.workflowsUsed, settings.quotas.workflowsLimit)}
                </span>
              </div>
              <div className="w-full bg-gray-200 rounded-full h-2">
                <div
                  className={`h-2 rounded-full ${
                    settings.quotas.workflowsUsed / settings.quotas.workflowsLimit > 0.8
                      ? 'bg-red-600'
                      : settings.quotas.workflowsUsed / settings.quotas.workflowsLimit > 0.6
                      ? 'bg-yellow-600'
                      : 'bg-green-600'
                  }`}
                  style={{
                    width: `${Math.min((settings.quotas.workflowsUsed / settings.quotas.workflowsLimit) * 100, 100)}%`,
                  }}
                ></div>
              </div>
            </div>
          </div>
        </Card>
      )}
    </div>
  );
};

export default UserSettings;