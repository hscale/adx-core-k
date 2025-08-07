import { useState } from 'react'
import { Link } from 'react-router-dom'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { useTranslation } from 'react-i18next'
import { ArrowLeftIcon } from '@heroicons/react/24/outline'

import { authService } from '@/services/auth'
import Button from '@/components/ui/Button'
import toast from 'react-hot-toast'

const forgotPasswordSchema = z.object({
  email: z.string().email('Invalid email address'),
})

type ForgotPasswordFormData = z.infer<typeof forgotPasswordSchema>

export default function ForgotPasswordPage() {
  const { t } = useTranslation()
  const [isLoading, setIsLoading] = useState(false)
  const [isSubmitted, setIsSubmitted] = useState(false)

  const {
    register,
    handleSubmit,
    formState: { errors },
    setError,
  } = useForm<ForgotPasswordFormData>({
    resolver: zodResolver(forgotPasswordSchema),
  })

  const onSubmit = async (data: ForgotPasswordFormData) => {
    try {
      setIsLoading(true)
      await authService.forgotPassword(data.email)
      setIsSubmitted(true)
      toast.success(t('auth.forgotPassword.success'))
    } catch (error: any) {
      setError('root', {
        message: error.message || 'Failed to send reset email',
      })
    } finally {
      setIsLoading(false)
    }
  }

  if (isSubmitted) {
    return (
      <div className="space-y-6 text-center">
        <div className="mx-auto flex items-center justify-center h-12 w-12 rounded-full bg-green-100 dark:bg-green-900/20">
          <svg className="h-6 w-6 text-green-600 dark:text-green-400" fill="none" viewBox="0 0 24 24" strokeWidth={1.5} stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" d="M21.75 6.75v10.5a2.25 2.25 0 01-2.25 2.25h-15a2.25 2.25 0 01-2.25-2.25V6.75m19.5 0A2.25 2.25 0 0019.5 4.5h-15a2.25 2.25 0 00-2.25 2.25m19.5 0v.243a2.25 2.25 0 01-1.07 1.916l-7.5 4.615a2.25 2.25 0 01-2.36 0L3.32 8.91a2.25 2.25 0 01-1.07-1.916V6.75" />
          </svg>
        </div>
        
        <div>
          <h2 className="text-2xl font-bold text-gray-900 dark:text-white">
            Check your email
          </h2>
          <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
            We've sent a password reset link to your email address.
          </p>
        </div>

        <div className="space-y-4">
          <p className="text-sm text-gray-500 dark:text-gray-400">
            Didn't receive the email? Check your spam folder or try again.
          </p>
          
          <Button
            variant="outline"
            onClick={() => setIsSubmitted(false)}
            className="w-full"
          >
            Try again
          </Button>
        </div>

        <div className="text-center">
          <Link
            to="/auth/login"
            className="inline-flex items-center text-sm font-medium text-primary-600 hover:text-primary-500 dark:text-primary-400"
          >
            <ArrowLeftIcon className="h-4 w-4 mr-1" />
            {t('auth.forgotPassword.backToLogin')}
          </Link>
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-3xl font-bold tracking-tight text-gray-900 dark:text-white">
          {t('auth.forgotPassword.title')}
        </h2>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          {t('auth.forgotPassword.subtitle')}
        </p>
      </div>

      <form className="space-y-6" onSubmit={handleSubmit(onSubmit)}>
        {errors.root && (
          <div className="rounded-md bg-red-50 dark:bg-red-900/20 p-4">
            <div className="text-sm text-red-700 dark:text-red-400">
              {errors.root.message}
            </div>
          </div>
        )}

        <div>
          <label htmlFor="email" className="block text-sm font-medium text-gray-700 dark:text-gray-300">
            {t('auth.forgotPassword.email')}
          </label>
          <div className="mt-1">
            <input
              {...register('email')}
              type="email"
              autoComplete="email"
              className={`input ${errors.email ? 'input-error' : ''}`}
              placeholder="you@example.com"
            />
            {errors.email && (
              <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                {errors.email.message}
              </p>
            )}
          </div>
        </div>

        <div>
          <Button
            type="submit"
            className="w-full"
            loading={isLoading}
            disabled={isLoading}
          >
            {t('auth.forgotPassword.sendReset')}
          </Button>
        </div>

        <div className="text-center">
          <Link
            to="/auth/login"
            className="inline-flex items-center text-sm font-medium text-primary-600 hover:text-primary-500 dark:text-primary-400"
          >
            <ArrowLeftIcon className="h-4 w-4 mr-1" />
            {t('auth.forgotPassword.backToLogin')}
          </Link>
        </div>
      </form>
    </div>
  )
}