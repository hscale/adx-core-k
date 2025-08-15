import { Module, ModuleCategory, PricingModel, InstallationStatus, Platform } from '../types/module';
import * as semver from 'semver';

export const formatPrice = (price: number, pricingModel: PricingModel): string => {
  switch (pricingModel) {
    case PricingModel.Free:
      return 'Free';
    case PricingModel.OneTime:
      return `$${price.toFixed(2)}`;
    case PricingModel.Subscription:
      return `$${price.toFixed(2)}/month`;
    case PricingModel.Usage:
      return `$${price.toFixed(2)} per use`;
    case PricingModel.Enterprise:
      return 'Contact for pricing';
    default:
      return 'Price not available';
  }
};

export const getCategoryDisplayName = (category: ModuleCategory): string => {
  const categoryNames: Record<ModuleCategory, string> = {
    [ModuleCategory.BusinessManagement]: 'Business Management',
    [ModuleCategory.Analytics]: 'Analytics',
    [ModuleCategory.Communication]: 'Communication',
    [ModuleCategory.Integration]: 'Integration',
    [ModuleCategory.Security]: 'Security',
    [ModuleCategory.Productivity]: 'Productivity',
    [ModuleCategory.Development]: 'Development',
    [ModuleCategory.AI]: 'AI & Machine Learning',
    [ModuleCategory.Other]: 'Other',
  };
  
  return categoryNames[category] || category;
};

export const getInstallationStatusColor = (status: InstallationStatus): string => {
  const statusColors: Record<InstallationStatus, string> = {
    [InstallationStatus.NotInstalled]: 'text-gray-500',
    [InstallationStatus.Installing]: 'text-blue-500',
    [InstallationStatus.Installed]: 'text-yellow-500',
    [InstallationStatus.Activating]: 'text-blue-500',
    [InstallationStatus.Active]: 'text-green-500',
    [InstallationStatus.Deactivating]: 'text-yellow-500',
    [InstallationStatus.Uninstalling]: 'text-red-500',
    [InstallationStatus.Failed]: 'text-red-500',
  };
  
  return statusColors[status] || 'text-gray-500';
};

export const getInstallationStatusDisplayName = (status: InstallationStatus): string => {
  const statusNames: Record<InstallationStatus, string> = {
    [InstallationStatus.NotInstalled]: 'Not Installed',
    [InstallationStatus.Installing]: 'Installing...',
    [InstallationStatus.Installed]: 'Installed',
    [InstallationStatus.Activating]: 'Activating...',
    [InstallationStatus.Active]: 'Active',
    [InstallationStatus.Deactivating]: 'Deactivating...',
    [InstallationStatus.Uninstalling]: 'Uninstalling...',
    [InstallationStatus.Failed]: 'Failed',
  };
  
  return statusNames[status] || status;
};

export const getPlatformIcon = (platform: Platform): string => {
  const platformIcons: Record<Platform, string> = {
    [Platform.Web]: '🌐',
    [Platform.Desktop]: '💻',
    [Platform.Mobile]: '📱',
  };
  
  return platformIcons[platform] || '❓';
};

export const isModuleCompatible = (module: Module, currentAdxVersion: string): boolean => {
  const { minAdxVersion, maxAdxVersion } = module.compatibility;
  
  if (!semver.valid(currentAdxVersion)) {
    return false;
  }
  
  if (minAdxVersion && !semver.gte(currentAdxVersion, minAdxVersion)) {
    return false;
  }
  
  if (maxAdxVersion && !semver.lte(currentAdxVersion, maxAdxVersion)) {
    return false;
  }
  
  return true;
};

export const getSecurityScoreColor = (score: number): string => {
  if (score >= 90) return 'text-green-500';
  if (score >= 70) return 'text-yellow-500';
  if (score >= 50) return 'text-orange-500';
  return 'text-red-500';
};

export const getSecurityScoreLabel = (score: number): string => {
  if (score >= 90) return 'Excellent';
  if (score >= 70) return 'Good';
  if (score >= 50) return 'Fair';
  return 'Poor';
};

export const formatFileSize = (bytes: number): string => {
  const units = ['B', 'KB', 'MB', 'GB'];
  let size = bytes;
  let unitIndex = 0;
  
  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }
  
  return `${size.toFixed(1)} ${units[unitIndex]}`;
};

export const formatDuration = (milliseconds: number): string => {
  if (milliseconds < 1000) {
    return `${milliseconds}ms`;
  }
  
  const seconds = milliseconds / 1000;
  if (seconds < 60) {
    return `${seconds.toFixed(1)}s`;
  }
  
  const minutes = seconds / 60;
  if (minutes < 60) {
    return `${minutes.toFixed(1)}m`;
  }
  
  const hours = minutes / 60;
  return `${hours.toFixed(1)}h`;
};

export const generateModuleId = (name: string): string => {
  return name
    .toLowerCase()
    .replace(/[^a-z0-9]/g, '-')
    .replace(/-+/g, '-')
    .replace(/^-|-$/g, '');
};

export const validateModuleName = (name: string): string | null => {
  if (!name || name.trim().length === 0) {
    return 'Module name is required';
  }
  
  if (name.length < 3) {
    return 'Module name must be at least 3 characters long';
  }
  
  if (name.length > 50) {
    return 'Module name must be less than 50 characters long';
  }
  
  if (!/^[a-zA-Z0-9\s\-_]+$/.test(name)) {
    return 'Module name can only contain letters, numbers, spaces, hyphens, and underscores';
  }
  
  return null;
};

export const validateVersion = (version: string): string | null => {
  if (!version || version.trim().length === 0) {
    return 'Version is required';
  }
  
  if (!semver.valid(version)) {
    return 'Version must be a valid semantic version (e.g., 1.0.0)';
  }
  
  return null;
};

export const sortModules = (modules: Module[], sortBy: string, sortOrder: 'asc' | 'desc' = 'desc'): Module[] => {
  return [...modules].sort((a, b) => {
    let aValue: any;
    let bValue: any;
    
    switch (sortBy) {
      case 'name':
        aValue = a.name.toLowerCase();
        bValue = b.name.toLowerCase();
        break;
      case 'rating':
        aValue = a.rating;
        bValue = b.rating;
        break;
      case 'downloads':
        aValue = a.downloads;
        bValue = b.downloads;
        break;
      case 'lastUpdated':
        aValue = new Date(a.lastUpdated).getTime();
        bValue = new Date(b.lastUpdated).getTime();
        break;
      case 'price':
        aValue = a.price || 0;
        bValue = b.price || 0;
        break;
      default:
        return 0;
    }
    
    if (aValue < bValue) {
      return sortOrder === 'asc' ? -1 : 1;
    }
    if (aValue > bValue) {
      return sortOrder === 'asc' ? 1 : -1;
    }
    return 0;
  });
};

export const filterModules = (modules: Module[], searchTerm: string): Module[] => {
  if (!searchTerm || searchTerm.trim().length === 0) {
    return modules;
  }
  
  const term = searchTerm.toLowerCase();
  
  return modules.filter(module => 
    module.name.toLowerCase().includes(term) ||
    module.description.toLowerCase().includes(term) ||
    module.author.name.toLowerCase().includes(term) ||
    module.tags.some(tag => tag.toLowerCase().includes(term))
  );
};