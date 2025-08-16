import { FullConfig } from '@playwright/test';
import fs from 'fs';
import path from 'path';

async function globalTeardown(config: FullConfig) {
  console.log('Tearing down desktop testing environment...');

  // Generate test summary
  const testResultsDir = path.join(process.cwd(), 'test-results', 'desktop');
  
  if (fs.existsSync(testResultsDir)) {
    const files = fs.readdirSync(testResultsDir);
    const summary = {
      timestamp: new Date().toISOString(),
      totalFiles: files.length,
      files: files.filter(f => f.endsWith('.json') || f.endsWith('.xml')),
      screenshots: files.filter(f => f.endsWith('.png')),
      videos: files.filter(f => f.endsWith('.webm')),
    };

    fs.writeFileSync(
      path.join(testResultsDir, 'test-summary.json'),
      JSON.stringify(summary, null, 2)
    );

    console.log(`Desktop test summary generated: ${summary.totalFiles} files`);
  }

  // Clean up any remaining processes
  try {
    // Kill any remaining Tauri processes
    if (process.platform === 'win32') {
      require('child_process').execSync('taskkill /f /im adx-core.exe', { stdio: 'ignore' });
    } else {
      require('child_process').execSync('pkill -f adx-core', { stdio: 'ignore' });
    }
  } catch (error) {
    // Ignore errors - processes might not be running
  }

  console.log('Desktop testing environment teardown complete');
}

export default globalTeardown;