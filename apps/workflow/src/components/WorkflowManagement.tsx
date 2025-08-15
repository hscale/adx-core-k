import React, { useState } from 'react';
import { 
  Play, 
  Settings, 
  Plus, 
  Edit, 
  Trash2, 
  Copy,
  AlertTriangle,
  CheckCircle,
  Clock,
  Zap
} from 'lucide-react';
import { 
  useWorkflowTemplates, 
  useStartWorkflow, 
  useBulkCancelWorkflows 
} from '../hooks';
import { 
  WorkflowTemplate, 
  WorkflowComplexity,
  WorkflowParameter 
} from '../types';

const WorkflowManagement: React.FC = () => {
  const [selectedTemplate, setSelectedTemplate] = useState<WorkflowTemplate | null>(null);
  const [showStartDialog, setShowStartDialog] = useState(false);
  const [parameters, setParameters] = useState<Record<string, any>>({});
  const [showTemplateForm, setShowTemplateForm] = useState(false);

  const { data: templates = [], isLoading, error, refetch } = useWorkflowTemplates();
  const startWorkflowMutation = useStartWorkflow();
  const bulkCancelMutation = useBulkCancelWorkflows();

  const handleStartWorkflow = async () => {
    if (!selectedTemplate) return;

    try {
      await startWorkflowMutation.mutateAsync({
        templateId: selectedTemplate.id,
        parameters,
      });
      setShowStartDialog(false);
      setParameters({});
      setSelectedTemplate(null);
    } catch (error) {
      console.error('Failed to start workflow:', error);
    }
  };

  const handleParameterChange = (paramName: string, value: any) => {
    setParameters(prev => ({
      ...prev,
      [paramName]: value,
    }));
  };

  const validateParameters = (): boolean => {
    if (!selectedTemplate) return false;

    return selectedTemplate.parameters.every(param => {
      if (!param.required) return true;
      const value = parameters[param.name];
      return value !== undefined && value !== null && value !== '';
    });
  };

  const getComplexityColor = (complexity: WorkflowComplexity): string => {
    switch (complexity) {
      case WorkflowComplexity.SIMPLE:
        return 'text-green-600 bg-green-100';
      case WorkflowComplexity.MODERATE:
        return 'text-yellow-600 bg-yellow-100';
      case WorkflowComplexity.COMPLEX:
        return 'text-red-600 bg-red-100';
      default:
        return 'text-gray-600 bg-gray-100';
    }
  };

  const getComplexityIcon = (complexity: WorkflowComplexity) => {
    switch (complexity) {
      case WorkflowComplexity.SIMPLE:
        return <CheckCircle className="h-4 w-4" />;
      case WorkflowComplexity.MODERATE:
        return <Clock className="h-4 w-4" />;
      case WorkflowComplexity.COMPLEX:
        return <Zap className="h-4 w-4" />;
      default:
        return <AlertTriangle className="h-4 w-4" />;
    }
  };

  const renderParameterInput = (param: WorkflowParameter) => {
    const value = parameters[param.name] || param.defaultValue || '';

    switch (param.type) {
      case 'string':
        if (param.validation?.options) {
          return (
            <select
              value={value}
              onChange={(e) => handleParameterChange(param.name, e.target.value)}
              className="w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            >
              <option value="">Select an option</option>
              {param.validation.options.map(option => (
                <option key={option} value={option}>
                  {option}
                </option>
              ))}
            </select>
          );
        }
        return (
          <input
            type="text"
            value={value}
            onChange={(e) => handleParameterChange(param.name, e.target.value)}
            placeholder={param.description}
            className="w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        );

      case 'number':
        return (
          <input
            type="number"
            value={value}
            onChange={(e) => handleParameterChange(param.name, parseFloat(e.target.value))}
            min={param.validation?.min}
            max={param.validation?.max}
            placeholder={param.description}
            className="w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        );

      case 'boolean':
        return (
          <div className="flex items-center space-x-2">
            <input
              type="checkbox"
              checked={value}
              onChange={(e) => handleParameterChange(param.name, e.target.checked)}
              className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
            />
            <span className="text-sm text-gray-600">{param.description}</span>
          </div>
        );

      case 'date':
        return (
          <input
            type="date"
            value={value}
            onChange={(e) => handleParameterChange(param.name, e.target.value)}
            className="w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        );

      default:
        return (
          <textarea
            value={value}
            onChange={(e) => handleParameterChange(param.name, e.target.value)}
            placeholder={param.description}
            rows={3}
            className="w-full border border-gray-300 rounded-md px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
          />
        );
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
          <AlertTriangle className="h-5 w-5 text-red-600 mr-2" />
          <span className="text-red-800">Failed to load workflow templates</span>
        </div>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Workflow Management</h1>
          <p className="text-gray-600">
            Manage workflow templates and start new workflow executions
          </p>
        </div>
        <div className="flex space-x-2">
          <button
            onClick={() => setShowTemplateForm(true)}
            className="flex items-center space-x-2 px-4 py-2 border border-gray-300 rounded-md hover:bg-gray-50 transition-colors"
          >
            <Plus className="h-4 w-4" />
            <span>New Template</span>
          </button>
          <button
            onClick={() => refetch()}
            className="flex items-center space-x-2 px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 transition-colors"
          >
            <Settings className="h-4 w-4" />
            <span>Manage</span>
          </button>
        </div>
      </div>

      {/* Template Categories */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        {/* Business Workflows */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Business Workflows</h3>
          <div className="space-y-3">
            {templates
              .filter(t => t.category === 'business')
              .map(template => (
                <div
                  key={template.id}
                  className="p-3 border border-gray-200 rounded-md hover:bg-gray-50 cursor-pointer transition-colors"
                  onClick={() => {
                    setSelectedTemplate(template);
                    setShowStartDialog(true);
                  }}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="text-sm font-medium text-gray-900">
                        {template.name}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">
                        {template.description}
                      </div>
                    </div>
                    <div className={`
                      flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium
                      ${getComplexityColor(template.complexity)}
                    `}>
                      {getComplexityIcon(template.complexity)}
                      <span>{template.complexity}</span>
                    </div>
                  </div>
                </div>
              ))}
          </div>
        </div>

        {/* System Workflows */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">System Workflows</h3>
          <div className="space-y-3">
            {templates
              .filter(t => t.category === 'system')
              .map(template => (
                <div
                  key={template.id}
                  className="p-3 border border-gray-200 rounded-md hover:bg-gray-50 cursor-pointer transition-colors"
                  onClick={() => {
                    setSelectedTemplate(template);
                    setShowStartDialog(true);
                  }}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="text-sm font-medium text-gray-900">
                        {template.name}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">
                        {template.description}
                      </div>
                    </div>
                    <div className={`
                      flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium
                      ${getComplexityColor(template.complexity)}
                    `}>
                      {getComplexityIcon(template.complexity)}
                      <span>{template.complexity}</span>
                    </div>
                  </div>
                </div>
              ))}
          </div>
        </div>

        {/* Integration Workflows */}
        <div className="bg-white p-6 rounded-lg shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900 mb-4">Integration Workflows</h3>
          <div className="space-y-3">
            {templates
              .filter(t => t.category === 'integration')
              .map(template => (
                <div
                  key={template.id}
                  className="p-3 border border-gray-200 rounded-md hover:bg-gray-50 cursor-pointer transition-colors"
                  onClick={() => {
                    setSelectedTemplate(template);
                    setShowStartDialog(true);
                  }}
                >
                  <div className="flex items-center justify-between">
                    <div className="flex-1">
                      <div className="text-sm font-medium text-gray-900">
                        {template.name}
                      </div>
                      <div className="text-xs text-gray-500 mt-1">
                        {template.description}
                      </div>
                    </div>
                    <div className={`
                      flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium
                      ${getComplexityColor(template.complexity)}
                    `}>
                      {getComplexityIcon(template.complexity)}
                      <span>{template.complexity}</span>
                    </div>
                  </div>
                </div>
              ))}
          </div>
        </div>
      </div>

      {/* All Templates Table */}
      <div className="bg-white rounded-lg shadow overflow-hidden">
        <div className="px-6 py-4 border-b border-gray-200">
          <h3 className="text-lg font-semibold text-gray-900">All Workflow Templates</h3>
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-gray-200">
            <thead className="bg-gray-50">
              <tr>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Template
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Category
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Complexity
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Est. Duration
                </th>
                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                  Actions
                </th>
              </tr>
            </thead>
            <tbody className="bg-white divide-y divide-gray-200">
              {templates.map((template) => (
                <tr key={template.id} className="hover:bg-gray-50">
                  <td className="px-6 py-4">
                    <div>
                      <div className="text-sm font-medium text-gray-900">
                        {template.name}
                      </div>
                      <div className="text-sm text-gray-500">
                        {template.description}
                      </div>
                    </div>
                  </td>
                  <td className="px-6 py-4">
                    <span className="inline-flex px-2 py-1 text-xs font-semibold rounded-full bg-blue-100 text-blue-800">
                      {template.category}
                    </span>
                  </td>
                  <td className="px-6 py-4">
                    <div className={`
                      inline-flex items-center space-x-1 px-2 py-1 rounded-full text-xs font-medium
                      ${getComplexityColor(template.complexity)}
                    `}>
                      {getComplexityIcon(template.complexity)}
                      <span>{template.complexity}</span>
                    </div>
                  </td>
                  <td className="px-6 py-4 text-sm text-gray-900">
                    ~{Math.round(template.estimatedDuration / 60)} min
                  </td>
                  <td className="px-6 py-4">
                    <div className="flex items-center space-x-2">
                      <button
                        onClick={() => {
                          setSelectedTemplate(template);
                          setShowStartDialog(true);
                        }}
                        className="text-blue-600 hover:text-blue-800 transition-colors"
                        title="Start Workflow"
                      >
                        <Play className="h-4 w-4" />
                      </button>
                      <button
                        className="text-gray-600 hover:text-gray-800 transition-colors"
                        title="Edit Template"
                      >
                        <Edit className="h-4 w-4" />
                      </button>
                      <button
                        className="text-gray-600 hover:text-gray-800 transition-colors"
                        title="Clone Template"
                      >
                        <Copy className="h-4 w-4" />
                      </button>
                      <button
                        className="text-red-600 hover:text-red-800 transition-colors"
                        title="Delete Template"
                      >
                        <Trash2 className="h-4 w-4" />
                      </button>
                    </div>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Start Workflow Dialog */}
      {showStartDialog && selectedTemplate && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg shadow-xl max-w-2xl w-full mx-4 max-h-[90vh] overflow-y-auto">
            <div className="px-6 py-4 border-b border-gray-200">
              <h3 className="text-lg font-semibold text-gray-900">
                Start Workflow: {selectedTemplate.name}
              </h3>
              <p className="text-sm text-gray-600 mt-1">
                {selectedTemplate.description}
              </p>
            </div>

            <div className="px-6 py-4 space-y-4">
              {selectedTemplate.parameters.map((param) => (
                <div key={param.name}>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    {param.name}
                    {param.required && <span className="text-red-500 ml-1">*</span>}
                  </label>
                  {renderParameterInput(param)}
                  {param.description && (
                    <p className="text-xs text-gray-500 mt-1">{param.description}</p>
                  )}
                </div>
              ))}
            </div>

            <div className="px-6 py-4 border-t border-gray-200 flex justify-end space-x-3">
              <button
                onClick={() => {
                  setShowStartDialog(false);
                  setParameters({});
                  setSelectedTemplate(null);
                }}
                className="px-4 py-2 border border-gray-300 rounded-md text-gray-700 hover:bg-gray-50 transition-colors"
              >
                Cancel
              </button>
              <button
                onClick={handleStartWorkflow}
                disabled={!validateParameters() || startWorkflowMutation.isLoading}
                className="px-4 py-2 bg-blue-600 text-white rounded-md hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
              >
                {startWorkflowMutation.isLoading ? 'Starting...' : 'Start Workflow'}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};

export default WorkflowManagement;