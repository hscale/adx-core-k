import React, { useState, useMemo } from 'react';
import { 
  Search, 
  Filter, 
  Calendar,
  Download,
  Eye,
  RefreshCw,
  ChevronLeft,
  ChevronRight,
  AlertCircle
} from 'lucide-react';
import { useWorkflows } from '../hooks';
import { 
  WorkflowStatus, 
  WorkflowSearchParams,
  WorkflowFilter 
} from '../types';
import {
  formatWorkflowDuration,
  formatWorkflowTimestamp,
  formatRelativeTime,
  getWorkflowStatusColor,
  getWorkflowStatusIcon,
  getWorkflowTypeDisplayName,
  calculateSuccessRate,
  calculateAverageDuration
} from '../utils';

const WorkflowHistory: React.FC = () => {
  const [searchParams, setSearchParams] = useState<WorkflowSearchParams>({
    page: 1,
    limit: 20,
    sortBy: 'startedAt',
    sortOrder: 'desc',
  });
  const [showFilters, setShowFilters] = useState(false);
  const [selectedDateRange, setSelectedDateRange] = useState({
    start: '',
    end: '',
  });

  const { data: workflowData, isLoading, error, refetch } = useWorkflows(searchParams);

  const workflows = workflowData?.workflows || [];
  const totalWorkflows = workflowData?.total || 0;
  const hasMore = workflowData?.hasMore || false;

  // Calculate summary statistics
  const summaryStats = useMemo(() => {
    if (workflows.length === 0) return null;

    const completedWorkflows = workflows.filter(w => w.status === WorkflowStatus.COMPLETED);
    const failedWorkflows = workflows.filter(w => w.status === WorkflowStatus.FAILED);
    
    return {
      total: workflows.length,
      completed: completedWorkflows.length,
      failed: failedWorkflows.length,
      successRate: calculateSuccessRate(workflows),
      averageDuration: calculateAverageDuration(workflows),
    };
  }, [workflows]);

  const handleSearch = (query: string) => {
    setSearchParams(prev => ({
      ...prev,
      query,
      page: 1,
    }));
  };

  const handleFilterChange = (filter: Partial<WorkflowFilter>) => {
    setSearchParams(prev => ({
      ...prev,
      filter: {
        ...prev.filter,
        ...filter,
      },
      page: 1,
    }));
  };

  const handleDateRangeFilter = () => {
    if (selectedDateRange.start && selectedDateRange.end) {
      handleFilterChange({
        dateRange: selectedDateRange,
      });
    }
  };

  const handleSortChange = (sortBy: string) => {
    setSearchParams(prev => ({
      ...prev,
      sortBy,
      sortOrder: prev.sortBy === sortBy && prev.sortOrder === 'desc' ? 'asc' : 'desc',
      page: 1,
    }));
  };

  const handlePageChange = (page: number) => {
    setSearchParams(prev => ({
      ...prev,
      page,
    }));
  };

  const handleExport = () => {
    // TODO: Implement export functionality
    console.log('Export workflows');
  };

  const getStatusBadge = (status: WorkflowStatus) => {
    const color = getWorkflowStatusColor(status);
    const icon = getWorkflowStatusIcon(status);
    
    return (
      <span className={`
        inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium
        workflow-status-${status}
      `}>
        <span className="mr-1">{icon}</span>
        {status}
      </span>
    );
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-md p-4">
        <div className="flex items-center">
          <AlertCircle className="h-5 w-5 text-red-600 mr-2" />
          <span className="text-red-800">Failed to load workflow history</span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Workflow History</h1>
          <p className="text-gray-600">
            Browse and analyze past workflow executions
          </p>
        </div>
        <div className="flex space-x-2">
          <button
            onClick={handleExport}
            className="flex items-center space-x-2 px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
          >
            <Download className="h-4 w-4" />
            <span>Export</span>
          </button>
          <button
            onClick={() => refetch()}
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            <RefreshCw className="h-4 w-4" />
            <span>Refresh</span>
          </button>
        </div>
      </div>

      {/* Summary Statistics */}
      {summaryStats && (
        <div className="grid grid-cols-1 md:grid-cols-5 gap-4">
          <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
            <div className="text-2xl font-bold text-gray-900">{summaryStats.total}</div>
            <div className="text-sm text-gray-600">Total Workflows</div>
          </div>
          <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
            <div className="text-2xl font-bold text-green-600">{summaryStats.completed}</div>
            <div className="text-sm text-gray-600">Completed</div>
          </div>
          <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
            <div className="text-2xl font-bold text-red-600">{summaryStats.failed}</div>
            <div className="text-sm text-gray-600">Failed</div>
          </div>
          <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
            <div className="text-2xl font-bold text-blue-600">{summaryStats.successRate}%</div>
            <div className="text-sm text-gray-600">Success Rate</div>
          </div>
          <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
            <div className="text-2xl font-bold text-purple-600">
              {formatWorkflowDuration(summaryStats.averageDuration)}
            </div>
            <div className="text-sm text-gray-600">Avg Duration</div>
          </div>
        </div>
      )}

      {/* Search and Filters */}
      <div className="bg-white p-4 rounded-lg shadow-sm border border-gray-200">
        <div className="flex items-center space-x-4 mb-4">
          <div className="flex-1 relative">
            <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
            <input
              type="text"
              placeholder="Search workflows..."
              value={searchParams.query || ''}
              onChange={(e) => handleSearch(e.target.value)}
              className="w-full pl-10 pr-4 py-2 border border-gray-300 rounded-md focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          <button
            onClick={() => setShowFilters(!showFilters)}
            className="flex items-center space-x-2 px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
          >
            <Filter className="h-4 w-4" />
            <span>Filters</span>
          </button>
        </div>

        {showFilters && (
          <div className="border-t pt-4 space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {/* Status Filter */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Status
                </label>
                <select
                  multiple
                  className="w-full border border-gray-300 rounded-md p-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  onChange={(e) => {
                    const selectedStatuses = Array.from(e.target.selectedOptions, option => option.value) as WorkflowStatus[];
                    handleFilterChange({ status: selectedStatuses });
                  }}
                >
                  {Object.values(WorkflowStatus).map(status => (
                    <option key={status} value={status}>
                      {status}
                    </option>
                  ))}
                </select>
              </div>

              {/* Date Range Filter */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Date Range
                </label>
                <div className="space-y-2">
                  <input
                    type="date"
                    value={selectedDateRange.start}
                    onChange={(e) => setSelectedDateRange(prev => ({ ...prev, start: e.target.value }))}
                    className="w-full border border-gray-300 rounded-md p-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                  <input
                    type="date"
                    value={selectedDateRange.end}
                    onChange={(e) => setSelectedDateRange(prev => ({ ...prev, end: e.target.value }))}
                    className="w-full border border-gray-300 rounded-md p-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                  />
                  <button
                    onClick={handleDateRangeFilter}
                    className="w-full px-3 py-1 bg-blue-600 text-white rounded text-sm hover:bg-blue-700 transition-colors"
                  >
                    Apply Date Filter
                  </button>
                </div>
              </div>

              {/* Quick Filters */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-2">
                  Quick Filters
                </label>
                <div className="space-y-2">
                  <button
                    onClick={() => handleFilterChange({ status: [WorkflowStatus.FAILED] })}
                    className="w-full px-3 py-2 text-left border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
                  >
                    Failed Workflows
                  </button>
                  <button
                    onClick={() => handleFilterChange({ status: [WorkflowStatus.COMPLETED] })}
                    className="w-full px-3 py-2 text-left border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
                  >
                    Completed Workflows
                  </button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Workflow List */}
      <div className="bg-white rounded-lg shadow overflow-hidden">
        {workflows.length === 0 ? (
          <div className="p-8 text-center">
            <Calendar className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 mb-2">
              No workflows found
            </h3>
            <p className="text-gray-600">
              No workflows match your search criteria.
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th 
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100"
                    onClick={() => handleSortChange('type')}
                  >
                    Workflow Type
                  </th>
                  <th 
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100"
                    onClick={() => handleSortChange('status')}
                  >
                    Status
                  </th>
                  <th 
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100"
                    onClick={() => handleSortChange('startedAt')}
                  >
                    Started
                  </th>
                  <th 
                    className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider cursor-pointer hover:bg-gray-100"
                    onClick={() => handleSortChange('duration')}
                  >
                    Duration
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    User
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {workflows.map((workflow) => (
                  <tr key={workflow.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4">
                      <div>
                        <div className="text-sm font-medium text-gray-900">
                          {getWorkflowTypeDisplayName(workflow.type)}
                        </div>
                        <div className="text-sm text-gray-500">
                          {workflow.id}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4">
                      {getStatusBadge(workflow.status)}
                    </td>
                    <td className="px-6 py-4">
                      <div>
                        <div className="text-sm text-gray-900">
                          {formatWorkflowTimestamp(workflow.startedAt)}
                        </div>
                        <div className="text-sm text-gray-500">
                          {formatRelativeTime(workflow.startedAt)}
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900">
                      {workflow.duration 
                        ? formatWorkflowDuration(workflow.duration)
                        : '-'
                      }
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-900">
                      {workflow.metadata.userEmail}
                    </td>
                    <td className="px-6 py-4">
                      <button
                        className="text-blue-600 hover:text-blue-800 transition-colors"
                        title="View Details"
                      >
                        <Eye className="h-4 w-4" />
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}

        {/* Pagination */}
        {workflows.length > 0 && (
          <div className="bg-white px-4 py-3 flex items-center justify-between border-t border-gray-200 sm:px-6">
            <div className="flex-1 flex justify-between sm:hidden">
              <button
                onClick={() => handlePageChange(searchParams.page! - 1)}
                disabled={searchParams.page === 1}
                className="relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Previous
              </button>
              <button
                onClick={() => handlePageChange(searchParams.page! + 1)}
                disabled={!hasMore}
                className="ml-3 relative inline-flex items-center px-4 py-2 border border-gray-300 text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
              >
                Next
              </button>
            </div>
            <div className="hidden sm:flex-1 sm:flex sm:items-center sm:justify-between">
              <div>
                <p className="text-sm text-gray-700">
                  Showing{' '}
                  <span className="font-medium">
                    {((searchParams.page! - 1) * searchParams.limit!) + 1}
                  </span>{' '}
                  to{' '}
                  <span className="font-medium">
                    {Math.min(searchParams.page! * searchParams.limit!, totalWorkflows)}
                  </span>{' '}
                  of{' '}
                  <span className="font-medium">{totalWorkflows}</span>{' '}
                  results
                </p>
              </div>
              <div>
                <nav className="relative z-0 inline-flex rounded-md shadow-sm -space-x-px">
                  <button
                    onClick={() => handlePageChange(searchParams.page! - 1)}
                    disabled={searchParams.page === 1}
                    className="relative inline-flex items-center px-2 py-2 rounded-l-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <ChevronLeft className="h-5 w-5" />
                  </button>
                  <span className="relative inline-flex items-center px-4 py-2 border border-gray-300 bg-white text-sm font-medium text-gray-700">
                    Page {searchParams.page}
                  </span>
                  <button
                    onClick={() => handlePageChange(searchParams.page! + 1)}
                    disabled={!hasMore}
                    className="relative inline-flex items-center px-2 py-2 rounded-r-md border border-gray-300 bg-white text-sm font-medium text-gray-500 hover:bg-gray-50 disabled:opacity-50 disabled:cursor-not-allowed"
                  >
                    <ChevronRight className="h-5 w-5" />
                  </button>
                </nav>
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  );
};

export default WorkflowHistory;