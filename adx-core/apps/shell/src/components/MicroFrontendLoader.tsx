import React from 'react';
import { LoaderIcon } from 'lucide-react';

const MicroFrontendLoader: React.FC = () => {
  return (
    <div className="flex items-center justify-center min-h-[400px]">
      <div className="text-center">
        <LoaderIcon className="h-8 w-8 animate-spin text-blue-500 mx-auto mb-4" />
        <p className="text-gray-600 dark:text-gray-400">Loading application...</p>
      </div>
    </div>
  );
};

export default MicroFrontendLoader;