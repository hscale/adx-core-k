import React from 'react';
import { useAuthStore, useTenantStore } from '@adx-core/shared-context';
import { Card, CardContent, CardHeader, CardTitle } from '@adx-core/design-system';
import { Users, Building2, FolderOpen, Workflow, Activity, TrendingUp } from 'lucide-react';

export const Dashboard: React.FC = () => {
  const { user } = useAuthStore();
  const { currentTenant } = useTenantStore();

  const stats = [
    {
      name: 'Total Users',
      value: '1,234',
      change: '+12%',
      changeType: 'positive' as const,
      icon: Users,
    },
    {
      name: 'Active Workflows',
      value: '23',
      change: '+5%',
      changeType: 'positive' as const,
      icon: Workflow,
    },
    {
      name: 'Files Stored',
      value: '5.2GB',
      change: '+18%',
      changeType: 'positive' as const,
      icon: FolderOpen,
    },
    {
      name: 'API Calls',
      value: '12.5K',
      change: '-2%',
      changeType: 'negative' as const,
      icon: Activity,
    },
  ];

  const recentActivities = [
    {
      id: 1,
      type: 'workflow',
      title: 'User onboarding workflow completed',
      description: 'New user john.doe@example.com successfully onboarded',
      timestamp: '2 minutes ago',
    },
    {
      id: 2,
      type: 'tenant',
      title: 'Tenant settings updated',
      description: 'Theme changed to dark mode',
      timestamp: '15 minutes ago',
    },
    {
      id: 3,
      type: 'file',
      title: 'File uploaded',
      description: 'document.pdf uploaded to shared folder',
      timestamp: '1 hour ago',
    },
    {
      id: 4,
      type: 'user',
      title: 'New user registered',
      description: 'jane.smith@example.com joined the platform',
      timestamp: '2 hours ago',
    },
  ];

  return (
    <div className="space-y-6">
      {/* Welcome Section */}
      <div className="bg-gradient-to-r from-primary-600 to-primary-700 rounded-lg p-6 text-white">
        <h1 className="text-2xl font-bold mb-2">
          Welcome back, {user?.name || 'User'}!
        </h1>
        <p className="text-primary-100">
          {currentTenant ? (
            <>Managing <span className="font-medium">{currentTenant.name}</span> tenant</>
          ) : (
            'Ready to get started with ADX Core'
          )}
        </p>
      </div>

      {/* Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        {stats.map((stat) => {
          const Icon = stat.icon;
          return (
            <Card key={stat.name}>
              <CardContent className="p-6">
                <div className="flex items-center justify-between">
                  <div>
                    <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                      {stat.name}
                    </p>
                    <p className="text-2xl font-bold text-gray-900 dark:text-gray-100">
                      {stat.value}
                    </p>
                  </div>
                  <div className="h-12 w-12 rounded-full bg-primary-100 dark:bg-primary-900/20 flex items-center justify-center">
                    <Icon className="h-6 w-6 text-primary-600 dark:text-primary-400" />
                  </div>
                </div>
                <div className="mt-4 flex items-center">
                  <TrendingUp className={`h-4 w-4 mr-1 ${
                    stat.changeType === 'positive' 
                      ? 'text-green-500' 
                      : 'text-red-500'
                  }`} />
                  <span className={`text-sm font-medium ${
                    stat.changeType === 'positive' 
                      ? 'text-green-600 dark:text-green-400' 
                      : 'text-red-600 dark:text-red-400'
                  }`}>
                    {stat.change}
                  </span>
                  <span className="text-sm text-gray-600 dark:text-gray-400 ml-1">
                    from last month
                  </span>
                </div>
              </CardContent>
            </Card>
          );
        })}
      </div>

      {/* Recent Activity */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        <Card>
          <CardHeader>
            <CardTitle>Recent Activity</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="space-y-4">
              {recentActivities.map((activity) => (
                <div key={activity.id} className="flex items-start space-x-3">
                  <div className="h-2 w-2 rounded-full bg-primary-500 mt-2 flex-shrink-0" />
                  <div className="flex-1 min-w-0">
                    <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                      {activity.title}
                    </p>
                    <p className="text-sm text-gray-600 dark:text-gray-400">
                      {activity.description}
                    </p>
                    <p className="text-xs text-gray-500 dark:text-gray-500 mt-1">
                      {activity.timestamp}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-2 gap-4">
              <button className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-left">
                <Users className="h-6 w-6 text-primary-600 dark:text-primary-400 mb-2" />
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  Add User
                </p>
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  Invite new team member
                </p>
              </button>
              
              <button className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-left">
                <FolderOpen className="h-6 w-6 text-primary-600 dark:text-primary-400 mb-2" />
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  Upload File
                </p>
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  Add new document
                </p>
              </button>
              
              <button className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-left">
                <Workflow className="h-6 w-6 text-primary-600 dark:text-primary-400 mb-2" />
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  Start Workflow
                </p>
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  Create new process
                </p>
              </button>
              
              <button className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors text-left">
                <Building2 className="h-6 w-6 text-primary-600 dark:text-primary-400 mb-2" />
                <p className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  Tenant Settings
                </p>
                <p className="text-xs text-gray-600 dark:text-gray-400">
                  Configure tenant
                </p>
              </button>
            </div>
          </CardContent>
        </Card>
      </div>
    </div>
  );
};