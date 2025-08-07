import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useQuery } from '@tanstack/react-query'
import { 
  CloudArrowUpIcon,
  FolderPlusIcon,
  MagnifyingGlassIcon,
  ViewColumnsIcon,
  ListBulletIcon,
  DocumentIcon,
  FolderIcon,
} from '@heroicons/react/24/outline'

import { apiService } from '@/services/api'
import { FileItem } from '@/types'
import Button from '@/components/ui/Button'
import LoadingSpinner from '@/components/ui/LoadingSpinner'

type ViewMode = 'grid' | 'list'

export default function FilesPage() {
  const { t } = useTranslation()
  const [searchQuery, setSearchQuery] = useState('')
  const [viewMode, setViewMode] = useState<ViewMode>('grid')
  const [currentPath, setCurrentPath] = useState('/')

  // Fetch files
  const {
    data: files = [],
    isLoading,
    error,
  } = useQuery({
    queryKey: ['files', currentPath, searchQuery],
    queryFn: () => apiService.get<FileItem[]>('/files', {
      params: {
        path: currentPath,
        search: searchQuery || undefined,
      },
    }),
  })

  const handleSearch = (e: React.FormEvent) => {
    e.preventDefault()
  }

  const formatFileSize = (bytes?: number) => {
    if (!bytes) return '-'
    const sizes = ['B', 'KB', 'MB', 'GB', 'TB']
    const i = Math.floor(Math.log(bytes) / Math.log(1024))
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`
  }

  const getFileIcon = (file: FileItem) => {
    if (file.type === 'folder') {
      return <FolderIcon className="h-8 w-8 text-blue-500" />
    }
    return <DocumentIcon className="h-8 w-8 text-gray-400" />
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
          Failed to load files: {(error as any).message}
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
            {t('files.title')}
          </h1>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            Manage and organize your files and folders
          </p>
        </div>
        <div className="flex gap-2">
          <Button 
            variant="outline" 
            leftIcon={<FolderPlusIcon className="h-4 w-4" />}
          >
            {t('files.createFolder')}
          </Button>
          <Button leftIcon={<CloudArrowUpIcon className="h-4 w-4" />}>
            {t('files.upload')}
          </Button>
        </div>
      </div>

      {/* Toolbar */}
      <div className="card">
        <div className="card-body">
          <div className="flex flex-col sm:flex-row gap-4 items-center justify-between">
            {/* Search */}
            <form onSubmit={handleSearch} className="flex-1 max-w-md">
              <div className="relative">
                <MagnifyingGlassIcon className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
                <input
                  type="text"
                  placeholder="Search files..."
                  value={searchQuery}
                  onChange={(e) => setSearchQuery(e.target.value)}
                  className="input pl-10"
                />
              </div>
            </form>

            {/* View mode toggle */}
            <div className="flex rounded-lg border border-gray-200 dark:border-gray-700">
              <button
                onClick={() => setViewMode('grid')}
                className={`p-2 rounded-l-lg ${
                  viewMode === 'grid'
                    ? 'bg-primary-50 text-primary-600 dark:bg-primary-900/20 dark:text-primary-400'
                    : 'text-gray-400 hover:text-gray-500 dark:hover:text-gray-300'
                }`}
              >
                <ViewColumnsIcon className="h-4 w-4" />
              </button>
              <button
                onClick={() => setViewMode('list')}
                className={`p-2 rounded-r-lg border-l border-gray-200 dark:border-gray-700 ${
                  viewMode === 'list'
                    ? 'bg-primary-50 text-primary-600 dark:bg-primary-900/20 dark:text-primary-400'
                    : 'text-gray-400 hover:text-gray-500 dark:hover:text-gray-300'
                }`}
              >
                <ListBulletIcon className="h-4 w-4" />
              </button>
            </div>
          </div>
        </div>
      </div>

      {/* Breadcrumb */}
      {currentPath !== '/' && (
        <nav className="flex" aria-label="Breadcrumb">
          <ol className="flex items-center space-x-4">
            <li>
              <button
                onClick={() => setCurrentPath('/')}
                className="text-gray-400 hover:text-gray-500 dark:hover:text-gray-300"
              >
                Home
              </button>
            </li>
            {currentPath.split('/').filter(Boolean).map((segment, index, array) => (
              <li key={index} className="flex items-center">
                <svg className="flex-shrink-0 h-4 w-4 text-gray-300 dark:text-gray-600 mx-2" fill="currentColor" viewBox="0 0 20 20">
                  <path fillRule="evenodd" d="M7.293 14.707a1 1 0 010-1.414L10.586 10 7.293 6.707a1 1 0 011.414-1.414l4 4a1 1 0 010 1.414l-4 4a1 1 0 01-1.414 0z" clipRule="evenodd" />
                </svg>
                <span className="text-sm font-medium text-gray-500 dark:text-gray-400">
                  {segment}
                </span>
              </li>
            ))}
          </ol>
        </nav>
      )}

      {/* Files content */}
      <div className="card">
        {files.length === 0 ? (
          <div className="card-body text-center py-12">
            <FolderIcon className="mx-auto h-12 w-12 text-gray-400" />
            <h3 className="mt-2 text-sm font-medium text-gray-900 dark:text-white">
              No files
            </h3>
            <p className="mt-1 text-sm text-gray-500 dark:text-gray-400">
              Get started by uploading a file or creating a folder.
            </p>
            <div className="mt-6 flex gap-2 justify-center">
              <Button size="sm" leftIcon={<CloudArrowUpIcon className="h-4 w-4" />}>
                Upload files
              </Button>
              <Button 
                variant="outline" 
                size="sm" 
                leftIcon={<FolderPlusIcon className="h-4 w-4" />}
              >
                New folder
              </Button>
            </div>
          </div>
        ) : viewMode === 'grid' ? (
          <div className="card-body">
            <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-6 gap-4">
              {files.map((file) => (
                <div
                  key={file.id}
                  className="group relative p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:border-primary-300 dark:hover:border-primary-600 cursor-pointer transition-colors"
                  onClick={() => {
                    if (file.type === 'folder') {
                      setCurrentPath(`${currentPath}${file.name}/`.replace('//', '/'))
                    }
                  }}
                >
                  <div className="flex flex-col items-center text-center">
                    {getFileIcon(file)}
                    <div className="mt-2 text-sm font-medium text-gray-900 dark:text-white truncate w-full">
                      {file.name}
                    </div>
                    {file.type === 'file' && (
                      <div className="text-xs text-gray-500 dark:text-gray-400">
                        {formatFileSize(file.size)}
                      </div>
                    )}
                  </div>
                </div>
              ))}
            </div>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200 dark:divide-gray-700">
              <thead className="bg-gray-50 dark:bg-gray-800">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('files.properties.name')}
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('files.properties.size')}
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('files.properties.type')}
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('files.properties.modified')}
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white dark:bg-gray-900 divide-y divide-gray-200 dark:divide-gray-700">
                {files.map((file) => (
                  <tr 
                    key={file.id} 
                    className="hover:bg-gray-50 dark:hover:bg-gray-800 cursor-pointer"
                    onClick={() => {
                      if (file.type === 'folder') {
                        setCurrentPath(`${currentPath}${file.name}/`.replace('//', '/'))
                      }
                    }}
                  >
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        {getFileIcon(file)}
                        <div className="ml-3">
                          <div className="text-sm font-medium text-gray-900 dark:text-white">
                            {file.name}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {formatFileSize(file.size)}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {file.type === 'folder' ? 'Folder' : file.mimeType || 'File'}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 dark:text-gray-400">
                      {new Date(file.updatedAt).toLocaleDateString()}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  )
}