import React, { useState, useEffect } from 'react';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { AlertCircle, Loader2, CheckCircle, Copy, Download, Smartphone } from 'lucide-react';
import QRCode from 'qrcode';
import { useAuth } from '../hooks';
import { mfaSetupSchema, type MFASetupFormData, extractWorkflowData } from '../utils';
import type { MFASecret } from '../types';

interface MFASetupProps {
  onComplete?: () => void;
  onSkip?: () => void;
}

export const MFASetup: React.FC<MFASetupProps> = ({ onComplete, onSkip }) => {
  const { setupMFA, confirmMFA, isMFASetupPending, isMFAConfirmPending, error, clearError } = useAuth();
  const [step, setStep] = useState<'setup' | 'verify' | 'complete'>('setup');
  const [mfaSecret, setMfaSecret] = useState<MFASecret | null>(null);
  const [qrCodeUrl, setQrCodeUrl] = useState<string>('');
  const [backupCodes, setBackupCodes] = useState<string[]>([]);
  const [copiedSecret, setCopiedSecret] = useState(false);
  const [copiedCodes, setCopiedCodes] = useState(false);

  const {
    register,
    handleSubmit,
    formState: { errors },
    reset,
  } = useForm<MFASetupFormData>({
    resolver: zodResolver(mfaSetupSchema),
    defaultValues: {
      secret: '',
      code: '',
    },
  });

  // Initialize MFA setup
  useEffect(() => {
    const initializeMFA = async () => {
      try {
        clearError();
        const secretResponse = await setupMFA();
        const secret = extractWorkflowData(secretResponse);
        setMfaSecret(secret);
        
        // Generate QR code
        const qrUrl = await QRCode.toDataURL(secret.qrCode);
        setQrCodeUrl(qrUrl);
        
        // Set secret in form
        reset({ secret: secret.secret, code: '' });
      } catch (error) {
        console.error('MFA setup failed:', error);
      }
    };

    if (step === 'setup') {
      initializeMFA();
    }
  }, [step, setupMFA, clearError, reset]);

  const onSubmit = async (data: MFASetupFormData) => {
    try {
      clearError();
      const resultResponse = await confirmMFA(data);
      const result = extractWorkflowData(resultResponse);
      setBackupCodes(result.backupCodes);
      setStep('complete');
    } catch (error) {
      console.error('MFA confirmation failed:', error);
    }
  };

  const copyToClipboard = async (text: string, type: 'secret' | 'codes') => {
    try {
      await navigator.clipboard.writeText(text);
      if (type === 'secret') {
        setCopiedSecret(true);
        setTimeout(() => setCopiedSecret(false), 2000);
      } else {
        setCopiedCodes(true);
        setTimeout(() => setCopiedCodes(false), 2000);
      }
    } catch (error) {
      console.error('Failed to copy to clipboard:', error);
    }
  };

  const downloadBackupCodes = () => {
    const content = `ADX Core - Backup Codes\n\nGenerated: ${new Date().toLocaleString()}\n\n${backupCodes.join('\n')}\n\nKeep these codes safe and secure. Each code can only be used once.`;
    const blob = new Blob([content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = 'adx-core-backup-codes.txt';
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleComplete = () => {
    if (onComplete) {
      onComplete();
    }
  };

  if (step === 'setup') {
    return (
      <div className="auth-container">
        <div className="auth-card">
          <div className="auth-header">
            <Smartphone className="mx-auto h-12 w-12 text-primary-600 mb-4" />
            <h2 className="auth-title">Set up Two-Factor Authentication</h2>
            <p className="auth-subtitle">
              Secure your account with an additional layer of protection
            </p>
          </div>

          {error && (
            <div className="alert alert-error">
              <AlertCircle className="h-4 w-4" />
              <span>{error}</span>
            </div>
          )}

          {isMFASetupPending ? (
            <div className="text-center py-8">
              <Loader2 className="loading-spinner mx-auto mb-4" />
              <p className="text-gray-600">Setting up two-factor authentication...</p>
            </div>
          ) : mfaSecret ? (
            <div className="space-y-6">
              <div className="text-center">
                <h3 className="text-lg font-medium text-gray-900 mb-4">
                  Step 1: Scan QR Code
                </h3>
                <p className="text-sm text-gray-600 mb-4">
                  Use your authenticator app (Google Authenticator, Authy, etc.) to scan this QR code:
                </p>
                
                {qrCodeUrl && (
                  <div className="inline-block p-4 bg-white border rounded-lg">
                    <img src={qrCodeUrl} alt="MFA QR Code" className="w-48 h-48" />
                  </div>
                )}
              </div>

              <div className="text-center">
                <h3 className="text-lg font-medium text-gray-900 mb-2">
                  Or enter this code manually:
                </h3>
                <div className="bg-gray-50 p-3 rounded-md border">
                  <code className="text-sm font-mono break-all">{mfaSecret.secret}</code>
                  <button
                    type="button"
                    onClick={() => copyToClipboard(mfaSecret.secret, 'secret')}
                    className="ml-2 text-primary-600 hover:text-primary-500"
                  >
                    {copiedSecret ? (
                      <CheckCircle className="h-4 w-4" />
                    ) : (
                      <Copy className="h-4 w-4" />
                    )}
                  </button>
                </div>
              </div>

              <div className="text-center">
                <button
                  onClick={() => setStep('verify')}
                  className="btn btn-primary"
                >
                  I've added the account
                </button>
              </div>

              {onSkip && (
                <div className="text-center">
                  <button
                    onClick={onSkip}
                    className="text-sm text-gray-500 hover:text-gray-700"
                  >
                    Skip for now
                  </button>
                </div>
              )}
            </div>
          ) : null}
        </div>
      </div>
    );
  }

  if (step === 'verify') {
    return (
      <div className="auth-container">
        <div className="auth-card">
          <div className="auth-header">
            <Smartphone className="mx-auto h-12 w-12 text-primary-600 mb-4" />
            <h2 className="auth-title">Verify Your Setup</h2>
            <p className="auth-subtitle">
              Enter the 6-digit code from your authenticator app
            </p>
          </div>

          {error && (
            <div className="alert alert-error">
              <AlertCircle className="h-4 w-4" />
              <span>{error}</span>
            </div>
          )}

          <form className="space-y-6" onSubmit={handleSubmit(onSubmit)}>
            <div className="form-group">
              <label htmlFor="code" className="form-label">
                Verification Code
              </label>
              <input
                {...register('code')}
                type="text"
                id="code"
                maxLength={6}
                className={`form-input text-center text-2xl tracking-widest ${errors.code ? 'form-input-error' : ''}`}
                placeholder="000000"
                autoComplete="one-time-code"
              />
              {errors.code && (
                <p className="form-error">{errors.code.message}</p>
              )}
            </div>

            <div className="flex space-x-3">
              <button
                type="button"
                onClick={() => setStep('setup')}
                className="btn btn-secondary flex-1"
              >
                Back
              </button>
              <button
                type="submit"
                disabled={isMFAConfirmPending}
                className="btn btn-primary flex-1"
              >
                {isMFAConfirmPending ? (
                  <>
                    <Loader2 className="loading-spinner mr-2" />
                    Verifying...
                  </>
                ) : (
                  'Verify & Enable'
                )}
              </button>
            </div>
          </form>
        </div>
      </div>
    );
  }

  if (step === 'complete') {
    return (
      <div className="auth-container">
        <div className="auth-card">
          <div className="text-center">
            <CheckCircle className="mx-auto h-12 w-12 text-green-500 mb-4" />
            <h2 className="auth-title text-green-600">MFA Enabled Successfully!</h2>
            <p className="text-gray-600 mb-6">
              Your account is now protected with two-factor authentication.
            </p>
          </div>

          <div className="space-y-6">
            <div>
              <h3 className="text-lg font-medium text-gray-900 mb-4">
                Save Your Backup Codes
              </h3>
              <p className="text-sm text-gray-600 mb-4">
                Store these backup codes in a safe place. You can use them to access your account if you lose your authenticator device.
              </p>
              
              <div className="bg-gray-50 p-4 rounded-md border">
                <div className="grid grid-cols-2 gap-2 text-sm font-mono">
                  {backupCodes.map((code, index) => (
                    <div key={index} className="text-center py-1">
                      {code}
                    </div>
                  ))}
                </div>
              </div>

              <div className="flex space-x-3 mt-4">
                <button
                  onClick={() => copyToClipboard(backupCodes.join('\n'), 'codes')}
                  className="btn btn-secondary flex-1"
                >
                  {copiedCodes ? (
                    <>
                      <CheckCircle className="h-4 w-4 mr-2" />
                      Copied!
                    </>
                  ) : (
                    <>
                      <Copy className="h-4 w-4 mr-2" />
                      Copy Codes
                    </>
                  )}
                </button>
                <button
                  onClick={downloadBackupCodes}
                  className="btn btn-secondary flex-1"
                >
                  <Download className="h-4 w-4 mr-2" />
                  Download
                </button>
              </div>
            </div>

            <div className="alert alert-warning">
              <AlertCircle className="h-4 w-4" />
              <div>
                <p className="font-medium">Important:</p>
                <p className="text-sm">Each backup code can only be used once. Generate new codes if you run out.</p>
              </div>
            </div>

            <button
              onClick={handleComplete}
              className="btn btn-primary btn-full"
            >
              Continue to Dashboard
            </button>
          </div>
        </div>
      </div>
    );
  }

  return null;
};