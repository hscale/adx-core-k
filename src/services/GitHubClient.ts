import { Octokit } from '@octokit/rest';
import { GitHubConfig, GitHubIssue } from '../config/types';
import { logger } from '../utils/logger';

/**
 * Rate limiting information from GitHub API
 */
interface RateLimitInfo {
  limit: number;
  remaining: number;
  reset: number;
  used: number;
}

/**
 * GitHub API client with rate limiting and error handling
 */
export class GitHubClient {
  private octokit: Octokit;
  private owner: string;
  private repo: string;
  private config: GitHubConfig;
  private lastRateLimitCheck: number = 0;
  private rateLimitInfo: RateLimitInfo | null = null;

  constructor(config: GitHubConfig) {
    this.config = config;
    
    // Parse repository owner/repo from config
    const [owner, repo] = config.repository.split('/');
    if (!owner || !repo) {
      throw new Error(`Invalid repository format: ${config.repository}. Expected format: "owner/repo"`);
    }
    
    this.owner = owner;
    this.repo = repo;
    
    // Initialize Octokit with configuration
    this.octokit = new Octokit({
      auth: config.token,
      baseUrl: config.apiUrl,
      userAgent: 'kiro-github-task-sync/1.0.0',
      request: {
        timeout: 10000, // 10 second timeout
      },
    });

    logger.debug('GitHubClient initialized', { 
      repository: config.repository,
      apiUrl: config.apiUrl 
    });
  }

  /**
   * Create a new GitHub issue
   */
  async createIssue(title: string, body: string, labels: string[] = []): Promise<GitHubIssue> {
    await this.checkRateLimit();
    
    try {
      logger.debug('Creating GitHub issue', { title, labels });
      
      const response = await this.withRetry(async () => {
        return await this.octokit.rest.issues.create({
          owner: this.owner,
          repo: this.repo,
          title,
          body,
          labels,
        });
      });

      const issue = this.mapToGitHubIssue(response.data);
      logger.info('Created GitHub issue', { 
        number: issue.number, 
        title: issue.title,
        url: issue.html_url 
      });
      
      return issue;
    } catch (error) {
      logger.error('Failed to create GitHub issue', { title, error: this.formatError(error) });
      throw this.handleApiError(error, 'create issue');
    }
  }

  /**
   * Update an existing GitHub issue
   */
  async updateIssue(issueNumber: number, title: string, body: string): Promise<GitHubIssue> {
    await this.checkRateLimit();
    
    try {
      logger.debug('Updating GitHub issue', { issueNumber, title });
      
      const response = await this.withRetry(async () => {
        return await this.octokit.rest.issues.update({
          owner: this.owner,
          repo: this.repo,
          issue_number: issueNumber,
          title,
          body,
        });
      });

      const issue = this.mapToGitHubIssue(response.data);
      logger.info('Updated GitHub issue', { 
        number: issue.number, 
        title: issue.title,
        url: issue.html_url 
      });
      
      return issue;
    } catch (error) {
      logger.error('Failed to update GitHub issue', { 
        issueNumber, 
        title, 
        error: this.formatError(error) 
      });
      throw this.handleApiError(error, 'update issue');
    }
  }

  /**
   * Close a GitHub issue
   */
  async closeIssue(issueNumber: number): Promise<GitHubIssue> {
    await this.checkRateLimit();
    
    try {
      logger.debug('Closing GitHub issue', { issueNumber });
      
      const response = await this.withRetry(async () => {
        return await this.octokit.rest.issues.update({
          owner: this.owner,
          repo: this.repo,
          issue_number: issueNumber,
          state: 'closed',
        });
      });

      const issue = this.mapToGitHubIssue(response.data);
      logger.info('Closed GitHub issue', { 
        number: issue.number, 
        title: issue.title,
        url: issue.html_url 
      });
      
      return issue;
    } catch (error) {
      logger.error('Failed to close GitHub issue', { 
        issueNumber, 
        error: this.formatError(error) 
      });
      throw this.handleApiError(error, 'close issue');
    }
  }

  /**
   * Find an issue by label (typically Kiro task ID)
   */
  async findIssueByLabel(label: string): Promise<GitHubIssue | null> {
    await this.checkRateLimit();
    
    try {
      logger.debug('Finding GitHub issue by label', { label });
      
      const response = await this.withRetry(async () => {
        return await this.octokit.rest.issues.listForRepo({
          owner: this.owner,
          repo: this.repo,
          labels: label,
          state: 'all', // Search both open and closed issues
          per_page: 1, // We only expect one issue per Kiro task
        });
      });

      if (response.data.length === 0) {
        logger.debug('No issue found with label', { label });
        return null;
      }

      const issue = this.mapToGitHubIssue(response.data[0]);
      logger.debug('Found issue by label', { 
        label, 
        issueNumber: issue.number, 
        title: issue.title 
      });
      
      return issue;
    } catch (error) {
      logger.error('Failed to find GitHub issue by label', { 
        label, 
        error: this.formatError(error) 
      });
      throw this.handleApiError(error, 'find issue by label');
    }
  }

  /**
   * Get current rate limit information
   */
  async getRateLimit(): Promise<RateLimitInfo> {
    try {
      const response = await this.octokit.rest.rateLimit.get();
      const rateLimit = response.data.rate;
      
      this.rateLimitInfo = {
        limit: rateLimit.limit,
        remaining: rateLimit.remaining,
        reset: rateLimit.reset,
        used: rateLimit.used,
      };
      
      this.lastRateLimitCheck = Date.now();
      return this.rateLimitInfo;
    } catch (error) {
      logger.warn('Failed to get rate limit info', { error: this.formatError(error) });
      // Return a conservative estimate if we can't get rate limit info
      return {
        limit: 5000,
        remaining: 1000,
        reset: Math.floor(Date.now() / 1000) + 3600,
        used: 4000,
      };
    }
  }

  /**
   * Check rate limit and wait if necessary
   */
  private async checkRateLimit(): Promise<void> {
    // Check rate limit every 5 minutes or if we don't have info
    const shouldCheck = !this.rateLimitInfo || 
      (Date.now() - this.lastRateLimitCheck) > 5 * 60 * 1000;
    
    if (shouldCheck) {
      await this.getRateLimit();
    }

    if (this.rateLimitInfo && this.rateLimitInfo.remaining <= this.config.rateLimitBuffer) {
      const resetTime = this.rateLimitInfo.reset * 1000;
      const waitTime = resetTime - Date.now() + 1000; // Add 1 second buffer
      
      if (waitTime > 0) {
        logger.warn('Rate limit approaching, waiting for reset', {
          remaining: this.rateLimitInfo.remaining,
          resetTime: new Date(resetTime).toISOString(),
          waitTimeMs: waitTime,
        });
        
        await this.sleep(waitTime);
        await this.getRateLimit(); // Refresh after waiting
      }
    }
  }

  /**
   * Execute a function with retry logic
   */
  private async withRetry<T>(fn: () => Promise<T>): Promise<T> {
    let lastError: Error;
    
    for (let attempt = 1; attempt <= this.config.maxRetries + 1; attempt++) {
      try {
        return await fn();
      } catch (error) {
        lastError = error as Error;
        
        // Don't retry on certain errors
        if (this.isNonRetryableError(error)) {
          throw error;
        }
        
        if (attempt <= this.config.maxRetries) {
          const delay = this.config.retryDelay * Math.pow(2, attempt - 1); // Exponential backoff
          logger.warn('API call failed, retrying', {
            attempt,
            maxRetries: this.config.maxRetries,
            delayMs: delay,
            error: this.formatError(error),
          });
          
          await this.sleep(delay);
        }
      }
    }
    
    throw lastError!;
  }

  /**
   * Check if an error should not be retried
   */
  private isNonRetryableError(error: unknown): boolean {
    if (typeof error === 'object' && error !== null && 'status' in error) {
      const status = (error as { status: number }).status;
      // Don't retry on client errors (4xx) except rate limiting (429)
      return status >= 400 && status < 500 && status !== 429;
    }
    return false;
  }

  /**
   * Handle API errors and convert to meaningful error messages
   */
  private handleApiError(error: unknown, operation: string): Error {
    if (typeof error === 'object' && error !== null) {
      if ('status' in error) {
        const status = (error as { status: number }).status;
        const message = 'message' in error ? (error as { message: string }).message : 'Unknown error';
        
        switch (status) {
          case 401:
            return new Error(`GitHub authentication failed. Please check your token. Operation: ${operation}`);
          case 403:
            if (message.includes('rate limit')) {
              return new Error(`GitHub rate limit exceeded. Operation: ${operation}`);
            }
            return new Error(`GitHub access forbidden. Check repository permissions. Operation: ${operation}`);
          case 404:
            return new Error(`GitHub repository or resource not found. Operation: ${operation}`);
          case 422:
            return new Error(`GitHub API validation error: ${message}. Operation: ${operation}`);
          default:
            return new Error(`GitHub API error (${status}): ${message}. Operation: ${operation}`);
        }
      }
    }
    
    return new Error(`Unexpected error during ${operation}: ${this.formatError(error)}`);
  }

  /**
   * Format error for logging
   */
  private formatError(error: unknown): string {
    if (error instanceof Error) {
      return error.message;
    }
    if (typeof error === 'object' && error !== null) {
      return JSON.stringify(error);
    }
    return String(error);
  }

  /**
   * Map GitHub API response to our GitHubIssue interface
   */
  private mapToGitHubIssue(issue: any): GitHubIssue {
    return {
      id: issue.id,
      number: issue.number,
      title: issue.title,
      body: issue.body || '',
      state: issue.state,
      labels: issue.labels.map((label: any) => ({
        name: typeof label === 'string' ? label : label.name,
        color: typeof label === 'string' ? '' : (label.color || ''),
      })),
      html_url: issue.html_url,
      created_at: issue.created_at,
      updated_at: issue.updated_at,
    };
  }

  /**
   * Sleep for specified milliseconds
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  /**
   * Test the GitHub connection and permissions
   */
  async testConnection(): Promise<{ success: boolean; message: string }> {
    try {
      // Test authentication by getting user info
      const userResponse = await this.octokit.rest.users.getAuthenticated();
      logger.debug('GitHub authentication successful', { user: userResponse.data.login });
      
      // Test repository access by getting repo info
      await this.octokit.rest.repos.get({
        owner: this.owner,
        repo: this.repo,
      });
      
      // Check if we have issues permission by trying to list issues
      await this.octokit.rest.issues.listForRepo({
        owner: this.owner,
        repo: this.repo,
        per_page: 1,
      });
      
      return {
        success: true,
        message: `Successfully connected to ${this.config.repository} as ${userResponse.data.login}`,
      };
    } catch (error) {
      const errorMessage = this.handleApiError(error, 'test connection').message;
      return {
        success: false,
        message: errorMessage,
      };
    }
  }
}