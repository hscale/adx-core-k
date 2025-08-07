import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { 
  PlusIcon,
  PlayIcon,
  PauseIcon,
  StopIcon,
  MagnifyingGlassIcon,
  FunnelIcon,
  EllipsisVerticalIcon,
} from '@heroicons/react/24/outline'

import { apiService } from '@/services/api'
import { Workflow, PaginatedResponse } from '@/types'
import Button from '@/components/ui/Button'
import LoadingSpinner from '@/components/ui/LoadingSpinner'

export default function WorkflowsPage() {
  const { t } = useTranslation()
  const [searchQuery, setSearchQuery] = useState('')
  const [currentPage, setCurrentPage] = useState(1)
  const [selectedStatus, setSelectedStatus] = useState<string>('all')

  // Fetch workflows
  const {
    data: workflowsData,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['workflows', currentPage, searchQuery, selectedStatus],
    queryFn: () => apiService.get<PaginatedResponse<Workflow>>('/workflows', {
      params: {
        page: currentPage,
        limit: 10,
        search: searchQuery || undefined,
        status: selectedStatus !== 'all' ? selectedStatus : undefined,
      },
    }),
  })

  const workflows = workflowsData?.data || []
  const pagination = workflowsData?.pagination

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
    setCurrentPage(1)
  }

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'active':
        return 'bg-green-100 text-green-800 dark:bg-green-900/20 dark:text-green-400'
      case 'draft':
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
      case 'paused':
        return 'bg-yellow-100 text-yellow-800 dark:bg-yellow-900/20 dark:text-yellow-400'
      case 'archived':
        return 'bg-red-100 text-red-800 dark:bg-red-900/20 dark:text-red-400'
      default:
        return 'bg-gray-100 text-gray-800 dark:bg-gray-900/20 dark:text-gray-400'
    }
  }

  const getStatusIcon = (status: string) => {
    switch (status) {
      case 'active':
        return <PlayIcon className="h-4 w-4 text-green-600 dark:text-green-400" />
      case 'paused':
        return <PauseIcon className="h-4 w-4 text-yellow-600 dark:text-yellow-400" />
      case 'archived':
        return <StopIcon className="h-4 w-4 text-red-600 dark:text-red-400" />
      default:
        return null
    }
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
          Failed to load workflows: {(error as any).message}
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
            {t('workflows.title')}
          </h1>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Automate your business processes with AI-powered workflows
          </p>
        </div>
        <Button leftIcon={<PlusIcon className="h-4 w-4" />}>
          {t('workflows.createWorkflow')}
        </Button>
      </div>

      {/* Filters */}
      <div className="card">
        <div className="card-body">
          <div className="flex flex-col sm:flex-row gap-4">
            {/* Search */}
            <form onSubmit={handleSearch} className="flex-1">
              <div className="relative">
                <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search workflows..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="input pl-10"
                />
              </div>
            </form>

            {/* Status filter */}
            <div className="flex items-center gap-2">
              <FunnelIcon className="h-4 w-4 text-gray-400" />
              <select
                value={selectedStatus}
                onChange={(e) => setSelectedStatus(e.target.value)}
                className="input min-w-[120px]"
              >
                <option value="all">All Status</option>
                <option value="draft">Draft</option>
                <option value="active">Active</option>
                <option value="paused">Paused</option>
                <option value="archived">Archived</option>
              </select>
            </div>
          </div>
        </div>
      </div>

      {/* Workflows grid */}
      {workflows.length === 0 ? (
        <div className="card">
          <div className="card-body text-center py-12">
            <svg className="mx-auto h-12 w-12 text-gray-400" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
              <path strokeLinecap="round" strokeLinejoin="round" d="M9.594 3.94c.09-.542.56-.94 1.11-.94h2.593c.55 0 1.02.398 1.11.94l.213 1.281c.063.374.313.686.645.87.074.04.147.083.22.127.324.196.72.257 1.075.124l1.217-.456a1.125 1.125 0 011.37.49l1.296 2.247a1.125 1.125 0 01-.26 1.431l-1.003.827c-.293.24-.438.613-.431.992a6.759 6.759 0 010 .255c-.007.378.138.75.43.99l1.005.828c.424.35.534.954.26 1.43l-1.298 2.247a1.125 1.125 0 01-1.369.491l-1.217-.456c-.355-.133-.75-.072-1.076.124a6.57 6.57 0 01-.22.128c-.331.183-.581.495-.644.869l-.213 1.28c-.09.543-.56.941-1.11.941h-2.594c-.55 0-1.02-.398-1.11-.94l-.213-1.281c-.062-.374-.312-.686-.644-.87a6.52 6.52 0 01-.22-.127c-.325-.196-.72-.257-1.076-.124l-1.217.456a1.125 1.125 0 01-1.369-.49l-1.297-2.247a1.125 1.125 0 01.26-1.431l1.004-.827c.292-.24.437-.613.43-.992a6.932 6.932 0 010-.255c.007-.378-.138-.75-.43-.99l-1.004-.828a1.125 1.125 0 01-.26-1.43l1.297-2.247a1.125 1.125 0 011.37-.491l1.216.456c.356.133.751.072 1.076-.124.072-.044.146-.087.22-.128.332-.183.582-.495.644-.869l.214-1.281z" />
              <path strokeLinecap="round" strokeLinejoin="round" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
            </svg>
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
              No workflows
            </h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
              Get started by creating your first automated workflow.
            </p>
            <div className="mt-6">
              <Button leftIcon={<PlusIcon className="h-4 w-4" />}>
                Create workflow
              </Button>
            </div>
          </div>
        </div>
      ) : (
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {workflows.map((workflow) => (
            <div key={workflow.id} className="card hover:shadow-md transition-shadow">
              <div className="card-body">
                <div className="flex items-start justify-between">
                  <div className="flex-1">
                    <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                      {workflow.name}
                    </h3>
                    {workflow.description && (
                      <p className="mt-1 text-sm text-gray-500 dark:text-gray-400 line-clamp-2">
                        {workflow.description}
                      </p>
                    )}
                  </div>
                  <button className="text-gray-400 hover:text-gray-500 dark:hover:text-gray-300">
                    <EllipsisVerticalIcon className="h-5 w-5" />
                  </button>
                </div>

                <div className="mt-4 flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    {getStatusIcon(workflow.status)}
                    <span className={`inline-flex px-2 py-1 text-xs font-semibold rounded-full ${getStatusColor(workflow.status)}`}>
                      {t(`workflows.status.${workflow.status}`)}
                    </span>
                  </div>
                  <div className="text-xs text-gray-500 dark:text-gray-400">
                    {workflow.definition.steps.length} steps
                  </div>
                </div>

                <div className="mt-4 text-xs text-gray-500 dark:text-gray-400">
                  Created {new Date(workflow.createdAt).toLocaleDateString()}
                </div>
              </div>

              <div className="card-footer">
                <div className="flex gap-2">
                  {workflow.status === 'draft' && (
                    <Button size="sm" variant="outline" className="flex-1">
                      Edit
                    </Button>
                  )}
                  {workflow.status === 'active' && (
                    <Button size="sm" variant="outline" className="flex-1">
                      <PauseIcon className="h-4 w-4 mr-1" />
                      Pause
                    </Button>
                  )}
                  {workflow.status === 'paused' && (
                    <Button size="sm" variant="outline" className="flex-1">
                      <PlayIcon className="h-4 w-4 mr-1" />
                      Resume
                    </Button>
                  )}
                  <Button size="sm" className="flex-1">
                    <PlayIcon className="h-4 w-4 mr-1" />
                    Run
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