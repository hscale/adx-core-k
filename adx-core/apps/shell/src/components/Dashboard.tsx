import React from 'react';
import { useTranslation } from 'react-i18next';
import { useTenantContext, useUserContext } from '@adx-core/shared-context';
import { Card, Button } from '@adx-core/design-system';
import { 
  UserIcon, 
  FolderIcon, 
  GitBranchIcon as WorkflowIcon,
  PackageIcon as ModuleIcon,
  TrendingUpIcon,
  ActivityIcon
} from 'lucide-react';

const Dashboard: React.FC = () => {
  const { t } = useTranslation();
  const { currentTenant } = useTenantContext();
  const { currentUser } = useUserContext();

  const quickActions = [
    {
      title: t('dashboard.quickActions.uploadFile'),
      description: t('dashboard.quickActions.uploadFileDesc'),
      icon: FolderIcon,
      href: '/files/upload',
      color: 'bg-blue-500',
    },
    {
      title: t('dashboard.quickActions.manageUsers'),
      description: t('dashboard.quickActions.manageUsersDesc'),
      icon: UserIcon,
      href: '/users',
      color: 'bg-green-500',
    },
    {
      title: t('dashboard.quickActions.viewWorkflows'),
      description: t('dashboard.quickActions.viewWorkflowsDesc'),
      icon: WorkflowIcon,
      href: '/workflows',
      color: 'bg-purple-500',
    },
    {
      title: t('dashboard.quickActions.browseModules'),
      description: t('dashboard.quickActions.browseModulesDesc'),
      icon: ModuleIcon,
      href: '/modules',
      color: 'bg-orange-500',
    },
  ];

  const stats = [
    {
      title: t('dashboard.stats.totalUsers'),
      value: '1,234',
      change: '+12%',
      icon: UserIcon,
    },
    {
      title: t('dashboard.stats.totalFiles'),
      value: '5,678',
      change: '+8%',
      icon: FolderIcon,
    },
    {
      title: t('dashboard.stats.activeWorkflows'),
      value: '23',
      change: '+15%',
      icon: WorkflowIcon,
    },
    {
      title: t('dashboard.stats.installedModules'),
      value: '12',
      change: '+2%',
      icon: ModuleIcon,
    },
  ];

  return (
    <div className="max-w-7xl mx-auto py-6 sm:px-6 lg:px-8">
      <div className="px-4 py-6 sm:px-0">
        <div className="mb-8">
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            {t('dashboard.welcome', { name: currentUser?.name || 'User' })}
          </h1>
          {currentTenant && (
            <p className="mt-2 text-gray-600 dark:text-gray-400">
              {t('dashboard.currentTenant', { tenant: currentTenant.name })}
            </p>
          )}
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6 mb-8">
          {stats.map((stat, index) => {
            const Icon = stat.icon;
            return (
              <Card key={index} className="p-6">
                <div className="flex items-center">
                  <div className="flex-shrink-0">
                    <Icon className="h-8 w-8 text-gray-400" />
                  </div>
                  <div className="ml-5 w-0 flex-1">
                    <dl>
                      <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                        {stat.title}
                      </dt>
                      <dd className="flex items-baseline">
                        <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                          {stat.value}
                        </div>
                        <div className="ml-2 flex items-baseline text-sm font-semibold text-green-600">
                          <TrendingUpIcon className="self-center flex-shrink-0 h-4 w-4 text-green-500" />
                          <span className="sr-only">Increased by</span>
                          {stat.change}
                        </div>
                      </dd>
                    </dl>
                  </div>
                </div>
              </Card>
            );
          })}
        </div>

        {/* Quick Actions */}
        <div className="mb-8">
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
            {t('dashboard.quickActions.title')}
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
            {quickActions.map((action, index) => {
              const Icon = action.icon;
              return (
                <Card key={index} className="p-6 hover:shadow-lg transition-shadow cursor-pointer">
                  <div className="flex items-center mb-4">
                    <div className={`p-2 rounded-lg ${action.color}`}>
                      <Icon className="h-6 w-6 text-white" />
                    </div>
                  </div>
                  <h3 className="text-lg font-medium text-gray-900 dark:text-white mb-2">
                    {action.title}
                  </h3>
                  <p className="text-gray-600 dark:text-gray-400 text-sm mb-4">
                    {action.description}
                  </p>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => window.location.href = action.href}
                  >
                    {t('dashboard.quickActions.getStarted')}
                  </Button>
                </Card>
              );
            })}
          </div>
        </div>

        {/* Recent Activity */}
        <div>
          <h2 className="text-xl font-semibold text-gray-900 dark:text-white mb-4">
            {t('dashboard.recentActivity.title')}
          </h2>
          <Card className="p-6">
            <div className="flex items-center justify-center h-32">
              <div className="text-center">
                <ActivityIcon className="h-12 w-12 text-gray-400 mx-auto mb-4" />
                <p className="text-gray-500 dark:text-gray-400">
                  {t('dashboard.recentActivity.noActivity')}
                </p>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  );
};

export default Dashboard;