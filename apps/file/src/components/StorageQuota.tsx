import React from 'react';
import { HardDrive, AlertTriangle } from 'lucide-react';
import { StorageQuota as StorageQuotaType } from '../types/file';
import { formatFileSize } from '../utils/fileUtils';

interface StorageQuotaProps {
  quota: StorageQuotaType;
  className?: string;
  showDetails?: boolean;
}

export const StorageQuota: React.FC<StorageQuotaProps> = ({
  quota,
  className = '',
  showDetails = false,
}) => {
  const getQuotaColor = (percentage: number) => {
    if (percentage >= 90) return 'text-red-600';
    if (percentage >= 75) return 'text-yellow-600';
    return 'text-green-600';
  };

  const getProgressColor = (percentage: number) => {
    if (percentage >= 90) return 'bg-red-500';
    if (percentage >= 75) return 'bg-yellow-500';
    return 'bg-blue-500';
  };

  const isNearLimit = quota.percentage >= 85;

  return (
    <div className={`storage-quota ${className}`}>
      {/* Compact View */}
      {!showDetails && (
        <div className="flex items-center space-x-2 text-sm">
          <HardDrive className="w-4 h-4 text-gray-500" />
          <div className="flex items-center space-x-2">
            <div className="w-16 h-2 bg-gray-200 rounded-full overflow-hidden">
              <div
                className={`h-full transition-all duration-300 ${getProgressColor(quota.percentage)}`}
                style={{ width: `${Math.min(quota.percentage, 100)}%` }}
              />
            </div>
            <span className={`font-medium ${getQuotaColor(quota.percentage)}`}>
              {Math.round(quota.percentage)}%
            </span>
          </div>
          {isNearLimit && (
            <AlertTriangle className="w-4 h-4 text-yellow-500" />
          )}
        </div>
      )}

      {/* Detailed View */}
      {showDetails && (
        <div className="bg-white border border-gray-200 rounded-lg p-4">
          <div className="flex items-center justify-between mb-3">
            <div className="flex items-center space-x-2">
              <HardDrive className="w-5 h-5 text-gray-500" />
              <h3 className="text-sm font-medium text-gray-900">Storage Usage</h3>
            </div>
            {isNearLimit && (
              <div className="flex items-center space-x-1 text-yellow-600">
                <AlertTriangle className="w-4 h-4" />
                <span className="text-xs font-medium">Near Limit</span>
              </div>
            )}
          </div>

          {/* Progress Bar */}
          <div className="mb-3">
            <div className="flex justify-between text-xs text-gray-600 mb-1">
              <span>{formatFileSize(quota.used)} used</span>
              <span>{formatFileSize(quota.limit)} total</span>
            </div>
            <div className="w-full h-3 bg-gray-200 rounded-full overflow-hidden">
              <div
                className={`h-full transition-all duration-300 ${getProgressColor(quota.percentage)}`}
                style={{ width: `${Math.min(quota.percentage, 100)}%` }}
              />
            </div>
            <div className="text-center mt-1">
              <span className={`text-sm font-medium ${getQuotaColor(quota.percentage)}`}>
                {Math.round(quota.percentage)}% used
              </span>
            </div>
          </div>

          {/* Breakdown */}
          <div className="space-y-2">
            <h4 className="text-xs font-medium text-gray-700 uppercase tracking-wide">
              Storage Breakdown
            </h4>
            <div className="grid grid-cols-2 gap-2 text-xs">
              <div className="flex justify-between">
                <span className="text-gray-600">Files:</span>
                <span className="font-medium">{formatFileSize(quota.breakdown.files)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Images:</span>
                <span className="font-medium">{formatFileSize(quota.breakdown.images)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Documents:</span>
                <span className="font-medium">{formatFileSize(quota.breakdown.documents)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Videos:</span>
                <span className="font-medium">{formatFileSize(quota.breakdown.videos)}</span>
              </div>
              <div className="flex justify-between">
                <span className="text-gray-600">Other:</span>
                <span className="font-medium">{formatFileSize(quota.breakdown.other)}</span>
              </div>
            </div>
          </div>

          {/* Warning Message */}
          {isNearLimit && (
            <div className="mt-3 p-2 bg-yellow-50 border border-yellow-200 rounded text-xs">
              <p className="text-yellow-800">
                You're approaching your storage limit. Consider deleting unused files or upgrading your plan.
              </p>
            </div>
          )}

          {/* Available Space */}
          <div className="mt-3 pt-3 border-t border-gray-100">
            <div className="flex justify-between text-xs">
              <span className="text-gray-600">Available:</span>
              <span className="font-medium text-green-600">
                {formatFileSize(quota.limit - quota.used)}
              </span>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};