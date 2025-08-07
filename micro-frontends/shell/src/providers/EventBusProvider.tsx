import React, { createContext, useContext, ReactNode, useCallback, useRef } from 'react'

type EventCallback = (data: any) => void

interface EventBus {
  emit: (event: string, data?: any) => void
  on: (event: string, callback: EventCallback) => () => void
  off: (event: string, callback: EventCallback) => void
}

const EventBusContext = createContext<EventBus | undefined>(undefined)

interface EventBusProviderProps {
  children: ReactNode
}

export const EventBusProvider: React.FC<EventBusProviderProps> = ({ children }) => {
  const listeners = useRef<Map<string, Set<EventCallback>>>(new Map())

  const emit = useCallback((event: string, data?: any) => {
    const eventListeners = listeners.current.get(event)
    if (eventListeners) {
      eventListeners.forEach(callback => {
        try {
          callback(data)
        } catch (error) {
          console.error(`Error in event listener for ${event}:`, error)
        }
      })
    }
  }, [])

  const on = useCallback((event: string, callback: EventCallback) => {
    if (!listeners.current.has(event)) {
      listeners.current.set(event, new Set())
    }
    
    const eventListeners = listeners.current.get(event)!
    eventListeners.add(callback)

    // Return unsubscribe function
    return () => {
      eventListeners.delete(callback)
      if (eventListeners.size === 0) {
        listeners.current.delete(event)
      }
    }
  }, [])

  const off = useCallback((event: string, callback: EventCallback) => {
    const eventListeners = listeners.current.get(event)
    if (eventListeners) {
      eventListeners.delete(callback)
      if (eventListeners.size === 0) {
        listeners.current.delete(event)
      }
    }
  }, [])

  const eventBus: EventBus = {
    emit,
    on,
    off
  }

  return (
    <EventBusContext.Provider value={eventBus}>
      {children}
    </EventBusContext.Provider>
  )
}

export const useEventBus = () => {
  const context = useContext(EventBusContext)
  if (context === undefined) {
    throw new Error('useEventBus must be used within an EventBusProvider')
  }
  return context
}

// Common event types for type safety
export const EventTypes = {
  AUTH_LOGIN: 'auth.login',
  AUTH_LOGOUT: 'auth.logout',
  TENANT_SWITCH: 'tenant.switch',
  FILE_UPLOAD: 'file.upload',
  FILE_DELETE: 'file.delete',
  USER_UPDATE: 'user.update',
  WORKFLOW_START: 'workflow.start',
  WORKFLOW_COMPLETE: 'workflow.complete',
} as const