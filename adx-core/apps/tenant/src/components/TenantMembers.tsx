import React from 'react';
import { Card } from '@adx-core/design-system';
import { UsersIcon } from 'lucide-react';

const TenantMembers: React.FC = () => (
  <div className="p-6">
    <Card className="p-8 text-center">
      <UsersIcon className="h-16 w-16 text-blue-500 mx-auto mb-4" />
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Tenant Members</h1>
      <p className="text-gray-600 dark:text-gray-400">Member management coming soon</p>
    </Card>
  </div>
);

export default TenantMembers;