#!/usr/bin/env node

import { GitHubClient } from './src/services/GitHubClient';
import { TaskParser } from './src/services/TaskParser';
import { readFileSync } from 'fs';
import { logger } from './src/utils/logger';

async function syncTask5Completion() {
  try {
    logger.info('🔄 Syncing Task 5 completion status to GitHub');
    
    // Read the tasks file
    const tasksContent = readFileSync('.kiro/specs/adx-core/tasks.md', 'utf-8');
    const parser = new TaskParser();
    const tasks = parser.parseContent(tasksContent, '.kiro/specs/adx-core/tasks.md');
    
    // Debug: log all parsed tasks
    logger.info('📋 Parsed tasks:', {
      count: tasks.length,
      taskIds: tasks.map(t => ({ id: t.id, title: t.title.substring(0, 50) }))
    });
    
    // Find Task 5
    const task5 = tasks.find(task => task.id === 5 || task.id === '5');
    if (!task5) {
      logger.error('Task 5 not found. Available tasks:', tasks.map(t => ({ id: t.id, type: typeof t.id })));
      throw new Error('Task 5 not found');
    }
    
    logger.info('📋 Task 5 Details:', {
      id: task5.id,
      title: task5.title,
      status: task5.status,
      phase: task5.phase,
      requirements: task5.requirements
    });
    
    // Check if we have GitHub credentials for actual sync
    const hasGitHubToken = process.env.GITHUB_TOKEN && process.env.GITHUB_TOKEN !== 'your_github_token_here';
    
    if (!hasGitHubToken) {
      logger.info('🔍 DRY RUN - Task 5 Analysis:');
      logger.info(`   Status: ${task5.status}`);
      logger.info(`   Title: ${task5.title}`);
      logger.info(`   Component: temporal`);
      logger.info(`   Phase: ${task5.phase}`);
      logger.info(`   Requirements: ${task5.requirements.join(', ')}`);
      
      if (task5.status === 'completed') {
        logger.info('✅ Task 5 is completed - would close GitHub issue');
        logger.info('🏷️  Would add labels: status:completed, component:temporal');
        logger.info('💬 Would add completion comment with timestamp');
      } else {
        logger.info(`🔄 Task 5 status: ${task5.status} - would update GitHub issue`);
      }
      
      logger.info('💡 To perform actual sync, set GITHUB_TOKEN environment variable');
      return;
    }
    
    // Initialize GitHub client for actual sync
    const githubClient = new GitHubClient({
      token: process.env.GITHUB_TOKEN!,
      repository: process.env.GITHUB_REPOSITORY || 'hscale/adx-core-k'
    });
    
    // Find existing issue for Task 5
    const issues = await githubClient.searchIssues('label:adx-core:task-5');
    let issue = issues.length > 0 ? issues[0] : null;
    
    if (!issue) {
      logger.info('📝 Creating new GitHub issue for Task 5');
      issue = await githubClient.createIssue({
        title: `Task 5: ${task5.title}`,
        body: `## ${task5.title}\n\n${task5.description}\n\n**Phase:** ${task5.phase}\n**Requirements:** ${task5.requirements.join(', ')}\n\n**Status:** ${task5.status}`,
        labels: [
          'adx-core:task-5',
          'component:temporal',
          'phase:2',
          `status:${task5.status}`
        ]
      });
    }
    
    // Update issue based on task status
    if (task5.status === 'completed' && issue.state === 'open') {
      logger.info('✅ Closing completed Task 5 issue');
      await githubClient.closeIssue(issue.number);
      await githubClient.addComment(issue.number, 
        `✅ **Task Completed**\n\nTask 5: Temporal SDK Integration has been completed.\n\n**Completion Date:** ${new Date().toISOString()}\n\n**Key Achievements:**\n- Replaced placeholder Temporal client with actual Temporal Rust SDK\n- Updated Cargo.toml dependencies to include real Temporal SDK\n- Implemented proper Temporal client connection and configuration\n- Created Temporal worker registration and task queue management\n- Added Temporal workflow and activity registration system\n- Tested Temporal connectivity and basic workflow execution`
      );
      
      // Update labels
      await githubClient.updateIssueLabels(issue.number, [
        'adx-core:task-5',
        'component:temporal',
        'phase:2',
        'status:completed'
      ]);
    }
    
    logger.info('✅ Task 5 sync completed successfully');
    
  } catch (error) {
    logger.error('❌ Failed to sync Task 5:', {
      error: error instanceof Error ? error.message : String(error),
      stack: error instanceof Error ? error.stack : undefined
    });
    process.exit(1);
  }
}

// Run the sync
syncTask5Completion();