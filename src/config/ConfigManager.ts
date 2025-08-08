import { promises as fs } from 'fs';
import { join } from 'path';
import { GitHubConfig, GitHubConfigSchema, SetupOptions, SetupOptionsSchema } from './types';
import { logger } from '../utils/logger';

export class ConfigManager {
  private readonly configPath: string;
  private readonly configDir: string;

  constructor() {
    this.configDir = join(process.cwd(), '.kiro', 'settings');
    this.configPath = join(this.configDir, 'github.json');
  }

  /**
   * Setup GitHub sync configuration
   */
  async setup(options: SetupOptions): Promise<void> {
    const validatedOptions = SetupOptionsSchema.parse(options);
    
    // Load existing config or create default
    let config: Partial<GitHubConfig>;
    try {
      config = await this.loadRaw();
    } catch {
      config = {
        enabled: true,
        labelPrefix: 'kiro:',
        syncOnSave: true,
        apiUrl: 'https://api.github.com',
        maxRetries: 3,
        retryDelay: 1000,
        rateLimitBuffer: 100,
      };
    }

    // Update config with provided options
    if (validatedOptions.token) {
      config.token = validatedOptions.token;
    }
    
    if (validatedOptions.repo) {
      config.repository = validatedOptions.repo;
    }
    
    if (validatedOptions.labelPrefix !== undefined) {
      config.labelPrefix = validatedOptions.labelPrefix;
    }
    
    if (validatedOptions.syncOnSave !== undefined) {
      config.syncOnSave = validatedOptions.syncOnSave;
    }

    // Prompt for missing required fields if not provided
    if (!config.token) {
      throw new Error('GitHub token is required. Provide it with --token option or set GITHUB_TOKEN environment variable.');
    }
    
    if (!config.repository) {
      throw new Error('GitHub repository is required. Provide it with --repo option in format "owner/repo".');
    }

    // Validate the complete configuration
    const validatedConfig = GitHubConfigSchema.parse(config);
    
    // Save configuration
    await this.save(validatedConfig);
    
    logger.info('Configuration saved successfully');
  }

  /**
   * Load and validate configuration
   */
  async load(): Promise<GitHubConfig> {
    try {
      const config = await this.loadRaw();
      
      // Allow token from environment variable if not in config
      if (!config.token && process.env['GITHUB_TOKEN']) {
        config.token = process.env['GITHUB_TOKEN'];
      }
      
      return GitHubConfigSchema.parse(config);
    } catch (error) {
      if (error instanceof Error && 'code' in error && error.code === 'ENOENT') {
        throw new Error('GitHub sync not configured. Run "kiro github setup" first.');
      }
      throw error;
    }
  }

  /**
   * Load raw configuration without validation
   */
  private async loadRaw(): Promise<Partial<GitHubConfig>> {
    const configData = await fs.readFile(this.configPath, 'utf-8');
    return JSON.parse(configData);
  }

  /**
   * Save configuration to file
   */
  async save(config: GitHubConfig): Promise<void> {
    // Ensure config directory exists
    await fs.mkdir(this.configDir, { recursive: true });
    
    // Save configuration (excluding sensitive token from file if it's from env)
    const configToSave = { ...config };
    if (process.env['GITHUB_TOKEN'] && config.token === process.env['GITHUB_TOKEN']) {
      delete (configToSave as Partial<GitHubConfig>).token;
    }
    
    await fs.writeFile(
      this.configPath,
      JSON.stringify(configToSave, null, 2),
      'utf-8'
    );
  }

  /**
   * Enable or disable sync
   */
  async setEnabled(enabled: boolean): Promise<void> {
    const config = await this.load();
    config.enabled = enabled;
    await this.save(config);
  }

  /**
   * Check if configuration exists
   */
  async exists(): Promise<boolean> {
    try {
      await fs.access(this.configPath);
      return true;
    } catch {
      return false;
    }
  }

  /**
   * Get configuration file path
   */
  getConfigPath(): string {
    return this.configPath;
  }
}