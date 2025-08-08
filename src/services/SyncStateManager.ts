import { promises as fs } from 'fs';
import { join, dirname } from 'path';
import { SyncState, KiroTask } from '../config/types';
import { logger } from '../utils/logger';
import { createHash } from 'crypto';

/**
 * Manages sync state for tracking task-to-issue mappings
 */
export class SyncStateManager {
  private readonly stateFilePath: string;
  private syncState: Map<string, SyncState> = new Map();
  private loaded: boolean = false;

  constructor(stateFilePath?: string) {
    this.stateFilePath = stateFilePath || join(process.cwd(), '.kiro', '.github-sync-state.json');
  }

  /**
   * Load sync state from file
   */
  async load(): Promise<void> {
    try {
      logger.debug('Loading sync state', { filePath: this.stateFilePath });
      
      const data = await fs.readFile(this.stateFilePath, 'utf-8');
      const stateArray: SyncState[] = JSON.parse(data);
      
      // Convert array to Map for efficient lookups
      this.syncState = new Map(
        stateArray.map(state => [state.taskId, state])
      );
      
      this.loaded = true;
      
      logger.info('Loaded sync state', { 
        stateCount: this.syncState.size,
        filePath: this.stateFilePath 
      });
    } catch (error) {
      if (this.isFileNotFoundError(error)) {
        logger.info('No existing sync state file found, starting fresh');
        this.syncState = new Map();
        this.loaded = true;
      } else {
        logger.error('Failed to load sync state', { 
          filePath: this.stateFilePath, 
          error: this.formatError(error) 
        });
        throw new Error(`Failed to load sync state: ${this.formatError(error)}`);
      }
    }
  }

  /**
   * Save sync state to file
   */
  async save(): Promise<void> {
    try {
      // Ensure directory exists
      await fs.mkdir(dirname(this.stateFilePath), { recursive: true });
      
      // Convert Map to array for JSON serialization
      const stateArray = Array.from(this.syncState.values());
      
      const data = JSON.stringify(stateArray, null, 2);
      await fs.writeFile(this.stateFilePath, data, 'utf-8');
      
      logger.debug('Saved sync state', { 
        stateCount: this.syncState.size,
        filePath: this.stateFilePath 
      });
    } catch (error) {
      logger.error('Failed to save sync state', { 
        filePath: this.stateFilePath, 
        error: this.formatError(error) 
      });
      throw new Error(`Failed to save sync state: ${this.formatError(error)}`);
    }
  }

  /**
   * Get sync state for a task
   */
  getTaskState(taskId: string): SyncState | null {
    this.ensureLoaded();
    return this.syncState.get(taskId) || null;
  }

  /**
   * Set sync state for a task
   */
  setTaskState(taskId: string, state: SyncState): void {
    this.ensureLoaded();
    this.syncState.set(taskId, state);
    
    logger.debug('Updated task state', { 
      taskId, 
      githubIssueNumber: state.githubIssueNumber,
      lastSynced: state.lastSynced 
    });
  }

  /**
   * Update sync state for a task after successful sync
   */
  updateTaskState(
    taskId: string, 
    githubIssueNumber: number, 
    taskHash: string,
    filePath: string
  ): void {
    const state: SyncState = {
      taskId,
      githubIssueNumber,
      lastSynced: new Date().toISOString(),
      lastHash: taskHash,
      filePath,
    };
    
    this.setTaskState(taskId, state);
  }

  /**
   * Remove sync state for a task
   */
  removeTaskState(taskId: string): boolean {
    this.ensureLoaded();
    const removed = this.syncState.delete(taskId);
    
    if (removed) {
      logger.debug('Removed task state', { taskId });
    }
    
    return removed;
  }

  /**
   * Check if a task needs syncing based on content hash
   */
  needsSync(task: KiroTask, currentHash: string): boolean {
    this.ensureLoaded();
    
    const existingState = this.syncState.get(task.id);
    
    // If no existing state, needs sync
    if (!existingState) {
      logger.debug('Task needs sync: no existing state', { taskId: task.id });
      return true;
    }
    
    // If hash changed, needs sync
    if (existingState.lastHash !== currentHash) {
      logger.debug('Task needs sync: hash changed', { 
        taskId: task.id,
        oldHash: existingState.lastHash,
        newHash: currentHash 
      });
      return true;
    }
    
    // If file path changed, needs sync (task moved)
    if (existingState.filePath !== task.filePath) {
      logger.debug('Task needs sync: file path changed', { 
        taskId: task.id,
        oldPath: existingState.filePath,
        newPath: task.filePath 
      });
      return true;
    }
    
    logger.debug('Task does not need sync', { taskId: task.id });
    return false;
  }

  /**
   * Generate content hash for a task
   */
  generateTaskHash(task: KiroTask): string {
    const content = JSON.stringify({
      title: task.title,
      description: task.description,
      status: task.status,
      requirements: task.requirements,
    });
    
    return createHash('md5').update(content).digest('hex');
  }

  /**
   * Get all sync states
   */
  getAllStates(): SyncState[] {
    this.ensureLoaded();
    return Array.from(this.syncState.values());
  }

  /**
   * Get sync states for a specific file
   */
  getStatesForFile(filePath: string): SyncState[] {
    this.ensureLoaded();
    return Array.from(this.syncState.values())
      .filter(state => state.filePath === filePath);
  }

  /**
   * Clean up orphaned sync states (tasks that no longer exist)
   */
  cleanupOrphanedStates(existingTaskIds: string[]): string[] {
    this.ensureLoaded();
    
    const existingIds = new Set(existingTaskIds);
    const orphanedIds: string[] = [];
    
    for (const [taskId] of this.syncState) {
      if (!existingIds.has(taskId)) {
        orphanedIds.push(taskId);
        this.syncState.delete(taskId);
      }
    }
    
    if (orphanedIds.length > 0) {
      logger.info('Cleaned up orphaned sync states', { 
        orphanedCount: orphanedIds.length,
        orphanedIds 
      });
    }
    
    return orphanedIds;
  }

  /**
   * Get statistics about sync state
   */
  getStats(): {
    totalTasks: number;
    syncedTasks: number;
    lastSyncTime: string | null;
    fileCount: number;
  } {
    this.ensureLoaded();
    
    const states = Array.from(this.syncState.values());
    const filePaths = new Set(states.map(state => state.filePath));
    
    let lastSyncTime: string | null = null;
    if (states.length > 0) {
      lastSyncTime = states
        .map(state => state.lastSynced)
        .sort()
        .reverse()[0];
    }
    
    return {
      totalTasks: states.length,
      syncedTasks: states.length, // All states represent synced tasks
      lastSyncTime,
      fileCount: filePaths.size,
    };
  }

  /**
   * Export sync state for backup or migration
   */
  async exportState(): Promise<string> {
    this.ensureLoaded();
    
    const exportData = {
      version: '1.0.0',
      exportedAt: new Date().toISOString(),
      states: Array.from(this.syncState.values()),
    };
    
    return JSON.stringify(exportData, null, 2);
  }

  /**
   * Import sync state from backup
   */
  async importState(data: string): Promise<void> {
    try {
      const importData = JSON.parse(data);
      
      if (!importData.states || !Array.isArray(importData.states)) {
        throw new Error('Invalid import data format');
      }
      
      // Validate each state
      for (const state of importData.states) {
        if (!this.isValidSyncState(state)) {
          throw new Error(`Invalid sync state: ${JSON.stringify(state)}`);
        }
      }
      
      // Clear existing state and import new data
      this.syncState = new Map(
        importData.states.map((state: SyncState) => [state.taskId, state])
      );
      
      this.loaded = true;
      
      logger.info('Imported sync state', { 
        stateCount: this.syncState.size,
        importVersion: importData.version 
      });
    } catch (error) {
      logger.error('Failed to import sync state', { error: this.formatError(error) });
      throw new Error(`Failed to import sync state: ${this.formatError(error)}`);
    }
  }

  /**
   * Clear all sync state
   */
  clear(): void {
    this.syncState.clear();
    this.loaded = true;
    logger.info('Cleared all sync state');
  }

  /**
   * Get the file path where sync state is stored
   */
  getStateFilePath(): string {
    return this.stateFilePath;
  }

  /**
   * Check if sync state is loaded
   */
  isLoaded(): boolean {
    return this.loaded;
  }

  /**
   * Ensure sync state is loaded
   */
  private ensureLoaded(): void {
    if (!this.loaded) {
      throw new Error('Sync state not loaded. Call load() first.');
    }
  }

  /**
   * Validate sync state object
   */
  private isValidSyncState(state: any): state is SyncState {
    return (
      typeof state === 'object' &&
      typeof state.taskId === 'string' &&
      typeof state.githubIssueNumber === 'number' &&
      typeof state.lastSynced === 'string' &&
      typeof state.lastHash === 'string' &&
      typeof state.filePath === 'string'
    );
  }

  /**
   * Check if error is a file not found error
   */
  private isFileNotFoundError(error: unknown): boolean {
    return (
      (error instanceof Error && 'code' in error && error.code === 'ENOENT') ||
      (typeof error === 'object' && error !== null && 'code' in error && (error as any).code === 'ENOENT')
    );
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
}