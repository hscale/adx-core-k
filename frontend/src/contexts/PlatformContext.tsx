import React, { createContext, useContext, useEffect, useState } from 'react'
import { PlatformInfo, PlatformCapabilities } from '@/types'

interface PlatformContextType {
  platform: PlatformInfo
  isOnline: boolean
}

export const PlatformContext = createContext<PlatformContextType | undefined>(undefined)

export function PlatformProvider({ children }: { children: React.ReactNode }) {
  const [platform, setPlatform] = useState<PlatformInfo>(() => {
    return detectPlatform()
  })
  
  const [isOnline, setIsOnline] = useState(navigator.onLine)

  // Detect platform capabilities
  function detectPlatform(): PlatformInfo {
    const userAgent = navigator.userAgent.toLowerCase()
    const isMobile = /android|webos|iphone|ipad|ipod|blackberry|iemobile|opera mini/i.test(userAgent)
    const isDesktop = !isMobile && (window as any).__TAURI__ !== undefined
    const isWeb = !isDesktop && !isMobile

    // Detect OS
    let os = 'unknown'
    if (userAgent.includes('windows')) os = 'windows'
    else if (userAgent.includes('mac')) os = 'macos'
    else if (userAgent.includes('linux')) os = 'linux'
    else if (userAgent.includes('android')) os = 'android'
    else if (userAgent.includes('ios') || userAgent.includes('iphone') || userAgent.includes('ipad')) os = 'ios'

    // Detect capabilities
    const capabilities: PlatformCapabilities = {
      notifications: 'Notification' in window,
      fileSystem: isDesktop || 'showOpenFilePicker' in window,
      camera: 'mediaDevices' in navigator && 'getUserMedia' in navigator.mediaDevices,
      geolocation: 'geolocation' in navigator,
      clipboard: 'clipboard' in navigator,
    }

    // Platform type
    let type: 'web' | 'desktop' | 'mobile' = 'web'
    if (isDesktop) type = 'desktop'
    else if (isMobile) type = 'mobile'

    return {
      type,
      os,
      version: (window as any).__TAURI__?.version || '1.0.0',
      isMobile,
      isDesktop,
      isWeb,
      capabilities,
    }
  }

  // Listen for online/offline events
  useEffect(() => {
    const handleOnline = () => setIsOnline(true)
    const handleOffline = () => setIsOnline(false)

    window.addEventListener('online', handleOnline)
    window.addEventListener('offline', handleOffline)

    return () => {
      window.removeEventListener('online', handleOnline)
      window.removeEventListener('offline', handleOffline)
    }
  }, [])

  // Update platform info on resize (for responsive detection)
  useEffect(() => {
    const handleResize = () => {
      setPlatform(detectPlatform())
    }

    window.addEventListener('resize', handleResize)
    return () => window.removeEventListener('resize', handleResize)
  }, [])

  // Add platform-specific CSS classes
  useEffect(() => {
    const body = document.body
    body.classList.remove('platform-web', 'platform-desktop', 'platform-mobile')
    body.classList.add(`platform-${platform.type}`)
    
    if (platform.os !== 'unknown') {
      body.classList.add(`os-${platform.os}`)
    }
  }, [platform])

  const value: PlatformContextType = {
    platform,
    isOnline,
  }

  return <PlatformContext.Provider value={value}>{children}</PlatformContext.Provider>
}

export function usePlatform() {
  const context = useContext(PlatformContext)
  if (context === undefined) {
    throw new Error('usePlatform must be used within a PlatformProvider')
  }
  return context
}