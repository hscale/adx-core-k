import React, { useState } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { 
  HomeIcon, 
  UsersIcon, 
  BuildingOfficeIcon,
  DocumentIcon,
  Cog8ToothIcon,
  UserCircleIcon,
  Bars3Icon,
  XMarkIcon,
  BellIcon,
  MagnifyingGlassIcon,
  SunIcon,
  MoonIcon,
  ComputerDesktopIcon,
} from '@heroicons/react/24/outline'

import { useAuth } from '@/hooks/useAuth'
import { useTheme } from '@/contexts/ThemeContext'
import { useTenant } from '@/contexts/TenantContext'
import { usePlatform } from '@/hooks/usePlatform'
import Button from '@/components/ui/Button'

interface DashboardLayoutProps {
  children: React.ReactNode
}

export default function DashboardLayout({ children }: DashboardLayoutProps) {
  const { t } = useTranslation()
  const { user, logout } = useAuth()
  const { theme, setTheme, isDark } = useTheme()
  const { tenant, availableTenants, switchTenant } = useTenant()
  const { platform } = usePlatform()
  const location = useLocation()
  const [sidebarOpen, setSidebarOpen] = useState(false)

  const navigation = [
    { name: t('navigation.dashboard'), href: '/dashboard', icon: HomeIcon },
    { name: t('navigation.users'), href: '/users', icon: UsersIcon },
    { name: t('navigation.tenants'), href: '/tenants', icon: BuildingOfficeIcon },
    { name: t('navigation.files'), href: '/files', icon: DocumentIcon },
    { name: t('navigation.workflows'), href: '/workflows', icon: Cog8ToothIcon },
  ]

  const themeIcons = {
    light: SunIcon,
    dark: MoonIcon,
    auto: ComputerDesktopIcon,
  }

  const ThemeIcon = themeIcons[theme]

  const cycleTheme = () => {
    const themes: Array<typeof theme> = ['light', 'dark', 'auto']
    const currentIndex = themes.indexOf(theme)
    const nextIndex = (currentIndex + 1) % themes.length
    setTheme(themes[nextIndex])
  }

  return (
    <div className="min-h-screen bg-gray-50 dark:bg-gray-900">
      {/* Mobile sidebar */}
      <div className={`lg:hidden ${sidebarOpen ? 'block' : 'hidden'}`}>
        <div className="fixed inset-0 z-40 flex">
          <div className="fixed inset-0 bg-gray-600 bg-opacity-75" onClick={() => setSidebarOpen(false)} />
          <div className="relative flex w-full max-w-xs flex-1 flex-col bg-white dark:bg-gray-800">
            <div className="absolute top-0 right-0 -mr-12 pt-2">
              <button
                type="button"
                className="ml-1 flex h-10 w-10 items-center justify-center rounded-full focus:outline-none focus:ring-2 focus:ring-inset focus:ring-white"
                onClick={() => setSidebarOpen(false)}
              >
                <XMarkIcon className="h-6 w-6 text-white" />
              </button>
            </div>
            <SidebarContent navigation={navigation} currentPath={location.pathname} />
          </div>
        </div>
      </div>

      {/* Desktop sidebar */}
      <div className="hidden lg:fixed lg:inset-y-0 lg:z-50 lg:flex lg:w-72 lg:flex-col">
        <div className="flex grow flex-col gap-y-5 overflow-y-auto bg-white dark:bg-gray-800 border-r border-gray-200 dark:border-gray-700">
          <SidebarContent navigation={navigation} currentPath={location.pathname} />
        </div>
      </div>

      {/* Main content */}
      <div className="lg:pl-72">
        {/* Top bar */}
        <div className="sticky top-0 z-40 flex h-16 shrink-0 items-center gap-x-4 border-b border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 px-4 shadow-sm sm:gap-x-6 sm:px-6 lg:px-8">
          <button
            type="button"
            className="-m-2.5 p-2.5 text-gray-700 dark:text-gray-300 lg:hidden"
            onClick={() => setSidebarOpen(true)}
          >
            <Bars3Icon className="h-6 w-6" />
          </button>

          {/* Separator */}
          <div className="h-6 w-px bg-gray-200 dark:bg-gray-700 lg:hidden" />

          <div className="flex flex-1 gap-x-4 self-stretch lg:gap-x-6">
            {/* Search */}
            <form className="relative flex flex-1" action="#" method="GET">
              <label htmlFor="search-field" className="sr-only">
                Search
              </label>
              <MagnifyingGlassIcon className="pointer-events-none absolute inset-y-0 left-0 h-full w-5 text-gray-400" />
              <input
                id="search-field"
                className="block h-full w-full border-0 py-0 pl-8 pr-0 text-gray-900 dark:text-white placeholder:text-gray-400 focus:ring-0 sm:text-sm bg-transparent"
                placeholder="Search..."
                type="search"
                name="search"
              />
            </form>

            <div className="flex items-center gap-x-4 lg:gap-x-6">
              {/* Theme toggle */}
              <button
                type="button"
                className="-m-2.5 p-2.5 text-gray-400 hover:text-gray-500 dark:hover:text-gray-300"
                onClick={cycleTheme}
                title={`Current theme: ${theme}`}
              >
                <ThemeIcon className="h-6 w-6" />
              </button>

              {/* Notifications */}
              <button
                type="button"
                className="-m-2.5 p-2.5 text-gray-400 hover:text-gray-500 dark:hover:text-gray-300"
              >
                <BellIcon className="h-6 w-6" />
              </button>

              {/* Separator */}
              <div className="hidden lg:block lg:h-6 lg:w-px lg:bg-gray-200 dark:lg:bg-gray-700" />

              {/* Profile dropdown */}
              <div className="relative">
                <button
                  type="button"
                  className="-m-1.5 flex items-center p-1.5"
                  id="user-menu-button"
                >
                  <span className="sr-only">Open user menu</span>
                  <UserCircleIcon className="h-8 w-8 text-gray-400" />
                  <span className="hidden lg:flex lg:items-center">
                    <span className="ml-4 text-sm font-semibold leading-6 text-gray-900 dark:text-white">
                      {user?.firstName || user?.email}
                    </span>
                  </span>
                </button>
              </div>
            </div>
          </div>
        </div>

        {/* Page content */}
        <main className="py-10">
          <div className="px-4 sm:px-6 lg:px-8">
            {children}
          </div>
        </main>
      </div>
    </div>
  )
}

function SidebarContent({ navigation, currentPath }: { navigation: any[], currentPath: string }) {
  const { t } = useTranslation()
  const { user, logout } = useAuth()
  const { tenant } = useTenant()

  return (
    <>
      {/* Logo */}
      <div className="flex h-16 shrink-0 items-center px-6">
        <div className="flex items-center">
          {tenant?.settings?.branding?.logo ? (
            <img
              className="h-8 w-auto"
              src={tenant.settings.branding.logo}
              alt={tenant.name}
            />
          ) : (
            <div className="text-xl font-bold text-primary-600">
              {tenant?.name || 'ADX CORE'}
            </div>
          )}
        </div>
      </div>

      {/* Navigation */}
      <nav className="flex flex-1 flex-col px-6">
        <ul role="list" className="flex flex-1 flex-col gap-y-7">
          <li>
            <ul role="list" className="-mx-2 space-y-1">
              {navigation.map((item) => {
                const isActive = currentPath === item.href
                return (
                  <li key={item.name}>
                    <Link
                      to={item.href}
                      className={`group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold ${
                        isActive
                          ? 'bg-primary-50 text-primary-600 dark:bg-primary-900/20 dark:text-primary-400'
                          : 'text-gray-700 hover:text-primary-600 hover:bg-gray-50 dark:text-gray-300 dark:hover:text-primary-400 dark:hover:bg-gray-800'
                      }`}
                    >
                      <item.icon className="h-6 w-6 shrink-0" />
                      {item.name}
                    </Link>
                  </li>
                )
              })}
            </ul>
          </li>

          {/* Bottom section */}
          <li className="mt-auto">
            <div className="space-y-1">
              <Link
                to="/settings"
                className="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-primary-600 hover:bg-gray-50 dark:text-gray-300 dark:hover:text-primary-400 dark:hover:bg-gray-800"
              >
                <Cog8ToothIcon className="h-6 w-6 shrink-0" />
                {t('navigation.settings')}
              </Link>
              <Link
                to="/profile"
                className="group flex gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-primary-600 hover:bg-gray-50 dark:text-gray-300 dark:hover:text-primary-400 dark:hover:bg-gray-800"
              >
                <UserCircleIcon className="h-6 w-6 shrink-0" />
                {t('navigation.profile')}
              </Link>
              <button
                onClick={logout}
                className="group flex w-full gap-x-3 rounded-md p-2 text-sm leading-6 font-semibold text-gray-700 hover:text-red-600 hover:bg-red-50 dark:text-gray-300 dark:hover:text-red-400 dark:hover:bg-red-900/20"
              >
                <svg className="h-6 w-6 shrink-0" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
                  <path strokeLinecap="round" strokeLinejoin="round" d="M15.75 9V5.25A2.25 2.25 0 0013.5 3h-6a2.25 2.25 0 00-2.25 2.25v13.5A2.25 2.25 0 007.5 21h6a2.25 2.25 0 002.25-2.25V15M12 9l-3 3m0 0l3 3m-3-3h12.75" />
                </svg>
                {t('common.logout')}
              </button>
            </div>
          </li>
        </ul>
      </nav>
    </>
  )
}