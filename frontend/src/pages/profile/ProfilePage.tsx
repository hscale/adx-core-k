import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { useForm } from 'react-hook-form'
import { zodResolver } from '@hookform/resolvers/zod'
import { z } from 'zod'
import { 
  UserCircleIcon,
  CameraIcon,
  KeyIcon,
} from '@heroicons/react/24/outline'

import { useAuth } from '@/hooks/useAuth'
import Button from '@/components/ui/Button'
import { User } from '@/types'

const profileSchema = z.object({
  firstName: z.string().min(1, 'First name is required'),
  lastName: z.string().min(1, 'Last name is required'),
  email: z.string().email('Invalid email address'),
})

const passwordSchema = z.object({
  currentPassword: z.string().min(1, 'Current password is required'),
  newPassword: z.string().min(8, 'Password must be at least 8 characters'),
  confirmPassword: z.string(),
}).refine((data) => data.newPassword === data.confirmPassword, {
  message: "Passwords don't match",
  path: ["confirmPassword"],
})

type ProfileFormData = z.infer<typeof profileSchema>
type PasswordFormData = z.infer<typeof passwordSchema>

export default function ProfilePage() {
  const { t } = useTranslation()
  const { user, updateProfile, changePassword } = useAuth()
  const [isEditingProfile, setIsEditingProfile] = useState(false)
  const [isChangingPassword, setIsChangingPassword] = useState(false)

  const profileForm = useForm<ProfileFormData>({
    resolver: zodResolver(profileSchema),
    defaultValues: {
      firstName: user?.firstName || '',
      lastName: user?.lastName || '',
      email: user?.email || '',
    },
  })

  const passwordForm = useForm<PasswordFormData>({
    resolver: zodResolver(passwordSchema),
  })

  const onProfileSubmit = async (data: ProfileFormData) => {
    try {
      await updateProfile(data)
      setIsEditingProfile(false)
    } catch (error) {
      console.error('Failed to update profile:', error)
    }
  }

  const onPasswordSubmit = async (data: PasswordFormData) => {
    try {
      await changePassword(data.currentPassword, data.newPassword)
      setIsChangingPassword(false)
      passwordForm.reset()
    } catch (error) {
      console.error('Failed to change password:', error)
    }
  }

  if (!user) {
    return (
      <div className="text-center py-12">
        <div className="text-gray-500 dark:text-gray-400">
          Loading profile...
        </div>
      </div>
    )
  }

  return (
    <div className="space-y-6">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          {t('profile.title')}
        </h1>
        <p className="mt-2 text-sm text-gray-600 dark:text-gray-400">
          Manage your personal information and account settings
        </p>
      </div>

      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Profile Picture */}
        <div className="card">
          <div className="card-body text-center">
            <div className="relative inline-block">
              {user.avatar ? (
                <img
                  src={user.avatar}
                  alt={user.firstName || user.email}
                  className="h-24 w-24 rounded-full object-cover"
                />
              ) : (
                <div className="h-24 w-24 rounded-full bg-primary-100 dark:bg-primary-900/20 flex items-center justify-center">
                  <UserCircleIcon className="h-16 w-16 text-primary-600 dark:text-primary-400" />
                </div>
              )}
              <button className="absolute bottom-0 right-0 h-8 w-8 rounded-full bg-primary-600 text-white flex items-center justify-center hover:bg-primary-700 transition-colors">
                <CameraIcon className="h-4 w-4" />
              </button>
            </div>
            <div className="mt-4">
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                {user.firstName && user.lastName 
                  ? `${user.firstName} ${user.lastName}`
                  : user.email
                }
              </h3>
              <p className="text-sm text-gray-500 dark:text-gray-400">
                {user.role.charAt(0).toUpperCase() + user.role.slice(1)}
              </p>
            </div>
          </div>
        </div>

        {/* Profile Information */}
        <div className="lg:col-span-2">
          <div className="card">
            <div className="card-header">
              <div className="flex items-center justify-between">
                <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                  Profile Information
                </h3>
                {!isEditingProfile && (
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={() => setIsEditingProfile(true)}
                  >
                    Edit
                  </Button>
                )}
              </div>
            </div>
            <div className="card-body">
              {isEditingProfile ? (
                <form onSubmit={profileForm.handleSubmit(onProfileSubmit)} className="space-y-4">
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        First Name
                      </label>
                      <input
                        {...profileForm.register('firstName')}
                        type="text"
                        className="input mt-1"
                      />
                      {profileForm.formState.errors.firstName && (
                        <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                          {profileForm.formState.errors.firstName.message}
                        </p>
                      )}
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                        Last Name
                      </label>
                      <input
                        {...profileForm.register('lastName')}
                        type="text"
                        className="input mt-1"
                      />
                      {profileForm.formState.errors.lastName && (
                        <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                          {profileForm.formState.errors.lastName.message}
                        </p>
                      )}
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                      Email
                    </label>
                    <input
                      {...profileForm.register('email')}
                      type="email"
                      className="input mt-1"
                    />
                    {profileForm.formState.errors.email && (
                      <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                        {profileForm.formState.errors.email.message}
                      </p>
                    )}
                  </div>
                  <div className="flex gap-3">
                    <Button type="submit" size="sm">
                      Save Changes
                    </Button>
                    <Button
                      type="button"
                      variant="outline"
                      size="sm"
                      onClick={() => {
                        setIsEditingProfile(false)
                        profileForm.reset()
                      }}
                    >
                      Cancel
                    </Button>
                  </div>
                </form>
              ) : (
                <div className="space-y-4">
                  <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
                    <div>
                      <label className="block text-sm font-medium text-gray-500 dark:text-gray-400">
                        First Name
                      </label>
                      <div className="mt-1 text-sm text-gray-900 dark:text-white">
                        {user.firstName || 'Not set'}
                      </div>
                    </div>
                    <div>
                      <label className="block text-sm font-medium text-gray-500 dark:text-gray-400">
                        Last Name
                      </label>
                      <div className="mt-1 text-sm text-gray-900 dark:text-white">
                        {user.lastName || 'Not set'}
                      </div>
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500 dark:text-gray-400">
                      Email
                    </label>
                    <div className="mt-1 text-sm text-gray-900 dark:text-white">
                      {user.email}
                    </div>
                  </div>
                  <div>
                    <label className="block text-sm font-medium text-gray-500 dark:text-gray-400">
                      Member Since
                    </label>
                    <div className="mt-1 text-sm text-gray-900 dark:text-white">
                      {new Date(user.createdAt).toLocaleDateString()}
                    </div>
                  </div>
                  {user.lastLoginAt && (
                    <div>
                      <label className="block text-sm font-medium text-gray-500 dark:text-gray-400">
                        Last Login
                      </label>
                      <div className="mt-1 text-sm text-gray-900 dark:text-white">
                        {new Date(user.lastLoginAt).toLocaleString()}
                      </div>
                    </div>
                  )}
                </div>
              )}
            </div>
          </div>
        </div>
      </div>

      {/* Change Password */}
      <div className="card">
        <div className="card-header">
          <div className="flex items-center justify-between">
            <div className="flex items-center">
              <KeyIcon className="h-5 w-5 text-gray-400 mr-2" />
              <h3 className="text-lg font-medium text-gray-900 dark:text-white">
                Change Password
              </h3>
            </div>
            {!isChangingPassword && (
              <Button
                variant="outline"
                size="sm"
                onClick={() => setIsChangingPassword(true)}
              >
                Change Password
              </Button>
            )}
          </div>
        </div>
        {isChangingPassword && (
          <div className="card-body">
            <form onSubmit={passwordForm.handleSubmit(onPasswordSubmit)} className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Current Password
                </label>
                <input
                  {...passwordForm.register('currentPassword')}
                  type="password"
                  className="input mt-1"
                />
                {passwordForm.formState.errors.currentPassword && (
                  <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                    {passwordForm.formState.errors.currentPassword.message}
                  </p>
                )}
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  New Password
                </label>
                <input
                  {...passwordForm.register('newPassword')}
                  type="password"
                  className="input mt-1"
                />
                {passwordForm.formState.errors.newPassword && (
                  <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                    {passwordForm.formState.errors.newPassword.message}
                  </p>
                )}
              </div>
              <div>
                <label className="block text-sm font-medium text-gray-700 dark:text-gray-300">
                  Confirm New Password
                </label>
                <input
                  {...passwordForm.register('confirmPassword')}
                  type="password"
                  className="input mt-1"
                />
                {passwordForm.formState.errors.confirmPassword && (
                  <p className="mt-1 text-sm text-red-600 dark:text-red-400">
                    {passwordForm.formState.errors.confirmPassword.message}
                  </p>
                )}
              </div>
              <div className="flex gap-3">
                <Button type="submit" size="sm">
                  Update Password
                </Button>
                <Button
                  type="button"
                  variant="outline"
                  size="sm"
                  onClick={() => {
                    setIsChangingPassword(false)
                    passwordForm.reset()
                  }}
                >
                  Cancel
                </Button>
              </div>
            </form>
          </div>
        )}
      </div>
    </div>
  )
}