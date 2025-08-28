export type EventHandler = (data: any) => void;
export type EventType = string;

export interface EventData {
  type: EventType;
  data?: any;
  timestamp: number;
  source?: string;
}

export interface EventBusContextType {
  emit: (eventType: EventType, data?: any, source?: string) => void;
  subscribe: (eventType: EventType, handler: EventHandler) => () => void;
  subscribePattern: (pattern: string, handler: EventHandler) => () => void;
  unsubscribe: (eventType: EventType, handler: EventHandler) => void;
  clear: () => void;
}