import React from 'react';
import { Card, Button } from '@adx-core/design-system';
import { UserPlusIcon } from 'lucide-react';

const RegisterForm: React.FC = () => {
  return (
    <div className="min-h-screen flex items-center justify-center py-12 px-4 sm:px-6 lg:px-8">
      <Card className="max-w-md w-full space-y-8 p-8">
        <div className="text-center">
          <UserPlusIcon className="h-16 w-16 text-blue-500 mx-auto mb-4" />
          <h2 className="text-3xl font-extrabold text-gray-900 dark:text-white">
            Create Account
          </h2>
          <p className="mt-2 text-gray-600 dark:text-gray-400">
            Registration form coming soon
          </p>
          <Button className="mt-4" onClick={() => window.history.back()}>
            Go Back
          </Button>
        </div>
      </Card>
    </div>
  );
};

export default RegisterForm;