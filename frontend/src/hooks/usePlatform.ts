import { useContext } from 'react'
import { PlatformContext } from '@/contexts/PlatformContext'

export function usePlatform() {
  const context = useContext(PlatformContext)
  if (context === undefined) {
    throw new Error('usePlatform must be used within a PlatformProvider')
  }
  return context
}