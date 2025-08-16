import { chromium, FullConfig } from '@playwright/test';
import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

async function globalSetup(config: FullConfig) {
  console.log('Setting up desktop testing environment...');

  // Create test results directory
  const testResultsDir = path.join(process.cwd(), 'test-results', 'desktop');
  if (!fs.existsSync(testResultsDir)) {
    fs.mkdirSync(testResultsDir, { recursive: true });
  }

  // Check if Tauri is available
  try {
    execSync('cargo tauri --version', { stdio: 'pipe' });
    console.log('Tauri CLI is available');
  } catch (error) {
    console.warn('Tauri CLI not found. Desktop app tests may fail.');
  }

  // Check if desktop app is built
  const desktopAppPaths = [
    'src-tauri/target/debug/adx-core',
    'src-tauri/target/debug/adx-core.exe',
    'src-tauri/target/debug/bundle/macos/ADX Core.app',
  ];

  let desktopAppFound = false;
  for (const appPath of desktopAppPaths) {
    if (fs.existsSync(appPath)) {
      desktopAppFound = true;
      console.log(`Desktop app found at: ${appPath}`);
      break;
    }
  }

  if (!desktopAppFound) {
    console.warn('Desktop app not found. Building debug version...');
    try {
      execSync('cargo tauri build --debug', { stdio: 'inherit' });
      console.log('Desktop app built successfully');
    } catch (error) {
      console.error('Failed to build desktop app:', error);
    }
  }

  // Start a browser instance for shared context
  const browser = await chromium.launch();
  const context = await browser.newContext();
  
  // Pre-warm the application
  const page = await context.newPage();
  try {
    await page.goto(config.projects[0].use?.baseURL || 'http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]', { timeout: 30000 });
    console.log('Application pre-warmed successfully');
  } catch (error) {
    console.warn('Failed to pre-warm application:', error);
  }
  
  await page.close();
  await context.close();
  await browser.close();

  // Store test environment info
  const envInfo = {
    platform: process.platform,
    arch: process.arch,
    nodeVersion: process.version,
    timestamp: new Date().toISOString(),
    tauriAvailable: desktopAppFound,
  };

  fs.writeFileSync(
    path.join(testResultsDir, 'environment-info.json'),
    JSON.stringify(envInfo, null, 2)
  );

  console.log('Desktop testing environment setup complete');
}

export default globalSetup;