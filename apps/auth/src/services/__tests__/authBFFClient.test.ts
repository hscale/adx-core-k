import { describe, it, expect, vi, beforeEach } from 'vitest';
import { AuthBFFClient } from '../authBFFClient';
import type { LoginRequest, RegisterRequest } from '../../types';

// Mock fetch globally
global.fetch = vi.fn();

describe('AuthBFFClient', () => {
  let client: AuthBFFClient;

  beforeEach(() => {
    client = new AuthBFFClient({
      baseUrl: 'http://localhost:4001',
      timeout: 5000,
    });
    vi.clearAllMocks();
  });

  describe('login', () => {
    it('should make a login request to the BFF service', async () => {
      const mockResponse = {
        type: 'sync',
        data: {
          user: {
            id: '1',
            email: 'test@example.com',
            name: 'Test User',
            roles: ['user'],
            permissions: ['read'],
          },
          token: 'mock-token',
          refreshToken: 'mock-refresh-token',
          expiresAt: '2024-01-01T00:00:00Z',
        },
      };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      const loginRequest: LoginRequest = {
        email: 'test@example.com',
        password: 'password123',
      };

      const result = await client.login(loginRequest);

      expect(fetch).toHaveBeenCalledWith(
        'http://localhost:4001/workflows/user-login',
        expect.objectContaining({
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            ...loginRequest,
            synchronous: true,
          }),
        })
      );

      expect(result).toEqual(mockResponse);
    });

    it('should handle login errors', async () => {
      (fetch as any).mockResolvedValueOnce({
        ok: false,
        status: 401,
        statusText: 'Unauthorized',
        json: () => Promise.resolve({
          error: { message: 'Invalid credentials' },
        }),
      });

      const loginRequest: LoginRequest = {
        email: 'test@example.com',
        password: 'wrong-password',
      };

      await expect(client.login(loginRequest)).rejects.toThrow('Invalid credentials');
    });
  });

  describe('register', () => {
    it('should make a registration request to the BFF service', async () => {
      const mockResponse = {
        type: 'async',
        operationId: 'op-123',
        statusUrl: '/workflows/op-123/status',
      };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      const registerRequest: RegisterRequest = {
        email: 'test@example.com',
        password: 'password123',
        confirmPassword: 'password123',
        firstName: 'Test',
        lastName: 'User',
        acceptTerms: true,
      };

      const result = await client.register(registerRequest);

      expect(fetch).toHaveBeenCalledWith(
        'http://localhost:4001/workflows/user-registration',
        expect.objectContaining({
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(registerRequest),
        })
      );

      expect(result).toEqual(mockResponse);
    });
  });

  describe('workflow status polling', () => {
    it('should poll workflow status until completion', async () => {
      const mockStatusResponses = [
        { status: 'running', progress: { percentage: 50 } },
        { status: 'running', progress: { percentage: 75 } },
        { status: 'completed', result: { success: true } },
      ];

      let callCount = 0;
      (fetch as any).mockImplementation(() => {
        const response = mockStatusResponses[callCount++];
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(response),
        });
      });

      const result = await client.pollWorkflowStatus('op-123');

      expect(result).toEqual({ success: true });
      expect(fetch).toHaveBeenCalledTimes(3);
    });

    it('should reject on workflow failure', async () => {
      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve({
          status: 'failed',
          error: 'Workflow execution failed',
        }),
      });

      await expect(client.pollWorkflowStatus('op-123')).rejects.toThrow('Workflow execution failed');
    });
  });

  describe('health check', () => {
    it('should check BFF service health', async () => {
      const mockResponse = {
        status: 'healthy',
        timestamp: '2024-01-01T00:00:00Z',
      };

      (fetch as any).mockResolvedValueOnce({
        ok: true,
        json: () => Promise.resolve(mockResponse),
      });

      const result = await client.healthCheck();

      expect(fetch).toHaveBeenCalledWith(
        'http://localhost:4001/health',
        expect.objectContaining({
          headers: {
            'Content-Type': 'application/json',
          },
        })
      );

      expect(result).toEqual(mockResponse);
    });
  });
});