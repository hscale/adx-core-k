import { promises as fs } from 'fs';
import { basename, dirname } from 'path';
import { KiroTask } from '../config/types';
import { logger } from '../utils/logger';
import { createHash } from 'crypto';

/**
 * Task status patterns for checkbox detection
 */
const TASK_STATUS_PATTERNS = {
  completed: /^\s*-\s*\[\s*x\s*\]\s*/i,
  in_progress: /^\s*-\s*\[\s*-\s*\]\s*/,
  not_started: /^\s*-\s*\[\s*\]\s*/,
} as const;

/**
 * Task ID extraction patterns
 */
const TASK_ID_PATTERNS = [
  /^\s*-\s*\[[^\]]*\]\s*(\d+(?:\.\d+)*)\s+(.+)$/,    // "- [x] 1.1 Task title" (without dot)
  /^\s*-\s*\[[^\]]*\]\s*(\d+(?:\.\d+)*)\.\s*(.+)$/,  // "- [x] 1.1. Task title" (with dot)
  /^\s*-\s*\[[^\]]*\]\s*(.+)$/,                       // "- [x] Task title" (no ID)
];

/**
 * Requirements extraction pattern
 */
const REQUIREMENTS_PATTERN = /_Requirements:\s*([^_\n]+)/i;

/**
 * Parser for extracting Kiro tasks from markdown files
 */
export class TaskParser {
  /**
   * Parse a task file and extract all tasks
   */
  async parseTaskFile(filePath: string): Promise<KiroTask[]> {
    try {
      logger.debug('Parsing task file', { filePath });
      
      const content = await fs.readFile(filePath, 'utf-8');
      const tasks = this.parseContent(content, filePath);
      
      logger.info('Parsed tasks from file', { 
        filePath, 
        taskCount: tasks.length,
        tasks: tasks.map(t => ({ id: t.id, title: t.title, status: t.status }))
      });
      
      return tasks;
    } catch (error) {
      logger.error('Failed to parse task file', { filePath, error: this.formatError(error) });
      throw new Error(`Failed to parse task file ${filePath}: ${this.formatError(error)}`);
    }
  }

  /**
   * Parse markdown content and extract tasks
   */
  parseContent(content: string, filePath: string): KiroTask[] {
    const lines = content.split('\n');
    const tasks: KiroTask[] = [];
    const specName = this.extractSpecName(filePath);
    
    let currentTask: Partial<KiroTask> | null = null;
    let descriptionLines: string[] = [];
    let taskRequirements: string[] | undefined = undefined;
    
    for (let i = 0; i < lines.length; i++) {
      const line = lines[i];
      if (!line) continue; // Skip undefined lines
      const lineNumber = i + 1;
      
      // Check if this line is a task
      const taskMatch = this.parseTaskLine(line);
      
      if (taskMatch) {
        // Save previous task if exists
        if (currentTask) {
          const task = this.finalizeTask(currentTask, descriptionLines, filePath, specName, taskRequirements);
          if (task) {
            tasks.push(task);
          }
        }
        
        // Start new task
        currentTask = {
          id: taskMatch.id,
          title: taskMatch.title,
          status: taskMatch.status,
          lineNumber,
        };
        descriptionLines = [];
        taskRequirements = undefined;
      } else if (currentTask && this.isDescriptionLine(line)) {
        // Check if this line contains requirements
        const requirements = this.extractRequirements(line);
        if (requirements) {
          taskRequirements = requirements;
        } else {
          // Add to current task description (but not requirements lines)
          if (!REQUIREMENTS_PATTERN.test(line.trim())) {
            descriptionLines.push(line.trim());
          }
        }
      } else if (currentTask && line.trim() === '') {
        // Empty line - might be end of task description
        continue;
      } else if (currentTask && this.isNewSection(line)) {
        // New section started - finalize current task
        const task = this.finalizeTask(currentTask, descriptionLines, filePath, specName, taskRequirements);
        if (task) {
          tasks.push(task);
        }
        currentTask = null;
        descriptionLines = [];
        taskRequirements = undefined;
      }
    }
    
    // Finalize last task if exists
    if (currentTask) {
      const task = this.finalizeTask(currentTask, descriptionLines, filePath, specName, taskRequirements);
      if (task) {
        tasks.push(task);
      }
    }
    
    return tasks;
  }

  /**
   * Parse a single task line
   */
  private parseTaskLine(line: string): {
    id: string;
    title: string;
    status: KiroTask['status'];
    requirements?: string[];
  } | null {
    // Check if line matches task pattern
    const status = this.extractTaskStatus(line);
    if (!status) {
      return null;
    }
    
    // Extract task ID and title
    for (const pattern of TASK_ID_PATTERNS) {
      const match = line.match(pattern);
      if (match) {
        const [, idOrTitle, title] = match;
        
        if (!idOrTitle) continue;
        
        // Determine if first capture group is ID or title
        const isNumericId = /^\d+(?:\.\d+)*$/.test(idOrTitle);
        const taskId = isNumericId ? idOrTitle : this.generateTaskId(idOrTitle);
        const taskTitle = isNumericId ? (title || idOrTitle) : idOrTitle;
        
        return {
          id: taskId,
          title: taskTitle.trim(),
          status,
        };
      }
    }
    
    return null;
  }

  /**
   * Extract task status from checkbox pattern
   */
  private extractTaskStatus(line: string): KiroTask['status'] | null {
    // Check in order of specificity - completed first, then in_progress, then not_started
    if (TASK_STATUS_PATTERNS.completed.test(line)) {
      return 'completed';
    }
    
    // For in_progress, be very specific - only match exactly [-] (no extra spaces around dash)
    if (/^\s*-\s*\[-\]\s*/.test(line)) {
      return 'in_progress';
    }
    
    // For not_started, match empty brackets or any other malformed checkbox
    if (/^\s*-\s*\[.*?\]\s*/.test(line)) {
      return 'not_started';
    }
    
    return null;
  }

  /**
   * Extract requirements from task line
   */
  private extractRequirements(line: string): string[] | undefined {
    const match = line.match(REQUIREMENTS_PATTERN);
    if (match && match[1]) {
      return match[1]
        .split(',')
        .map(req => req.trim())
        .filter(req => req.length > 0);
    }
    return undefined;
  }

  /**
   * Generate a task ID from title if no numeric ID is present
   */
  private generateTaskId(title: string): string {
    // Create a hash-based ID from the title
    const hash = createHash('md5').update(title.toLowerCase()).digest('hex');
    return `task-${hash.substring(0, 8)}`;
  }

  /**
   * Check if a line is part of task description
   */
  private isDescriptionLine(line: string): boolean {
    const trimmed = line.trim();
    
    // Skip empty lines
    if (!trimmed) {
      return false;
    }
    
    // Skip markdown headers
    if (trimmed.startsWith('#')) {
      return false;
    }
    
    // Skip other task lines
    if (this.extractTaskStatus(trimmed)) {
      return false;
    }
    
    // Include bullet points and indented content
    if (trimmed.startsWith('-') || trimmed.startsWith('*') || line.startsWith('  ')) {
      return true;
    }
    
    // Include requirements line
    if (REQUIREMENTS_PATTERN.test(trimmed)) {
      return true;
    }
    
    return false;
  }

  /**
   * Check if a line indicates a new section
   */
  private isNewSection(line: string): boolean {
    const trimmed = line.trim();
    
    // Markdown headers
    if (trimmed.startsWith('#')) {
      return true;
    }
    
    // Horizontal rules
    if (trimmed.match(/^-{3,}$/) || trimmed.match(/^\*{3,}$/) || trimmed.match(/^_{3,}$/)) {
      return true;
    }
    
    return false;
  }

  /**
   * Finalize a task with its description
   */
  private finalizeTask(
    taskData: Partial<KiroTask>,
    descriptionLines: string[],
    filePath: string,
    specName: string,
    requirements?: string[]
  ): KiroTask | null {
    if (!taskData.id || !taskData.title || !taskData.status || !taskData.lineNumber) {
      return null;
    }
    
    // Clean up description
    const description = descriptionLines
      .filter(line => line.length > 0)
      .join('\n')
      .trim();
    
    const task: KiroTask = {
      id: taskData.id,
      title: taskData.title,
      status: taskData.status,
      filePath,
      lineNumber: taskData.lineNumber,
      specName,
    };
    
    if (description.length > 0) {
      task.description = description;
    }
    
    if (requirements) {
      task.requirements = requirements;
    }
    
    return task;
  }

  /**
   * Extract spec name from file path
   */
  private extractSpecName(filePath: string): string {
    // Extract from path like ".kiro/specs/github-task-management/tasks.md"
    const pathParts = filePath.split('/');
    const specsIndex = pathParts.findIndex(part => part === 'specs');
    
    if (specsIndex >= 0 && specsIndex < pathParts.length - 1) {
      return pathParts[specsIndex + 1] || 'unknown';
    }
    
    // Fallback to directory name
    return basename(dirname(filePath));
  }

  /**
   * Generate GitHub issue description with Kiro context
   */
  generateIssueDescription(task: KiroTask, additionalContext?: string): string {
    let description = '';
    
    // Add task description if available
    if (task.description) {
      description += task.description + '\n\n';
    }
    
    // Add additional context if provided
    if (additionalContext) {
      description += additionalContext + '\n\n';
    }
    
    // Add Kiro metadata
    description += '---\n';
    description += '**Kiro Task Information**\n\n';
    description += `- **Task ID:** ${task.id}\n`;
    description += `- **Spec:** ${task.specName}\n`;
    description += `- **Status:** ${task.status}\n`;
    description += `- **Source:** ${task.filePath}:${task.lineNumber}\n`;
    
    if (task.requirements && task.requirements.length > 0) {
      description += `- **Requirements:** ${task.requirements.join(', ')}\n`;
    }
    
    description += `- **Last Updated:** ${new Date().toISOString()}\n`;
    description += '\n*This issue was automatically created by Kiro GitHub Task Sync*';
    
    return description;
  }

  /**
   * Generate a hash of task content for change detection
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
   * Check if two tasks are equivalent (same content)
   */
  areTasksEquivalent(task1: KiroTask, task2: KiroTask): boolean {
    return this.generateTaskHash(task1) === this.generateTaskHash(task2);
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
   * Validate task file format
   */
  async validateTaskFile(filePath: string): Promise<{ valid: boolean; errors: string[] }> {
    const errors: string[] = [];
    
    try {
      const content = await fs.readFile(filePath, 'utf-8');
      const tasks = this.parseContent(content, filePath);
      
      // Check for duplicate task IDs
      const taskIds = new Set<string>();
      for (const task of tasks) {
        if (taskIds.has(task.id)) {
          errors.push(`Duplicate task ID found: ${task.id}`);
        }
        taskIds.add(task.id);
      }
      
      // Check for tasks without titles
      const tasksWithoutTitles = tasks.filter(task => !task.title.trim());
      if (tasksWithoutTitles.length > 0) {
        errors.push(`Found ${tasksWithoutTitles.length} tasks without titles`);
      }
      
      // Check for malformed task lines
      const lines = content.split('\n');
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        if (line && line.match(/^\s*-\s*\[/) && !this.parseTaskLine(line)) {
          errors.push(`Malformed task line at ${i + 1}: ${line.trim()}`);
        }
      }
      
    } catch (error) {
      errors.push(`Failed to read or parse file: ${this.formatError(error)}`);
    }
    
    return {
      valid: errors.length === 0,
      errors,
    };
  }
}