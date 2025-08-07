import { useTranslation } from 'react-i18next'
import { useAuth } from '@/hooks/useAuth'
import { useTenant } from '@/contexts/TenantContext'
import { 
  UsersIcon, 
  DocumentIcon, 
  Cog8ToothIcon,
  ChartBarIcon,
} from '@heroicons/react/24/outline'

export default function DashboardPage() {
  const { t } = useTranslation()
  const { user } = useAuth()
  const { tenant } = useTenant()

  const stats = [
    {
      name: t('dashboard.stats.totalUsers'),
      value: '1,234',
      icon: UsersIcon,
      change: '+12%',
      changeType: 'positive',
    },
    {
      name: t('dashboard.stats.activeWorkflows'),
      value: '56',
      icon: Cog8ToothIcon,
      change: '+8%',
      changeType: 'positive',
    },
    {
      name: t('dashboard.stats.filesStored'),
      value: '2.4TB',
      icon: DocumentIcon,
      change: '+23%',
      changeType: 'positive',
    },
    {
      name: t('dashboard.stats.apiCalls'),
      value: '12,345',
      icon: ChartBarIcon,
      change: '-2%',
      changeType: 'negative',
    },
  ]

  const recentActivity = [
    {
      id: 1,
      type: 'user_created',
      message: 'New user john@example.com registered',
      time: '2 minutes ago',
    },
    {
      id: 2,
      type: 'workflow_completed',
      message: 'Data processing workflow completed successfully',
      time: '5 minutes ago',
    },
    {
      id: 3,
      type: 'file_uploaded',
      message: 'Document.pdf uploaded to shared folder',
      time: '10 minutes ago',
    },
    {
      id: 4,
      type: 'user_login',
      message: 'Admin user logged in from new device',
      time: '15 minutes ago',
    },
  ]

  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          {t('dashboard.title')}
        </h1>
        <p className="mt-2 text-lg text-gray-600 dark:text-gray-400">
          {t('dashboard.welcome', { name: user?.firstName || user?.email })}
        </p>
      </div>

      {/* Stats */}
      <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-4">
        {stats.map((stat) => (
          <div key={stat.name} className="card">
            <div className="card-body">
              <div className="flex items-center">
                <div className="flex-shrink-0">
                  <stat.icon className="h-8 w-8 text-primary-600" />
                </div>
                <div className="ml-4 w-0 flex-1">
                  <dl>
                    <dt className="text-sm font-medium text-gray-500 dark:text-gray-400 truncate">
                      {stat.name}
                    </dt>
                    <dd className="flex items-baseline">
                      <div className="text-2xl font-semibold text-gray-900 dark:text-white">
                        {stat.value}
                      </div>
                      <div className={`ml-2 flex items-baseline text-sm font-semibold ${
                        stat.changeType === 'positive' 
                          ? 'text-green-600 dark:text-green-400' 
                          : 'text-red-600 dark:text-red-400'
                      }`}>
                        {stat.change}
                      </div>
                    </dd>
                  </dl>
                </div>
              </div>
            </div>
          </div>
        ))}
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
        {/* Recent Activity */}
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              {t('dashboard.recentActivity')}
            </h3>
          </div>
          <div className="card-body">
            <div className="flow-root">
              <ul className="-mb-8">
                {recentActivity.map((activity, activityIdx) => (
                  <li key={activity.id}>
                    <div className="relative pb-8">
                      {activityIdx !== recentActivity.length - 1 ? (
                        <span
                          className="absolute top-4 left-4 -ml-px h-full w-0.5 bg-gray-200 dark:bg-gray-700"
                          aria-hidden="true"
                        />
                      ) : null}
                      <div className="relative flex space-x-3">
                        <div>
                          <span className="h-8 w-8 rounded-full bg-primary-500 flex items-center justify-center ring-8 ring-white dark:ring-gray-800">
                            <div className="h-2 w-2 bg-white rounded-full" />
                          </span>
                        </div>
                        <div className="min-w-0 flex-1 pt-1.5 flex justify-between space-x-4">
                          <div>
                            <p className="text-sm text-gray-500 dark:text-gray-400">
                              {activity.message}
                            </p>
                          </div>
                          <div className="text-right text-sm whitespace-nowrap text-gray-500 dark:text-gray-400">
                            {activity.time}
                          </div>
                        </div>
                      </div>
                    </div>
                  </li>
                ))}
              </ul>
            </div>
          </div>
        </div>

        {/* Quick Actions */}
        <div className="card">
          <div className="card-header">
            <h3 className="text-lg font-medium text-gray-900 dark:text-white">
              {t('dashboard.quickActions')}
            </h3>
          </div>
          <div className="card-body">
            <div className="grid grid-cols-2 gap-4">
              <button className="flex flex-col items-center p-4 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg hover:border-primary-500 dark:hover:border-primary-400 transition-colors">
                <UsersIcon className="h-8 w-8 text-gray-400 mb-2" />
                <span className="text-sm font-medium text-gray-900 dark:text-white">Add User</span>
              </button>
              <button className="flex flex-col items-center p-4 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg hover:border-primary-500 dark:hover:border-primary-400 transition-colors">
                <DocumentIcon className="h-8 w-8 text-gray-400 mb-2" />
                <span className="text-sm font-medium text-gray-900 dark:text-white">Upload File</span>
              </button>
              <button className="flex flex-col items-center p-4 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg hover:border-primary-500 dark:hover:border-primary-400 transition-colors">
                <Cog8ToothIcon className="h-8 w-8 text-gray-400 mb-2" />
                <span className="text-sm font-medium text-gray-900 dark:text-white">New Workflow</span>
              </button>
              <button className="flex flex-col items-center p-4 border-2 border-dashed border-gray-300 dark:border-gray-600 rounded-lg hover:border-primary-500 dark:hover:border-primary-400 transition-colors">
                <ChartBarIcon className="h-8 w-8 text-gray-400 mb-2" />
                <span className="text-sm font-medium text-gray-900 dark:text-white">View Reports</span>
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}