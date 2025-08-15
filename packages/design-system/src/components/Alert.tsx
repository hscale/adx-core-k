import React from 'react';
import { AlertCircle, CheckCircle, Info, XCircle } from 'lucide-react';
import { cn } from '../utils';

export interface AlertProps {
  variant?: 'info' | 'success' | 'warning' | 'error';
  title?: string;
  children: React.ReactNode;
  className?: string;
  icon?: boolean;
  onClose?: () => void;
}

const alertVariants = {
  info: {
    className: 'alert-info',
    icon: Info,
  },
  success: {
    className: 'alert-success',
    icon: CheckCircle,
  },
  warning: {
    className: 'alert-warning',
    icon: AlertCircle,
  },
  error: {
    className: 'alert-error',
    icon: XCircle,
  },
};

export function Alert({
  variant = 'info',
  title,
  children,
  className,
  icon = true,
  onClose,
}: AlertProps) {
  const variantConfig = alertVariants[variant];
  const IconComponent = variantConfig.icon;

  return (
    <div className={cn('alert', variantConfig.className, className)}>
      <div className="flex">
        {icon && (
          <div className="flex-shrink-0">
            <IconComponent className="h-5 w-5" />
          </div>
        )}
        <div className={cn('flex-1', icon && 'ml-3')}>
          {title && (
            <h3 className="text-sm font-medium mb-1">
              {title}
            </h3>
          )}
          <div className="text-sm">
            {children}
          </div>
        </div>
        {onClose && (
          <div className="ml-auto pl-3">
            <button
              onClick={onClose}
              className="inline-flex text-sm hover:opacity-75 transition-opacity"
              aria-label="Close alert"
            >
              <XCircle className="h-4 w-4" />
            </button>
          </div>
        )}
      </div>
    </div>
  );
}