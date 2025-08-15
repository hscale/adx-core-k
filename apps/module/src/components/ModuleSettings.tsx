import React, { useState } from 'react';
import { Save, RefreshCw, AlertTriangle, CheckCircle, Settings, Shield, Cpu, HardDrive } from 'lucide-react';
import { useModuleConfiguration } from '../hooks/useModules';
import { ModuleConfiguration, ResourceLimits } from '../types/module';

interface ModuleSettingsProps {
  moduleId: string;
}

export const ModuleSettings: React.FC<ModuleSettingsProps> = ({ moduleId }) => {
  const { configuration, updateConfiguration } = useModuleConfiguration(moduleId);
  const [localConfig, setLocalConfig] = useState<Partial<ModuleConfiguration>>({});
  const [hasChanges, setHasChanges] = useState(false);

  React.useEffect(() => {
    if (configuration.data) {
      setLocalConfig(configuration.data);
      setHasChanges(false);
    }
  }, [configuration.data]);

  const handleSettingChange = (key: string, value: any) => {
    setLocalConfig(prev => ({
      ...prev,
      settings: {
        ...prev.settings,
        [key]: value,
      },
    }));
    setHasChanges(true);
  };

  const handleResourceChange = (resource: keyof ResourceLimits, value: string) => {
    setLocalConfig(prev => ({
      ...prev,
      resources: {
        ...prev.resources,
        [resource]: value,
      },
    }));
    setHasChanges(true);
  };

  const handlePermissionToggle = (permission: string) => {
    setLocalConfig(prev => {
      const currentPermissions = prev.permissions || [];
      const newPermissions = currentPermissions.includes(permission)
        ? currentPermissions.filter(p => p !== permission)
        : [...currentPermissions, permission];
      
      return {
        ...prev,
        permissions: newPermissions,
      };
    });
    setHasChanges(true);
  };

  const handleSave = async () => {
    try {
      await updateConfiguration.mutateAsync(localConfig);
      setHasChanges(false);
    } catch (error) {
      console.error('Failed to update configuration:', error);
    }
  };

  const handleReset = () => {
    if (configuration.data) {
      setLocalConfig(configuration.data);
      setHasChanges(false);
    }
  };

  if (configuration.isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
      </div>
    );
  }

  if (configuration.error) {
    return (
      <div className="bg-red-50 border border-red-200 rounded-lg p-4">
        <div className="flex items-center gap-2">
          <AlertTriangle className="w-5 h-5 text-red-600" />
          <p className="text-red-800">
            Failed to load module configuration: {(configuration.error as Error).message}
          </p>
        </div>
      </div>
    );
  }

  const config = localConfig;
  const availablePermissions = [
    'database:read',
    'database:write',
    'api:external',
    'files:read',
    'files:write',
    'tenant:read',
    'user:read',
    'workflow:execute',
  ];

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <Settings className="w-6 h-6 text-gray-600" />
          <div>
            <h1 className="text-2xl font-bold text-gray-900">Module Settings</h1>
            <p className="text-gray-600">Configure module behavior and permissions</p>
          </div>
        </div>
        
        {hasChanges && (
          <div className="flex items-center gap-2">
            <button
              onClick={handleReset}
              className="px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
            >
              <RefreshCw className="w-4 h-4 mr-2" />
              Reset
            </button>
            <button
              onClick={handleSave}
              disabled={updateConfiguration.isPending}
              className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
            >
              <Save className="w-4 h-4 mr-2" />
              {updateConfiguration.isPending ? 'Saving...' : 'Save Changes'}
            </button>
          </div>
        )}
      </div>

      {/* Status */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <div className={`w-3 h-3 rounded-full ${config.enabled ? 'bg-green-500' : 'bg-gray-400'}`} />
            <span className="font-medium text-gray-900">
              Module Status: {config.enabled ? 'Enabled' : 'Disabled'}
            </span>
          </div>
          <button
            onClick={() => {
              setLocalConfig(prev => ({ ...prev, enabled: !prev.enabled }));
              setHasChanges(true);
            }}
            className={`px-4 py-2 rounded-lg ${
              config.enabled
                ? 'bg-red-100 text-red-700 hover:bg-red-200'
                : 'bg-green-100 text-green-700 hover:bg-green-200'
            }`}
          >
            {config.enabled ? 'Disable Module' : 'Enable Module'}
          </button>
        </div>
      </div>

      {/* General Settings */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <h2 className="text-lg font-medium text-gray-900 mb-4">General Settings</h2>
        
        <div className="space-y-4">
          {config.settings && Object.entries(config.settings).map(([key, value]) => (
            <div key={key}>
              <label className="block text-sm font-medium text-gray-700 mb-1">
                {key.replace(/([A-Z])/g, ' $1').replace(/^./, str => str.toUpperCase())}
              </label>
              {typeof value === 'boolean' ? (
                <label className="flex items-center">
                  <input
                    type="checkbox"
                    checked={value}
                    onChange={(e) => handleSettingChange(key, e.target.checked)}
                    className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                  />
                  <span className="ml-2 text-sm text-gray-600">
                    {value ? 'Enabled' : 'Disabled'}
                  </span>
                </label>
              ) : typeof value === 'number' ? (
                <input
                  type="number"
                  value={value}
                  onChange={(e) => handleSettingChange(key, Number(e.target.value))}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              ) : (
                <input
                  type="text"
                  value={String(value)}
                  onChange={(e) => handleSettingChange(key, e.target.value)}
                  className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              )}
            </div>
          ))}
          
          {(!config.settings || Object.keys(config.settings).length === 0) && (
            <p className="text-gray-500 text-sm">No configurable settings available for this module.</p>
          )}
        </div>
      </div>

      {/* Resource Limits */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="flex items-center gap-2 mb-4">
          <Cpu className="w-5 h-5 text-gray-600" />
          <h2 className="text-lg font-medium text-gray-900">Resource Limits</h2>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Memory Limit
            </label>
            <input
              type="text"
              value={config.resources?.memory || ''}
              onChange={(e) => handleResourceChange('memory', e.target.value)}
              placeholder="e.g., 512MB"
              className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              CPU Limit
            </label>
            <input
              type="text"
              value={config.resources?.cpu || ''}
              onChange={(e) => handleResourceChange('cpu', e.target.value)}
              placeholder="e.g., 0.5"
              className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Storage Limit
            </label>
            <input
              type="text"
              value={config.resources?.storage || ''}
              onChange={(e) => handleResourceChange('storage', e.target.value)}
              placeholder="e.g., 1GB"
              className="w-full border border-gray-300 rounded-lg px-3 py-2 focus:ring-2 focus:ring-blue-500 focus:border-transparent"
            />
          </div>
          
          <div>
            <label className="block text-sm font-medium text-gray-700 mb-1">
              Network Access
            </label>
            <label className="flex items-center">
              <input
                type="checkbox"
                checked={config.resources?.networkAccess || false}
                onChange={(e) => handleResourceChange('networkAccess', e.target.checked)}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span className="ml-2 text-sm text-gray-600">
                {config.resources?.networkAccess ? 'Allowed' : 'Blocked'}
              </span>
            </label>
          </div>
        </div>
      </div>

      {/* Permissions */}
      <div className="bg-white rounded-lg shadow-sm border border-gray-200 p-6">
        <div className="flex items-center gap-2 mb-4">
          <Shield className="w-5 h-5 text-gray-600" />
          <h2 className="text-lg font-medium text-gray-900">Permissions</h2>
        </div>
        
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          {availablePermissions.map((permission) => (
            <label key={permission} className="flex items-center">
              <input
                type="checkbox"
                checked={config.permissions?.includes(permission) || false}
                onChange={() => handlePermissionToggle(permission)}
                className="rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
              <span className="ml-3 text-sm text-gray-700">
                {permission.replace(/:/g, ' - ').replace(/([A-Z])/g, ' $1').toLowerCase()}
              </span>
            </label>
          ))}
        </div>
      </div>

      {/* Save Status */}
      {updateConfiguration.isSuccess && (
        <div className="bg-green-50 border border-green-200 rounded-lg p-4">
          <div className="flex items-center gap-2">
            <CheckCircle className="w-5 h-5 text-green-600" />
            <p className="text-green-800">Configuration saved successfully!</p>
          </div>
        </div>
      )}

      {updateConfiguration.error && (
        <div className="bg-red-50 border border-red-200 rounded-lg p-4">
          <div className="flex items-center gap-2">
            <AlertTriangle className="w-5 h-5 text-red-600" />
            <p className="text-red-800">
              Failed to save configuration: {(updateConfiguration.error as Error).message}
            </p>
          </div>
        </div>
      )}
    </div>
  );
};