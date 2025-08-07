import React, { createContext, useContext, useEffect, useState } from 'react'
import { User, LoginCredentials, RegisterData } from '@/types'
import { authService } from '@/services/auth'
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query'
import toast from 'react-hot-toast'

interface AuthContextType {
  user: User | null
  isLoading: boolean
  isAuthenticated: boolean
  login: (credentials: LoginCredentials) => Promise<void>
  register: (data: RegisterData) => Promise<void>
  logout: () => Promise<void>
  updateProfile: (data: Partial<User>) => Promise<void>
  changePassword: (currentPassword: string, newPassword: string) => Promise<void>
}

export const AuthContext = createContext<AuthContextType | undefined>(undefined)

export function AuthProvider({ children }: { children: React.ReactNode }) {
  const [isInitialized, setIsInitialized] = useState(false)
  const queryClient = useQueryClient()

  // Get current user
  const {
    data: user,
    isLoading,
    error,
  } = useQuery({
    queryKey: ['auth', 'user'],
    queryFn: authService.getCurrentUser,
    enabled: authService.isAuthenticated() && isInitialized,
    retry: false,
    staleTime: 5 * 60 * 1000, // 5 minutes
  })

  // Initialize auth state
  useEffect(() => {
    const storedUser = authService.getStoredUser()
    if (storedUser && authService.isAuthenticated()) {
      queryClient.setQueryData(['auth', 'user'], storedUser)
    }
    setIsInitialized(true)
  }, [queryClient])

  // Handle auth errors
  useEffect(() => {
    if (error && authService.isAuthenticated()) {
      console.error('Auth error:', error)
      logout()
    }
  }, [error])

  // Login mutation
  const loginMutation = useMutation({
    mutationFn: authService.login,
    onSuccess: (response) => {
      queryClient.setQueryData(['auth', 'user'], response.user)
      toast.success('Welcome back!')
    },
    onError: (error: any) => {
      toast.error(error.message || 'Login failed')
    },
  })

  // Register mutation
  const registerMutation = useMutation({
    mutationFn: authService.register,
    onSuccess: (response) => {
      queryClient.setQueryData(['auth', 'user'], response.user)
      toast.success('Account created successfully!')
    },
    onError: (error: any) => {
      toast.error(error.message || 'Registration failed')
    },
  })

  // Logout mutation
  const logoutMutation = useMutation({
    mutationFn: authService.logout,
    onSuccess: () => {
      queryClient.clear()
      toast.success('Logged out successfully')
    },
    onError: (error: any) => {
      console.error('Logout error:', error)
      // Clear data anyway
      queryClient.clear()
    },
  })

  // Update profile mutation
  const updateProfileMutation = useMutation({
    mutationFn: authService.updateProfile,
    onSuccess: (updatedUser) => {
      queryClient.setQueryData(['auth', 'user'], updatedUser)
      localStorage.setItem('user', JSON.stringify(updatedUser))
      toast.success('Profile updated successfully')
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to update profile')
    },
  })

  // Change password mutation
  const changePasswordMutation = useMutation({
    mutationFn: ({ currentPassword, newPassword }: { currentPassword: string; newPassword: string }) =>
      authService.changePassword(currentPassword, newPassword),
    onSuccess: () => {
      toast.success('Password changed successfully')
    },
    onError: (error: any) => {
      toast.error(error.message || 'Failed to change password')
    },
  })

  const login = async (credentials: LoginCredentials) => {
    await loginMutation.mutateAsync(credentials)
  }

  const register = async (data: RegisterData) => {
    await registerMutation.mutateAsync(data)
  }

  const logout = async () => {
    await logoutMutation.mutateAsync()
  }

  const updateProfile = async (data: Partial<User>) => {
    await updateProfileMutation.mutateAsync(data)
  }

  const changePassword = async (currentPassword: string, newPassword: string) => {
    await changePasswordMutation.mutateAsync({ currentPassword, newPassword })
  }

  const value: AuthContextType = {
    user: user || null,
    isLoading: !isInitialized || isLoading,
    isAuthenticated: !!user && authService.isAuthenticated(),
    login,
    register,
    logout,
    updateProfile,
    changePassword,
  }

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>
}

export function useAuth() {
  const context = useContext(AuthContext)
  if (context === undefined) {
    throw new Error('useAuth must be used within an AuthProvider')
  }
  return context
}