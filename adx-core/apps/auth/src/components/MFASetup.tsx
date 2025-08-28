import React from 'react';
import { Card, Button } from '@adx-core/design-system';
import { ShieldIcon } from 'lucide-react';

const MFASetup: React.FC = () => {
  return (
    <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
      <Card className="max-w-md w-full space-y-8 p-8">
        <div className="text-center">
          <ShieldIcon className="h-16 w-16 text-green-500 mx-auto mb-4" />
          <h2 className="text-3xl font-extrabold text-gray-900 dark:text-white">
            MFA Setup
          </h2>
          <p className="mt-2 text-gray-600 dark:text-gray-400">
            Multi-factor authentication setup coming soon
          </p>
          <Button className="mt-4" onClick={() => window.history.back()}>
            Go Back
          </Button>
        </div>
      </Card>
    </div>
  );
};

export default MFASetup;