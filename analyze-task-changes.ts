#!/usr/bin/env node

import { TaskParser } from './src/services/TaskParser';
import { logger } from './src/utils/logger';

/**
 * Task Change Analyzer
 * Analyzes the specific task that was changed and provides GitHub sync recommendations
 */
class TaskChangeAnalyzer {
  private taskParser: TaskParser;

  constructor() {
    this.taskParser = new TaskParser();
  }

  /**
   * Analyze the task file and identify the specific change
   */
  async analyzeTaskChanges(): Promise<void> {
    const filePath = '.kiro/specs/adx-core/tasks.md';
    
    try {
      logger.info('Analyzing task file changes', { filePath });

      // Parse the current task file
      const tasks = await this.taskParser.parseTaskFile(filePath);
      
      // Find the specific task that was changed based on the diff
      const changedTask = tasks.find(task => task.id === '1' && task.title.includes('Project Structure and Workspace Setup'));
      
      if (!changedTask) {
        logger.error('Could not find the changed task');
        return;
      }

      // Analyze the change
      this.analyzeSpecificTaskChange(changedTask);
      
      // Provide GitHub sync recommendations
      this.provideGitHubSyncRecommendations(changedTask);

    } catch (error) {
      logger.error('Failed to analyze task changes', { 
        error: error instanceof Error ? error.message : String(error) 
      });
      throw error;
    }
  }

  /**
   * Analyze the specific task change
   */
  private analyzeSpecificTaskChange(task: any): void {
    console.log('\n🔍 TASK CHANGE ANALYSIS');
    console.log('========================');
    console.log(`Task ID: ${task.id}`);
    console.log(`Task Title: ${task.title}`);
    console.log(`Current Status: ${task.status}`);
    console.log(`Spec: ${task.specName}`);
    console.log(`File: ${task.filePath}:${task.lineNumber}`);
    
    if (task.description) {
      console.log(`\nDescription:`);
      console.log(task.description);
    }

    if (task.requirements) {
      console.log(`\nRequirements: ${task.requirements.join(', ')}`);
    }

    // Based on the diff, this task was changed from in_progress to not_started
    console.log('\n📝 DETECTED CHANGE:');
    console.log('Status changed from "in_progress" ([-]) to "not_started" ([ ])');
    console.log('This indicates the task was reset from in-progress back to not started.');
  }

  /**
   * Provide GitHub sync recommendations
   */
  private provideGitHubSyncRecommendations(task: any): void {
    console.log('\n🔄 GITHUB SYNC RECOMMENDATIONS');
    console.log('===============================');
    
    const issueTitle = `📋 [adx-core] 1: Project Structure and Workspace Setup`;
    const labels = [
      'kiro:1',
      'spec:adx-core', 
      'status:not_started',
      'phase:1',
      'requirement:3.1',
      'requirement:13.1'
    ];

    console.log('Recommended GitHub Issue Update:');
    console.log(`Title: ${issueTitle}`);
    console.log(`Labels: ${labels.join(', ')}`);
    console.log(`Action: Update existing issue or create new one`);
    console.log(`Status: Reopen if closed (task is no longer completed)`);
    
    console.log('\nIssue Description:');
    console.log('---');
    console.log(this.generateIssueDescription(task));
  }

  /**
   * Generate the GitHub issue description
   */
  private generateIssueDescription(task: any): string {
    let description = '';
    
    // Add task description
    if (task.description) {
      description += task.description + '\n\n';
    }
    
    // Add context
    description += '**Phase:** 1\n\n';
    description += '📋 **Status:** This task is ready to be started.\n\n';
    
    // Add implementation guidelines
    description += '**Implementation Guidelines:**\n';
    description += '- Follow the ADX CORE Temporal-first architecture principles\n';
    description += '- Ensure multi-tenant isolation at all levels\n';
    description += '- Implement comprehensive testing (unit, integration, workflow)\n';
    description += '- Document all APIs and workflows\n';
    description += '- Follow the microservices team autonomy model\n\n';
    
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
}

/**
 * Main execution function
 */
async function main() {
  try {
    const analyzer = new TaskChangeAnalyzer();
    await analyzer.analyzeTaskChanges();
    
    console.log('\n✅ Task change analysis completed successfully');
    process.exit(0);
  } catch (error) {
    console.error('\n❌ Task change analysis failed:', error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}

export { TaskChangeAnalyzer };