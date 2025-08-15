import React, { useState } from 'react';
import { Settings, Power, PowerOff, Trash2, ExternalLink, AlertTriangle } from 'lucide-react';
import { useModules } from '../hooks/useModules';
import { InstallationStatus } from '../types/module';
import { 
  getInstallationStatusColor,
  getInstallationStatusDisplayName,
  formatPrice,
  getCategoryDisplayName
} from '../utils/moduleUtils';

export const ModuleManager: React.FC = () => {
  const [selectedModuleId, setSelectedModuleId] = useState<string | null>(null);
  
  const { 
    installedModules, 
    activateModule, 
    deactivateModule, 
    uninstallModule 
  } = useModules();

  const handleActivateModule = async (moduleId: string) => {
    try {
      await activateModule.mutateAsync(moduleId);
    } catch (error) {
      console.error('Failed to activate module:', error);
    }
  };

  const handleDeactivateModule = async (moduleId: string) => {
    try {
      await deactivateModule.mutateAsync(moduleId);
    } catch (error) {
      console.error('Failed to deactivate module:', error);
    }
  };

  const handleUninstallModule = async (moduleId: string) => {
    if (window.confirm('Are you sure you want to uninstall this module? This action cannot be undone.')) {
      try {
        await uninstallModule.mutateAsync(moduleId);
      } catch (error) {
        console.error('Failed to uninstall module:', error);
      }
    }
  };

  const getActionButton = (module: any) => {
    const isLoading = activateModule.isPending || deactivateModule.isPending || uninstallModule.isPending;
    
    switch (module.installationStatus) {
      case InstallationStatus.Installed:
        return (
          <button
            onClick={() => handleActivateModule(module.id)}
            disabled={isLoading}
            className="flex items-center gap-2 px-3 py-1 bg-green-600 text-white rounded hover:bg-green-700 disabled:opacity-50"
          >
            <Power className="w-4 h-4" />
            Activate
          </button>
        );
      
      case InstallationStatus.Active:
        return (
          <button
            onClick={() => handleDeactivateModule(module.id)}
            disabled={isLoading}
            className="flex items-center gap-2 px-3 py-1 bg-yellow-600 text-white rounded hover:bg-yellow-700 disabled:opacity-50"
          >
            <PowerOff className="w-4 h-4" />
            Deactivate
          </button>
        );
      
      default:
        return null;
    }
  };

  if (installedModules.isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (installedModules.error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 text-red-600" />
          <p className="text-red-800">
            Failed to load installed modules: {(installedModules.error as Error).message}
          </p>
        </div>
      </div>
    );
  }

  const modules = installedModules.data || [];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-2xl font-bold text-gray-900">Installed Modules</h1>
          <p className="text-gray-600">Manage your installed modules and their configurations</p>
        </div>
      </div>

      {modules.length === 0 ? (
        <div className="text-center py-12">
          <p className="text-gray-500 mb-4">No modules installed yet.</p>
          <button
            onClick={() => window.location.hash = '#/marketplace'}
            className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700"
          >
            Browse Marketplace
          </button>
        </div>
      ) : (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200">
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-50 border-b border-gray-200">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Module
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Category
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Version
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Status
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="bg-white divide-y divide-gray-200">
                {modules.map((module) => (
                  <tr key={module.id} className="hover:bg-gray-50">
                    <td className="px-6 py-4 whitespace-nowrap">
                      <div className="flex items-center">
                        <div className="flex-shrink-0 h-10 w-10">
                          <div className="h-10 w-10 rounded-lg bg-gradient-to-br from-blue-50 to-indigo-100 flex items-center justify-center">
                            <span className="text-lg font-bold text-blue-600">
                              {module.name.charAt(0).toUpperCase()}
                            </span>
                          </div>
                        </div>
                        <div className="ml-4">
                          <div className="text-sm font-medium text-gray-900">
                            {module.name}
                          </div>
                          <div className="text-sm text-gray-500">
                            by {module.author.name}
                          </div>
                        </div>
                      </div>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
                        {getCategoryDisplayName(module.category)}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-900">
                      {module.version}
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap">
                      <span className={`inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium ${
                        module.installationStatus === InstallationStatus.Active
                          ? 'bg-green-100 text-green-800'
                          : module.installationStatus === InstallationStatus.Failed
                          ? 'bg-red-100 text-red-800'
                          : 'bg-yellow-100 text-yellow-800'
                      }`}>
                        {getInstallationStatusDisplayName(module.installationStatus || InstallationStatus.Installed)}
                      </span>
                    </td>
                    <td className="px-6 py-4 whitespace-nowrap text-sm font-medium">
                      <div className="flex items-center gap-2">
                        {getActionButton(module)}
                        
                        <button
                          onClick={() => setSelectedModuleId(
                            selectedModuleId === module.id ? null : module.id
                          )}
                          className="flex items-center gap-1 px-3 py-1 border border-gray-300 rounded hover:bg-gray-50"
                        >
                          <Settings className="w-4 h-4" />
                          Settings
                        </button>
                        
                        <button
                          onClick={() => window.open(module.documentationUrl, '_blank')}
                          className="flex items-center gap-1 px-3 py-1 border border-gray-300 rounded hover:bg-gray-50"
                        >
                          <ExternalLink className="w-4 h-4" />
                          Docs
                        </button>
                        
                        <button
                          onClick={() => handleUninstallModule(module.id)}
                          disabled={uninstallModule.isPending}
                          className="flex items-center gap-1 px-3 py-1 border border-red-300 text-red-600 rounded hover:bg-red-50 disabled:opacity-50"
                        >
                          <Trash2 className="w-4 h-4" />
                          Uninstall
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        </div>
      )}

      {/* Module Details Panel */}
      {selectedModuleId && (
        <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
          {(() => {
            const selectedModule = modules.find(m => m.id === selectedModuleId);
            if (!selectedModule) return null;

            return (
              <div>
                <div className="flex items-center justify-between mb-4">
                  <h3 className="text-lg font-medium text-gray-900">
                    {selectedModule.name} Settings
                  </h3>
                  <button
                    onClick={() => setSelectedModuleId(null)}
                    className="text-gray-400 hover:text-gray-600"
                  >
                    ×
                  </button>
                </div>
                
                <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                  <div>
                    <h4 className="font-medium text-gray-900 mb-2">Module Information</h4>
                    <dl className="space-y-2 text-sm">
                      <div>
                        <dt className="text-gray-500">Description:</dt>
                        <dd className="text-gray-900">{selectedModule.description}</dd>
                      </div>
                      <div>
                        <dt className="text-gray-500">Version:</dt>
                        <dd className="text-gray-900">{selectedModule.version}</dd>
                      </div>
                      <div>
                        <dt className="text-gray-500">Last Updated:</dt>
                        <dd className="text-gray-900">
                          {new Date(selectedModule.lastUpdated).toLocaleDateString()}
                        </dd>
                      </div>
                      <div>
                        <dt className="text-gray-500">Downloads:</dt>
                        <dd className="text-gray-900">{selectedModule.downloads.toLocaleString()}</dd>
                      </div>
                    </dl>
                  </div>
                  
                  <div>
                    <h4 className="font-medium text-gray-900 mb-2">Configuration</h4>
                    <p className="text-sm text-gray-500 mb-4">
                      Module-specific settings will be displayed here when available.
                    </p>
                    <button className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700">
                      Configure Module
                    </button>
                  </div>
                </div>
              </div>
            );
          })()}
        </div>
      )}
    </div>
  );
};