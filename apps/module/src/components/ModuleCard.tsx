import React from 'react';
import { Star, Download, Shield, ExternalLink, Check, Clock } from 'lucide-react';
import { Module } from '../types/module';
import { 
  formatPrice, 
  getCategoryDisplayName, 
  getPlatformIcon,
  getSecurityScoreColor,
  getSecurityScoreLabel
} from '../utils/moduleUtils';

interface ModuleCardProps {
  module: Module;
  isInstalled: boolean;
  onInstall: () => void;
  isInstalling: boolean;
}

export const ModuleCard: React.FC<ModuleCardProps> = ({
  module,
  isInstalled,
  onInstall,
  isInstalling,
}) => {
  return (
    <div className="bg-white rounded-lg shadow-sm border border-gray-200 hover:shadow-md transition-shadow">
      {/* Module Image/Icon */}
      <div className="aspect-video bg-gradient-to-br from-blue-50 to-indigo-100 rounded-t-lg flex items-center justify-center">
        {module.screenshots.length > 0 ? (
          <img
            src={module.screenshots[0].thumbnail || module.screenshots[0].url}
            alt={module.name}
            className="w-full h-full object-cover rounded-t-lg"
          />
        ) : (
          <div className="text-4xl font-bold text-blue-600">
            {module.name.charAt(0).toUpperCase()}
          </div>
        )}
      </div>

      <div className="p-4">
        {/* Header */}
        <div className="flex items-start justify-between mb-2">
          <div className="flex-1 min-w-0">
            <h3 className="font-semibold text-gray-900 truncate">{module.name}</h3>
            <p className="text-sm text-gray-600">by {module.author.name}</p>
          </div>
          <div className="flex items-center gap-1 text-sm text-gray-600">
            <Star className="w-4 h-4 fill-yellow-400 text-yellow-400" />
            {module.rating.toFixed(1)}
          </div>
        </div>

        {/* Description */}
        <p className="text-sm text-gray-600 mb-3 line-clamp-2">
          {module.description}
        </p>

        {/* Category and Platforms */}
        <div className="flex items-center justify-between mb-3">
          <span className="inline-flex items-center px-2 py-1 rounded-full text-xs font-medium bg-blue-100 text-blue-800">
            {getCategoryDisplayName(module.category)}
          </span>
          <div className="flex items-center gap-1">
            {module.supportedPlatforms.map((platform) => (
              <span key={platform} className="text-sm" title={platform}>
                {getPlatformIcon(platform)}
              </span>
            ))}
          </div>
        </div>

        {/* Stats */}
        <div className="flex items-center justify-between text-xs text-gray-500 mb-3">
          <div className="flex items-center gap-1">
            <Download className="w-3 h-3" />
            {module.downloads.toLocaleString()}
          </div>
          <div className="flex items-center gap-1">
            <Shield className={`w-3 h-3 ${getSecurityScoreColor(module.securityScanResults.score)}`} />
            {getSecurityScoreLabel(module.securityScanResults.score)}
          </div>
        </div>

        {/* Price */}
        <div className="flex items-center justify-between mb-4">
          <span className="font-semibold text-gray-900">
            {formatPrice(module.price || 0, module.pricingModel)}
          </span>
          <span className="text-xs text-gray-500">v{module.version}</span>
        </div>

        {/* Actions */}
        <div className="flex gap-2">
          {isInstalled ? (
            <div className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-green-50 text-green-700 rounded-lg">
              <Check className="w-4 h-4" />
              Installed
            </div>
          ) : (
            <button
              onClick={onInstall}
              disabled={isInstalling}
              className="flex-1 flex items-center justify-center gap-2 px-3 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {isInstalling ? (
                <>
                  <Clock className="w-4 h-4 animate-spin" />
                  Installing...
                </>
              ) : (
                <>
                  <Download className="w-4 h-4" />
                  Install
                </>
              )}
            </button>
          )}
          
          <button
            onClick={() => window.open(module.documentationUrl, '_blank')}
            className="px-3 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
            title="View Documentation"
          >
            <ExternalLink className="w-4 h-4" />
          </button>
        </div>

        {/* Tags */}
        {module.tags.length > 0 && (
          <div className="mt-3 flex flex-wrap gap-1">
            {module.tags.slice(0, 3).map((tag) => (
              <span
                key={tag}
                className="inline-flex items-center px-2 py-1 rounded text-xs bg-gray-100 text-gray-600"
              >
                {tag}
              </span>
            ))}
            {module.tags.length > 3 && (
              <span className="text-xs text-gray-500">
                +{module.tags.length - 3} more
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  );
};