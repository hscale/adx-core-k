import React, { useEffect, useState } from 'react';
import { useNavigate, useSearchParams } from 'react-router-dom';
import { AlertCircle, Loader2, CheckCircle } from 'lucide-react';
import { useAuth } from '../hooks';
import { SSOProvider } from '../types';

interface SSOLoginProps {
  provider: SSOProvider;
}

export const SSOLogin: React.FC<SSOLoginProps> = ({ provider }) => {
  const navigate = useNavigate();
  const [searchParams] = useSearchParams();
  const { initiateSSO, error, clearError } = useAuth();
  const [status, setStatus] = useState<'initiating' | 'redirecting' | 'processing' | 'success' | 'error'>('initiating');

  const providerConfig = {
    [SSOProvider.GOOGLE]: {
      name: 'Google',
      color: 'bg-red-500',
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24">
          <path
            fill="currentColor"
            d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"
          />
          <path
            fill="currentColor"
            d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"
          />
          <path
            fill="currentColor"
            d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"
          />
          <path
            fill="currentColor"
            d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"
          />
        </svg>
      ),
    },
    [SSOProvider.MICROSOFT]: {
      name: 'Microsoft',
      color: 'bg-blue-500',
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24">
          <path fill="#f25022" d="M1 1h10v10H1z" />
          <path fill="#00a4ef" d="M13 1h10v10H13z" />
          <path fill="#7fba00" d="M1 13h10v10H1z" />
          <path fill="#ffb900" d="M13 13h10v10H13z" />
        </svg>
      ),
    },
    [SSOProvider.GITHUB]: {
      name: 'GitHub',
      color: 'bg-gray-800',
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 0c-6.626 0-12 5.373-12 12 0 5.302 3.438 9.8 8.207 11.387.599.111.793-.261.793-.577v-2.234c-3.338.726-4.033-1.416-4.033-1.416-.546-1.387-1.333-1.756-1.333-1.756-1.089-.745.083-.729.083-.729 1.205.084 1.839 1.237 1.839 1.237 1.07 1.834 2.807 1.304 3.492.997.107-.775.418-1.305.762-1.604-2.665-.305-5.467-1.334-5.467-5.931 0-1.311.469-2.381 1.236-3.221-.124-.303-.535-1.524.117-3.176 0 0 1.008-.322 3.301 1.23.957-.266 1.983-.399 3.003-.404 1.02.005 2.047.138 3.006.404 2.291-1.552 3.297-1.23 3.297-1.23.653 1.653.242 2.874.118 3.176.77.84 1.235 1.911 1.235 3.221 0 4.609-2.807 5.624-5.479 5.921.43.372.823 1.102.823 2.222v3.293c0 .319.192.694.801.576 4.765-1.589 8.199-6.086 8.199-11.386 0-6.627-5.373-12-12-12z" />
        </svg>
      ),
    },
    [SSOProvider.OKTA]: {
      name: 'Okta',
      color: 'bg-blue-600',
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm0-14c-3.31 0-6 2.69-6 6s2.69 6 6 6 6-2.69 6-6-2.69-6-6-6z" />
        </svg>
      ),
    },
    [SSOProvider.SAML]: {
      name: 'SAML',
      color: 'bg-purple-600',
      icon: (
        <svg className="h-5 w-5" viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2L2 7v10c0 5.55 3.84 9.74 9 11 5.16-1.26 9-5.45 9-11V7l-10-5z" />
        </svg>
      ),
    },
  };

  const config = providerConfig[provider];

  useEffect(() => {
    const code = searchParams.get('code');
    const state = searchParams.get('state');
    const error = searchParams.get('error');

    if (error) {
      setStatus('error');
      return;
    }

    if (code && state) {
      // Handle SSO callback
      setStatus('processing');
      // This would be handled by the auth service
      // For now, we'll simulate success
      setTimeout(() => {
        setStatus('success');
        navigate('/dashboard');
      }, 2000);
    } else {
      // Initiate SSO
      const initiateSSOFlow = async () => {
        try {
          clearError();
          setStatus('initiating');
          
          await initiateSSO({
            provider,
            redirectUrl: window.location.origin + '/auth/sso/callback',
          });
          
          setStatus('redirecting');
        } catch (error) {
          console.error('SSO initiation failed:', error);
          setStatus('error');
        }
      };

      initiateSSOFlow();
    }
  }, [provider, searchParams, initiateSSO, clearError, navigate]);

  const getStatusMessage = () => {
    switch (status) {
      case 'initiating':
        return `Connecting to ${config.name}...`;
      case 'redirecting':
        return `Redirecting to ${config.name}...`;
      case 'processing':
        return 'Processing authentication...';
      case 'success':
        return 'Authentication successful!';
      case 'error':
        return 'Authentication failed';
      default:
        return 'Loading...';
    }
  };

  const getStatusIcon = () => {
    switch (status) {
      case 'initiating':
      case 'redirecting':
      case 'processing':
        return <Loader2 className="loading-spinner" />;
      case 'success':
        return <CheckCircle className="h-8 w-8 text-green-500" />;
      case 'error':
        return <AlertCircle className="h-8 w-8 text-red-500" />;
      default:
        return <Loader2 className="loading-spinner" />;
    }
  };

  return (
    <div className="auth-container">
      <div className="auth-card">
        <div className="text-center">
          <div className={`mx-auto h-16 w-16 rounded-full ${config.color} flex items-center justify-center text-white mb-6`}>
            {config.icon}
          </div>
          
          <h2 className="auth-title">Sign in with {config.name}</h2>
          
          <div className="my-8">
            {getStatusIcon()}
          </div>
          
          <p className="text-gray-600 mb-6">
            {getStatusMessage()}
          </p>

          {error && (
            <div className="alert alert-error mb-6">
              <AlertCircle className="h-4 w-4" />
              <span>{error}</span>
            </div>
          )}

          {status === 'error' && (
            <div className="space-y-4">
              <button
                onClick={() => window.location.reload()}
                className="btn btn-primary"
              >
                Try Again
              </button>
              <button
                onClick={() => navigate('/auth/login')}
                className="btn btn-secondary"
              >
                Back to Login
              </button>
            </div>
          )}

          {(status === 'initiating' || status === 'redirecting') && (
            <div className="text-center">
              <button
                onClick={() => navigate('/auth/login')}
                className="text-sm text-gray-500 hover:text-gray-700"
              >
                Cancel
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};