import React from 'react';
import { NavLink } from 'react-router-dom';
import { Store, Package, Settings, Code } from 'lucide-react';

export const Navigation: React.FC = () => {
  const navItems = [
    {
      to: '/marketplace',
      icon: Store,
      label: 'Marketplace',
      description: 'Browse and install modules',
    },
    {
      to: '/installed',
      icon: Package,
      label: 'Installed',
      description: 'Manage installed modules',
    },
    {
      to: '/developer',
      icon: Code,
      label: 'Developer',
      description: 'Create and test modules',
    },
  ];

  return (
    <nav className="bg-white shadow-sm border-b border-gray-200">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center gap-8">
            <div className="flex items-center gap-2">
              <Package className="w-6 h-6 text-blue-600" />
              <span className="font-semibold text-gray-900">Module Manager</span>
            </div>
            
            <div className="flex items-center gap-1">
              {navItems.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  className={({ isActive }) =>
                    `flex items-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors ${
                      isActive
                        ? 'bg-blue-100 text-blue-700'
                        : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
                    }`
                  }
                  title={item.description}
                >
                  <item.icon className="w-4 h-4" />
                  {item.label}
                </NavLink>
              ))}
            </div>
          </div>
          
          <div className="flex items-center gap-4">
            <span className="text-sm text-gray-500">ADX Core Module System</span>
          </div>
        </div>
      </div>
    </nav>
  );
};