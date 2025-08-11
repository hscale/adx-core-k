#!/usr/bin/env node

import { TaskParser } from './src/services/TaskParser';
import { logger } from './src/utils/logger';
import { execSync } from 'child_process';

/**
 * GitHub Issues Creator
 * Creates GitHub issues for all Kiro tasks using GitHub CLI
 */
class GitHubIssuesCreator {
  private taskParser: TaskParser;
  private repository = 'hscale/adx-core-k';

  constructor() {
    this.taskParser = new TaskParser();
  }

  /**
   * Create GitHub issues for all tasks
   */
  async createAllIssues(): Promise<void> {
    try {
      logger.info('Starting GitHub issues creation for all tasks...');

      // Parse the task file
      const tasks = await this.taskParser.parseTaskFile('.kiro/specs/adx-core/tasks.md');
      logger.info(`Found ${tasks.length} tasks to sync`);

      // Create issues for each task
      for (const task of tasks) {
        await this.createIssueForTask(task);
        // Add a small delay to avoid rate limiting
        await this.sleep(500);
      }

      logger.info('All GitHub issues created successfully');
    } catch (error) {
      logger.error('Failed to create GitHub issues', { 
        error: error instanceof Error ? error.message : String(error) 
      });
      throw error;
    }
  }

  /**
   * Create a GitHub issue for a single task
   */
  private async createIssueForTask(task: any): Promise<void> {
    try {
      const title = this.generateIssueTitle(task);
      const body = this.generateIssueBody(task);
      const labels = this.generateLabels(task);

      // Create issue using GitHub CLI
      const command = [
        'gh', 'issue', 'create',
        '--repo', this.repository,
        '--title', `"${title}"`,
        '--body', `"${body.replace(/"/g, '\\"')}"`,
        '--label', labels.join(',')
      ].join(' ');

      logger.info(`Creating issue for task ${task.id}: ${task.title}`);
      
      const result = execSync(command, { encoding: 'utf-8' });
      const issueUrl = result.trim();
      
      logger.info(`Created GitHub issue for task ${task.id}`, {
        taskId: task.id,
        taskTitle: task.title,
        issueUrl
      });

    } catch (error) {
      logger.error(`Failed to create issue for task ${task.id}`, {
        taskId: task.id,
        taskTitle: task.title,
        error: error instanceof Error ? error.message : String(error)
      });
      // Continue with other tasks even if one fails
    }
  }

  /**
   * Generate issue title from task
   */
  private generateIssueTitle(task: any): string {
    const statusEmoji = this.getStatusEmoji(task.status);
    return `${statusEmoji} [${task.specName}] ${task.id}: ${task.title}`;
  }

  /**
   * Get emoji for task status
   */
  private getStatusEmoji(status: string): string {
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
   * Generate issue body with comprehensive information
   */
  private generateIssueBody(task: any): string {
    let body = '';
    
    // Add task description
    if (task.description) {
      body += task.description + '\\n\\n';
    }
    
    // Add phase information
    const phaseMatch = task.id.match(/^(\\d+)/);
    if (phaseMatch) {
      const phaseNumber = phaseMatch[1];
      body += `**Phase:** ${phaseNumber}\\n\\n`;
    }

    // Add status-specific information
    switch (task.status) {
      case 'completed':
        body += '🎉 **Status:** This task has been completed!\\n\\n';
        break;
      case 'in_progress':
        body += '🔄 **Status:** This task is currently in progress.\\n\\n';
        break;
      case 'not_started':
        body += '📋 **Status:** This task is ready to be started.\\n\\n';
        break;
    }

    // Add implementation guidelines
    body += '**Implementation Guidelines:**\\n';
    body += '- Follow the ADX CORE Temporal-first architecture principles\\n';
    body += '- Ensure multi-tenant isolation at all levels\\n';
    body += '- Implement comprehensive testing (unit, integration, workflow)\\n';
    body += '- Document all APIs and workflows\\n';
    body += '- Follow the microservices team autonomy model\\n\\n';

    // Add Kiro metadata
    body += '---\\n';
    body += '**Kiro Task Information**\\n\\n';
    body += `- **Task ID:** ${task.id}\\n`;
    body += `- **Spec:** ${task.specName}\\n`;
    body += `- **Status:** ${task.status}\\n`;
    body += `- **Source:** ${task.filePath}:${task.lineNumber}\\n`;
    
    if (task.requirements && task.requirements.length > 0) {
      body += `- **Requirements:** ${task.requirements.join(', ')}\\n`;
    }
    
    body += `- **Last Updated:** ${new Date().toISOString()}\\n`;
    body += '\\n*This issue was automatically created by Kiro GitHub Task Sync*';
    
    return body;
  }

  /**
   * Generate labels for the issue
   */
  private generateLabels(task: any): string[] {
    const labels = [
      `kiro:${task.id}`,           // Unique task identifier
      `spec:${task.specName}`,     // Spec name
      `status:${task.status}`,     // Current status
    ];

    // Add phase label if task ID suggests a phase
    const phaseMatch = task.id.match(/^(\\d+)/);
    if (phaseMatch) {
      labels.push(`phase:${phaseMatch[1]}`);
    }

    // Add requirement labels if available
    if (task.requirements) {
      task.requirements.forEach((req: string) => {
        const cleanReq = req.replace(/[^a-zA-Z0-9-]/g, '-').toLowerCase();
        labels.push(`requirement:${cleanReq}`);
      });
    }

    return labels;
  }

  /**
   * Sleep for specified milliseconds
   */
  private sleep(ms: number): Promise<void> {
    return new Promise(resolve => setTimeout(resolve, ms));
  }
}

/**
 * Main execution function
 */
async function main() {
  try {
    console.log('🚀 Creating GitHub issues for all Kiro tasks...');
    
    const creator = new GitHubIssuesCreator();
    await creator.createAllIssues();
    
    console.log('✅ All GitHub issues created successfully!');
    console.log('🔗 View issues at: https://github.com/hscale/adx-core-k/issues');
    process.exit(0);
  } catch (error) {
    console.error('❌ Failed to create GitHub issues:', error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}

export { GitHubIssuesCreator };