import React, { createContext, useContext, useRef, useCallback } from 'react';

type EventHandler = (data: any) => void;
type EventType = string;

interface EventBusContextType {
  emit: (eventType: EventType, data?: any) => void;
  subscribe: (eventType: EventType, handler: EventHandler) => () => void;
  subscribePattern: (pattern: string, handler: EventHandler) => () => void;
}

const EventBusContext = createContext<EventBusContextType | null>(null);

export const useEventBus = () => {
  const context = useContext(EventBusContext);
  if (!context) {
    throw new Error('useEventBus must be used within an EventBusProvider');
  }
  return context;
};

interface EventBusProviderProps {
  children: React.ReactNode;
}

export const EventBusProvider: React.FC<EventBusProviderProps> = ({ children }) => {
  const listeners = useRef<Map<EventType, Set<EventHandler>>>(new Map());
  const patternListeners = useRef<Map<string, Set<EventHandler>>>(new Map());

  const emit = useCallback((eventType: EventType, data?: any) => {
    // Emit to exact listeners
    const exactListeners = listeners.current.get(eventType);
    if (exactListeners) {
      exactListeners.forEach(handler => {
        try {
          handler({ type: eventType, data, timestamp: Date.now() });
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
            handler({ type: eventType, data, timestamp: Date.now() });
          } catch (error) {
            console.error(`Error in pattern handler for ${pattern}:`, error);
          }
        });
      }
    });
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

  return (
    <EventBusContext.Provider value={{ emit, subscribe, subscribePattern }}>
      {children}
    </EventBusContext.Provider>
  );
};

// Pattern matching utility
function matchesPattern(eventType: string, pattern: string): boolean {
  if (pattern.endsWith('*')) {
    const prefix = pattern.slice(0, -1);
    return eventType.startsWith(prefix);
  }
  return eventType === pattern;
}