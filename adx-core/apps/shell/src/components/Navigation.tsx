import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { 
  HomeIcon, 
  UserIcon, 
  FolderIcon, 
  SettingsIcon as CogIcon, 
  GitBranchIcon as WorkflowIcon,
  PackageIcon as ModuleIcon,
  BuildingIcon as TenantIcon
} from 'lucide-react';
import { useTenantContext } from '@adx-core/shared-context';
import TenantSwitcher from './TenantSwitcher';
import UserMenu from './UserMenu';
import ThemeToggle from './ThemeToggle';
import LanguageSelector from './LanguageSelector';

const Navigation: React.FC = () => {
  const { t } = useTranslation();
  const location = useLocation();
  const { currentTenant } = useTenantContext();

  const navigationItems = [
    { path: '/', icon: HomeIcon, label: t('navigation.dashboard') },
    { path: '/users', icon: UserIcon, label: t('navigation.users') },
    { path: '/files', icon: FolderIcon, label: t('navigation.files') },
    { path: '/workflows', icon: WorkflowIcon, label: t('navigation.workflows') },
    { path: '/modules', icon: ModuleIcon, label: t('navigation.modules') },
    { path: '/tenant', icon: TenantIcon, label: t('navigation.tenant') },
  ];

  const isActive = (path: string) => {
    if (path === '/') {
      return location.pathname === '/';
    }
    return location.pathname.startsWith(path);
  };

  return (
    <nav className="bg-white dark:bg-gray-800 shadow-sm border-b border-gray-200 dark:border-gray-700">
      <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
        <div className="flex justify-between h-16">
          <div className="flex">
            <div className="flex-shrink-0 flex items-center">
              <h1 className="text-xl font-bold text-gray-900 dark:text-white">
                ADX Core
              </h1>
            </div>
            <div className="hidden sm:ml-6 sm:flex sm:space-x-8">
              {navigationItems.map((item) => {
                const Icon = item.icon;
                return (
                  <Link
                    key={item.path}
                    to={item.path}
                    className={`inline-flex items-center px-1 pt-1 border-b-2 text-sm font-medium ${
                      isActive(item.path)
                        ? 'border-blue-500 text-gray-900 dark:text-white'
                        : 'border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300 dark:text-gray-400 dark:hover:text-gray-300'
                    }`}
                  >
                    <Icon className="w-4 h-4 mr-2" />
                    {item.label}
                  </Link>
                );
              })}
            </div>
          </div>
          <div className="flex items-center space-x-4">
            <LanguageSelector />
            <ThemeToggle />
            {currentTenant && <TenantSwitcher />}
            <UserMenu />
          </div>
        </div>
      </div>
    </nav>
  );
};

export default Navigation;