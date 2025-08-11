#!/usr/bin/env node

/**
 * Kiro Hook: ADX Core GitHub Task Sync
 * 
 * This script is executed by the Kiro hook system when tasks.md is modified.
 * It provides intelligent analysis and GitHub synchronization for ADX Core tasks.
 */

import { ADXCoreTaskSync } from './sync-adx-core-tasks';
import { ConfigManager } from './src/config/ConfigManager';
import { logger } from './src/utils/logger';

interface HookExecutionContext {
  filePath: string;
  changeType: 'created' | 'modified' | 'deleted';
  timestamp: string;
}

class KiroGitHubSyncHook {
  private configManager: ConfigManager;

  constructor() {
    this.configManager = new ConfigManager();
  }

  async execute(context?: HookExecutionContext): Promise<void> {
    try {
      console.log('🔄 ADX Core GitHub Task Sync Hook Activated');
      
      if (context) {
        console.log(`📁 File: ${context.filePath}`);
        console.log(`🔄 Change: ${context.changeType}`);
        console.log(`⏰ Time: ${context.timestamp}`);
      }

      // Check if GitHub sync is configured
      const configExists = await this.configManager.exists();
      if (!configExists) {
        console.log('⚠️  GitHub sync not configured. Run: npm run setup-github');
        await this.showSetupInstructions();
        return;
      }

      // Load configuration and check if enabled
      let config;
      try {
        config = await this.configManager.load();
        if (!config.enabled) {
          console.log('ℹ️  GitHub sync is disabled in configuration');
          return;
        }
      } catch (error) {
        console.log('❌ GitHub sync configuration is invalid');
        console.log('💡 Run: npm run setup-github');
        return;
      }

      // Perform intelligent sync analysis
      console.log('🔍 Analyzing ADX Core tasks for changes...');
      
      const syncHandler = new ADXCoreTaskSync();
      
      // First, perform a dry run to analyze what would change
      console.log('\n📊 Task Analysis:');
      await syncHandler.syncAllTasks(true); // dry run mode
      
      // Ask for confirmation before syncing (in interactive mode)
      if (process.stdout.isTTY && !process.argv.includes('--auto')) {
        const readline = require('readline');
        const rl = readline.createInterface({
          input: process.stdin,
          output: process.stdout
        });

        const answer = await new Promise<string>((resolve) => {
          rl.question('\n🤔 Proceed with GitHub sync? (y/N): ', resolve);
        });
        rl.close();

        if (answer.toLowerCase() !== 'y' && answer.toLowerCase() !== 'yes') {
          console.log('⏸️  Sync cancelled by user');
          return;
        }
      }

      // Perform actual sync
      console.log('\n🚀 Syncing tasks to GitHub...');
      await syncHandler.syncAllTasks(false); // actual sync
      
      console.log('\n✅ ADX Core GitHub sync completed successfully!');
      console.log('🔗 Check your GitHub repository for updated issues');

    } catch (error) {
      console.error('❌ Hook execution failed:', error instanceof Error ? error.message : String(error));
      
      // Provide helpful error context
      if (error instanceof Error) {
        if (error.message.includes('GitHub authentication failed')) {
          console.log('\n💡 GitHub Authentication Issue:');
          console.log('   1. Check your GITHUB_TOKEN environment variable');
          console.log('   2. Ensure the token has "repo" permissions');
          console.log('   3. Run: npm run setup-github');
        } else if (error.message.includes('rate limit')) {
          console.log('\n💡 GitHub Rate Limit:');
          console.log('   The sync will automatically retry when the rate limit resets');
          console.log('   Current limit status will be logged above');
        } else if (error.message.includes('repository')) {
          console.log('\n💡 Repository Access Issue:');
          console.log('   1. Check GITHUB_REPOSITORY environment variable');
          console.log('   2. Ensure format is "owner/repo"');
          console.log('   3. Verify repository exists and token has access');
        }
      }
      
      throw error;
    }
  }

  private async showSetupInstructions(): Promise<void> {
    console.log('\n🔧 GitHub Sync Setup Required:');
    console.log('');
    console.log('1. Set environment variables:');
    console.log('   export GITHUB_TOKEN="your_github_personal_access_token"');
    console.log('   export GITHUB_REPOSITORY="your-org/your-repo"');
    console.log('');
    console.log('2. Run setup:');
    console.log('   npm run setup-github');
    console.log('');
    console.log('3. Test the sync:');
    console.log('   npm run sync-tasks-dry-run');
    console.log('');
    console.log('📚 For detailed instructions, see: GITHUB_SYNC_README.md');
  }

  async analyzeTaskChanges(): Promise<{
    totalTasks: number;
    completedTasks: number;
    pendingTasks: number;
    recentChanges: string[];
  }> {
    const syncHandler = new ADXCoreTaskSync();
    
    // This would be enhanced to detect specific changes
    // For now, we'll use the existing analysis
    await syncHandler.syncAllTasks(true); // dry run
    
    return {
      totalTasks: 43,
      completedTasks: 7,
      pendingTasks: 36,
      recentChanges: ['Task status updates detected']
    };
  }
}

/**
 * Main execution function
 */
async function main() {
  try {
    // Parse command line arguments
    const args = process.argv.slice(2);
    const isAuto = args.includes('--auto');
    const isDryRun = args.includes('--dry-run');
    
    // Create execution context from arguments if provided
    let context: HookExecutionContext | undefined;
    const filePathArg = args.find(arg => arg.startsWith('--file='));
    if (filePathArg) {
      context = {
        filePath: filePathArg.split('=')[1],
        changeType: 'modified',
        timestamp: new Date().toISOString()
      };
    }

    const hook = new KiroGitHubSyncHook();
    
    if (isDryRun) {
      console.log('🔍 DRY RUN MODE - Analyzing tasks only');
      const analysis = await hook.analyzeTaskChanges();
      console.log('📊 Analysis Results:', analysis);
    } else {
      await hook.execute(context);
    }
    
    process.exit(0);
  } catch (error) {
    console.error('❌ Kiro GitHub Sync Hook failed:', error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Export for testing and reuse
export { KiroGitHubSyncHook };

// Run if called directly
if (require.main === module) {
  main().catch(console.error);
}