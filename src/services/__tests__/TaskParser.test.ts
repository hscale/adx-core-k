import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { promises as fs } from 'fs';
import { TaskParser } from '../TaskParser';
import { KiroTask } from '../../config/types';

// Mock fs module
vi.mock('fs', () => ({
  promises: {
    readFile: vi.fn(),
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

describe('TaskParser', () => {
  let parser: TaskParser;
  const mockReadFile = vi.mocked(fs.readFile);

  beforeEach(() => {
    parser = new TaskParser();
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('parseContent', () => {
    it('should parse tasks with numeric IDs', () => {
      const content = `
# Implementation Tasks

- [ ] 1. Setup project structure
  - Create basic folder structure
  - Initialize configuration
  - _Requirements: 1.1_

- [-] 2.1 Implement core features
  - Build main functionality
  - _Requirements: 2.1, 2.2_

- [x] 3.2.1 Testing
  - Write unit tests
  - _Requirements: 3.1_
      `;

      const tasks = parser.parseContent(content, '.kiro/specs/test-spec/tasks.md');

      expect(tasks).toHaveLength(3);
      
      expect(tasks[0]).toEqual({
        id: '1',
        title: 'Setup project structure',
        description: '- Create basic folder structure\n- Initialize configuration',
        status: 'not_started',
        filePath: '.kiro/specs/test-spec/tasks.md',
        lineNumber: 4,
        specName: 'test-spec',
        requirements: ['1.1'],
      });

      expect(tasks[1]).toEqual({
        id: '2.1',
        title: 'Implement core features',
        description: '- Build main functionality',
        status: 'in_progress',
        filePath: '.kiro/specs/test-spec/tasks.md',
        lineNumber: 9,
        specName: 'test-spec',
        requirements: ['2.1', '2.2'],
      });

      expect(tasks[2]).toEqual({
        id: '3.2.1',
        title: 'Testing',
        description: '- Write unit tests',
        status: 'completed',
        filePath: '.kiro/specs/test-spec/tasks.md',
        lineNumber: 13,
        specName: 'test-spec',
        requirements: ['3.1'],
      });
    });

    it('should parse tasks without numeric IDs', () => {
      const content = `
- [ ] Setup development environment
  - Install dependencies
  - Configure tools

- [x] Write documentation
  - Create README
  - Add examples
      `;

      const tasks = parser.parseContent(content, '.kiro/specs/test/tasks.md');

      expect(tasks).toHaveLength(2);
      expect(tasks[0].id).toMatch(/^task-[a-f0-9]{8}$/);
      expect(tasks[0].title).toBe('Setup development environment');
      expect(tasks[0].status).toBe('not_started');
      
      expect(tasks[1].id).toMatch(/^task-[a-f0-9]{8}$/);
      expect(tasks[1].title).toBe('Write documentation');
      expect(tasks[1].status).toBe('completed');
    });

    it('should handle different checkbox formats', () => {
      const content = `
- [ ] Not started task
- [-] In progress task  
- [x] Completed task (lowercase)
- [X] Completed task (uppercase)
- [  ] Task with spaces
- [ -] Mixed format
      `;

      const tasks = parser.parseContent(content, 'test.md');

      expect(tasks).toHaveLength(6);
      expect(tasks[0].status).toBe('not_started');
      expect(tasks[1].status).toBe('in_progress');
      expect(tasks[2].status).toBe('completed');
      expect(tasks[3].status).toBe('completed');
      expect(tasks[4].status).toBe('not_started');
      expect(tasks[5].status).toBe('not_started'); // Mixed format defaults to not_started
    });

    it('should extract requirements correctly', () => {
      const content = `
- [ ] Task with single requirement
  - _Requirements: 1.1_

- [ ] Task with multiple requirements
  - _Requirements: 2.1, 2.2, 2.3_

- [ ] Task with requirements in different format
  - _requirements: 3.1, 3.2_

- [ ] Task without requirements
  - Some description
      `;

      const tasks = parser.parseContent(content, 'test.md');

      expect(tasks).toHaveLength(4);
      expect(tasks[0].requirements).toEqual(['1.1']);
      expect(tasks[1].requirements).toEqual(['2.1', '2.2', '2.3']);
      expect(tasks[2].requirements).toEqual(['3.1', '3.2']);
      expect(tasks[3].requirements).toBeUndefined();
    });

    it('should handle tasks without descriptions', () => {
      const content = `
- [ ] 1. Simple task
- [x] 2. Another simple task

## Next Section
      `;

      const tasks = parser.parseContent(content, 'test.md');

      expect(tasks).toHaveLength(2);
      expect(tasks[0].description).toBeUndefined();
      expect(tasks[1].description).toBeUndefined();
    });

    it('should stop parsing description at new sections', () => {
      const content = `
- [ ] 1. Task with description
  - First point
  - Second point

## New Section

- [ ] 2. Second task
      `;

      const tasks = parser.parseContent(content, 'test.md');

      expect(tasks).toHaveLength(2);
      expect(tasks[0].description).toBe('- First point\n- Second point');
      expect(tasks[1].description).toBeUndefined();
    });

    it('should extract spec name from file path', () => {
      const content = '- [ ] Test task';
      
      const tasks1 = parser.parseContent(content, '.kiro/specs/github-sync/tasks.md');
      const tasks2 = parser.parseContent(content, '/path/to/my-project/tasks.md');
      
      expect(tasks1[0].specName).toBe('github-sync');
      expect(tasks2[0].specName).toBe('my-project');
    });
  });

  describe('parseTaskFile', () => {
    it('should read and parse file successfully', async () => {
      const content = `
- [ ] 1. Test task
  - Description here
      `;

      mockReadFile.mockResolvedValue(content);

      const tasks = await parser.parseTaskFile('test.md');

      expect(mockReadFile).toHaveBeenCalledWith('test.md', 'utf-8');
      expect(tasks).toHaveLength(1);
      expect(tasks[0].title).toBe('Test task');
    });

    it('should handle file read errors', async () => {
      mockReadFile.mockRejectedValue(new Error('File not found'));

      await expect(parser.parseTaskFile('nonexistent.md')).rejects.toThrow(
        'Failed to parse task file nonexistent.md: File not found'
      );
    });
  });

  describe('generateIssueDescription', () => {
    it('should generate description with all metadata', () => {
      const task: KiroTask = {
        id: '1.1',
        title: 'Test Task',
        description: 'Task description here',
        status: 'not_started',
        filePath: '.kiro/specs/test/tasks.md',
        lineNumber: 5,
        specName: 'test',
        requirements: ['1.1', '2.1'],
      };

      const description = parser.generateIssueDescription(task, 'Additional context');

      expect(description).toContain('Task description here');
      expect(description).toContain('Additional context');
      expect(description).toContain('**Task ID:** 1.1');
      expect(description).toContain('**Spec:** test');
      expect(description).toContain('**Status:** not_started');
      expect(description).toContain('**Source:** .kiro/specs/test/tasks.md:5');
      expect(description).toContain('**Requirements:** 1.1, 2.1');
      expect(description).toContain('automatically created by Kiro');
    });

    it('should handle task without description and requirements', () => {
      const task: KiroTask = {
        id: '1',
        title: 'Simple Task',
        status: 'completed',
        filePath: 'tasks.md',
        lineNumber: 1,
        specName: 'test',
      };

      const description = parser.generateIssueDescription(task);

      expect(description).not.toContain('undefined');
      expect(description).toContain('**Task ID:** 1');
      expect(description).not.toContain('**Requirements:**');
    });
  });

  describe('generateTaskHash', () => {
    it('should generate consistent hash for same task content', () => {
      const task1: KiroTask = {
        id: '1',
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

      const hash1 = parser.generateTaskHash(task1);
      const hash2 = parser.generateTaskHash(task2);

      expect(hash1).toBe(hash2);
      expect(hash1).toMatch(/^[a-f0-9]{32}$/);
    });

    it('should generate different hash for different content', () => {
      const task1: KiroTask = {
        id: '1',
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

      const hash1 = parser.generateTaskHash(task1);
      const hash2 = parser.generateTaskHash(task2);

      expect(hash1).not.toBe(hash2);
    });
  });

  describe('areTasksEquivalent', () => {
    it('should return true for equivalent tasks', () => {
      const task1: KiroTask = {
        id: '1',
        title: 'Test Task',
        description: 'Description',
        status: 'not_started',
        filePath: 'test.md',
        lineNumber: 1,
        specName: 'test',
      };

      const task2: KiroTask = {
        ...task1,
        filePath: 'different.md',
        lineNumber: 5,
      };

      expect(parser.areTasksEquivalent(task1, task2)).toBe(true);
    });

    it('should return false for different tasks', () => {
      const task1: KiroTask = {
        id: '1',
        title: 'Test Task',
        status: 'not_started',
        filePath: 'test.md',
        lineNumber: 1,
        specName: 'test',
      };

      const task2: KiroTask = {
        ...task1,
        status: 'completed',
      };

      expect(parser.areTasksEquivalent(task1, task2)).toBe(false);
    });
  });

  describe('validateTaskFile', () => {
    it('should validate correct task file', async () => {
      const content = `
- [ ] 1. First task
- [x] 2. Second task
      `;

      mockReadFile.mockResolvedValue(content);

      const result = await parser.validateTaskFile('test.md');

      expect(result.valid).toBe(true);
      expect(result.errors).toHaveLength(0);
    });

    it('should detect duplicate task IDs', async () => {
      const content = `
- [ ] 1. First task
- [x] 1. Duplicate task
      `;

      mockReadFile.mockResolvedValue(content);

      const result = await parser.validateTaskFile('test.md');

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Duplicate task ID found: 1');
    });

    it('should detect malformed task lines', async () => {
      const content = `
- [ ] 1. Good task
- [invalid] Bad task
- [ Incomplete checkbox
      `;

      mockReadFile.mockResolvedValue(content);

      const result = await parser.validateTaskFile('test.md');

      expect(result.valid).toBe(false);
      expect(result.errors.length).toBeGreaterThan(0);
    });

    it('should handle file read errors', async () => {
      mockReadFile.mockRejectedValue(new Error('Permission denied'));

      const result = await parser.validateTaskFile('test.md');

      expect(result.valid).toBe(false);
      expect(result.errors).toContain('Failed to read or parse file: Permission denied');
    });
  });

  describe('edge cases', () => {
    it('should handle empty file', () => {
      const tasks = parser.parseContent('', 'test.md');
      expect(tasks).toHaveLength(0);
    });

    it('should handle file with only headers', () => {
      const content = `
# Main Header
## Sub Header
### Another Header
      `;

      const tasks = parser.parseContent(content, 'test.md');
      expect(tasks).toHaveLength(0);
    });

    it('should handle mixed content', () => {
      const content = `
# Project Tasks

Some introduction text here.

- [ ] 1. First task
  - Task description

Regular paragraph text.

- [x] 2. Second task

## Another Section

More text here.

- [-] 3. Third task
      `;

      const tasks = parser.parseContent(content, 'test.md');

      expect(tasks).toHaveLength(3);
      expect(tasks[0].title).toBe('First task');
      expect(tasks[1].title).toBe('Second task');
      expect(tasks[2].title).toBe('Third task');
    });

    it('should handle tasks with complex descriptions', () => {
      const content = `
- [ ] 1. Complex task
  - First bullet point
  - Second bullet point with **bold** text
  - Third point with \`code\`
  - Nested list:
    - Sub item 1
    - Sub item 2
  - _Requirements: 1.1, 1.2_
      `;

      const tasks = parser.parseContent(content, 'test.md');

      expect(tasks).toHaveLength(1);
      expect(tasks[0].description).toContain('First bullet point');
      expect(tasks[0].description).toContain('**bold**');
      expect(tasks[0].description).toContain('`code`');
      expect(tasks[0].description).toContain('Sub item 1');
      expect(tasks[0].requirements).toEqual(['1.1', '1.2']);
    });
  });
});