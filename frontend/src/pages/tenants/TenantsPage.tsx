import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { 
  PlusIcon,
  BuildingOfficeIcon,
  MagnifyingGlassIcon,
  EllipsisVerticalIcon,
  GlobeAltIcon,
  UserGroupIcon,
} from '@heroicons/react/24/outline'

import { apiService } from '@/services/api'
import { Tenant, PaginatedResponse } from '@/types'
import Button from '@/components/ui/Button'
import LoadingSpinner from '@/components/ui/LoadingSpinner'

export default function TenantsPage() {
  const { t } = useTranslation()
  const [searchQuery, setSearchQuery] = useState('')
  const [currentPage, setCurrentPage] = useState(1)

  // Fetch tenants
  const {
    data: tenantsData,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['tenants', currentPage, searchQuery],
    queryFn: () => apiService.get<PaginatedResponse<Tenant>>('/tenants', {
      params: {
        page: currentPage,
        limit: 12,
        search: searchQuery || undefined,
      },
    }),
  })

  const tenants = tenantsData?.data || []
  const pagination = tenantsData?.pagination

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    setCurrentPage(1)
  }

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <LoadingSpinner size="lg" />
      </div>
    )
  }

  if (error) {
    return (
      <div className="text-center py-12">
        <div className="text-red-600 dark:text-red-400">
          Failed to load tenants: {(error as any).message}
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            {t('tenants.title')}
          </h1>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Manage organizations and their configurations
          </p>
        </div>
        <Button leftIcon={<PlusIcon className="h-4 w-4" />}>
          Add Tenant
        </Button>
      </div>

      {/* Search */}
      <div className="card">
        <div className="card-body">
          <form onSubmit={handleSearch} className="max-w-md">
            <div className="relative">
              <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
              <input
                type="text"
                placeholder="Search tenants..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="input pl-10"
              />
            </div>
          </form>
        </div>
      </div>

      {/* Tenants grid */}
      {tenants.length === 0 ? (
        <div className="card">
          <div className="card-body text-center py-12">
            <BuildingOfficeIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
              No tenants
            </h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
              Get started by adding your first tenant organization.
            </p>
            <div className="mt-6">
              <Button leftIcon={<PlusIcon className="h-4 w-4" />}>
                Add tenant
              </Button>
            </div>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
          {tenants.map((tenant) => (
            <div key={tenant.id} className="card hover:shadow-md transition-shadow">
              <div className="card-body">
                <div className="flex items-start justify-between">
                  <div className="flex items-center">
                    {tenant.settings?.branding?.logo ? (
                      <img
                        src={tenant.settings.branding.logo}
                        alt={tenant.name}
                        className="h-10 w-10 rounded-lg object-cover"
                      />
                    ) : (
                      <div className="h-10 w-10 rounded-lg bg-primary-100 dark:bg-primary-900/20 flex items-center justify-center">
                        <BuildingOfficeIcon className="h-6 w-6 text-primary-600 dark:text-primary-400" />
                      </div>
                    )}
                    <div className="ml-3">
                      <h3 className="text-sm font-medium text-gray-900 dark:text-white">
                        {tenant.name}
                      </h3>
                      {tenant.domain && (
                        <p className="text-xs text-gray-500 dark:text-gray-400 flex items-center mt-1">
                          <GlobeAltIcon className="h-3 w-3 mr-1" />
                          {tenant.domain}
                        </p>
                      )}
                    </div>
                  </div>
                  <button className="text-gray-400 hover:text-gray-500 dark:hover:text-gray-300">
                    <EllipsisVerticalIcon className="h-5 w-5" />
                  </button>
                </div>

                <div className="mt-4">
                  <div className="flex items-center justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Status</span>
                    <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${
                      tenant.isActive
                        ? 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
                        : 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
                    }`}>
                      {tenant.isActive ? 'Active' : 'Inactive'}
                    </span>
                  </div>

                  <div className="mt-2 flex items-center justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Features</span>
                    <span className="text-gray-900 dark:text-white">
                      {tenant.settings.features?.length || 0} enabled
                    </span>
                  </div>

                  <div className="mt-2 flex items-center justify-between text-sm">
                    <span className="text-gray-500 dark:text-gray-400">Theme</span>
                    <span className="text-gray-900 dark:text-white capitalize">
                      {tenant.settings.theme || 'auto'}
                    </span>
                  </div>
                </div>

                <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
                  <div className="flex items-center text-xs text-gray-500 dark:text-gray-400">
                    <UserGroupIcon className="h-4 w-4 mr-1" />
                    Created {new Date(tenant.createdAt).toLocaleDateString()}
                  </div>
                </div>
              </div>

              <div className="card-footer">
                <div className="flex gap-2">
                  <Button size="sm" variant="outline" className="flex-1">
                    Configure
                  </Button>
                  <Button size="sm" className="flex-1">
                    Manage
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      {/* Pagination */}
      {pagination && pagination.totalPages > 1 && (
        <div className="card">
          <div className="card-footer">
            <div className="flex items-center justify-between">
              <div className="text-sm text-gray-700 dark:text-gray-300">
                Showing {((pagination.page - 1) * pagination.limit) + 1} to{' '}
                {Math.min(pagination.page * pagination.limit, pagination.total)} of{' '}
                {pagination.total} results
              </div>
              <div className="flex gap-2">
                <Button
                  variant="outline"
                  size="sm"
                  disabled={pagination.page <= 1}
                  onClick={() => setCurrentPage(pagination.page - 1)}
                >
                  Previous
                </Button>
                <Button
                  variant="outline"
                  size="sm"
                  disabled={pagination.page >= pagination.totalPages}
                  onClick={() => setCurrentPage(pagination.page + 1)}
                >
                  Next
                </Button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}