#!/usr/bin/env node

import { GitHubClient } from './src/services/GitHubClient';
import { ConfigManager } from './src/config/ConfigManager';
import { logger } from './src/utils/logger';

/**
 * GitHub Task Sync Handler for Task 1 Completion
 * Syncs the completion status of Task 1: Project Structure and Workspace Setup
 */
class Task1CompletionSync {
  private configManager: ConfigManager;

  constructor() {
    this.configManager = new ConfigManager();
  }

  async syncTaskCompletion(): Promise<void> {
    try {
      logger.info('Starting Task 1 completion sync to GitHub');

      // Load GitHub configuration
      const config = await this.configManager.load();
      if (!config.enabled) {
        logger.warn('GitHub sync is disabled, skipping sync');
        return;
      }

      // Initialize GitHub client
      const githubClient = new GitHubClient(config);

      // Task details
      const taskDetails = {
        id: '1',
        title: 'Project Structure and Workspace Setup',
        status: 'completed',
        spec: 'adx-core',
        phase: '1',
        requirements: ['3.1', '13.1'],
        description: this.getTaskDescription(),
        completionEvidence: this.getCompletionEvidence(),
      };

      // Check if issue already exists
      const label = `${config.labelPrefix}${taskDetails.id}`;
      const existingIssue = await githubClient.findIssueByLabel(label);

      if (existingIssue) {
        await this.updateExistingIssue(existingIssue, taskDetails, githubClient, config);
      } else {
        await this.createNewIssue(taskDetails, githubClient, config);
      }

      logger.info('Task 1 completion sync completed successfully');
    } catch (error) {
      logger.error('Failed to sync Task 1 completion', {
        error: error instanceof Error ? error.message : String(error)
      });
      throw error;
    }
  }

  private async updateExistingIssue(
    existingIssue: any,
    taskDetails: any,
    githubClient: GitHubClient,
    config: any
  ): Promise<void> {
    const title = this.generateIssueTitle(taskDetails);
    const body = this.generateIssueDescription(taskDetails);
    const labels = this.generateLabels(taskDetails, config);

    // Close the issue since task is completed
    if (existingIssue.state === 'open') {
      await githubClient.closeIssue(existingIssue.number);
      logger.info('Closed GitHub issue for completed task', {
        taskId: taskDetails.id,
        issueNumber: existingIssue.number
      });
    }

    // Update issue content
    await githubClient.updateIssue(existingIssue.number, title, body);
    
    // Update labels
    await githubClient.updateIssueLabels(existingIssue.number, labels);

    logger.info('Updated GitHub issue for completed task', {
      taskId: taskDetails.id,
      issueNumber: existingIssue.number,
      issueUrl: existingIssue.html_url
    });
  }

  private async createNewIssue(
    taskDetails: any,
    githubClient: GitHubClient,
    config: any
  ): Promise<void> {
    const title = this.generateIssueTitle(taskDetails);
    const body = this.generateIssueDescription(taskDetails);
    const labels = this.generateLabels(taskDetails, config);

    const issue = await githubClient.createIssue(title, body, labels);

    // Immediately close it since task is completed
    await githubClient.closeIssue(issue.number);

    logger.info('Created and closed GitHub issue for completed task', {
      taskId: taskDetails.id,
      issueNumber: issue.number,
      issueUrl: issue.html_url
    });
  }

  private generateIssueTitle(taskDetails: any): string {
    return `✅ [${taskDetails.spec}] ${taskDetails.id}: ${taskDetails.title}`;
  }

  private generateLabels(taskDetails: any, config: any): string[] {
    const labels = [
      `${config.labelPrefix}${taskDetails.id}`,
      `spec:${taskDetails.spec}`,
      `status:${taskDetails.status}`,
      `phase:${taskDetails.phase}`,
    ];

    // Add requirement labels
    taskDetails.requirements.forEach((req: string) => {
      labels.push(`requirement:${req}`);
    });

    return labels;
  }

  private generateIssueDescription(taskDetails: any): string {
    return `## ${taskDetails.title}

${taskDetails.description}

**Phase:** ${taskDetails.phase}

✅ **Status:** This task has been completed successfully!

**Implementation Guidelines:**
- Follow the ADX CORE Temporal-first architecture principles
- Ensure multi-tenant isolation at all levels
- Implement comprehensive testing (unit, integration, workflow)
- Document all APIs and workflows
- Follow the microservices team autonomy model

**Completion Evidence:**
${taskDetails.completionEvidence}

---
**Kiro Task Information**

- **Task ID:** ${taskDetails.id}
- **Spec:** ${taskDetails.spec}
- **Status:** ${taskDetails.status}
- **Source:** .kiro/specs/adx-core/tasks.md:13
- **Requirements:** ${taskDetails.requirements.map(r => `${r} (${this.getRequirementDescription(r)})`).join(', ')}
- **Last Updated:** ${new Date().toISOString()}

*This issue was automatically synced by Kiro GitHub Task Sync*`;
  }

  private getTaskDescription(): string {
    return `- [x] Create root \`adx-core/\` directory with Rust workspace structure
- [x] Initialize workspace \`Cargo.toml\` with microservices members (auth-service, user-service, file-service, tenant-service, workflow-service)
- [x] Create \`services/shared/\` crate for common utilities, types, and Temporal abstractions
- [x] Set up \`infrastructure/docker/\` directory with development Docker Compose files
- [x] Create \`scripts/\` directory with development and deployment automation scripts
- [x] Initialize Git repository with proper \`.gitignore\` for Rust and Node.js projects`;
  }

  private getCompletionEvidence(): string {
    return `Based on the current project structure, this task has been completed with:
- ✅ Rust workspace structure established in \`adx-core/\`
- ✅ Workspace \`Cargo.toml\` with microservice members configured
- ✅ Shared crate with Temporal abstractions implemented
- ✅ Docker infrastructure setup with Temporal configuration
- ✅ Development and deployment scripts created
- ✅ Git repository initialized with appropriate \`.gitignore\`

**Key Deliverables Verified:**
- \`adx-core/Cargo.toml\` - Workspace configuration with all microservices
- \`adx-core/services/shared/\` - Common utilities and Temporal abstractions
- \`adx-core/infrastructure/docker/\` - Development Docker Compose files
- \`adx-core/scripts/\` - Development and deployment automation
- \`adx-core/docs/\` - Temporal setup and workflow documentation
- Proper Git repository structure with \`.gitignore\``;
  }

  private getRequirementDescription(requirement: string): string {
    const descriptions: Record<string, string> = {
      '3.1': 'Temporal-first backend microservices',
      '13.1': 'Team autonomy and vertical ownership',
    };
    return descriptions[requirement] || requirement;
  }
}

/**
 * Main execution function
 */
async function main() {
  try {
    const syncHandler = new Task1CompletionSync();
    await syncHandler.syncTaskCompletion();
    
    console.log('✅ Task 1 completion sync completed successfully');
    process.exit(0);
  } catch (error) {
    console.error('❌ Task 1 completion sync failed:', error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}

export { Task1CompletionSync };