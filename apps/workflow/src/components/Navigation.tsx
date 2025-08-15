import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { 
  Activity, 
  BarChart3, 
  Clock, 
  Settings, 
  Workflow 
} from 'lucide-react';

const Navigation: React.FC = () => {
  const location = useLocation();

  const navItems = [
    {
      path: '/',
      label: 'Dashboard',
      icon: Activity,
    },
    {
      path: '/monitor',
      label: 'Monitor',
      icon: Workflow,
    },
    {
      path: '/history',
      label: 'History',
      icon: Clock,
    },
    {
      path: '/analytics',
      label: 'Analytics',
      icon: BarChart3,
    },
    {
      path: '/management',
      label: 'Management',
      icon: Settings,
    },
  ];

  const isActive = (path: string) => {
    if (path === '/') {
      return location.pathname === '/';
    }
    return location.pathname.startsWith(path);
  };

  return (
    <nav className="bg-white shadow-sm border-b border-gray-200">
      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-16">
          <div className="flex items-center space-x-8">
            <div className="flex items-center space-x-2">
              <Workflow className="h-8 w-8 text-blue-600" />
              <h1 className="text-xl font-semibold text-gray-900">
                Workflow Management
              </h1>
            </div>
            
            <div className="flex space-x-1">
              {navItems.map((item) => {
                const Icon = item.icon;
                return (
                  <Link
                    key={item.path}
                    to={item.path}
                    className={`
                      flex items-center space-x-2 px-3 py-2 rounded-md text-sm font-medium transition-colors
                      ${
                        isActive(item.path)
                          ? 'bg-blue-100 text-blue-700'
                          : 'text-gray-600 hover:text-gray-900 hover:bg-gray-100'
                      }
                    `}
                  >
                    <Icon className="h-4 w-4" />
                    <span>{item.label}</span>
                  </Link>
                );
              })}
            </div>
          </div>
          
          <div className="flex items-center space-x-4">
            <div className="text-sm text-gray-600">
              Workflow Management v1.0
            </div>
          </div>
        </div>
      </div>
    </nav>
  );
};

export default Navigation;