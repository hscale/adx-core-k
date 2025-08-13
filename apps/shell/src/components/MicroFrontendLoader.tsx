import React from 'react';
import { Loader2 } from 'lucide-react';

interface MicroFrontendLoaderProps {
  name?: string;
  message?: string;
}

export const MicroFrontendLoader: React.FC<MicroFrontendLoaderProps> = ({ 
  name, 
  message = 'Loading...' 
}) => {
  return (
    <div className="flex items-center justify-center min-h-[400px] p-4">
      <div className="text-center space-y-4">
        <div className="mx-auto flex h-12 w-12 items-center justify-center">
          <Loader2 className="h-8 w-8 animate-spin text-primary-600" />
        </div>
        <div className="space-y-2">
          <h3 className="text-lg font-medium text-gray-900 dark:text-gray-100">
            {name ? `Loading ${name}` : message}
          </h3>
          <p className="text-sm text-gray-600 dark:text-gray-400">
            Please wait while we load the application...
          </p>
        </div>
      </div>
    </div>
  );
};