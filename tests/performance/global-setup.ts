import { chromium, FullConfig } from '@playwright/test';
import fs from 'fs';
import path from 'path';

async function globalSetup(config: FullConfig) {
  console.log('Setting up performance testing environment...');

  // Create test results directory
  const testResultsDir = path.join(process.cwd(), 'test-results', 'performance');
  if (!fs.existsSync(testResultsDir)) {
    fs.mkdirSync(testResultsDir, { recursive: true });
  }

  // Start a browser instance for baseline measurements
  const browser = await chromium.launch();
  const context = await browser.newContext();
  const page = await context.newPage();

  // Measure baseline performance
  console.log('Measuring baseline performance...');
  const baselineStart = Date.now();
  
  try {
    await page.goto(config.projects[0].use?.baseURL || 'http://localhost:3000');
    await page.waitForSelector('[data-testid="app-shell"]', { timeout: 30000 });
    
    const baselineEnd = Date.now();
    const baselineLoadTime = baselineEnd - baselineStart;
    
    // Get performance metrics
    const performanceMetrics = await page.evaluate(() => {
      const navigation = performance.getEntriesByType('navigation')[0] as PerformanceNavigationTiming;
      return {
        domContentLoaded: navigation.domContentLoadedEventEnd - navigation.domContentLoadedEventStart,
        loadComplete: navigation.loadEventEnd - navigation.loadEventStart,
        firstPaint: performance.getEntriesByName('first-paint')[0]?.startTime || 0,
        firstContentfulPaint: performance.getEntriesByName('first-contentful-paint')[0]?.startTime || 0,
      };
    });

    const baseline = {
      timestamp: new Date().toISOString(),
      loadTime: baselineLoadTime,
      metrics: performanceMetrics,
      platform: process.platform,
      arch: process.arch,
      nodeVersion: process.version,
    };

    fs.writeFileSync(
      path.join(testResultsDir, 'baseline-performance.json'),
      JSON.stringify(baseline, null, 2)
    );

    console.log(`Baseline performance recorded: ${baselineLoadTime}ms load time`);
  } catch (error) {
    console.warn('Failed to measure baseline performance:', error);
  }

  await page.close();
  await context.close();
  await browser.close();

  console.log('Performance testing environment setup complete');
}

export default globalSetup;