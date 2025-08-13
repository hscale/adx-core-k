import React, { createContext, useContext, useRef, useCallback, ReactNode } from 'react';
import type { EventHandler, EventType, EventBusContextType, EventData } from './types';

const EventBusContext = createContext<EventBusContextType | null>(null);

interface EventBusProviderProps {
  children: ReactNode;
}

export const EventBusProvider: React.FC<EventBusProviderProps> = ({ children }) => {
  const listeners = useRef<Map<EventType, Set<EventHandler>>>(new Map());
  const patternListeners = useRef<Map<string, Set<EventHandler>>>(new Map());

  const emit = useCallback((eventType: EventType, data?: any, source?: string) => {
    const eventData: EventData = {
      type: eventType,
      data,
      timestamp: Date.now(),
      source,
    };

    // Emit to exact listeners
    const exactListeners = listeners.current.get(eventType);
    if (exactListeners) {
      exactListeners.forEach(handler => {
        try {
          handler(eventData);
        } catch (error) {
          console.error(`Error in event handler for ${eventType}:`, error);
        }
      });
    }

    // Emit to pattern listeners
    patternListeners.current.forEach((handlers, pattern) => {
      if (matchesPattern(eventType, pattern)) {
        handlers.forEach(handler => {
          try {
            handler(eventData);
          } catch (error) {
            console.error(`Error in pattern handler for ${pattern}:`, error);
          }
        });
      }
    });

    // Also emit as custom DOM event for cross-micro-frontend communication
    if (typeof window !== 'undefined') {
      window.dispatchEvent(new CustomEvent(`adx:${eventType}`, {
        detail: eventData
      }));
    }
  }, []);

  const subscribe = useCallback((eventType: EventType, handler: EventHandler) => {
    if (!listeners.current.has(eventType)) {
      listeners.current.set(eventType, new Set());
    }
    listeners.current.get(eventType)!.add(handler);

    return () => {
      const eventListeners = listeners.current.get(eventType);
      if (eventListeners) {
        eventListeners.delete(handler);
        if (eventListeners.size === 0) {
          listeners.current.delete(eventType);
        }
      }
    };
  }, []);

  const subscribePattern = useCallback((pattern: string, handler: EventHandler) => {
    if (!patternListeners.current.has(pattern)) {
      patternListeners.current.set(pattern, new Set());
    }
    patternListeners.current.get(pattern)!.add(handler);

    return () => {
      const patternHandlers = patternListeners.current.get(pattern);
      if (patternHandlers) {
        patternHandlers.delete(handler);
        if (patternHandlers.size === 0) {
          patternListeners.current.delete(pattern);
        }
      }
    };
  }, []);

  const unsubscribe = useCallback((eventType: EventType, handler: EventHandler) => {
    const eventListeners = listeners.current.get(eventType);
    if (eventListeners) {
      eventListeners.delete(handler);
      if (eventListeners.size === 0) {
        listeners.current.delete(eventType);
      }
    }
  }, []);

  const clear = useCallback(() => {
    listeners.current.clear();
    patternListeners.current.clear();
  }, []);

  const contextValue: EventBusContextType = {
    emit,
    subscribe,
    subscribePattern,
    unsubscribe,
    clear,
  };

  return (
    <EventBusContext.Provider value={contextValue}>
      {children}
    </EventBusContext.Provider>
  );
};

export const useEventBus = (): EventBusContextType => {
  const context = useContext(EventBusContext);
  if (!context) {
    throw new Error('useEventBus must be used within EventBusProvider');
  }
  return context;
};

// Pattern matching utility
function matchesPattern(eventType: string, pattern: string): boolean {
  if (pattern.endsWith('*')) {
    const prefix = pattern.slice(0, -1);
    return eventType.startsWith(prefix);
  }
  if (pattern.includes('*')) {
    const regex = new RegExp('^' + pattern.replace(/\*/g, '.*') + '$');
    return regex.test(eventType);
  }
  return eventType === pattern;
}