import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { 
  UserIcon,
  ShieldCheckIcon,
  BellIcon,
  GlobeAltIcon,
  CreditCardIcon,
  Cog6ToothIcon,
  PaintBrushIcon,
} from '@heroicons/react/24/outline'

import { useTheme } from '@/contexts/ThemeContext'
import Button from '@/components/ui/Button'

type SettingsTab = 'general' | 'appearance' | 'security' | 'notifications' | 'integrations' | 'billing' | 'advanced'

export default function SettingsPage() {
  const { t, i18n } = useTranslation()
  const { theme, setTheme } = useTheme()
  const [activeTab, setActiveTab] = useState<SettingsTab>('general')

  const tabs = [
    { id: 'general' as const, name: t('settings.general'), icon: Cog6ToothIcon },
    { id: 'appearance' as const, name: t('settings.appearance.title'), icon: PaintBrushIcon },
    { id: 'security' as const, name: t('settings.security'), icon: ShieldCheckIcon },
    { id: 'notifications' as const, name: t('settings.notifications'), icon: BellIcon },
    { id: 'integrations' as const, name: t('settings.integrations'), icon: GlobeAltIcon },
    { id: 'billing' as const, name: t('settings.billing'), icon: CreditCardIcon },
    { id: 'advanced' as const, name: t('settings.advanced'), icon: UserIcon },
  ]

  const languages = [
    { code: 'en', name: 'English' },
    { code: 'es', name: 'Español' },
    { code: 'fr', name: 'Français' },
    { code: 'de', name: 'Deutsch' },
    { code: 'ja', name: '日本語' },
    { code: 'zh', name: '中文' },
  ]

  const timezones = [
    'UTC',
    'America/New_York',
    'America/Los_Angeles',
    'Europe/London',
    'Europe/Paris',
    'Asia/Tokyo',
    'Asia/Shanghai',
  ]

  const renderTabContent = () => {
    switch (activeTab) {
      case 'general':
        return (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                General Settings
              </h3>
              <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                Configure your basic account preferences
              </p>
            </div>

            <div className="grid grid-cols-1 gap-6">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Organization Name
                </label>
                <input
                  type="text"
                  className="input mt-1"
                  defaultValue="My Organization"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Default Timezone
                </label>
                <select className="input mt-1">
                  {timezones.map((tz) => (
                    <option key={tz} value={tz}>
                      {tz}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </div>
        )

      case 'appearance':
        return (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                {t('settings.appearance.title')}
              </h3>
              <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                Customize how the application looks and feels
              </p>
            </div>

            <div className="space-y-6">
              {/* Theme */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-3">
                  {t('settings.appearance.theme')}
                </label>
                <div className="grid grid-cols-3 gap-3">
                  {(['light', 'dark', 'auto'] as const).map((themeOption) => (
                    <button
                      key={themeOption}
                      onClick={() => setTheme(themeOption)}
                      className={`p-4 border-2 rounded-lg text-center transition-colors ${
                        theme === themeOption
                          ? 'border-primary-500 bg-primary-50 dark:bg-primary-900/20'
                          : 'border-gray-200 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'
                      }`}
                    >
                      <div className="text-sm font-medium text-gray-900 dark:text-white">
                        {t(`settings.appearance.themes.${themeOption}`)}
                      </div>
                    </button>
                  ))}
                </div>
              </div>

              {/* Language */}
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  {t('settings.appearance.language')}
                </label>
                <select
                  className="input mt-1"
                  value={i18n.language}
                  onChange={(e) => i18n.changeLanguage(e.target.value)}
                >
                  {languages.map((lang) => (
                    <option key={lang.code} value={lang.code}>
                      {lang.name}
                    </option>
                  ))}
                </select>
              </div>
            </div>
          </div>
        )

      case 'security':
        return (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                Security Settings
              </h3>
              <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                Manage your account security and authentication
              </p>
            </div>

            <div className="space-y-6">
              <div className="card">
                <div className="card-body">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="text-sm font-medium text-gray-900 dark:text-white">
                        Two-Factor Authentication
                      </h4>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        Add an extra layer of security to your account
                      </p>
                    </div>
                    <Button variant="outline" size="sm">
                      Enable
                    </Button>
                  </div>
                </div>
              </div>

              <div className="card">
                <div className="card-body">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="text-sm font-medium text-gray-900 dark:text-white">
                        Password
                      </h4>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        Last changed 3 months ago
                      </p>
                    </div>
                    <Button variant="outline" size="sm">
                      Change
                    </Button>
                  </div>
                </div>
              </div>

              <div className="card">
                <div className="card-body">
                  <div className="flex items-center justify-between">
                    <div>
                      <h4 className="text-sm font-medium text-gray-900 dark:text-white">
                        Active Sessions
                      </h4>
                      <p className="text-sm text-gray-500 dark:text-gray-400">
                        Manage your active login sessions
                      </p>
                    </div>
                    <Button variant="outline" size="sm">
                      View All
                    </Button>
                  </div>
                </div>
              </div>
            </div>
          </div>
        )

      case 'notifications':
        return (
          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                Notification Preferences
              </h3>
              <p className="mt-1 text-sm text-gray-600 dark:text-gray-400">
                Choose what notifications you want to receive
              </p>
            </div>

            <div className="space-y-4">
              {[
                { id: 'email', label: 'Email notifications', description: 'Receive notifications via email' },
                { id: 'push', label: 'Push notifications', description: 'Receive push notifications in your browser' },
                { id: 'workflow', label: 'Workflow updates', description: 'Get notified when workflows complete' },
                { id: 'security', label: 'Security alerts', description: 'Important security notifications' },
              ].map((notification) => (
                <div key={notification.id} className="flex items-center justify-between py-3">
                  <div>
                    <div className="text-sm font-medium text-gray-900 dark:text-white">
                      {notification.label}
                    </div>
                    <div className="text-sm text-gray-500 dark:text-gray-400">
                      {notification.description}
                    </div>
                  </div>
                  <input
                    type="checkbox"
                    className="h-4 w-4 text-primary-600 focus:ring-primary-500 border-gray-300 rounded"
                    defaultChecked
                  />
                </div>
              ))}
            </div>
          </div>
        )

      default:
        return (
          <div className="text-center py-12">
            <div className="text-gray-500 dark:text-gray-400">
              Settings for {activeTab} coming soon...
            </div>
          </div>
        )
    }
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          {t('settings.title')}
        </h1>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          Manage your account settings and preferences
        </p>
      </div>

      <div className="flex flex-col lg:flex-row gap-6">
        {/* Sidebar */}
        <div className="lg:w-64">
          <nav className="space-y-1">
            {tabs.map((tab) => (
              <button
                key={tab.id}
                onClick={() => setActiveTab(tab.id)}
                className={`w-full flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors ${
                  activeTab === tab.id
                    ? 'bg-primary-50 text-primary-600 dark:bg-primary-900/20 dark:text-primary-400'
                    : 'text-gray-700 hover:text-gray-900 hover:bg-gray-50 dark:text-gray-300 dark:hover:text-white dark:hover:bg-gray-800'
                }`}
              >
                <tab.icon className="mr-3 h-5 w-5" />
                {tab.name}
              </button>
            ))}
          </nav>
        </div>

        {/* Content */}
        <div className="flex-1">
          <div className="card">
            <div className="card-body">
              {renderTabContent()}
            </div>
            <div className="card-footer">
              <div className="flex justify-end gap-3">
                <Button variant="outline">
                  Cancel
                </Button>
                <Button>
                  Save Changes
                </Button>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}