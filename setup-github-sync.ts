#!/usr/bin/env node

import { ConfigManager } from './src/config/ConfigManager';
import { logger } from './src/utils/logger';

/**
 * Setup script for GitHub sync configuration
 */
async function setupGitHubSync() {
  try {
    console.log('🔧 Setting up GitHub sync for ADX Core tasks...');
    
    const configManager = new ConfigManager();
    
    // Check if config already exists
    const configExists = await configManager.exists();
    if (configExists) {
      console.log('ℹ️  GitHub sync configuration already exists');
      
      try {
        const config = await configManager.load();
        console.log(`✅ Current configuration:`);
        console.log(`   Repository: ${config.repository}`);
        console.log(`   Label Prefix: ${config.labelPrefix}`);
        console.log(`   Sync Enabled: ${config.enabled}`);
        console.log(`   API URL: ${config.apiUrl}`);
        return;
      } catch (error) {
        console.log('⚠️  Configuration exists but is invalid, will recreate...');
      }
    }
    
    // Get configuration from environment variables or prompt
    const token = process.env.GITHUB_TOKEN;
    const repository = process.env.GITHUB_REPOSITORY || 'your-org/your-repo';
    
    if (!token) {
      console.error('❌ GITHUB_TOKEN environment variable is required');
      console.log('   Please set GITHUB_TOKEN with a GitHub personal access token');
      console.log('   The token needs "repo" permissions for the target repository');
      process.exit(1);
    }
    
    if (repository === 'your-org/your-repo') {
      console.error('❌ GITHUB_REPOSITORY environment variable is required');
      console.log('   Please set GITHUB_REPOSITORY in format "owner/repo"');
      console.log('   Example: GITHUB_REPOSITORY="myorg/adx-core"');
      process.exit(1);
    }
    
    // Setup configuration
    await configManager.setup({
      token,
      repo: repository,
      labelPrefix: 'adx-core:',
      syncOnSave: true,
    });
    
    console.log('✅ GitHub sync configuration created successfully!');
    console.log(`   Repository: ${repository}`);
    console.log(`   Label Prefix: adx-core:`);
    console.log(`   Sync Enabled: true`);
    
    // Test the connection
    console.log('🔍 Testing GitHub connection...');
    const { GitHubClient } = await import('./src/services/GitHubClient');
    const config = await configManager.load();
    const githubClient = new GitHubClient(config);
    
    const connectionTest = await githubClient.testConnection();
    if (connectionTest.success) {
      console.log('✅ GitHub connection successful!');
      console.log(`   ${connectionTest.message}`);
    } else {
      console.error('❌ GitHub connection failed:');
      console.error(`   ${connectionTest.message}`);
      process.exit(1);
    }
    
    console.log('\n🚀 Setup complete! You can now run:');
    console.log('   npm run sync-tasks    # Sync all tasks');
    console.log('   node sync-adx-core-tasks.ts    # Direct execution');
    
  } catch (error) {
    console.error('❌ Setup failed:', error instanceof Error ? error.message : String(error));
    process.exit(1);
  }
}

// Run setup
setupGitHubSync().catch(console.error);