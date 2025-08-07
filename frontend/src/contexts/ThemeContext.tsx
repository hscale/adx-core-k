import React, { createContext, useContext, useEffect, useState } from 'react'
import { Theme } from '@/types'

interface ThemeContextType {
  theme: Theme
  setTheme: (theme: Theme) => void
  isDark: boolean
  toggleTheme: () => void
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined)

export function ThemeProvider({ children }: { children: React.ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => {
    const stored = localStorage.getItem('theme') as Theme
    return stored || 'auto'
  })

  const [isDark, setIsDark] = useState(false)

  // Update theme
  const setTheme = (newTheme: Theme) => {
    setThemeState(newTheme)
    localStorage.setItem('theme', newTheme)
  }

  // Toggle between light and dark
  const toggleTheme = () => {
    setTheme(isDark ? 'light' : 'dark')
  }

  // Apply theme to document
  useEffect(() => {
    const applyTheme = () => {
      const root = document.documentElement
      
      if (theme === 'auto') {
        const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
        setIsDark(prefersDark)
        root.classList.toggle('dark', prefersDark)
      } else {
        const isDarkTheme = theme === 'dark'
        setIsDark(isDarkTheme)
        root.classList.toggle('dark', isDarkTheme)
      }
    }

    applyTheme()

    // Listen for system theme changes when in auto mode
    if (theme === 'auto') {
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = () => applyTheme()
      
      mediaQuery.addEventListener('change', handleChange)
      return () => mediaQuery.removeEventListener('change', handleChange)
    }
  }, [theme])

  const value: ThemeContextType = {
    theme,
    setTheme,
    isDark,
    toggleTheme,
  }

  return <ThemeContext.Provider value={value}>{children}</ThemeContext.Provider>
}

export function useTheme() {
  const context = useContext(ThemeContext)
  if (context === undefined) {
    throw new Error('useTheme must be used within a ThemeProvider')
  }
  return context
}