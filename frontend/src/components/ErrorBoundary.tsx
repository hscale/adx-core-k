import React from 'react'
import { ExclamationTriangleIcon } from '@heroicons/react/24/outline'
import Button from './ui/Button'

interface ErrorBoundaryState {
  hasError: boolean
  error?: Error
}

interface ErrorBoundaryProps {
  children: React.ReactNode
}

export default class ErrorBoundary extends React.Component<ErrorBoundaryProps, ErrorBoundaryState> {
  constructor(props: ErrorBoundaryProps) {
    super(props)
    this.state = { hasError: false }
  }

  static getDerivedStateFromError(error: Error): ErrorBoundaryState {
    return { hasError: true, error }
  }

  componentDidCatch(error: Error, errorInfo: React.ErrorInfo) {
    console.error('Error caught by boundary:', error, errorInfo)
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-gray-900">
          <div className="max-w-md w-full bg-white dark:bg-gray-800 shadow-lg rounded-lg p-6">
            <div className="flex items-center justify-center w-12 h-12 mx-auto bg-red-100 dark:bg-red-900/20 rounded-full">
              <ExclamationTriangleIcon className="w-6 h-6 text-red-600 dark:text-red-400" />
            </div>
            
            <div className="mt-4 text-center">
              <h1 className="text-lg font-semibold text-gray-900 dark:text-white">
                Something went wrong
              </h1>
              <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
                We're sorry, but something unexpected happened. Please try refreshing the page.
              </p>
              
              {process.env.NODE_ENV === 'development' && this.state.error && (
                <details className="mt-4 text-left">
                  <summary className="cursor-pointer text-sm text-gray-500 dark:text-gray-400">
                    Error details
                  </summary>
                  <pre className="mt-2 text-xs text-red-600 dark:text-red-400 bg-red-50 dark:bg-red-900/10 p-2 rounded overflow-auto">
                    {this.state.error.stack}
                  </pre>
                </details>
              )}
            </div>
            
            <div className="mt-6 flex gap-3">
              <Button
                onClick={() => window.location.reload()}
                className="flex-1"
              >
                Refresh Page
              </Button>
              <Button
                variant="outline"
                onClick={() => this.setState({ hasError: false, error: undefined })}
                className="flex-1"
              >
                Try Again
              </Button>
            </div>
          </div>
        </div>
      )
    }

    return this.props.children
  }
}