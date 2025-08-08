#!/usr/bin/env node

import { Command } from 'commander';
import { config } from 'dotenv';
import { GitHubSyncService } from './services/GitHubSyncService';
import { ConfigManager } from './config/ConfigManager';
import { logger } from './utils/logger';

// Load environment variables
config();

const program = new Command();

program
  .name('kiro-github-sync')
  .description('Sync Kiro tasks to GitHub issues')
  .version('1.0.0');

program
  .command('setup')
  .description('Setup GitHub sync configuration')
  .option('-t, --token <token>', 'GitHub personal access token')
  .option('-r, --repo <repo>', 'GitHub repository (owner/repo)')
  .action(async (options) => {
    try {
      const configManager = new ConfigManager();
      await configManager.setup(options);
      logger.info('GitHub sync configuration completed successfully');
    } catch (error) {
      logger.error('Setup failed:', error);
      process.exit(1);
    }
  });

program
  .command('sync')
  .description('Manually trigger sync of all tasks')
  .action(async () => {
    try {
      const configManager = new ConfigManager();
      const config = await configManager.load();
      
      if (!config.enabled) {
        logger.warn('GitHub sync is disabled. Enable it first with the setup command.');
        return;
      }
      
      const syncService = new GitHubSyncService(config);
      await syncService.syncAllTasks();
      logger.info('Manual sync completed successfully');
    } catch (error) {
      logger.error('Sync failed:', error);
      process.exit(1);
    }
  });

program
  .command('status')
  .description('Show sync status and configuration')
  .action(async () => {
    try {
      const configManager = new ConfigManager();
      const config = await configManager.load();
      
      console.log('GitHub Sync Status:');
      console.log(`  Enabled: ${config.enabled}`);
      console.log(`  Repository: ${config.repository}`);
      console.log(`  Label Prefix: ${config.labelPrefix}`);
      console.log(`  Sync on Save: ${config.syncOnSave}`);
      
      if (config.enabled) {
        const syncService = new GitHubSyncService(config);
        const status = await syncService.getStatus();
        console.log(`  Last Sync: ${status.lastSync || 'Never'}`);
        console.log(`  Synced Tasks: ${status.syncedTasks}`);
      }
    } catch (error) {
      logger.error('Failed to get status:', error);
      process.exit(1);
    }
  });

program
  .command('enable')
  .description('Enable GitHub sync')
  .action(async () => {
    try {
      const configManager = new ConfigManager();
      await configManager.setEnabled(true);
      logger.info('GitHub sync enabled');
    } catch (error) {
      logger.error('Failed to enable sync:', error);
      process.exit(1);
    }
  });

program
  .command('disable')
  .description('Disable GitHub sync')
  .action(async () => {
    try {
      const configManager = new ConfigManager();
      await configManager.setEnabled(false);
      logger.info('GitHub sync disabled');
    } catch (error) {
      logger.error('Failed to disable sync:', error);
      process.exit(1);
    }
  });

program
  .command('watch')
  .description('Start file watcher for automatic sync')
  .action(async () => {
    try {
      const configManager = new ConfigManager();
      const config = await configManager.load();
      
      if (!config.enabled) {
        logger.warn('GitHub sync is disabled. Enable it first with the setup command.');
        return;
      }
      
      const syncService = new GitHubSyncService(config);
      await syncService.startWatcher();
      
      // Keep the process running
      process.on('SIGINT', async () => {
        logger.info('Stopping file watcher...');
        await syncService.stopWatcher();
        process.exit(0);
      });
      
      logger.info('File watcher started. Press Ctrl+C to stop.');
    } catch (error) {
      logger.error('Failed to start watcher:', error);
      process.exit(1);
    }
  });

program.parse();