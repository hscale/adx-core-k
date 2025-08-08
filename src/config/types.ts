import { z } from 'zod';

/**
 * GitHub sync configuration schema
 */
export const GitHubConfigSchema = z.object({
  enabled: z.boolean().default(true),
  token: z.string().min(1, 'GitHub token is required'),
  repository: z.string().regex(/^[^/]+\/[^/]+$/, 'Repository must be in format "owner/repo"'),
  labelPrefix: z.string().default('kiro:'),
  syncOnSave: z.boolean().default(true),
  apiUrl: z.string().url().default('https://api.github.com'),
  maxRetries: z.number().int().min(0).default(3),
  retryDelay: z.number().int().min(100).default(1000),
  rateLimitBuffer: z.number().int().min(0).default(100),
});

export type GitHubConfig = z.infer<typeof GitHubConfigSchema>;

/**
 * Setup options for initial configuration
 */
export const SetupOptionsSchema = z.object({
  token: z.string().optional(),
  repo: z.string().optional(),
  labelPrefix: z.string().optional(),
  syncOnSave: z.boolean().optional(),
});

export type SetupOptions = z.infer<typeof SetupOptionsSchema>;

/**
 * Sync state for tracking task-to-issue mappings
 */
export const SyncStateSchema = z.object({
  taskId: z.string(),
  githubIssueNumber: z.number(),
  lastSynced: z.string().datetime(),
  lastHash: z.string(),
  filePath: z.string(),
});

export type SyncState = z.infer<typeof SyncStateSchema>;

/**
 * Sync status information
 */
export interface SyncStatus {
  lastSync: string | null;
  syncedTasks: number;
  errors: string[];
  isWatching: boolean;
}

/**
 * Task information extracted from markdown files
 */
export interface KiroTask {
  id: string;
  title: string;
  description?: string;
  status: 'not_started' | 'in_progress' | 'completed';
  filePath: string;
  lineNumber: number;
  specName: string;
  requirements?: string[];
}

/**
 * GitHub issue information
 */
export interface GitHubIssue {
  id: number;
  number: number;
  title: string;
  body: string;
  state: 'open' | 'closed';
  labels: Array<{
    name: string;
    color: string;
  }>;
  html_url: string;
  created_at: string;
  updated_at: string;
}