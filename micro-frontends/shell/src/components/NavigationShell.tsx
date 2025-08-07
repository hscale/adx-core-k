import React, { ReactNode } from 'react'
import { Link, useLocation } from 'react-router-dom'
import { 
  HomeIcon, 
  UsersIcon, 
  FolderIcon, 
  CogIcon,
  WorkflowIcon,
  BuildingOfficeIcon
} from 'lucide-react'

interface NavigationShellProps {
  children: ReactNode
}

const navigation = [
  { name: 'Dashboard', href: '/', icon: HomeIcon },
  { name: 'Files', href: '/files', icon: FolderIcon },
  { name: 'Users', href: '/users', icon: UsersIcon },
  { name: 'Workflows', href: '/workflows', icon: WorkflowIcon },
  { name: 'Tenants', href: '/tenants', icon: BuildingOfficeIcon },
]

export const NavigationShell: React.FC<NavigationShellProps> = ({ children }) => {
  const location = useLocation()
  
  // Don't show navigation on auth pages
  if (location.pathname.startsWith('/auth')) {
    return <>{children}</>
  }

  return (
    <div className="flex h-screen bg-gray-100">
      {/* Sidebar */}
      <div className="flex flex-col w-64 bg-white shadow-lg">
        {/* Logo */}
        <div className="flex items-center justify-center h-16 px-4 bg-blue-600">
          <h1 className="text-xl font-bold text-white">ADX CORE</h1>
        </div>
        
        {/* Navigation */}
        <nav className="flex-1 px-4 py-6 space-y-2">
          {navigation.map((item) => {
            const isActive = location.pathname === item.href || 
              (item.href !== '/' && location.pathname.startsWith(item.href))
            
            return (
              <Link
                key={item.name}
                to={item.href}
                className={`flex items-center px-3 py-2 text-sm font-medium rounded-md transition-colors ${
                  isActive
                    ? 'bg-blue-100 text-blue-700'
                    : 'text-gray-600 hover:bg-gray-100 hover:text-gray-900'
                }`}
              >
                <item.icon className="w-5 h-5 mr-3" />
                {item.name}
              </Link>
            )
          })}
        </nav>
        
        {/* User menu */}
        <div className="p-4 border-t border-gray-200">
          <div className="flex items-center">
            <div className="flex-shrink-0">
              <div className="w-8 h-8 bg-gray-300 rounded-full flex items-center justify-center">
                <span className="text-sm font-medium text-gray-700">U</span>
              </div>
            </div>
            <div className="ml-3">
              <p className="text-sm font-medium text-gray-700">User</p>
              <p className="text-xs text-gray-500">user@example.com</p>
            </div>
          </div>
        </div>
      </div>
      
      {/* Main content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Header */}
        <header className="bg-white shadow-sm border-b border-gray-200">
          <div className="px-6 py-4">
            <div className="flex items-center justify-between">
              <h2 className="text-lg font-semibold text-gray-900">
                {navigation.find(item => 
                  location.pathname === item.href || 
                  (item.href !== '/' && location.pathname.startsWith(item.href))
                )?.name || 'Dashboard'}
              </h2>
              
              <div className="flex items-center space-x-4">
                {/* Tenant selector placeholder */}
                <select className="text-sm border border-gray-300 rounded-md px-3 py-1">
                  <option>Default Tenant</option>
                </select>
                
                {/* Settings */}
                <button className="p-2 text-gray-400 hover:text-gray-600">
                  <CogIcon className="w-5 h-5" />
                </button>
              </div>
            </div>
          </div>
        </header>
        
        {/* Page content */}
        <main className="flex-1 overflow-auto">
          {children}
        </main>
      </div>
    </div>
  )
}