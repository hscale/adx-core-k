import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { promises as fs } from 'fs';
import { SyncStateManager } from '../SyncStateManager';
import { SyncState, KiroTask } from '../../config/types';

// Mock fs module
vi.mock('fs', () => ({
  promises: {
    readFile: vi.fn(),
    writeFile: vi.fn(),
    mkdir: vi.fn(),
  },
}));

// Mock logger
vi.mock('../../utils/logger', () => ({
  logger: {
    debug: vi.fn(),
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
  },
}));

describe('SyncStateManager', () => {
  let manager: SyncStateManager;
  const mockReadFile = vi.mocked(fs.readFile);
  const mockWriteFile = vi.mocked(fs.writeFile);
  const mockMkdir = vi.mocked(fs.mkdir);

  beforeEach(() => {
    manager = new SyncStateManager('/test/.kiro/.github-sync-state.json');
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('load', () => {
    it('should load sync state from file', async () => {
      const mockStates: SyncState[] = [
        {
          taskId: '1.1',
          githubIssueNumber: 123,
          lastSynced: '2024-01-01T00:00:00Z',
          lastHash: 'hash123',
          filePath: 'test.md',
        },
        {
          taskId: '1.2',
          githubIssueNumber: 124,
          lastSynced: '2024-01-01T01:00:00Z',
          lastHash: 'hash124',
          filePath: 'test.md',
        },
      ];

      mockReadFile.mockResolvedValue(JSON.stringify(mockStates));

      await manager.load();

      expect(mockReadFile).toHaveBeenCalledWith('/test/.kiro/.github-sync-state.json', 'utf-8');
      expect(manager.getTaskState('1.1')).toEqual(mockStates[0]);
      expect(manager.getTaskState('1.2')).toEqual(mockStates[1]);
    });

    it('should handle missing state file', async () => {
      const error = new Error('File not found') as any;
      error.code = 'ENOENT';
      mockReadFile.mockRejectedValue(error);

      await manager.load();

      expect(manager.getTaskState('nonexistent')).toBeNull();
    });

    it('should handle invalid JSON', async () => {
      mockReadFile.mockResolvedValue('invalid json');

      await expect(manager.load()).rejects.toThrow('Failed to load sync state');
    });
  });

  describe('save', () => {
    it('should save sync state to file', async () => {
      mockMkdir.mockResolvedValue(undefined);
      mockWriteFile.mockResolvedValue(undefined);

      await manager.load(); // Initialize empty state
      
      manager.setTaskState('1.1', {
        taskId: '1.1',
        githubIssueNumber: 123,
        lastSynced: '2024-01-01T00:00:00Z',
        lastHash: 'hash123',
        filePath: 'test.md',
      });

      await manager.save();

      expect(mockMkdir).toHaveBeenCalledWith('/test/.kiro', { recursive: true });
      expect(mockWriteFile).toHaveBeenCalledWith(
        '/test/.kiro/.github-sync-state.json',
        expect.stringContaining('"taskId": "1.1"'),
        'utf-8'
      );
    });

    it('should handle save errors', async () => {
      mockMkdir.mockResolvedValue(undefined);
      mockWriteFile.mockRejectedValue(new Error('Permission denied'));

      await manager.load();

      await expect(manager.save()).rejects.toThrow('Failed to save sync state');
    });
  });

  describe('task state management', () => {
    beforeEach(async () => {
      mockReadFile.mockRejectedValue({ code: 'ENOENT' });
      await manager.load();
    });

    it('should get and set task state', () => {
      const state: SyncState = {
        taskId: '1.1',
        githubIssueNumber: 123,
        lastSynced: '2024-01-01T00:00:00Z',
        lastHash: 'hash123',
        filePath: 'test.md',
      };

      expect(manager.getTaskState('1.1')).toBeNull();

      manager.setTaskState('1.1', state);
      expect(manager.getTaskState('1.1')).toEqual(state);
    });

    it('should update task state', () => {
      manager.updateTaskState('1.1', 123, 'hash123', 'test.md');

      const state = manager.getTaskState('1.1');
      expect(state).not.toBeNull();
      expect(state!.taskId).toBe('1.1');
      expect(state!.githubIssueNumber).toBe(123);
      expect(state!.lastHash).toBe('hash123');
      expect(state!.filePath).toBe('test.md');
      expect(state!.lastSynced).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
    });

    it('should remove task state', () => {
      const state: SyncState = {
        taskId: '1.1',
        githubIssueNumber: 123,
        lastSynced: '2024-01-01T00:00:00Z',
        lastHash: 'hash123',
        filePath: 'test.md',
      };

      manager.setTaskState('1.1', state);
      expect(manager.getTaskState('1.1')).toEqual(state);

      const removed = manager.removeTaskState('1.1');
      expect(removed).toBe(true);
      expect(manager.getTaskState('1.1')).toBeNull();

      const removedAgain = manager.removeTaskState('1.1');
      expect(removedAgain).toBe(false);
    });
  });

  describe('needsSync', () => {
    beforeEach(async () => {
      mockReadFile.mockRejectedValue({ code: 'ENOENT' });
      await manager.load();
    });

    it('should return true for new task', () => {
      const task: KiroTask = {
        id: '1.1',
        title: 'Test Task',
        status: 'not_started',
        filePath: 'test.md',
        lineNumber: 1,
        specName: 'test',
      };

      expect(manager.needsSync(task, 'hash123')).toBe(true);
    });

    it('should return false for unchanged task', () => {
      const task: KiroTask = {
        id: '1.1',
        title: 'Test Task',
        status: 'not_started',
        filePath: 'test.md',
        lineNumber: 1,
        specName: 'test',
      };

      manager.updateTaskState('1.1', 123, 'hash123', 'test.md');
      expect(manager.needsSync(task, 'hash123')).toBe(false);
    });

    it('should return true for changed task content', () => {
      const task: KiroTask = {
        id: '1.1',
        title: 'Test Task',
        status: 'not_started',
        filePath: 'test.md',
        lineNumber: 1,
        specName: 'test',
      };

      manager.updateTaskState('1.1', 123, 'hash123', 'test.md');
      expect(manager.needsSync(task, 'hash456')).toBe(true);
    });

    it('should return true for moved task', () => {
      const task: KiroTask = {
        id: '1.1',
        title: 'Test Task',
        status: 'not_started',
        filePath: 'new-test.md',
        lineNumber: 1,
        specName: 'test',
      };

      manager.updateTaskState('1.1', 123, 'hash123', 'test.md');
      expect(manager.needsSync(task, 'hash123')).toBe(true);
    });
  });

  describe('generateTaskHash', () => {
    it('should generate consistent hash for same task content', () => {
      const task1: KiroTask = {
        id: '1.1',
        title: 'Test Task',
        description: 'Description',
        status: 'not_started',
        filePath: 'test.md',
        lineNumber: 1,
        specName: 'test',
        requirements: ['1.1'],
      };

      const task2: KiroTask = {
        ...task1,
        filePath: 'different.md', // Different file path shouldn't affect hash
        lineNumber: 5, // Different line number shouldn't affect hash
      };

      const hash1 = manager.generateTaskHash(task1);
      const hash2 = manager.generateTaskHash(task2);

      expect(hash1).toBe(hash2);
      expect(hash1).toMatch(/^[a-f0-9]{32}$/);
    });

    it('should generate different hash for different content', () => {
      const task1: KiroTask = {
        id: '1.1',
        title: 'Test Task',
        status: 'not_started',
        filePath: 'test.md',
        lineNumber: 1,
        specName: 'test',
      };

      const task2: KiroTask = {
        ...task1,
        title: 'Different Task',
      };

      const hash1 = manager.generateTaskHash(task1);
      const hash2 = manager.generateTaskHash(task2);

      expect(hash1).not.toBe(hash2);
    });
  });

  describe('utility methods', () => {
    beforeEach(async () => {
      mockReadFile.mockRejectedValue({ code: 'ENOENT' });
      await manager.load();
    });

    it('should get all states', () => {
      manager.updateTaskState('1.1', 123, 'hash123', 'test1.md');
      manager.updateTaskState('1.2', 124, 'hash124', 'test2.md');

      const allStates = manager.getAllStates();
      expect(allStates).toHaveLength(2);
      expect(allStates.map(s => s.taskId)).toContain('1.1');
      expect(allStates.map(s => s.taskId)).toContain('1.2');
    });

    it('should get states for specific file', () => {
      manager.updateTaskState('1.1', 123, 'hash123', 'test1.md');
      manager.updateTaskState('1.2', 124, 'hash124', 'test1.md');
      manager.updateTaskState('2.1', 125, 'hash125', 'test2.md');

      const test1States = manager.getStatesForFile('test1.md');
      expect(test1States).toHaveLength(2);
      expect(test1States.map(s => s.taskId)).toContain('1.1');
      expect(test1States.map(s => s.taskId)).toContain('1.2');

      const test2States = manager.getStatesForFile('test2.md');
      expect(test2States).toHaveLength(1);
      expect(test2States[0].taskId).toBe('2.1');
    });

    it('should cleanup orphaned states', () => {
      manager.updateTaskState('1.1', 123, 'hash123', 'test.md');
      manager.updateTaskState('1.2', 124, 'hash124', 'test.md');
      manager.updateTaskState('1.3', 125, 'hash125', 'test.md');

      const orphanedIds = manager.cleanupOrphanedStates(['1.1', '1.3']);

      expect(orphanedIds).toEqual(['1.2']);
      expect(manager.getTaskState('1.1')).not.toBeNull();
      expect(manager.getTaskState('1.2')).toBeNull();
      expect(manager.getTaskState('1.3')).not.toBeNull();
    });

    it('should get stats', () => {
      manager.updateTaskState('1.1', 123, 'hash123', 'test1.md');
      manager.updateTaskState('1.2', 124, 'hash124', 'test1.md');
      manager.updateTaskState('2.1', 125, 'hash125', 'test2.md');

      const stats = manager.getStats();

      expect(stats.totalTasks).toBe(3);
      expect(stats.syncedTasks).toBe(3);
      expect(stats.fileCount).toBe(2);
      expect(stats.lastSyncTime).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
    });

    it('should clear all state', () => {
      manager.updateTaskState('1.1', 123, 'hash123', 'test.md');
      manager.updateTaskState('1.2', 124, 'hash124', 'test.md');

      expect(manager.getAllStates()).toHaveLength(2);

      manager.clear();

      expect(manager.getAllStates()).toHaveLength(0);
      expect(manager.getTaskState('1.1')).toBeNull();
    });
  });

  describe('import/export', () => {
    beforeEach(async () => {
      mockReadFile.mockRejectedValue({ code: 'ENOENT' });
      await manager.load();
    });

    it('should export state', async () => {
      manager.updateTaskState('1.1', 123, 'hash123', 'test.md');
      manager.updateTaskState('1.2', 124, 'hash124', 'test.md');

      const exported = await manager.exportState();
      const exportData = JSON.parse(exported);

      expect(exportData.version).toBe('1.0.0');
      expect(exportData.exportedAt).toMatch(/^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}/);
      expect(exportData.states).toHaveLength(2);
      expect(exportData.states.map((s: any) => s.taskId)).toContain('1.1');
      expect(exportData.states.map((s: any) => s.taskId)).toContain('1.2');
    });

    it('should import state', async () => {
      const importData = {
        version: '1.0.0',
        exportedAt: '2024-01-01T00:00:00Z',
        states: [
          {
            taskId: '1.1',
            githubIssueNumber: 123,
            lastSynced: '2024-01-01T00:00:00Z',
            lastHash: 'hash123',
            filePath: 'test.md',
          },
          {
            taskId: '1.2',
            githubIssueNumber: 124,
            lastSynced: '2024-01-01T01:00:00Z',
            lastHash: 'hash124',
            filePath: 'test.md',
          },
        ],
      };

      await manager.importState(JSON.stringify(importData));

      expect(manager.getTaskState('1.1')).toEqual(importData.states[0]);
      expect(manager.getTaskState('1.2')).toEqual(importData.states[1]);
    });

    it('should handle invalid import data', async () => {
      await expect(manager.importState('invalid json')).rejects.toThrow(
        'Failed to import sync state'
      );

      await expect(manager.importState(JSON.stringify({ invalid: true }))).rejects.toThrow(
        'Invalid import data format'
      );

      await expect(
        manager.importState(JSON.stringify({ states: [{ invalid: 'state' }] }))
      ).rejects.toThrow('Invalid sync state');
    });
  });

  describe('error handling', () => {
    it('should throw error when accessing unloaded state', () => {
      const unloadedManager = new SyncStateManager();

      expect(() => unloadedManager.getTaskState('1.1')).toThrow(
        'Sync state not loaded. Call load() first.'
      );

      expect(() => unloadedManager.setTaskState('1.1', {} as SyncState)).toThrow(
        'Sync state not loaded. Call load() first.'
      );
    });

    it('should report loaded status correctly', async () => {
      expect(manager.isLoaded()).toBe(false);

      mockReadFile.mockRejectedValue({ code: 'ENOENT' });
      await manager.load();

      expect(manager.isLoaded()).toBe(true);
    });

    it('should return correct state file path', () => {
      expect(manager.getStateFilePath()).toBe('/test/.kiro/.github-sync-state.json');
    });
  });
});