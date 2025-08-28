import React from 'react';
import { Card } from '@adx-core/design-system';
import { BuildingIcon } from 'lucide-react';

const TenantDashboard: React.FC = () => (
  <div className="p-6">
    <Card className="p-8 text-center">
      <BuildingIcon className="h-16 w-16 text-orange-500 mx-auto mb-4" />
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Tenant Dashboard</h1>
      <p className="text-gray-600 dark:text-gray-400">Tenant management features coming soon</p>
    </Card>
  </div>
);

export default TenantDashboard;