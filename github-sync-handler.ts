#!/usr/bin/env node

import { TaskParser } from './src/services/TaskParser';
import { GitHubClient } from './src/services/GitHubClient';
import { ConfigManager } from './src/config/ConfigManager';
import { logger } from './src/utils/logger';
import { KiroTask, GitHubConfig } from './src/config/types';

/**
 * GitHub Task Sync Handler
 * Analyzes task file changes and syncs with GitHub issues
 */
class GitHubTaskSyncHandler {
  private taskParser: TaskParser;
  private configManager: ConfigManager;

  constructor() {
    this.taskParser = new TaskParser();
    this.configManager = new ConfigManager();
  }

  /**
   * Main handler for task file changes
   */
  async handleTaskFileChange(filePath: string): Promise<void> {
    try {
      logger.info('Handling task file change', { filePath });

      // Load GitHub configuration
      const config = await this.loadConfig();
      if (!config) {
        logger.warn('GitHub sync not configured, skipping sync');
        return;
      }

      // Parse the task file
      const tasks = await this.taskParser.parseTaskFile(filePath);
      logger.info(`Found ${tasks.length} tasks in file`, { filePath });

      // Initialize GitHub client
      const githubClient = new GitHubClient(config);

      // Process each task
      for (const task of tasks) {
        await this.syncTask(task, githubClient, config);
      }

      logger.info('Task file sync completed successfully', { filePath });
    } catch (error) {
      logger.error('Failed to handle task file change', {
        filePath,
        error: error instanceof Error ? error.message : String(error)
      });
      throw error;
    }
  }

  /**
   * Sync a single task with GitHub
   */
  private async syncTask(task: KiroTask, githubClient: GitHubClient, config: GitHubConfig): Promise<void> {
    try {
      const label = `${config.labelPrefix}${task.id}`;

      // Check if issue already exists
      const existingIssue = await githubClient.findIssueByLabel(label);

      if (existingIssue) {
        // Update existing issue
        await this.updateExistingIssue(task, existingIssue, githubClient, config);
      } else {
        // Create new issue
        await this.createNewIssue(task, githubClient, config);
      }
    } catch (error) {
      logger.error('Failed to sync task', {
        taskId: task.id,
        taskTitle: task.title,
        error: error instanceof Error ? error.message : String(error)
      });
      // Continue with other tasks even if one fails
    }
  }

  /**
   * Create a new GitHub issue for a task
   */
  private async createNewIssue(task: KiroTask, githubClient: GitHubClient, config: GitHubConfig): Promise<void> {
    const title = this.generateIssueTitle(task);
    const body = this.taskParser.generateIssueDescription(task, this.generateAdditionalContext(task));
    const labels = this.generateLabels(task, config);

    const issue = await githubClient.createIssue(title, body, labels);

    logger.info('Created new GitHub issue for task', {
      taskId: task.id,
      taskTitle: task.title,
      issueNumber: issue.number,
      issueUrl: issue.html_url
    });
  }

  /**
   * Update an existing GitHub issue
   */
  private async updateExistingIssue(
    task: KiroTask,
    existingIssue: any,
    githubClient: GitHubClient,
    _config: GitHubConfig
  ): Promise<void> {
    const title = this.generateIssueTitle(task);
    const body = this.taskParser.generateIssueDescription(task, this.generateAdditionalContext(task));

    // Check if the issue needs to be closed/reopened based on task status
    const shouldBeClosed = task.status === 'completed';
    const isCurrentlyClosed = existingIssue.state === 'closed';

    if (shouldBeClosed && !isCurrentlyClosed) {
      // Close the issue
      await githubClient.closeIssue(existingIssue.number);
      logger.info('Closed GitHub issue for completed task', {
        taskId: task.id,
        issueNumber: existingIssue.number
      });
    } else if (!shouldBeClosed && isCurrentlyClosed) {
      // Reopen by updating (GitHub will reopen automatically if we update a closed issue)
      await githubClient.updateIssue(existingIssue.number, title, body);
      logger.info('Reopened GitHub issue for active task', {
        taskId: task.id,
        issueNumber: existingIssue.number
      });
    } else {
      // Just update the content
      await githubClient.updateIssue(existingIssue.number, title, body);
      logger.info('Updated GitHub issue for task', {
        taskId: task.id,
        issueNumber: existingIssue.number
      });
    }
  }

  /**
   * Generate issue title from task
   */
  private generateIssueTitle(task: KiroTask): string {
    const statusEmoji = this.getStatusEmoji(task.status);
    return `${statusEmoji} [${task.specName}] ${task.id}: ${task.title}`;
  }

  /**
   * Get emoji for task status
   */
  private getStatusEmoji(status: KiroTask['status']): string {
    switch (status) {
      case 'completed':
        return '✅';
      case 'in_progress':
        return '🔄';
      case 'not_started':
        return '📋';
      default:
        return '❓';
    }
  }

  /**
   * Generate labels for the issue
   */
  private generateLabels(task: KiroTask, config: GitHubConfig): string[] {
    const labels = [
      `${config.labelPrefix}${task.id}`, // Unique task identifier
      `spec:${task.specName}`,           // Spec name
      `status:${task.status}`,           // Current status
    ];

    // Add phase label if task ID suggests a phase
    const phaseMatch = task.id.match(/^(\d+)/);
    if (phaseMatch) {
      labels.push(`phase:${phaseMatch[1]}`);
    }

    // Add requirement labels if available
    if (task.requirements) {
      task.requirements.forEach(req => {
        labels.push(`requirement:${req.replace(/[^a-zA-Z0-9-]/g, '-').toLowerCase()}`);
      });
    }

    return labels;
  }

  /**
   * Generate additional context for the issue description
   */
  private generateAdditionalContext(task: KiroTask): string {
    let context = '';

    // Add phase information
    const phaseMatch = task.id.match(/^(\d+)/);
    if (phaseMatch) {
      const phaseNumber = phaseMatch[1];
      context += `**Phase:** ${phaseNumber}\n\n`;
    }

    // Add status-specific information
    switch (task.status) {
      case 'completed':
        context += '🎉 **Status:** This task has been completed!\n\n';
        break;
      case 'in_progress':
        context += '🔄 **Status:** This task is currently in progress.\n\n';
        break;
      case 'not_started':
        context += '📋 **Status:** This task is ready to be started.\n\n';
        break;
    }

    // Add implementation guidelines
    context += '**Implementation Guidelines:**\n';
    context += '- Follow the ADX CORE Temporal-first architecture principles\n';
    context += '- Ensure multi-tenant isolation at all levels\n';
    context += '- Implement comprehensive testing (unit, integration, workflow)\n';
    context += '- Document all APIs and workflows\n';
    context += '- Follow the microservices team autonomy model\n\n';

    return context;
  }

  /**
   * Load GitHub configuration
   */
  private async loadConfig(): Promise<GitHubConfig | null> {
    try {
      const config = await this.configManager.load();

      if (!config.enabled) {
        logger.info('GitHub sync is disabled in configuration');
        return null;
      }

      return config;
    } catch (error) {
      if (error instanceof Error && error.message.includes('not configured')) {
        logger.warn('GitHub sync not configured. Run setup first.');
        return null;
      }
      throw error;
    }
  }
}

/**
 * Main execution function
 */
async function main() {
  const filePath = '.kiro/specs/adx-core/tasks.md';

  try {
    const handler = new GitHubTaskSyncHandler();
    await handler.handleTaskFileChange(filePath);

    console.log('✅ GitHub task sync completed successfully');
    process.exit(0);
  } catch (error) {
    console.error('❌ GitHub task sync failed:', error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}

export { GitHubTaskSyncHandler };