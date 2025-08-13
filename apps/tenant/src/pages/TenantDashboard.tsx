import React from 'react';
import { 
  UsersIcon, 
  SettingsIcon, 
  BarChart3Icon,
  HardDriveIcon,
  ActivityIcon,
  AlertTriangleIcon,
} from 'lucide-react';
import { useCurrentTenant } from '../hooks';
import { TenantSwitcher } from '../components';
import {
  formatSubscriptionTier,
  getSubscriptionTierColorClass,
} from '../utils';

interface TenantDashboardProps {
  className?: string;
}

export const TenantDashboard: React.FC<TenantDashboardProps> = ({
  className = '',
}) => {
  const { data: currentTenant, isLoading: tenantLoading } = useCurrentTenant();
  // const { data: members, isLoading: membersLoading } = useTenantMembers(currentTenant?.id || '');

  if (tenantLoading) {
    return (
      <div className={`animate-pulse space-y-6 ${className}`}>
        <div className="h-8 bg-gray-200 rounded w-1/3"></div>
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {[...Array(4)].map((_, i) => (
            <div key={i} className="bg-white p-6 rounded-lg shadow">
              <div className="h-4 bg-gray-200 rounded w-1/2 mb-2"></div>
              <div className="h-8 bg-gray-200 rounded w-3/4"></div>
            </div>
          ))}
        </div>
      </div>
    );
  }

  if (!currentTenant) {
    return (
      <div className={`text-center py-12 ${className}`}>
        <AlertTriangleIcon className="w-16 h-16 text-gray-400 mx-auto mb-4" />
        <h3 className="text-lg font-medium text-gray-900 mb-2">No Tenant Selected</h3>
        <p className="text-gray-500 mb-6">
          Please select a tenant to view the dashboard.
        </p>
        <div className="max-w-xs mx-auto">
          <TenantSwitcher />
        </div>
      </div>
    );
  }

  const quotas = currentTenant.quotas;
  const storagePercentage = quotas.maxStorageGB > 0 ? (quotas.currentStorageGB / quotas.maxStorageGB) * 100 : 0;
  const userPercentage = quotas.maxUsers > 0 ? (quotas.currentUsers / quotas.maxUsers) * 100 : 0;
  const apiPercentage = quotas.maxApiCallsPerHour > 0 ? (quotas.currentApiCallsThisHour / quotas.maxApiCallsPerHour) * 100 : 0;
  const workflowPercentage = quotas.maxWorkflowsPerHour > 0 ? (quotas.currentWorkflowsThisHour / quotas.maxWorkflowsPerHour) * 100 : 0;

  const getUsageColor = (percentage: number) => {
    if (percentage >= 90) return 'text-red-600 bg-red-100';
    if (percentage >= 75) return 'text-yellow-600 bg-yellow-100';
    return 'text-green-600 bg-green-100';
  };

  const getProgressColor = (percentage: number) => {
    if (percentage >= 90) return 'bg-red-500';
    if (percentage >= 75) return 'bg-yellow-500';
    return 'bg-green-500';
  };

  return (
    <div className={`space-y-6 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">
            {currentTenant.name}
          </h1>
          <p className="text-gray-500">
            {currentTenant.description || 'Tenant dashboard and overview'}
          </p>
        </div>
        <div className="flex items-center space-x-4">
          <span
            className={`inline-flex items-center px-3 py-1 rounded-full text-sm font-medium ${getSubscriptionTierColorClass(
              currentTenant.subscriptionTier
            )}`}
          >
            {formatSubscriptionTier(currentTenant.subscriptionTier)}
          </span>
          <div className="w-64">
            <TenantSwitcher />
          </div>
        </div>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {/* Users */}
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Team Members</p>
              <p className="text-2xl font-bold text-gray-900">
                {quotas.currentUsers}
              </p>
              <p className="text-xs text-gray-500">
                of {quotas.maxUsers} allowed
              </p>
            </div>
            <div className="p-3 bg-blue-100 rounded-full">
              <UsersIcon className="w-6 h-6 text-blue-600" />
            </div>
          </div>
          <div className="mt-4">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-500">Usage</span>
              <span className={`font-medium ${getUsageColor(userPercentage).split(' ')[0]}`}>
                {userPercentage.toFixed(1)}%
              </span>
            </div>
            <div className="mt-1 w-full bg-gray-200 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${getProgressColor(userPercentage)}`}
                style={{ width: `${Math.min(userPercentage, 100)}%` }}
              />
            </div>
          </div>
        </div>

        {/* Storage */}
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Storage</p>
              <p className="text-2xl font-bold text-gray-900">
                {quotas.currentStorageGB.toFixed(1)} GB
              </p>
              <p className="text-xs text-gray-500">
                of {quotas.maxStorageGB} GB allowed
              </p>
            </div>
            <div className="p-3 bg-green-100 rounded-full">
              <HardDriveIcon className="w-6 h-6 text-green-600" />
            </div>
          </div>
          <div className="mt-4">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-500">Usage</span>
              <span className={`font-medium ${getUsageColor(storagePercentage).split(' ')[0]}`}>
                {storagePercentage.toFixed(1)}%
              </span>
            </div>
            <div className="mt-1 w-full bg-gray-200 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${getProgressColor(storagePercentage)}`}
                style={{ width: `${Math.min(storagePercentage, 100)}%` }}
              />
            </div>
          </div>
        </div>

        {/* API Calls */}
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">API Calls</p>
              <p className="text-2xl font-bold text-gray-900">
                {quotas.currentApiCallsThisHour.toLocaleString()}
              </p>
              <p className="text-xs text-gray-500">
                this hour (limit: {quotas.maxApiCallsPerHour.toLocaleString()})
              </p>
            </div>
            <div className="p-3 bg-purple-100 rounded-full">
              <BarChart3Icon className="w-6 h-6 text-purple-600" />
            </div>
          </div>
          <div className="mt-4">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-500">Usage</span>
              <span className={`font-medium ${getUsageColor(apiPercentage).split(' ')[0]}`}>
                {apiPercentage.toFixed(1)}%
              </span>
            </div>
            <div className="mt-1 w-full bg-gray-200 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${getProgressColor(apiPercentage)}`}
                style={{ width: `${Math.min(apiPercentage, 100)}%` }}
              />
            </div>
          </div>
        </div>

        {/* Workflows */}
        <div className="bg-white p-6 rounded-lg shadow">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Workflows</p>
              <p className="text-2xl font-bold text-gray-900">
                {quotas.currentWorkflowsThisHour}
              </p>
              <p className="text-xs text-gray-500">
                this hour (limit: {quotas.maxWorkflowsPerHour})
              </p>
            </div>
            <div className="p-3 bg-orange-100 rounded-full">
              <ActivityIcon className="w-6 h-6 text-orange-600" />
            </div>
          </div>
          <div className="mt-4">
            <div className="flex items-center justify-between text-xs">
              <span className="text-gray-500">Usage</span>
              <span className={`font-medium ${getUsageColor(workflowPercentage).split(' ')[0]}`}>
                {workflowPercentage.toFixed(1)}%
              </span>
            </div>
            <div className="mt-1 w-full bg-gray-200 rounded-full h-2">
              <div
                className={`h-2 rounded-full ${getProgressColor(workflowPercentage)}`}
                style={{ width: `${Math.min(workflowPercentage, 100)}%` }}
              />
            </div>
          </div>
        </div>
      </div>

      {/* Quick Actions */}
      <div className="bg-white rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-medium text-gray-900">Quick Actions</h3>
        </div>
        <div className="p-6">
          <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
            <button className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors">
              <UsersIcon className="w-8 h-8 text-blue-600 mr-3" />
              <div className="text-left">
                <p className="font-medium text-gray-900">Manage Members</p>
                <p className="text-sm text-gray-500">Invite and manage team members</p>
              </div>
            </button>
            
            <button className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors">
              <SettingsIcon className="w-8 h-8 text-green-600 mr-3" />
              <div className="text-left">
                <p className="font-medium text-gray-900">Tenant Settings</p>
                <p className="text-sm text-gray-500">Configure tenant preferences</p>
              </div>
            </button>
            
            <button className="flex items-center p-4 border border-gray-200 rounded-lg hover:bg-gray-50 transition-colors">
              <BarChart3Icon className="w-8 h-8 text-purple-600 mr-3" />
              <div className="text-left">
                <p className="font-medium text-gray-900">View Analytics</p>
                <p className="text-sm text-gray-500">Usage reports and insights</p>
              </div>
            </button>
          </div>
        </div>
      </div>

      {/* Recent Activity */}
      <div className="bg-white rounded-lg shadow">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-medium text-gray-900">Recent Activity</h3>
        </div>
        <div className="p-6">
          <div className="text-center py-8">
            <ActivityIcon className="w-12 h-12 text-gray-400 mx-auto mb-4" />
            <p className="text-gray-500">No recent activity to display</p>
          </div>
        </div>
      </div>
    </div>
  );
};