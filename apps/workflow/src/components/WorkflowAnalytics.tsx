import React, { useState } from 'react';
import { 
  BarChart, 
  Bar, 
  XAxis, 
  YAxis, 
  CartesianGrid, 
  Tooltip, 
  ResponsiveContainer,
  LineChart,
  Line,
  PieChart,
  Pie,
  Cell,
  Area,
  AreaChart
} from 'recharts';
import { 
  TrendingUp, 
  TrendingDown, 
  Activity, 
  Clock, 
  AlertTriangle,
  CheckCircle,
  RefreshCw
} from 'lucide-react';
import { useWorkflowAnalytics } from '../hooks';
import { formatWorkflowDuration } from '../utils';

const COLORS = ['#3B82F6', '#10B981', '#F59E0B', '#EF4444', '#8B5CF6', '#06B6D4'];

const WorkflowAnalytics: React.FC = () => {
  const [timeRange, setTimeRange] = useState('7d');
  const { data: analytics, isLoading, error, refetch } = useWorkflowAnalytics(timeRange);

  const timeRangeOptions = [
    { value: '1d', label: 'Last 24 Hours' },
    { value: '7d', label: 'Last 7 Days' },
    { value: '30d', label: 'Last 30 Days' },
    { value: '90d', label: 'Last 90 Days' },
  ];

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (error || !analytics) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-md p-4">
        <div className="flex items-center">
          <AlertTriangle className="h-5 w-5 text-red-600 mr-2" />
          <span className="text-red-800">Failed to load workflow analytics</span>
        </div>
      </div>
    );
  }

  const { metrics, trends, topWorkflowTypes, errorAnalysis, performanceMetrics } = analytics;

  // Prepare chart data
  const trendData = trends.map(trend => ({
    timestamp: new Date(trend.timestamp).toLocaleDateString(),
    total: trend.totalExecutions,
    successful: trend.successfulExecutions,
    failed: trend.failedExecutions,
    avgDuration: trend.averageDuration,
  }));

  const workflowTypeData = topWorkflowTypes.map(type => ({
    name: type.type.replace(/_/g, ' ').replace(/\b\w/g, l => l.toUpperCase()),
    count: type.count,
    successRate: type.successRate,
    avgDuration: type.averageDuration,
  }));

  const statusDistribution = [
    { name: 'Completed', value: metrics.completedWorkflows, color: '#10B981' },
    { name: 'Failed', value: metrics.failedWorkflows, color: '#EF4444' },
    { name: 'Running', value: metrics.runningWorkflows, color: '#3B82F6' },
  ];

  const errorTrendData = errorAnalysis.errorTrends.map(trend => ({
    timestamp: new Date(trend.timestamp).toLocaleDateString(),
    errorCount: trend.errorCount,
    errorRate: trend.errorRate,
  }));

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Workflow Analytics</h1>
          <p className="text-gray-600">
            Insights and performance metrics for your workflows
          </p>
        </div>
        <div className="flex items-center space-x-4">
          <select
            value={timeRange}
            onChange={(e) => setTimeRange(e.target.value)}
            className="border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          >
            {timeRangeOptions.map(option => (
              <option key={option.value} value={option.value}>
                {option.label}
              </option>
            ))}
          </select>
          <button
            onClick={() => refetch()}
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            <RefreshCw className="h-4 w-4" />
            <span>Refresh</span>
          </button>
        </div>
      </div>

      {/* Key Metrics */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
        <div className="workflow-metric-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Total Workflows</p>
              <p className="text-2xl font-bold text-gray-900">{metrics.totalWorkflows}</p>
            </div>
            <Activity className="h-8 w-8 text-blue-600" />
          </div>
          <div className="mt-2 flex items-center text-sm">
            <TrendingUp className="h-4 w-4 text-green-600 mr-1" />
            <span className="text-green-600">+12% from last period</span>
          </div>
        </div>

        <div className="workflow-metric-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Success Rate</p>
              <p className="text-2xl font-bold text-gray-900">{metrics.successRate}%</p>
            </div>
            <CheckCircle className="h-8 w-8 text-green-600" />
          </div>
          <div className="mt-2 flex items-center text-sm">
            <TrendingUp className="h-4 w-4 text-green-600 mr-1" />
            <span className="text-green-600">+2.3% from last period</span>
          </div>
        </div>

        <div className="workflow-metric-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Avg Duration</p>
              <p className="text-2xl font-bold text-gray-900">
                {formatWorkflowDuration(metrics.averageDuration)}
              </p>
            </div>
            <Clock className="h-8 w-8 text-purple-600" />
          </div>
          <div className="mt-2 flex items-center text-sm">
            <TrendingDown className="h-4 w-4 text-green-600 mr-1" />
            <span className="text-green-600">-15% faster</span>
          </div>
        </div>

        <div className="workflow-metric-card">
          <div className="flex items-center justify-between">
            <div>
              <p className="text-sm font-medium text-gray-600">Error Rate</p>
              <p className="text-2xl font-bold text-gray-900">{metrics.errorRate}%</p>
            </div>
            <AlertTriangle className="h-8 w-8 text-red-600" />
          </div>
          <div className="mt-2 flex items-center text-sm">
            <TrendingDown className="h-4 w-4 text-green-600 mr-1" />
            <span className="text-green-600">-1.2% from last period</span>
          </div>
        </div>
      </div>

      {/* Charts Grid */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Workflow Trends */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Workflow Execution Trends</h3>
          <ResponsiveContainer width="100%" height={300}>
            <AreaChart data={trendData}>
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis dataKey="timestamp" />
              <YAxis />
              <Tooltip />
              <Area 
                type="monotone" 
                dataKey="successful" 
                stackId="1" 
                stroke="#10B981" 
                fill="#10B981" 
                fillOpacity={0.6}
                name="Successful"
              />
              <Area 
                type="monotone" 
                dataKey="failed" 
                stackId="1" 
                stroke="#EF4444" 
                fill="#EF4444" 
                fillOpacity={0.6}
                name="Failed"
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* Status Distribution */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Workflow Status Distribution</h3>
          <ResponsiveContainer width="100%" height={300}>
            <PieChart>
              <Pie
                data={statusDistribution}
                cx="50%"
                cy="50%"
                labelLine={false}
                label={({ name, percent }) => `${name} ${(percent * 100).toFixed(0)}%`}
                outerRadius={80}
                fill="#8884d8"
                dataKey="value"
              >
                {statusDistribution.map((entry, index) => (
                  <Cell key={`cell-${index}`} fill={entry.color} />
                ))}
              </Pie>
              <Tooltip />
            </PieChart>
          </ResponsiveContainer>
        </div>

        {/* Top Workflow Types */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Top Workflow Types</h3>
          <ResponsiveContainer width="100%" height={300}>
            <BarChart data={workflowTypeData} layout="horizontal">
              <CartesianGrid strokeDasharray="3 3" />
              <XAxis type="number" />
              <YAxis dataKey="name" type="category" width={100} />
              <Tooltip />
              <Bar dataKey="count" fill="#3B82F6" />
            </BarChart>
          </ResponsiveContainer>
        </div>

        {/* Performance Metrics */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Performance Distribution</h3>
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-600">P50 Duration</span>
              <span className="text-sm font-bold text-gray-900">
                {formatWorkflowDuration(performanceMetrics.p50Duration)}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-600">P95 Duration</span>
              <span className="text-sm font-bold text-gray-900">
                {formatWorkflowDuration(performanceMetrics.p95Duration)}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-600">P99 Duration</span>
              <span className="text-sm font-bold text-gray-900">
                {formatWorkflowDuration(performanceMetrics.p99Duration)}
              </span>
            </div>
            <div className="flex items-center justify-between">
              <span className="text-sm font-medium text-gray-600">Throughput/Hour</span>
              <span className="text-sm font-bold text-gray-900">
                {performanceMetrics.throughputPerHour}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Error Analysis */}
      <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
        <h3 className="text-lg font-semibold text-gray-900 mb-4">Error Analysis</h3>
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
          {/* Error Trends */}
          <div>
            <h4 className="text-md font-medium text-gray-700 mb-3">Error Rate Trends</h4>
            <ResponsiveContainer width="100%" height={200}>
              <LineChart data={errorTrendData}>
                <CartesianGrid strokeDasharray="3 3" />
                <XAxis dataKey="timestamp" />
                <YAxis />
                <Tooltip />
                <Line 
                  type="monotone" 
                  dataKey="errorRate" 
                  stroke="#EF4444" 
                  strokeWidth={2}
                  name="Error Rate %"
                />
              </LineChart>
            </ResponsiveContainer>
          </div>

          {/* Top Errors */}
          <div>
            <h4 className="text-md font-medium text-gray-700 mb-3">Top Error Types</h4>
            <div className="space-y-3">
              {errorAnalysis.topErrors.slice(0, 5).map((error, index) => (
                <div key={index} className="flex items-center justify-between p-3 bg-red-50 rounded-md">
                  <div className="flex-1">
                    <div className="text-sm font-medium text-red-900">
                      {error.errorCode}
                    </div>
                    <div className="text-xs text-red-700 truncate">
                      {error.errorMessage}
                    </div>
                  </div>
                  <div className="text-right">
                    <div className="text-sm font-bold text-red-900">
                      {error.count}
                    </div>
                    <div className="text-xs text-red-700">
                      {error.percentage}%
                    </div>
                  </div>
                </div>
              ))}
            </div>
          </div>
        </div>

        {/* Retry Analysis */}
        <div className="mt-6 grid grid-cols-1 md:grid-cols-4 gap-4">
          <div className="bg-gray-50 p-4 rounded-md">
            <div className="text-lg font-bold text-gray-900">
              {errorAnalysis.retryAnalysis.totalRetries}
            </div>
            <div className="text-sm text-gray-600">Total Retries</div>
          </div>
          <div className="bg-gray-50 p-4 rounded-md">
            <div className="text-lg font-bold text-gray-900">
              {errorAnalysis.retryAnalysis.averageRetriesPerWorkflow.toFixed(1)}
            </div>
            <div className="text-sm text-gray-600">Avg Retries/Workflow</div>
          </div>
          <div className="bg-gray-50 p-4 rounded-md">
            <div className="text-lg font-bold text-gray-900">
              {errorAnalysis.retryAnalysis.retrySuccessRate}%
            </div>
            <div className="text-sm text-gray-600">Retry Success Rate</div>
          </div>
          <div className="bg-gray-50 p-4 rounded-md">
            <div className="text-lg font-bold text-gray-900">
              {errorAnalysis.retryAnalysis.maxRetriesReached}
            </div>
            <div className="text-sm text-gray-600">Max Retries Reached</div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default WorkflowAnalytics;