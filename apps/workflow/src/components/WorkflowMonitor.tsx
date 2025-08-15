import React, { useState, useMemo } from 'react';
import { 
  Play, 
  Pause, 
  Square, 
  RefreshCw, 
  Search, 
  Filter,
  Eye,
  AlertCircle,
  CheckCircle,
  Clock,
  XCircle
} from 'lucide-react';
import { useRunningWorkflows, useCancelWorkflow, useRetryWorkflow } from '../hooks';
import { useMultipleWorkflowStreams } from '../hooks/useWorkflowStream';
import { 
  WorkflowStatus, 
  Workflow,
  WorkflowFilter 
} from '../types';
import {
  formatWorkflowDuration,
  formatRelativeTime,
  getWorkflowStatusColor,
  getWorkflowStatusIcon,
  canCancelWorkflow,
  canRetryWorkflow,
  getWorkflowTypeDisplayName,
  estimateTimeRemaining,
  formatEstimatedTimeRemaining
} from '../utils';

const WorkflowMonitor: React.FC = () => {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedWorkflows, setSelectedWorkflows] = useState<string[]>([]);
  const [showFilters, setShowFilters] = useState(false);
  const [filters, setFilters] = useState<WorkflowFilter>({});

  const { data: runningWorkflows = [], isLoading, error, refetch } = useRunningWorkflows();
  const cancelWorkflowMutation = useCancelWorkflow();
  const retryWorkflowMutation = useRetryWorkflow();

  // Stream real-time updates for running workflows
  const runningWorkflowIds = runningWorkflows.map(w => w.id);
  const { workflows: streamedWorkflows, connections, errors: streamErrors } = 
    useMultipleWorkflowStreams(runningWorkflowIds);

  // Merge streamed data with initial data
  const workflows = useMemo(() => {
    return runningWorkflows.map(workflow => 
      streamedWorkflows[workflow.id] || workflow
    );
  }, [runningWorkflows, streamedWorkflows]);

  // Filter workflows based on search and filters
  const filteredWorkflows = useMemo(() => {
    let filtered = workflows;

    // Search filter
    if (searchQuery) {
      filtered = filtered.filter(workflow =>
        workflow.type.toLowerCase().includes(searchQuery.toLowerCase()) ||
        workflow.id.toLowerCase().includes(searchQuery.toLowerCase()) ||
        workflow.metadata.userEmail.toLowerCase().includes(searchQuery.toLowerCase())
      );
    }

    // Status filter
    if (filters.status && filters.status.length > 0) {
      filtered = filtered.filter(workflow =>
        filters.status!.includes(workflow.status)
      );
    }

    // Type filter
    if (filters.type && filters.type.length > 0) {
      filtered = filtered.filter(workflow =>
        filters.type!.includes(workflow.type)
      );
    }

    return filtered;
  }, [workflows, searchQuery, filters]);

  const handleCancelWorkflow = async (workflowId: string) => {
    try {
      await cancelWorkflowMutation.mutateAsync({
        workflowId,
        reason: 'Cancelled by user from monitor'
      });
    } catch (error) {
      console.error('Failed to cancel workflow:', error);
    }
  };

  const handleRetryWorkflow = async (workflowId: string) => {
    try {
      await retryWorkflowMutation.mutateAsync(workflowId);
    } catch (error) {
      console.error('Failed to retry workflow:', error);
    }
  };

  const handleBulkCancel = async () => {
    if (selectedWorkflows.length === 0) return;
    
    try {
      await Promise.all(
        selectedWorkflows.map(id => 
          cancelWorkflowMutation.mutateAsync({
            workflowId: id,
            reason: 'Bulk cancelled by user'
          })
        )
      );
      setSelectedWorkflows([]);
    } catch (error) {
      console.error('Failed to bulk cancel workflows:', error);
    }
  };

  const toggleWorkflowSelection = (workflowId: string) => {
    setSelectedWorkflows(prev =>
      prev.includes(workflowId)
        ? prev.filter(id => id !== workflowId)
        : [...prev, workflowId]
    );
  };

  const selectAllWorkflows = () => {
    setSelectedWorkflows(filteredWorkflows.map(w => w.id));
  };

  const clearSelection = () => {
    setSelectedWorkflows([]);
  };

  const getStatusIcon = (status: WorkflowStatus) => {
    switch (status) {
      case WorkflowStatus.RUNNING:
        return <Play className="h-4 w-4 text-blue-600" />;
      case WorkflowStatus.COMPLETED:
        return <CheckCircle className="h-4 w-4 text-green-600" />;
      case WorkflowStatus.FAILED:
        return <XCircle className="h-4 w-4 text-red-600" />;
      case WorkflowStatus.PENDING:
        return <Clock className="h-4 w-4 text-yellow-600" />;
      case WorkflowStatus.CANCELLED:
        return <Square className="h-4 w-4 text-gray-600" />;
      default:
        return <AlertCircle className="h-4 w-4 text-gray-600" />;
    }
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
          <span className="text-red-800">Failed to load running workflows</span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Workflow Monitor</h1>
          <p className="text-gray-600">
            Real-time monitoring of active workflows
          </p>
        </div>
        <button
          onClick={() => refetch()}
          className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
        >
          <RefreshCw className="h-4 w-4" />
          <span>Refresh</span>
        </button>
      </div>

      {/* Search and Filters */}
      <div className="flex items-center space-x-4">
        <div className="flex-1 relative">
          <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 h-4 w-4 text-gray-400" />
          <input
            type="text"
            placeholder="Search workflows..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
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

      {/* Bulk Actions */}
      {selectedWorkflows.length > 0 && (
        <div className="bg-blue-50 border border-blue-200 rounded-md p-4">
          <div className="flex items-center justify-between">
            <span className="text-blue-800">
              {selectedWorkflows.length} workflow{selectedWorkflows.length === 1 ? '' : 's'} selected
            </span>
            <div className="flex space-x-2">
              <button
                onClick={handleBulkCancel}
                className="px-3 py-1 bg-red-600 text-white rounded text-sm hover:bg-red-700 transition-colors"
              >
                Cancel Selected
              </button>
              <button
                onClick={clearSelection}
                className="px-3 py-1 bg-gray-600 text-white rounded text-sm hover:bg-gray-700 transition-colors"
              >
                Clear Selection
              </button>
            </div>
          </div>
        </div>
      )}

      {/* Workflow List */}
      <div className="bg-white rounded-lg shadow overflow-hidden">
        {filteredWorkflows.length === 0 ? (
          <div className="p-8 text-center">
            <Workflow className="h-12 w-12 text-gray-400 mx-auto mb-4" />
            <h3 className="text-lg font-medium text-gray-900 mb-2">
              No workflows found
            </h3>
            <p className="text-gray-600">
              {workflows.length === 0 
                ? "No workflows are currently running."
                : "No workflows match your search criteria."
              }
            </p>
          </div>
        ) : (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-gray-200">
              <thead className="bg-gray-50">
                <tr>
                  <th className="px-6 py-3 text-left">
                    <input
                      type="checkbox"
                      checked={selectedWorkflows.length === filteredWorkflows.length}
                      onChange={() => 
                        selectedWorkflows.length === filteredWorkflows.length 
                          ? clearSelection() 
                          : selectAllWorkflows()
                      }
                      className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                    />
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Workflow
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Progress
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
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
                {filteredWorkflows.map((workflow) => {
                  const isConnected = connections[workflow.id];
                  const streamError = streamErrors[workflow.id];
                  const timeRemaining = estimateTimeRemaining(workflow);
                  
                  return (
                    <tr key={workflow.id} className="hover:bg-gray-50">
                      <td className="px-6 py-4">
                        <input
                          type="checkbox"
                          checked={selectedWorkflows.includes(workflow.id)}
                          onChange={() => toggleWorkflowSelection(workflow.id)}
                          className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                        />
                      </td>
                      <td className="px-6 py-4">
                        <div>
                          <div className="text-sm font-medium text-gray-900">
                            {getWorkflowTypeDisplayName(workflow.type)}
                          </div>
                          <div className="text-sm text-gray-500">
                            {workflow.id}
                          </div>
                          {!isConnected && (
                            <div className="text-xs text-red-600 mt-1">
                              ⚠️ Connection lost
                            </div>
                          )}
                        </div>
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex items-center space-x-2">
                          {getStatusIcon(workflow.status)}
                          <span className={`
                            inline-flex px-2 py-1 text-xs font-semibold rounded-full
                            workflow-status-${workflow.status}
                          `}>
                            {workflow.status}
                          </span>
                        </div>
                      </td>
                      <td className="px-6 py-4">
                        {workflow.progress && (
                          <div className="space-y-1">
                            <div className="flex items-center justify-between text-sm">
                              <span>{workflow.progress.currentStep}</span>
                              <span>{workflow.progress.percentage}%</span>
                            </div>
                            <div className="workflow-progress-bar">
                              <div 
                                className="workflow-progress-fill"
                                style={{ width: `${workflow.progress.percentage}%` }}
                              />
                            </div>
                            {timeRemaining && (
                              <div className="text-xs text-gray-500">
                                ETA: {formatEstimatedTimeRemaining(timeRemaining)}
                              </div>
                            )}
                          </div>
                        )}
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {workflow.duration 
                          ? formatWorkflowDuration(workflow.duration)
                          : formatRelativeTime(workflow.startedAt)
                        }
                      </td>
                      <td className="px-6 py-4 text-sm text-gray-900">
                        {workflow.metadata.userEmail}
                      </td>
                      <td className="px-6 py-4">
                        <div className="flex items-center space-x-2">
                          <button
                            className="text-blue-600 hover:text-blue-800 transition-colors"
                            title="View Details"
                          >
                            <Eye className="h-4 w-4" />
                          </button>
                          {canCancelWorkflow(workflow) && (
                            <button
                              onClick={() => handleCancelWorkflow(workflow.id)}
                              className="text-red-600 hover:text-red-800 transition-colors"
                              title="Cancel Workflow"
                            >
                              <Square className="h-4 w-4" />
                            </button>
                          )}
                          {canRetryWorkflow(workflow) && (
                            <button
                              onClick={() => handleRetryWorkflow(workflow.id)}
                              className="text-green-600 hover:text-green-800 transition-colors"
                              title="Retry Workflow"
                            >
                              <RefreshCw className="h-4 w-4" />
                            </button>
                          )}
                        </div>
                      </td>
                    </tr>
                  );
                })}
              </tbody>
            </table>
          </div>
        )}
      </div>
    </div>
  );
};

export default WorkflowMonitor;