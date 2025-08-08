import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { GitHubClient } from '../GitHubClient';
import { GitHubConfig } from '../../config/types';

// Create mock functions
const mockCreate = vi.fn();
const mockUpdate = vi.fn();
const mockListForRepo = vi.fn();
const mockGetRateLimit = vi.fn();
const mockGetAuthenticated = vi.fn();
const mockGetRepo = vi.fn();

// Mock the Octokit module
vi.mock('@octokit/rest', () => {
  return {
    Octokit: vi.fn(() => ({
      rest: {
        issues: {
          create: mockCreate,
          update: mockUpdate,
          listForRepo: mockListForRepo,
        },
        rateLimit: {
          get: mockGetRateLimit,
        },
        users: {
          getAuthenticated: mockGetAuthenticated,
        },
        repos: {
          get: mockGetRepo,
        },
      },
    })),
  };
});

// Mock the logger
vi.mock('../../utils/logger', () => ({
  logger: {
    debug: vi.fn(),
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}));

describe('GitHubClient', () => {
  let client: GitHubClient;
  let config: GitHubConfig;

  beforeEach(() => {
    config = {
      enabled: true,
      token: 'test-token',
      repository: 'owner/repo',
      labelPrefix: 'kiro:',
      syncOnSave: true,
      apiUrl: 'https://api.github.com',
      maxRetries: 3,
      retryDelay: 100, // Reduced for faster tests
      rateLimitBuffer: 100,
    };

    // Setup default rate limit response
    mockGetRateLimit.mockResolvedValue({
      data: {
        rate: {
          limit: 5000,
          remaining: 4000,
          reset: Math.floor(Date.now() / 1000) + 3600,
          used: 1000,
        },
      },
    });

    client = new GitHubClient(config);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('constructor', () => {
    it('should initialize with valid config', () => {
      expect(() => new GitHubClient(config)).not.toThrow();
    });

    it('should throw error for invalid repository format', () => {
      const invalidConfig = { ...config, repository: 'invalid-repo' };
      expect(() => new GitHubClient(invalidConfig)).toThrow('Invalid repository format');
    });
  });

  describe('createIssue', () => {
    it('should create a new issue successfully', async () => {
      const mockIssueData = {
        id: 123,
        number: 1,
        title: 'Test Issue',
        body: 'Test body',
        state: 'open',
        labels: [{ name: 'kiro:1.1', color: 'blue' }],
        html_url: 'https://github.com/owner/repo/issues/1',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      };

      mockCreate.mockResolvedValue({ data: mockIssueData });

      const result = await client.createIssue('Test Issue', 'Test body', ['kiro:1.1']);

      expect(mockCreate).toHaveBeenCalledWith({
        owner: 'owner',
        repo: 'repo',
        title: 'Test Issue',
        body: 'Test body',
        labels: ['kiro:1.1'],
      });

      expect(result).toEqual({
        id: 123,
        number: 1,
        title: 'Test Issue',
        body: 'Test body',
        state: 'open',
        labels: [{ name: 'kiro:1.1', color: 'blue' }],
        html_url: 'https://github.com/owner/repo/issues/1',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      });
    });

    it('should handle API errors gracefully', async () => {
      mockCreate.mockRejectedValue({
        status: 401,
        message: 'Bad credentials',
      });

      await expect(client.createIssue('Test', 'Body')).rejects.toThrow(
        'GitHub authentication failed'
      );
    });

    it('should retry on transient errors', async () => {
      mockCreate
        .mockRejectedValueOnce({ status: 500, message: 'Internal server error' })
        .mockResolvedValue({
          data: {
            id: 123,
            number: 1,
            title: 'Test',
            body: 'Body',
            state: 'open',
            labels: [],
            html_url: 'https://github.com/owner/repo/issues/1',
            created_at: '2024-01-01T00:00:00Z',
            updated_at: '2024-01-01T00:00:00Z',
          },
        });

      const result = await client.createIssue('Test', 'Body');
      expect(mockCreate).toHaveBeenCalledTimes(2);
      expect(result.number).toBe(1);
    });
  });

  describe('updateIssue', () => {
    it('should update an existing issue successfully', async () => {
      const mockIssueData = {
        id: 123,
        number: 1,
        title: 'Updated Issue',
        body: 'Updated body',
        state: 'open',
        labels: [],
        html_url: 'https://github.com/owner/repo/issues/1',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T01:00:00Z',
      };

      mockUpdate.mockResolvedValue({ data: mockIssueData });

      const result = await client.updateIssue(1, 'Updated Issue', 'Updated body');

      expect(mockUpdate).toHaveBeenCalledWith({
        owner: 'owner',
        repo: 'repo',
        issue_number: 1,
        title: 'Updated Issue',
        body: 'Updated body',
      });

      expect(result.title).toBe('Updated Issue');
      expect(result.body).toBe('Updated body');
    });
  });

  describe('closeIssue', () => {
    it('should close an issue successfully', async () => {
      const mockIssueData = {
        id: 123,
        number: 1,
        title: 'Test Issue',
        body: 'Test body',
        state: 'closed',
        labels: [],
        html_url: 'https://github.com/owner/repo/issues/1',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T01:00:00Z',
      };

      mockUpdate.mockResolvedValue({ data: mockIssueData });

      const result = await client.closeIssue(1);

      expect(mockUpdate).toHaveBeenCalledWith({
        owner: 'owner',
        repo: 'repo',
        issue_number: 1,
        state: 'closed',
      });

      expect(result.state).toBe('closed');
    });
  });

  describe('findIssueByLabel', () => {
    it('should find an issue by label', async () => {
      const mockIssueData = {
        id: 123,
        number: 1,
        title: 'Test Issue',
        body: 'Test body',
        state: 'open',
        labels: [{ name: 'kiro:1.1', color: 'blue' }],
        html_url: 'https://github.com/owner/repo/issues/1',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
      };

      mockListForRepo.mockResolvedValue({
        data: [mockIssueData],
      });

      const result = await client.findIssueByLabel('kiro:1.1');

      expect(mockListForRepo).toHaveBeenCalledWith({
        owner: 'owner',
        repo: 'repo',
        labels: 'kiro:1.1',
        state: 'all',
        per_page: 1,
      });

      expect(result).not.toBeNull();
      expect(result!.number).toBe(1);
      expect(result!.labels).toHaveLength(1);
      expect(result!.labels[0]?.name).toBe('kiro:1.1');
    });

    it('should return null when no issue is found', async () => {
      mockListForRepo.mockResolvedValue({
        data: [],
      });

      const result = await client.findIssueByLabel('kiro:nonexistent');

      expect(result).toBeNull();
    });
  });

  describe('rate limiting', () => {
    it('should check rate limit before API calls', async () => {
      mockCreate.mockResolvedValue({
        data: {
          id: 123,
          number: 1,
          title: 'Test',
          body: 'Body',
          state: 'open',
          labels: [],
          html_url: 'https://github.com/owner/repo/issues/1',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      });

      await client.createIssue('Test', 'Body');

      expect(mockGetRateLimit).toHaveBeenCalled();
    });

    it('should wait when rate limit is low', async () => {
      // Mock low rate limit
      mockGetRateLimit.mockResolvedValue({
        data: {
          rate: {
            limit: 5000,
            remaining: 50, // Below buffer of 100
            reset: Math.floor(Date.now() / 1000) + 1, // Reset in 1 second
            used: 4950,
          },
        },
      });

      mockCreate.mockResolvedValue({
        data: {
          id: 123,
          number: 1,
          title: 'Test',
          body: 'Body',
          state: 'open',
          labels: [],
          html_url: 'https://github.com/owner/repo/issues/1',
          created_at: '2024-01-01T00:00:00Z',
          updated_at: '2024-01-01T00:00:00Z',
        },
      });

      const startTime = Date.now();
      await client.createIssue('Test', 'Body');
      const endTime = Date.now();

      // Should have waited at least 1 second
      expect(endTime - startTime).toBeGreaterThan(900);
    });
  });

  describe('testConnection', () => {
    it('should test connection successfully', async () => {
      mockGetAuthenticated.mockResolvedValue({
        data: { login: 'testuser' },
      });

      mockGetRepo.mockResolvedValue({
        data: { name: 'repo' },
      });

      mockListForRepo.mockResolvedValue({
        data: [],
      });

      const result = await client.testConnection();

      expect(result.success).toBe(true);
      expect(result.message).toContain('Successfully connected');
      expect(result.message).toContain('testuser');
    });

    it('should handle connection failure', async () => {
      mockGetAuthenticated.mockRejectedValue({
        status: 401,
        message: 'Bad credentials',
      });

      const result = await client.testConnection();

      expect(result.success).toBe(false);
      expect(result.message).toContain('authentication failed');
    });
  });

  describe('error handling', () => {
    it('should handle 404 errors appropriately', async () => {
      mockCreate.mockRejectedValue({
        status: 404,
        message: 'Not Found',
      });

      await expect(client.createIssue('Test', 'Body')).rejects.toThrow(
        'repository or resource not found'
      );
    });

    it('should handle 422 validation errors', async () => {
      mockCreate.mockRejectedValue({
        status: 422,
        message: 'Validation Failed',
      });

      await expect(client.createIssue('Test', 'Body')).rejects.toThrow(
        'validation error'
      );
    });

    it('should not retry on 4xx errors (except 429)', async () => {
      mockCreate.mockRejectedValue({
        status: 400,
        message: 'Bad Request',
      });

      await expect(client.createIssue('Test', 'Body')).rejects.toThrow();
      
      // Should only be called once (no retries)
      expect(mockCreate).toHaveBeenCalledTimes(1);
    });
  });
});