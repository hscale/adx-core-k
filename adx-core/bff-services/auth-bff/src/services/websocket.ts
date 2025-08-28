import { WebSocketServer, WebSocket } from 'ws';
import { IncomingMessage } from 'http';
import { parse as parseUrl } from 'url';
import jwt from 'jsonwebtoken';
import { v4 as uuidv4 } from 'uuid';
import { WebSocketMessage, AuthStatusUpdate } from '../types/auth.js';
import { RedisClient } from './redis.js';

export interface WebSocketConnection {
  id: string;
  ws: WebSocket;
  userId: string;
  tenantId: string;
  sessionId: string;
  lastPing: number;
  subscriptions: Set<string>;
}

export class WebSocketService {
  private wss: WebSocketServer;
  private connections = new Map<string, WebSocketConnection>();
  private userConnections = new Map<string, Set<string>>();
  private tenantConnections = new Map<string, Set<string>>();
  private heartbeatInterval: NodeJS.Timeout;
  private redisClient: RedisClient;

  constructor(
    server: any,
    redisClient: RedisClient,
    private jwtSecret: string,
    private heartbeatIntervalMs = 30000
  ) {
    this.redisClient = redisClient;
    
    this.wss = new WebSocketServer({
      server,
      path: '/ws',
      verifyClient: this.verifyClient.bind(this),
    });

    this.wss.on('connection', this.handleConnection.bind(this));
    this.startHeartbeat();

    console.log('🔌 WebSocket service initialized');
  }

  private verifyClient(info: { origin: string; secure: boolean; req: IncomingMessage }): boolean {
    try {
      const url = parseUrl(info.req.url || '', true);
      const token = url.query.token as string;

      if (!token) {
        console.log('WebSocket connection rejected: No token provided');
        return false;
      }

      // Verify JWT token
      jwt.verify(token, this.jwtSecret);
      return true;
    } catch (error) {
      console.log('WebSocket connection rejected: Invalid token', error.message);
      return false;
    }
  }

  private async handleConnection(ws: WebSocket, req: IncomingMessage): Promise<void> {
    try {
      const url = parseUrl(req.url || '', true);
      const token = url.query.token as string;
      
      const decoded = jwt.verify(token, this.jwtSecret) as any;
      const connectionId = uuidv4();

      const connection: WebSocketConnection = {
        id: connectionId,
        ws,
        userId: decoded.sub,
        tenantId: decoded.tenant_id,
        sessionId: decoded.session_id,
        lastPing: Date.now(),
        subscriptions: new Set(),
      };

      // Store connection
      this.connections.set(connectionId, connection);

      // Index by user and tenant
      if (!this.userConnections.has(connection.userId)) {
        this.userConnections.set(connection.userId, new Set());
      }
      this.userConnections.get(connection.userId)!.add(connectionId);

      if (!this.tenantConnections.has(connection.tenantId)) {
        this.tenantConnections.set(connection.tenantId, new Set());
      }
      this.tenantConnections.get(connection.tenantId)!.add(connectionId);

      // Set up event handlers
      ws.on('message', (data) => this.handleMessage(connectionId, data));
      ws.on('close', () => this.handleDisconnection(connectionId));
      ws.on('error', (error) => this.handleError(connectionId, error));
      ws.on('pong', () => this.handlePong(connectionId));

      // Send welcome message
      this.sendToConnection(connectionId, {
        type: 'connected',
        data: {
          connectionId,
          userId: connection.userId,
          tenantId: connection.tenantId,
        },
        timestamp: new Date().toISOString(),
      });

      console.log(`WebSocket connection established: ${connectionId} (User: ${connection.userId}, Tenant: ${connection.tenantId})`);
    } catch (error) {
      console.error('Error handling WebSocket connection:', error);
      ws.close(1008, 'Authentication failed');
    }
  }

  private handleMessage(connectionId: string, data: any): void {
    try {
      const connection = this.connections.get(connectionId);
      if (!connection) return;

      const message = JSON.parse(data.toString());
      
      switch (message.type) {
        case 'ping':
          this.sendToConnection(connectionId, {
            type: 'pong',
            data: { timestamp: new Date().toISOString() },
            timestamp: new Date().toISOString(),
          });
          break;

        case 'subscribe':
          if (message.channels && Array.isArray(message.channels)) {
            message.channels.forEach((channel: string) => {
              connection.subscriptions.add(channel);
            });
            this.sendToConnection(connectionId, {
              type: 'subscribed',
              data: { channels: message.channels },
              timestamp: new Date().toISOString(),
            });
          }
          break;

        case 'unsubscribe':
          if (message.channels && Array.isArray(message.channels)) {
            message.channels.forEach((channel: string) => {
              connection.subscriptions.delete(channel);
            });
            this.sendToConnection(connectionId, {
              type: 'unsubscribed',
              data: { channels: message.channels },
              timestamp: new Date().toISOString(),
            });
          }
          break;

        default:
          console.log(`Unknown WebSocket message type: ${message.type}`);
      }
    } catch (error) {
      console.error(`Error handling WebSocket message from ${connectionId}:`, error);
    }
  }

  private handleDisconnection(connectionId: string): void {
    const connection = this.connections.get(connectionId);
    if (!connection) return;

    // Remove from indexes
    const userConnections = this.userConnections.get(connection.userId);
    if (userConnections) {
      userConnections.delete(connectionId);
      if (userConnections.size === 0) {
        this.userConnections.delete(connection.userId);
      }
    }

    const tenantConnections = this.tenantConnections.get(connection.tenantId);
    if (tenantConnections) {
      tenantConnections.delete(connectionId);
      if (tenantConnections.size === 0) {
        this.tenantConnections.delete(connection.tenantId);
      }
    }

    // Remove connection
    this.connections.delete(connectionId);

    console.log(`WebSocket connection closed: ${connectionId}`);
  }

  private handleError(connectionId: string, error: Error): void {
    console.error(`WebSocket error for connection ${connectionId}:`, error);
    this.handleDisconnection(connectionId);
  }

  private handlePong(connectionId: string): void {
    const connection = this.connections.get(connectionId);
    if (connection) {
      connection.lastPing = Date.now();
    }
  }

  private startHeartbeat(): void {
    this.heartbeatInterval = setInterval(() => {
      const now = Date.now();
      const staleConnections: string[] = [];

      this.connections.forEach((connection, connectionId) => {
        if (now - connection.lastPing > this.heartbeatIntervalMs * 2) {
          staleConnections.push(connectionId);
        } else {
          // Send ping
          if (connection.ws.readyState === WebSocket.OPEN) {
            connection.ws.ping();
          }
        }
      });

      // Clean up stale connections
      staleConnections.forEach(connectionId => {
        const connection = this.connections.get(connectionId);
        if (connection) {
          connection.ws.terminate();
          this.handleDisconnection(connectionId);
        }
      });
    }, this.heartbeatIntervalMs);
  }

  // Public methods for sending messages
  sendToConnection(connectionId: string, message: WebSocketMessage): boolean {
    const connection = this.connections.get(connectionId);
    if (!connection || connection.ws.readyState !== WebSocket.OPEN) {
      return false;
    }

    try {
      connection.ws.send(JSON.stringify(message));
      return true;
    } catch (error) {
      console.error(`Error sending message to connection ${connectionId}:`, error);
      return false;
    }
  }

  sendToUser(userId: string, message: WebSocketMessage): number {
    const userConnections = this.userConnections.get(userId);
    if (!userConnections) return 0;

    let sentCount = 0;
    userConnections.forEach(connectionId => {
      if (this.sendToConnection(connectionId, { ...message, userId })) {
        sentCount++;
      }
    });

    return sentCount;
  }

  sendToTenant(tenantId: string, message: WebSocketMessage): number {
    const tenantConnections = this.tenantConnections.get(tenantId);
    if (!tenantConnections) return 0;

    let sentCount = 0;
    tenantConnections.forEach(connectionId => {
      if (this.sendToConnection(connectionId, { ...message, tenantId })) {
        sentCount++;
      }
    });

    return sentCount;
  }

  sendToChannel(channel: string, message: WebSocketMessage): number {
    let sentCount = 0;
    
    this.connections.forEach((connection, connectionId) => {
      if (connection.subscriptions.has(channel)) {
        if (this.sendToConnection(connectionId, message)) {
          sentCount++;
        }
      }
    });

    return sentCount;
  }

  broadcast(message: WebSocketMessage): number {
    let sentCount = 0;
    
    this.connections.forEach((connection, connectionId) => {
      if (this.sendToConnection(connectionId, message)) {
        sentCount++;
      }
    });

    return sentCount;
  }

  // Auth-specific notification methods
  notifyAuthStatusUpdate(update: AuthStatusUpdate): void {
    const message: WebSocketMessage = {
      type: 'auth_status_update',
      data: update,
      timestamp: new Date().toISOString(),
    };

    // Send to specific user
    this.sendToUser(update.userId, message);

    // If tenant-specific, also send to tenant
    if (update.tenantId) {
      this.sendToTenant(update.tenantId, message);
    }
  }

  notifySessionExpired(userId: string, sessionId: string): void {
    this.notifyAuthStatusUpdate({
      type: 'session_expired',
      userId,
      sessionId,
    });
  }

  notifyTenantSwitched(userId: string, oldTenantId: string, newTenantId: string): void {
    this.notifyAuthStatusUpdate({
      type: 'tenant_switched',
      userId,
      tenantId: newTenantId,
      data: { oldTenantId, newTenantId },
    });
  }

  notifyProfileUpdated(userId: string, tenantId: string, changes: any): void {
    this.notifyAuthStatusUpdate({
      type: 'profile_updated',
      userId,
      tenantId,
      data: changes,
    });
  }

  // Statistics and monitoring
  getConnectionStats(): {
    totalConnections: number;
    userConnections: number;
    tenantConnections: number;
    connectionsByTenant: Record<string, number>;
  } {
    const connectionsByTenant: Record<string, number> = {};
    
    this.tenantConnections.forEach((connections, tenantId) => {
      connectionsByTenant[tenantId] = connections.size;
    });

    return {
      totalConnections: this.connections.size,
      userConnections: this.userConnections.size,
      tenantConnections: this.tenantConnections.size,
      connectionsByTenant,
    };
  }

  // Cleanup
  close(): void {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
    }

    this.connections.forEach((connection) => {
      connection.ws.close(1001, 'Server shutting down');
    });

    this.wss.close();
    console.log('WebSocket service closed');
  }
}