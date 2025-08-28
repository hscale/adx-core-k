import React from 'react';
import { Card } from '@adx-core/design-system';
import { CreditCardIcon } from 'lucide-react';

const TenantBilling: React.FC = () => (
  <div className="p-6">
    <Card className="p-8 text-center">
      <CreditCardIcon className="h-16 w-16 text-green-500 mx-auto mb-4" />
      <h1 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Tenant Billing</h1>
      <p className="text-gray-600 dark:text-gray-400">Billing management coming soon</p>
    </Card>
  </div>
);

export default TenantBilling;