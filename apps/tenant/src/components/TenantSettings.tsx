import React, { useState } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { SaveIcon, AlertTriangleIcon } from 'lucide-react';
import { useCurrentTenant, useUpdateTenant } from '../hooks';
import { updateTenantSchema } from '../utils';
import { UpdateTenantRequest } from '../types';

interface TenantSettingsProps {
  className?: string;
}

interface SettingsFormData {
  name: string;
  description: string;
  timezone: string;
  dateFormat: string;
  language: string;
  theme: 'light' | 'dark' | 'system';
  emailNotifications: boolean;
  pushNotifications: boolean;
  smsNotifications: boolean;
  mfaRequired: boolean;
  sessionTimeout: number;
  primaryColor: string;
  secondaryColor: string;
  customDomain: string;
}

export const TenantSettings: React.FC<TenantSettingsProps> = ({
  className = '',
}) => {
  const { data: currentTenant, isLoading } = useCurrentTenant();
  const updateTenantMutation = useUpdateTenant();
  const [activeTab, setActiveTab] = useState<'general' | 'security' | 'branding'>('general');

  const {
    register,
    handleSubmit,
    formState: { errors, isDirty },
    reset,
    watch,
  } = useForm<SettingsFormData>({
    resolver: zodResolver(updateTenantSchema.partial()),
    defaultValues: {
      name: currentTenant?.name || '',
      description: currentTenant?.description || '',
      timezone: currentTenant?.settings?.timezone || 'UTC',
      dateFormat: currentTenant?.settings?.dateFormat || 'MM/DD/YYYY',
      language: currentTenant?.settings?.language || 'en',
      theme: currentTenant?.settings?.theme || 'system',
      emailNotifications: currentTenant?.settings?.notifications?.email ?? true,
      pushNotifications: currentTenant?.settings?.notifications?.push ?? true,
      smsNotifications: currentTenant?.settings?.notifications?.sms ?? false,
      mfaRequired: currentTenant?.settings?.security?.mfaRequired ?? false,
      sessionTimeout: currentTenant?.settings?.security?.sessionTimeout || 60,
      primaryColor: currentTenant?.settings?.branding?.primaryColor || '#3B82F6',
      secondaryColor: currentTenant?.settings?.branding?.secondaryColor || '#6B7280',
      customDomain: currentTenant?.settings?.branding?.customDomain || '',
    },
  });

  React.useEffect(() => {
    if (currentTenant) {
      reset({
        name: currentTenant.name,
        description: currentTenant.description || '',
        timezone: currentTenant.settings?.timezone || 'UTC',
        dateFormat: currentTenant.settings?.dateFormat || 'MM/DD/YYYY',
        language: currentTenant.settings?.language || 'en',
        theme: currentTenant.settings?.theme || 'system',
        emailNotifications: currentTenant.settings?.notifications?.email ?? true,
        pushNotifications: currentTenant.settings?.notifications?.push ?? true,
        smsNotifications: currentTenant.settings?.notifications?.sms ?? false,
        mfaRequired: currentTenant.settings?.security?.mfaRequired ?? false,
        sessionTimeout: currentTenant.settings?.security?.sessionTimeout || 60,
        primaryColor: currentTenant.settings?.branding?.primaryColor || '#3B82F6',
        secondaryColor: currentTenant.settings?.branding?.secondaryColor || '#6B7280',
        customDomain: currentTenant.settings?.branding?.customDomain || '',
      });
    }
  }, [currentTenant, reset]);

  const onSubmit = async (data: SettingsFormData) => {
    if (!currentTenant) return;

    const updateRequest: UpdateTenantRequest = {
      name: data.name !== currentTenant.name ? data.name : undefined,
      description: data.description !== currentTenant.description ? data.description : undefined,
      settings: {
        timezone: data.timezone,
        dateFormat: data.dateFormat,
        language: data.language,
        theme: data.theme,
        notifications: {
          email: data.emailNotifications,
          push: data.pushNotifications,
          sms: data.smsNotifications,
        },
        security: {
          mfaRequired: data.mfaRequired,
          sessionTimeout: data.sessionTimeout,
          passwordPolicy: {
            minLength: 8,
            requireUppercase: true,
            requireLowercase: true,
            requireNumbers: true,
            requireSpecialChars: false,
            maxAge: 90,
          },
        },
        branding: {
          primaryColor: data.primaryColor,
          secondaryColor: data.secondaryColor,
          customDomain: data.customDomain || undefined,
        },
      },
    };

    try {
      await updateTenantMutation.mutateAsync({
        tenantId: currentTenant.id,
        request: updateRequest,
      });
    } catch (error) {
      console.error('Failed to update tenant settings:', error);
    }
  };

  if (isLoading) {
    return (
      <div className={`animate-pulse space-y-4 ${className}`}>
        <div className="h-8 bg-gray-200 rounded w-1/4"></div>
        <div className="space-y-3">
          <div className="h-4 bg-gray-200 rounded w-full"></div>
          <div className="h-4 bg-gray-200 rounded w-3/4"></div>
          <div className="h-4 bg-gray-200 rounded w-1/2"></div>
        </div>
      </div>
    );
  }

  if (!currentTenant) {
    return (
      <div className={`text-center py-8 ${className}`}>
        <AlertTriangleIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
        <p className="text-gray-500">No tenant selected</p>
      </div>
    );
  }

  const tabs = [
    { id: 'general', name: 'General', icon: '⚙️' },
    { id: 'security', name: 'Security', icon: '🔒' },
    { id: 'branding', name: 'Branding', icon: '🎨' },
  ] as const;

  return (
    <div className={`bg-white rounded-lg shadow ${className}`}>
      <div className="border-b border-gray-200">
        <nav className="flex space-x-8 px-6">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              onClick={() => setActiveTab(tab.id)}
              className={`py-4 px-1 border-b-2 font-medium text-sm ${
                activeTab === tab.id
                  ? 'border-blue-500 text-blue-600'
                  : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300'
              }`}
            >
              <span className="mr-2">{tab.icon}</span>
              {tab.name}
            </button>
          ))}
        </nav>
      </div>

      <form onSubmit={handleSubmit(onSubmit)} className="p-6">
        {activeTab === 'general' && (
          <div className="space-y-6">
            <div>
              <label htmlFor="name" className="block text-sm font-medium text-gray-700">
                Tenant Name
              </label>
              <input
                type="text"
                id="name"
                {...register('name')}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              />
              {errors.name && (
                <p className="mt-1 text-sm text-red-600">{errors.name.message}</p>
              )}
            </div>

            <div>
              <label htmlFor="description" className="block text-sm font-medium text-gray-700">
                Description
              </label>
              <textarea
                id="description"
                rows={3}
                {...register('description')}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                placeholder="Optional description for your tenant"
              />
              {errors.description && (
                <p className="mt-1 text-sm text-red-600">{errors.description.message}</p>
              )}
            </div>

            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label htmlFor="timezone" className="block text-sm font-medium text-gray-700">
                  Timezone
                </label>
                <select
                  id="timezone"
                  {...register('timezone')}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                >
                  <option value="UTC">UTC</option>
                  <option value="America/New_York">Eastern Time</option>
                  <option value="America/Chicago">Central Time</option>
                  <option value="America/Denver">Mountain Time</option>
                  <option value="America/Los_Angeles">Pacific Time</option>
                  <option value="Europe/London">London</option>
                  <option value="Europe/Paris">Paris</option>
                  <option value="Asia/Tokyo">Tokyo</option>
                </select>
              </div>

              <div>
                <label htmlFor="language" className="block text-sm font-medium text-gray-700">
                  Language
                </label>
                <select
                  id="language"
                  {...register('language')}
                  className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                >
                  <option value="en">English</option>
                  <option value="es">Spanish</option>
                  <option value="fr">French</option>
                  <option value="de">German</option>
                  <option value="ja">Japanese</option>
                  <option value="zh">Chinese</option>
                </select>
              </div>
            </div>

            <div>
              <label htmlFor="theme" className="block text-sm font-medium text-gray-700">
                Theme
              </label>
              <select
                id="theme"
                {...register('theme')}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              >
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </div>

            <div>
              <h4 className="text-sm font-medium text-gray-700 mb-3">Notifications</h4>
              <div className="space-y-3">
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    {...register('emailNotifications')}
                    className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
                  />
                  <span className="ml-2 text-sm text-gray-700">Email notifications</span>
                </label>
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    {...register('pushNotifications')}
                    className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
                  />
                  <span className="ml-2 text-sm text-gray-700">Push notifications</span>
                </label>
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    {...register('smsNotifications')}
                    className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
                  />
                  <span className="ml-2 text-sm text-gray-700">SMS notifications</span>
                </label>
              </div>
            </div>
          </div>
        )}

        {activeTab === 'security' && (
          <div className="space-y-6">
            <div>
              <label className="flex items-center">
                <input
                  type="checkbox"
                  {...register('mfaRequired')}
                  className="rounded border-gray-300 text-blue-600 shadow-sm focus:border-blue-300 focus:ring focus:ring-blue-200 focus:ring-opacity-50"
                />
                <span className="ml-2 text-sm font-medium text-gray-700">
                  Require multi-factor authentication
                </span>
              </label>
              <p className="mt-1 text-sm text-gray-500">
                All users in this tenant will be required to set up MFA
              </p>
            </div>

            <div>
              <label htmlFor="sessionTimeout" className="block text-sm font-medium text-gray-700">
                Session Timeout (minutes)
              </label>
              <input
                type="number"
                id="sessionTimeout"
                min="5"
                max="1440"
                {...register('sessionTimeout', { valueAsNumber: true })}
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              />
              <p className="mt-1 text-sm text-gray-500">
                Users will be automatically logged out after this period of inactivity
              </p>
              {errors.sessionTimeout && (
                <p className="mt-1 text-sm text-red-600">{errors.sessionTimeout.message}</p>
              )}
            </div>
          </div>
        )}

        {activeTab === 'branding' && (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div>
                <label htmlFor="primaryColor" className="block text-sm font-medium text-gray-700">
                  Primary Color
                </label>
                <div className="mt-1 flex items-center space-x-3">
                  <input
                    type="color"
                    id="primaryColor"
                    {...register('primaryColor')}
                    className="h-10 w-20 border border-gray-300 rounded-md"
                  />
                  <input
                    type="text"
                    {...register('primaryColor')}
                    placeholder="#3B82F6"
                    className="flex-1 border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  />
                </div>
                {errors.primaryColor && (
                  <p className="mt-1 text-sm text-red-600">{errors.primaryColor.message}</p>
                )}
              </div>

              <div>
                <label htmlFor="secondaryColor" className="block text-sm font-medium text-gray-700">
                  Secondary Color
                </label>
                <div className="mt-1 flex items-center space-x-3">
                  <input
                    type="color"
                    id="secondaryColor"
                    {...register('secondaryColor')}
                    className="h-10 w-20 border border-gray-300 rounded-md"
                  />
                  <input
                    type="text"
                    {...register('secondaryColor')}
                    placeholder="#6B7280"
                    className="flex-1 border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                  />
                </div>
                {errors.secondaryColor && (
                  <p className="mt-1 text-sm text-red-600">{errors.secondaryColor.message}</p>
                )}
              </div>
            </div>

            <div>
              <label htmlFor="customDomain" className="block text-sm font-medium text-gray-700">
                Custom Domain
              </label>
              <input
                type="url"
                id="customDomain"
                {...register('customDomain')}
                placeholder="https://your-domain.com"
                className="mt-1 block w-full border-gray-300 rounded-md shadow-sm focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              />
              <p className="mt-1 text-sm text-gray-500">
                Optional custom domain for your tenant
              </p>
              {errors.customDomain && (
                <p className="mt-1 text-sm text-red-600">{errors.customDomain.message}</p>
              )}
            </div>

            <div className="bg-gray-50 p-4 rounded-md">
              <h4 className="text-sm font-medium text-gray-700 mb-2">Preview</h4>
              <div className="flex items-center space-x-3">
                <div
                  className="w-8 h-8 rounded-full"
                  style={{ backgroundColor: watch('primaryColor') }}
                />
                <div
                  className="w-8 h-8 rounded-full"
                  style={{ backgroundColor: watch('secondaryColor') }}
                />
                <span className="text-sm text-gray-600">Color scheme preview</span>
              </div>
            </div>
          </div>
        )}

        <div className="flex justify-end pt-6 border-t border-gray-200">
          <button
            type="submit"
            disabled={!isDirty || updateTenantMutation.isPending}
            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50 disabled:cursor-not-allowed"
          >
            {updateTenantMutation.isPending ? (
              <div className="animate-spin rounded-full h-4 w-4 border-b-2 border-white mr-2" />
            ) : (
              <SaveIcon className="w-4 h-4 mr-2" />
            )}
            Save Changes
          </button>
        </div>
      </form>
    </div>
  );
};