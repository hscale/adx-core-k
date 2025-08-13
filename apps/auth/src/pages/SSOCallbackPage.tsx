import React from 'react';
import { useParams } from 'react-router-dom';
import { SSOLogin } from '../components';
import { SSOProvider } from '../types';

export const SSOCallbackPage: React.FC = () => {
  const { provider } = useParams<{ provider: string }>();

  const ssoProvider = provider?.toUpperCase() as SSOProvider;

  if (!ssoProvider || !Object.values(SSOProvider).includes(ssoProvider)) {
    return (
      <div className="auth-container">
        <div className="auth-card">
          <div className="text-center">
            <h2 className="auth-title text-red-600">Invalid SSO Provider</h2>
            <p className="text-gray-600">The requested SSO provider is not supported.</p>
          </div>
        </div>
      </div>
    );
  }

  return <SSOLogin provider={ssoProvider} />;
};