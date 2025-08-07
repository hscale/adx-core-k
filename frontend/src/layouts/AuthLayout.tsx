import { Outlet } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { usePlatform } from '@/hooks/usePlatform'

export default function AuthLayout() {
  const { t } = useTranslation()
  const { platform } = usePlatform()

  return (
    <div className="min-h-screen flex">
      {/* Left side - Branding */}
      <div className="hidden lg:flex lg:w-1/2 bg-gradient-to-br from-primary-600 to-primary-800 relative overflow-hidden">
        <div className="absolute inset-0 bg-black/20" />
        <div className="relative z-10 flex flex-col justify-center px-12 text-white">
          <div className="max-w-md">
            <h1 className="text-4xl font-bold mb-6">
              ADX CORE
            </h1>
            <p className="text-xl text-primary-100 mb-8">
              AI-Powered Performance Excellence Platform
            </p>
            <div className="space-y-4">
              <div className="flex items-center">
                <div className="w-2 h-2 bg-primary-300 rounded-full mr-3" />
                <span>Multi-tenant SaaS platform</span>
              </div>
              <div className="flex items-center">
                <div className="w-2 h-2 bg-primary-300 rounded-full mr-3" />
                <span>AI-powered workflow automation</span>
              </div>
              <div className="flex items-center">
                <div className="w-2 h-2 bg-primary-300 rounded-full mr-3" />
                <span>Enterprise-grade security</span>
              </div>
              <div className="flex items-center">
                <div className="w-2 h-2 bg-primary-300 rounded-full mr-3" />
                <span>Universal cross-platform support</span>
              </div>
            </div>
          </div>
        </div>
        
        {/* Decorative elements */}
        <div className="absolute top-0 right-0 w-64 h-64 bg-primary-500/20 rounded-full -translate-y-32 translate-x-32" />
        <div className="absolute bottom-0 left-0 w-48 h-48 bg-primary-400/20 rounded-full translate-y-24 -translate-x-24" />
      </div>

      {/* Right side - Auth forms */}
      <div className="flex-1 flex flex-col justify-center py-12 px-4 sm:px-6 lg:px-20 xl:px-24">
        <div className="mx-auto w-full max-w-sm lg:w-96">
          {/* Mobile logo */}
          <div className="lg:hidden text-center mb-8">
            <h1 className="text-2xl font-bold text-gray-900 dark:text-white">
              ADX CORE
            </h1>
            <p className="text-sm text-gray-600 dark:text-gray-400 mt-1">
              AI-Powered Performance Excellence
            </p>
          </div>

          <Outlet />

          {/* Platform indicator */}
          {platform.type !== 'web' && (
            <div className="mt-8 text-center">
              <span className="text-xs text-gray-500 dark:text-gray-400">
                {platform.type === 'desktop' ? '🖥️ Desktop App' : '📱 Mobile App'} • v{platform.version}
              </span>
            </div>
          )}
        </div>
      </div>
    </div>
  )
}