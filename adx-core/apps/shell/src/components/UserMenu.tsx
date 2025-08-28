import React, { useState } from 'react';
import { UserIcon, SettingsIcon, LogOutIcon, ChevronDownIcon } from 'lucide-react';
import { useUserContext } from '@adx-core/shared-context';
import { Button } from '@adx-core/design-system';

const UserMenu: React.FC = () => {
  const { currentUser, logout } = useUserContext();
  const [isOpen, setIsOpen] = useState(false);

  const handleLogout = async () => {
    try {
      await logout();
    } catch (error) {
      console.error('Logout failed:', error);
    }
  };

  if (!currentUser) {
    return (
      <Button
        variant="outline"
        size="sm"
        onClick={() => window.location.href = '/auth/login'}
      >
        Sign In
      </Button>
    );
  }

  return (
    <div className="relative">
      <Button
        variant="ghost"
        size="sm"
        onClick={() => setIsOpen(!isOpen)}
        className="flex items-center space-x-2"
      >
        <div className="w-8 h-8 bg-blue-500 rounded-full flex items-center justify-center">
          <UserIcon className="h-4 w-4 text-white" />
        </div>
        <span className="hidden md:block">{currentUser.name}</span>
        <ChevronDownIcon className="h-4 w-4" />
      </Button>

      {isOpen && (
        <div className="absolute right-0 mt-2 w-48 bg-white dark:bg-gray-800 rounded-md shadow-lg ring-1 ring-black ring-opacity-5 z-50">
          <div className="py-1">
            <div className="px-4 py-2 border-b border-gray-200 dark:border-gray-700">
              <p className="text-sm font-medium text-gray-900 dark:text-white">
                {currentUser.name}
              </p>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {currentUser.email}
              </p>
            </div>
            
            <button
              onClick={() => {
                setIsOpen(false);
                window.location.href = '/users/profile';
              }}
              className="flex items-center w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
            >
              <SettingsIcon className="h-4 w-4 mr-3" />
              Profile Settings
            </button>
            
            <button
              onClick={handleLogout}
              className="flex items-center w-full px-4 py-2 text-sm text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700"
            >
              <LogOutIcon className="h-4 w-4 mr-3" />
              Sign Out
            </button>
          </div>
        </div>
      )}
    </div>
  );
};

export default UserMenu;