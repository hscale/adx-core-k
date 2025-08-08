import { describe, it, expect, beforeEach, afterEach } from 'vitest';
import { promises as fs } from 'fs';
import { join } from 'path';
import { ConfigManager } from './ConfigManager';

describe('ConfigManager', () => {
  let configManager: ConfigManager;
  let testConfigDir: string;

  beforeEach(() => {
    configManager = new ConfigManager();
    testConfigDir = join(process.cwd(), '.kiro', 'settings');
  });

  afterEach(async () => {
    // Clean up test files
    try {
      await fs.unlink(join(testConfigDir, 'github.json'));
    } catch {
      // Ignore if file doesn't exist
    }
  });

  it('should create config manager instance', () => {
    expect(configManager).toBeInstanceOf(ConfigManager);
  });

  it('should return correct config path', () => {
    const configPath = configManager.getConfigPath();
    expect(configPath).toContain('.kiro/settings/github.json');
  });

  it('should check if config exists', async () => {
    const exists = await configManager.exists();
    // Config might or might not exist depending on test environment
    expect(typeof exists).toBe('boolean');
  });

  it('should throw error when loading non-existent config', async () => {
    // Ensure config doesn't exist
    try {
      await fs.unlink(configManager.getConfigPath());
    } catch {
      // Ignore if already doesn't exist
    }

    await expect(configManager.load()).rejects.toThrow('GitHub sync not configured');
  });
});